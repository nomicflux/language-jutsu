# Listening experience and learning evidence

Research date: 7 October 2026. The interface details and practice recipes below are design proposals.

The same phrase boundaries can support two experiences: interactive loops and audio processed ahead of time for listening while walking, driving, or working. The unattended mode automatically repeats, changes speed, and joins chunks across the selected recording. The interactive mode lets the learner adjust those choices directly. See [automated study audio](./automated-study-audio.md) for the prepared listening sequences.

Arabic and Japanese are the primary initial languages, with Spanish also included. Repetition for familiarization is central, and the user considers the illustrated six-to-34-second expansion reasonable. See [language priorities](./language-priorities.md) for recognition and segmentation implications.

## A concrete interactive listening session

After importing a file or an audio link, the learner sees suggested phrases. Selecting one plays that part of the recording. A waveform shows the selection, with an optional transcript as another way to select a passage.

Consider this constructed Spanish example, used only to illustrate the interaction:

“Si tienes tiempo después del trabajo, podemos ir a tomar algo.”

Meaning: “If you have time after work, we can go and get something to drink.” Suggested pieces might be “Si tienes tiempo”, “después del trabajo”, and “podemos ir a tomar algo”. These boundaries are suggestions for practice; a particular recording might join the first two in one continuous breath.

1. Hear the complete utterance at its original speed.
2. Select the difficult passage, such as “después del trabajo”, and repeat it. Adjust speed if needed.
3. If that remains difficult, select a smaller region within it. The learner chooses where attention is needed; shortening should not always discard the end or beginning.
4. Expand to hear “Si tienes tiempo después del trabajo”, then the complete utterance again.
5. Return to the original speed, or try a faster speed if desired, and continue into the surrounding recording.

In interactive use, the learner can move directly to a larger span when ready, return to a smaller one, or keep listening normally. In unattended use, a chosen preparation recipe supplies the repeat counts and transitions in advance.

## What repetition and speed research supports

The [broader listening research survey](./listening-research-survey.md) adds work on recognizing sounds and words, captions, extensive listening, pauses, spacing, attention, and concurrent walking.

Jensen and Vinther’s 2003 study involved 84 upper-intermediate Spanish learners and two experiments. It compared three repetitions of an utterance in fast–slow–fast, fast–slow–slow, and fast–fast–fast combinations with a baseline group. The publisher’s abstract reports significant pre/post effects on comprehension, decoding strategies, and linguistic features, but no established difference between the treatment conditions. This supports exploring repetition; it does not establish that slowing speech is necessary or that one sequence is best. [Exact Repetition as Input Enhancement in Second Language Acquisition](https://onlinelibrary.wiley.com/doi/abs/10.1111/1467-9922.00230).

Meinardi’s 2009 article reports preliminary tests in which slowing authentic speech often improved perception and understanding, particularly for faster samples with more words. That concerns understanding the presented samples; it does not establish durable learning or transfer to unfamiliar recordings. [Speed bumps for authentic listening material](https://www.cambridge.org/core/journals/recall/article/abs/speed-bumps-for-authentic-listening-material/419EE91CA8D2160EE97E44A63766060F).

**Design implication:** make speeds, repetition, and chunk sizes configurable during preparation as well as interactive practice. A normal–slow–normal sequence is a candidate recipe, alongside normal-speed repetition. The research above does not validate a universal speed, chunk length, repeat count, or progressive joining method.

An illustrative speed control could offer 0.75×, 0.9×, 1.0×, and 1.1× plus finer adjustment. These numbers are starting ideas for auditioning, not research-derived optimums. An adjustable pause between repetitions would let the learner process what they heard; it should be possible to remove the pause for continuous listening.

## What makes a useful phrase

A practice unit should be reasonably small and intelligible. Significant pauses can supply sufficient boundaries, even when the resulting units do not match clauses. Pause-based splitting is the starting proposal; transcript punctuation and alignment can optionally help refine it.

whisper.cpp labels word timing experimental, and WhisperX documents alignment limitations. Boundary correction would let the learner move either edge while previewing the audio, split a phrase, or combine adjacent phrases. A wrong transcript should not prevent selecting a span directly. [whisper.cpp](https://github.com/ggml-org/whisper.cpp), [WhisperX limitations](https://github.com/m-bain/whisperX#limitations-).

For languages without spaces between written words, a selectable sound region still works even when automatic text tokenization is uncertain. Do not assume that a model token is a complete spoken word. For connected speech, a little surrounding sound can make an isolated part more recognizable; the appropriate margin should come from listening rather than a universal padding rule.

## Joining the pieces

For adjacent pieces, expanding the selection and playing the original continuous passage would retain the speaker’s timing, pauses, and transitions. Concatenating separately padded clips could repeat boundary sounds or insert artificial gaps. This is a proposed interpretation of “joining them up”; edited concatenation remains a different possible behavior.

One possible automatic progression is A, B, A+B, C, A+B+C, applied across the recording. In interactive practice, the learner can instead start with the whole phrase and expand only the difficult region until it reaches that whole.

Keep practice pauses separate from pauses in the recording. Joining should remove the added practice gaps while preserving the original pauses. An automatic recipe can advance and accelerate after preset repetitions; that is scheduled practice, not a claim that the learner has mastered the phrase.

## Text and meaning support

Two experiences are plausible:

| Approach | What the learner gets | Main tradeoff |
| --- | --- | --- |
| Audio first | Listen and select sound; reveal the transcript when helpful | Greater listening emphasis, but finding an unknown word can be difficult |
| Transcript alongside audio | Select text to loop its corresponding audio | Easier navigation, but the learner may follow the text rather than recognize the speech |

Optional transcript visibility would let the interactive player serve both approaches. Unattended playback must work without either. Recognized text can disagree with the recording, so correction can be available during preparation. Familiarity with a sound does not by itself supply an unknown word’s meaning; a short meaning hint is a possible addition if vocabulary understanding is part of the desired experience.

## Existing tools to learn from

WorkAudioBook’s developer listing already describes automatic phrase splitting, repeat and pause controls, playback speed, and waveform selection down to individual fragments. It is a close functional reference. The listing does not establish how clean its boundaries are on the intended recordings. [WorkAudioBook developer listing](https://play.google.com/store/apps/details?hl=en&id=com.povalyaev.WorkAudioBook).

Audacity offers selection looping and a Change Tempo effect that preserves pitch. It is a useful reference for manual span selection and comparing slowed audio. Its controls belong to an audio editor; the opportunity here is to make shrinking and growing a listening span feel immediate. [Audacity looping](https://www.audacityteam.org/manual/toolbar/transport/), [Audacity Change Tempo](https://www.audacityteam.org/features/speed-changer/).

## Choices to discuss next

- **Material within the selected languages:** Shami and Masri are the starting Arabic priorities, alongside Japanese and with Spanish also included. Clean narration, podcasts, drama dialogue, and overlapping conversation create different phrase and timing problems.
- **Preparation and listening devices:** a desktop waveform could help with optional boundary edits, while exported study audio could play on the device already used for walking, driving, or working.
- **Automatic recipe:** choose how often chunks repeat, how much they build up, and when complete passages return. No readiness signal is needed during unattended playback.
- **Purpose of text:** navigation only, help when stuck, or a visible part of studying the words.

A useful next research exercise would compare a prepared study sequence with interactive loops on a short representative recording. For the prepared version, listen without operating controls and judge whether repetition, transitions, and larger passages remain understandable when attention is intermittent. Both versions need audible boundaries and clear speed changes.
