# Anki cards from the lesson

Use this prepared guidance to author the requested cards and package them with the existing app. Apply existing-deck instructions only when adding to an existing deck. Do not repeat the underlying research, inspect or reimplement the builder, or test migration from hypothetical earlier decks.

Derive the cards from the prepared lesson's teaching and processed clips. Deliver a `.apkg` containing those cards, templates and audio for the chosen new or existing deck. Use the lesson's language, dialect, level and support language. Creating a package does not authorize changing the user's live Anki collection.

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
| Vocab | Show **one word**, or **one phrase only when the taught meaning requires the phrase**, such as an idiom or a word with its required preposition. Mark the target in enough source context to identify its sense; ask for that meaning. Do not turn an ordinary sentence into a vocabulary target. | Give the contextual meaning, then the full source sentence with the target marked, translation when applicable, needed explanation and context audio. Explain an inflected form only if needed to connect it to the headword. |
| Grammar | The card's sentence with only the grammatical piece blanked, all vocabulary visible, and a short hint saying what the piece does. The construction may span several words or discontinuous parts. Supply enough context and vocabulary support that answering does not also require a vocabulary test. Do not combine grammatical features even when the same words express them; make separate cards for those points. | Give the answer, the full unblanked source sentence with the construction marked, translation when applicable, a brief explanation of this point and context audio. Avoid conjugation tables, unrelated uses and grammar-analysis quizzes. |
| Listening | Play a **reasonably short processed phrase**, generally around the session's chunk length or shorter. Ask only “What was said?”, or show the phrase with **one word blanked** and ask for the word heard. For full recall, hide the transcript; for cloze, show every other word. Keep translations and the requested answer off the front. Do not add meaning or grammar questions. | Show the verified phrase or missing word in its full source sentence, marking what was requested. Add translation when applicable, a brief spoken-form explanation if needed, and replayable audio. If the clip is a complete sentence, show it once. |

**Audio is context for Vocab and Grammar; it is the tested material for Listening.** Context audio need not contain only the tested point. The same source sentence and recording can support several cards with different atomic targets. Each card's sentence is the sentence that fits that card. For Listening, the answer is exactly the phrase or missing word requested on the front; back highlighting cannot change that task. Allow oral or mental recall with self-checking; add typing only if requested.

## Back-of-card support

Retain the full source sentence and genuine dialect. For an elliptical reply, retain the complete utterance and any preceding turn needed to understand it; do not invent a longer sentence.

Include a written translation of the full sentence when the user requests it **or the learner is B1 or below**. At higher levels, omit translation unless requested. Honor an explicit request for no translations. Use the established support language; do not assume English. If level or translation language is unresolved and changes the cards, ask a concise follow-up using the lesson's calibration approach rather than guessing an equivalence between proficiency scales.

Explain only the target meaning, form or usage; omit explanation when the answer and context suffice. Use readable text, appropriate script direction and established reading aids without revealing a listening answer on the front. Translations are written support; audio remains entirely in the target language.

## Use the lesson's processed audio

Lessons and cards use the same base audio. Start with the relevant Language Jutsu-processed lesson clip: it contains the language being taught and reviewed. Reuse that output when it suits the card's brief review; an isolated-word recording is not a prerequisite.

If the lesson player is too long for the card, render the relevant sentence or exchange from the retained original through [preparation.md](preparation.md). Preserve the lesson's wording, meaning and teaching point. Use original-source timestamps from the verified lesson evidence, not offsets in an expanded practice track.

Choose the format JSON for the learner's level and the card's brief review task: appropriate chunk targets, speed progression, repetitions and joining. Jutsu still owns segmentation, sequencing and assembly. Do not trim a rendered practice track by hand, concatenate clips, substitute TTS or attach an entire lesson player to every card. Only an explicit no-processing request changes the processing requirement.

Review the actual rendered speech against the lesson's verified sentence: the target must be audible, boundaries must retain words, and repetition and duration must serve one short review. **An unclear lesson or card clip is a failed audio or passage selection for the whole lesson.** Reselect clear audio, rerun Jutsu, and rebuild the affected lesson, notes, activities and cards together. Do not deliver either as ready, drop required points or relax card requirements to work around unclear speech. The back may replay the front's clip; a phrase front needs its full sentence and contextual audio on the back.

## Package for Anki

Use the app's Rust builder for the three Language Jutsu note types:

```sh
cargo run -- anki cards.json --output lesson.apkg
# Or, with the built executable:
language-jutsu anki cards.json --output lesson.apkg
```

The command packages authored cards and processed audio. It does not select learning points, verify linguistic accuracy, process audio or access a live Anki collection. Existing output files are never replaced; use a new output path for a revision.

The JSON input contains `deck_id` (positive integer), `deck_name`, `language` (source language), optional `direction` (`auto`, `ltr` or `rtl`, default `auto`), and a nonempty `cards` array. Choose a unique deck ID for a new deck and retain it for revisions. For additions, use the inspected destination's name and ID. The builder uses its own versioned Language Jutsu models; it cannot adapt arbitrary existing note-type schemas. Reuse compatible destination note types as required above instead of silently replacing them with these models.

Each card requires `key`, `kind` (`vocab`, `grammar` or `listening`), `prompt`, `answer`, `sentence` and `context_audio`. Optional fields:

| Field | Meaning |
| --- | --- |
| `marks` | Literal, nonoverlapping fragments to emphasize in the full sentence; each must occur exactly once. Several fragments can mark one discontinuous construction. |
| `front` | Plain text replacing the default front text. Vocab/Grammar otherwise show the marked sentence. Listening otherwise shows no transcript; supply a one-word cloze here when appropriate. |
| `support` | Vocabulary or other help on Vocab/Grammar fronts. Never shown on Listening fronts. |
| `translation`, `reading`, `explanation`, `situation` | Back support; omitted fields are empty and their sections hidden. Supply the support required by the lesson and level. |
| `listening_audio` | Listening front clip when different from `context_audio`; otherwise the context clip is used. Only valid for Listening. |

All written content is plain text, escaped by the builder. `marks` adds emphasis; do not put HTML in fields. Author prompts in the established support language. Audio paths resolve relative to the manifest, or may be absolute. Use unique media basenames across lessons. The builder embeds referenced files, deduplicates reused paths and rejects different files sharing a basename. The full context audio appears on every back. Full-sentence Listening answers equal to `sentence` are shown only once.

Minimal input shape (replace this illustrative content and path with the actual lesson):

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
    "sentence": "She is looking for her keys.",
    "marks": ["looking for"],
    "translation": "Ella está buscando sus llaves.",
    "context_audio": "lesson-01-context.mp3"
  }]
}
```

Keep each card's `key` unique within its deck and stable across wording revisions. The note GUID is derived from the deck ID and key, so editing an answer does not create a new note. A new learning point needs a new key; keep the type stable for a continuing note. The manifest records deck identity and keys; model IDs are fixed in the builder. Invalid fields, keys, media or marks fail before publishing a package.

## Review and deliver

Return to the actual lesson, including its activities, and find the card question that elicits each taught or practised vocabulary and grammar point. Check the question and expected answer, not just a stored target or a matching glossary count. A point appearing only in another card's sentence, translation or explanation is not covered. Add missing cards and split overloaded tasks; do not shrink the lesson to match an incomplete deck. Keep this correspondence current when the lesson changes.

Use every final front before its back. Determine what the learner is being asked to recall using only that front, attempt it at their level, then check whether the back answers exactly that task with the required sentence, translation, explanation and audio. Apply the type-specific atomicity above to the actual question and answer, not its label or metadata. Remove unnecessary framing while preserving context and coverage. Repair failures and review the complete revised lesson and deck together.

Import the exact package into a disposable Anki collection and use its rendered cards to check the chosen deck, script direction, readable context, answer leakage and working audio. When adding to an existing deck, use a copy of that actual destination and confirm its existing cards, media, settings and study history remain unchanged. When delivering a revision, confirm it updates only the intended notes. Do not construct hypothetical prior decks or test unrelated app behavior. Package checks establish mechanics, not learning quality; do not test by modifying the user's live collection.

Save the package, manifest with its stable identities, and audio/source links under the lesson directory; record the new/existing choice, destination and package path in `task.md`. Deliver the `.apkg` with its destination, counts of added cards by type and a brief instruction to import it using Anki's **File → Import**. Do not claim a completed learner review session or user acceptance from authoring and import checks.

## Research basis

The Japanese Deck and its vocabulary/grammar skills informed contextual meanings, independent cards, succinct explanations and learner review. This multilingual workflow is self-contained and uses this lesson's language and card requirements.

[Wozniak's practitioner guidance](https://www.super-memory.com/articles/20rules.htm) supports understanding before memorization and simple recall items. [Anki's package documentation](https://docs.ankiweb.net/manual/importing/packaged-decks) explains import and revision behavior. These citations record the basis for the guidance above; they are not additional research tasks for lesson preparation.
