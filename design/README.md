# Language Jutsu: first CLI design

Draft date: 7 October 2026. Design documents and editable configuration only; application implementation has not started. The [research index](../research/README.md) preserves the earlier requests and findings.

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

## Documents and configuration

- [Formula and exact sequence semantics](./formula.md): the two loop orders, joining, repeat placement, gaps, and worked examples.
- [CLI and processing model](./cli-and-processing.md): command examples, required formula loading, audio units, source-time preservation, and rendering.
- [Editable default formula](../formulas/default.json): the actual proposed JSON file, shared by the examples.

The specified CLI syntax, output layout, joining group size, repeat count, and gap are proposals for workshop. The requirement to obtain a formula from JSON, rather than hardcode a default recipe, comes from the user.

## Proposed first-pass experience

Supply an audio file, language setting, optional formula path, and output path. The CLI finds practice units, constructs the requested sequence, and renders it for later listening. It advances without learner feedback. The proposed output is one study audio file containing the rendered chunks in order, suitable for an ordinary player.

When a formula path is omitted, load the distributed default JSON file. The processing core always receives a parsed formula. Missing or invalid formula data is an error; there is no built-in fallback recipe.

Use significant pauses as the starting cuts. Transcription or word alignment can optionally help refine them, but clause-exact boundaries are not required. Joined spans use the continuous original recording, preserving its internal timing.

## Additional levers worth considering now

| Lever | What it changes | Draft treatment |
| --- | --- | --- |
| Repetitions | How many consecutive times a particular span plays at a particular speed | Include in the proposed formula; default 1, with speed variations already supplying repeated exposure |
| Gap between plays | Processing time after each occurrence, independently of speech speed | Include; proposed 500 ms, without claiming an optimum |
| Joining group size | How many neighboring units participate in one local build before advancing | Include; proposed 3, avoiding cumulative joins from the beginning of an entire recording |
| Pause sensitivity | Which natural pauses become unit boundaries | Treat as input-segmentation settings, separate from the formula's playback order; thresholds remain to be auditioned |
| Desired unit length | Whether a pause-defined unit is too short or too long for comfortable practice | Treat as a segmentation preference; preserve intelligibility rather than enforce blind cuts at exact seconds |
| Boundary margins | How much surrounding source audio is retained around a cut | Tune technically if needed to avoid losing speech; joins must not duplicate margins |

Pauses deserve comparison with slowdown: Sugai and colleagues found that extra pause time aided comprehension in their English-learning setting, while slower articulation did not necessarily do so. This supports separate controls, without selecting the draft gap value. [Study](https://www.jstage.jst.go.jp/article/arele/27/0/27_97/_article/-char/en).

Later possibilities include an introductory whole-group play, a final normal-speed replay, and returning to passages after intervening material or in later sessions. Broader spacing research supports investigating revisits, but does not determine a phrase-audio schedule. These features are not included in the draft formula. [Spacing meta-analysis](https://onlinelibrary.wiley.com/doi/abs/10.1111/lang.12479).

Neither loop order is established here as pedagogically superior. The default uses `timing-then-joining` to make the first example concrete; changing the JSON selects the second.
