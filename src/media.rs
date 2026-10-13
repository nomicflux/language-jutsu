use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsString,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Take, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::NamedTempFile;

use crate::cli::LanguageSetting;
use crate::source::{AcquiredSource, Origin};
use crate::window::{self, FrameRange};

#[derive(Deserialize)]
struct Probe {
    streams: Vec<SourceStream>,
}

#[derive(Deserialize, Serialize)]
struct SourceStream {
    index: u32,
    sample_rate: String,
    channels: NonZeroU32,
    channel_layout: Option<String>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(Serialize)]
pub(crate) struct DecodedInfo {
    source_path: PathBuf,
    origin: Origin,
    language: LanguageSetting,
    source_stream: SourceStream,
    sample_format: &'static str,
    pub(crate) sample_rate: NonZeroU32,
    pub(crate) channels: NonZeroU32,
    pub(crate) frame_count: u64,
    requested_window: window::Request,
    pub(crate) selected_range: FrameRange,
    original_duration_seconds: f64,
    selected_duration_seconds: f64,
    selected_start_seconds: f64,
    selected_end_seconds: f64,
}

impl DecodedInfo {
    pub(crate) fn duration_seconds(&self) -> f64 {
        self.frame_count as f64 / f64::from(self.sample_rate.get())
    }

    pub(crate) fn selected_frame_count(&self) -> u64 {
        self.selected_range.end_frame - self.selected_range.start_frame
    }

    pub(crate) fn channel_layout(&self) -> Option<&str> {
        self.source_stream.channel_layout.as_deref()
    }

    fn document(&self) -> Result<Value, serde_json::Error> {
        let mut document = serde_json::to_value(self)?;
        document["pcm_frame_count"] = self.selected_frame_count().into();
        Ok(document)
    }
}

pub(crate) struct DecodedAudio {
    pcm: NamedTempFile,
    info: DecodedInfo,
}

pub(crate) fn decode(
    input: &AcquiredSource,
    language: LanguageSetting,
    window: window::Request,
) -> Result<DecodedAudio, Box<dyn Error>> {
    let source_path = input.path().canonicalize()?;
    let probe_output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(&source_path)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("cannot start ffprobe (required for audio decoding): {error}"))?;
    if !probe_output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&probe_output.stderr).trim()
        )
        .into());
    }
    let probe: Probe = serde_json::from_slice(&probe_output.stdout)
        .map_err(|error| format!("invalid ffprobe metadata: {error}"))?;
    let stream = probe
        .streams
        .into_iter()
        .next()
        .ok_or("input has no audio stream")?;
    let sample_rate = stream
        .sample_rate
        .parse::<NonZeroU32>()
        .map_err(|error| format!("invalid audio sample rate: {error}"))?;
    let channels = stream.channels;
    let pcm = NamedTempFile::new()?;

    let result = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-xerror", "-i"])
        .arg(&source_path)
        .arg("-map")
        .arg(format!("0:{}", stream.index))
        .args(["-vn", "-sn", "-dn", "-c:a", "pcm_f32le", "-ar"])
        .arg(sample_rate.to_string())
        .arg("-ac")
        .arg(channels.to_string())
        .args(["-f", "f32le", "pipe:1"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(pcm.as_file().try_clone()?))
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("cannot start ffmpeg (required for audio decoding): {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "ffmpeg decoding failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )
        .into());
    }

    let bytes = pcm.as_file().metadata()?.len();
    let frame_bytes = u64::from(channels.get()) * 4;
    if bytes % frame_bytes != 0 {
        return Err("decoder produced an incomplete PCM frame".into());
    }
    let frame_count = bytes / frame_bytes;
    if frame_count == 0 {
        return Err("decoded audio is empty".into());
    }
    let selected_range = window.resolve(sample_rate.get(), frame_count)?;

    Ok(DecodedAudio {
        pcm,
        info: DecodedInfo {
            source_path,
            origin: input.origin(),
            language,
            source_stream: stream,
            sample_format: "f32le",
            sample_rate,
            channels,
            frame_count,
            requested_window: window,
            original_duration_seconds: frame_count as f64 / f64::from(sample_rate.get()),
            selected_duration_seconds: (selected_range.end_frame - selected_range.start_frame)
                as f64
                / f64::from(sample_rate.get()),
            selected_start_seconds: selected_range.start_frame as f64
                / f64::from(sample_rate.get()),
            selected_end_seconds: selected_range.end_frame as f64 / f64::from(sample_rate.get()),
            selected_range,
        },
    })
}

impl DecodedAudio {
    pub(crate) fn open_selected_pcm(&self) -> Result<Take<File>, io::Error> {
        self.open_range_pcm(&self.info.selected_range)
    }

    pub(crate) fn open_range_pcm(&self, range: &FrameRange) -> Result<Take<File>, io::Error> {
        if range.start_frame < self.info.selected_range.start_frame
            || range.end_frame > self.info.selected_range.end_frame
            || range.start_frame >= range.end_frame
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "PCM range is outside the selected window",
            ));
        }
        let frame_bytes = u64::from(self.info.channels.get()) * 4;
        let mut file = File::open(self.pcm.path())?;
        file.seek(SeekFrom::Start(range.start_frame * frame_bytes))?;
        Ok(file.take((range.end_frame - range.start_frame) * frame_bytes))
    }

    pub(crate) fn info(&self) -> &DecodedInfo {
        &self.info
    }

    pub(crate) fn publish(self, output: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let mut sidecar_name = OsString::from(output.as_os_str());
        sidecar_name.push(".json");
        let sidecar = PathBuf::from(sidecar_name);
        if output.try_exists()? || sidecar.try_exists()? {
            return Err(
                "decoded PCM or metadata output already exists; nothing was replaced".into(),
            );
        }
        let parent = output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut pcm_output = NamedTempFile::new_in(parent)?;
        io::copy(&mut self.open_selected_pcm()?, pcm_output.as_file_mut())?;
        pcm_output.flush()?;
        let mut metadata_output = NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(metadata_output.as_file_mut(), &self.info.document()?)?;
        writeln!(metadata_output)?;
        metadata_output.flush()?;

        pcm_output
            .persist_noclobber(output)
            .map_err(|error| format!("cannot save decoded PCM: {error}"))?;
        if let Err(error) = metadata_output.persist_noclobber(&sidecar) {
            std::fs::remove_file(output)?;
            return Err(format!("cannot save decoded metadata: {error}").into());
        }
        Ok(sidecar)
    }
}
