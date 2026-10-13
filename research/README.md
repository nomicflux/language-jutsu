# Language Jutsu research and design

Research began 7 October 2026; YouTube ingestion research added 9 October 2026. Current implementation and planned extensions are recorded in the design documents.

The [first-pass CLI design](../design/README.md) now makes the requested formula sequencing concrete. The research below remains background; draft design choices are identified in the design documents.

The intended Rust application would turn foreign-language recordings into phrases that a learner can slow down, speed up, repeat in smaller pieces, and gradually hear together. It should support both interactive practice and preparing audio ahead of time for listening without app interaction. The leading proposals are adjustable loops for interactive use and automatically generated study tracks for walking, driving, or working.

## Original request

Conversation context, quoted verbatim:

> I want to create a new project in Rust, likely with Wispr.
>
> It should take an audio file (or link to one) in a foreign language, cut it up into phrases, and play it for a learner to be able to become familiarized with the words - slowing it down, speeding it up, taking smaller chunks in repetition, then joining them up.
>
> This is research and brainstorming for now, not coding. Do the research, help me think through the design. Keep research documents.

## Additional use case

Conversation context, quoted verbatim:

> Ok. I want some automated approaches as well - one of the main use cases will be to take audio, process it ahead of time, and then I can listen to it while walking, driving, working, etc., where I will not have the focus or ability to manually play around with an app. I'm providing an audio that I know I want to study thouroughly.

## Language priorities and repetition

Conversation context, quoted verbatim:

> 6 seconds of speech into 34 seconds of practice: seems reasonable, the point is repetition to familiarize oneself with a new language.
>
> First languages will be Arabic, Japanese, and Spanish. Arabic and Japanese will be the most important.

Further conversation context, quoted verbatim:

> Arabic dialects: Shami and Masri are the most important to start.

Arabic and Japanese are the primary languages; Spanish is also in the initial scope. Shami and Masri are the starting Arabic priorities. The illustrated expansion is acceptable to the user. Repetition serves familiarization, and generated duration helps plan listening sessions. The exact recipe remains a proposal.

## Practice-unit boundaries

Conversation context, quoted verbatim:

> Phrase boundaries don't have to be clause-exact; significant pauses in speech can be sufficient. The point is more to have reasonably small and intelligible units to work off of.

Significant pauses are sufficient candidate boundaries. Units should be reasonably small and intelligible; clause matching and word-level alignment are not required.

## Research documents

- [Spotify ingestion](./spotify-ingestion.md): public podcast/RSS audio, selective passthrough links, and limits of Spotify playback APIs.
- [YouTube ingestion](./youtube-ingestion.md): current downloader/runtime requirements and integration into the implemented CLI.
- [Broader listening research](./listening-research-survey.md): sound familiarization, word recognition, comprehension, captions, extensive listening, spacing, attention, and walking studies.
- [Listening experience and learning evidence](./listening-design.md): concrete learner interaction, repetition research, phrase boundaries, and questions for further design.
- [Automated study audio](./automated-study-audio.md): preparing an entire selected recording for unattended repetition, speed changes, progressive joining, and revisits.
- [Rust audio and speech technology options](./technical-options.md): Wispr and Whisper, timing and alignment, playback, file and link ingestion, and interface choices.
- [Arabic, Japanese, and Spanish](./language-priorities.md): language-specific recognition and chunking findings, with Arabic and Japanese prioritized.

The documents distinguish sourced findings from design proposals. The initial languages and their priorities are selected; the stack, platform, and exact learning sequence remain open. Library capabilities come from documentation; suitability for actual learner recordings still needs listening trials.

The broader survey responds to the user's request for research beyond the current task design. Its findings and optional ideas do not add requirements to the application.

## Working direction

Keep the original recording intact. Use significant pauses to derive reasonably small, intelligible practice units, then use them for manual loops or a prepared sequence of repetitions, speed changes, and joins. Transcription and alignment can optionally help with smaller cuts or long stretches without pauses. The unattended mode advances without selecting difficult passages, reading a transcript, or confirming comprehension during playback. Boundary correction can remain available during preparation.

Wispr Flow has a public API, but its published behavior includes editing spoken content, and its documented responses do not expose word timings. Whisper has a documented timing route and a Rust-compatible local engine. See the [speech-engine comparison](./technical-options.md#wispr-and-whisper).

The next useful design comparison is repetition and progressive joining on Shami, Masri, and Japanese recordings, with Spanish included. Recording types and the preferred listening device remain open.
