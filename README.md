# Language Jutsu

Language Jutsu turns foreign-language recordings into study audio for repeated listening practice. It decodes a local file or a YouTube video, selects a source window, finds pause-based practice units, and renders a learning formula with pitch-preserving speed changes, progressive joining and practice gaps. It also transcribes source audio with timestamps (whisper.cpp, built into the executable) and packages lesson flashcards for Anki.

The bundled [language-jutsu-listening](skills/language-jutsu-listening/SKILL.md) skill lets Claude Code or Codex prepare and teach complete interactive lessons around that study audio.

## Setup

Supported systems: macOS with [Homebrew](https://brew.sh), Debian/Ubuntu on x86_64, and Windows on x64. On any other system the setup script stops with a message naming these before installing anything; use [Manual setup](#manual-setup) instead.

Clone the repository and run the setup script from its directory. Keep the checkout in place afterwards: the CLI runs from it and the installed skill links to it. Rerunning setup is safe; it installs only what is missing.

### macOS

```sh
./setup.sh
```

Installs whichever of `rust`, `cmake`, `ffmpeg` and `deno` are missing with `brew install`. If `yt-dlp` is missing, downloads the official standalone `yt-dlp_macos` executable to `/usr/local/bin/yt-dlp` (asks for your password through `sudo`). libclang, which the build uses to generate the whisper.cpp bindings, comes with the Xcode Command Line Tools that Homebrew requires.

### Debian and Ubuntu (x86_64)

```sh
./setup.sh
```

Runs `sudo apt-get install -y --no-install-recommends ca-certificates curl unzip build-essential cmake libclang-dev ffmpeg`. Then installs Rust with [rustup](https://rustup.rs) if `cargo` is missing, Deno with its [official installer](https://docs.deno.com/runtime/getting_started/installation/) if `deno` is missing, and the official standalone `yt-dlp_linux` executable to `/usr/local/bin/yt-dlp` if `yt-dlp` is missing. rustup and the Deno installer add `~/.cargo/bin` and `~/.deno/bin` to your shell configuration; open a new terminal afterwards. Transcription runs on the CPU.

### Windows (x64)

In PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File setup.ps1
```

Installs with winget: `yt-dlp.yt-dlp` if `yt-dlp` is missing (its package also installs Deno and FFmpeg with ffprobe), `Kitware.CMake` if `cmake` is missing, and `LLVM.LLVM` (libclang) if `C:\Program Files\LLVM\bin\libclang.dll` is missing. Runs [rustup-init](https://rustup.rs) if `cargo` is missing; rustup offers to install the Visual Studio C++ build tools. The skill is linked with directory junctions.

### What every setup does

1. Builds the CLI with `cargo build`.
2. Runs `cargo run -- download-model`, which downloads the default Whisper model (`ggml-large-v3-turbo.bin`, about 1.5 GiB) into the standard Hugging Face cache: `~/.cache/huggingface/hub` (on Windows `%USERPROFILE%\.cache\huggingface\hub`), or `$HF_HUB_CACHE`, or `$HF_HOME/hub`. Nothing is stored in the repository.
3. Links the listening skill into `~/.agents/skills` (Codex) and `~/.claude/skills` (Claude Code). An existing different installation is never replaced.

No setup step installs Python.

### Manual setup

Install Rust/Cargo, CMake with a C/C++ compiler, libclang, FFmpeg with ffprobe, and for YouTube input the official standalone [yt-dlp](https://github.com/yt-dlp/yt-dlp#release-files) executable with a Deno (2.3.0 or later) or Node (22 or later) runtime. Then:

```sh
cargo build
cargo run -- download-model
./scripts/install-skill.sh ~/.agents/skills
./scripts/install-skill.sh ~/.claude/skills
```

`install-skill.sh` without an argument installs into `~/.agents/skills`.

## Command-line usage

Run commands from the repository with `cargo run --`. Every processing command takes one source (a file path or `--url`), `--language` and `--output`. `cargo run -- --help` lists all options. Existing output files are never replaced.

### Study audio from a local file

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json \
  --output study.mp3
```

### Study audio from a YouTube URL

```sh
cargo run -- --url 'https://www.youtube.com/watch?v=mK-xzqnJmSQ&list=PLk-eaWp6xpi7I13VRRnI7cEw7bz3e3I6s&index=6' \
  --language ar --dialect shami --start 03:00 --end 03:10 \
  --format formats/default.json --output youtube-study.mp3
```

### Choose a format

The repository ships three complete formats:

| Format | Chunk target | Order | Speeds | Repetitions | Gap |
| --- | --- | --- | --- | --- | --- |
| [formats/default.json](formats/default.json) | 6 s | `timing-then-joining` | 0.75, 0.9, 1.0, 1.1 | 1 | 500 ms |
| [formats/joining-then-timing.json](formats/joining-then-timing.json) | 6 s | `joining-then-timing` | 0.75, 0.9, 1.0, 1.1 | 1 | 500 ms |
| [formats/shorter.json](formats/shorter.json) | 3 s | `joining-then-timing` | 0.75, 0.85, 1.0, 1.1 | 3 | 1000 ms |

All three join up to 3 units. The same passage with every speed for one span before the next span:

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/joining-then-timing.json --output study-joining.mp3
```

Shorter chunks with more repetition, from YouTube:

```sh
cargo run -- --url 'https://youtu.be/mK-xzqnJmSQ' \
  --language ar --dialect shami --start 03:00 --end 03:10 \
  --format formats/shorter.json --output youtube-shorter.mp3
```

Override a format's chunk target for one run, in seconds of original source audio:

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json --chunk-target-size-seconds 2 \
  --output study-2s.mp3
```

### Generate study audio

The formula supplies the speeds and their order. FFmpeg's `atempo` changes duration while preserving pitch; factors outside 0.5–2 use a chain of filters within that range. At 1.0×, the source PCM is copied unchanged. Joined spans are stretched as continuous source audio, retaining their original pauses. The formula's `gap_ms` adds silence between plays, including group transitions, with no final practice gap. Gaps round upward to whole sample frames and are independent of speed.

Output may be `.mp3` (using FFmpeg's `libmp3lame` encoder) or `.wav` (lossless float PCM for review). Sample rate and channel layout are retained. MP3 requires mono/stereo and a supported sample rate; use WAV when the source cannot be preserved in MP3. Rendering processes one play at a time, assembles PCM in temporary files, and encodes once. Existing outputs are never replaced; a failed backend does not publish an incomplete track.

The summary reports the actual rendered PCM frame count and duration. Tempo processing can vary slightly from the sequence estimate; MP3 container duration may additionally include codec padding. Pitch and sequence tests passed. The user accepted the supplied Shami sample's speech quality on 8 October 2026; Masri, Japanese, Spanish, and unattended listening checks remain pending.

### Select part of a recording

`--start` is inclusive and `--end` is exclusive, on the original recording's timeline. Each accepts seconds (`95.5`), `MM:SS`, or `HH:MM:SS`, with up to nine fractional-second digits. Omitted bounds mean the beginning or end of the recording. Selection applies before pause detection and formula expansion; source frame positions remain absolute, while planned playback begins at zero.

The decoder reads the recording once, then selects samples whose source times lie within the requested interval. Bounds between sample positions round upward to a frame boundary. Negative, reversed, empty, or out-of-range windows fail without publishing output. `--check` validates syntax and explicit bound ordering; checking bounds against the actual recording requires decoding. Window selection works in every processing mode.

### YouTube input

Use exactly one file or URL. Supported links are YouTube watch URLs, `youtu.be`, and Shorts, for completed videos or processed live archives accessible without login. Video links with playlist parameters select only that video; playlist-only, channel, and search links fail. The downloaded audio-only container is decoded directly, without intermediate lossy conversion.

The Rust app launches yt-dlp, which uses the JavaScript runtime (Deno, or Node, which the CLI enables) for YouTube's extraction challenges. `--start`/`--end` select exact decoded source frames after the full audio download. Timestamp and playlist parameters in the supplied URL are retained as provenance; acquisition uses only the video identity. `--check` validates the URL and complete format offline, without yt-dlp or FFmpeg, and does not verify video availability or duration. All processing modes support URL input.

By default, URL imports use a temporary directory retained through processing and cleaned afterward, including after failures. Decode sidecars and segmentation/sequence reports include the submitted URL, resolved webpage URL, video ID, and full extraction metadata under `origin`. Without `--save-source`, `source_path` will no longer exist after the run and each URL run downloads again. Local-file origins retain their canonical path.

Add `--save-source source.webm` to a URL processing command to keep the full downloaded audio, before window selection, plus `source.webm.source.json`:

```sh
cargo run -- --url 'https://youtu.be/mK-xzqnJmSQ' \
  --language ar --dialect shami --start 03:00 --end 03:10 \
  --format formats/default.json --output youtube-study-saved.mp3 \
  --save-source source.webm
```

The extension does not convert the downloaded container. Reuse `source.webm` as the positional input in later commands to avoid downloading again; keep its sidecar beside it to preserve the original YouTube provenance. The sidecar binds provenance to the downloaded bytes using SHA-256 and size, and reuse fails if they no longer match. Neither file is overwritten. A successfully saved source remains available if a later processing step fails.

Downloader settings are isolated from personal yt-dlp configuration, filesystem caching is disabled, and backend diagnostics are surfaced. Missing tools, unavailable videos, live broadcasts, and failed/partial downloads produce errors without a completed study output.

### Check configuration

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --format formats/default.json \
  --output study.mp3 --check
```

`--check` verifies a regular local file or a supported YouTube URL and validates the complete format. It reports the supplied language and the chunk target and playback settings without inspecting audio, making network requests, or creating output.

### Expand the learning formula

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json \
  --sequence-only --output sequence.json
```

The report includes the full segmentation and effective format, plus ordered plays with group index, unit membership, original source frame range, speed and its array position, repeat position, estimated output timing, and following gap. Indices are zero-based, and unit membership is a half-open range. Joining reads the continuous original source interval; practice gaps remain separate.

`timing-then-joining` visits all spans at one speed before the next speed. `joining-then-timing` visits all speeds for one span before the next span. Both honor duplicates, consecutive repetitions, local playback groups, and partial final groups. Joins slide across group boundaries, capped at `group_size`: A, B, C becomes A, B, AB, C, BC with limit 2, or A, B, AB, C, ABC with limit 3 or more. No added gap follows the final play. Estimates precede rendering and can differ slightly from actual DSP/codec timing. The unit count and playback format expand once into an owned sequence containing every play and following gap. Rendering, reports, and duration estimates consume those stored entries; count and duration overflow fail before publication.

### Find practice units

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json \
  --segment-only --output units.json
```

The report retains source metadata, effective settings, all detected quiet intervals, and contiguous unit frame ranges, with derived timestamps for review. Original pauses and every frame in the selected window are preserved.

Pause detection adapts to local speech/background levels and nearby gap durations, independently of the requested chunk size. Brief energy dips are excluded, and extra pause duration has diminishing benefit. Cuts use the nearest safe point inside a pause, balancing pause quality against distance from the target. There are no detector settings to tune; a quieter speaker's region is analyzed at its own level rather than treated as one long gap.

The window end is a final candidate, and short-fragment handling scales with the target. Without a useful pause, a chunk can exceed the target; audio is retained rather than cut blindly. An entirely zero signal fails explicitly. Pause classification is an acoustic heuristic, so music, overlapping voices, and highly variable noise can still make boundaries uncertain.

The `--chunk-target-size-seconds` override changes effective settings without rewriting the format file. Fractional seconds are accepted. The target refers to original source duration before tempo changes. It applies to segmentation, sequencing, and rendering and conflicts with `--decode-only`.

### Save render metadata

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json --output study.wav \
  --metadata-output study.json
```

The JSON contains `source`, the effective `format`, sequence counts and `estimated_duration_seconds`, and the output's absolute path, sample rate, channels, `pcm_frames`, and actual `pcm_duration_seconds`. The actual duration is the rendered PCM length; MP3 container padding can differ. Source metadata includes the full decoded duration, selected duration and source start/end seconds, alongside the existing frame ranges. `--metadata-output` applies only to rendering; transcription, decode, segmentation and sequence modes already produce their own metadata. If requested render metadata cannot be published, the newly rendered audio is removed so the command does not leave an incomplete result.

### Transcribe source audio

Transcription runs [whisper.cpp](https://github.com/ggml-org/whisper.cpp) inside the Rust executable (Metal-accelerated on macOS). Models are files from the [ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) Hugging Face repository, kept in the Hugging Face cache:

```sh
cargo run -- download-model
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --transcribe --output transcript.txt
cargo run -- --url 'https://youtu.be/mK-xzqnJmSQ' \
  --language ar --start 03:00 --end 03:10 \
  --transcribe --output youtube-transcript.txt
```

`download-model` fetches `ggml-large-v3-turbo.bin` unless another model file is named, for example `cargo run -- download-model ggml-small.bin`. `--whisper-model` selects the model file for transcription and has the same default; a model not yet cached is downloaded on first use. Models are pinned to one commit of the repository, so a cached model is used without any network access. Downloads go through [hf-hub](https://github.com/huggingface/hf-hub): if two runs start downloading the same model at the same time, one downloads and the other stops with a lock error ([by design](https://github.com/huggingface/hf-hub/issues/115)); run it again once the download has finished. On Windows, hf-hub stores a model twice in the cache (about 3 GiB for the default model), because it copies files where other systems use links. No playback format is loaded or accepted for transcription. File and YouTube inputs, source retention, and original-timeline windows work in this mode.

The outputs are `transcript.txt` and `transcript.txt.json`. JSON includes source provenance, requested language/dialect, model and whisper.cpp version, decoding settings, text, segments with `no_speech_prob`, and words with probabilities. Segment and word times are estimated **original-source seconds**, shifted by the selected start frame divided by the source sample rate. Transcription always transcribes, never translates into English. Use a language code supported by Whisper; an unsupported code, or a language the chosen model lacks, fails explicitly. A dialect label is metadata, not a promise of model dialect accuracy. Transcripts do not establish speaker identities or correct answers for tutoring.

[Boundary evidence](design/transcription-and-metadata.md) traces the transcription pipeline and saved metadata shapes to their construction, serialization, and parsing code.

### Test the decoder

```sh
cargo run -- recording.mp3 --language ar --dialect shami \
  --start 01:35 --end 01:45 \
  --format formats/default.json --output decoded.f32le --decode-only
```

This produces raw, interleaved 32-bit little-endian float PCM and `decoded.f32le.json`. The JSON records the original source, selected audio stream and its probe metadata, language/dialect, sample rate, channel count, encoding, requested window, and resolved `selected_range`. `frame_count` describes the full decoded source; `pcm_frame_count` counts the frames actually exported from the selected window. Source sample rate and channel count are preserved; the original media is retained.

To listen to the PCM, use the sample rate and channel count from the JSON to convert it to WAV, for example for a 48 kHz stereo recording:

```sh
ffmpeg -f f32le -ar 48000 -ac 2 -i decoded.f32le -c:a pcm_f32le decoded.wav
```

### Build an Anki package

```sh
cargo run -- anki cards.json --output lesson.apkg
```

Packages authored Vocab, Grammar and Listening cards with embedded audio into a new `.apkg`, without accessing a live Anki collection. Each card submits its source chunk as one `context` object containing `audio` (the recording path) and `sentence` (its full corresponding text). The JSON manifest also supplies deck identity, language and card content; note identities remain stable across revisions. See the [input format and card requirements](skills/language-jutsu-listening/references/anki.md#package-for-anki).

### Use a standalone binary

Keep the single editable format JSON beside the binary in this layout:

```text
language-jutsu
formats/default.json
```

Then the binary can run from any working directory without `--format`:

```sh
/path/to/language-jutsu recording.mp3 --language ja --output study.mp3
```

Input, output, and an explicitly supplied format path resolve relative to the caller's working directory. The default format path resolves relative to the executable. Automatic packaging is not implemented.

## Format files

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

| Field | Meaning |
| --- | --- |
| `chunking.chunk_target_size_seconds` | Target practice-unit length in original source seconds; pause detection adapts automatically. |
| `playback.sequencing` | `timing-then-joining` or `joining-then-timing` (see [Expand the learning formula](#expand-the-learning-formula)). |
| `playback.segments` | Playback-speed multipliers in play order; duplicates and order are preserved. |
| `playback.joining.group_size` | Units per local playback group, and the most consecutive units in one join. |
| `playback.repetitions` | Times each speed/span occurrence plays before advancing. |
| `playback.gap_ms` | Silence between plays. |

Custom files replace the complete format. The retired `quiet_threshold_dbfs`, `analysis_window_ms`, `minimum_pause_ms`, and `minimum_unit_ms` fields are rejected. See the [adaptive chunking plan](design/adaptive-chunking-plan.md) for the implemented algorithm and validation.

## Guided listening skill

Setup links the skill into `~/.agents/skills` and `~/.claude/skills`. Keep the checkout in place; `git pull` updates the skill. Codex also discovers it directly when working in this checkout through `.agents/skills`; see [Codex skill discovery](https://learn.chatgpt.com/docs/build-skills).

Invoke it with `/language-jutsu-listening` in Claude Code or `$language-jutsu-listening` in Codex (restart the app if it does not appear), and describe the conversation you want to study. The skill finds a matching scene, transcribes it, renders processed study audio with the CLI, and teaches through it: chunked listening, vocabulary and grammar notes tied to the speech, your responses, and a return to the connected conversation. The rewritten lesson design is undergoing a real learner session; human usability is not yet accepted.

### What you can specify

The skill settles these from your request and asks short follow-ups only where a missing answer would change the lesson:

| Option | What to say |
| --- | --- |
| Level | A CEFR, JLPT or other level, or a description such as "beginner", "basic conversation", "I can follow the main idea". |
| Session time | The whole session, including listening, explanations and answering, e.g. "30 minutes". |
| Chunk size and pace | Seconds, or a focus on words, short phrases, sentences or exchanges; more repetition and slower practice, or more continuous conversation. |
| Supplemental materials | Vocabulary and grammar notes are included unless you decline them. Ask for transcripts, readings, pronunciation guidance, or a particular instruction language. |
| Listening goal | What is hardest: recognizing known words in speech, unfamiliar expressions, or keeping up with the conversation. |
| Content | Language and dialect, topic, source type, a specific video or file and bounds, and what you already know about it (e.g. a show's characters). |
| Interaction | Whether you will read and type during the lesson or respond aloud. |
| Format | A preferred playback format; explicitly chosen formats are kept. |

### Anki flashcards

Ask for flashcards and say whether to **create a new deck** or **add to an existing deck**. For an existing deck, name it and provide an export or access. The cards cover every vocabulary and grammar point taught in the lesson, as Vocab, Grammar and Listening cards with the lesson's processed audio, packaged as an `.apkg` to import with Anki's **File → Import**.

### Reminders

On request, the skill sets up two separate daily reminders in the same chat: one at 17:00 to choose and prepare tomorrow's lesson, and one at 12:00 to start today's, in America/Chicago. Each time is editable independently. Installing the skill does not activate them. They use the desktop app's scheduled tasks; see [scheduling.md](skills/language-jutsu-listening/references/scheduling.md).

### Example prompts

```text
/language-jutsu-listening Shami Arabic, basic conversation level, 30 minutes.
Use https://www.youtube.com/watch?v=mK-xzqnJmSQ from 03:00 to 05:00.
I lose track of who is replying to what. Short phrases, lots of repetition.
```

```text
$language-jutsu-listening Japanese, JLPT N3. A scene from a slice-of-life anime
about ordering food. 20 minutes, I'll type my answers. Include readings for kanji,
and make a new Anki deck from the lesson.
```

```text
/language-jutsu-listening Set up my daily preparation and start reminders.
```

Lessons are saved under `listening/` in the repository (`listening/YYYY-MM-DD/task.md`), so a later session resumes from the pending turn.

## Languages

Pass a code or one of these names to `--language`. Names are case-insensitive and normalize to the listed code in metadata and transcription requests. Other nonempty language labels remain accepted for audio processing; `--dialect` remains an optional label.

| Language / accepted names | Code | Whisper transcription |
| --- | --- | --- |
| French | `fr` | Supported by multilingual models |
| Italian | `it` | Supported by multilingual models |
| German | `de` | Supported by multilingual models |
| Mandarin, Chinese | `zh` | Supported by multilingual models |
| Cantonese | `yue` | Requires a model with the Cantonese token, such as large-v3 or the default large-v3-turbo |
| Korean | `ko` | Supported by multilingual models |
| Farsi, Persian | `fa` | Supported by multilingual models |
| Indonesian | `id` | Supported by multilingual models |
| Bengali | `bn` | Supported by multilingual models |
| Hausa | `ha` | Supported by multilingual models |
| Tamil | `ta` | Supported by multilingual models |
| Hindi | `hi` | Supported by multilingual models |
| Turkish | `tr` | Supported by multilingual models |
| Vietnamese | `vi` | Supported by multilingual models |
| Kiswahili, Swahili | `sw` | Supported by multilingual models |
| isiZulu, Zulu | `zu` | Not in the Whisper language table |
| isiXhosa, Xhosa | `xh` | Not in the Whisper language table |
| Sesotho | `st` | Not in the Whisper language table |

All listed languages can use decoding, segmentation, sequencing and study-audio rendering. Transcription support follows the [whisper.cpp language table](https://github.com/ggml-org/whisper.cpp/blob/master/src/whisper.cpp) (`g_lang`); the [large-v3 model introduced the Cantonese token](https://huggingface.co/openai/whisper-large-v3).

## Validation status

The integration suite invokes the real CLI and FFmpeg. It checks exact decoded samples, window selection, pause boundaries, both sequence orders, joined pauses, pitch/duration, stereo channels, repeats, gaps, and output protection. Additional checks exercise standalone defaults from another directory, live edits to the default recipe, `--check` without media tools, an unwritable directory, and an unavailable encoder. Transcription tests run whisper.cpp with `ggml-tiny.bin` on real public-domain English and Japanese speech ([sources](tests/fixtures/SOURCES.md)). Synthetic language-label checks do not establish spoken-audio quality in those languages.

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
