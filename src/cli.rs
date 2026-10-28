use std::{fmt, path::PathBuf};

use crate::{
    segmentation::ChunkTargetSize,
    source::{Source, YouTubeVideo},
    window,
};
use clap::{ArgGroup, Parser, Subcommand};
use serde::Serialize;

#[derive(Parser)]
#[command(
    version,
    about,
    subcommand_negates_reqs = true,
    args_conflicts_with_subcommands = true
)]
#[command(group(ArgGroup::new("source").args(["input", "url"]).required(true).multiple(false)))]
#[command(group(ArgGroup::new("mode").args(["check", "decode_only", "segment_only", "sequence_only", "transcribe"]).multiple(false)))]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,

    /// Source audio file.
    input: Option<PathBuf>,

    /// Single YouTube video to download before processing.
    #[arg(long)]
    url: Option<YouTubeVideo>,

    /// Start on the original source timeline: seconds, MM:SS, or HH:MM:SS.
    #[arg(long)]
    start: Option<window::Timestamp>,

    /// Exclusive end on the original source timeline; omitted means end of recording.
    #[arg(long)]
    end: Option<window::Timestamp>,

    /// Source language code or name.
    ///
    /// Named choices: french (fr), italian (it), german (de), mandarin (zh),
    /// cantonese (yue), korean (ko), farsi/persian (fa), indonesian (id),
    /// bengali (bn), hausa (ha), tamil (ta), hindi (hi), turkish (tr), vietnamese (vi),
    /// kiswahili/swahili (sw), isizulu/zulu (zu), isixhosa/xhosa (xh), sesotho (st).
    /// Other nonempty labels remain valid for audio processing.
    /// Transcription depends on Whisper's language and model support.
    #[arg(long, required = true, value_parser = language_code)]
    language: Option<String>,

    /// Optional dialect label retained in metadata.
    #[arg(long, value_parser = nonempty_label)]
    dialect: Option<String>,

    /// Complete format JSON; otherwise read formats/default.json beside the executable.
    #[arg(long, conflicts_with = "transcribe")]
    format: Option<PathBuf>,

    /// Target source chunk duration in seconds; overrides format.chunking.
    #[arg(long, conflicts_with_all = ["decode_only", "transcribe"])]
    chunk_target_size_seconds: Option<ChunkTargetSize>,

    /// Study audio, decoded PCM, JSON report, or transcript text.
    #[arg(long, required = true)]
    output: Option<PathBuf>,

    /// Check the source and complete format without processing audio or downloading.
    #[arg(long)]
    check: bool,

    /// Decode to interleaved f32le PCM and write a companion .json metadata file.
    #[arg(long)]
    decode_only: bool,

    /// Decode and find pause-based units without generating study audio.
    #[arg(long)]
    segment_only: bool,

    /// Expand the formula into a JSON sequence without rendering audio.
    #[arg(long)]
    sequence_only: bool,

    /// Transcribe the selected source audio with Whisper, without a playback format.
    #[arg(long)]
    transcribe: bool,

    /// Whisper model file in the ggerganov/whisper.cpp Hugging Face repository (transcription only).
    #[arg(long, requires = "transcribe", default_value = crate::transcription::DEFAULT_MODEL)]
    whisper_model: String,

    /// Retain the full downloaded source and its provenance sidecar at this path.
    #[arg(long, requires = "url", conflicts_with = "check")]
    save_source: Option<PathBuf>,

    /// Write render metadata, including actual duration, to this JSON file.
    #[arg(long, conflicts_with_all = ["check", "decode_only", "segment_only", "sequence_only", "transcribe"])]
    metadata_output: Option<PathBuf>,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Package authored vocabulary, grammar and listening cards with audio for Anki.
    Anki {
        /// Card manifest JSON; each card requires context: {audio, sentence} for one chunk.
        /// Relative audio paths resolve beside this file.
        input: PathBuf,
        /// New .apkg file; existing files are never replaced.
        #[arg(long)]
        output: PathBuf,
    },
    /// Download a Whisper model into the Hugging Face cache for --transcribe.
    DownloadModel {
        /// Model file in the ggerganov/whisper.cpp Hugging Face repository.
        #[arg(default_value = crate::transcription::DEFAULT_MODEL)]
        model: String,
    },
}

fn nonempty_label(value: &str) -> Result<String, String> {
    if value.trim().is_empty() {
        Err("language and dialect labels must not be empty".into())
    } else {
        Ok(value.to_owned())
    }
}

fn language_code(value: &str) -> Result<String, String> {
    let label = nonempty_label(value)?;
    let code = match label.trim().to_ascii_lowercase().as_str() {
        "fr" | "french" => "fr",
        "it" | "italian" => "it",
        "de" | "german" => "de",
        "zh" | "mandarin" | "chinese" => "zh",
        "yue" | "cantonese" => "yue",
        "ko" | "korean" => "ko",
        "fa" | "farsi" | "persian" => "fa",
        "id" | "indonesian" => "id",
        "bn" | "bengali" => "bn",
        "ha" | "hausa" => "ha",
        "ta" | "tamil" => "ta",
        "hi" | "hindi" => "hi",
        "tr" | "turkish" => "tr",
        "vi" | "vietnamese" => "vi",
        "sw" | "kiswahili" | "swahili" => "sw",
        "zu" | "isizulu" | "zulu" => "zu",
        "xh" | "isixhosa" | "xhosa" => "xh",
        "st" | "sesotho" => "st",
        _ => return Ok(label),
    };
    Ok(code.to_owned())
}

#[derive(Serialize)]
pub(crate) struct LanguageSetting {
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dialect: Option<String>,
}

impl LanguageSetting {
    pub(crate) fn code(&self) -> &str {
        &self.code
    }
}

impl fmt::Display for LanguageSetting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.dialect {
            Some(dialect) => write!(formatter, "{} ({dialect})", self.code),
            None => formatter.write_str(&self.code),
        }
    }
}

pub(crate) enum Mode {
    Check,
    Decode,
    Segment,
    Sequence,
    Render,
    Transcribe,
}

pub(crate) struct Request {
    pub(crate) source: Source,
    pub(crate) language: LanguageSetting,
    pub(crate) format_path: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) mode: Mode,
    pub(crate) window: window::Request,
    pub(crate) chunk_target_size_seconds: Option<ChunkTargetSize>,
    pub(crate) whisper: crate::transcription::Options,
    pub(crate) save_source: Option<PathBuf>,
    pub(crate) metadata_output: Option<PathBuf>,
}

impl TryFrom<Args> for Request {
    type Error = String;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let window = window::Request::new(args.start, args.end)?;
        let source = match (args.input, args.url) {
            (Some(path), None) => Source::File(path),
            (None, Some(video)) => Source::YouTube(video),
            _ => return Err("supply exactly one audio file or --url".into()),
        };
        let language = LanguageSetting {
            code: args.language.ok_or("--language is required")?,
            dialect: args.dialect,
        };
        let format_path = match args.format {
            Some(path) => path,
            None => {
                let executable = std::env::current_exe()
                    .map_err(|error| format!("cannot locate executable: {error}"))?;
                executable
                    .parent()
                    .ok_or("executable has no parent directory")?
                    .join("formats/default.json")
            }
        };

        let mode = if args.transcribe {
            Mode::Transcribe
        } else if args.decode_only {
            Mode::Decode
        } else if args.check {
            Mode::Check
        } else if args.segment_only {
            Mode::Segment
        } else if args.sequence_only {
            Mode::Sequence
        } else {
            Mode::Render
        };
        Ok(Self {
            source,
            language,
            format_path,
            output: args.output.ok_or("--output is required")?,
            mode,
            window,
            chunk_target_size_seconds: args.chunk_target_size_seconds,
            whisper: crate::transcription::Options {
                model: args.whisper_model,
            },
            save_source: args.save_source,
            metadata_output: args.metadata_output,
        })
    }
}
