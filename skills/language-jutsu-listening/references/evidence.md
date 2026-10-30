# Evidence for transcript corrections

Use this guidance to resolve a specific wording problem while preparing the lesson. It does not call for a separate evidence paper, implementation analysis or a routine sequence of model comparisons.

## Change only what the source supports

Retain the original transcript. Change it only when there is a specific reason and source evidence establishing the correction; otherwise leave it unchanged. Leaving uncertain ASR unchanged does not make it verified teaching material.

For each actual edit, briefly record the original-source interval, before/after wording and the particular evidence that proves the change and fits the recorded speech. Include deletions, additions, spelling, word boundaries and punctuation when they change interpretation. Keep this concise justification with the working transcript, outside learner-facing cards; no separate report is required.

Use the scene, surrounding turns, speaker intentions, referents and dialect knowledge to understand the exchange and identify possible readings. Establish the proposed reading from the speech and specific contextual evidence: for example, an explicit repetition, a name spelled out, or a later reply resolving a referent. General plausibility, a more familiar expression, fluent wording, repeated decoding or recognizer agreement does not prove a correction. Do not claim to have heard a distinction that the available tools did not let you assess.

Never change wording merely because another form is more common, standard, polite, grammatical or expected. Preserve spoken repetitions, false starts, unusual expressions and apparent mistakes. Do not silently remove a clitic, syllable or distinction between names to obtain a familiar form.

Publisher text must belong to the exact recording. Translated subtitles can support meaning but cannot establish source-language wording. Keep publisher text, unprompted transcription and your interpretation distinct. Check answer-critical speaker relationships, negation, numbers and referents in their enclosing exchange.

Resolve wording needed for teaching and answers using the recording and its context. If it remains uncertain, leave the transcript unchanged and mark the uncertainty separately. Replace unsuitable passages you selected and continue preparing the lesson. Do not teach uncertain wording as established, use it as an answer key, or reduce the requested substantive conversation to its one easy fact.

Apply supported corrections consistently to dependent translations, vocabulary, grammar, questions and cards. Their wording must come from the source, not merely agree with an edited transcript. This evidence work supports constructing the lesson; it is not a substitute for it.

## Existing tools for a specific unresolved detail

Use the retained recording and relevant surrounding turns first. When a new decode can resolve a specific problem, Jutsu can transcribe its enclosing exchange without the candidate answer in the prompt. A repeat decode is another estimate, not independent proof.

The existing `scripts/compare_audio.py` is an optional Qwen2-Audio comparison using original-speed contextual WAV. Do not write a replacement tool, inspect its implementation or set up another model as a routine lesson stage. This helper has failed on some Arabic recordings; repeating a failed comparison does not resolve uncertainty.

Prepare its input through Jutsu with the bundled one-play format, keeping enough context for the disputed wording:

```sh
"$jutsu" "$source" \
  --language "$language" --start "$start" --end "$end" \
  --format "$skill/assets/reference.json" \
  --output "$lesson/evidence.wav" --metadata-output "$lesson/evidence.json"

"$audio_python" "$skill/scripts/compare_audio.py" \
  --audio "$lesson/evidence.wav" --model "$audio_model" \
  --language "$language" --output "$lesson/audio-report.json"
```

The helper accepts at most 30 seconds. If the specific disputed relationship requires more context, use overlapping contextual excerpts through Jutsu. This analysis input is not a learner-facing practice player.

Read `result.text` as a candidate account, not a verdict. Its `result.segments` contain generation timing, not source-audio timing; never use them to select source intervals. Compare the proposed wording with the recording and exact-source evidence under the correction rule above. If the helper is unavailable or inconclusive, that does not authorize a transcript change.
