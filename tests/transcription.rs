#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

use serde_json::Value;
use tempfile::TempDir;

struct Fixture {
    dir: TempDir,
}

impl Fixture {
    fn new(mode: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let samples: Vec<i16> = (0..16000).collect();
        write_wav(&dir.path().join("input.wav"), &samples);
        let helper = include_str!("helpers/whisper_python.py");
        let (shebang, body) = helper.split_once('\n').unwrap();
        let fake_python = dir.path().join("python");
        fs::write(&fake_python, format!("{shebang}\nMODE = {mode:?}\n{body}")).unwrap();
        fs::set_permissions(&fake_python, fs::Permissions::from_mode(0o755)).unwrap();
        Self { dir }
    }

    fn run(&self) -> std::process::Output {
        // There is no format file or --format: transcription is standalone.
        Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
            .arg(self.dir.path().join("input.wav"))
            .args([
                "--language",
                "ja",
                "--transcribe",
                "--start",
                "0.10001",
                "--end",
                "0.40001",
                "--whisper-model",
                "test/model",
                "--whisper-python",
            ])
            .arg(self.dir.path().join("python"))
            .arg("--output")
            .arg(self.dir.path().join("transcript.txt"))
            .output()
            .unwrap()
    }

    fn assert_no_outputs(&self) {
        assert!(!self.dir.path().join("transcript.txt").exists());
        assert!(!self.dir.path().join("transcript.txt.json").exists());
    }
}

fn write_wav(path: &Path, samples: &[i16]) {
    let data_size = (samples.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&16000_u32.to_le_bytes());
    bytes.extend_from_slice(&32000_u32.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).unwrap();
}

#[test]
fn selected_audio_is_transcribed_without_format_and_times_map_to_source() {
    let fixture = Fixture::new("ok");
    let result = fixture.run();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read_to_string(fixture.dir.path().join("transcript.txt")).unwrap(),
        "お茶にします。\n"
    );
    let json: Value =
        serde_json::from_slice(&fs::read(fixture.dir.path().join("transcript.txt.json")).unwrap())
            .unwrap();
    assert_eq!(json["timestamp_reference"], "original_source_seconds");
    assert_eq!(json["source_offset_seconds"], 1601.0 / 16000.0);
    assert_eq!(json["segments"][0]["start"], 1601.0 / 16000.0 + 0.02);
    assert_eq!(json["segments"][0]["end"], 1601.0 / 16000.0 + 0.25);
    assert_eq!(
        json["segments"][0]["words"][0]["start"],
        1601.0 / 16000.0 + 0.04
    );
    assert_eq!(
        json["segments"][0]["words"][0]["end"],
        1601.0 / 16000.0 + 0.12
    );
    assert_eq!(json["segments"][0]["words"][0]["probability"], 0.8);
    assert_eq!(json["segments"][0]["avg_logprob"], -0.3);
    assert_eq!(json["backend"]["name"], "mlx-whisper");
    assert_eq!(json["backend"]["version"], "test-version");
    assert_eq!(json["backend"]["model"], "test/model");
    assert_eq!(json["settings"]["task"], "transcribe");
    assert_eq!(json["source"]["selected_range"]["start_frame"], 1601);
    assert_eq!(json["source"]["selected_range"]["end_frame"], 6401);
    assert!(!json.to_string().contains("backend diagnostics"));
}

#[test]
fn helper_failure_or_invalid_response_leaves_no_outputs() {
    for (mode, error) in [
        ("backend_failure", "test model failed"),
        ("invalid_json", "invalid mlx-whisper helper response"),
        ("bad_timestamp", "invalid timestamps"),
        ("wrong_language", "unexpected model, language, or settings"),
        ("missing_dependency", "--whisper-python"),
    ] {
        let fixture = Fixture::new(mode);
        let result = fixture.run();
        assert!(!result.status.success(), "{mode}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(error),
            "{mode}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        fixture.assert_no_outputs();
    }
}

#[test]
fn existing_text_or_sidecar_is_never_replaced() {
    for existing in ["transcript.txt", "transcript.txt.json"] {
        let fixture = Fixture::new("ok");
        fs::write(fixture.dir.path().join(existing), "keep me").unwrap();
        let result = fixture.run();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("already exists"));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join(existing)).unwrap(),
            "keep me"
        );
        let other = if existing.ends_with("json") {
            "transcript.txt"
        } else {
            "transcript.txt.json"
        };
        assert!(!fixture.dir.path().join(other).exists());
    }
}
