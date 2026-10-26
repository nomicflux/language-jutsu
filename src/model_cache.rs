//! Whisper model files from the Hugging Face cache, through hf-hub.

use std::{env, error::Error, ffi::OsString, path::PathBuf};

use hf_hub::HFClient;

// The ggerganov/whisper.cpp commit the models are pinned to. With a commit hash, hf-hub
// returns a cached file without any network request.
const MODEL_REVISION: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";

/// Downloads a model file into the Hugging Face cache, or returns the cached copy.
pub(crate) fn fetch_model(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let cache = cache_dir_from(|key| env::var_os(key), env::home_dir())
        .ok_or("no home folder for the Hugging Face cache")?;
    HFClient::builder()
        .cache_dir(cache)
        .build_sync()?
        .model("ggerganov", "whisper.cpp")
        .download_file()
        .filename(name)
        .revision(MODEL_REVISION)
        .send()
        .map_err(|error| format!("cannot fetch Whisper model {name}: {error}").into())
}

/// The Hugging Face cache folder: `HF_HUB_CACHE`, else `$HF_HOME/hub`, else
/// `$XDG_CACHE_HOME/huggingface/hub`, else `~/.cache/huggingface/hub`.
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
