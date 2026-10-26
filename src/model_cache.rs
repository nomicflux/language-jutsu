//! Whisper model files in the Hugging Face Hub cache, laid out as
//! https://huggingface.co/docs/hub/local-cache specifies.

use std::{
    env,
    error::Error,
    ffi::OsString,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use reqwest::{
    blocking::{Client, Response},
    redirect::Policy,
};
use sha2::{Digest, Sha256};

const RESOLVE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";
const REPO_FOLDER: &str = "models--ggerganov--whisper.cpp";

/// The Hub's description of a file: the commit `main` resolves to and the file's SHA-256 (its etag).
struct Remote {
    commit: String,
    etag: String,
}

/// Returns the cached model file, downloading it into the Hugging Face cache first if needed.
pub(crate) fn fetch_model(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    fetch(name).map_err(|error| format!("cannot fetch Whisper model {name}: {error}").into())
}

fn fetch(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let cache = cache_dir_from(|key| env::var_os(key), env::home_dir())
        .ok_or("no home folder for the Hugging Face cache")?;
    let repo = cache.join(REPO_FOLDER);
    if let Some(path) = cached(&repo, name)? {
        return Ok(path);
    }
    let remote = describe(name)?;
    // Held until this function returns; a waiting process then finds the snapshot below.
    let _lock = lock(&cache, &remote.etag)?;
    let snapshot = repo.join("snapshots").join(&remote.commit).join(name);
    if !snapshot.exists() {
        place(&repo, name, &remote, &snapshot)?;
    }
    fs::create_dir_all(repo.join("refs"))?;
    fs::write(repo.join("refs").join("main"), &remote.commit)?;
    Ok(snapshot)
}

/// `HF_HUB_CACHE`, else `$HF_HOME/hub`, else `$XDG_CACHE_HOME/huggingface/hub`, else `~/.cache/huggingface/hub`.
fn cache_dir_from(
    var: impl Fn(&str) -> Option<OsString>,
    home: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(cache) = var("HF_HUB_CACHE") {
        return Some(cache.into());
    }
    let hf_home = var("HF_HOME")
        .map(PathBuf::from)
        .or_else(|| var("XDG_CACHE_HOME").map(|xdg| PathBuf::from(xdg).join("huggingface")))
        .or_else(|| home.map(|home| home.join(".cache").join("huggingface")))?;
    Some(hf_home.join("hub"))
}

/// The snapshot `refs/main` points to, if that file is present; no network.
fn cached(repo: &Path, name: &str) -> io::Result<Option<PathBuf>> {
    let commit = match fs::read_to_string(repo.join("refs").join("main")) {
        Ok(commit) => commit,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let snapshot = repo.join("snapshots").join(commit.trim()).join(name);
    Ok(snapshot.exists().then_some(snapshot))
}

fn describe(name: &str) -> Result<Remote, Box<dyn Error>> {
    let client = Client::builder()
        .redirect(Policy::none())
        .timeout(None::<Duration>)
        .build()?;
    let response = client.head(format!("{RESOLVE_URL}{name}")).send()?;
    let status = response.status();
    if !status.is_success() && !status.is_redirection() {
        return Err(format!("the Hub answered {status}").into());
    }
    Ok(Remote {
        commit: header(&response, "x-repo-commit")?,
        etag: header(&response, "x-linked-etag")?
            .trim_matches('"')
            .to_owned(),
    })
}

fn header(response: &Response, name: &str) -> Result<String, String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .ok_or_else(|| format!("the Hub sent no {name} header"))
}

/// The Hub's per-blob lock; blocks until no other process holds it. Released on drop or exit.
fn lock(cache: &Path, etag: &str) -> io::Result<File> {
    let dir = cache.join(".locks").join(REPO_FOLDER);
    fs::create_dir_all(&dir)?;
    let file = File::create(dir.join(format!("{etag}.lock")))?;
    file.lock()?;
    Ok(file)
}

#[cfg(unix)]
fn place(repo: &Path, name: &str, remote: &Remote, snapshot: &Path) -> Result<(), Box<dyn Error>> {
    let blob = repo.join("blobs").join(&remote.etag);
    if !blob.exists() {
        download(repo, name, &remote.etag, &blob)?;
    }
    fs::create_dir_all(snapshot.parent().unwrap())?;
    // A link left dangling by a deleted blob is replaced.
    match fs::remove_file(snapshot) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error.into()),
        _ => {}
    }
    std::os::unix::fs::symlink(Path::new("../../blobs").join(&remote.etag), snapshot)?;
    Ok(())
}

/// Windows has no unprivileged symlinks; the Hub's no-symlink layout keeps the only copy in the snapshot.
#[cfg(windows)]
fn place(repo: &Path, name: &str, remote: &Remote, snapshot: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(snapshot.parent().unwrap())?;
    download(repo, name, &remote.etag, snapshot)
}

/// Streams the file into `blobs/<etag>.incomplete` and renames it to `target` only if its SHA-256 is `etag`.
fn download(repo: &Path, name: &str, etag: &str, target: &Path) -> Result<(), Box<dyn Error>> {
    let partial = repo.join("blobs").join(format!("{etag}.incomplete"));
    fs::create_dir_all(partial.parent().unwrap())?;
    let response = Client::builder()
        .timeout(None::<Duration>)
        .build()?
        .get(format!("{RESOLVE_URL}{name}"))
        .send()?
        .error_for_status()?;
    let digest = save(response, &partial)?;
    if digest != etag {
        return Err(format!("downloaded {name} has SHA-256 {digest}, not the Hub's {etag}").into());
    }
    fs::rename(&partial, target)?;
    Ok(())
}

/// Writes `body` to a new file at `path` and returns its SHA-256; the file is closed on return.
fn save(mut body: impl Read, path: &Path) -> io::Result<String> {
    let mut output = File::create(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        let count = body.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
        output.write_all(&buffer[..count])?;
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(vars: &[(&str, &str)], home: Option<&str>) -> Option<PathBuf> {
        let lookup = |key: &str| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| OsString::from(value))
        };
        cache_dir_from(lookup, home.map(PathBuf::from))
    }

    #[test]
    fn cache_folder_follows_the_hub_order() {
        let all = [
            ("HF_HUB_CACHE", "/hub-cache"),
            ("HF_HOME", "/hf-home"),
            ("XDG_CACHE_HOME", "/xdg"),
        ];
        let home = Some("/home/u");
        let expected = |path: &str| Some(PathBuf::from(path));
        assert_eq!(resolve(&all, home), expected("/hub-cache"));
        assert_eq!(resolve(&all[1..], home), expected("/hf-home/hub"));
        assert_eq!(resolve(&all[2..], home), expected("/xdg/huggingface/hub"));
        assert_eq!(
            resolve(&[], home),
            expected("/home/u/.cache/huggingface/hub")
        );
        assert_eq!(resolve(&[], None), None);
    }
}
