"""Language Jutsu's stdin/stdout bridge to mlx-whisper (Apple silicon).

Read one JSON request from stdin; emit one JSON response to stdout. Diagnostic
output belongs on stderr. The Rust executable embeds this file.
"""

import contextlib
import importlib.metadata
import json
import sys


def transcribe(request):
    try:
        import mlx_whisper
    except ImportError as error:
        raise RuntimeError(
            "mlx-whisper is unavailable in this Python environment; install "
            "mlx-whisper with this interpreter on Apple silicon, then pass "
            "that interpreter using --whisper-python"
        ) from error

    settings = {
        "language": request["language"],
        "task": "transcribe",
        "word_timestamps": True,
        "temperature": 0.0,
        "condition_on_previous_text": False,
        "initial_prompt": None,
    }
    result = mlx_whisper.transcribe(
        request["audio_path"],
        path_or_hf_repo=request["model"],
        verbose=None,
        **settings,
    )
    return {
        "backend": {
            "name": "mlx-whisper",
            "version": importlib.metadata.version("mlx-whisper"),
            "model": request["model"],
        },
        "settings": settings,
        "transcript": result,
    }


def main():
    try:
        request = json.load(sys.stdin)
        # Imports, model downloads, and backend diagnostics must not corrupt JSON.
        with contextlib.redirect_stdout(sys.stderr):
            response = transcribe(request)
        json.dump(response, sys.stdout, ensure_ascii=False, allow_nan=False)
        sys.stdout.write("\n")
    except Exception as error:
        print(f"mlx-whisper helper failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
