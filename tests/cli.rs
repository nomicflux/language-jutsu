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
    formula: PathBuf,
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
        let formula = directory.join("formula.json");
        let output = directory.join("study.mp3");
        fs::write(&input, b"configuration-check fixture, not decoded audio").unwrap();
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("formulas/default.json"),
            &formula,
        )
        .unwrap();
        Self {
            directory,
            input,
            formula,
            output,
        }
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
            .arg(&self.input)
            .args(["--formula"])
            .arg(&self.formula)
            .arg("--output")
            .arg(&self.output)
            .args(arguments)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).unwrap();
    }
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
fn rereads_custom_formula_and_preserves_speed_order_and_duplicates() {
    let fixture = Fixture::new();
    let mut formula: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.formula).unwrap()).unwrap();
    formula["sequencing"] = "joining-then-timing".into();
    formula["segments"] = serde_json::json!([1.1, 0.75, 0.75]);
    formula["joining"]["group_size"] = 2.into();
    formula["repetitions"] = 3.into();
    formula["gap_ms"] = 0.into();
    fs::write(&fixture.formula, serde_json::to_vec(&formula).unwrap()).unwrap();

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
fn missing_formula_file_or_required_field_never_falls_back() {
    let fixture = Fixture::new();
    fs::remove_file(&fixture.formula).unwrap();
    let output = fixture.run(&["--language", "es", "--check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot open formula"));
    fs::write(&fixture.formula, b"{}").unwrap();
    let output = fixture.run(&["--language", "es", "--check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing field"));
    assert!(!fixture.output.exists());
}

#[test]
fn rejects_invalid_formula_controls() {
    let fixture = Fixture::new();
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.formula).unwrap()).unwrap();
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
            formula[key] = value.clone();
        }
        fs::write(&fixture.formula, serde_json::to_vec(&formula).unwrap()).unwrap();
        let output = fixture.run(&["--language", "ja", "--check"]);
        assert!(!output.status.success(), "accepted {invalid}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid formula"));
        assert!(!fixture.output.exists());
    }
}

#[test]
fn rejects_non_arabic_dialect_and_missing_input() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--language", "ja", "--dialect", "masri", "--check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("only valid with --language ar"));
    fs::remove_file(&fixture.input).unwrap();
    let output = fixture.run(&["--language", "ar", "--dialect", "masri", "--check"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read input"));
}

#[test]
fn default_formula_is_an_external_required_file() {
    let fixture = Fixture::new();
    let binary = fixture
        .directory
        .join(format!("language-jutsu{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_language-jutsu"), &binary).unwrap();
    let formulas = fixture.directory.join("formulas");
    fs::create_dir(&formulas).unwrap();
    let default = formulas.join("default.json");
    fs::copy(&fixture.formula, &default).unwrap();

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
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot open formula"));
    assert!(!fixture.output.exists());
}

#[test]
fn unfinished_rendering_fails_explicitly_without_creating_output() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--language", "es"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not implemented in this skeleton"));
    assert!(!fixture.output.exists());
}
