# Language Jutsu: first CLI design

Updated: 9 October 2026. CLI configuration, decoding, source-window selection, segmentation, formula expansion, pitch-preserving tempo processing, and MP3/WAV rendering are implemented. The user accepted speech quality of the Shami excerpt. Automated CLI validation and a full-recording render passed; other target-language and unattended listening checks remain pending. The [research index](../research/README.md) preserves the earlier requests and findings.

## First-pass request

Conversation context, quoted verbatim:

> First pass: CLI that takes an audio file with a given language setting, and outputs learning chunks by slowing down, speeding up, and buliding up piecewise. Formula can be given, with a default saved (as a JSON file, not hardcoded; code should require a formula).

The requested distinction is between these orders:

```text
0.75A, 0.75B, 0.75(A+B), 0.9A, 0.9B, 0.9(A+B), ...

0.75A, 0.9A, 1.0A, 1.1A, 0.75B, 0.9B, 1.0B, 1.1B,
0.75(A+B), ...
```

The draft retains `sequencing` and `segments` from the user's illustrative JSON. Here `segments` contains speed multipliers; audio units and joined spans are separate concepts.

Earlier requirements remain active: reasonably small, intelligible units; significant pauses sufficient for boundaries; thorough repetition and progressive joining; preparation for unattended listening; Arabic and Japanese prioritized, with Shami and Masri first for Arabic and Spanish also included. File input is the first CLI pass. Link input and interactive controls remain part of the broader app design.

## Input window and added pauses

`--start` and `--end` select a window on the original recording's timeline before segmentation. Omitted bounds mean the recording's beginning or end. These options are implemented for decoding, segmentation, and sequence planning. Source positions remain absolute and the planned playback timeline starts at zero. See the [CLI model](./cli-and-processing.md#source-window) for boundary semantics.

Conversation context, quoted verbatim:

> Ok. I want to add in pauses to the overall formatting too, as the research suggested.

Added processing pauses are requested output behavior. `gap_ms` inserts them between plays, including group transitions, with no added gap after the final play. The editable JSON currently supplies 500 ms, fixed in output time independently of speed and rounded upward to a whole frame. Original pauses within joined spans remain intact.

## Documents and configuration

- [Adaptive chunking](./adaptive-chunking-plan.md): automatic level estimation and confidence-ranked pauses; target duration is the only chunking setting.

- [YouTube URL input](./youtube-input-plan.md): implemented `--url` acquisition through yt-dlp, feeding the existing CLI pipeline and preserving source origin.
- [Audio processing implementation plan](./audio-processing-plan.md): media input, pause-based units, formula expansion, tempo rendering, export, and actual audio validation.
- [Formula and exact sequence semantics](./formula.md): the two loop orders, joining, repeat placement, gaps, and worked examples.
- [CLI and processing model](./cli-and-processing.md): command examples, one-file format loading, audio units, source-time preservation, and rendering.
- [Editable default format](../formats/default.json): the actual proposed JSON file, shared by the examples.

The specific CLI syntax, output layout, group size, repeat count, and pause duration remain workshop choices. External JSON formulas and added processing pauses are user requirements.

## Proposed first-pass experience

Supply an audio file, language setting, optional format path, and output path. The CLI finds practice units, constructs the requested sequence, and renders it for later listening. It advances without learner feedback. The proposed output is one study audio file containing the rendered chunks in order, suitable for an ordinary player.

One format JSON contains `chunking` and `playback`. Use `--format`, or load the distributed `formats/default.json` beside the executable. The processing core always receives a parsed format. Missing or invalid data fails; there is no built-in fallback recipe.

Use significant pauses as the starting cuts. Transcription or word alignment can optionally help refine them, but clause-exact boundaries are not required. Joined spans use the continuous original recording, preserving its internal timing.

## Additional levers worth considering now

| Lever | What it changes | Draft treatment |
| --- | --- | --- |
| Repetitions | How many consecutive times a particular span plays at a particular speed | Include in the proposed formula; default 1, with speed variations already supplying repeated exposure |
| Gap between plays | Processing time after each occurrence, independently of speech speed | Requested output behavior; JSON currently supplies 500 ms, without claiming an optimum |
| Joining group size | How many neighboring units participate in one local build before advancing | Include; proposed 3, avoiding cumulative joins from the beginning of an entire recording |
| Pause sensitivity | Which natural pauses become unit boundaries | Estimated automatically from the audio; the learner supplies the target duration in the same format as playback settings |
| Target chunk length | How much source speech to practice at once | `chunking.chunk_target_size_seconds` in the format JSON, default 6; `--chunk-target-size-seconds` overrides it, selecting qualifying pauses nearest that duration |
| Boundary margins | How much surrounding source audio is retained around a cut | Tune technically if needed to avoid losing speech; joins must not duplicate margins |

Pauses deserve comparison with slowdown: Sugai and colleagues found that extra pause time aided comprehension in their English-learning setting, while slower articulation did not necessarily do so. This supports separate controls, without selecting the draft gap value. [Study](https://www.jstage.jst.go.jp/article/arele/27/0/27_97/_article/-char/en).

Later possibilities include an introductory whole-group play, a final normal-speed replay, and returning to passages after intervening material or in later sessions. Broader spacing research supports investigating revisits, but does not determine a phrase-audio schedule. These features are not included in the draft formula. [Spacing meta-analysis](https://onlinelibrary.wiley.com/doi/abs/10.1111/lang.12479).

Neither loop order is established here as pedagogically superior. The default uses `timing-then-joining` to make the first example concrete; changing the JSON selects the second.
