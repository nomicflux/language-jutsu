# CLI and processing model

Draft date: 7 October 2026. These are proposed interfaces and data concepts, not implemented commands or Rust code.

## Command interface

```text
language-jutsu input.wav --language ar --dialect shami --output study.mp3

language-jutsu input.wav --language ar --dialect masri \
  --formula formulas/my-formula.json --output study.mp3

language-jutsu input.wav --language ja --output study.mp3

language-jutsu input.wav --language es --output study.mp3
```

Input file, language, and output path are required. Proposed language values are `ar`, `ja`, and `es`. The optional Arabic dialect setting records `shami` or `masri`; it does not silently default to either. An Arabic file can be supplied without a dialect label. Supplying the dialect flag with another language is an input error.

The language setting is retained as run context and reported in the CLI summary. If language-specific speech analysis is used, it receives this setting. Basic pause detection is language-independent; the design does not claim different acoustic cuts just because the label changes.

The proposed output is one MP3 study track containing the sequence of learning chunks. Individual files per practice group could be an alternative export layout, but are not needed to define the sequence. The output format remains a design choice to workshop.

## Formula resolution

With `--formula`, read exactly that JSON file. Without it, read the distributed `formulas/default.json` alongside the installed executable. Explicit relative paths are resolved against the caller's working directory; the distributed default's location is relative to the executable.

The [default JSON](../formulas/default.json) is a separate editable distribution asset, not a JSON string compiled into the binary.

The CLI resolves and parses the formula before media processing. The processing core requires a complete `Formula` argument and has no optional-formula overload or default constructor. Missing, unreadable, or invalid formula data produces an error. No recipe values are supplied by a hidden code fallback.

## Domain concepts

| Concept | Information retained |
| --- | --- |
| Language setting | Arabic with an optional supplied dialect, Japanese, or Spanish |
| Audio source | Original media, duration, sample rate/timebase, channels, and language setting |
| Practice unit | A nonempty source-time interval in the original recording |
| Practice group | Ordered consecutive units participating in one local build |
| Formula | Sequencing choice, ordered positive speed factors, group size, repeat count, and output gap duration |
| Planned play | Group and source-unit membership, original source interval, speed, and occurrence identity |
| Learning sequence | Ordered plays and added inter-play gaps, with estimated output duration |

The JSON's strings and numbers are parsed into these domain concepts. Internally, distinguish speed factors, source intervals, and output gap durations. Distinguish the two sequencing choices explicitly. A model token or a rewritten transcript is not a practice unit.

Joined spans retain their participating units as well as the enclosing source interval. Output positions are derived from the sequence; they do not replace original source positions. Repeated occurrences of the same span remain distinct plays.

## Transformation flow

```text
CLI arguments + formula file → parsed request with required Formula
audio file → decoded Audio source
Audio source + segmentation settings → ordered Practice units
Practice units + Formula → Practice groups → Learning sequence
Audio source + Learning sequence → rendered study audio
```

Formula parsing, audio analysis, sequence construction, and rendering have separate responsibilities. The sequence can be constructed before audio rendering, allowing the CLI to report its estimated listening duration. These are boundaries in the design, not additional user-operated stages.

## Input segmentation

Significant pauses are sufficient boundaries. A simple starting proposal is to detect sustained low-volume intervals and place a cut within each suitable pause. Source units stay ordered, and the full spoken material receives practice. Very short fragments can be merged; long units can use smaller pauses for further cuts.

Pause sensitivity has at least two settings: the signal level treated as quiet and how long quiet must persist. FFmpeg's `silencedetect` exposes both and is one candidate alongside speech-activity detection. Low amplitude is a proxy, so music, noise, and quiet speech need listening checks. No numerical segmentation defaults have been selected. [FFmpeg silence detection](https://ffmpeg.org/ffmpeg-filters.html#silencedetect).

Desired unit length is a preference rather than a reason to cut through speech at an exact second. A long run without usable pauses remains a limitation of simple pause detection: finer speech-assisted cuts may be needed. Keep a larger intelligible unit when a finer cut is uncertain; do not silently omit it.

Language-specific transcription and alignment remain optional refinements. The first-pass CLI does not depend on proving Wispr's timing or verbatim capabilities. See the [speech technology research](../research/technical-options.md).

## Rendering

Read each play's continuous original source interval, apply its tempo change, and append the next practice gap where one is specified. For A+B, stretch the complete source interval once. Do not combine independently stretched versions of A and B with practice silence between them.

Keep pitch stable as tempo changes. Decode before composing the sequence and encode the finished result, avoiding repeated compression of small pieces. The actual time-stretch engine remains a choice to audition. FFmpeg supplies tempo and trim filters; Signalsmith Stretch is another researched candidate. [Rendering options](../research/technical-options.md#slower-and-faster-playback).

The CLI summary should report the input, supplied language/dialect, formula path, unit/group/play counts, estimated duration before rendering, and output path and actual duration afterward. Duration is a planning aid, not a reason to reduce the requested repetition.

Input errors, formula errors, inability to form usable units, and rendering failures should be distinguishable to the caller. No output should be reported as complete when rendering fails.
