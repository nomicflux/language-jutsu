# From audio to defensible learning questions

## Build the evidence before the question

Obtain any publisher transcript/captions for the exact recording. Keep source text, unprompted Jutsu/Whisper output, and your interpretation distinct. Translated subtitles, summaries, nearby episodes, and titles are not the recording's transcript. Compare the audible propositions needed for the proposed task: speaker, number, negation, referent, and whether a turn is a question, assertion, quotation, or imagined event.

In the lesson's private answer notes, save each answer proposition with the source words, speaker/turn support, original interval, corroborating source, uncertainty, and enclosing replay bounds. Derive the prompt from supported relationships, not trivia selected before listening evidence exists. Write an achievable response frame that does not give away the answer. Accept equivalent meaning, not just the saved sentence.

If text is missing or doubtful, Jutsu transcribes the audio. For a disputed answer-critical detail, transcribe its enclosing exchange again without supplying the candidate answer. Wider context may fix a cut word or referent. A repeated decode is another estimate, not independent proof.

## Compare actual audio when needed

The bundled `scripts/compare_audio.py` implements the exercised local Qwen2-Audio route. It takes the actual original-speed contextual WAV, not Whisper text, and asks for transcription and supporting words without the intended answer. It is needed when publisher evidence is unavailable or an answer-critical disagreement remains; it does not establish universal language/dialect coverage or guarantee correctness.

On a supported Apple Silicon machine, use an existing configured runtime or prepare a dedicated durable environment. The investigated runtime version is pinned here so this helper's boundary is reproducible:

```sh
python3 -m venv "$runtime"
"$runtime/bin/python" -m pip install \
  'mlx-audio @ git+https://github.com/Blaizzy/mlx-audio.git@e66ad2ff4eb96ce1149000187b30370cbcd707aa'
"$runtime/bin/python" -c \
  'from huggingface_hub import snapshot_download; import sys; snapshot_download("mlx-community/Qwen2-Audio-7B-Instruct-4bit", local_dir=sys.argv[1])' \
  "$audio_model"
```

Set `runtime` and `audio_model` to chosen durable setup locations; this downloads a multi-gigabyte model. Save the resulting Python/model paths in `listening/settings.md`. Do not install or redownload for every lesson. If the runtime is unavailable, complete acquisition and transcription work, then resolve that concrete dependency before claiming this comparison happened.

```sh
"$audio_python" "$skill/scripts/compare_audio.py" \
  --audio "$lesson/repair.wav" --model "$audio_model" \
  --language "$language" --output "$lesson/audio-report.json"
```

The backend only consumes 30 seconds. The helper refuses longer files. For longer requested content, **the skill** renders and analyzes overlapping excerpts that retain complete conversational turns, recording each original interval; it does not truncate the source or ask the learner to work around this analysis limit. If one needed relationship crosses excerpts, retain overlap and compare it across them. All requested audio still belongs to the task.

Read `result.text` as a report to compare with Whisper and any publisher evidence. Its `result.segments` contain generation timing, not audio timing: never use them to cut replays. The script records the input path, hash, measured duration, model path, runtime version, neutral prompt, full model result, and elapsed time. Actual construction is in [compare_audio.py](../scripts/compare_audio.py); the model's report is not a verdict.

## Resolve what the evidence can establish

Models can agree and still be wrong. Compare their exact supporting words, the surrounding audio-derived turns, and any reliable publisher evidence. Investigate disagreements on the same requested content. Do not infer speaker identities from Whisper fields or correct a spelling from a weaker model merely because it produced a confident sentence.

A doubtful name spelling may coexist with a supported travel duration. Ask about the supported meaning while keeping the name uncertainty explicit. If the user specifically asked to settle that name, it remains an unresolved part of their task; asking a different question does not complete it. Likewise, unresolved negation or numbers cannot become a correct-answer key. Preserve alternatives and explain the limit in the target language while working on the supported portions. Do not narrow the skill's language/content scope to whichever samples these models handled well.

When a learner's answer conflicts with supported evidence, use its contextual replay and a focused question. When the evidence itself is unresolved, do not mark the learner wrong. Supplying the transcript is teaching support; after it is supplied, record subsequent success as supported comprehension rather than unaided listening.
