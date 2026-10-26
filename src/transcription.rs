use std::{
    error::Error,
    ffi::OsString,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use hf_hub::HFClientSync;
use serde::Serialize;
use tempfile::NamedTempFile;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperError,
    WhisperSegment, WhisperState, WhisperToken,
};

use crate::media::{DecodedAudio, DecodedInfo};

pub(crate) const DEFAULT_MODEL: &str = "ggml-large-v3-turbo.bin";
const SAMPLE_RATE: &str = "16000";
// Whisper's word splitting: these languages are split per Unicode character group.
const SPACELESS_LANGUAGES: [&str; 6] = ["zh", "ja", "th", "lo", "my", "yue"];
const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

pub(crate) struct Options {
    pub(crate) model: String,
}

#[derive(Serialize)]
struct Backend {
    name: &'static str,
    version: &'static str,
    model: String,
}

#[derive(Serialize)]
struct Settings {
    language: String,
    task: &'static str,
    word_timestamps: bool,
    temperature: f64,
    condition_on_previous_text: bool,
    initial_prompt: Option<String>,
}

#[derive(Serialize)]
struct Transcript {
    text: String,
    language: String,
    segments: Vec<Segment>,
}

#[derive(Serialize)]
struct Segment {
    id: u64,
    start: f64,
    end: f64,
    text: String,
    words: Vec<Word>,
    no_speech_prob: f64,
}

#[derive(Serialize, Debug, PartialEq)]
struct Word {
    word: String,
    start: f64,
    end: f64,
    probability: f64,
}

/// One decoded text token: its raw bytes may hold part of a UTF-8 character.
struct Piece {
    bytes: Vec<u8>,
    start: f64,
    end: f64,
    probability: f64,
}

#[derive(Serialize)]
struct Document<'a> {
    source: &'a DecodedInfo,
    backend: Backend,
    settings: Settings,
    timestamp_reference: &'static str,
    source_offset_seconds: f64,
    #[serde(flatten)]
    transcript: Transcript,
}

/// Downloads a model file into the Hugging Face cache, or returns the cached copy.
pub(crate) fn fetch_model(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    HFClientSync::new()?
        .model("ggerganov", "whisper.cpp")
        .download_file()
        .filename(name)
        .send()
        .map_err(|error| format!("cannot fetch Whisper model {name}: {error}").into())
}

pub(crate) fn publish(
    decoded: &DecodedAudio,
    language: &str,
    options: &Options,
    output: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let sidecar = sidecar_path(output);
    if output.try_exists()? || sidecar.try_exists()? {
        return Err("transcript or metadata output already exists; nothing was replaced".into());
    }
    let language_id = whisper_language(language)?;
    let context = load_model(&options.model, language, language_id)?;
    let mut transcript = transcribe(&context, &whisper_audio(decoded)?, language, language_id)?;
    let info = decoded.info();
    let offset = info.selected_range.start_frame as f64 / f64::from(info.sample_rate.get());
    map_timestamps(&mut transcript, offset)?;
    let document = document(info, &options.model, language, offset, transcript);
    write_outputs(&document, output, &sidecar)?;
    Ok(sidecar)
}

fn whisper_language(language: &str) -> Result<i32, String> {
    whisper_rs::get_lang_id(language)
        .ok_or_else(|| format!("Whisper does not support language code {language}"))
}

fn sidecar_path(output: &Path) -> PathBuf {
    let mut name = OsString::from(output.as_os_str());
    name.push(".json");
    PathBuf::from(name)
}

fn load_model(
    name: &str,
    language: &str,
    language_id: i32,
) -> Result<WhisperContext, Box<dyn Error>> {
    let path = fetch_model(name)?;
    whisper_rs::install_logging_hooks();
    let context = WhisperContext::new_with_params(&path, WhisperContextParameters::default())?;
    // Language tokens precede the translate token; smaller models lack later languages.
    if context.token_lang(language_id) >= context.token_translate() {
        return Err(format!("{name} does not support language {language}").into());
    }
    Ok(context)
}

/// Selected audio as 16 kHz mono samples, the input Whisper requires.
fn whisper_audio(decoded: &DecodedAudio) -> Result<Vec<f32>, Box<dyn Error>> {
    let pcm = NamedTempFile::new()?;
    resample(decoded, pcm.as_file())?;
    Ok(f32_samples(&fs::read(pcm.path())?))
}

fn resample(decoded: &DecodedAudio, output: &File) -> Result<(), Box<dyn Error>> {
    let mut child = resampler(decoded.info(), output)?;
    let copied = io::copy(
        &mut decoded.open_selected_pcm()?,
        &mut child.stdin.take().unwrap(),
    );
    let result = child.wait_with_output()?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("cannot prepare transcription audio: {}", stderr.trim()).into());
    }
    Ok(copied.map(drop)?)
}

fn resampler(info: &DecodedInfo, output: &File) -> Result<Child, Box<dyn Error>> {
    let input_rate = info.sample_rate.to_string();
    let input_channels = info.channels.to_string();
    Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-f", "f32le", "-ar", &input_rate])
        .args(["-ac", &input_channels, "-i", "pipe:0"])
        .args(["-ar", SAMPLE_RATE, "-ac", "1", "-f", "f32le", "pipe:1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::from(output.try_clone()?))
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start ffmpeg for transcription audio: {error}").into())
}

fn f32_samples(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|sample| f32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]))
        .collect()
}

fn transcribe(
    context: &WhisperContext,
    audio: &[f32],
    language: &str,
    language_id: i32,
) -> Result<Transcript, Box<dyn Error>> {
    let mut state = context.create_state()?;
    state.full(params(language), audio)?;
    if state.full_lang_id_from_state() != language_id {
        return Err("whisper.cpp returned an unexpected language".into());
    }
    // Word grouping keys on Whisper's code, also when the request named the language.
    let code = whisper_rs::get_lang_str(language_id).ok_or("unknown Whisper language id")?;
    let segments = segments(&state, context.token_eot(), code)?;
    let text = segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect();
    Ok(Transcript {
        text,
        language: language.to_owned(),
        segments,
    })
}

fn segments(state: &WhisperState, eot: i32, code: &str) -> Result<Vec<Segment>, WhisperError> {
    state
        .as_iter()
        .enumerate()
        .map(|(id, segment)| build_segment(id as u64, &segment, eot, code))
        .collect()
}

fn params(language: &str) -> FullParams<'_, '_> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language));
    params.set_translate(false);
    params.set_temperature(0.0);
    params.set_temperature_inc(0.0);
    params.set_no_context(true);
    params.set_token_timestamps(true);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params
}

fn build_segment(
    id: u64,
    segment: &WhisperSegment,
    eot: i32,
    language: &str,
) -> Result<Segment, WhisperError> {
    let pieces = (0..segment.n_tokens())
        .filter_map(|index| segment.get_token(index))
        .filter(|token| token.token_id() < eot)
        .map(|token| piece(&token))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Segment {
        id,
        start: centiseconds(segment.start_timestamp()),
        end: centiseconds(segment.end_timestamp()),
        text: segment.to_str_lossy()?.into_owned(),
        words: words(&pieces, language),
        no_speech_prob: f64::from(segment.no_speech_probability()),
    })
}

fn piece(token: &WhisperToken) -> Result<Piece, WhisperError> {
    let data = token.token_data();
    Ok(Piece {
        bytes: token.to_bytes()?.to_vec(),
        start: centiseconds(data.t0),
        end: centiseconds(data.t1),
        probability: f64::from(data.p),
    })
}

fn centiseconds(value: i64) -> f64 {
    value as f64 / 100.0
}

/// Groups tokens into words as Whisper's reference tokenizer does.
fn words(pieces: &[Piece], language: &str) -> Vec<Word> {
    let spaceless = SPACELESS_LANGUAGES.contains(&language);
    let mut groups: Vec<Vec<&Piece>> = Vec::new();
    for subword in subwords(pieces) {
        let text = String::from_utf8_lossy(&joined(&subword)).into_owned();
        let starts_word = spaceless || text.starts_with(' ') || PUNCTUATION.contains(text.trim());
        match groups.last_mut() {
            Some(group) if !starts_word => group.extend(subword),
            _ => groups.push(subword),
        }
    }
    groups.iter().map(|group| word(group)).collect()
}

/// Splits tokens where their accumulated bytes form complete UTF-8 text.
fn subwords(pieces: &[Piece]) -> Vec<Vec<&Piece>> {
    let mut subwords = Vec::new();
    let mut current = Vec::new();
    for piece in pieces {
        current.push(piece);
        if std::str::from_utf8(&joined(&current)).is_ok() {
            subwords.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        subwords.push(current);
    }
    subwords
}

fn joined(pieces: &[&Piece]) -> Vec<u8> {
    pieces
        .iter()
        .flat_map(|piece| piece.bytes.iter().copied())
        .collect()
}

fn word(pieces: &[&Piece]) -> Word {
    let total: f64 = pieces.iter().map(|piece| piece.probability).sum();
    Word {
        word: String::from_utf8_lossy(&joined(pieces)).into_owned(),
        start: pieces[0].start,
        end: pieces[pieces.len() - 1].end,
        probability: total / pieces.len() as f64,
    }
}

fn map_timestamps(transcript: &mut Transcript, offset: f64) -> Result<(), Box<dyn Error>> {
    for segment in &mut transcript.segments {
        shift(&mut segment.start, &mut segment.end, offset)?;
        for word in &mut segment.words {
            map_word(word, offset)?;
        }
    }
    Ok(())
}

fn map_word(word: &mut Word, offset: f64) -> Result<(), Box<dyn Error>> {
    if !word.probability.is_finite() || !(0.0..=1.0).contains(&word.probability) {
        return Err("whisper.cpp returned an invalid word probability".into());
    }
    shift(&mut word.start, &mut word.end, offset)
}

fn shift(start: &mut f64, end: &mut f64, offset: f64) -> Result<(), Box<dyn Error>> {
    if !start.is_finite() || !end.is_finite() || *start < 0.0 || *start > *end {
        return Err("whisper.cpp returned invalid timestamps".into());
    }
    *start += offset;
    *end += offset;
    Ok(())
}

fn document<'a>(
    source: &'a DecodedInfo,
    model: &str,
    language: &str,
    offset: f64,
    transcript: Transcript,
) -> Document<'a> {
    Document {
        source,
        backend: Backend {
            name: "whisper.cpp",
            version: whisper_rs::WHISPER_CPP_VERSION,
            model: model.to_owned(),
        },
        settings: settings(language),
        timestamp_reference: "original_source_seconds",
        source_offset_seconds: offset,
        transcript,
    }
}

/// The decoding settings `params` applies, recorded with the transcript.
fn settings(language: &str) -> Settings {
    Settings {
        language: language.to_owned(),
        task: "transcribe",
        word_timestamps: true,
        temperature: 0.0,
        condition_on_previous_text: false,
        initial_prompt: None,
    }
}

fn write_outputs(document: &Document, output: &Path, sidecar: &Path) -> Result<(), Box<dyn Error>> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut text_output = NamedTempFile::new_in(parent)?;
    writeln!(text_output, "{}", document.transcript.text.trim())?;
    let mut json_output = NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(json_output.as_file_mut(), document)?;
    writeln!(json_output)?;
    text_output
        .persist_noclobber(output)
        .map_err(|error| format!("cannot save transcript: {error}"))?;
    if let Err(error) = json_output.persist_noclobber(sidecar) {
        fs::remove_file(output)?;
        return Err(format!("cannot save transcript metadata: {error}").into());
    }
    Ok(())
}
