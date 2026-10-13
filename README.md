# Language Jutsu

A Rust CLI project for preparing foreign-language recordings for repeated listening practice. Language and dialect labels come from the requested recording; audio processing does not restrict them to a fixed list.

The CLI decodes audio, selects a source window, finds pause-based units, and renders the learning formula with pitch-preserving tempo changes and practice gaps. `--decode-only`, `--segment-only`, and `--sequence-only` expose the earlier stages for testing. `--transcribe` generates source-language text and timestamps through an MLX Whisper helper.

## Install the guided listening skill

After cloning this repository, run from its directory:

```sh
./scripts/install-skill.sh
```

This links the bundled [language-jutsu-listening](skills/language-jutsu-listening/SKILL.md) skill into `~/.agents/skills` for use across Codex projects. Keep the checkout in place; `git pull` updates the skill. Rerunning the installer is safe; it refuses to replace a different installation. For a custom skills directory, pass its path as the only argument.

In Codex, invoke `$language-jutsu-listening` and describe the conversation you want to study, including any language, dialect, topic, level, and playback-format preferences. If the skill does not appear, restart Codex. Codex also discovers it directly when working in this checkout through `.agents/skills`; see [Codex skill discovery](https://learn.chatgpt.com/docs/build-skills).

The skill prepares conversational audio, grounds questions in source evidence, elicits responses, and provides contextual replays. It includes separate preparation and start reminder setup; installation does not activate reminders.

Install audio dependencies separately: Rust/Cargo and [FFmpeg/ffprobe](#generate-study-audio), plus the [YouTube dependencies](#youtube-input) for URLs and [MLX Whisper](#transcribe-source-audio) for transcription.

## Generate study audio

Install FFmpeg and ffprobe on PATH, then run:

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json \
  --output study.mp3
```

The formula supplies the speeds and their order. FFmpeg's `atempo` changes duration while preserving pitch; factors outside 0.5–2 use a chain of filters within that range. At 1.0×, the source PCM is copied unchanged. Joined spans are stretched as continuous source audio, retaining their original pauses. The formula's `gap_ms` adds silence between plays, including group transitions, with no final practice gap. Gaps round upward to whole sample frames and are independent of speed.

Output may be `.mp3` (using FFmpeg's `libmp3lame` encoder) or `.wav` (lossless float PCM for review). Sample rate and channel layout are retained. MP3 requires mono/stereo and a supported sample rate; use WAV when the source cannot be preserved in MP3. Rendering processes one play at a time, assembles PCM in temporary files, and encodes once. Existing outputs are never replaced; a failed backend does not publish an incomplete track.

The summary reports the actual rendered PCM frame count and duration. Tempo processing can vary slightly from the sequence estimate; MP3 container duration may additionally include codec padding. Pitch and sequence tests passed. The user accepted the supplied Shami sample's speech quality on 8 October 2026; Masri, Japanese, Spanish, and unattended listening checks remain pending.

## YouTube input

Supply `--url` in place of the positional audio file:

```sh
cargo run -- --url 'https://www.youtube.com/watch?v=mK-xzqnJmSQ&list=PLk-eaWp6xpi7I13VRRnI7cEw7bz3e3I6s&index=6' \
  --language ar --dialect shami --start 03:00 --end 03:10 \
  --format formats/default.json --output youtube-study.mp3
```

Use exactly one file or URL. Supported links are YouTube watch URLs, `youtu.be`, and Shorts, for completed videos or processed live archives accessible without login. Video links with playlist parameters select only that video; playlist-only, channel, and search links fail. The downloaded audio-only container is decoded directly, without intermediate lossy conversion.

Install `yt-dlp`, its matching EJS components, and a supported Node or Deno runtime, alongside FFmpeg/ffprobe. For example, Homebrew provides `brew install yt-dlp`; its dependency set can include Deno. Other installation methods must provide EJS as described in the [downloader setup guide](https://github.com/yt-dlp/yt-dlp/wiki/EJS). Our Rust app launches yt-dlp, which uses the JavaScript runtime for YouTube's extraction challenges. Application code remains Rust. The CLI enables Node; yt-dlp also discovers its default Deno runtime.

`--start`/`--end` select exact decoded source frames after the full audio download. Timestamp and playlist parameters in the supplied URL are retained as provenance; acquisition uses only the video identity. `--check` validates the URL and complete format offline, without yt-dlp or FFmpeg, and does not verify video availability or duration. All processing modes support URL input.

By default, URL imports use a temporary directory retained through processing and cleaned afterward, including after failures. Decode sidecars and segmentation/sequence reports include the submitted URL, resolved webpage URL, video ID, and full extraction metadata under `origin`. Without `--save-source`, `source_path` will no longer exist after the run and each URL run downloads again. Local-file origins retain their canonical path.

Add `--save-source source.webm` to a URL processing command to keep the full downloaded audio, before window selection, plus `source.webm.source.json`. The extension does not convert the downloaded container. Reuse `source.webm` as the positional input in later commands to avoid downloading again; keep its sidecar beside it to preserve the original YouTube provenance. The sidecar binds provenance to the downloaded bytes using SHA-256 and size, and reuse fails if they no longer match. Neither file is overwritten. A successfully saved source remains available if a later processing step fails.

Downloader settings are isolated from personal yt-dlp configuration, filesystem caching is disabled, and backend diagnostics are surfaced. Missing tools, unavailable videos, live broadcasts, and failed/partial downloads produce errors without a completed study output. Existing output protection remains in place.

## Transcribe source audio

MLX Whisper requires an Apple Silicon Mac, FFmpeg, and a Python environment containing `mlx-whisper` (tested with 0.4.3). Create an environment at a durable path and install the helper dependency:

```sh
python3 -m venv .venv-whisper
.venv-whisper/bin/python -m pip install 'mlx-whisper==0.4.3'
cargo run -- recording.mp3 --language ja --start 0.3 --end 20.3 \
  --transcribe --whisper-python .venv-whisper/bin/python \
  --whisper-model mlx-community/whisper-large-v3-turbo \
  --output transcript.txt
```

The Python helper is embedded in the Rust executable. `--whisper-python` defaults to `python3`; `--whisper-model` defaults to the model repository shown above and also accepts a local model directory. A repository model is downloaded by MLX Whisper when needed. No playback format is loaded or accepted for transcription. File and YouTube inputs, source retention, and original-timeline windows work in this mode.

The outputs are `transcript.txt` and `transcript.txt.json`. JSON includes source provenance, requested language/dialect, model and backend version, decoding settings, text, segments, words, and ASR diagnostics. Segment and word times are estimated **original-source seconds**, shifted by the selected start frame divided by the source sample rate. The helper always uses `task="transcribe"`, never English translation. Use a language code supported by Whisper; an unsupported code fails explicitly. A dialect label is metadata, not a promise of model dialect accuracy. Transcripts do not establish speaker identities or correct answers for tutoring.

## Save render metadata

```sh
cargo run -- recording.mp3 --language ja --start 0.3 --end 20.3 \
  --format formats/default.json --output study.wav \
  --metadata-output study.json
```

The JSON contains `source`, the effective `format`, sequence counts and `estimated_duration_seconds`, and the output's absolute path, sample rate, channels, `pcm_frames`, and actual `pcm_duration_seconds`. The actual duration is the rendered PCM length; MP3 container padding can differ. Source metadata includes the full decoded duration, selected duration and source start/end seconds, alongside the existing frame ranges. `--metadata-output` applies only to rendering; transcription, decode, segmentation and sequence modes already produce their own metadata. If requested render metadata cannot be published, the newly rendered audio is removed so the command does not leave an incomplete result.

[Boundary evidence](design/transcription-and-metadata.md) traces the helper and saved metadata shapes to their construction, serialization, and parsing code.

## Check configuration

```sh
cargo run -- --help
cargo run -- recording.wav --language ar --dialect shami \
  --format formats/default.json \
  --output study.mp3 --check
```

`--check` verifies a regular local file or a supported YouTube URL and validates the complete format. It reports the supplied language and the chunk target and playback settings without inspecting audio, making network requests, or creating output.

## Select part of a recording

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json \
  --sequence-only --output selected-sequence.json
```

`--start` is inclusive and `--end` is exclusive, on the original recording's timeline. Each accepts seconds (`95.5`), `MM:SS`, or `HH:MM:SS`, with up to nine fractional-second digits. Omitted bounds mean the beginning or end of the recording. Selection applies before pause detection and formula expansion; source frame positions remain absolute, while planned playback begins at zero.

The decoder reads the recording once, then selects samples whose source times lie within the requested interval. Bounds between sample positions round upward to a frame boundary. Negative, reversed, empty, or out-of-range windows fail without publishing output. `--check` validates syntax and explicit bound ordering; checking bounds against the actual recording requires decoding. Window selection also works with `--decode-only` and `--segment-only`.

## Find practice units

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --format formats/default.json \
  --segment-only --output units.json
```

The report retains source metadata, effective settings, all detected quiet intervals, and contiguous unit frame ranges, with derived timestamps for review. Original pauses and every frame in the selected window are preserved.

`chunking.chunk_target_size_seconds` in the format JSON defaults to **6**. Override it for a run with `--chunk-target-size-seconds 2` or `--chunk-target-size-seconds 10`. Fractional seconds are accepted. The target refers to original source duration before tempo changes; the formula and practice gaps remain separate.

Pause detection is automatic. The detector estimates local quiet and active levels from the audio, keeps brief gaps as candidates, and ranks cuts by proximity to the target and pause confidence. Confidence considers relative quietness and duration; there is no user-selected minimum pause duration or absolute volume threshold. A quieter speaker's region is analyzed at its own level rather than treated as one long gap.

The window end is a final candidate, and short-fragment handling scales with the target. Without a useful pause, a chunk can exceed the target; audio is retained rather than cut blindly. An entirely zero signal fails explicitly. Pause classification is an acoustic heuristic, so music, overlapping voices, and highly variable noise can still make boundaries uncertain.

The CLI override changes effective settings without rewriting the file. It applies to segmentation, sequencing, and rendering and conflicts with `--decode-only`. `--check` validates the complete format without media tools. The retired `quiet_threshold_dbfs`, `analysis_window_ms`, `minimum_pause_ms`, and `minimum_unit_ms` fields are rejected in custom files; all repository presets have been migrated. See the [adaptive chunking plan](design/adaptive-chunking-plan.md) for the implemented algorithm and validation.

## Expand the learning formula

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --format formats/default.json \
  --sequence-only --output sequence.json
```

The report includes the full segmentation and effective format, plus ordered plays with group index, unit membership, original source frame range, speed and its array position, repeat position, estimated output timing, and following gap. Indices are zero-based, and unit membership is a half-open range. Joining reads the continuous original source interval; practice gaps remain separate.

`timing-then-joining` visits all spans at one speed before the next speed. `joining-then-timing` visits all speeds for one span before the next span. Both honor duplicates, consecutive repetitions, local groups, and partial final groups. No added gap follows the final play. Estimates precede rendering and can differ slightly from actual DSP/codec timing. Occurrences are generated lazily and streamed to JSON; count and duration overflow fail before publication.

## Test the decoder

Install FFmpeg and ffprobe on PATH, then run:

```sh
cargo run -- recording.wav --language ar --dialect shami \
  --format formats/default.json --output decoded.f32le --decode-only
```

This produces raw, interleaved 32-bit little-endian float PCM and `decoded.f32le.json`. The JSON records the original source, selected audio stream and its probe metadata, language/dialect, sample rate, channel count, encoding, requested window, and resolved `selected_range`. `frame_count` describes the full decoded source; `pcm_frame_count` counts the frames actually exported from the selected window. Existing output files are not replaced. Source sample rate and channel count are preserved; the original media is retained.

To listen to the PCM, use the sample rate and channel count from the JSON to convert it to WAV, for example for a 48 kHz stereo recording:

```sh
ffmpeg -f f32le -ar 48000 -ac 2 -i decoded.f32le -c:a pcm_f32le decoded.wav
```

## One format file

Processing modes other than `--transcribe` load one complete JSON file with `chunking` and `playback` sections. Use `--format PATH`, or place [formats/default.json](formats/default.json) beside the executable. Development commands should supply the repository path explicitly. The file is read once per run; missing files, sections, or required fields are errors. Values never fall back to compiled defaults.

```json
{
  "chunking": {
    "chunk_target_size_seconds": 6
  },
  "playback": {
    "sequencing": "timing-then-joining",
    "segments": [0.75, 0.9, 1.0, 1.1],
    "joining": {
      "group_size": 3
    },
    "repetitions": 1,
    "gap_ms": 500
  }
}
```

`chunking` contains the target size; pause detection adapts automatically. `playback` controls speed order, sequencing, joining, repetitions, and practice gaps. Custom files replace the complete format. Ordered and duplicate speeds are preserved. `--decode-only` validates the same complete format and then exports PCM without segmentation or playback rendering.

The old `--formula` and `--segmentation` options have been replaced by `--format`. All repository presets have been migrated to `formats/`, including `joining-then-timing.json` and your `shorter.json` recipe. Each preset is a complete file.

## Use a standalone binary

Keep the single editable format JSON beside the binary in this layout:

```text
language-jutsu
formats/default.json
```

Then the binary can run from any working directory without configuration flags:

```sh
/path/to/language-jutsu recording.mp3 --language ja --output study.mp3
```

Input, output, and an explicitly supplied format path resolve relative to the caller's working directory. The default format path resolves relative to the executable. Automatic packaging is not implemented.

## Validation status

The integration suite invokes the real CLI and FFmpeg. It checks exact decoded samples, window selection, pause boundaries, both sequence orders, joined pauses, pitch/duration, stereo channels, repeats, gaps, and output protection. Additional checks exercise standalone defaults from another directory, live edits to the default recipe, `--check` without media tools, an unwritable directory, and an unavailable encoder. Synthetic language-label checks do not establish spoken-audio quality in those languages.

The supplied full Shami recording also rendered successfully using the default formula: 101 units, 34 groups, and 672 plays. [Full study MP3](palestinian-study.mp3) contains 94 minutes 44.111 seconds of decoded audio at 48 kHz stereo. A complete independent decode matched the renderer's 272,837,320 PCM frames; the original recording and earlier decode artifacts retained their hashes. [Run summary](palestinian-study.log). All 38 tests, formatting, and Clippy passed on 8 October 2026. Listening checks in the remaining languages and an unattended listen remain open.

## Design and research

- [YouTube URL input design and validation](design/youtube-input-plan.md)
- [First-pass CLI design](design/README.md)
- [Formula semantics and worked sequences](design/formula.md)
- [CLI and processing model](design/cli-and-processing.md)
- [Audio processing implementation plan](design/audio-processing-plan.md)
- [Listening research](research/README.md)

## Development checks

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
