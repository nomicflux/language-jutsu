#!/usr/bin/env python3
"""Exercise a packaged runtime with only its own tools and macOS tools on PATH."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import zipfile

root = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("payload", type=Path)
parser.add_argument("--youtube-url", help="Also exercise a live YouTube acquisition and render")
args = parser.parse_args()
payload = args.payload.resolve()
with tempfile.TemporaryDirectory(prefix="Jutsu installed runtime ") as temporary:
    install = Path(temporary)
    shutil.copytree(payload / "runtime", install / "runtime", symlinks=True)
    (install / "bin").mkdir()
    launcher = install / "bin/language-jutsu"
    shutil.copy2(root / "packaging/macos/launcher.sh", launcher)
    launcher.chmod(0o755)
    env = dict(os.environ, PATH=f"{install}/runtime:/usr/bin:/bin:/usr/sbin:/sbin")
    def run(*args):
        result = subprocess.run([str(a) for a in args], cwd=install, env=env, capture_output=True, text=True)
        if result.returncode:
            print(result.stdout, file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            result.check_returncode()
        return result.stdout
    run(install / "runtime/yt-dlp", "--version")
    run(install / "runtime/deno", "eval", "if (1 + 1 !== 2) throw Error('runtime failed')")
    shutil.copy2(root / "tests/fixtures/jfk.wav", install / "speech.wav")
    run(launcher, "speech.wav", "--language", "en", "--start", "0", "--end", "4", "--output", "study.mp3")
    run(install / "runtime/ffmpeg", "-nostdin", "-v", "error", "-i", "study.mp3", "-f", "null", "-")
    run(launcher, "download-model")
    run(launcher, "speech.wav", "--language", "en", "--start", "0", "--end", "4", "--transcribe", "--output", "transcript.txt")
    assert (install / "transcript.txt").read_text().strip()
    manifest = {"deck_id": 1369839123, "deck_name": "Installer check", "language": "en", "cards": [{
        "key": "listening", "kind": "listening", "prompt": "What was said?",
        "answer": "And so, my fellow Americans", "context": {"audio": "speech.wav", "sentence": "And so, my fellow Americans"},
        "marks": [], "explanation": "Installer packaging check."}]}
    (install / "cards.json").write_text(json.dumps(manifest))
    run(launcher, "anki", "cards.json", "--output", "lesson.apkg")
    with zipfile.ZipFile(install / "lesson.apkg") as deck:
        assert "collection.anki2" in deck.namelist()
    if args.youtube_url:
        run(launcher, "--url", args.youtube_url, "--language", "en", "--start", "0", "--end", "4", "--output", "youtube.mp3")
        run(install / "runtime/ffmpeg", "-nostdin", "-v", "error", "-i", "youtube.mp3", "-f", "null", "-")
        print("Live YouTube acquisition and MP3 render passed.")
    print("Packaged runtime passed: helper execution, MP3 rendering, default model download/cache, transcription, Anki.")
