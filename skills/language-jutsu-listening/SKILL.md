---
name: language-jutsu-listening
description: "Prepare user-requested conversational audio with Language Jutsu, guide listening through evidence-grounded questions and contextual replays, and manage separate preparation and start reminders."
---

# Language Jutsu Listening

Turn the user's requested conversation into a complete listening task, then help them interpret it through their responses. Keep all learner-facing prompts, feedback, and study audio in the requested target language, without English or phrasebook content.

## Start from the actual request

The user chooses language, dialect, topic, level, source, interval, and explicit processing format. Preserve those choices. Examples in documentation supply no defaults or assignments. Extract the literal request before searching or rendering; ask only for a missing fact that prevents preparation. Reuse established preferences only when they are actually applicable.

Resolve this skill directory through any symlink; the Language Jutsu repository is two directories above it. Use a durable `listening/` directory there unless the user has chosen another location. Save the repository, lesson-root, Python, and model paths in `listening/settings.md`; use absolute paths for later scheduled runs. Read the current CLI help and [preparation.md](references/preparation.md) when preparing or repairing audio.

- **New content request / preparation reply:** Prepare for the date named by the user or pending preparation reminder; an immediate practice request is for today. Save the literal request and date before working.
- **Start / continue listening:** Read that date's `task.md`, evidence, and pending prompt. Continue the saved interaction; do not download or render already prepared material again.
- **Reminder setup or changes:** Read [scheduling.md](references/scheduling.md). Skill installation alone does not activate schedules. Use the authorized schedule setup request to create the two separate reminders.

## Prepare a usable complete task

Acquire the requested conversational recording and retain the original for tomorrow and later repairs. Find publisher text for that exact recording, and use Language Jutsu's MLX Whisper transcription for audio-derived text and source timing. Build answer evidence before writing questions. Read [evidence.md](references/evidence.md) for missing text, disagreements, or an audio-model comparison.

Choose a coherent conversational exchange when the request permits selecting an interval. Resolve any named format to its actual JSON; never silently replace a user's format. If processing is unspecified, choose and describe an appropriate design for this exchange. Run sequence planning and render with the same source, interval, and format. Read the measured complete duration, joins, repetitions, speeds, and gaps. A thirty-minute practice track is not a workable task. Change assistant-controlled choices and rerender when necessary; if explicit user constraints conflict, report the concrete conflict and ask which may change. Skipping, early stopping, or substituting an original-only player cannot rescue a failed practice track.

Before marking ready, use the complete exercise as the learner would: full main track, opening prompt, plausible correct/partial/incorrect/help responses, and complete contextual repairs. Check that the question is answerable from the selected conversation and that the response form is achievable. A successful render is not enough.

## Conduct the interaction

Present one main audio player with its actual approximate duration and a brief target-language explanation of its parts and joined passes. **Before playback**, give a specific listening purpose and response frame: for example, assign the speakers' choices, reconstruct a route and time, or explain what changed between two turns. Pick the task from this recording's supported meaning; those examples are not a fixed menu. Permit an unknown marker. Keep the answer and transcript hidden initially.

Ask the learner to respond after the complete track. A reply starts the next chat turn; do not claim to detect playback completion. Render local audio as `![target-language label](/absolute/path/audio.wav)`.

Compare the learner's expressed meaning with the saved evidence, accepting equivalent wording:

- **Supported answer:** Acknowledge it, then elicit a relevant connection or supporting expression when that advances this task. Do not turn every successful answer into another quiz.
- **Partial answer:** Acknowledge the supported part. Ask one narrower question about the missing relationship and deliver its enclosing question/answer replay.
- **Incorrect answer:** Identify the specific mismatch without inventing why they misheard. Replay the relevant surrounding turns and elicit the correction before summarizing it.
- **Needs help:** First distinguish an unclear prompt from unclear audio. Simplify the question; replay the enclosing exchange. If needed, show the relevant verified target-language words and meaning, replay with that support, then ask the learner to connect the answer back to the conversation. Record that text support was given; it does not prove unaided recognition.
- **Uncertain evidence:** Investigate the recording again using the evidence procedure. Do not mark the learner wrong on an unresolved detail or require them to supply your missing transcript.

For a reported listening difficulty, locate it in the transcript and original timeline, render the contextual replay through Jutsu, and elicit an interpretation of that exchange. If the location is unclear, ask for a word, speaker turn, or approximate place; do not fabricate a timestamp.

When continuing the scene would help or is requested, explicitly introduce a **new, imagined conversation** in the target language. Take a role and respond to what the learner actually says, one turn at a time. Assess intelligibility and relevance; invented dialogue has no source answer key. Never add it to the recording's transcript.

## Save the handoff

Maintain `listening/YYYY-MM-DD/task.md` with normal file tools. It is an assistant-readable record, not an application API:

- Literal request, date/timezone, target language/dialect, and readiness or the specific unresolved failure.
- Source URL/page and retained local original; selected original interval; actual format; transcript and evidence files.
- Main player path, render metadata path, complete measured duration, and the exact opening/pending learner prompt.
- Each answer proposition's original evidence and contextual replay bounds; acceptable paraphrases and unresolved details.
- Latest actual learner response, help already supplied, and next turn. Keep any simulated test responses explicitly separate.

Keep answer notes private during initial elicitation. After each learner turn, update the pending prompt and support record so the next scheduled event or resumed chat can continue accurately. Tomorrow's preparation must not overwrite today's unfinished task.
