# Acquisition, transcription, and practice audio

Use the current Language Jutsu checkout and its `--help`. The commands below use absolute paths resolved into shell variables: `repo`, `lesson`, `source`, `language`, `format`, `start`, `end`, `whisper_python`, `whisper_model`, and `skill`. They are invocation patterns, not values to copy from a scenario. Add `--dialect` only when the request supplies or establishes one. When no interval is selected, omit `--start` and `--end`.

## Get the requested conversation

For discovery, search in the requested language/dialect and topic, open the full publisher/media pages, and inspect the actual candidate recording's transcript/audio evidence. Establish responsive speaker turns, target language without English, topic, dialect, and suitable difficulty. A title or automated language label cannot establish all of these. Reject mismatches and continue the same search, preserving the request.

For supplied content, preserve its recording identity and explicit bounds. If a matching conversation can be selected and the request permits selection, retain whole question/answer exchanges. When the recording conflicts with a requirement that cannot be repaired by an allowed selection, explain the concrete conflict instead of silently changing the source.

Jutsu's `--url` is for a single supported YouTube video URL. Acquire and transcribe once while retaining the full downloaded original:

```sh
cargo run --quiet --manifest-path "$repo/Cargo.toml" -- \
  --url "$url" --save-source "$lesson/original.audio" \
  --language "$language" --transcribe \
  --whisper-python "$whisper_python" --whisper-model "$whisper_model" \
  --output "$lesson/transcript.txt"
```

The retained original and its `.source.json` companion stay together. Use that local original for all later commands; the sidecar restores its remote provenance. Do not use a temporary download path from an earlier invocation as tomorrow's source.

For a public publisher/RSS/direct-media URL, resolve the actual audio link from the full page or feed, download that recording into the lesson directory, and record submitted page, final media URL, publisher identity, and download date in `sources.md`. For a supplied local file, use a retained copy or a stable original path. Do not pass a generic page or Spotify URL to the YouTube-only parser or use a preview as the full requested recording. Investigate the provider's actual public media route; an acquisition failure does not authorize content substitution.

## Produce source-language text

MLX Whisper runs locally on supported Apple Silicon hardware. Configure the Python executable containing `mlx-whisper` and an explicit local model or model repository. The Jutsu README gives dependency setup. Record usable durable paths in `listening/settings.md`, not investigation `/tmp` paths. Test the configured runtime by actually transcribing the lesson.

```sh
cargo run --quiet --manifest-path "$repo/Cargo.toml" -- "$source" \
  --language "$language" --start "$start" --end "$end" --transcribe \
  --whisper-python "$whisper_python" --whisper-model "$whisper_model" \
  --output "$lesson/transcript.txt"
```

The text file has a structured `.json` companion: `text`, `language`, `segments` (with `words`), `source`, `backend`, `settings`, `timestamp_reference`, and `source_offset_seconds`. Segment/word `start` and `end` are already original-source seconds. Never align against the expanded practice track or add the offset a second time. Transcription preserves the spoken language; do not request translation. Whisper does not supply speaker identities, questions, or correct answers. Use source turn context and publisher labels for identities; if identities are unknown, refer to first/second speaker. [Code proof](boundaries.md) traces these interfaces.

Read [evidence.md](evidence.md) before converting this text into answer claims.

## Plan and render the complete exercise

Resolve the actual named format in `formats/`. Copy the chosen JSON to the lesson directory so tomorrow uses the same processing settings. For unspecified processing, choose a design that serves this conversational exchange and explain that choice; a tested example or a file named `shorter` is not a default. Always pass the format explicitly because omitted lookup is relative to the executable.

```sh
cargo run --quiet --manifest-path "$repo/Cargo.toml" -- "$source" \
  --language "$language" --start "$start" --end "$end" \
  --format "$format" --sequence-only --output "$lesson/sequence.json"

cargo run --quiet --manifest-path "$repo/Cargo.toml" -- "$source" \
  --language "$language" --start "$start" --end "$end" \
  --format "$format" --output "$lesson/practice.wav" \
  --metadata-output "$lesson/practice.json"
```

Inspect the ordered plays, joins, and estimate in the sequence document, then read `output.pcm_duration_seconds` from render metadata for the full listening activity. `sequence.estimated_duration_seconds` is a separate estimate, not an actual playback checkpoint. Use a new output filename when revising; Jutsu deliberately does not overwrite outputs.

The preparation review considers the entire main track and the concrete learning interaction following it. Reject a thirty-minute track; do not present it with a skip/early-stop escape. If the user left the interval or processing design open, revise those choices and render again. Preserve explicit user choices and ask only when their conflict prevents a workable result. Do not turn a successful example's duration into a universal limit.

## Prepare a contextual repair

Take bounds from the evidence on the original recording, including the prompt, answer, and relevant preceding referent. Render through Jutsu using the bundled one-play format:

```sh
cargo run --quiet --manifest-path "$repo/Cargo.toml" -- "$source" \
  --language "$language" --start "$repair_start" --end "$repair_end" \
  --format "$skill/assets/reference.json" \
  --output "$lesson/repair.wav" --metadata-output "$lesson/repair.json"
```

This format plays the continuous selected source once at its original speed with no added gaps. It is a diagnostic replay after the complete main task, not a replacement for an unusable main task. Ask a narrower, source-grounded question immediately before this player; reconnect the learner's answer to the original conversation afterward.
