# Anki cards from the lesson

Use this prepared guidance to author the requested cards and package them with the existing app. Apply existing-deck instructions only when adding to an existing deck. Do not repeat the underlying research, inspect or reimplement the builder, or test migration from hypothetical earlier decks.

Derive the cards from the prepared lesson's teaching and processed clips. Deliver a `.apkg` containing those cards, templates and audio for the chosen new or existing deck. Use the lesson's language, dialect, level and support language. Creating a package does not authorize changing the user's live Anki collection.

**Use the containing Jutsu chunk directly as one `context` object: `audio` is its recording; `sentence` is its full text. Submit them together through the CLI. They are the same material in different media, containing the card target.** Include all speech in that chunk, including surrounding sentences or turns. Do not compose a separate context sentence or pair shortened text with the whole recording.

## Choose the destination

Unless the request already specifies it, ask: **Create a new deck, or add these lesson cards to an existing deck?** For an existing deck, establish the exact deck/subdeck and obtain its export or authorized access. Inspect a copy of its cards, note types, templates, IDs and media before authoring additions. Do not silently create a new deck when the chosen existing deck is unavailable; obtain the missing input.

Compare proposed learning points with the existing cards and add only new work. Reuse compatible note types without changing their fields or templates; if they cannot express the required card types, create distinct note types for the additions. Preserve existing notes, identities, media, deck settings and study history. Package only the additions for the selected destination, rather than rebuilding the existing deck from an older export.

## Cover the lesson and separate its points

Work through the actual lesson's excerpts, notes, explanations and activities. For every vocabulary sense, expression, grammatical construction and distinct use taught or practised, make a card that elicits that point. An explicit user-requested subset limits that scope; for an existing destination deck, reuse its coverage and add the missing points. Determine the card count from this work. Do not substitute highlights, a quota or a type ratio for lesson coverage. Short chunks, basic questions and a beginner level determine support and difficulty, not which taught points are omitted.

Split a glossary row or explanation that teaches several points into separate cards. A listening exercise does not replace the vocabulary and grammar cards for the points taught in that speech. Remove duplicate tasks, not distinct learning points.

Verify wording, meaning and readings against the lesson evidence; raw ASR is not an answer key. Each card must be understandable independently of other cards or memory of the lesson. Keep timestamps, correction histories and production records outside card fields.

## Three separate card types

Use exactly three card types: Vocab, Grammar and Listening. Choose the type by what the learner must recall, then write that question and its expected answer before formatting the card. Use distinct note types, each with one front/back template unless the user requests additional directions. Atomicity is specific to the task:

| Type | Front: one recall task | Back: answer and learning support |
| --- | --- | --- |
| Vocab | Mark **one word**, or **one phrase only when the taught meaning requires the phrase**, in the chunk's full text; ask for its contextual meaning. Do not turn an ordinary sentence into a vocabulary target. | Give the contextual meaning, then the chunk's full text with the target marked, translation when applicable, needed explanation and the same chunk audio. Explain an inflected form only if needed to connect it to the headword. |
| Grammar | The chunk's full text with only the grammatical piece blanked, all vocabulary visible, and a short hint saying what the piece does. The construction may span several words or discontinuous parts. Supply vocabulary support so answering does not also require a vocabulary test. Do not combine grammatical features even when the same words express them; make separate cards for those points. | Give the answer, the chunk's full unblanked text with the construction marked, translation when applicable, a brief explanation of this point and the same chunk audio. Avoid conjugation tables, unrelated uses and grammar-analysis quizzes. |
| Listening | Play the chunk. Ask only “What was said?”, or show its full text with **one word blanked** and ask for the word heard. For full recall, hide the text; for cloze, show every other word. Keep translations and the requested answer off the front. Do not add meaning or grammar questions. | Show the chunk's full text, marking the requested material, and the same chunk audio. For a cloze, give the missing word. Add translation when applicable and a brief spoken-form explanation if needed. Show the full context text only once. |

Several cards can test different targets within the same unchanged audio/text chunk. For Listening, the answer is exactly what was requested on the front; back highlighting cannot change that task. Allow oral or mental recall with self-checking; add typing only if requested.

## Back-of-card support

Retain the chunk's full speech and genuine dialect, including elliptical replies and surrounding turns.

Include a written translation of the full context text when the user requests it **or the learner is B1 or below**. At higher levels, omit translation unless requested. Honor an explicit request for no translations. Use the established support language; do not assume English. If level or translation language is unresolved and changes the cards, ask a concise follow-up using the lesson's calibration approach rather than guessing an equivalence between proficiency scales.

Explain only the target meaning, form or usage; omit explanation when the answer and context suffice. Use readable text, appropriate script direction and established reading aids without revealing a listening answer on the front. Translations are written support; audio remains entirely in the target language.

## Use the containing chunk once at full speed

For every card, use the existing Jutsu chunk that contains its spoken material. Take the chunk's original-source bounds from the lesson's sequence report and render that span from the retained original through [preparation.md](preparation.md). Preserve the whole chunk, including surrounding speech; do not choose new sentence or phrase boundaries or trim an expanded study track.

The audio file contains one playback at full original speed (1×). Use `playback.segments: [1.0]`, `repetitions: 1`, `joining.group_size: 1` and `gap_ms: 0`. Reuse an existing file if it already contains exactly this playback. This rule applies to both listening-front and back-of-card audio.

Review the actual rendered speech against its full written context, with the whole chunk retained and playback once at original speed. **An unclear lesson or card clip is a failed audio or passage selection for the whole lesson.** Reselect clear audio, rerun Jutsu, and rebuild the affected lesson, notes, activities and cards together. Do not deliver either as ready, drop required points or relax card requirements to work around unclear speech. The back includes the same chunk in text and audio.

## Package for Anki

Use the app's Rust builder for the three Language Jutsu note types:

```sh
cargo run -- anki cards.json --output lesson.apkg
# Or, with the built executable:
language-jutsu anki cards.json --output lesson.apkg
```

The command packages authored cards and their audio. It does not select learning points, verify linguistic accuracy, process audio or access a live Anki collection. Existing output files are never replaced; use a new output path for a revision.

The JSON input contains `deck_id` (positive integer), `deck_name`, `language` (source language), optional `direction` (`auto`, `ltr` or `rtl`, default `auto`), and a nonempty `cards` array. Choose a unique deck ID for a new deck and retain it for revisions. For additions, use the inspected destination's name and ID. The builder uses its own versioned Language Jutsu models; it cannot adapt arbitrary existing note-type schemas. Reuse compatible destination note types as required above instead of silently replacing them with these models.

Each card requires `key`, `kind` (`vocab`, `grammar` or `listening`), `prompt`, `answer` and one `context` object with both `audio` (the containing chunk's recording path) and `sentence` (that chunk's full text). The CLI rejects separate card-level `sentence`, `context_audio` and `listening_audio` fields. Listening fronts and all backs use the audio from this same object. Optional fields:

| Field | Meaning |
| --- | --- |
| `marks` | Literal, nonoverlapping target fragments to emphasize in the chunk's text; each must occur exactly once. Several fragments can mark one discontinuous construction. |
| `front` | Plain text controlling presentation of the same chunk. Vocab defaults to its marked text. For Grammar, provide its full text with only the grammatical piece blanked. Listening defaults to hidden text; for cloze, provide its full text with one word blanked. Do not substitute shorter or different context. |
| `support` | Vocabulary or other help on Vocab/Grammar fronts. Never shown on Listening fronts. |
| `translation`, `reading`, `explanation`, `situation` | Back support; omitted fields are empty and their sections hidden. Supply the support required by the lesson and level. |

All written content is plain text, escaped by the builder. `marks` adds emphasis to `context.sentence`; do not put HTML in fields. Author prompts in the established support language. Audio paths resolve relative to the manifest, or may be absolute. Use unique media basenames across lessons. The builder embeds referenced files, deduplicates reused paths and rejects different files sharing a basename. The full context audio appears on every back. Full-sentence Listening answers equal to `context.sentence` are shown only once.

Minimal input shape (here the whole audio chunk says “She is looking for her keys.”; replace this illustrative audio/text pair with the actual lesson chunk):

```json
{
  "deck_id": 1792909000,
  "deck_name": "English Practice",
  "language": "en",
  "direction": "ltr",
  "cards": [{
    "key": "lesson-01-look-for",
    "kind": "vocab",
    "prompt": "¿Qué significa la expresión marcada?",
    "answer": "Buscar.",
    "context": {
      "audio": "lesson-01-context.mp3",
      "sentence": "She is looking for her keys."
    },
    "marks": ["looking for"],
    "translation": "Ella está buscando sus llaves."
  }]
}
```

Keep each card's `key` unique within its deck and stable across wording revisions. The note GUID is derived from the deck ID and key, so editing an answer does not create a new note. A new learning point needs a new key; keep the type stable for a continuing note. The manifest records deck identity and keys; model IDs are fixed in the builder. Invalid fields, keys, media or marks fail before publishing a package.

## Review and deliver

Return to the actual lesson, including its activities, and find the card question that elicits each taught or practised vocabulary and grammar point. Check the question and expected answer, not just a stored target or a matching glossary count. A point appearing only in another card's sentence, translation or explanation is not covered. Add missing cards and split overloaded tasks; do not shrink the lesson to match an incomplete deck. Keep this correspondence current when the lesson changes.

Use every final front before its back. Determine what the learner is being asked to recall using only that front, attempt it at their level, then check whether the back answers exactly that task with the required sentence, translation, explanation and audio. Apply the type-specific atomicity above to the actual question and answer, not its label or metadata. Remove unnecessary framing while preserving context and coverage. Repair failures and review the complete revised lesson and deck together.

Save the package, manifest with its stable identities, and audio/source links under the lesson directory; record the new/existing choice, destination and package path in `task.md`. Deliver the `.apkg` with its destination, counts of added cards by type and a brief instruction to import it using Anki's **File → Import**. Do not claim a completed learner review session or user acceptance from authoring.

## Research basis

The Japanese Deck and its vocabulary/grammar skills informed contextual meanings, independent cards, succinct explanations and learner review. This multilingual workflow is self-contained and uses this lesson's language and card requirements.

[Wozniak's practitioner guidance](https://www.super-memory.com/articles/20rules.htm) supports understanding before memorization and simple recall items. [Anki's package documentation](https://docs.ankiweb.net/manual/importing/packaged-decks) explains import and revision behavior. These citations record the basis for the guidance above; they are not additional research tasks for lesson preparation.
