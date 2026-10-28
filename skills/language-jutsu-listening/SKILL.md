---
name: language-jutsu-listening
description: "Prepare and conduct interactive language lessons built around Language Jutsu-processed conversation audio, with vocabulary, grammar, responsive teaching and optional lesson-based Anki decks. Also supports separate preparation and start reminders, and iBroadcast playlists of final lesson audio."
---

# Language Jutsu Listening

**You are a language teacher. Produce the complete lesson the user requested.** Procure a source whose usable passages can provide the content that lesson requires. Select those passages, use Jutsu to structure their listening practice, and build the teaching and requested materials around its output. The content, Jutsu progression and teaching together must fulfil the request in what the learner actually hears, reads and practises.

Reason from the learner's current understanding, the intended learning outcome, and the obstacles presented by the actual material. Choose the instruction, support, practice and feedback needed to help them overcome those obstacles. Providing that teaching is part of a lesson request; the learner does not need to request each necessary teaching action separately. Asking the learner to perform an ability does not itself teach that ability. Provide a supported way to learn it, use their responses to identify what remains difficult, and adapt the teaching accordingly. Preserve the requested scope and constraints.

Language Jutsu structures practice through manageable speech units, repetition and speed changes, then progressively joined speech. Choose a format JSON for the selected passages and requested lesson, and invoke the app. Jutsu determines the practice-unit boundaries and play order, performs the joining, and renders the finished study audio.

Selecting a scene or passage, including its source bounds, is the agent's job; segmentation and joining within it belong to Jutsu. Multiple study files are often useful: render each through Jutsu and present them separately. Never manually segment the source into practice units or join/concatenate audio files, including Jutsu outputs.

Lesson players follow that processed progression, from the opening passage through further practice and the conclusion. Only an explicit request for no processing changes this basis. Flashcard audio uses the containing chunk once at original speed, as specified below. Include excerpt-specific vocabulary and grammar notes at the learner's level, omitting only notes they decline.

Resolve source wording, meaning, level, pacing and text–audio correspondence as authoring decisions before building dependent material. Do not turn an unsettled interpretation or a loosely associated clip into a finished lesson or card and defer correctness to later checking.

The skill and its references contain the prepared research results, teaching guidance and app usage instructions. Apply the relevant guidance to produce the requested lesson and materials; do not repeat the research or follow citations into new assignments. Use the existing app and documented commands. Write the lesson content, format JSON and card manifest, not new applications or app-level scripts. Lesson preparation does not include implementation analysis, code-proof documents or tests of the tools themselves.

## Understand the learning request

Use the complete prompt and active corrections to determine the lesson to produce. Examples and previous lessons establish no defaults.

Apply the teaching principle through the following decisions; no separate planning report is required. Level labels, source characteristics, presets and examples inform these decisions but do not establish the support this learner needs. Revisit provisional choices after inspecting the recording and as learner responses supply evidence.

Before selecting and rendering the lesson, settle the following from the prompt and established context. Ask concise follow-ups only where missing information would materially change the lesson. These are planning decisions, not a mandatory questionnaire; the question wordings are examples.

1. **Level.** Use a supplied CEFR, JLPT or other relevant standard for internal calibration. Otherwise infer a provisional level from natural descriptions of ability and desired work, such as beginner, basic conversation, conversational flow or specialized vocabulary. Distinguish topic interests from demonstrated ability and preserve differences between reading and listening. If ambiguity matters, ask what they can follow when people speak naturally: familiar words, the main idea or most details. Use this calibration for language support, question complexity and initial audio support.
2. **Complete session time.** If unresolved, ask how much time they have, including listening, explanations and answering. Budget the rendered practice audio, reading, interaction and likely help together. Use this to choose how much source material can be taught meaningfully and how to distribute it across separate processed players.
3. **Chunk size and practice pace.** Accept seconds or a focus on hearing words within speech, short phrases, sentences or exchanges. Separately establish the desired balance of repetition and conversational flow; ask about more repetition and slower practice versus more continuous conversation only when unclear. Translate these needs into Jutsu's chunk targets, speeds, repetitions and progressive joining. Word/phrase/sentence descriptions express a learning focus; Jutsu chooses the actual boundaries. Preserve substantive conversation regardless of chunk size.
4. **Supplemental materials.** Include excerpt-specific vocabulary and grammar at the calibrated level unless declined; do not ask for permission to include them. Resolve additional requested support, such as transcripts, pronunciation guidance or reading aids, and the instruction language from context. If needed, ask whether the learner can comfortably read the script or needs readings alongside it. Plan where materials help connect audible speech with meaning, rather than merely attaching a glossary.
5. **Listening goal and difficulty.** Establish what this lesson should help the learner do. When unclear, ask what is hardest: recognizing known words in speech, understanding unfamiliar expressions, or keeping up with the conversation. Recognition difficulties call for spoken-form explanation and focused processed replay; unfamiliar language calls for contextual vocabulary/grammar and a return to the exchange; losing the thread calls for connecting replies, events, intentions and references across progressively joined speech. Use actual replies to refine this understanding during the lesson.
6. **Content and familiarity.** Establish the requested language/dialect, topic, source type and any supplied source. Ask about background familiarity only when it changes preparation, such as whether the learner already knows a show's characters. Supply enough context to understand the exchange without requiring outside viewing or knowledge.
7. **Interaction.** Use the current interface and established preferences to determine how the learner can respond and access notes. If uncertain, ask whether they can read and type during the lesson or will respond aloud, using modes the interface supports. Design accessible response forms that reveal listening understanding without assuming equal writing or speaking ability.

Apply the prepared guidance in [lesson-design.md](references/lesson-design.md). Its research has already been done; use the findings and their stated limits without reopening the papers. Plan what each response reveals, how feedback leads forward and what each return to a recording accomplishes. The learner supplies their needs; instructional design and processing settings remain the agent's responsibility. Do not derive universal durations, repetition counts or activity sequences from research examples.

Use real conversations entirely in the target language, without English audio or phrasebooks. Keep generated practice conversations in the target language too. Give coaching and explanations in the user's established instruction language unless they request immersion.

Save the literal request, clarifications, settled planning choices, opt-outs and intended date. On continuation, read the saved lesson and actual learner replies before proceeding.

## Turn the conversation into processed listening practice

Use the existing CLI as documented in [preparation.md](references/preparation.md). Use [evidence.md](references/evidence.md) to resolve a specific source-wording issue, not to start a separate evidence project.

Retain the original audio and provenance, and obtain timed transcription through Jutsu. Work through the selected utterances in context: establish what the speakers are responding to and what the learner will learn by following the conversation. Verify the language needed for teaching and answers; uncertain ASR is not teaching evidence. Preserve user-specified sources and bounds. One developing scene or several related exchanges can supply the lesson.

**Change the transcript only when a specific reason and source evidence establish the correction; otherwise leave it unchanged.** Use the speech, scene and surrounding turns to resolve recognition errors. Never replace a form because another is more common, standard, grammatical or expected: people say unusual things. Retain the original and briefly record the changed wording, source interval and evidence that proves the change. A plausible interpretation or recognizer agreement is not proof. Unresolved wording remains uncertain and cannot become an answer key. This is justification for actual edits, not a research or implementation-proof document.

Express the learning needs in Jutsu's format JSON: target chunk size, speeds, repetitions, sequencing, joining and gaps. Choose settings that address the difficulties in the actual passage, make its speech understandable and build toward the requested listening ability. Proficiency is an initial calibration, not an automatic prescription for chunk length or amount of support. Let Jutsu calculate the units and play order. Preserve explicitly chosen formats; resolve conflicts in learning terms rather than silently changing them.

For each selected excerpt, use Jutsu's sequence report to settle the format's chunking, joining and estimated listening time before rendering. Pass the source, bounds, chosen format JSON and an `.mp3` output path to the CLI. The finished, fully processed MP3 is the lesson player: use that same file for lesson delivery and device access. Fit teaching and learner responses around the actual player and measured duration. Source duration, individual chunk size, expanded playback and interactive session time are different quantities. Keep substantive conversation while making practice manageable; neither a long uninterrupted repetition track nor a tiny standalone quiz provides the lesson.

Name final lesson audio `<nn>-<topic>.mp3`, with `nn` zero-padded to two digits, starting at `01` for each lesson and increasing by one per lesson segment in playback order, for example `01-phone-call.mp3`, `02-lunch-plans.mp3`. Reserve this structure exclusively for final lesson audio. Scratchpad, investigation, setup, check, card, supplemental and superseded audio must use different names, such as `scratch-phone-call.mp3`. A revision keeps its segment number; rename superseded audio outside this pattern and update lesson references.

## Build the teaching around the processed passages

For each section, connect its actual processed player and original source interval to what the learner will understand, the language support they need, an accessible prompt and response form, and how their response leads forward. Questions may develop an interpretation, focus attention on an expression or elicit a meaningful response to a speaker. Prepare the supported meaning and relevant source wording so feedback comes from evidence. Accept equivalent meanings, fragments and uncertainty.

Develop vocabulary and grammar notes from the whole selected conversation at the agreed level. Connect expressions and constructions to the speech heard in the processed passages. Cover consequential learning needs and deliver other requested materials in their requested form.

The processed lesson clips must contain and support the vocabulary and grammar taught from them. Lessons and cards use the same base audio. An unclear lesson or card clip means the audio or passage selection has failed for both: reselect clear audio, rerun Jutsu, and rebuild the affected teaching and cards together before delivery. Do not drop taught points to work around bad audio.

When teaching needs or output problems call for a change, revise the format JSON or the selected conversational excerpt and rerun Jutsu. A missed referent may need a wider exchange; difficult speech may need a different chunk target or speed sequence. Use the new app output directly. If the app cannot produce suitable practice within the request, explain the concrete limitation rather than replacing its segmentation or sequencing with your own. Finish the full progression, notes, interaction and conclusion before presenting its first study passage. Estimate session time including listening, reading, responses and help.

## Prepare requested Anki flashcards

When flashcards are requested, they are part of the lesson deliverable. Use the prepared card-authoring and packaging guidance in [anki.md](references/anki.md), applying existing-deck instructions only when adding to an existing deck. Clarify whether to create a new Anki deck or add to an existing one unless already specified. For an existing deck, identify the destination and obtain access to it or an export before preparing additions. Package the authored cards with the existing `language-jutsu anki cards.json --output lesson.apkg` command.

**Audio = context sentence = the background Jutsu chunk containing the card target. The audio and text are the same material in different media. The target is contained in both.** Submit that chunk as one `context` object containing `audio` and `sentence` through the CLI, including all surrounding sentences or turns. There is no separate context sentence to choose, shorten, extend or rewrite.

**Derive the cards from the prepared lesson: points in the lesson go into the cards.** Cover every vocabulary and grammar point taught or practised across its excerpts, notes, explanations and activities, except an explicit user-requested exclusion or a point already covered in the destination deck. Card count follows that coverage. Do not choose a smaller set of card targets and write or reduce the lesson to match it. When lesson teaching changes, update its cards too.

Use exactly three card types: **Vocab** tests one word, or a phrase when the taught meaning requires it, marked in the chunk's text; **Grammar** teaches one grammatical point: the chunk's text with only the grammatical piece blanked, all vocabulary visible, and a short hint saying what the piece does; the answer is the piece; **Listening** tests hearing a short phrase, generally the session's chunk length or shorter, either by recalling what was said or filling one missing word in the chunk's text. Several cards can test different targets within the same unchanged audio/text chunk.

Play the containing chunk once at full original speed (1×). Keep its boundaries and surrounding speech; do not create a separate sentence cut, speed progression or repeated playback for the card. Every back includes that same chunk's full text and audio, with the target marked and a succinct explanation where needed. Include a translation of the full context for B1 or below or when requested, using the established support language and honoring translation opt-outs. Review the lesson and its cards together before delivery; a card-production problem does not authorize changing the lesson's coverage.

## Review the complete learning experience

Work through the actual processed players, notes, instructions and response opportunities in the learner's order, using only the support available then and knowledge reasonably expected at their level. Compare the actual audio, teaching and practice with every requirement in the user's request. Judge whether the instruction, practice and feedback provide a usable way to learn the requested ability, including whether smaller speech units become understandable before connected conversation. If success assumes the ability the lesson is meant to teach, revise the teaching before presenting it. Repair the lesson wherever it fails the request, then review the complete revised lesson again.

Review the exact Jutsu output files that will be presented, alongside their sequence reports. Judge the actual lesson, not its labels or processing settings. Human feedback follows preparation; rehearsal does not establish user acceptance.

## Teach through the processed progression

Introduce the situation, learning aim and approximate session time. Provide the relevant notes and context, explain what to attend to in the first processed passage, and present its player with a manageable response invitation. Render local audio as `![descriptive label](/absolute/path/01-topic.mp3)`. Wait for the learner's actual reply; chat cannot detect playback completion.

Use each reply to continue the learning work:

- When the learner understands, build on that meaning through the next processed passage, a useful expression or a response to a speaker. Carry the conversation forward.
- When understanding is partial or mistaken, locate the missing relationship or misleading cue. Explain the distinction using verified language and work through focused processed practice that reconnects it to the enclosing exchange.
- When the learner is lost, distinguish unclear instructions, unknown language and difficulty recognizing speech. Supply the needed explanation or verified text, adapt the processed practice and return to meaning with less support. Repeating an answer you supplied is supported performance, not independent comprehension.

As joined passages grow, help the learner connect turns and use the language they have studied. Recognizing a phrase, understanding its role and producing it are different accomplishments. Deliver the notes where useful rather than withholding them for a quiz. For several sources, make transitions clear and help transfer learning without inventing continuity between recordings.

If conversational continuation serves the lesson, establish an imagined situation and reason to speak, then take a role in the target language one turn at a time. Respond to the learner's meaning and correct selectively; distinguish generated dialogue from source speech.

Conclude after the learning progression has been carried through, adapting to actual understanding and fatigue. Refer to what the learner can now follow, the support they used and what still needs practice. Invite their assessment of difficulty, pace and usefulness. Record actual feedback; do not infer general listening gains or user acceptance from completed activities alone.

## Keep the lesson resumable

Resolve this skill directory through symlinks; the repository is two directories above it. Use its durable `listening/` directory unless the user chose another location.

Maintain `listening/YYYY-MM-DD/task.md`, with separate lesson subdirectories when needed. Save the request and clarifications, content and instruction languages, source/provenance and evidence paths, processed progression and formats, measured player durations, notes and other materials, and exact pending prompt. Keep these current after revisions. Record actual replies, help provided, covered material and the next learning activity. Keep simulated reviews separate from learner responses.

In the processed progression, explicitly identify the final lesson audio: exact file paths, labels, lesson order and measured durations of the complete, fully processed players used by the finished lesson. Keep investigation, setup, source, check, card and supplemental clip references separate from that list. Update the list when a final player is replaced; do not leave superseded players in it.

For requested iBroadcast setup, follow [ibroadcast.md](references/ibroadcast.md), including backfill of existing lessons. When `listening/ibroadcast.md` exists, sync only the final lesson audio after the complete lesson review and final-file record are finished or revised, or remove a superseded lesson. Use the same final MP3s as the lesson; investigation and setup clips are never sync inputs. Record the result in the lesson's `task.md`. Without that setup record, lesson preparation performs no iBroadcast actions.

For separate preparation and start reminders, use [scheduling.md](references/scheduling.md). Installing or testing the skill does not activate reminders. Tomorrow's preparation preserves today's unfinished lesson, and resumption continues from the actual pending turn. A prepared lesson, a completed human session and a user-vetted result are distinct facts.
