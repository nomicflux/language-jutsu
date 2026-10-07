# Automated study audio for unattended listening

Research date: 7 October 2026. Automatic preparation and unattended listening are requested capabilities; the recipes and export formats below are design proposals.

The learner supplies a recording they already want to study thoroughly. The application processes it ahead of time into audio that supplies its own repetition, slower and faster versions, smaller chunks, and progressive joins. Listening can proceed while walking, driving, or working, without choosing passages, reading text, answering prompts, or confirming readiness.

## Preparation and listening

During preparation, import the file or link, automatically find phrases and smaller useful units, choose a recipe, and generate the result. Default preparation should work without editing each phrase. Optional boundary correction and a short preview can remain available.

During listening, the result runs by itself. A portable study track is the simplest delivery proposal: repetitions and speed changes are already in the audio, so an ordinary player can play it. A dedicated app could also execute the same sequence. Preparation controls belong before listening; pause, resume, and seek can be available without being necessary to follow the lesson.

## Three automatic approaches

| Approach | What happens automatically | Main tradeoff |
| --- | --- | --- |
| Phrase repetition | Each phrase plays at normal speed, slower, then normal again before advancing | Simple and predictable, but does little to reconnect neighboring phrases |
| Progressive joining | Play the whole utterance, practice its smaller pieces, and build back to the whole | Closest to the requested thorough chunk practice; substantially expands listening duration |
| Passage revisits | After local practice, replay the complete passage; revisit it in later parts or listening sessions | Restores context and provides repeat exposure after intervening material, but adds time |

These approaches can combine. A leading recipe is progressive joining within an utterance, followed by an uninterrupted passage replay after several utterances. Full-passage revisits can also form a lighter track for subsequent listens.

## A worked progressive sequence

Suppose an utterance has three adjacent chunks A, B, and C, each two seconds long. Use the original continuous source interval whenever playing A+B or A+B+C.

| Step | Audio played | Speed | Plays |
| --- | --- | --- | --- |
| 1 | A+B+C | 1.0× | 1 |
| 2 | A | 0.8×, then 1.0× | 2 |
| 3 | B | 0.8×, then 1.0× | 2 |
| 4 | A+B | 1.0× | 1 |
| 5 | C | 0.8×, then 1.0× | 2 |
| 6 | A+B+C | 1.0× | 1 |

An optional final 1.1× play supplies a faster version. That can be chosen at preparation time; reaching it does not depend on feedback during listening. The exact speeds and counts are experimental settings, not established learning optimums.

The core sequence is nine plays totaling 29.5 seconds of sound. Adding a half-second gap between plays gives 33.5 seconds for a six-second original utterance. The optional faster play and one additional gap raise that to about 39.5 seconds. The user considers the roughly six-to-34-second expansion reasonable: repetition is intended to familiarize them with the language. This accepts the illustrated duration, without fixing a universal expansion ratio or recipe.

Short gaps separate repeated plays without requiring the learner to speak or respond.

## Choosing chunks automatically

Use significant pauses as the starting boundaries. The requested units are reasonably small and intelligible; they do not have to match clauses. Preparation can propose merging very short fragments or dividing a long span at smaller pauses. Pause sensitivity and useful lengths remain settings to explore on recordings, without a universal threshold chosen here.

Transcription and alignment are optional aids for finer cuts or speech with few pauses. They are not prerequisites for pause-based preparation. All units receive the chosen practice treatment; the learner should not have to identify the hard ones first. If a finer cut is uncertain, keep the enclosing unit.

Limit cumulative joining to a local utterance or passage. Building A, A+B, A+B+C through an entire recording would replay its beginning increasingly often and create disproportionate duration. Finish a local group, advance, and return to larger uninterrupted context at intervals. Whole-recording replay can remain available.

## Thorough coverage and listening duration

Apply the recipe across the supplied recording in source order. Keep all spoken material in either small-chunk practice or enclosing-span practice, and include joined context. Uncertain speech should receive enclosing-span practice rather than be silently skipped.

Show the generated listening duration before export. It is the sum of every selected source duration divided by its playback speed, plus added pauses. Repeated and overlapping joined spans count each time they play.

Duration is a planning aid. A long generated track alone is not a reason to reduce the chosen repetition. Arabic and Japanese are the primary initial languages, alongside Spanish; see [language-specific chunking research](./language-priorities.md).

For a long result, generate ordered session files ending at complete practice groups. A desired session length can determine those breaks without truncating the source or weakening the chosen recipe. The last session may be shorter. This serves intermittent available time while preserving the selected recording's coverage.

## Evidence and limits

See the [broader listening research survey](./listening-research-survey.md#timing-attention-and-retention) for additional studies of pauses, spacing, attention, and walking.

Jensen and Vinther compared three repetitions in different fast/slow combinations for Spanish learners. Their abstract reports improvements but no established difference between treatment combinations. That supports investigating predetermined repetition without claiming a best speed ladder. [Exact Repetition as Input Enhancement in Second Language Acquisition](https://onlinelibrary.wiley.com/doi/abs/10.1111/1467-9922.00230).

A 2023 conference poster reports 73 English learners listening to a 20-minute story on five consecutive days, with vocabulary outcomes measured immediately afterward. It supports considering repeated listens across sessions, but does not establish long-term retention, progressive joining, or effectiveness while doing other work. [Chang, The Effects of Repeated Listening on Second Language Incidental Vocabulary Acquisition](https://discovery.ucl.ac.uk/id/eprint/10188114/).

A 2017 German-listening thesis could not draw conclusions about spaced versus massed practice because only 13 participants had usable data and posttest scores could not distinguish them. It cannot establish an optimum revisit interval. [Emery, Spaced Versus Massed Practice in L2 German Listening Comprehension](https://scholarsarchive.byu.edu/etd/6295/).

The proposed automation schedules exposure; it cannot infer comprehension from the number of plays. The useful next comparison is to listen to two prepared versions of the same short recording without touching controls: straightforward phrase repetition versus progressive joining with passage replays. Judge understandable transitions, reconnecting words into fluent speech, repetition fatigue, and total duration.
