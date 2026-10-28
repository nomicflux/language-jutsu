use std::{fs, io::Read, path::Path, process::Command};

use serde_json::{Value, json};
use tempfile::TempDir;

fn manifest() -> Value {
    let shared = json!({
        "key": "keys", "kind": "vocab", "prompt": "What does keys mean?",
        "answer": "llaves",
        "context": {"audio": "context.wav", "sentence": "She has been looking for her keys."},
        "marks": ["keys"],
        "translation": "Ella ha estado buscando sus llaves.",
        "explanation": "Literal <text> & quotes: \"test\"."
    });
    let mut grammar = shared.clone();
    grammar["key"] = json!("continuous");
    grammar["kind"] = json!("grammar");
    grammar["prompt"] = json!("What does the marked construction express?");
    grammar["marks"] = json!(["has been", "looking"]);
    grammar["answer"] = json!("An activity continuing up to now.");
    grammar["support"] = json!("look for = buscar; keys = llaves");
    let mut listening = shared.clone();
    listening["key"] = json!("whole");
    listening["kind"] = json!("listening");
    listening["prompt"] = json!("What was said?");
    listening["answer"] = shared["context"]["sentence"].clone();
    listening["marks"] = json!([]);
    let mut cloze = listening.clone();
    cloze["key"] = json!("heard-for");
    cloze["prompt"] = json!("Which word did you hear?");
    cloze["front"] = json!("She has been looking ___ her keys.");
    cloze["answer"] = json!("for");
    cloze["marks"] = json!(["for"]);
    json!({"deck_id": 123456789, "deck_name": "Builder test", "language": "en",
        "direction": "ltr", "cards": [shared, grammar, listening, cloze]})
}

fn run(root: &Path, value: &Value, output: &str) -> std::process::Output {
    fs::write(root.join("cards.json"), serde_json::to_vec(value).unwrap()).unwrap();
    // Resolve media relative to the manifest, not the command's working directory.
    Command::new(env!("CARGO_BIN_EXE_language-jutsu"))
        .args([
            "anki",
            root.join("cards.json").to_str().unwrap(),
            "--output",
        ])
        .arg(root.join(output))
        .output()
        .unwrap()
}

fn unpack(root: &Path, filename: &str) -> (rusqlite::Connection, Vec<(String, Vec<u8>)>) {
    let mut zip = zip::ZipArchive::new(fs::File::open(root.join(filename)).unwrap()).unwrap();
    let mut db = Vec::new();
    zip.by_name("collection.anki2")
        .unwrap()
        .read_to_end(&mut db)
        .unwrap();
    let db_path = root.join(format!("{filename}.db"));
    fs::write(&db_path, db).unwrap();
    let mut mapping = String::new();
    zip.by_name("media")
        .unwrap()
        .read_to_string(&mut mapping)
        .unwrap();
    let mapping: std::collections::BTreeMap<String, String> =
        serde_json::from_str(&mapping).unwrap();
    let media = mapping
        .into_iter()
        .map(|(id, name)| {
            let mut bytes = Vec::new();
            zip.by_name(&id).unwrap().read_to_end(&mut bytes).unwrap();
            (name, bytes)
        })
        .collect();
    (rusqlite::Connection::open(db_path).unwrap(), media)
}

fn notes(db: &rusqlite::Connection) -> Vec<(String, Vec<String>)> {
    db.prepare("SELECT guid, flds FROM notes ORDER BY id")
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get::<_, String>(1)?
                    .split('\x1f')
                    .map(str::to_owned)
                    .collect(),
            ))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn packages_three_types_embedded_media_and_stable_revision_identities() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    // The builder packages bytes; audio processing is tested by the audio CLI tests.
    fs::write(root.join("context.wav"), b"context audio bytes").unwrap();
    let mut value = manifest();
    let result = run(root, &value, "first.apkg");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let (db, mut media) = unpack(root, "first.apkg");
    media.sort();
    assert_eq!(
        media,
        vec![("context.wav".into(), b"context audio bytes".to_vec())]
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM cards WHERE did=123456789", [], |r| {
            r.get::<_, i64>(0)
        })
        .unwrap(),
        4
    );
    let first = notes(&db);
    assert_eq!(first.len(), 4);
    let vocab = &first[0].1;
    assert_eq!(vocab[2], "She has been looking for her <b>keys</b>.");
    assert_eq!(vocab[5], "She has been looking for her <b>keys</b>.");
    assert_eq!(
        vocab[8],
        "Literal &lt;text&gt; &amp; quotes: &quot;test&quot;."
    );
    assert_eq!(vocab[9], "[sound:context.wav]");
    assert_eq!(vocab[10], "");
    assert_eq!(
        first[1].1[2],
        "She <b>has been</b> <b>looking</b> for her keys."
    );
    assert_eq!(first[2].1[2], ""); // Full listening hides the transcript.
    assert_eq!(first[2].1[4], ""); // Full sentence appears once on the back.
    assert_eq!(first[2].1[5], "She has been looking for her keys.");
    assert_eq!(first[3].1[2], "She has been looking ___ her keys.");
    assert_eq!(first[3].1[5], "She has been looking <b>for</b> her keys.");
    for (_, fields) in &first[2..] {
        assert_eq!(fields[9], "[sound:context.wav]");
        assert_eq!(fields[10], fields[9]); // Both sides use the context object's audio.
    }
    let models: String = db
        .query_row("SELECT models FROM col", [], |r| r.get(0))
        .unwrap();
    let models: Value = serde_json::from_str(&models).unwrap();
    assert_eq!(models.as_object().unwrap().len(), 3);
    for model in models.as_object().unwrap().values() {
        assert_eq!(model["tmpls"].as_array().unwrap().len(), 1);
        let front = model["tmpls"][0]["qfmt"].as_str().unwrap();
        assert!(!front.contains("{{Answer}}") && !front.contains("{{Translation}}"));
        assert!(
            model["tmpls"][0]["afmt"]
                .as_str()
                .unwrap()
                .contains("{{ContextAudio}}")
        );
    }
    value["cards"][0]["answer"] = json!("revised meaning");
    value["cards"][1]["front"] = json!("She ___ been looking for her keys.");
    let result = run(root, &value, "revised.apkg");
    assert!(result.status.success());
    let (revised, _) = unpack(root, "revised.apkg");
    let second = notes(&revised);
    assert_eq!(
        first.iter().map(|n| &n.0).collect::<Vec<_>>(),
        second.iter().map(|n| &n.0).collect::<Vec<_>>()
    );
    assert_eq!(second[0].1[4], "revised meaning");
    assert_eq!(second[1].1[2], "She ___ been looking for her keys.");
}

#[test]
fn invalid_inputs_publish_nothing_and_existing_output_is_preserved() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("context.wav"), b"context").unwrap();
    for (field, invalid, expected) in [
        ("kind", json!("other"), "unknown variant"),
        (
            "context",
            json!({"audio": "missing.wav", "sentence": "She has been looking for her keys."}),
            "card \"keys\"",
        ),
        ("marks", json!(["absent"]), "must occur exactly once"),
        ("marks", json!(["keys", "her keys"]), "must not overlap"),
        ("answer", json!(""), "answer must not be empty"),
        (
            "explanation",
            json!("bad\u{1f}field"),
            "U+001F field separator",
        ),
        ("typo", json!(true), "unknown field"),
    ] {
        let mut value = manifest();
        value["cards"][0][field] = invalid;
        let result = run(root, &value, "invalid.apkg");
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!root.join("invalid.apkg").exists());
    }
    let mut value = manifest();
    value["cards"][1]["key"] = json!("keys");
    let result = run(root, &value, "duplicate.apkg");
    assert!(String::from_utf8_lossy(&result.stderr).contains("duplicate card key"));
    assert!(!root.join("duplicate.apkg").exists());
    fs::create_dir(root.join("other")).unwrap();
    fs::write(root.join("other/context.wav"), b"different").unwrap();
    value = manifest();
    value["cards"][1]["context"]["audio"] = json!("other/context.wav");
    let result = run(root, &value, "collision.apkg");
    assert!(String::from_utf8_lossy(&result.stderr).contains("share basename"));
    assert!(!root.join("collision.apkg").exists());
    fs::write(root.join("existing.apkg"), b"keep this").unwrap();
    let result = run(root, &manifest(), "existing.apkg");
    assert!(!result.status.success());
    assert_eq!(fs::read(root.join("existing.apkg")).unwrap(), b"keep this");
}

#[test]
fn context_requires_audio_and_sentence_together_and_rejects_separate_fields() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("context.wav"), b"context").unwrap();
    for (context, expected) in [
        (json!({"audio": "context.wav"}), "missing field `sentence`"),
        (
            json!({"sentence": "She has keys."}),
            "missing field `audio`",
        ),
        (
            json!({"audio": "context.wav", "sentence": " "}),
            "context.sentence must not be empty",
        ),
    ] {
        let mut value = manifest();
        value["cards"][0]["context"] = context;
        let result = run(root, &value, "incomplete.apkg");
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!root.join("incomplete.apkg").exists());
    }
    for field in ["sentence", "context_audio", "listening_audio"] {
        let mut value = manifest();
        // A separate field is rejected even when a complete context is present.
        value["cards"][2][field] = json!("separate");
        let result = run(root, &value, "separate.apkg");
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(&format!("unknown field `{field}`")),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!root.join("separate.apkg").exists());
    }
}
