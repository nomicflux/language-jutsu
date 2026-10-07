# Rust audio and speech technology options

Research date: 7 October 2026. Documentation-based comparison of candidate technologies; audio quality and performance on the intended recordings remain unmeasured.

Rust can coordinate importing, analysis, automatic study sequencing, rendering, and interactive playback. Processing ahead of time makes an exported study track a leading option for unattended listening. The difficult choices remain accurate boundaries and clear speech at altered speed.

Arabic and Japanese are the primary initial languages, with Spanish also required. Significant pauses can supply practice boundaries without transcription. [Language-specific research](./language-priorities.md) covers optional transcription and alignment: Japanese alignment units can be individual characters, and Arabic recognition quality depends on the speech variety.

## Wispr and Whisper

### Wispr Flow

Wispr has an official Voice Interface API with REST and WebSocket access. Its introduction says that output can remove fillers, apply spoken corrections, and make messages more concise. Those features suit dictation, but they can remove exactly the content a language learner is hearing. [Wispr API introduction](https://api-docs.wisprflow.ai/introduction).

The documented REST input is base64-encoded 16 kHz WAV, limited to 25 MB or six minutes. Its response exposes text, detected language, processing time, and generated-token count; it does not expose word or phrase timestamps. Processing time is not an audio boundary. [Wispr REST transcription](https://api-docs.wisprflow.ai/rest_api_transcribe).

The published request schema offers language and dictation context, but no documented verbatim switch. WebSocket examples expose partial/final text and packet position, which does not establish word alignment. **Conclusion from these schemas:** Wispr is not yet a demonstrated source for faithful timed transcripts. A vendor-supported verbatim mode and alignment output, or a separate alignment approach, would need confirmation before choosing it. [Request schema](https://api-docs.wisprflow.ai/request_schema), [WebSocket API](https://api-docs.wisprflow.ai/websocket_api).

Billing is token-based and contract-dependent; the public billing page does not give enough information for a cost-per-hour comparison. [Wispr usage and billing](https://api-docs.wisprflow.ai/usage_billing).

### Whisper locally through Rust

whisper-rs provides Rust bindings to whisper.cpp, with documented Metal, CUDA, and Vulkan feature flags. Its example consumes 16 kHz mono floating-point audio and returns segment timings in centiseconds. [whisper-rs documentation](https://docs.rs/crate/whisper-rs/latest).

whisper.cpp supports VAD, or voice activity detection, using a separate model. Its word-timestamp feature is explicitly experimental. The project’s example model table lists roughly 466 MiB on disk and 852 MB memory for small, versus 2.9 GiB and 3.9 GB for large; these are project estimates, not measurements on the intended device. [whisper.cpp documentation](https://github.com/ggml-org/whisper.cpp).

This is the leading candidate **if local transcription is desired**. Rust bindings still bring a C/C++ engine and model assets. Choose a multilingual model for foreign-language input; a model named with `.en` is an English-specific choice. Timing quality, model size, and acceptable import delay need comparison on the actual language and hardware.

### Hosted Whisper

OpenAI’s file-transcription guide documents word and segment timestamps for `whisper-1`, using `verbose_json` and `timestamp_granularities`. That parameter is specific to `whisper-1`; the other listed transcription models are not interchangeable for this output. Uploads are limited to 25 MB. Long-input processing chunks would need their offsets mapped back to the original recording; they need not become the learner’s phrases. [OpenAI file transcription](https://developers.openai.com/api/docs/guides/speech-to-text).

A hosted engine could reduce local model setup, but requires sending audio to a service and paying for analysis. Repeated practice could still use the retained audio locally. This remains an alternative until offline behavior and acceptable upload handling are decided.

## Speech detection and alignment

Three different results matter:

| Result | What it supplies | What it does not establish |
| --- | --- | --- |
| Voice activity detection | Likely regions containing speech; gaps can suggest pause-based cuts | The words or clause boundaries |
| Transcription | A hypothesis about the spoken text | Exact boundaries suitable for tiny loops |
| Alignment | Estimated locations of transcript units in the audio | Correct transcript content or ideal practice chunks |

WhisperX combines transcription with forced alignment to obtain word timestamps. Its documentation describes language-specific alignment models and limitations with overlapping speech and text outside an alignment model’s dictionary. It is a Python-based option, so choosing it would introduce another runtime or service alongside Rust. **Possible use:** a comparison reference or an alignment candidate if simpler timing is audibly inadequate. [WhisperX](https://github.com/m-bain/whisperX).

**Proposed segmentation approach:** start with significant pauses and derive reasonably small, intelligible units. Clause-exact boundaries are not required. Transcription, punctuation, and alignment are optional aids for finer cuts or uninterrupted speech. Optional preparation controls can split, merge, or nudge boundaries; reviewing every unit must not be necessary to generate study audio. Keep an enclosing unit when a finer cut is uncertain. Preserve missing or uncertain alignment as uncertainty when alignment is used.

## Slower and faster playback

For learning speech, pitch-preserving tempo changes are the leading proposal. Rodio’s ordinary speed control changes pitch by the same factor as speed. It therefore does not by itself provide the desired natural-sounding slowdown. [Rodio Player speed documentation](https://docs.rs/rodio/latest/rodio/struct.Player.html#method.set_speed).

| Candidate | Documented capabilities and integration | Main consideration |
| --- | --- | --- |
| Signalsmith Stretch | C++ time stretching and pitch shifting; MIT licensed; the Rust `ssstretch` wrapper exists | Strong candidate to audition; processor latency and clip starts/ends need handling |
| Rubber Band | C++ library with independent tempo and pitch controls | GPL distribution or commercial licensing needs to fit the eventual application |
| SoundTouch | C++ tempo changes without pitch change; LGPL v2.1 with a commercial alternative | Another candidate to audition for short speech loops |
| FFmpeg `atempo` | Documented tempo filter, including runtime tempo commands | Useful for rendered comparison clips; bringing an external media tool adds packaging work |

Sources: [Signalsmith Stretch](https://signalsmith-audio.co.uk/code/stretch/), [Rust ssstretch wrapper](https://docs.rs/ssstretch/latest/ssstretch/struct.Stretch.html), [Rubber Band](https://breakfastquay.com/rubberband/), [SoundTouch](https://www.surina.net/soundtouch/), [FFmpeg atempo](https://ffmpeg.org/ffmpeg-filters.html#atempo).

Signalsmith documents input and output latency, pre-roll, and flushing at the end of finite audio. Those details matter here: a short loop should not lose its first consonant, acquire unwanted startup silence, or truncate its end. A candidate processor should be judged by listening to real speech, including repeated restarts and changes of speed. Documentation does not establish which library sounds best for this use.

For unattended listening, render the complete study sequence ahead of time: source spans, altered speeds, repeats, joins, and practice pauses. Playback then needs only ordinary audio controls. Interactive practice can render the current selection or stream through the stretcher for live speed changes. Both approaches can preserve the original audio.

## Preparing an automatic study track

The proposed flow is: import the recording, analyze it, derive phrases and subchunks, apply a preparation recipe, and render playable audio. The recipe visits all spoken material in the selected recording; transcript confidence should not become a filter that silently drops difficult passages. Detailed recipe alternatives are in [automated study audio](./automated-study-audio.md).

Retain each planned play's source range, speed, following pause, and any containing phrase or passage. Output positions are derived from those richer instructions when rendering; one source span may occur many times. Keep this sequence alongside the original media and analysis so a different recipe can be rendered without treating a repeated MP3 as the source of truth.

FFmpeg documents trimming, tempo adjustment, and concatenation, providing a candidate batch-rendering path alongside Rust-controlled DSP. The concat filter requires each input segment to start at timestamp zero. Continuous joined phrases should be read as one original source span; concatenation joins the separate practice steps. Encode the finished sequence rather than repeatedly compressing and joining tiny encoded files. [FFmpeg audio trimming](https://ffmpeg.org/ffmpeg-filters.html#atrim), [tempo](https://ffmpeg.org/ffmpeg-filters.html#atempo), [concatenation](https://ffmpeg.org/ffmpeg-filters.html#concat).

An exported MP3 or another format supported by the chosen player is a leading delivery proposal. Ordered session files can divide a long result at completed practice groups. Rendering on a computer and listening on a phone would avoid requiring the Rust application itself to run on the listening device. A dedicated app remains useful for preparation and interactive practice, but screen interaction is not required for the unattended sequence.

## Audio output and decoding

Rodio is a Rust playback library built on CPAL. Its repository currently warns of an engine rewrite and differences between repository examples and released versions. CPAL is the lower-level cross-platform audio I/O option if direct scheduling becomes necessary. Neither choice automatically solves time stretching. [Rodio](https://github.com/RustAudio/rodio), [CPAL](https://github.com/RustAudio/cpal).

Symphonia provides Rust media demuxing and audio decoding. Supported formats and codecs depend on feature flags; its current table lists HE-AAC and Opus gaps in the built-in implementation. Therefore an extension such as `.m4a` or `.webm` does not guarantee that every enclosed audio stream is supported. [Symphonia format and codec tables](https://github.com/pdeljanov/Symphonia).

A Rust decoder could cover a defined input set. FFmpeg is a possible broader conversion path if actual recordings need it. Keep the original-quality playback source separate from any mono 16 kHz analysis copy; the speech engine’s input format should not reduce the learner’s only copy.

## Files and links

An audio-file link and a web page containing media require different import handling. The first supplies media bytes; the second may require a site-specific extractor. yt-dlp documents audio extraction and its FFmpeg/ffprobe dependency. It is a possible external tool for supported media pages, with site compatibility and packaging to consider. It does not make every URL a usable audio source. [yt-dlp audio extraction](https://github.com/yt-dlp/yt-dlp#post-processing-options).

The proposal should retain both local-file input and link input. An initial supported link set could be direct audio URLs, then named media sites if those are the intended sources. Retaining an imported recording locally would make repetition and seeking independent of the network. The intended meaning of “link” remains open; no requirement for arbitrary websites has been inferred.

## Data and playback organization

The original media and analysis supply both interactive selections and automatically prepared study sequences.

**Proposed data treatment:** retain original media, its timebase, raw analysis output, and the learner’s boundary/text edits separately. Practice spans refer to that original timeline. Preserve transcript units as returned, including missing timings; derive displayed phrases from those richer results. Reanalysis should not silently replace learner corrections.

Speed changes alter output duration, not the stored source boundaries. The player must map its audible position back to source time, including time-stretch latency. Replaying adjacent pieces together should read the continuous original range and remove only added practice gaps. This avoids accumulated timing drift and duplicated padding from concatenated clips.

The learner could manually select and play audio while transcription is running or unavailable.

## Interface candidates

| Candidate | Documented basis | Design tradeoff |
| --- | --- | --- |
| egui with eframe | Rust GUI that runs natively and on the web | A direct Rust route for a waveform-oriented desktop tool; egui does not target a native-looking interface |
| Tauri | Rust application logic with an HTML/CSS/JavaScript frontend and system webview | A web-style interface with Rust media work, at the cost of using two frontend/backend technology layers |

Sources: [egui](https://github.com/emilk/egui), [Tauri overview](https://v2.tauri.app/start/).

Preparation and playback need not use the same interface or device. Desktop is a candidate for importing and optional precise editing; portable exported audio could serve unattended listening. A mobile app is another possible delivery route. Neither platform nor interface toolkit is selected.

## Current technical direction

The leading preparation proposal is a Rust application that finds significant pauses, derives reasonably small and intelligible units, and renders a study track. whisper-rs/whisper.cpp, hosted Whisper, and WhisperX remain options for additional text and timing assistance; Signalsmith Stretch and FFmpeg remain rendering candidates. Keep manual audio selection available for interactive practice and optional preparation edits. Wispr remains a transcription option to investigate, with faithful-text and timing limitations noted above. Pause-based preparation does not depend on resolving those transcription choices.

An audible comparison on Shami, Masri, and Japanese recordings, alongside Spanish, would help settle the choices: clean cuts, intelligible slowdown, repeat restarts, joined phrases, and unattended transitions. For batch preparation, processing duration matters more than real-time transcription latency. Suitability within the selected languages, hardware performance, and service costs remain open.
