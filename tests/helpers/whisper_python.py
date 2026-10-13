#!/usr/bin/env python3
"""Exercise the embedded helper without a GPU; MODE is set by the Rust test."""
import importlib.metadata
import json
import os
import struct
import subprocess
import sys
import types


def fake_transcribe(audio_path, **kwargs):
    assert kwargs == {
        "path_or_hf_repo": "test/model",
        "verbose": None,
        "language": "ja",
        "task": "transcribe",
        "word_timestamps": True,
        "temperature": 0.0,
        "condition_on_previous_text": False,
        "initial_prompt": None,
    }, kwargs
    pcm = subprocess.check_output([
        "ffmpeg", "-v", "error", "-i", audio_path,
        "-f", "f32le", "pipe:1",
    ])
    # Original 16 kHz ramp, selected [0.10001, 0.40001): frame-ceiled bounds.
    assert len(pcm) == 4800 * 4, len(pcm)
    first, = struct.unpack("<f", pcm[:4])
    assert first == 1601 / 32768, first
    print("backend diagnostics must not enter transcript JSON")
    if MODE == "backend_failure":
        raise RuntimeError("test model failed")
    return {
        "text": " お茶にします。",
        "language": "es" if MODE == "wrong_language" else "ja",
        "segments": [{
            "id": 0, "seek": 0, "start": 0.02, "end": 0.25,
            "text": " お茶にします。", "temperature": 0.0,
            "avg_logprob": -0.3, "compression_ratio": 1.2,
            "no_speech_prob": 0.01,
            "words": [{
                "word": "お茶", "start": 0.04,
                "end": -1 if MODE == "bad_timestamp" else 0.12,
                "probability": 0.8,
            }],
        }],
    }


if MODE == "invalid_json":
    print("not JSON")
    sys.exit(0)
if MODE == "missing_dependency":
    sys.modules["mlx_whisper"] = None
else:
    sys.modules["mlx_whisper"] = types.SimpleNamespace(transcribe=fake_transcribe)
    importlib.metadata.version = lambda name: "test-version"
exec(sys.argv[2])
