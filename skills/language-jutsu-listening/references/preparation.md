# Build the processed listening progression

Use the existing Language Jutsu app and the commands below; use `--help` for a missing option detail. The app already implements transcription, segmentation, sequencing, joining, rendering and Anki packaging. Do not inspect or reimplement those operations to prepare a lesson. Resolve `skill`, `jutsu`, `formats`, and the lesson base as described in SKILL.md. The commands below use absolute paths resolved into shell variables: `jutsu`, `lesson`, `source`, `language`, `format`, `start`, `end`, `whisper_model`, and `skill`. They are invocation patterns, not values to copy from a scenario. Add `--dialect` only when the request supplies or establishes one. When no interval is selected, omit `--start` and `--end`.

## Get the requested conversation

For discovery, search for material for the requested lesson, open the publisher/media pages, and inspect the passages you intend to use at original speed, with their transcript for language and context. Determine whether those passages can supply the experience the lesson requires. Titles, automated labels and transcripts alone cannot establish audible qualities.

For supplied content, preserve its recording identity and explicit bounds. If selection is permitted, retain meaningful question/answer exchanges. One recording may supply several players; several recordings may supply a lesson. Assess their combined teaching substance and keep each source identifiable. When a recording conflicts with a requirement that cannot be repaired by an allowed selection, explain the conflict instead of silently changing the source.

Jutsu's `--url` is for a single supported YouTube video URL. Acquire and transcribe once while retaining the full downloaded original:

```sh
"$jutsu" \
  --url "$url" --save-source "$lesson/original.audio" \
  --language "$language" --transcribe \
  --whisper-model "$whisper_model" \
  --output "$lesson/transcript.txt"
```

The retained original and its `.source.json` companion stay together. Use that local original for all later commands; the sidecar restores its remote provenance. Do not use a temporary download path from an earlier invocation as tomorrow's source.

For a long episode with a known relevant interval, add `--start` and `--end` to that acquisition/transcription command. `--save-source` still retains the full original for later selections; transcription then processes the selected interval. Repeat acquisition only for genuinely different sources.

For a public publisher/RSS/direct-media URL, resolve the actual audio link from the full page or feed, download that recording into the lesson directory, and record submitted page, final media URL, publisher identity, and download date in `sources.md`. For a supplied local file, use a retained copy or a stable original path. Do not pass a generic page or Spotify URL to the YouTube-only parser or use a preview as the full requested recording. Investigate the provider's actual public media route; an acquisition failure does not authorize substituting a user-specified recording. Replace an inaccessible agent-selected source with suitable material and continue.

## Produce source-language text

Transcription runs whisper.cpp inside the Jutsu executable. `whisper_model` is a model file in the ggerganov/whisper.cpp Hugging Face repository, `ggml-large-v3-turbo.bin` unless the lesson needs another; Jutsu reads it from the Hugging Face cache and downloads it there when missing. The Jutsu README gives setup. Test the configured runtime by actually transcribing the lesson.

```sh
"$jutsu" "$source" \
  --language "$language" --start "$start" --end "$end" --transcribe \
  --whisper-model "$whisper_model" \
  --output "$lesson/transcript.txt"
```

The text file has a structured `.json` companion: `text`, `language`, `segments` (with `words`), `source`, `backend`, `settings`, `timestamp_reference`, and `source_offset_seconds`. Segment/word `start` and `end` are already original-source seconds. Never align against the expanded practice track or add the offset a second time. Transcription preserves the spoken language; do not request translation. Whisper does not supply speaker identities, questions, or correct answers. Use source turn context and publisher labels for identities; if identities are unknown, refer to first/second speaker.

Transcription output is a hypothesis. Establish the wording and meaning from the recording and its context before using it in teaching or answers.

## Render the study passages

For each selected conversational excerpt, give Jutsu a source, excerpt bounds, format JSON and an `.mp3` output path. The app finds practice units, generates the play sequence, joins source spans and renders the finished study player directly as MP3. Use that complete output for the lesson and device access; do not routinely render WAV and convert it later. Render multiple excerpts as separate study files when useful; never concatenate them or manually choose practice-unit cuts. The same workflow supplies the opening passage and later practice. If the user explicitly requests no processing, use their requested unprocessed audio instead.

Resolve any explicitly named format in `$formats`. Otherwise, translate the learner's level and preferences into suitable chunk sizes, repetitions, speeds and chaining; clarify unclear learning preferences in natural language before choosing their settings. Read the actual presets and select or adapt one to serve the lesson; a tested example or a file named `shorter` is not a default. Save the chosen JSON in the lesson directory so tomorrow uses the same processing settings. Always pass the format explicitly because omitted lookup is relative to the executable.

`chunking.chunk_target_size_seconds` guides the app's automatic segmentation, while `--start`/`--end` select the conversational excerpt. The playback section configures the app's repetitions, speeds, joins and gaps. `joining.group_size: 1` produces no multi-unit joins; use a suitable larger group size when configuring joined practice. Do not defeat segmentation with an oversized target and manually supplied phrase windows, or assemble a file's internal play order yourself. Run the sequence command first and use its units, plays and estimated duration to choose a suitable format before running the render command. Both commands use the same source, bounds and chosen format, with distinct output names.

```sh
"$jutsu" "$source" \
  --language "$language" --start "$start" --end "$end" \
  --format "$format" --sequence-only --output "$lesson/sequence.json"

"$jutsu" "$source" \
  --language "$language" --start "$start" --end "$end" \
  --format "$format" --output "$lesson/scratch-practice.mp3" \
  --metadata-output "$lesson/practice.json"
```

Inspect `plays` and `estimated_seconds` in the sequence-only report, then read `output.pcm_duration_seconds` from render metadata for the full listening activity. Render metadata stores its estimate separately as `sequence.estimated_duration_seconds`; it is not an actual playback checkpoint. Use a new scratch filename when revising; Jutsu deliberately does not overwrite outputs.

Build and review the teaching around these finished app outputs. Read the app's sequence report to understand its cuts and joins, and measure total planned listening as well as each player's length. Allow time for responses and explanations. If the practice needs adjustment, revise the format JSON or conversational excerpt and rerun Jutsu. Rename each reviewed final MP3 to `<nn>-<topic>.mp3` as specified in SKILL.md, without re-encoding; keep scratch audio outside that naming structure. Update lesson links and record the exact final paths and matching reports in `task.md`. Preserve explicit user choices and clarify unresolved learning preferences or conflicts before making the affected choices.

## Adapt the processed practice

For focused practice, select the relevant exchange on the original recording, retaining the prompt, response and referents. Configure its practice in the format JSON, rerun the same sequence/render commands under distinct output names, and review the resulting audio before presenting it. Revise processing or replace unsuitable passages you selected, then rebuild the affected lesson material. Jutsu performs all segmentation, sequencing and joining.
