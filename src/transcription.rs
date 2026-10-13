use std::{
    error::Error,
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::media::DecodedAudio;

const HELPER: &str = include_str!("../scripts/mlx_whisper_helper.py");

pub(crate) struct Options {
    pub(crate) python: PathBuf,
    pub(crate) model: String,
}

#[derive(Serialize)]
struct HelperRequest<'a> {
    audio_path: &'a Path,
    language: &'a str,
    model: &'a str,
}

#[derive(Deserialize, Serialize)]
struct Backend {
    name: String,
    version: String,
    model: String,
}

#[derive(Deserialize, Serialize)]
struct Settings {
    language: String,
    task: String,
    word_timestamps: bool,
    temperature: f64,
    condition_on_previous_text: bool,
    initial_prompt: Option<String>,
}

#[derive(Deserialize)]
struct HelperResponse {
    backend: Backend,
    settings: Settings,
    transcript: Transcript,
}

#[derive(Deserialize, Serialize)]
struct Transcript {
    text: String,
    language: String,
    segments: Vec<Segment>,
}

#[derive(Deserialize, Serialize)]
struct Segment {
    id: u64,
    start: f64,
    end: f64,
    text: String,
    words: Vec<Word>,
    temperature: f64,
    avg_logprob: f64,
    compression_ratio: f64,
    no_speech_prob: f64,
}

#[derive(Deserialize, Serialize)]
struct Word {
    start: f64,
    end: f64,
    word: String,
    probability: f64,
}

#[derive(Serialize)]
struct Document<'a> {
    source: &'a crate::media::DecodedInfo,
    backend: Backend,
    settings: Settings,
    timestamp_reference: &'static str,
    source_offset_seconds: f64,
    #[serde(flatten)]
    transcript: Transcript,
}

pub(crate) fn publish(
    decoded: &DecodedAudio,
    language: &str,
    options: &Options,
    output: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut sidecar_name = OsString::from(output.as_os_str());
    sidecar_name.push(".json");
    let sidecar = PathBuf::from(sidecar_name);
    if output.try_exists()? || sidecar.try_exists()? {
        return Err("transcript or metadata output already exists; nothing was replaced".into());
    }

    let wav = selected_wav(decoded)?;
    let request = HelperRequest {
        audio_path: wav.path(),
        language,
        model: &options.model,
    };
    let mut child = Command::new(&options.python)
        .args(["-c", HELPER])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!(
                "cannot start Whisper Python interpreter {}: {error}; use --whisper-python to select it",
                options.python.display()
            )
        })?;
    let input_result = serde_json::to_writer(child.stdin.take().unwrap(), &request);
    let result = child.wait_with_output()?;
    if !result.status.success() {
        return Err(format!(
            "mlx-whisper helper failed ({}): {}",
            result.status,
            String::from_utf8_lossy(&result.stderr).trim()
        )
        .into());
    }
    input_result?;
    let mut response: HelperResponse = serde_json::from_slice(&result.stdout)
        .map_err(|error| format!("invalid mlx-whisper helper response: {error}"))?;
    validate_response(&response, language, &options.model)?;

    let info = decoded.info();
    let offset = info.selected_range.start_frame as f64 / f64::from(info.sample_rate.get());
    map_timestamps(&mut response.transcript, offset)?;
    let document = Document {
        source: info,
        backend: response.backend,
        settings: response.settings,
        timestamp_reference: "original_source_seconds",
        source_offset_seconds: offset,
        transcript: response.transcript,
    };
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut text_output = NamedTempFile::new_in(parent)?;
    writeln!(text_output, "{}", document.transcript.text.trim())?;
    text_output.flush()?;
    let mut json_output = NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(json_output.as_file_mut(), &document)?;
    writeln!(json_output)?;
    json_output.flush()?;
    text_output
        .persist_noclobber(output)
        .map_err(|error| format!("cannot save transcript: {error}"))?;
    if let Err(error) = json_output.persist_noclobber(&sidecar) {
        std::fs::remove_file(output)?;
        return Err(format!("cannot save transcript metadata: {error}").into());
    }
    Ok(sidecar)
}

fn selected_wav(decoded: &DecodedAudio) -> Result<NamedTempFile, Box<dyn Error>> {
    let wav = NamedTempFile::new()?;
    let info = decoded.info();
    let mut child = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-f", "f32le", "-ar"])
        .arg(info.sample_rate.to_string())
        .arg("-ac")
        .arg(info.channels.to_string())
        .args(["-i", "pipe:0", "-c:a", "pcm_f32le", "-f", "wav", "pipe:1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::from(wav.as_file().try_clone()?))
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start ffmpeg for transcription audio: {error}"))?;
    let copied = io::copy(
        &mut decoded.open_selected_pcm()?,
        &mut child.stdin.take().unwrap(),
    );
    let result = child.wait_with_output()?;
    if !result.status.success() {
        return Err(format!(
            "cannot prepare transcription audio: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )
        .into());
    }
    copied?;
    Ok(wav)
}

fn validate_response(
    response: &HelperResponse,
    language: &str,
    model: &str,
) -> Result<(), Box<dyn Error>> {
    if response.backend.name != "mlx-whisper"
        || response.backend.version.is_empty()
        || response.backend.model != model
        || response.settings.language != language
        || response.transcript.language != language
        || response.settings.task != "transcribe"
        || !response.settings.word_timestamps
        || response.settings.temperature != 0.0
        || response.settings.condition_on_previous_text
        || response.settings.initial_prompt.is_some()
    {
        return Err("mlx-whisper helper returned unexpected model, language, or settings".into());
    }
    Ok(())
}

fn map_timestamps(transcript: &mut Transcript, offset: f64) -> Result<(), Box<dyn Error>> {
    fn map(start: &mut f64, end: &mut f64, offset: f64) -> Result<(), Box<dyn Error>> {
        if !start.is_finite() || !end.is_finite() || *start < 0.0 || *start > *end {
            return Err("mlx-whisper returned invalid timestamps".into());
        }
        *start += offset;
        *end += offset;
        Ok(())
    }
    for segment in &mut transcript.segments {
        map(&mut segment.start, &mut segment.end, offset)?;
        for word in &mut segment.words {
            map(&mut word.start, &mut word.end, offset)?;
            if !word.probability.is_finite() || !(0.0..=1.0).contains(&word.probability) {
                return Err("mlx-whisper returned an invalid word probability".into());
            }
        }
    }
    Ok(())
}
