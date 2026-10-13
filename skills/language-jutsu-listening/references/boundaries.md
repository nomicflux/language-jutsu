# Implemented boundaries

These excerpts were checked against the implementation on 13 October 2026. Inspect current code when changing an interface. Lesson `task.md` is read and written by the assistant; it is not a JSON request accepted by Jutsu. Scheduling uses the exposed desktop tool contract, whose implementation is not available here.

The Language Jutsu CLI, Rust/Python transcription bridge, source retention, and rendered metadata shapes are traced with code excerpts in [transcription-and-metadata.md](../../../design/transcription-and-metadata.md).

## Independent audio report

The bundled [compare_audio.py](../scripts/compare_audio.py) parses four arguments at lines 15–20: `--audio`, `--model`, `--language`, `--output`. It resolves real files, reads `ffprobe` JSON `format.duration`, checks the backend type and the 30-second input bound, and constructs a neutral prompt without candidate answers. The ffprobe implementation is external; its parser expectation is visible in the helper and its output was exercised, not established from FFmpeg source here. Lines 44–64 construct and serialize the model request/report:

```python
model = load_model(model_path)
result = model.generate(
    str(audio), prompt=prompt, max_tokens=1024, temperature=0.0, verbose=False,
)
payload = {
    "audio": str(audio),
    "audio_sha256": hashlib.sha256(audio.read_bytes()).hexdigest(),
    "audio_duration_seconds": duration, "model": str(model_path),
    "mlx_audio_version": version("mlx-audio"), "prompt": prompt,
    "result": asdict(result), "elapsed_seconds": time.monotonic() - started,
}
json.dump(payload, handle, ensure_ascii=False, indent=2, allow_nan=False)
```

The pinned backend's [prompt construction](https://github.com/Blaizzy/mlx-audio/blob/e66ad2ff4eb96ce1149000187b30370cbcd707aa/mlx_audio/stt/models/qwen2_audio/qwen2_audio.py#L353) loads the audio, embeds its features, and combines audio markers with the supplied text:

```python
content = "\n".join(audio_parts) + "\n" + user_prompt
chat = [{"role": "user", "content": content}]
prompt_str = tokenizer.apply_chat_template(chat, tokenize=False, add_generation_prompt=True)
```

Its [response construction](https://github.com/Blaizzy/mlx-audio/blob/e66ad2ff4eb96ce1149000187b30370cbcd707aa/mlx_audio/stt/models/qwen2_audio/qwen2_audio.py#L519) returns:

```python
segments = [{"start": 0.0, "end": elapsed, "text": text}]
return STTOutput(
    text=text,
    segments=segments,
    prompt_tokens=prompt_tokens,
    generation_tokens=gen_tokens,
    total_tokens=prompt_tokens + gen_tokens,
    total_time=elapsed,
    prompt_tps=prompt_tokens / prefill_time if prefill_time > 0 else 0,
    generation_tps=gen_tokens / gen_time if gen_time > 0 else 0,
)
```

The complete [dataclass](https://github.com/Blaizzy/mlx-audio/blob/e66ad2ff4eb96ce1149000187b30370cbcd707aa/mlx_audio/stt/models/base.py#L6) serialized by `asdict` is:

```python
@dataclass
class STTOutput:
    text: str
    segments: List[dict] = None
    language: str = None
    prompt_tokens: int = 0
    generation_tokens: int = 0
    total_tokens: int = 0
    prompt_tps: float = 0.0
    generation_tps: float = 0.0
    total_time: float = 0.0
```

These exact pinned source files were fetched and inspected during skill development, and the packaged wrapper was exercised with actual audio. The skill consumes `result.text` and compares its words with separate evidence; `result.segments` is not alignment.
