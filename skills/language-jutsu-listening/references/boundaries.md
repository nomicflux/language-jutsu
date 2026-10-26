# Implemented boundaries

These excerpts were checked against the implementation on 13 October 2026. Inspect current code when changing an interface. Lesson `task.md` is read and written by the assistant; it is not a JSON request accepted by Jutsu. Scheduling uses the exposed desktop tool contract, whose implementation is not available here.

The Language Jutsu CLI, Rust/Python transcription bridge, source retention, and rendered metadata shapes are traced with code excerpts in [transcription-and-metadata.md](../../../design/transcription-and-metadata.md).

## The app constructs the study audio

Verified against the current source on 20 October 2026. The CLI parses source, excerpt bounds, format path and output path into `Request` in [cli.rs:145](../../../src/cli.rs#L145). The format file supplies settings, not a hand-authored list of units or plays. [format.rs:10](../../../src/format.rs#L10) defines and parses that JSON:

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Format {
    pub(crate) chunking: Settings,
    pub(crate) playback: Formula,
}
// Format::load, after opening the supplied path:
let mut format: Self = serde_json::from_reader(BufReader::new(file))
    .map_err(|error| format!("invalid format {}: {error}", path.display()))?;
format.chunking.apply_override(target_override);
```

The nested input fields are defined by [segmentation.rs:46](../../../src/segmentation.rs#L46) and [formula.rs:37](../../../src/formula.rs#L37):

```rust
pub(crate) struct Settings {
    chunk_target_size_seconds: ChunkTargetSize,
}
struct RawJoining {
    group_size: NonZeroUsize,
}
struct RawFormula {
    sequencing: Sequencing,
    segments: Vec<f64>,
    joining: RawJoining,
    repetitions: NonZeroUsize,
    gap_ms: u64,
}
```

Serde reads `ChunkTargetSize` from a validated positive `f64`; `Sequencing` uses kebab-case enum names. `Formula::try_from` validates speeds and maps the raw fields into the app's playback formula. [app.rs:25](../../../src/app.rs#L25) loads that format; its render path at [app.rs:66](../../../src/app.rs#L66) performs these steps (intervening reporting omitted):

```rust
let decoded = decode(&input, request.language, request.window)?;
let analysis = segmentation::analyze(&decoded, settings)?;
let sequence = sequence::Sequence::build(analysis.units.len(), formula)?;
let plan = sequence::Plan::build(&analysis, &format, decoded.info(), &sequence)?;
let frames = render::publish(&decoded, &analysis.units, &sequence, &request.output)?;
```

[sequence.rs:45](../../../src/sequence.rs#L45) expands the unit count and playback formula once into an owned `Sequence`: every play's unit range, speed, repetition and following gap are stored. A joining group size of one emits no multi-unit spans. [render.rs:95](../../../src/render.rs#L95) iterates `sequence.plays()`, reads each source span, applies its stored tempo and inserts its stored gap; [render.rs:139](../../../src/render.rs#L139) encodes and publishes the completed file at the requested output path. The renderer receives the sequence and source units, with no playback-formula argument. The returned study artifact is this file. Its optional metadata and the sequence-report shapes are shown below; they describe the app's output, not an assistant-built assembly of component files.

## Playback reports used by lesson preparation

The two output reports deliberately have different shapes. [sequence.rs:162](../../../src/sequence.rs#L162) constructs the sequence-only report from the stored sequence:

```rust
write!(file, "{{\"segmentation\":")?;
// Serialize segmentation and format, then:
write!(file,
    ",\"group_count\":{},\"play_count\":{},\"estimated_seconds\":{},\"plays\":[",
    self.sequence.group_count, self.sequence.plays().len(), self.estimated_seconds
)?;
let mut document = serde_json::to_value(play)?;
document["source_range"] = serde_json::to_value(play.source_range(&self.analysis.units))?;
let row = serde_json::json!({
    "index": index, "play": document,
    "estimated_start_seconds": output_seconds,
    "estimated_sound_seconds": sound_seconds,
    "following_gap_ms": play.following_gap.as_millis()
});
serde_json::to_writer(file.as_file_mut(), &row)?;
```

[metadata.rs:70](../../../src/metadata.rs#L70) constructs and serializes render metadata:

```rust
let document = serde_json::json!({
"schema_version": 1,
"source": source,
"format": format.document(),
"sequence": {
    "group_count": plan.sequence.group_count,
    "play_count": plan.sequence.plays().len(),
    "estimated_duration_seconds": plan.estimated_seconds,
},
"output": {
    "path": audio_output.canonicalize()?,
    "sample_rate": source.sample_rate,
    "channels": source.channels,
    "pcm_frames": rendered_frames,
    "pcm_duration_seconds": rendered_frames as f64 / f64::from(source.sample_rate.get()),
},
});
serde_json::to_writer_pretty(file.as_file_mut(), &document)?;
```

The assistant reads these reports; there is no separate lesson-state parser in Jutsu. Estimates come from planned spans/speeds; actual duration comes from rendered PCM frames. The rewrite exercised both report types and checked each player's actual duration against ffprobe.

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
