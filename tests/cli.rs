use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    input: PathBuf,
    format: PathBuf,
    output: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "language-jutsu-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let input = directory.join("recording.wav");
        let format = directory.join("format.json");
        let output = directory.join("study.mp3");
        fs::write(&input, b"configuration-check fixture, not decoded audio").unwrap();
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("formats/default.json"),
            &format,
        )
        .unwrap();
        Self {
            directory,
            input,
            format,
            output,
        }
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
            .arg(&self.input)
            .arg("--format")
            .arg(&self.format)
            .arg("--output")
            .arg(&self.output)
            .args(arguments)
            .output()
            .unwrap()
    }

    fn installed_binary(&self) -> PathBuf {
        let binary = self
            .directory
            .join(format!("language-jutsu{}", std::env::consts::EXE_SUFFIX));
        fs::copy(env!("CARGO_BIN_EXE_language-jutsu"), &binary).unwrap();
        let directory = self.directory.join("formats");
        fs::create_dir(&directory).unwrap();
        fs::copy(&self.format, directory.join("default.json")).unwrap();
        binary
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).unwrap();
    }
}

fn write_wav(path: &std::path::Path, sample_rate: u32, channels: u16, samples: &[i16]) {
    let data_size = u32::try_from(samples.len() * 2).unwrap();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * u32::from(channels) * 2).to_le_bytes());
    bytes.extend_from_slice(&(channels * 2).to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).unwrap();
}

fn assert_decoded(fixture: &Fixture, samples: &[i16], sample_rate: u32, channels: u16) {
    let output = fixture.run(&["--language", "ar", "--dialect", "masri", "--decode-only"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pcm = fs::read(&fixture.output).unwrap();
    assert_eq!(pcm.len(), samples.len() * 4);
    for (bytes, source) in pcm.chunks_exact(4).zip(samples) {
        let decoded = f32::from_le_bytes(bytes.try_into().unwrap());
        assert!((decoded - f32::from(*source) / 32768.0).abs() < 1e-7);
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(format!("{}.json", fixture.output.display())).unwrap())
            .unwrap();
    assert_eq!(metadata["sample_format"], "f32le");
    assert_eq!(metadata["sample_rate"], sample_rate);
    assert_eq!(metadata["channels"], channels);
    assert_eq!(
        metadata["frame_count"],
        samples.len() / usize::from(channels)
    );
    assert_eq!(metadata["language"]["code"], "ar");
    assert_eq!(metadata["language"]["dialect"], "masri");
    assert_eq!(
        metadata["source_path"],
        fixture.input.canonicalize().unwrap().to_str().unwrap()
    );
}

#[test]
fn checks_real_json_and_arabic_dialect_without_writing_audio() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--language", "ar", "--dialect", "shami", "--check"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Language: ar (shami)"));
    assert!(stdout.contains("Speeds: 0.75x, 0.9x, 1x, 1.1x"));
    assert!(stdout.contains("Audio contents have not been analyzed"));
    assert!(!fixture.output.exists());
}

#[test]
fn rereads_custom_format_and_preserves_speed_order_and_duplicates() {
    let fixture = Fixture::new();
    let mut formula: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    formula["playback"]["sequencing"] = "joining-then-timing".into();
    formula["playback"]["segments"] = serde_json::json!([1.1, 0.75, 0.75]);
    formula["playback"]["joining"]["group_size"] = 2.into();
    formula["playback"]["repetitions"] = 3.into();
    formula["playback"]["gap_ms"] = 0.into();
    fs::write(&fixture.format, serde_json::to_vec(&formula).unwrap()).unwrap();

    let output = fixture.run(&["--language", "ja", "--check"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for setting in [
        "joining-then-timing (span first)",
        "Speeds: 1.1x, 0.75x, 0.75x",
        "Joining group size: 2",
        "Repetitions: 3",
        "Gap: 0 ms",
    ] {
        assert!(stdout.contains(setting), "missing {setting}: {stdout}");
    }
}

#[test]
fn missing_format_file_or_required_field_never_falls_back() {
    let fixture = Fixture::new();
    fs::remove_file(&fixture.format).unwrap();
    let output = fixture.run(&["--language", "es", "--check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot open format"));
    fs::write(&fixture.format, b"{}").unwrap();
    let output = fixture.run(&["--language", "es", "--check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing field"));
    assert!(!fixture.output.exists());
}

#[test]
fn rejects_invalid_playback_controls() {
    let fixture = Fixture::new();
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    for invalid in [
        serde_json::json!({"segments": []}),
        serde_json::json!({"segments": [0.0]}),
        serde_json::json!({"segments": [-0.75]}),
        serde_json::json!({"joining": {"group_size": 0}}),
        serde_json::json!({"repetitions": 0}),
        serde_json::json!({"gap_ms": -1}),
        serde_json::json!({"sequencing": "unknown"}),
    ] {
        let mut formula = original.clone();
        for (key, value) in invalid.as_object().unwrap() {
            formula["playback"][key] = value.clone();
        }
        fs::write(&fixture.format, serde_json::to_vec(&formula).unwrap()).unwrap();
        let output = fixture.run(&["--language", "ja", "--check"]);
        assert!(!output.status.success(), "accepted {invalid}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid format"));
        assert!(!fixture.output.exists());
    }
}

#[test]
fn accepts_requested_language_and_dialect_and_rejects_missing_input() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--language", "pt", "--dialect", "Brazilian", "--check"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Language: pt (Brazilian)"));
    fs::remove_file(&fixture.input).unwrap();
    let output = fixture.run(&["--language", "ar", "--dialect", "masri", "--check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read input"));
}

#[test]
fn named_languages_render_and_save_canonical_codes_without_restricting_other_labels() {
    let mut fixture = Fixture::new();
    fixture.output = fixture.directory.join("study.wav");
    write_wav(&fixture.input, 1000, 1, &[10000; 2000]);
    set_recipe(&fixture, &[1.0], "joining-then-timing", 1, 0, 1);
    let metadata_path = fixture.directory.join("study.json");
    for (label, code) in [
        ("French", "fr"),
        ("ITALIAN", "it"),
        ("German", "de"),
        (" Mandarin ", "zh"),
        ("Cantonese", "yue"),
        ("Korean", "ko"),
        ("Farsi", "fa"),
        ("Hindi", "hi"),
        ("Kiswahili", "sw"),
        ("isiZulu", "zu"),
        ("isiXhosa", "xh"),
        ("Sesotho", "st"),
        ("Chinese", "zh"),
        ("Persian", "fa"),
        ("Swahili", "sw"),
        ("Zulu", "zu"),
        ("Xhosa", "xh"),
        ("YUE", "yue"),
        ("custom-language", "custom-language"),
    ] {
        let result = fixture.run(&[
            "--language",
            label,
            "--metadata-output",
            metadata_path.to_str().unwrap(),
        ]);
        assert!(
            result.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
        assert_eq!(metadata["source"]["language"]["code"], code, "{label}");
        assert_eq!(
            read_audio(&fixture.output),
            vec![10000.0 / 32768.0; 2000],
            "{label}"
        );
        fs::remove_file(&fixture.output).unwrap();
        fs::remove_file(&metadata_path).unwrap();
    }
}

#[test]
fn default_format_is_an_external_required_file() {
    let fixture = Fixture::new();
    let binary = fixture.installed_binary();
    let default = fixture.directory.join("formats/default.json");

    let run = || {
        Command::new(&binary)
            .current_dir(&fixture.directory)
            .arg(&fixture.input)
            .args([
                "--language",
                "ar",
                "--dialect",
                "masri",
                "--check",
                "--output",
            ])
            .arg(&fixture.output)
            .output()
            .unwrap()
    };
    let output = run();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Language: ar (masri)"));

    fs::remove_file(&default).unwrap();
    let output = run();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot open format"));
    assert!(!fixture.output.exists());
}

#[test]
fn default_render_produces_playable_mp3() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 32000, 1, &[10000; 4096]);
    let output = fixture.run(&["--language", "es"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pcm = read_audio(&fixture.output);
    assert!(!pcm.is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Study audio saved:"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Decoded frames: 4096"));
}

#[test]
fn decoder_preserves_mono_and_stereo_samples_and_native_rates() {
    for (rate, channels) in [(22050, 1), (48000, 2)] {
        let mut fixture = Fixture::new();
        let input = fixture.directory.join("録音 sample.media");
        let samples: Vec<i16> = (0..4096 * channels)
            .map(|index| ((index % 251) as i16 - 125) * 100)
            .collect();
        write_wav(&input, rate, channels, &samples);
        fixture.input = input;
        let original = fs::read(&fixture.input).unwrap();
        assert_decoded(&fixture, &samples, rate, channels);
        assert_eq!(fs::read(&fixture.input).unwrap(), original);
    }
}

#[test]
fn decoder_handles_lossless_compressed_audio() {
    let mut fixture = Fixture::new();
    let samples: Vec<i16> = (0..4096)
        .map(|index| ((index % 251) as i16 - 125) * 100)
        .collect();
    write_wav(&fixture.input, 32000, 1, &samples);
    let compressed = fixture.directory.join("recording.flac");
    let encoded = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(&fixture.input)
        .args(["-c:a", "flac"])
        .arg(&compressed)
        .output()
        .expect("real ffmpeg is required for decoder integration tests");
    assert!(
        encoded.status.success(),
        "{}",
        String::from_utf8_lossy(&encoded.stderr)
    );
    fixture.input = compressed;
    assert_decoded(&fixture, &samples, 32000, 1);
}

#[test]
fn decoder_handles_mp3_and_counts_decoded_frames() {
    let mut fixture = Fixture::new();
    write_wav(&fixture.input, 32000, 1, &[1000; 32000]);
    let compressed = fixture.directory.join("recording.mp3");
    let encoded = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(&fixture.input)
        .args(["-c:a", "libmp3lame"])
        .arg(&compressed)
        .output()
        .expect("real ffmpeg is required for decoder integration tests");
    assert!(
        encoded.status.success(),
        "{}",
        String::from_utf8_lossy(&encoded.stderr)
    );
    fixture.input = compressed;
    let output = fixture.run(&["--language", "ja", "--decode-only"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(format!("{}.json", fixture.output.display())).unwrap())
            .unwrap();
    assert_eq!(metadata["sample_rate"], 32000);
    assert_eq!(metadata["channels"], 1);
    assert_eq!(metadata["frame_count"], 32000);
    assert_eq!(fs::metadata(&fixture.output).unwrap().len(), 32000 * 4);
    assert_eq!(metadata["source_stream"]["codec_name"], "mp3");
}

#[test]
fn decoder_selects_first_audio_stream() {
    let mut fixture = Fixture::new();
    let first = vec![1234; 4096];
    write_wav(&fixture.input, 22050, 1, &first);
    let second = fixture.directory.join("second.wav");
    write_wav(&second, 48000, 2, &[-4321; 8192]);
    let multi = fixture.directory.join("multi.mka");
    let encoded = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(&fixture.input)
        .arg("-i")
        .arg(&second)
        .args(["-map", "0:a:0", "-map", "1:a:0", "-c:a", "copy"])
        .arg(&multi)
        .output()
        .expect("real ffmpeg is required for decoder integration tests");
    assert!(
        encoded.status.success(),
        "{}",
        String::from_utf8_lossy(&encoded.stderr)
    );
    fixture.input = multi;
    assert_decoded(&fixture, &first, 22050, 1);
}

#[test]
fn decoding_rejects_corrupt_media_and_missing_backend_without_output() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--language", "ja", "--decode-only"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ffprobe failed"));
    assert!(!fixture.output.exists());
    assert!(!PathBuf::from(format!("{}.json", fixture.output.display())).exists());

    let empty_path = fixture.directory.join("empty-path");
    fs::create_dir(&empty_path).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
        .env("PATH", &empty_path)
        .arg(&fixture.input)
        .arg("--format")
        .arg(&fixture.format)
        .arg("--output")
        .arg(&fixture.output)
        .args(["--language", "ja", "--decode-only"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot start ffprobe"));
    assert!(!fixture.output.exists());
}

#[test]
fn decoder_discards_partial_audio_when_decoding_fails() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 32000, 1, &[1000; 4096]);
    let file = fs::OpenOptions::new()
        .write(true)
        .open(&fixture.input)
        .unwrap();
    file.set_len(file.metadata().unwrap().len() - 1).unwrap();
    let output = fixture.run(&["--language", "es", "--decode-only"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ffmpeg decoding failed"));
    assert!(!fixture.output.exists());
    assert!(!PathBuf::from(format!("{}.json", fixture.output.display())).exists());
}

#[test]
fn segmentation_uses_safe_pause_interiors_and_preserves_all_frames() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    let mut samples = vec![10000; 2000];
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 2000]);
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 2000]);
    write_wav(&fixture.input, 1000, 1, &samples);
    let output = fixture.run(&["--language", "ar", "--dialect", "shami", "--segment-only"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    let units = report["units"].as_array().unwrap();
    assert_eq!(units.len(), 3);
    assert_eq!(units[0]["start_frame"], 0);
    assert_eq!(units[0]["end_frame"], 2030);
    assert_eq!(units[1]["start_frame"], 2030);
    assert_eq!(units[1]["end_frame"], 4430);
    assert_eq!(units[2]["start_frame"], 4430);
    assert_eq!(units[2]["end_frame"], 6800);
    assert_eq!(report["source"]["frame_count"], 6800);
    assert_eq!(report["source"]["language"]["dialect"], "shami");
    assert_eq!(report["quiet_intervals"][0]["start_frame"], 2000);
    assert_eq!(report["quiet_intervals"][0]["end_frame"], 2400);
}

#[test]
fn segmentation_does_not_cancel_opposite_stereo_channels_or_cut_edge_silence() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    let mut samples = vec![0; 400 * 2];
    samples.extend((0..2000).flat_map(|_| [10000, -10000]));
    samples.extend(vec![0; 400 * 2]);
    samples.extend((0..2000).flat_map(|_| [10000, -10000]));
    samples.extend(vec![0; 400 * 2]);
    write_wav(&fixture.input, 1000, 2, &samples);
    let output = fixture.run(&["--language", "ja", "--segment-only"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(report["units"].as_array().unwrap().len(), 2);
    assert_eq!(report["units"][0]["start_frame"], 0);
    assert_eq!(report["units"][0]["end_frame"], 2430);
    assert_eq!(report["units"][1]["end_frame"], 5200);
    assert_eq!(report["quiet_intervals"].as_array().unwrap().len(), 3);
}

#[test]
fn segmentation_merges_short_tail_and_retains_long_uninterrupted_audio() {
    let fixture = Fixture::new();
    let mut samples = vec![10000; 2000];
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 200]);
    write_wav(&fixture.input, 1000, 1, &samples);
    let output = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(report["units"].as_array().unwrap().len(), 1);
    assert_eq!(report["units"][0]["end_frame"], 2600);
    fs::remove_file(&fixture.output).unwrap();

    write_wav(&fixture.input, 1000, 1, &[10000; 9000]);
    let output = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(report["units"].as_array().unwrap().len(), 1);
    assert_eq!(report["units"][0]["end_frame"], 9000);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Unit 1: 0.000–9.000 seconds"));
}

#[test]
fn adaptive_chunking_rereads_target_without_replacing_existing_report() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    let mut samples = vec![10000; 2000];
    samples.extend(vec![0; 300]);
    samples.extend(vec![10000; 2000]);
    write_wav(&fixture.input, 1000, 1, &samples);
    let output = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(output.status.success());
    let original_report = fs::read(&fixture.output).unwrap();
    let report: serde_json::Value = serde_json::from_slice(&original_report).unwrap();
    assert_eq!(report["units"].as_array().unwrap().len(), 2);
    let output = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&fixture.output).unwrap(), original_report);
    fs::remove_file(&fixture.output).unwrap();
    set_chunk_target(&fixture, 6.0);
    let output = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(report["units"].as_array().unwrap().len(), 1);
    assert_eq!(report["units"][0]["end_frame"], 4300);
}

#[test]
fn segmentation_rejects_all_quiet_audio_and_bad_settings_without_output() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 1000, 1, &[0; 2000]);
    let output = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("selected audio has no nonzero signal")
    );
    assert!(!fixture.output.exists());

    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    for (key, value) in [
        ("quiet_threshold_dbfs", serde_json::json!(0)),
        ("analysis_window_ms", serde_json::json!(0)),
        ("analysis_window_ms", serde_json::json!(500)),
        ("minimum_unit_ms", serde_json::json!(9000)),
    ] {
        let mut settings = original.clone();
        settings["chunking"][key] = value;
        fs::write(&fixture.format, serde_json::to_vec(&settings).unwrap()).unwrap();
        let output = fixture.run(&["--language", "es", "--check"]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid format"));
    }
    fs::remove_file(&fixture.format).unwrap();
    let output = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot open format"));
}

#[test]
fn sequence_modes_match_the_two_requested_orders_and_keep_joined_pauses() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    let mut samples = vec![10000; 2000];
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 2000]);
    write_wav(&fixture.input, 1000, 1, &samples);
    let mut formula: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    formula["playback"]["joining"]["group_size"] = 2.into();
    for mode in ["timing-then-joining", "joining-then-timing"] {
        formula["playback"]["sequencing"] = mode.into();
        fs::write(&fixture.format, serde_json::to_vec(&formula).unwrap()).unwrap();
        let output = fixture.run(&["--language", "ar", "--dialect", "shami", "--sequence-only"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        let rows = report["plays"].as_array().unwrap();
        let actual: Vec<_> = rows
            .iter()
            .map(|row| {
                let play = &row["play"];
                (
                    play["unit_start"].as_u64().unwrap(),
                    play["unit_end_exclusive"].as_u64().unwrap(),
                    play["speed"].as_f64().unwrap(),
                )
            })
            .collect();
        let expected = if mode == "timing-then-joining" {
            vec![
                (0, 1, 0.75),
                (1, 2, 0.75),
                (0, 2, 0.75),
                (0, 1, 0.9),
                (1, 2, 0.9),
                (0, 2, 0.9),
                (0, 1, 1.0),
                (1, 2, 1.0),
                (0, 2, 1.0),
                (0, 1, 1.1),
                (1, 2, 1.1),
                (0, 2, 1.1),
            ]
        } else {
            vec![
                (0, 1, 0.75),
                (0, 1, 0.9),
                (0, 1, 1.0),
                (0, 1, 1.1),
                (1, 2, 0.75),
                (1, 2, 0.9),
                (1, 2, 1.0),
                (1, 2, 1.1),
                (0, 2, 0.75),
                (0, 2, 0.9),
                (0, 2, 1.0),
                (0, 2, 1.1),
            ]
        };
        assert_eq!(actual, expected);
        assert_eq!(report["format"], formula);
        assert_eq!(report["group_count"], 1);
        assert_eq!(report["play_count"], 12);
        let expected_seconds = 8.8 * (1.0 / 0.75 + 1.0 / 0.9 + 1.0 + 1.0 / 1.1) + 5.5;
        assert!((report["estimated_seconds"].as_f64().unwrap() - expected_seconds).abs() < 1e-9);
        for (index, row) in rows.iter().enumerate() {
            let play = &row["play"];
            assert_eq!(row["following_gap_ms"], if index == 11 { 0 } else { 500 });
            if play["unit_end_exclusive"] == 2 && play["unit_start"] == 0 {
                assert_eq!(play["source_range"]["start_frame"], 0);
                assert_eq!(play["source_range"]["end_frame"], 4400);
            }
        }
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn sequence_preserves_duplicate_speeds_and_repeats_with_a_joined_tail() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    let mut samples = vec![10000; 2000];
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 2000]);
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 2000]);
    write_wav(&fixture.input, 1000, 1, &samples);
    let mut formula: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    formula["playback"]["segments"] = serde_json::json!([1.1, 0.75, 0.75]);
    formula["playback"]["joining"]["group_size"] = 2.into();
    formula["playback"]["repetitions"] = 2.into();
    formula["playback"]["gap_ms"] = 0.into();
    for mode in ["timing-then-joining", "joining-then-timing"] {
        formula["playback"]["sequencing"] = mode.into();
        fs::write(&fixture.format, serde_json::to_vec(&formula).unwrap()).unwrap();
        let output = fixture.run(&["--language", "ja", "--sequence-only"]);
        assert!(output.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        let rows = report["plays"].as_array().unwrap();
        assert_eq!(report["group_count"], 2);
        assert_eq!(report["play_count"], 30);
        assert_eq!(rows.len(), 30);
        for pair in rows.chunks_exact(2) {
            assert_eq!(
                pair[0]["play"]["source_range"],
                pair[1]["play"]["source_range"]
            );
            assert_eq!(
                pair[0]["play"]["speed_index"],
                pair[1]["play"]["speed_index"]
            );
            assert_eq!(pair[0]["play"]["repetition_index"], 0);
            assert_eq!(pair[1]["play"]["repetition_index"], 1);
            assert_eq!(pair[0]["following_gap_ms"], 0);
        }
        let tail: Vec<_> = rows[18..]
            .iter()
            .step_by(2)
            .map(|row| {
                (
                    row["play"]["unit_start"].as_u64().unwrap(),
                    row["play"]["speed"].as_f64().unwrap(),
                )
            })
            .collect();
        let expected = if mode == "timing-then-joining" {
            vec![
                (2, 1.1),
                (1, 1.1),
                (2, 0.75),
                (1, 0.75),
                (2, 0.75),
                (1, 0.75),
            ]
        } else {
            vec![
                (2, 1.1),
                (2, 0.75),
                (2, 0.75),
                (1, 1.1),
                (1, 0.75),
                (1, 0.75),
            ]
        };
        assert_eq!(tail, expected);
        for row in &rows[18..] {
            assert_eq!(row["play"]["group_index"], 1);
            assert_eq!(row["play"]["unit_end_exclusive"], 3);
        }
        let last = rows.last().unwrap();
        assert!(
            (last["estimated_start_seconds"].as_f64().unwrap()
                + last["estimated_sound_seconds"].as_f64().unwrap()
                - report["estimated_seconds"].as_f64().unwrap())
            .abs()
                < 1e-9
        );
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn sequence_and_audio_use_sliding_joins_up_to_the_limit() {
    for (unit_count, limit, expected) in [
        (3, 1, vec![(0, 1), (1, 2), (2, 3)]),
        (3, 2, vec![(0, 1), (1, 2), (0, 2), (2, 3), (1, 3)]),
        (3, 3, vec![(0, 1), (1, 2), (0, 2), (2, 3), (0, 3)]),
        (3, 4, vec![(0, 1), (1, 2), (0, 2), (2, 3), (0, 3)]),
        (
            4,
            2,
            vec![(0, 1), (1, 2), (0, 2), (2, 3), (1, 3), (3, 4), (2, 4)],
        ),
        (
            4,
            3,
            vec![(0, 1), (1, 2), (0, 2), (2, 3), (0, 3), (3, 4), (1, 4)],
        ),
    ] {
        let mut fixture = Fixture::new();
        fixture.output = fixture.directory.join("practice.wav");
        set_chunk_target(&fixture, 2.0);
        let mut samples = Vec::new();
        for index in 0..unit_count {
            if index > 0 {
                samples.extend(vec![0; 400]);
            }
            samples.extend(vec![10000 + index * 2000; 2000]);
        }
        write_wav(&fixture.input, 1000, 1, &samples);
        let boundaries = if unit_count == 3 {
            vec![0, 2030, 4430, 6800]
        } else {
            vec![0, 2030, 4430, 6830, 9200]
        };
        let mut expected_audio = Vec::new();
        for (index, &(first, end)) in expected.iter().enumerate() {
            if index > 0 {
                expected_audio.extend(vec![0.0; 500]);
            }
            expected_audio.extend(
                samples[boundaries[first]..boundaries[end]]
                    .iter()
                    .map(|&sample| f32::from(sample) / 32768.0),
            );
        }
        for mode in ["timing-then-joining", "joining-then-timing"] {
            set_recipe(&fixture, &[1.0], mode, 1, 500, limit);
            let output = fixture.run(&["--language", "es", "--sequence-only"]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: serde_json::Value =
                serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
            let rows = report["plays"].as_array().unwrap();
            let membership: Vec<_> = rows
                .iter()
                .map(|row| {
                    (
                        row["play"]["unit_start"].as_u64().unwrap() as usize,
                        row["play"]["unit_end_exclusive"].as_u64().unwrap() as usize,
                    )
                })
                .collect();
            assert_eq!(
                membership, expected,
                "{unit_count} units, limit {limit}, {mode}"
            );
            assert_eq!(report["play_count"], expected.len());
            assert_eq!(rows.last().unwrap()["following_gap_ms"], 0);
            assert!(
                (report["estimated_seconds"].as_f64().unwrap()
                    - expected_audio.len() as f64 / 1000.0)
                    .abs()
                    < 1e-9
            );
            fs::remove_file(&fixture.output).unwrap();
            let output = fixture.run(&["--language", "es"]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(read_audio(&fixture.output), expected_audio);
            fs::remove_file(&fixture.output).unwrap();
        }
    }
}

#[test]
fn sequence_rejects_overflow_and_keeps_existing_output() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 1000, 1, &[10000; 2000]);
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    let mut formula = original.clone();
    formula["playback"]["repetitions"] = serde_json::json!(usize::MAX);
    fs::write(&fixture.format, serde_json::to_vec(&formula).unwrap()).unwrap();
    let output = fixture.run(&["--language", "es", "--sequence-only"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("play count overflow"));
    assert!(!fixture.output.exists());
    formula = original.clone();
    formula["playback"]["segments"] = serde_json::json!([1e-308]);
    fs::write(&fixture.format, serde_json::to_vec(&formula).unwrap()).unwrap();
    let output = fixture.run(&["--language", "es", "--sequence-only"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duration is not representable"));
    assert!(!fixture.output.exists());
    fs::write(&fixture.format, serde_json::to_vec(&original).unwrap()).unwrap();
    fs::write(&fixture.output, b"existing result").unwrap();
    let output = fixture.run(&["--language", "es", "--sequence-only"]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&fixture.output).unwrap(), b"existing result");
}

#[test]
fn input_window_decoder_exports_exact_stereo_samples_with_absolute_metadata() {
    let fixture = Fixture::new();
    let samples: Vec<i16> = (0..20000)
        .map(|index| ((index % 251) as i16 - 125) * 100)
        .collect();
    write_wav(&fixture.input, 1000, 2, &samples);
    let original = fs::read(&fixture.input).unwrap();
    let output = fixture.run(&[
        "--language",
        "ar",
        "--dialect",
        "shami",
        "--decode-only",
        "--start",
        "00:00:01.25",
        "--end",
        "2.75",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pcm = fs::read(&fixture.output).unwrap();
    assert_eq!(pcm.len(), 1500 * 2 * 4);
    for (bytes, source) in pcm.chunks_exact(4).zip(&samples[2500..5500]) {
        assert!(
            (f32::from_le_bytes(bytes.try_into().unwrap()) - f32::from(*source) / 32768.0).abs()
                < 1e-7
        );
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(format!("{}.json", fixture.output.display())).unwrap())
            .unwrap();
    assert_eq!(metadata["frame_count"], 10000);
    assert_eq!(metadata["pcm_frame_count"], 1500);
    assert_eq!(
        metadata["selected_range"],
        serde_json::json!({"start_frame":1250,"end_frame":2750})
    );
    assert_eq!(
        metadata["requested_window"]["start"],
        serde_json::json!({"secs":1,"nanos":250000000})
    );
    assert_eq!(fs::read(&fixture.input).unwrap(), original);
}

#[test]
fn omitted_window_bounds_select_beginning_or_end_without_changing_whole_source_metadata() {
    for (arguments, first, end) in [
        (vec![], 0, 3000),
        (vec!["--start", "1.25"], 1250, 3000),
        (vec!["--end", "00:02.5"], 0, 2500),
        (vec!["--start", "0", "--end", "3"], 0, 3000),
    ] {
        let fixture = Fixture::new();
        write_wav(&fixture.input, 1000, 1, &[10000; 3000]);
        let mut flags = vec!["--language", "es", "--decode-only"];
        flags.extend(arguments);
        let output = fixture.run(&flags);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: serde_json::Value = serde_json::from_slice(
            &fs::read(format!("{}.json", fixture.output.display())).unwrap(),
        )
        .unwrap();
        assert_eq!(
            metadata["selected_range"],
            serde_json::json!({"start_frame":first,"end_frame":end})
        );
        assert_eq!(metadata["frame_count"], 3000);
        assert_eq!(metadata["pcm_frame_count"], end - first);
        assert_eq!(
            fs::metadata(&fixture.output).unwrap().len(),
            (end - first) * 4
        );
    }
}

#[test]
fn fractional_bounds_choose_samples_in_the_half_open_window() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 1000, 1, &[1000, 2000, 3000, 4000, 5000]);
    let output = fixture.run(&[
        "--language",
        "ja",
        "--decode-only",
        "--start",
        "0.0005",
        "--end",
        "0.0025",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pcm = fs::read(&fixture.output).unwrap();
    assert_eq!(pcm.len(), 8);
    assert_eq!(
        f32::from_le_bytes(pcm[0..4].try_into().unwrap()),
        2000.0 / 32768.0
    );
    assert_eq!(
        f32::from_le_bytes(pcm[4..8].try_into().unwrap()),
        3000.0 / 32768.0
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(format!("{}.json", fixture.output.display())).unwrap())
            .unwrap();
    assert_eq!(
        metadata["selected_range"],
        serde_json::json!({"start_frame":1,"end_frame":3})
    );
}

#[test]
fn window_applies_before_segmentation_and_sequence_keeps_source_offsets() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    let mut samples = vec![10000; 2000];
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 2000]);
    samples.extend(vec![0; 400]);
    samples.extend(vec![10000; 2000]);
    write_wav(&fixture.input, 1000, 1, &samples);
    for mode in ["--segment-only", "--sequence-only"] {
        let output = fixture.run(&[
            "--language",
            "ar",
            "--dialect",
            "shami",
            mode,
            "--start",
            "1",
            "--end",
            "5",
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        let segmentation = if mode == "--sequence-only" {
            &report["segmentation"]
        } else {
            &report
        };
        assert_eq!(segmentation["source"]["frame_count"], 6800);
        assert_eq!(
            segmentation["source"]["selected_range"],
            serde_json::json!({"start_frame":1000,"end_frame":5000})
        );
        let units = segmentation["units"].as_array().unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0]["start_frame"], 1000);
        assert_eq!(units[0]["end_frame"], 2370);
        assert_eq!(units[1]["start_frame"], 2370);
        assert_eq!(units[1]["end_frame"], 5000);
        if mode == "--sequence-only" {
            let rows = report["plays"].as_array().unwrap();
            assert_eq!(rows[0]["estimated_start_seconds"], 0.0);
            assert_eq!(report["play_count"], 12);
            for row in rows {
                let range = &row["play"]["source_range"];
                assert!(range["start_frame"].as_u64().unwrap() >= 1000);
                assert!(range["end_frame"].as_u64().unwrap() <= 5000);
            }
            assert_eq!(
                rows[2]["play"]["source_range"],
                serde_json::json!({"start_frame":1000,"end_frame":5000})
            );
        }
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn window_does_not_use_active_audio_outside_the_selected_interval() {
    let fixture = Fixture::new();
    let mut samples = vec![10000; 2000];
    samples.extend(vec![0; 2000]);
    samples.extend(vec![10000; 2000]);
    write_wav(&fixture.input, 1000, 1, &samples);
    let output = fixture.run(&[
        "--language",
        "es",
        "--segment-only",
        "--start",
        "2",
        "--end",
        "4",
    ]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("selected audio has no nonzero signal")
    );
    assert!(!fixture.output.exists());
}

#[test]
fn invalid_windows_never_publish_output() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 1000, 1, &[10000; 2000]);
    for bounds in [
        vec!["--start", "NaN"],
        vec!["--end", "inf"],
        vec!["--start", "-1"],
        vec!["--start", "1:60"],
        vec!["--end", "1:60:00"],
        vec!["--end", "0.1234567890"],
        vec!["--start", "2", "--end", "1"],
        vec!["--start", "1", "--end", "1"],
        vec!["--start", "3"],
        vec!["--end", "3"],
        vec!["--start", "2"],
        vec!["--end", "0"],
        vec!["--start", "0.0001", "--end", "0.0002"],
        vec!["--start", "18446744073709551616"],
        vec!["--end", "18446744073709551615"],
    ] {
        let mut flags = vec!["--language", "es", "--decode-only"];
        flags.extend(bounds.clone());
        let output = fixture.run(&flags);
        assert!(!output.status.success(), "accepted {bounds:?}");
        assert!(!fixture.output.exists());
        assert!(!PathBuf::from(format!("{}.json", fixture.output.display())).exists());
    }
}

#[test]
fn decoder_never_overwrites_input_or_existing_output() {
    let mut fixture = Fixture::new();
    write_wav(&fixture.input, 32000, 1, &[1000; 4096]);
    fs::write(&fixture.output, b"existing output").unwrap();
    let output = fixture.run(&["--language", "es", "--decode-only"]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&fixture.output).unwrap(), b"existing output");

    fixture.output = fixture.input.clone();
    let original = fs::read(&fixture.input).unwrap();
    let output = fixture.run(&["--language", "es", "--decode-only"]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&fixture.input).unwrap(), original);
}

fn read_audio(path: &std::path::Path) -> Vec<f32> {
    let result = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(path)
        .args(["-c:a", "pcm_f32le", "-f", "f32le", "pipe:1"])
        .output()
        .expect("real ffmpeg is required for rendering tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    result
        .stdout
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect()
}

fn tone(rate: u32, frequency: f64, frames: usize) -> Vec<i16> {
    (0..frames)
        .map(|frame| {
            (12000.0 * (std::f64::consts::TAU * frequency * frame as f64 / f64::from(rate)).sin())
                as i16
        })
        .collect()
}

fn measured_frequency(samples: &[f32], rate: u32) -> f64 {
    let begin = samples.len() / 4;
    let end = samples.len() * 3 / 4;
    let crossings: Vec<_> = (begin..end - 1)
        .filter(|&index| samples[index] <= 0.0 && samples[index + 1] > 0.0)
        .collect();
    assert!(crossings.len() > 10);
    (crossings.len() - 1) as f64 * f64::from(rate)
        / (crossings.last().unwrap() - crossings[0]) as f64
}

fn set_recipe(
    fixture: &Fixture,
    speeds: &[f64],
    order: &str,
    repetitions: usize,
    gap: u64,
    group_size: usize,
) {
    let mut format: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    format["playback"] = serde_json::json!({
        "sequencing":order,"segments":speeds,"joining":{"group_size":group_size},"repetitions":repetitions,"gap_ms":gap
    });
    fs::write(&fixture.format, serde_json::to_vec(&format).unwrap()).unwrap();
}

#[test]
fn tempo_changes_duration_without_changing_pitch_and_preserves_stereo() {
    for (rate, channels, speed) in [
        (32000, 1, 0.75),
        (32000, 1, 0.9),
        (32000, 1, 1.0),
        (32000, 1, 1.1),
        (32000, 1, 0.25),
        (32000, 1, 4.0),
        (32000, 1, 3.0),
        (48000, 2, 0.9),
        (22050, 1, 1.1),
    ] {
        let mut fixture = Fixture::new();
        fixture.output = fixture.directory.join("practice.wav");
        let left = tone(rate, 440.0, (rate * 3) as usize);
        let right = tone(rate, 660.0, left.len());
        let samples: Vec<_> = left
            .iter()
            .zip(&right)
            .flat_map(|(&a, &b)| if channels == 2 { vec![a, b] } else { vec![a] })
            .collect();
        write_wav(&fixture.input, rate, channels, &samples);
        set_recipe(&fixture, &[speed], "timing-then-joining", 1, 0, 3);
        let result = fixture.run(&["--language", "ja"]);
        assert!(
            result.status.success(),
            "speed {speed}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let output = read_audio(&fixture.output);
        let duration = output.len() as f64 / f64::from(channels) / f64::from(rate);
        assert!(
            (duration - 3.0 / speed).abs() < 0.12,
            "speed {speed}: {duration}"
        );
        let left_out: Vec<_> = output.iter().step_by(channels as usize).copied().collect();
        assert!(
            (measured_frequency(&left_out, rate) - 440.0).abs() < 2.0,
            "speed {speed}"
        );
        if channels == 2 {
            let right_out: Vec<_> = output.iter().skip(1).step_by(2).copied().collect();
            assert!((measured_frequency(&right_out, rate) - 660.0).abs() < 2.0);
        }
        if speed == 1.0 {
            assert_eq!(output.len(), samples.len());
            for (&decoded, &sample) in output.iter().zip(&samples) {
                assert_eq!(decoded, f32::from(sample) / 32768.0);
            }
        }
    }
}

#[test]
fn rendering_keeps_exact_joins_repeats_group_gaps_and_selected_frames_at_normal_speed() {
    let mut fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    fixture.output = fixture.directory.join("practice.wav");
    let rate = 8000;
    let mut samples = vec![10000; 2000];
    samples.extend(tone(rate, 440.0, 16000));
    samples.extend(vec![0; 3200]);
    samples.extend(tone(rate, 660.0, 16000));
    samples.extend(vec![0; 3200]);
    samples.extend(tone(rate, 880.0, 16000));
    samples.extend(vec![10000; 2000]);
    write_wav(&fixture.input, rate, 1, &samples);
    set_recipe(&fixture, &[1.0, 1.0], "joining-then-timing", 2, 125, 2);
    let arguments = ["--language", "ar", "--start", "0.25", "--end", "7.05"];
    let mut sequence_flags = arguments.to_vec();
    sequence_flags.push("--sequence-only");
    let sequence = fixture.run(&sequence_flags);
    assert!(
        sequence.status.success(),
        "{}",
        String::from_utf8_lossy(&sequence.stderr)
    );
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    fs::remove_file(&fixture.output).unwrap();
    let result = fixture.run(&arguments);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = read_audio(&fixture.output);
    let mut expected = Vec::new();
    let plays = plan["plays"].as_array().unwrap();
    for (index, row) in plays.iter().enumerate() {
        let span = &row["play"]["source_range"];
        let start = span["start_frame"].as_u64().unwrap() as usize;
        let end = span["end_frame"].as_u64().unwrap() as usize;
        expected.extend(
            samples[start..end]
                .iter()
                .map(|&sample| f32::from(sample) / 32768.0),
        );
        if index + 1 < plays.len() {
            expected.extend(vec![0.0; 1000]);
        }
    }
    assert_eq!(plan["group_count"], 2);
    assert_eq!(output, expected);
}

#[test]
fn altered_speed_audio_follows_both_sequence_orders_and_retains_joined_pauses() {
    for order in ["timing-then-joining", "joining-then-timing"] {
        let mut fixture = Fixture::new();
        set_chunk_target(&fixture, 2.2);
        fixture.output = fixture.directory.join("practice.wav");
        let rate = 16000;
        let mut samples = tone(rate, 440.0, 32000);
        samples.extend(vec![0; 6400]);
        samples.extend(tone(rate, 660.0, 32000));
        write_wav(&fixture.input, rate, 1, &samples);
        set_recipe(&fixture, &[0.75, 1.1], order, 1, 250, 3);
        let result = fixture.run(&["--language", "es"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let output = read_audio(&fixture.output);
        // A/B source ranges are 2.2 seconds; AB retains the original 0.4-second pause.
        let spans = if order == "timing-then-joining" {
            vec![
                (0.75, 440.0, false),
                (0.75, 660.0, false),
                (0.75, 440.0, true),
                (1.1, 440.0, false),
                (1.1, 660.0, false),
                (1.1, 440.0, true),
            ]
        } else {
            vec![
                (0.75, 440.0, false),
                (1.1, 440.0, false),
                (0.75, 660.0, false),
                (1.1, 660.0, false),
                (0.75, 440.0, true),
                (1.1, 440.0, true),
            ]
        };
        let mut offset = 0.0;
        for (speed, frequency, joined) in spans {
            let first = ((offset + 0.5 / speed) * f64::from(rate)) as usize;
            let last = first + rate as usize / 2;
            assert!(
                (measured_frequency(&output[first..last], rate) - frequency).abs() < 2.0,
                "{order} at {offset}"
            );
            if joined {
                let first = ((offset + 3.0 / speed) * f64::from(rate)) as usize;
                assert!(
                    (measured_frequency(&output[first..first + rate as usize / 3], rate) - 660.0)
                        .abs()
                        < 2.0
                );
                let pause = ((offset + 2.2 / speed) * f64::from(rate)) as usize;
                let radius = rate as usize / 50;
                assert!(
                    output[pause - radius..pause + radius]
                        .iter()
                        .all(|value| value.abs() < 0.0001),
                    "missing joined pause for {order}"
                );
            }
            offset += if joined { 4.4 / speed } else { 2.2 / speed };
            offset += 0.25;
        }
        assert!((output.len() as f64 / f64::from(rate) - (offset - 0.25)).abs() < 0.3);
    }
}

#[test]
fn render_rejects_existing_outputs_unsupported_formats_and_mp3_conversion() {
    let mut fixture = Fixture::new();
    write_wav(&fixture.input, 1000, 1, &[10000; 2000]);
    let original = fs::read(&fixture.input).unwrap();
    for path in [
        fixture.output.clone(),
        fixture.directory.join("practice.ogg"),
    ] {
        fixture.output = path;
        let result = fixture.run(&["--language", "es"]);
        assert!(!result.status.success());
        assert!(!fixture.output.exists());
    }
    fixture.output = fixture.input.clone();
    assert!(!fixture.run(&["--language", "es"]).status.success());
    assert_eq!(fs::read(&fixture.input).unwrap(), original);
    fixture.output = fixture.directory.join("existing.wav");
    fs::write(&fixture.output, b"keep this").unwrap();
    assert!(!fixture.run(&["--language", "es"]).status.success());
    assert_eq!(fs::read(&fixture.output).unwrap(), b"keep this");
    fixture.output = fixture
        .directory
        .join("missing-parent")
        .join("practice.wav");
    set_recipe(&fixture, &[1.0], "timing-then-joining", 1, 0, 3);
    assert!(!fixture.run(&["--language", "es"]).status.success());
    assert!(!fixture.output.exists());
}

#[cfg(unix)]
#[test]
fn backend_tempo_and_encoder_failures_leave_no_published_track() {
    use std::os::unix::fs::PermissionsExt;
    let located = Command::new("/bin/sh")
        .args(["-c", "command -v ffmpeg"])
        .output()
        .unwrap();
    assert!(located.status.success());
    let real_ffmpeg = String::from_utf8(located.stdout)
        .unwrap()
        .trim()
        .replace('\'', "'\\''");
    for (trigger, diagnostic) in [
        ("-af", "tempo processing"),
        ("libmp3lame", "study encoding"),
    ] {
        let fixture = Fixture::new();
        write_wav(&fixture.input, 32000, 1, &tone(32000, 440.0, 64000));
        let original = fs::read(&fixture.input).unwrap();
        let wrapper = fixture.directory.join("ffmpeg");
        fs::write(&wrapper, format!("#!/bin/sh\nfor arg in \"$@\"; do\n  if [ \"$arg\" = '{trigger}' ]; then\n    printf 'partial output'\n    printf 'deliberate backend failure' >&2\n    exit 42\n  fi\ndone\nexec '{real_ffmpeg}' \"$@\"\n")).unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    fixture.directory.display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .arg(&fixture.input)
            .arg("--format")
            .arg(&fixture.format)
            .arg("--output")
            .arg(&fixture.output)
            .args(["--language", "ar"])
            .output()
            .unwrap();
        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains(diagnostic), "{error}");
        assert!(error.contains("deliberate backend failure"), "{error}");
        assert!(!fixture.output.exists());
        assert!(!String::from_utf8_lossy(&result.stdout).contains("Study audio saved:"));
        assert_eq!(fs::read(&fixture.input).unwrap(), original);
    }
}

#[test]
fn too_short_audio_for_tempo_backend_fails_without_publishing_empty_plays() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 8000, 1, &[10000]);
    set_recipe(&fixture, &[2.0], "timing-then-joining", 1, 0, 3);
    let result = fixture.run(&["--language", "es"]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("empty or incomplete PCM"));
    assert!(!fixture.output.exists());
}

#[test]
fn check_requires_complete_external_format_but_no_media_backend() {
    let fixture = Fixture::new();
    let binary = fixture.installed_binary();
    let caller = fixture.directory.join("caller");
    fs::create_dir(&caller).unwrap();
    fs::copy(&fixture.input, caller.join("recording.wav")).unwrap();
    let format = fixture.directory.join("formats/default.json");
    let run = |extra: &[&str]| {
        Command::new(&binary)
            .current_dir(&caller)
            .env("PATH", &caller)
            .args([
                "recording.wav",
                "--language",
                "ar",
                "--dialect",
                "masri",
                "--check",
                "--start",
                "01:35",
                "--end",
                "01:45",
                "--output",
                "unused.mp3",
            ])
            .args(extra)
            .output()
            .unwrap()
    };
    let result = run(&[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!caller.join("unused.mp3").exists());
    for contents in [
        None,
        Some(b"{}".as_slice()),
        Some(b"invalid JSON".as_slice()),
    ] {
        if let Some(contents) = contents {
            fs::write(&format, contents).unwrap();
        } else {
            fs::remove_file(&format).unwrap();
        }
        let result = run(&[]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("format"));
        assert!(!caller.join("unused.mp3").exists());
    }
    fs::copy(&fixture.format, caller.join("custom.json")).unwrap();
    assert!(run(&["--format", "custom.json"]).status.success());
    fs::remove_file(&format).unwrap();
    assert!(run(&["--format", "custom.json"]).status.success());
}

#[test]
fn installed_cli_renders_from_another_directory_and_rereads_external_recipe() {
    let fixture = Fixture::new();
    let binary = fixture.installed_binary();
    let caller = fixture.directory.join("caller");
    fs::create_dir(&caller).unwrap();
    let samples = tone(16000, 440.0, 32000);
    write_wav(&caller.join("recording.wav"), 16000, 1, &samples);
    let original = fs::read(caller.join("recording.wav")).unwrap();
    let formula = fixture.directory.join("formats/default.json");
    let mut initial: serde_json::Value =
        serde_json::from_slice(&fs::read(&formula).unwrap()).unwrap();
    initial["playback"] = serde_json::json!({"sequencing":"joining-then-timing", "segments":[1.0], "joining":{"group_size":3}, "repetitions":1, "gap_ms":100});
    fs::write(&formula, serde_json::to_vec(&initial).unwrap()).unwrap();
    let run = |output: &str, language: &[&str]| {
        Command::new(&binary)
            .current_dir(&caller)
            .args([
                "recording.wav",
                "--start",
                "0.25",
                "--end",
                "1.75",
                "--output",
                output,
            ])
            .args(language)
            .output()
            .unwrap()
    };
    let result = run("first.wav", &["--language", "ar", "--dialect", "masri"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let expected: Vec<_> = samples[4000..28000]
        .iter()
        .map(|&value| f32::from(value) / 32768.0)
        .collect();
    assert_eq!(read_audio(&caller.join("first.wav")), expected);
    let mut recipe: serde_json::Value =
        serde_json::from_slice(&fs::read(&formula).unwrap()).unwrap();
    recipe["playback"]["repetitions"] = 2.into();
    fs::write(&formula, serde_json::to_vec(&recipe).unwrap()).unwrap();
    for (output, language) in [("japanese.wav", "ja"), ("spanish.wav", "es")] {
        let result = run(output, &["--language", language]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut repeated = expected.clone();
        repeated.extend(vec![0.0; 1600]);
        repeated.extend(&expected);
        assert_eq!(read_audio(&caller.join(output)), repeated);
    }
    assert!(!fixture.directory.join("first.wav").exists());
    assert_eq!(fs::read(caller.join("recording.wav")).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn unwritable_destination_and_missing_encoder_do_not_publish_audio() {
    use std::os::unix::fs::PermissionsExt;
    let mut fixture = Fixture::new();
    write_wav(&fixture.input, 16000, 1, &tone(16000, 440.0, 32000));
    let locked = fixture.directory.join("locked");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o500)).unwrap();
    fixture.output = locked.join("practice.wav");
    let result = fixture.run(&["--language", "ja"]);
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(!result.status.success());
    assert!(!fixture.output.exists());
    assert!(!String::from_utf8_lossy(&result.stdout).contains("Study audio saved:"));

    fixture.output = fixture.directory.join("practice.mp3");
    let located = Command::new("/bin/sh")
        .args(["-c", "command -v ffmpeg"])
        .output()
        .unwrap();
    assert!(located.status.success());
    let real_ffmpeg = String::from_utf8(located.stdout)
        .unwrap()
        .trim()
        .replace('\'', "'\\''");
    let wrapper = fixture.directory.join("ffmpeg");
    fs::write(&wrapper, format!("#!/bin/sh\nremaining=$#\nwhile [ \"$remaining\" -gt 0 ]; do\n  arg=$1\n  shift\n  if [ \"$arg\" = libmp3lame ]; then arg=language_jutsu_missing_encoder; fi\n  set -- \"$@\" \"$arg\"\n  remaining=$((remaining-1))\ndone\nexec '{real_ffmpeg}' \"$@\"\n")).unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                fixture.directory.display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .arg(&fixture.input)
        .arg("--format")
        .arg(&fixture.format)
        .arg("--output")
        .arg(&fixture.output)
        .args(["--language", "es"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("study encoding failed"), "{error}");
    assert!(error.contains("Unknown encoder"), "{error}");
    assert!(!fixture.output.exists());
}

fn set_chunk_target(fixture: &Fixture, seconds: f64) {
    let mut settings: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    settings["chunking"]["chunk_target_size_seconds"] = seconds.into();
    fs::write(&fixture.format, serde_json::to_vec(&settings).unwrap()).unwrap();
}

#[test]
fn chunk_target_defaults_to_six_and_cli_selects_two_or_ten_with_identical_pauses() {
    let fixture = Fixture::new();
    let mut samples = vec![10000; 24000];
    for center in (2000..24000).step_by(2000) {
        samples[center - 200..center + 200].fill(0);
    }
    write_wav(&fixture.input, 1000, 1, &samples);
    let original_settings = fs::read(&fixture.format).unwrap();
    let mut quiet = None;
    for (target, boundaries) in [
        (None, vec![0, 6000, 12000, 18000, 24000]),
        (Some("2"), (0..=24000).step_by(2000).collect()),
        (Some("10"), vec![0, 10000, 20000, 24000]),
    ] {
        let mut flags = vec!["--language", "ar", "--segment-only"];
        if let Some(target) = target {
            flags.extend(["--chunk-target-size-seconds", target]);
        }
        let result = fixture.run(&flags);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        let units = report["units"].as_array().unwrap();
        assert_eq!(units.len(), boundaries.len() - 1);
        for (unit, pair) in units.iter().zip(boundaries.windows(2)) {
            assert_eq!(unit["start_frame"], pair[0]);
            assert_eq!(unit["end_frame"], pair[1]);
        }
        assert_eq!(
            report["settings"]["chunk_target_size_seconds"]
                .as_f64()
                .unwrap(),
            target.map(|value| value.parse().unwrap()).unwrap_or(6.0)
        );
        if let Some(prior) = &quiet {
            assert_eq!(prior, &report["quiet_intervals"]);
        } else {
            quiet = Some(report["quiet_intervals"].clone());
        }
        assert_eq!(fs::read(&fixture.format).unwrap(), original_settings);
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn target_uses_nearest_safe_point_and_preserves_a_short_tail() {
    let fixture = Fixture::new();
    // Approach the first target from inside the earlier pause. The next target
    // lies inside the pause around 4.0; neither cut needs to use a midpoint.
    let mut samples = vec![10000; 6300];
    for center in [1800, 2400, 3600, 4000, 6000] {
        let radius = if center == 3600 || center == 4000 {
            140
        } else {
            200
        };
        samples[center - radius..center + radius].fill(0);
    }
    write_wav(&fixture.input, 1000, 1, &samples);
    let result = fixture.run(&[
        "--language",
        "ja",
        "--chunk-target-size-seconds",
        "2",
        "--segment-only",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    let ranges: Vec<_> = report["units"]
        .as_array()
        .unwrap()
        .iter()
        .map(|unit| {
            (
                unit["start_frame"].as_u64().unwrap(),
                unit["end_frame"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(ranges, vec![(0, 1970), (1970, 3970), (3970, 6300)]);
}

#[test]
fn adaptive_pauses_follow_recording_levels_without_manual_settings() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 3.0);
    let mut expected = None;
    for (speech, background) in [(12000, 900), (1200, 90), (30000, 2250), (300, 22)] {
        let mut samples = vec![speech; 18000];
        for center in [3000, 6000, 9000, 12000, 15000] {
            samples[center - 80..center + 80].fill(background);
        }
        write_wav(&fixture.input, 1000, 1, &samples);
        let result = fixture.run(&["--language", "es", "--segment-only"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        let ranges: Vec<_> = report["units"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                (
                    row["start_frame"].as_u64().unwrap(),
                    row["end_frame"].as_u64().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            ranges,
            vec![
                (0, 3000),
                (3000, 6000),
                (6000, 9000),
                (9000, 12000),
                (12000, 15000),
                (15000, 18000)
            ]
        );
        if let Some(expected) = &expected {
            assert_eq!(&ranges, expected);
        } else {
            expected = Some(ranges);
        }
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn target_json_is_required_and_positive_and_override_supports_fractional_seconds() {
    let fixture = Fixture::new();
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    for bad in [
        serde_json::Value::Null,
        serde_json::json!(0),
        serde_json::json!(-2),
        serde_json::json!("6"),
        serde_json::json!(1e300),
    ] {
        let mut settings = original.clone();
        settings["chunking"]["chunk_target_size_seconds"] = bad;
        fs::write(&fixture.format, serde_json::to_vec(&settings).unwrap()).unwrap();
        let result = fixture.run(&["--language", "ja", "--check"]);
        assert!(!result.status.success());
        assert!(!fixture.output.exists());
    }
    let mut missing = original.clone();
    missing["chunking"]
        .as_object_mut()
        .unwrap()
        .remove("chunk_target_size_seconds");
    fs::write(&fixture.format, serde_json::to_vec(&missing).unwrap()).unwrap();
    let result = fixture.run(&["--language", "es", "--check"]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("chunk_target_size_seconds"));
    fs::write(&fixture.format, serde_json::to_vec(&original).unwrap()).unwrap();
    for bad in ["0", "-1", "NaN", "inf", "1e300"] {
        let result = fixture.run(&[
            "--language",
            "ja",
            "--chunk-target-size-seconds",
            bad,
            "--check",
        ]);
        assert!(!result.status.success(), "accepted {bad}");
        assert!(!fixture.output.exists());
    }
    set_chunk_target(&fixture, 10.0);
    let result = fixture.run(&[
        "--language",
        "ar",
        "--chunk-target-size-seconds",
        "2.5",
        "--check",
    ]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("Chunk target size: 2.5 seconds"));
}

#[test]
fn selected_window_target_changes_rendered_joins_and_preserves_exact_audio() {
    let mut fixture = Fixture::new();
    fixture.output = fixture.directory.join("practice.wav");
    let rate = 8000;
    let mut samples = tone(rate, 440.0, 88000);
    for center in [20000, 36000, 52000, 68000] {
        samples[center - 1600..center + 1600].fill(0);
    }
    write_wav(&fixture.input, rate, 1, &samples);
    set_recipe(&fixture, &[1.0], "timing-then-joining", 1, 125, 3);
    for target in ["2", "10"] {
        let flags = [
            "--language",
            "ar",
            "--start",
            "0.5",
            "--end",
            "10.5",
            "--chunk-target-size-seconds",
            target,
        ];
        let mut sequence = flags.to_vec();
        sequence.push("--sequence-only");
        let result = fixture.run(&sequence);
        assert!(result.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        assert_eq!(
            report["segmentation"]["units"].as_array().unwrap().len(),
            if target == "2" { 5 } else { 1 }
        );
        fs::remove_file(&fixture.output).unwrap();
        let result = fixture.run(&flags);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut expected = Vec::new();
        let plays = report["plays"].as_array().unwrap();
        for (index, row) in plays.iter().enumerate() {
            let span = &row["play"]["source_range"];
            let start = span["start_frame"].as_u64().unwrap() as usize;
            let end = span["end_frame"].as_u64().unwrap() as usize;
            assert!(start >= 4000 && end <= 84000);
            expected.extend(
                samples[start..end]
                    .iter()
                    .map(|&sample| f32::from(sample) / 32768.0),
            );
            if index + 1 < plays.len() {
                expected.extend(vec![0.0; 1000]);
            }
        }
        assert_eq!(read_audio(&fixture.output), expected);
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn every_mode_requires_both_sections_of_the_same_format_file() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 16000, 1, &tone(16000, 440.0, 32000));
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    for missing in ["chunking", "playback"] {
        let mut format = original.clone();
        format.as_object_mut().unwrap().remove(missing);
        fs::write(&fixture.format, serde_json::to_vec(&format).unwrap()).unwrap();
        for mode in [
            Some("--check"),
            Some("--decode-only"),
            Some("--segment-only"),
            Some("--sequence-only"),
            None,
        ] {
            let mut flags = vec!["--language", "ja"];
            if let Some(mode) = mode {
                flags.push(mode);
            }
            let result = fixture.run(&flags);
            assert!(
                !result.status.success(),
                "accepted missing {missing} in {mode:?}"
            );
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(
                error.contains("invalid format") && error.contains(missing),
                "{error}"
            );
            assert!(!fixture.output.exists());
            assert!(!PathBuf::from(format!("{}.json", fixture.output.display())).exists());
        }
    }
    let mut extra = original.clone();
    extra["unexpected"] = true.into();
    fs::write(&fixture.format, serde_json::to_vec(&extra).unwrap()).unwrap();
    assert!(
        !fixture
            .run(&["--language", "ja", "--check"])
            .status
            .success()
    );
    fs::write(&fixture.format, serde_json::to_vec(&original).unwrap()).unwrap();
    let result = fixture.run(&["--language", "ja", "--decode-only"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn cli_exposes_one_format_path_and_rejects_old_split_options() {
    let fixture = Fixture::new();
    for old in ["--formula", "--segmentation"] {
        let result = fixture.run(&["--language", "es", "--check", old, "obsolete.json"]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("unexpected argument"));
        assert!(!fixture.output.exists());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("--format"));
    assert!(!help.contains("--formula"));
    assert!(!help.contains("--segmentation"));
}

#[test]
fn three_second_target_can_use_a_brief_speaker_pause_before_a_later_sentence_pause() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 3.0);
    // An 80 ms quiet gap at the target used to be rejected by the 250 ms default,
    // leaving the first eligible cut at 6.55 seconds instead.
    let mut samples = vec![10000; 8000];
    samples[2960..3040].fill(500);
    samples[6420..6680].fill(500);
    write_wav(&fixture.input, 1000, 1, &samples);
    let result = fixture.run(&["--language", "ar", "--dialect", "shami", "--segment-only"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    let units = report["units"].as_array().unwrap();
    assert_eq!(units[0]["start_frame"], 0);
    assert_eq!(units[0]["end_frame"], 3000);
    assert_eq!(units[1]["end_frame"], 6450);
    assert_eq!(units.last().unwrap()["end_frame"], 8000);
    assert!(
        units
            .windows(2)
            .all(|pair| pair[0]["end_frame"] == pair[1]["start_frame"])
    );
    assert_eq!(
        report["settings"]["chunk_target_size_seconds"].as_f64(),
        Some(3.0)
    );
}

#[test]
fn automatic_detection_handles_changing_recording_gain_and_background() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 3.0);
    for (transition, speech, background) in [
        (12000, 600, 45),
        (12370, 600, 45),
        (10700, 600, 45),
        (12370, 12000, 2400),
    ] {
        let mut samples = vec![12000; 24000];
        samples[transition..].fill(speech);
        for center in (3000..24000).step_by(3000) {
            for (index, value) in samples[center - 80..center + 80].iter_mut().enumerate() {
                *value = if center - 80 + index < transition {
                    900
                } else {
                    background
                };
            }
        }
        write_wav(&fixture.input, 1000, 1, &samples);
        let result = fixture.run(&["--language", "ar", "--segment-only"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        let units = report["units"].as_array().unwrap();
        let ends: Vec<_> = units
            .iter()
            .map(|unit| unit["end_frame"].as_u64().unwrap())
            .collect();
        assert_eq!(
            ends.len(),
            8,
            "transition {transition}, speech {speech}, noise {background}"
        );
        // When gain changes inside a gap, its quieter half can be the best cut.
        // Every cut must still land inside the independently known quiet interval.
        for (&cut, center) in ends[..7].iter().zip((3000..24000).step_by(3000)) {
            assert!(
                (center - 80..=center + 80).contains(&cut),
                "cut {cut} at transition {transition}"
            );
        }
        assert_eq!(ends[7], 24000);
        assert_eq!(units[0]["start_frame"], 0);
        assert!(
            units
                .windows(2)
                .all(|pair| pair[0]["end_frame"] == pair[1]["start_frame"])
        );
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn confidence_favors_a_nearby_clear_pause_but_keeps_target_over_a_distant_long_pause() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 3.0);
    let mut samples = vec![10000; 10000];
    samples[2990..3010].fill(0);
    samples[3100..3300].fill(0);
    samples[6400..6700].fill(0);
    write_wav(&fixture.input, 1000, 1, &samples);
    let result = fixture.run(&["--language", "ja", "--segment-only"]);
    assert!(result.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(report["units"][0]["end_frame"], 3130);
}

#[test]
fn target_dips_do_not_displace_sustained_pauses_or_force_cuts_without_them() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    for width in [10, 20, 40] {
        let mut samples = vec![10000; 8000];
        samples[2000 - width / 2..2000 + width / 2].fill(0);
        samples[2430..3730].fill(500);
        write_wav(&fixture.input, 1000, 1, &samples);
        let result = fixture.run(&["--language", "ar", "--segment-only"]);
        assert!(result.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        // The sustained pause starts 430 ms beyond the target. Cut safely near
        // its beginning, not in the brief dip or at the long pause's midpoint.
        assert_eq!(report["units"][0]["end_frame"], 2460, "dip {width} ms");
        fs::remove_file(&fixture.output).unwrap();
    }

    let mut samples = vec![10000; 8000];
    for (index, center) in (500..8000).step_by(500).enumerate() {
        let width = [10, 20, 40][index % 3];
        samples[center - width / 2..center + width / 2].fill(0);
    }
    write_wav(&fixture.input, 1000, 1, &samples);
    let result = fixture.run(&["--language", "ar", "--segment-only"]);
    assert!(result.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(report["units"].as_array().unwrap().len(), 1);
    assert_eq!(report["units"][0]["start_frame"], 0);
    assert_eq!(report["units"][0]["end_frame"], 8000);
}

#[test]
fn pause_quality_and_target_distance_can_each_decide_the_cut() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 3.0);
    for (background, farther_center, expected) in [(6000, 3400, 3330), (500, 3900, 3000)] {
        let mut samples = vec![10000; 8000];
        samples[2920..3080].fill(background);
        samples[farther_center - 100..farther_center + 100].fill(0);
        write_wav(&fixture.input, 1000, 1, &samples);
        let result = fixture.run(&["--language", "es", "--segment-only"]);
        assert!(result.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        // Prefer a clearer nearby pause over a weak one on target, but retain
        // an already good on-target pause instead of chasing greater silence.
        assert_eq!(report["units"][0]["end_frame"], expected);
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn pause_persistence_adapts_to_local_gap_durations() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 3.0);
    let mut samples = vec![10000; 50000];
    for center in [1000, 5000, 7000, 31000, 35000, 37000] {
        samples[center - 10..center + 10].fill(0);
    }
    for center in [3000, 9000, 11000, 33000] {
        samples[center - 30..center + 30].fill(0);
    }
    for center in [34500, 38000, 41000] {
        samples[center - 300..center + 300].fill(0);
    }
    write_wav(&fixture.input, 1000, 1, &samples);
    for (start, end, expected) in [("0", "20", 3000), ("30", "50", 34230)] {
        let result = fixture.run(&[
            "--language",
            "es",
            "--start",
            start,
            "--end",
            end,
            "--segment-only",
        ]);
        assert!(result.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
        assert_eq!(report["units"][0]["end_frame"], expected);
        fs::remove_file(&fixture.output).unwrap();
    }
}

#[test]
fn equally_good_and_equally_distant_safe_cuts_choose_the_earlier_pause() {
    let fixture = Fixture::new();
    set_chunk_target(&fixture, 2.0);
    let mut samples = vec![10000; 5000];
    samples[1700..1900].fill(0);
    samples[2100..2300].fill(0);
    write_wav(&fixture.input, 1000, 1, &samples);
    let result = fixture.run(&["--language", "es", "--segment-only"]);
    assert!(result.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(report["units"][0]["end_frame"], 1870);
}

#[test]
fn auto_chunking_accepts_fractional_targets_and_requires_no_detector_fields() {
    let fixture = Fixture::new();
    let format: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
    assert_eq!(format["chunking"].as_object().unwrap().len(), 1);
    let result = fixture.run(&[
        "--language",
        "es",
        "--chunk-target-size-seconds",
        "0.5",
        "--check",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn url_check_is_offline_and_requires_exactly_one_supported_source() {
    let fixture = Fixture::new();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
            .env("PATH", &fixture.directory)
            .arg("--format")
            .arg(&fixture.format)
            .arg("--output")
            .arg(&fixture.output)
            .args(["--language", "ja", "--check"])
            .args(extra)
            .output()
            .unwrap()
    };
    for url in [
        "https://www.youtube.com/watch?v=Abc_1234567&list=PLignored&t=120",
        "http://youtu.be/Abc_1234567?t=12",
        "https://youtube.com/shorts/Abc_1234567",
        "https://m.youtube.com/watch?v=Abc_1234567",
        "https://music.youtube.com/watch?v=Abc_1234567",
    ] {
        let result = run(&["--url", url]);
        assert!(
            result.status.success(),
            "{url}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            String::from_utf8_lossy(&result.stdout)
                .contains("Remote availability and recording duration have not been checked")
        );
    }
    for flags in [
        vec![],
        vec![
            fixture.input.to_str().unwrap(),
            "--url",
            "https://youtu.be/Abc_1234567",
        ],
        vec!["--url", "https://youtube.com/playlist?list=PLexample"],
        vec!["--url", "https://youtube.com/@someone"],
        vec!["--url", "https://youtube.com/watch?v=a&v=b"],
        vec!["--url", "https://youtube.com.evil.test/watch?v=a"],
        vec!["--url", "https://user:password@youtube.com/watch?v=a"],
        vec!["--url", "file:///tmp/audio"],
        vec!["--url", "https://youtube.com/watch?v="],
        vec!["--url", "https://youtu.be/a/b"],
        vec!["--url", "https://example.com/audio.mp3"],
        vec!["--url", "not a URL"],
    ] {
        assert!(!run(&flags).status.success(), "accepted {flags:?}");
    }
    assert!(!fixture.output.exists());
    fs::remove_file(&fixture.format).unwrap();
    assert!(
        !run(&["--url", "https://youtu.be/Abc_1234567"])
            .status
            .success()
    );
}

#[cfg(unix)]
struct DownloaderFixture {
    binary_dir: PathBuf,
    metadata: PathBuf,
    calls: PathBuf,
    directories: PathBuf,
}

#[cfg(unix)]
impl DownloaderFixture {
    fn new(fixture: &Fixture) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let binary_dir = fixture.directory.join("downloader");
        fs::create_dir(&binary_dir).unwrap();
        let metadata = fixture.directory.join("remote.json");
        fs::write(&metadata, serde_json::to_vec(&serde_json::json!({
            "id":"Abc_1234567", "title":"Study fixture", "webpage_url":"https://www.youtube.com/watch?v=Abc_1234567",
            "live_status":"not_live","is_live":false,"vcodec":"none","acodec":"pcm_s16le",
            "custom":{"all_fields":[1,"retained",null]}
        })).unwrap()).unwrap();
        let script = binary_dir.join("yt-dlp");
        fs::write(&script, r#"#!/bin/sh
printf '%s\n' "$PWD" >> "$YOUTUBE_TEST_DIRECTORIES"
printf '%s\n' '--CALL--' "$@" >> "$YOUTUBE_TEST_CALLS"
for arg in "$@"; do
  if [ "$arg" = --dump-single-json ]; then
    if [ "$YOUTUBE_TEST_FAILURE" = query ]; then printf 'query failed' >&2; exit 7; fi
    /bin/cat "$YOUTUBE_TEST_METADATA"
    exit 0
  fi
done
if [ "$YOUTUBE_TEST_FAILURE" = download ]; then printf 'partial' > source.wav; printf 'download failed' >&2; exit 8; fi
if [ "$YOUTUBE_TEST_FAILURE" = multiple ]; then printf '%s\n%s\n' "$PWD/one.wav" "$PWD/two.wav"; exit 0; fi
if [ "$YOUTUBE_TEST_FAILURE" = outside ]; then printf '%s\n' "$YOUTUBE_TEST_AUDIO"; exit 0; fi
if [ "$YOUTUBE_TEST_FAILURE" = symlink ]; then /bin/ln -s "$YOUTUBE_TEST_AUDIO" source.wav; printf '%s\n' "$PWD/source.wav"; exit 0; fi
if [ "$YOUTUBE_TEST_FAILURE" = missing ]; then printf '%s\n' "$PWD/source.wav"; exit 0; fi
if [ "$YOUTUBE_TEST_FAILURE" = empty ]; then : > source.wav; printf '%s\n' "$PWD/source.wav"; exit 0; fi
/bin/cp "$YOUTUBE_TEST_AUDIO" source.wav
if [ "$YOUTUBE_TEST_FAILURE" = extra ]; then /bin/cp "$YOUTUBE_TEST_AUDIO" other.wav; fi
printf '%s\n' "$PWD/source.wav"
"#).unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        Self {
            binary_dir,
            metadata,
            calls: fixture.directory.join("calls.txt"),
            directories: fixture.directory.join("directories.txt"),
        }
    }

    fn run(&self, fixture: &Fixture, arguments: &[&str], failure: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    self.binary_dir.display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("YOUTUBE_TEST_METADATA", &self.metadata)
            .env("YOUTUBE_TEST_AUDIO", &fixture.input)
            .env("YOUTUBE_TEST_CALLS", &self.calls)
            .env("YOUTUBE_TEST_DIRECTORIES", &self.directories)
            .env("YOUTUBE_TEST_FAILURE", failure)
            .args([
                "--url",
                "https://youtu.be/Abc_1234567?list=PLignored&t=12",
                "--language",
                "ar",
                "--dialect",
                "masri",
            ])
            .arg("--format")
            .arg(&fixture.format)
            .arg("--output")
            .arg(&fixture.output)
            .args(arguments)
            .output()
            .unwrap()
    }

    fn assert_cleaned(&self) {
        for path in fs::read_to_string(&self.directories).unwrap().lines() {
            assert!(
                !std::path::Path::new(path).exists(),
                "import directory remains: {path}"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn url_modes_match_file_audio_and_keep_lossless_origin_after_cleanup() {
    for (mode, order) in [
        ("--decode-only", "timing-then-joining"),
        ("--segment-only", "timing-then-joining"),
        ("--sequence-only", "timing-then-joining"),
        ("--sequence-only", "joining-then-timing"),
        ("render", "timing-then-joining"),
        ("render", "joining-then-timing"),
    ] {
        let mut fixture = Fixture::new();
        let mut format: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.format).unwrap()).unwrap();
        format["playback"]["sequencing"] = order.into();
        fs::write(&fixture.format, serde_json::to_vec(&format).unwrap()).unwrap();
        fixture.output = fixture.directory.join(if mode == "render" {
            "study.wav"
        } else {
            "report.json"
        });
        let mut samples = tone(16000, 440.0, 32000);
        samples.extend(vec![0; 6400]);
        samples.extend(tone(16000, 660.0, 32000));
        write_wav(&fixture.input, 16000, 1, &samples);
        let original = fs::read(&fixture.input).unwrap();
        let downloader = DownloaderFixture::new(&fixture);
        let mut flags = vec![
            "--start",
            "1",
            "--end",
            "4",
            "--language",
            "ar",
            "--dialect",
            "masri",
        ];
        if mode != "render" {
            flags.push(mode);
        }
        let local = fixture.run(&flags);
        assert!(
            local.status.success(),
            "{}",
            String::from_utf8_lossy(&local.stderr)
        );
        let local_bytes = fs::read(&fixture.output).unwrap();
        let sidecar = PathBuf::from(format!("{}.json", fixture.output.display()));
        fs::remove_file(&fixture.output).unwrap();
        if mode == "--decode-only" {
            fs::remove_file(&sidecar).unwrap();
        }
        let mut remote_flags = vec!["--start", "1", "--end", "4"];
        if mode != "render" {
            remote_flags.push(mode);
        }
        let remote = downloader.run(&fixture, &remote_flags, "");
        assert!(
            remote.status.success(),
            "{}",
            String::from_utf8_lossy(&remote.stderr)
        );
        let source = if mode == "--decode-only" {
            assert_eq!(fs::read(&fixture.output).unwrap(), local_bytes);
            serde_json::from_slice::<serde_json::Value>(&fs::read(&sidecar).unwrap()).unwrap()
        } else if mode == "render" {
            // Both routes feed exactly the same decoded samples through the same tempo pipeline.
            assert_eq!(fs::read(&fixture.output).unwrap(), local_bytes);
            downloader.assert_cleaned();
            assert_eq!(fs::read(&fixture.input).unwrap(), original);
            continue;
        } else {
            let local: serde_json::Value = serde_json::from_slice(&local_bytes).unwrap();
            let remote: serde_json::Value =
                serde_json::from_slice(&fs::read(&fixture.output).unwrap()).unwrap();
            if mode == "--sequence-only" {
                assert_eq!(local["plays"], remote["plays"]);
                assert_eq!(
                    local["segmentation"]["units"],
                    remote["segmentation"]["units"]
                );
                remote["segmentation"]["source"].clone()
            } else {
                assert_eq!(local["units"], remote["units"]);
                remote["source"].clone()
            }
        };
        assert_eq!(source["origin"]["kind"], "youtube");
        assert_eq!(
            source["origin"]["video"]["submitted_url"],
            "https://youtu.be/Abc_1234567?list=PLignored&t=12"
        );
        assert_eq!(
            source["origin"]["webpage_url"],
            "https://www.youtube.com/watch?v=Abc_1234567"
        );
        let original_metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(&downloader.metadata).unwrap()).unwrap();
        assert_eq!(source["origin"]["metadata"], original_metadata);
        assert_eq!(
            source["selected_range"],
            serde_json::json!({"start_frame":16000,"end_frame":64000})
        );
        assert!(!std::path::Path::new(source["source_path"].as_str().unwrap()).exists());
        let calls = fs::read_to_string(&downloader.calls).unwrap();
        for expected in [
            "--ignore-config",
            "--no-playlist",
            "bestaudio",
            "--load-info-json",
            "source.%(ext)s",
            "after_move:filepath",
        ] {
            assert!(calls.lines().any(|line| line == expected));
        }
        assert!(
            calls
                .lines()
                .any(|line| line == "https://www.youtube.com/watch?v=Abc_1234567")
        );
        assert!(!calls.contains("PLignored"));
        downloader.assert_cleaned();
        assert_eq!(fs::read(&fixture.input).unwrap(), original);
    }
}

#[cfg(unix)]
#[test]
fn url_acquisition_failures_preserve_input_and_publish_nothing() {
    let fixture = Fixture::new();
    write_wav(&fixture.input, 16000, 1, &tone(16000, 440.0, 32000));
    let original = fs::read(&fixture.input).unwrap();
    let downloader = DownloaderFixture::new(&fixture);
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&downloader.metadata).unwrap()).unwrap();
    for change in [
        serde_json::json!({"_type":"playlist"}),
        serde_json::json!({"id":"different"}),
        serde_json::json!({"live_status":"is_live"}),
        serde_json::json!({"live_status":"is_upcoming"}),
        serde_json::json!({"live_status":"post_live"}),
        serde_json::json!({"vcodec":"h264"}),
        serde_json::json!({"acodec":"none"}),
        serde_json::json!({"webpage_url":"https://youtube.com/watch?v=other"}),
    ] {
        let mut invalid = metadata.clone();
        for (key, value) in change.as_object().unwrap() {
            invalid[key] = value.clone();
        }
        fs::write(&downloader.metadata, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(
            !downloader.run(&fixture, &[], "").status.success(),
            "accepted {change}"
        );
        assert!(!fixture.output.exists());
        downloader.assert_cleaned();
    }
    fs::write(&downloader.metadata, b"not JSON").unwrap();
    assert!(!downloader.run(&fixture, &[], "").status.success());
    fs::write(&downloader.metadata, serde_json::to_vec(&metadata).unwrap()).unwrap();
    for failure in [
        "query", "download", "missing", "empty", "outside", "multiple", "symlink", "extra",
    ] {
        let result = downloader.run(&fixture, &[], failure);
        assert!(!result.status.success(), "accepted {failure}");
        assert!(!fixture.output.exists());
        assert!(!String::from_utf8_lossy(&result.stdout).contains("Study audio saved:"));
        downloader.assert_cleaned();
    }
    assert_eq!(fs::read(&fixture.input).unwrap(), original);
    let result = Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
        .env("PATH", &fixture.directory)
        .args(["--url", "https://youtu.be/Abc_1234567", "--language", "ja"])
        .arg("--format")
        .arg(&fixture.format)
        .arg("--output")
        .arg(&fixture.output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("cannot start yt-dlp"));
}
