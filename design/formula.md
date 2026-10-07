# Formula and sequence semantics

Draft date: 7 October 2026. The required controls are timing changes and sequencing between timing and joining. The additional fields below are proposed first-pass controls.

## Editable JSON

The proposed default is saved at [formulas/default.json](../formulas/default.json):

```json
{
  "sequencing": "timing-then-joining",
  "segments": [0.75, 0.9, 1.0, 1.1],
  "joining": {
    "group_size": 3
  },
  "repetitions": 1,
  "gap_ms": 500
}
```

`segments` follows the user's example and means an ordered list of playback-speed multipliers. A factor below 1 slows speech; above 1 speeds it up; 1 preserves the original tempo. Pitch-preserving rendering is the proposed audio behavior.

All displayed values come from the JSON. The default factors, sequencing choice, group size, repetitions, and gap must not also appear as fallback constants in application code. Algorithms for expanding a formula can exist in code; the selected recipe comes from data.

## Build the spans before ordering the plays

A, B, and C denote consecutive pause-based audio units. Partition the source-ordered units into local groups of at most `group_size`. Finish a group's entire sequence before advancing to the next group. The last group can be shorter.

The proposed forward build emits the first unit, then each next unit followed by the growing prefix:

| Group | Ordered spans |
| --- | --- |
| A | A |
| A, B | A, B, A+B |
| A, B, C | A, B, A+B, C, A+B+C |
| A, B, C, D | A, B, A+B, C, A+B+C, D, A+B+C+D |

The one-unit case emits A once, without an identical extra “join.” `group_size: 1` supplies individual-unit practice. Larger values increase the extent of the cumulative build. Grouping and this forward pattern are design proposals, rather than user-selected defaults.

A+B means the original source interval from A's beginning to B's end. It includes their original intervening pause. It excludes added practice gaps. It is rendered as one continuous span, not assembled from separately slowed practice clips.

## `timing-then-joining`: speed is the outer loop

For each speed in `segments`, visit every span in the group's build order. Repeat each speed/span occurrence `repetitions` times before advancing.

For A and B with the default four speeds and one repetition:

```text
0.75A, 0.75B, 0.75(A+B),
0.9A,  0.9B,  0.9(A+B),
1.0A,  1.0B,  1.0(A+B),
1.1A,  1.1B,  1.1(A+B)
```

This matches the user's first example. For three units, each row instead visits `A, B, A+B, C, A+B+C`.

## `joining-then-timing`: span is the outer loop

For each span in the group's build order, visit every speed in `segments`. Repeat each speed/span occurrence `repetitions` times before advancing.

For the same A and B:

```text
0.75A,     0.9A,     1.0A,     1.1A,
0.75B,     0.9B,     1.0B,     1.1B,
0.75(A+B), 0.9(A+B), 1.0(A+B), 1.1(A+B)
```

This matches the user's second example. “Then” names nesting order in both labels: first the outer dimension, then the inner dimension.

## Repetitions and gaps

With `repetitions: 2`, the first sequence starts:

```text
0.75A, 0.75A, 0.75B, 0.75B, 0.75(A+B), 0.75(A+B), ...
```

The second starts:

```text
0.75A, 0.75A, 0.9A, 0.9A, 1.0A, 1.0A, 1.1A, 1.1A, ...
```

`gap_ms` is a fixed output-time silence between every pair of plays, including repeat, speed, span, and group transitions. No added gap follows the final play of the recording. A zero value permits back-to-back plays. Gaps are not slowed with the source audio; original pauses inside a source span are.

## Duration and example walkthrough

For each play, estimated sound duration is source-span duration divided by its speed. Sum all plays and add `(number of plays - 1) × gap_ms / 1000`. The renderer's actual output duration can differ slightly due to processing and encoding.

For A and B each two seconds long, with no original inter-unit pause, the spans total `2 + 2 + 4 = 8` seconds. The four-speed multiplier is:

```text
1/0.75 + 1/0.9 + 1/1.0 + 1/1.1 = approximately 4.353535
```

One repetition yields 12 plays, approximately 34.828 seconds of sound, plus 11 half-second gaps: approximately 40.328 seconds total.

For three two-second units, the five spans total `2 + 2 + 4 + 2 + 6 = 16` seconds. The proposed default yields 20 plays and approximately 79.157 seconds including 19 gaps. This recipe supplies more occurrences than the earlier six-to-34-second research illustration; that illustration did not fix the recipe.

Both sequencing choices have the same estimated duration for the same groups, speeds, repetition count, and fixed gaps. Their ordering differs.

## Parsing rules for the proposed format

- `sequencing` must be one of the two defined values.
- `segments` must be a nonempty array of finite, positive numbers. Preserve its supplied order and duplicates; do not sort or deduplicate it.
- `joining.group_size` and `repetitions` must be positive integers.
- `gap_ms` must be a nonnegative integer.
- All five top-level fields are required in this proposed extended format. A custom formula replaces the default formula completely; it is not merged with missing fields from another recipe.
- Use strict JSON: the comment in the user's illustrative example is explanatory and is omitted from the actual file.

Report invalid or missing fields before rendering. Unsupported renderer speeds must be reported rather than silently clamped. A missing JSON file must not trigger an internal default recipe.
