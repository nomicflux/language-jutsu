#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::Value;
use tempfile::TempDir;

struct Fixture {
    directory: TempDir,
    input: PathBuf,
    format: PathBuf,
    bin: PathBuf,
    calls: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("original.wav");
        let rate = 8000_u32;
        let samples: Vec<i16> = (0..rate * 2)
            .map(|n| ((f64::from(n) * 0.3).sin() * 8000.0) as i16)
            .collect();
        let bytes = u32::try_from(samples.len() * 2).unwrap();
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + bytes).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&rate.to_le_bytes());
        wav.extend_from_slice(&(rate * 2).to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&bytes.to_le_bytes());
        for sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }
        fs::write(&input, wav).unwrap();
        let format = directory.path().join("format.json");
        fs::write(
            &format,
            serde_json::to_vec(&serde_json::json!({
                "chunking": {"chunk_target_size_seconds": 10},
                "playback": {"sequencing": "timing-then-joining", "segments": [0.85, 1.0],
                    "joining": {"group_size": 1}, "repetitions": 1, "gap_ms": 400}
            }))
            .unwrap(),
        )
        .unwrap();
        let bin = directory.path().join("bin");
        fs::create_dir(&bin).unwrap();
        let downloader = bin.join("yt-dlp");
        fs::write(&downloader, r#"#!/bin/sh
printf 'call\n' >> "$METADATA_TEST_CALLS"
for arg in "$@"; do
  if [ "$arg" = --dump-single-json ]; then
    printf '%s\n' '{"id":"Abc_1234567","title":"Conversation","webpage_url":"https://www.youtube.com/watch?v=Abc_1234567","live_status":"not_live","is_live":false,"vcodec":"none","acodec":"pcm_s16le","publisher":{"name":"Source publisher"}}'
    exit 0
  fi
done
/bin/cp "$METADATA_TEST_AUDIO" source.wav
printf '%s\n' "$PWD/source.wav"
"#).unwrap();
        fs::set_permissions(downloader, fs::Permissions::from_mode(0o755)).unwrap();
        let calls = directory.path().join("calls.txt");
        Self {
            directory,
            input,
            format,
            bin,
            calls,
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.path().join(name)
    }

    fn command(&self, output: &Path, remote: bool) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_language-jutsu"));
        command
            .env(
                "PATH",
                format!("{}:{}", self.bin.display(), std::env::var("PATH").unwrap()),
            )
            .env("METADATA_TEST_CALLS", &self.calls)
            .env("METADATA_TEST_AUDIO", &self.input)
            .args(["--language", "ja", "--start", "0.25", "--end", "1.25"])
            .arg("--format")
            .arg(&self.format)
            .arg("--output")
            .arg(output);
        if remote {
            command.args(["--url", "https://youtu.be/Abc_1234567?t=20"]);
        }
        command
    }
}

fn success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn retained_full_download_reuses_provenance_and_render_metadata_measures_actual_frames() {
    let fixture = Fixture::new();
    let saved = fixture.path("saved.wav");
    let output = fixture.path("study.wav");
    let metadata = fixture.path("study.json");
    success(
        fixture
            .command(&output, true)
            .arg("--save-source")
            .arg(&saved)
            .arg("--metadata-output")
            .arg(&metadata)
            .output()
            .unwrap(),
    );
    assert_eq!(fs::read(&saved).unwrap(), fs::read(&fixture.input).unwrap());
    let retained = read_json(&fixture.path("saved.wav.source.json"));
    assert_eq!(retained["sha256"].as_str().unwrap().len(), 64);
    let report = read_json(&metadata);
    assert_eq!(report["source"]["origin"], retained["origin"]);
    assert_eq!(
        report["source"]["origin"]["metadata"]["publisher"]["name"],
        "Source publisher"
    );
    assert_eq!(
        report["source"]["source_path"],
        saved.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(report["source"]["original_duration_seconds"], 2.0);
    assert_eq!(report["source"]["selected_duration_seconds"], 1.0);
    assert_eq!(report["source"]["selected_start_seconds"], 0.25);
    assert_eq!(report["source"]["selected_end_seconds"], 1.25);
    assert_eq!(
        report["format"]["playback"]["segments"],
        serde_json::json!([0.85, 1.0])
    );
    assert_eq!(
        report["output"]["path"],
        output.canonicalize().unwrap().to_str().unwrap()
    );
    let decoded = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(&output)
        .args(["-f", "f32le", "-c:a", "pcm_f32le", "pipe:1"])
        .output()
        .unwrap();
    assert!(decoded.status.success());
    let frames = decoded.stdout.len() / 4;
    assert_eq!(report["output"]["pcm_frames"], frames);
    assert_eq!(
        report["output"]["pcm_duration_seconds"],
        frames as f64 / 8000.0
    );
    assert_ne!(
        report["output"]["pcm_duration_seconds"],
        report["sequence"]["estimated_duration_seconds"]
    );

    let reused = fixture.path("reused.f32le");
    success(
        fixture
            .command(&reused, false)
            .arg(&saved)
            .arg("--decode-only")
            .output()
            .unwrap(),
    );
    let reused_info = read_json(&fixture.path("reused.f32le.json"));
    assert_eq!(reused_info["origin"], report["source"]["origin"]);
    assert_eq!(reused_info["selected_duration_seconds"], 1.0);
    assert_eq!(
        fs::read_to_string(&fixture.calls).unwrap().lines().count(),
        2
    );
}

#[test]
fn retained_source_rejects_changed_bytes_and_never_overwrites_saved_files() {
    let fixture = Fixture::new();
    let saved = fixture.path("saved.wav");
    success(
        fixture
            .command(&fixture.path("first.f32le"), true)
            .arg("--save-source")
            .arg(&saved)
            .arg("--decode-only")
            .output()
            .unwrap(),
    );
    let original = fs::read(&saved).unwrap();
    let sidecar = fixture.path("saved.wav.source.json");
    let original_sidecar = fs::read(&sidecar).unwrap();
    let repeated = fixture
        .command(&fixture.path("second.f32le"), true)
        .arg("--save-source")
        .arg(&saved)
        .arg("--decode-only")
        .output()
        .unwrap();
    assert!(!repeated.status.success());
    assert_eq!(fs::read(&saved).unwrap(), original);
    assert_eq!(fs::read(&sidecar).unwrap(), original_sidecar);
    fs::remove_file(&saved).unwrap();
    let sidecar_collision = fixture
        .command(&fixture.path("third.f32le"), true)
        .arg("--save-source")
        .arg(&saved)
        .arg("--decode-only")
        .output()
        .unwrap();
    assert!(!sidecar_collision.status.success());
    assert!(!saved.exists());
    assert_eq!(fs::read(&sidecar).unwrap(), original_sidecar);
    let mut changed = original;
    *changed.last_mut().unwrap() ^= 1;
    fs::write(&saved, changed).unwrap();
    let rejected = fixture
        .command(&fixture.path("changed.f32le"), false)
        .arg(&saved)
        .arg("--decode-only")
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("SHA-256 mismatch"));
    assert!(!fixture.path("changed.f32le").exists());
}

#[test]
fn unusable_render_metadata_destination_leaves_no_audio_or_download() {
    let fixture = Fixture::new();
    let audio = fixture.path("study.wav");
    let metadata = fixture.path("study.json");
    fs::write(&metadata, "existing metadata").unwrap();
    for destination in [&metadata, &audio, &fixture.path("missing/study.json")] {
        let output = fixture
            .command(&audio, true)
            .arg("--metadata-output")
            .arg(destination)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!audio.exists());
        assert!(!fixture.calls.exists());
    }
    assert_eq!(fs::read_to_string(metadata).unwrap(), "existing metadata");
}

#[test]
fn metadata_publication_failure_rolls_back_new_audio() {
    let fixture = Fixture::new();
    let actual_ffmpeg = Command::new("which").arg("ffmpeg").output().unwrap();
    assert!(actual_ffmpeg.status.success());
    let wrapper = fixture.bin.join("ffmpeg");
    fs::write(
        &wrapper,
        r#"#!/bin/sh
"$METADATA_REAL_FFMPEG" "$@" || exit $?
for arg in "$@"; do
  if [ "$arg" = -rf64 ]; then
    printf 'racing metadata\n' > "$METADATA_RACE_PATH"
    break
  fi
done
"#,
    )
    .unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    let metadata = fixture.path("raced.json");
    let audio = fixture.path("raced.wav");
    let output = fixture
        .command(&audio, false)
        .arg(&fixture.input)
        .env(
            "METADATA_REAL_FFMPEG",
            String::from_utf8(actual_ffmpeg.stdout).unwrap().trim(),
        )
        .env("METADATA_RACE_PATH", &metadata)
        .arg("--metadata-output")
        .arg(&metadata)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot save render metadata"));
    assert!(!audio.exists());
    assert_eq!(fs::read_to_string(metadata).unwrap(), "racing metadata\n");
}
