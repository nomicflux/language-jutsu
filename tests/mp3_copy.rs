use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use tempfile::TempDir;

fn run(output: &Path) -> Output {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jfk.wav");
    Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
        .arg("mp3-copy")
        .arg(input)
        .args(["--title", "Ex 1", "--album", "2026-10-28 Lesson"])
        .args(["--artist", "Arabic", "--track", "2", "--output"])
        .arg(output)
        .output()
        .unwrap()
}

fn tag(path: &Path, name: &str) -> String {
    let probe = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries"])
        .arg(format!("format_tags={name}"))
        .args(["-of", "default=nw=1:nk=1"])
        .arg(path)
        .output()
        .unwrap();
    String::from_utf8(probe.stdout).unwrap().trim().to_owned()
}

#[test]
fn copy_is_an_mp3_with_the_given_tags() {
    let root = TempDir::new().unwrap();
    let output = root.path().join("copy.mp3");
    let result = run(&output);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(tag(&output, "title"), "Ex 1");
    assert_eq!(tag(&output, "album"), "2026-10-28 Lesson");
    assert_eq!(tag(&output, "artist"), "Arabic");
    assert_eq!(tag(&output, "track"), "2");
}

#[test]
fn copy_never_replaces_an_existing_file() {
    let root = TempDir::new().unwrap();
    let output = root.path().join("copy.mp3");
    fs::write(&output, b"keep").unwrap();
    let result = run(&output);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("output already exists"));
    assert_eq!(fs::read(&output).unwrap(), b"keep");
}

#[test]
fn copy_refuses_a_non_mp3_output() {
    let root = TempDir::new().unwrap();
    let output = root.path().join("copy.wav");
    let result = run(&output);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("output must have an .mp3 extension"));
    assert!(!output.exists());
}
