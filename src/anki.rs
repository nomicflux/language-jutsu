use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use genanki_rs::{Deck, Field, Model, Note, Package, Template};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    deck_id: i64,
    deck_name: String,
    language: String,
    #[serde(default)]
    direction: Direction,
    cards: Vec<Card>,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Direction {
    #[default]
    Auto,
    Ltr,
    Rtl,
}

impl Direction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Vocab,
    Grammar,
    Listening,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Vocab => "Vocab",
            Self::Grammar => "Grammar",
            Self::Listening => "Listening",
        }
    }

    fn model_id(self) -> i64 {
        // Versioned shared models; changing their field schema requires new IDs.
        match self {
            Self::Vocab => 1792908001,
            Self::Grammar => 1792908002,
            Self::Listening => 1792908003,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Card {
    key: String,
    kind: Kind,
    prompt: String,
    answer: String,
    sentence: String,
    context_audio: PathBuf,
    #[serde(default)]
    front: Option<String>,
    #[serde(default)]
    marks: Vec<String>,
    #[serde(default)]
    support: String,
    #[serde(default)]
    translation: String,
    #[serde(default)]
    reading: String,
    #[serde(default)]
    explanation: String,
    #[serde(default)]
    situation: String,
    #[serde(default)]
    listening_audio: Option<PathBuf>,
}

const FIELDS: [&str; 14] = [
    "Key",
    "Prompt",
    "FrontText",
    "Support",
    "Answer",
    "Sentence",
    "Translation",
    "Reading",
    "Explanation",
    "ContextAudio",
    "ListeningAudio",
    "Situation",
    "Language",
    "Direction",
];
const CSS: &str = r#".card{font:21px/1.55 Arial,sans-serif;text-align:start;max-width:680px;margin:16px auto;padding:0 16px;color:#202020;background:#fff}
.source{font-size:30px;line-height:1.8;margin:18px 0;white-space:pre-wrap}
.source b{font-weight:700;text-decoration:underline;text-underline-offset:5px}
.support{font-size:17px;color:#555;margin:14px 0;white-space:pre-wrap}
.answer{font-size:23px;font-weight:600;margin:16px 0;white-space:pre-wrap}
.nightMode.card{background:#202124;color:#eee}.nightMode .support{color:#ccc}"#;
const BACK: &str = r#"{{#Answer}}<div class="answer" dir="auto">{{Answer}}</div>{{/Answer}}
<div class="source" dir="{{Direction}}" lang="{{Language}}">{{Sentence}}</div>
{{#Translation}}<div dir="auto">{{Translation}}</div>{{/Translation}}
{{#Reading}}<div class="support" dir="auto">{{Reading}}</div>{{/Reading}}
{{#Explanation}}<div class="support" dir="auto">{{Explanation}}</div>{{/Explanation}}
<div>{{ContextAudio}}</div>{{#Situation}}<div class="support" dir="auto">{{Situation}}</div>{{/Situation}}"#;

fn model(kind: Kind) -> Model {
    let mut front = String::from(r#"<div dir="auto">{{Prompt}}</div>"#);
    if kind == Kind::Listening {
        front.push_str("<div>{{ListeningAudio}}</div>");
    }
    front.push_str(r#"{{#FrontText}}<div class="source" dir="{{Direction}}" lang="{{Language}}">{{FrontText}}</div>{{/FrontText}}"#);
    if kind != Kind::Listening {
        front.push_str(
            r#"{{#Support}}<div class="support" dir="auto">{{Support}}</div>{{/Support}}"#,
        );
    }
    Model::new(
        kind.model_id(),
        &format!("Language Jutsu v1 · {}", kind.name()),
        FIELDS.iter().map(|name| Field::new(name)).collect(),
        vec![Template::new(kind.name()).qfmt(&front).afmt(BACK)],
    )
    .css(CSS)
}

pub(crate) fn build(input: &Path, output: &Path) -> Result<()> {
    let manifest: Manifest = serde_json::from_slice(&fs::read(input)?)?;
    if output.extension().and_then(|ext| ext.to_str()) != Some("apkg") {
        return Err("output must have an .apkg extension".into());
    }
    if output.try_exists()? {
        return Err(format!("output already exists: {}", output.display()).into());
    }
    if manifest.deck_id <= 0 || manifest.cards.is_empty() {
        return Err("deck_id must be positive and cards must not be empty".into());
    }
    required("deck_name", &manifest.deck_name)?;
    required("language", &manifest.language)?;
    let base = input.parent().unwrap_or_else(|| Path::new("."));
    let mut media = BTreeMap::new();
    let mut keys = HashSet::new();
    let mut deck = Deck::new(manifest.deck_id, &manifest.deck_name, "");
    for card in &manifest.cards {
        let note = make_note(card, &manifest, base, &mut media)
            .map_err(|error| format!("card {:?}: {error}", card.key))?;
        if !keys.insert(&card.key) {
            return Err(format!("duplicate card key: {}", card.key).into());
        }
        deck.add_note(note);
    }
    let paths: Vec<_> = media.values().map(String::as_str).collect();
    let mut package = Package::new(vec![deck], paths)?;
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut file = NamedTempFile::new_in(parent)?;
    package.write(file.as_file_mut())?;
    file.persist_noclobber(output)?;
    println!(
        "Wrote {} cards to {}",
        manifest.cards.len(),
        output.display()
    );
    Ok(())
}

fn make_note(
    card: &Card,
    manifest: &Manifest,
    base: &Path,
    media: &mut BTreeMap<String, String>,
) -> Result<Note> {
    for (name, value) in [
        ("key", &card.key),
        ("prompt", &card.prompt),
        ("answer", &card.answer),
        ("sentence", &card.sentence),
    ] {
        required(name, value)?;
    }
    if card.kind != Kind::Listening && card.listening_audio.is_some() {
        return Err("listening_audio is only used for listening cards".into());
    }
    let sentence = marked(&card.sentence, &card.marks)?;
    let front = match &card.front {
        Some(text) => escape(text),
        None if card.kind == Kind::Listening => String::new(),
        None => sentence.clone(),
    };
    let context_audio = sound(base, &card.context_audio, media)?;
    let listening_audio = if card.kind == Kind::Listening {
        sound(
            base,
            card.listening_audio
                .as_deref()
                .unwrap_or(&card.context_audio),
            media,
        )?
    } else {
        String::new()
    };
    // The complete transcript is already shown as Sentence on a listening back.
    let answer = if card.kind == Kind::Listening && card.answer == card.sentence {
        String::new()
    } else {
        escape(&card.answer)
    };
    let values = [
        escape(&card.key),
        escape(&card.prompt),
        front,
        escape(&card.support),
        answer,
        sentence,
        escape(&card.translation),
        escape(&card.reading),
        escape(&card.explanation),
        context_audio,
        listening_audio,
        escape(&card.situation),
        escape(&manifest.language),
        manifest.direction.as_str().to_owned(),
    ];
    if values.iter().any(|value| value.contains('\u{1f}')) {
        return Err("card text must not contain Anki's U+001F field separator".into());
    }
    // Content edits preserve identity; a new learning point needs a new key.
    let guid = format!(
        "{:x}",
        Sha256::digest(format!("language-jutsu:{}:{}", manifest.deck_id, card.key))
    );
    Ok(Note::new(
        model(card.kind),
        values.iter().map(String::as_str).collect(),
    )?
    .guid(guid)
    .with_tag(card.kind.name()))
}

fn required(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(format!("{name} must not be empty").into())
    } else {
        Ok(())
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn marked(text: &str, marks: &[String]) -> Result<String> {
    let mut spans = Vec::new();
    for mark in marks {
        required("mark", mark)?;
        let matches: Vec<_> = text.match_indices(mark).collect();
        if matches.len() != 1 {
            return Err(format!(
                "mark {mark:?} must occur exactly once in sentence; use a unique fragment"
            )
            .into());
        }
        let start = matches[0].0;
        spans.push((start, start + mark.len()));
    }
    spans.sort_unstable();
    let mut result = String::new();
    let mut cursor = 0;
    for (start, end) in spans {
        if start < cursor {
            return Err("marks must not overlap".into());
        }
        result.push_str(&escape(&text[cursor..start]));
        result.push_str("<b>");
        result.push_str(&escape(&text[start..end]));
        result.push_str("</b>");
        cursor = end;
    }
    result.push_str(&escape(&text[cursor..]));
    Ok(result)
}

fn sound(base: &Path, path: &Path, media: &mut BTreeMap<String, String>) -> Result<String> {
    let path = base.join(path).canonicalize()?;
    if !path.is_file() {
        return Err(format!("audio is not a file: {}", path.display()).into());
    }
    let path_text = path.to_str().ok_or("audio path must be UTF-8")?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("audio needs a filename")?;
    if name.contains(['[', ']', '<', '>', '&', '"', '\'', ':', '\\'])
        || name.chars().any(char::is_control)
    {
        return Err(format!("audio filename is unsafe in a sound reference: {name}").into());
    }
    if let Some(previous) = media.insert(name.to_owned(), path_text.to_owned()) {
        if previous != path_text {
            return Err(format!(
                "different audio files share basename {name:?}; give them unique names"
            )
            .into());
        }
    }
    Ok(format!("[sound:{name}]"))
}
