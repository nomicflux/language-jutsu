use std::{
    error::Error,
    ffi::OsString,
    fmt, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::{NamedTempFile, TempDir};
use url::Url;

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct YouTubeVideo {
    submitted_url: String,
    video_id: String,
}

impl FromStr for YouTubeVideo {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(value).map_err(|error| format!("invalid YouTube URL: {error}"))?;
        if !matches!(url.scheme(), "https" | "http")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
        {
            return Err(
                "use an HTTP(S) YouTube video URL without credentials or a custom port".into(),
            );
        }
        let parts: Vec<_> = url
            .path_segments()
            .ok_or("invalid video URL path")?
            .collect();
        let id = match url.host_str() {
            Some("youtu.be" | "www.youtu.be") => match parts.as_slice() {
                [id] => (*id).to_owned(),
                _ => return Err("use a single YouTube video URL".into()),
            },
            Some("youtube.com" | "www.youtube.com" | "m.youtube.com" | "music.youtube.com") => match parts.as_slice() {
                ["watch"] => {
                    let ids: Vec<_> = url.query_pairs().filter(|(key, _)| key == "v").map(|(_, value)| value.into_owned()).collect();
                    if ids.len() != 1 { return Err("use a single YouTube video URL with one v parameter".into()); }
                    ids.into_iter().next().unwrap()
                },
                ["shorts", id] => (*id).to_owned(),
                _ => return Err("use a single YouTube video URL; playlists, channels, and searches are unsupported".into()),
            },
            _ => return Err("--url currently supports youtube.com and youtu.be video links".into()),
        };
        if id.is_empty()
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err("invalid YouTube video identity".into());
        }
        Ok(Self {
            submitted_url: value.into(),
            video_id: id,
        })
    }
}

impl YouTubeVideo {
    fn canonical_url(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.video_id)
    }
}

pub(crate) enum Source {
    File(PathBuf),
    YouTube(YouTubeVideo),
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => write!(formatter, "{}", path.display()),
            Self::YouTube(video) => formatter.write_str(&video.submitted_url),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(crate) enum Origin {
    File {
        path: PathBuf,
    },
    YouTube {
        video: YouTubeVideo,
        webpage_url: String,
        metadata: Value,
    },
}

pub(crate) enum AcquiredSource {
    File {
        path: PathBuf,
        origin: Origin,
    },
    YouTube {
        path: PathBuf,
        origin: Origin,
        _directory: TempDir,
    },
}

impl AcquiredSource {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::File { path, .. } | Self::YouTube { path, .. } => path,
        }
    }
    pub(crate) fn origin(&self) -> Origin {
        match self {
            Self::File { origin, .. } | Self::YouTube { origin, .. } => origin.clone(),
        }
    }

    pub(crate) fn retain_download(&mut self, output: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let Self::YouTube { path, origin, .. } = self else {
            return Err("--save-source requires a downloaded --url source".into());
        };
        let sidecar = provenance_path(output);
        if output.try_exists()? || sidecar.try_exists()? {
            return Err(
                "saved source or provenance output already exists; nothing was replaced".into(),
            );
        }
        let parent = output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut audio = NamedTempFile::new_in(parent)?;
        io::copy(&mut fs::File::open(&path)?, audio.as_file_mut())?;
        audio.flush()?;
        let provenance = RetainedSource {
            schema_version: 1,
            byte_length: audio.as_file().metadata()?.len(),
            sha256: sha256(audio.path())?,
            origin: origin.clone(),
        };
        let mut metadata = NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(metadata.as_file_mut(), &provenance)?;
        writeln!(metadata)?;
        metadata.flush()?;
        audio
            .persist_noclobber(output)
            .map_err(|error| format!("cannot save downloaded source: {error}"))?;
        if let Err(error) = metadata.persist_noclobber(&sidecar) {
            fs::remove_file(output)?;
            return Err(format!("cannot save source provenance: {error}").into());
        }
        *path = output.canonicalize()?;
        Ok(sidecar)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedSource {
    schema_version: u32,
    byte_length: u64,
    sha256: String,
    origin: Origin,
}

fn provenance_path(path: &Path) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".source.json");
    PathBuf::from(name)
}

fn sha256(path: &Path) -> Result<String, io::Error> {
    let mut input = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn file_origin(path: &Path) -> Result<Origin, Box<dyn Error>> {
    let sidecar = provenance_path(path);
    if !sidecar.try_exists()? {
        return Ok(Origin::File { path: path.into() });
    }
    let saved: RetainedSource =
        serde_json::from_reader(fs::File::open(&sidecar)?).map_err(|error| {
            format!(
                "invalid saved-source provenance {}: {error}",
                sidecar.display()
            )
        })?;
    if saved.schema_version != 1 {
        return Err("unsupported saved-source provenance version".into());
    }
    if fs::metadata(path)?.len() != saved.byte_length || sha256(path)? != saved.sha256 {
        return Err(
            "saved source no longer matches its provenance (content SHA-256 mismatch)".into(),
        );
    }
    let Origin::YouTube {
        video,
        webpage_url,
        metadata,
    } = &saved.origin
    else {
        return Err("saved-source provenance must describe a YouTube download".into());
    };
    let submitted: YouTubeVideo = video.submitted_url.parse()?;
    let canonical: YouTubeVideo = webpage_url.parse()?;
    if submitted.video_id != video.video_id
        || canonical.video_id != video.video_id
        || metadata["id"].as_str() != Some(video.video_id.as_str())
    {
        return Err("saved-source provenance has inconsistent YouTube identity".into());
    }
    Ok(saved.origin)
}

fn downloader(directory: &Path) -> Command {
    let mut command = Command::new("yt-dlp");
    command
        .current_dir(directory)
        .args([
            "--ignore-config",
            "--no-playlist",
            "--format",
            "bestaudio",
            "--no-cache-dir",
            "--js-runtimes",
            "node",
        ])
        .stdin(Stdio::null());
    command
}

fn run(command: &mut Command, stage: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let result = command.output().map_err(|error| {
        format!("cannot start yt-dlp for {stage} (required for --url): {error}")
    })?;
    if !result.status.success() {
        return Err(format!(
            "yt-dlp {stage} failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )
        .into());
    }
    if !result.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&result.stderr));
    }
    Ok(result.stdout)
}

impl Source {
    pub(crate) fn is_remote(&self) -> bool {
        matches!(self, Self::YouTube(_))
    }

    pub(crate) fn validate(&self) -> Result<(), Box<dyn Error>> {
        if let Self::File(path) = self {
            let metadata = fs::metadata(path)
                .map_err(|error| format!("cannot read input {}: {error}", path.display()))?;
            if !metadata.is_file() {
                return Err(format!("input is not a regular file: {}", path.display()).into());
            }
        }
        Ok(())
    }

    pub(crate) fn acquire(self) -> Result<AcquiredSource, Box<dyn Error>> {
        let video = match self {
            Self::File(path) => {
                let path = path.canonicalize()?;
                let origin = file_origin(&path)?;
                return Ok(AcquiredSource::File { path, origin });
            }
            Self::YouTube(video) => video,
        };
        let directory = tempfile::tempdir()?;
        let mut command = downloader(directory.path());
        command
            .args(["--dump-single-json", "--"])
            .arg(video.canonical_url());
        let metadata: Value = serde_json::from_slice(&run(&mut command, "metadata query")?)
            .map_err(|error| format!("invalid yt-dlp metadata: {error}"))?;
        if metadata
            .get("_type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind != "video")
            || metadata.get("entries").is_some()
        {
            return Err("yt-dlp returned a collection; a single video is required".into());
        }
        if metadata["id"].as_str() != Some(video.video_id.as_str()) {
            return Err("yt-dlp returned a different or missing video identity".into());
        }
        if !matches!(
            metadata["live_status"].as_str(),
            Some("not_live" | "was_live")
        ) || metadata["is_live"].as_bool() == Some(true)
        {
            return Err("YouTube source must be a completed video or processed archive; live/upcoming videos are unsupported".into());
        }
        if metadata["vcodec"].as_str() != Some("none")
            || !metadata["acodec"]
                .as_str()
                .is_some_and(|codec| !codec.is_empty() && codec != "none")
        {
            return Err("yt-dlp did not select an audio-only format".into());
        }
        let webpage_url = metadata["webpage_url"]
            .as_str()
            .ok_or("yt-dlp metadata is missing webpage_url")?
            .to_owned();
        let resolved: YouTubeVideo = webpage_url.parse()?;
        if resolved.video_id != video.video_id {
            return Err("yt-dlp webpage identity differs from requested video".into());
        }
        println!(
            "YouTube video: {} ({})",
            metadata["title"].as_str().unwrap_or("Untitled"),
            video.video_id
        );
        let info_path = directory.path().join("source.info.json");
        serde_json::to_writer(fs::File::create(&info_path)?, &metadata)?;
        let mut command = downloader(directory.path());
        command.args(["--load-info-json"]).arg(&info_path).args([
            "--no-simulate",
            "--output",
            "source.%(ext)s",
            "--print",
            "after_move:filepath",
        ]);
        let stdout = run(&mut command, "audio download")?;
        let text =
            std::str::from_utf8(&stdout).map_err(|_| "yt-dlp returned a non-UTF-8 media path")?;
        let lines: Vec<_> = text.lines().collect();
        if lines.len() != 1 || lines[0].is_empty() {
            return Err("yt-dlp must return exactly one completed media path".into());
        }
        let reported = PathBuf::from(lines[0]);
        let path = if reported.is_absolute() {
            reported
        } else {
            directory.path().join(reported)
        };
        let attributes = fs::symlink_metadata(&path)
            .map_err(|error| format!("downloaded media is missing: {error}"))?;
        let entries =
            fs::read_dir(directory.path())?.collect::<Result<Vec<_>, std::io::Error>>()?;
        if entries
            .iter()
            .filter(|entry| entry.path() != info_path)
            .count()
            != 1
        {
            return Err("yt-dlp must leave exactly one downloaded media file".into());
        }
        let path = path.canonicalize()?;
        if !attributes.is_file()
            || attributes.len() == 0
            || path.parent() != Some(directory.path().canonicalize()?.as_path())
            || path == info_path.canonicalize()?
        {
            return Err(
                "downloaded media must be a nonempty regular file inside the import directory"
                    .into(),
            );
        }
        Ok(AcquiredSource::YouTube {
            path,
            origin: Origin::YouTube {
                video,
                webpage_url,
                metadata,
            },
            _directory: directory,
        })
    }
}
