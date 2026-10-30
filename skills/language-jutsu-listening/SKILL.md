---
name: language-jutsu-listening
description: "Prepare and conduct interactive language lessons built around Language Jutsu-processed conversation audio, with vocabulary, grammar, responsive teaching and optional lesson-based Anki decks. Also supports separate preparation and start reminders, and iBroadcast playlists of final lesson audio."
---

# Language Jutsu Listening

**This is an explicit procedure. Follow the numbered steps in order.** Complete each applicable step before proceeding. At a linked procedure, execute its steps and return here. Skip a step only when its stated condition is false or the user explicitly changes it. Make the choices each step calls for; do not substitute another workflow.

The steps depend on one another. Review each result against the active user requirements and the relevant skill guidance before using it in the next step. If later work exposes an earlier failure, return to that step, discard work that depends on the failed choice, and rebuild and review it. Review belongs within each step, not in a separate final pass.

Use the existing Jutsu app through its documented commands. The research in the references is already complete: apply its findings. Do not redo it, inspect or reimplement the app, write replacement tools, or turn lesson preparation into software development.

## 1. Open the lesson record and settle the request

Resolve this skill directory through symlinks. Set `skill` to its absolute path and `jutsu` to `$skill/scripts/jutsu`. The installation or repository root is two directories above it:

- Mac installer: if the root contains `bin/language-jutsu`, use `runtime/formats/` under that root and `~/Documents/Language Jutsu/listening/` for lessons.
- Source checkout: use `formats/` and `listening/` under the repository.
- Windows source checkout: replace `"$jutsu"` in command examples with `cargo run --quiet --manifest-path <repository>/Cargo.toml --` in PowerShell; the bundled runner is Unix-only.

Set `formats` to the absolute preset directory. Honor a user-chosen lesson location. All `listening/` paths below and in references are relative to the lesson base's parent.

Read the dated `listening/YYYY-MM-DD/task.md` when it exists. On continuation, read the saved materials and actual learner replies, then resume the first unfinished step. For a revision, begin at the step whose input changed and redo its dependent work. For playlist management or iBroadcast setup only, start at step 7 with the existing lesson records. For reminder setup only, execute [scheduling.md](references/scheduling.md) and report its result. These requests do not start a new lesson; a lesson request does not activate reminders.

Apply [lesson-design.md](references/lesson-design.md) to settle these choices from the complete request and established context:

- Level and listening goal; distinguish listening, reading and responding abilities, and identify whether the obstacle is recognizing speech, unfamiliar language or following the conversation.
- Total session time, including audio, explanations, responses and help.
- Language, dialect, topic, source, explicit bounds and relevant background familiarity.
- Speech-unit focus and the balance of repetition, slower practice and conversational flow.
- Requested materials, instruction language, translations or reading aids, and how the learner can access notes and respond in the current interface.

Ask only for missing information needed to make these choices. The learner supplies their needs; you choose the teaching and processing. Examples, presets and prior lessons establish no defaults. Include vocabulary and grammar unless declined. If cards are requested for an existing deck, establish its exact identity and inspect an authorized copy or export as directed in [anki.md — Choose the destination](references/anki.md#choose-the-destination) before authoring additions.

Save the literal request, clarifications, settled choices, opt-outs and intended date in `task.md`, using a lesson subdirectory when needed. Update this record as each following step finishes or changes.

## 2. Select a conversation for teaching and establish its wording

Choose the source for the requested teaching: identify what this learner will learn from its speech and developing exchanges, what support they will need, and how that can fit the session. Use real conversation entirely in the target language, without English audio or phrasebooks. Apply the lesson-design guidance from step 1 when selecting it.

Follow [preparation.md — Get the requested conversation](references/preparation.md#get-the-requested-conversation) and [Produce source-language text](references/preparation.md#produce-source-language-text). Retain the original recording and provenance, and obtain timed transcription through Jutsu.

Establish what the speakers are responding to, the relevant referents and the language needed for teaching and answers. Transcription is a hypothesis, not an answer key.

For a specific wording problem, follow [evidence.md](references/evidence.md). Preserve the original transcript. Correct it only when a specific reason and source evidence establish the change; record the source interval, before/after wording and evidence briefly with the working transcript. Familiarity, grammaticality, plausibility or recognizer agreement does not prove a correction. Preserve unusual source wording. Do not use unresolved wording in teaching or answers.

Review the original-speed recording and established wording against all active requirements and the planned teaching. Confirm that the conversation supports the required vocabulary, grammar and connected meaning with help appropriate to this learner. If a source or passage you selected remains inaccessible or unsuitable, select a suitable replacement and repeat this step. Preserve user-specified sources and bounds; ask for a change only when necessary. Establish teachability and resolve the wording before building dependent material.

## 3. Render the listening progression through Jutsu

Follow [preparation.md — Render the study passages](references/preparation.md#render-the-study-passages). Choose the conversational excerpt bounds and a format JSON suited to this learner and passage: chunk target, speeds, repetitions, sequencing, joining and gaps. Preserve explicit user choices. Save the chosen format in the lesson directory.

Run the sequence command first. Inspect its units, joins, plays and estimated duration; adjust the format or excerpt if needed. Then render each passage through Jutsu to a separate scratch MP3 and read its measured duration. **Jutsu chooses every internal practice boundary, join and playback order. Never manually segment or concatenate audio.**

Use the complete processed players throughout the lesson, including the conclusion, unless the user explicitly requests no processing. Budget their actual durations together with reading, answers, feedback and help. Preserve substantive conversation; a tiny quiz or a long uninterrupted repetition track is not the lesson.

Review the Jutsu format and actual players against the learner's requirements and the teaching purpose established in step 2. Use the documented audio tools to check the outputs against the source, transcripts and sequence reports: speech clarity, text correspondence, cuts, repetitions, speeds, joins and measured duration. Judge whether the progression supports the intended learning within the session time before building the teaching around it.

If a command fails, resolve that failure with the documented tools and repeat it. If its output is unsuitable, revise the format or agent-selected passage and repeat the sequence/render procedure. Return to step 2 if the source must change. Use the resulting app output directly.

## 4. Build the complete teaching around those players

For each passage, prepare its situation, learning purpose, vocabulary and grammar support, response invitation, supported answer and responsive feedback. Connect these to the actual processed player and original-source interval. Accept equivalent meanings, fragments and uncertainty.

Teach a supported way to understand the speech before asking the learner to demonstrate that ability. Supply unfamiliar language in context; explain spoken-form recognition difficulties; help connect replies, events and intentions as the speech units join. A glossary or a comprehension question alone does not provide this teaching.

Cover consequential vocabulary and grammar across the whole selected conversation at the learner's level. The processed clips must contain and support every point taught from them. Prepare the full progression and conclusion before presenting the first player. Keep coaching in the established instruction language and any imagined practice dialogue in the target language; label generated dialogue as imagined.

Review the lesson plan against all active user requirements and [lesson-design.md](references/lesson-design.md). Work through its players, notes, instructions and response opportunities in learner order, using only the support available at each point. Judge whether it teaches this learner to understand smaller speech units and then connected conversation within the complete session time. Repair failed teaching and repeat this review before making cards.

A terminal teaching flaw—where the source cannot support the requested lesson—means source selection failed. Return to step 2, choose a suitable new source and restart the dependent preparation. Do not patch around that failure, reduce the teaching goals or keep unsuitable material because work has already been done. Correct source selection must establish teachability before reaching this step.

## 5. Prepare the requested flashcards

If flashcards were not requested, continue to step 6. Otherwise follow [anki.md](references/anki.md) in order: inspect the destination when applicable, cover the lesson's learning points, author the questions, prepare chunk audio, independently review and repair the prompts, then package.

Cover every vocabulary and grammar point taught or practised, except explicit exclusions and points already covered in the destination deck. Use exactly Vocab, Grammar and Listening. Test language proficiency rather than sentence memorization; grammar prompts must determine the answer for a competent user unfamiliar with the source.

**Each card's audio and context text are the same complete Jutsu chunk containing its target.** Use that chunk's original-source bounds, play it once at 1×, and retain all its surrounding speech in the text. Submit it as one `context: { audio, sentence }` object. Do not make a separate sentence cut or shortened context. Follow the reference's back-support and translation requirements.

Review the cards against the active user requirements and all requirements in [anki.md](references/anki.md), using the reviewed lesson to check coverage and consistency. Before Anki formatting, have independent reviewers attempt each front in a fresh context with notes and answers withheld. Compare their actual answers and valid alternatives, repair and retest failures, and review notes separately as concise language instruction. Review actual card audio against its complete written context. If this reveals a source, format or teaching failure, return to that step and rebuild the dependent lesson and cards before continuing.

Then package with `"$jutsu" anki cards.json --output lesson.apkg`. Preserve existing-deck identities, compatible templates, notes, media, settings and study history; package only the additions.

## 6. Save the reviewed lesson

Rename each reviewed final MP3, without re-encoding, to `<nn>-<topic>.mp3`, starting with `01` and increasing in lesson order. Reserve that pattern for final players only. A revision keeps its segment number; move superseded audio outside the pattern and update links.

Save the complete teaching, requested materials and exact pending learner prompt. In `task.md`, record provenance, evidence paths, formats, render/sequence reports, and the ordered final-audio list with absolute paths, labels and measured durations. Keep source, investigation, scratch, check, card and supplemental audio outside that final list. Record the deck destination, package path and counts by type when cards were requested.

## 7. Sync the final players when iBroadcast is enabled

If setup was requested, follow [ibroadcast.md — One-time setup](references/ibroadcast.md#one-time-setup), including its backfill. Otherwise sync when `listening/ibroadcast.md` exists; if neither condition applies, continue to step 8.

Execute [ibroadcast.md — Sync final players](references/ibroadcast.md#sync-final-players) in its numbered order. Start from the finished lesson and final-file record from step 6. Use the same final MP3s unchanged. Follow its upload, metadata, album verification, lesson-playlist and language-playlist sequence. A failed browser interaction calls for the tool's documented recovery for that interaction, then continuation of the same step; it does not change the playlist procedure.

Record the actual sync result and any unfinished action in the lesson's `task.md` and update `listening/ibroadcast.md`. Do not claim an upload or device check that was not confirmed.

## 8. Deliver and teach one turn at a time

Deliver requested artifacts. For Anki, link the `.apkg`, name its destination and counts by type, and give the instruction **File → Import**. Link confirmed playlists. Preparation alone does not authorize modifying the live Anki collection.

If the request was preparation or playlist management only, report the result and leave the saved learner prompt pending. Otherwise begin or resume teaching below.

Introduce the situation, learning aim and approximate session time. Provide the first passage's notes, explain what to attend to, and render its player as `![descriptive label](/absolute/path/01-topic.mp3)` with a manageable response invitation. **Wait for the actual learner reply; chat cannot detect playback completion.**

Continue from each reply:

- If understood, build on that meaning and move to the next processed passage or conversational turn.
- If partly understood or mistaken, explain the missing relationship or misleading cue using verified language, then return to focused processed practice within the exchange.
- If lost, distinguish unclear instructions, unfamiliar language and speech recognition difficulty; supply the needed support, adapt practice through steps 2–3 when necessary, and return to meaning with less support.

As joined passages grow, connect the turns and help the learner use the language. Distinguish recognition, understanding and production. Make transitions between sources clear. If using an imagined conversation, establish the situation, take one target-language turn at a time and correct selectively in response to the learner's meaning.

Conclude after carrying through the progression, adapting to actual understanding and fatigue. Describe what the learner followed, the help actually used and what still needs practice; invite feedback on difficulty, pace and usefulness. Record actual replies, help, covered material and the exact next activity in `task.md`. Keep simulated review separate: a prepared lesson is not a completed human session or evidence of user acceptance.
