#!/usr/bin/env python3
"""Obtain an independent audio report; this is not a correctness oracle."""

import argparse
from dataclasses import asdict
import hashlib
from importlib.metadata import version
import json
from pathlib import Path
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audio", required=True, type=Path)
    parser.add_argument("--model", required=True, type=Path)
    parser.add_argument("--language", required=True)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    audio = args.audio.resolve(strict=True)
    model_path = args.model.resolve(strict=True)
    if args.output.exists():
        parser.error("output already exists")
    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "json", str(audio)],
        check=True, capture_output=True, text=True,
    )
    duration = float(json.loads(probe.stdout)["format"]["duration"])
    if not 0 < duration <= 30:
        parser.error("Qwen2-Audio accepts at most 30 seconds; prepare contextual excerpts first")
    config = json.loads((model_path / "config.json").read_text())
    if config.get("model_type") != "qwen2_audio":
        parser.error("this helper supports the exercised qwen2_audio backend only")
    from mlx_audio.stt.utils import load_model

    prompt = (
        f"The requested language is {args.language}. Transcribe the audible speech in its "
        "original language. Describe what the speakers say and how their turns relate, "
        "quoting the audible words supporting your account. Distinguish questions from "
        "claims. If words or facts are unclear, say so; do not guess."
    )
    started = time.monotonic()
    model = load_model(model_path)
    result = model.generate(
        str(audio), prompt=prompt, max_tokens=1024, temperature=0.0, verbose=False,
    )
    if not result.text or not result.text.strip():
        raise RuntimeError("audio model returned no report")
    if result.generation_tokens >= 1024:
        raise RuntimeError("audio report reached the token limit; prepare a shorter contextual excerpt")
    payload = {
        "audio": str(audio),
        "audio_sha256": hashlib.sha256(audio.read_bytes()).hexdigest(),
        "audio_duration_seconds": duration,
        "model": str(model_path),
        "mlx_audio_version": version("mlx-audio"),
        "prompt": prompt,
        "result": asdict(result),
        "elapsed_seconds": time.monotonic() - started,
    }
    # result.segments contains generation elapsed time, not audio alignment.
    with args.output.open("x", encoding="utf-8") as handle:
        json.dump(payload, handle, ensure_ascii=False, indent=2, allow_nan=False)
        handle.write("\n")
    print(args.output.resolve())


if __name__ == "__main__":
    main()
