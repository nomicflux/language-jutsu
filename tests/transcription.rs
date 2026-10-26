use std::{fs, path::PathBuf, process::Command};

use serde_json::Value;
use tempfile::TempDir;

// Real whisper.cpp with the smallest multilingual model; fetched once into the HF cache.
const MODEL: &str = "ggml-tiny.bin";

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

struct Run {
    dir: TempDir,
    result: std::process::Output,
}

impl Run {
    fn new(input: &str, language: &str, model: &str, window: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        Self::in_dir(dir, input, language, model, window)
    }

    fn in_dir(dir: TempDir, input: &str, language: &str, model: &str, window: &[&str]) -> Self {
        // There is no format file or --format: transcription is standalone.
        let result = Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
            .arg(fixture(input))
            .args([
                "--language",
                language,
                "--transcribe",
                "--whisper-model",
                model,
            ])
            .args(window)
            .arg("--output")
            .arg(dir.path().join("transcript.txt"))
            .output()
            .unwrap();
        Self { dir, result }
    }

    fn metadata(&self) -> Value {
        let stderr = String::from_utf8_lossy(&self.result.stderr);
        assert!(self.result.status.success(), "{stderr}");
        let json = fs::read(self.dir.path().join("transcript.txt.json")).unwrap();
        serde_json::from_slice(&json).unwrap()
    }

    fn assert_failed_without_outputs(&self, error: &str) {
        let stderr = String::from_utf8_lossy(&self.result.stderr);
        assert!(!self.result.status.success());
        assert!(stderr.contains(error), "{stderr}");
        assert!(!self.dir.path().join("transcript.txt").exists());
        assert!(!self.dir.path().join("transcript.txt.json").exists());
    }
}

fn words(json: &Value) -> Vec<(String, f64)> {
    json["segments"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|segment| segment["words"].as_array().unwrap())
        .map(|word| {
            (
                word["word"].as_str().unwrap().to_owned(),
                word["start"].as_f64().unwrap(),
            )
        })
        .collect()
}

fn first_start(json: &Value, wanted: &str) -> f64 {
    words(json)
        .into_iter()
        .find(|(word, _)| word == wanted)
        .unwrap()
        .1
}

fn assert_words_rebuild_segments(json: &Value) {
    for segment in json["segments"].as_array().unwrap() {
        let joined: String = segment["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| word["word"].as_str().unwrap())
            .collect();
        assert_eq!(joined, segment["text"].as_str().unwrap());
        assert!(!joined.contains('\u{FFFD}'), "{joined}");
    }
}

#[test]
fn english_speech_is_transcribed_into_whole_words() {
    let json = Run::new("jfk.wav", "en", MODEL, &[]).metadata();
    let text = json["text"].as_str().unwrap();
    assert!(
        text.contains("ask not what your country can do for you"),
        "{text}"
    );
    assert_words_rebuild_segments(&json);
    let words: Vec<String> = words(&json).into_iter().map(|(word, _)| word).collect();
    assert!(words.contains(&" Americans".to_owned()), "{words:?}");
    assert!(words.contains(&" country".to_owned()), "{words:?}");
    assert_eq!(json["backend"]["name"], "whisper.cpp");
    assert_eq!(json["backend"]["model"], MODEL);
    assert_eq!(json["settings"]["task"], "transcribe");
}

#[test]
fn selected_window_keeps_times_on_the_original_recording() {
    let full = Run::new("jfk.wav", "en", MODEL, &[]).metadata();
    let window = Run::new("jfk.wav", "en", MODEL, &["--start", "3", "--end", "11"]).metadata();
    assert_eq!(window["timestamp_reference"], "original_source_seconds");
    assert_eq!(window["source_offset_seconds"], 3.0);
    assert_eq!(window["source"]["selected_range"]["start_frame"], 48000);
    let text = window["text"].as_str().unwrap();
    assert!(!text.contains("fellow Americans"), "{text}");
    let (whole, selected) = (
        first_start(&full, " country"),
        first_start(&window, " country"),
    );
    assert!((whole - selected).abs() < 0.3, "{whole} vs {selected}");
}

#[test]
fn japanese_speech_is_split_into_character_groups() {
    let json = Run::new("botchan-opening.mp3", "japanese", MODEL, &[]).metadata();
    let text = json["text"].as_str().unwrap();
    for spoken in ["リブリボックス", "子供", "学校"] {
        assert!(text.contains(spoken), "{text}");
    }
    assert_words_rebuild_segments(&json);
    let words: Vec<String> = words(&json).into_iter().map(|(word, _)| word).collect();
    assert!(
        words.contains(&"子".to_owned()) && words.contains(&"供".to_owned()),
        "{words:?}"
    );
}

#[test]
fn language_names_reach_whisper_as_codes_and_round_trip_in_metadata() {
    for (name, code) in [
        ("french", "fr"),
        ("italian", "it"),
        ("german", "de"),
        ("mandarin", "zh"),
        ("korean", "ko"),
        ("farsi", "fa"),
        ("hindi", "hi"),
        ("kiswahili", "sw"),
    ] {
        let json = Run::new("jfk.wav", name, MODEL, &["--end", "3"]).metadata();
        assert_eq!(json["source"]["language"]["code"], code);
        assert_eq!(json["settings"]["language"], code);
        assert_eq!(json["language"], code);
    }
}

#[test]
fn unsupported_language_or_model_leaves_no_outputs() {
    for (language, model, error) in [
        (
            "cantonese",
            MODEL,
            "ggml-tiny.bin does not support language yue",
        ),
        ("zulu", MODEL, "Whisper does not support language code zu"),
        (
            "en",
            "ggml-nope.bin",
            "cannot fetch Whisper model ggml-nope.bin",
        ),
    ] {
        Run::new("jfk.wav", language, model, &[]).assert_failed_without_outputs(error);
    }
}

#[test]
fn existing_text_or_sidecar_is_never_replaced() {
    for (existing, other) in [
        ("transcript.txt", "transcript.txt.json"),
        ("transcript.txt.json", "transcript.txt"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(existing), "keep me").unwrap();
        let run = Run::in_dir(dir, "jfk.wav", "en", MODEL, &[]);
        let stderr = String::from_utf8_lossy(&run.result.stderr);
        assert!(!run.result.status.success());
        assert!(stderr.contains("already exists"), "{stderr}");
        assert_eq!(
            fs::read_to_string(run.dir.path().join(existing)).unwrap(),
            "keep me"
        );
        assert!(!run.dir.path().join(other).exists());
    }
}

#[test]
fn download_model_caches_the_file_in_the_hugging_face_cache() {
    let result = Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
        .args(["download-model", MODEL])
        .output()
        .unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let path = stdout.trim().strip_prefix("Model cached: ").unwrap();
    assert!(path.contains("models--ggerganov--whisper.cpp"), "{path}");
    assert!(PathBuf::from(path).is_file());
}
