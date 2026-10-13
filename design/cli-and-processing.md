# CLI and processing model

Design date: 7 October 2026; updated 9 October 2026. Configuration, decoding, source-window selection, segmentation, sequence planning, and study-track rendering are implemented.

## Command interface

```text
language-jutsu input.wav --language ar --dialect shami --output study.mp3

language-jutsu input.wav --language ar --dialect masri \
  --format formats/my-format.json --output study.mp3

language-jutsu input.wav --language ja --output study.mp3

language-jutsu input.wav --language es --output study.mp3
```

Input file, language, and output path are required. Proposed language values are `ar`, `ja`, and `es`. The optional Arabic dialect setting records `shami` or `masri`; it does not silently default to either. An Arabic file can be supplied without a dialect label. Supplying the dialect flag with another language is an input error.

The language setting is retained as run context and reported in the CLI summary. If language-specific speech analysis is used, it receives this setting. Basic pause detection is language-independent; the design does not claim different acoustic cuts just because the label changes.

Output is one MP3 or float-PCM WAV study track containing the sequence of learning chunks. WAV provides a lossless review format; MP3 serves ordinary listening players.

## Source window

Optional `--start` and `--end` accept nonnegative seconds, `MM:SS`, or `HH:MM:SS`, with up to nine fractional-second digits. They refer to the decoded original timeline: start inclusive, end exclusive. Missing start means zero; missing end means the decoded recording's end. Explicit bounds must be ordered, within the recording, and select at least one frame. No bound is silently clamped.

The source is decoded once before selection, avoiding approximate compressed-media seeks. Each bound rounds upward to a sample-frame boundary, selecting exactly those sample positions inside the requested half-open interval. The full source metadata and requested times remain available alongside the resolved frame range. Segmentation reads only that range, and every unit and joined span keeps its absolute source positions. Output timing begins at zero.

`--decode-only` exports the selected PCM. Its sidecar retains the whole source's `frame_count` and adds `pcm_frame_count` for the exported selection. `--segment-only` and `--sequence-only` retain the selection in their source metadata. `--check` validates time syntax and explicit ordering without decoding; it cannot validate the recording's actual end.

## Format resolution

`--format` selects one complete JSON file. Without it, load `formats/default.json` beside the executable. Explicit paths remain relative to the caller's working directory. The format has `chunking` and `playback` sections, both required in every mode. Parse it once before media processing and apply `--chunk-target-size-seconds` to `chunking` for this run. Missing or invalid fields fail with a format diagnostic; the file remains unchanged and there are no compiled fallback settings.

The processing core receives the validated format. Segmentation reads its chunking settings, while sequence construction and rendering read its playback formula. The distributed JSON and migrated presets contain both sections. The separate configuration-path options have been replaced by `--format`.

## Domain concepts

| Concept | Information retained |
| --- | --- |
| Language setting | Arabic with an optional supplied dialect, Japanese, or Spanish |
| Audio source | Original media, duration, sample rate/timebase, channels, language setting, requested window, and resolved source frame range |
| Practice unit | A nonempty source-time interval in the original recording |
| Practice group | Ordered consecutive units participating in one local build |
| Formula | Sequencing choice, ordered positive speed factors, group size, repeat count, and output gap duration |
| Planned play | Group and source-unit membership, original source interval, speed, and occurrence identity |
| Learning sequence | Ordered plays and added inter-play gaps, with estimated output duration |

The JSON's strings and numbers are parsed into these domain concepts. Internally, distinguish speed factors, source intervals, and output gap durations. Distinguish the two sequencing choices explicitly. A model token or a rewritten transcript is not a practice unit.

Joined spans retain their participating units as well as the enclosing source interval. Output positions are derived from the sequence; they do not replace original source positions. Repeated occurrences of the same span remain distinct plays.

## Transformation flow

```text
CLI arguments + format file → parsed request with required Format
audio file + optional window bounds → decoded Audio source with selected frame range
Audio source + segmentation settings → ordered Practice units
Practice units + Formula → Practice groups → Learning sequence
Audio source + Learning sequence → rendered study audio
```

Formula parsing, audio analysis, sequence construction, and rendering have separate responsibilities. The sequence can be constructed before audio rendering, allowing the CLI to report its estimated listening duration. These are boundaries in the design, not additional user-operated stages.

## Input segmentation

The format's `chunking` section contains only `chunk_target_size_seconds`, default 6 in the distributed file. `--chunk-target-size-seconds` overrides it for a run. It measures source duration before tempo processing.

Detection fits local low/high energy levels from per-channel PCM measurements. Short low-energy gaps remain candidates, with confidence based on their contrast against both neighboring sides and their duration. Unconfirmed low-energy regions are reanalyzed to distinguish quieter speakers from pauses. Boundary selection ranks confidence and target proximity, favoring earlier locations on ties. Fragment handling scales with the target rather than using a fixed minimum duration.

The window end is a final candidate. Source positions remain absolute, every source frame is retained, and long uninterrupted spans remain whole when no useful pause exists. The model is an acoustic heuristic, not grammatical or word alignment. The [adaptive chunking plan](./adaptive-chunking-plan.md) records the algorithm and actual tests. Manual detector fields have been removed from the format.

Language-specific transcription and alignment remain optional refinements. The first-pass CLI does not depend on proving Wispr's timing or verbatim capabilities. See the [speech technology research](../research/technical-options.md).

## Rendering

Read each play's continuous original source interval, apply its tempo change, and append the next practice gap where one is specified. For A+B, stretch the complete source interval once. Do not combine independently stretched versions of A and B with practice silence between them.

Pitch-preserving processing uses FFmpeg `atempo`, chained in 0.5–2 factors for speeds outside that interval. At 1.0×, copy original PCM unchanged. Each play reads its exact frame range into a temporary clip; processing the entire joined span keeps original pauses continuous. Append the configured output gap, rounded upward to whole frames, except after the final play. Assemble temporary PCM and encode once to MP3 or WAV. This avoids repeated lossy compression and a filter graph growing with the number of plays. [Rendering options](../research/technical-options.md#slower-and-faster-playback).

Working sample rate, channel count, and known channel layout are preserved. Unsupported MP3 source rates/channel counts produce an error directing the caller to WAV, without automatic conversion. Backend failures discard temporary output; successful files are published without replacing an existing destination. The CLI reports actual assembled PCM duration separately from the pre-render estimate. The user accepted the Shami sample's speech quality; other target-language listening checks remain pending.

The CLI summary should report the input, supplied language/dialect, format path, unit/group/play counts, estimated duration before rendering, and output path and actual duration afterward. Duration is a planning aid, not a reason to reduce the requested repetition.

Input errors, format errors, inability to form usable units, and rendering failures should be distinguishable to the caller. No output should be reported as complete when rendering fails.
