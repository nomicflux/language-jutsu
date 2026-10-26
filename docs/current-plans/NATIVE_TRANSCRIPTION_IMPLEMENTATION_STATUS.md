# Native transcription — implementation status

## Phase 1 — Native transcription: complete (2026-10-26)

### Delivered

- **Native transcription:** `--transcribe` runs whisper.cpp in-process (whisper-rs 0.16, Metal on macOS; whisper.cpp 1.8.3). No Python remains in the transcription path.
- **Model cache:** `language-jutsu download-model [MODEL]` caches `ggerganov/whisper.cpp` model files in the Hugging Face cache (default `ggml-large-v3-turbo.bin`). `--whisper-model` names the file; a cached model works offline.
- **Language checks:** unsupported language codes and languages missing from the loaded model fail before any output.
- **Word grouping:** follows Whisper's reference rule. It keys on whisper.cpp's canonical code, so `--language japanese` groups like `ja`; real Botchan speech exposed this bug.
- **Deletions (U11):** `scripts/mlx_whisper_helper.py`, `tests/helpers/whisper_python.py`, `--whisper-python`, MLX model-directory paths.
- **Lint fix (U12):** the `src/anki.rs` nested `if` is collapsed.
- **Docs:** the README transcription section, `SKILL.md:94`, `preparation.md`, `boundaries.md:5`, and `design/transcription-and-metadata.md` (untracked user draft, updated, not committed) are updated.
- **U13:** stale Whisper Python and MLX model entries are removed from `listening/settings.md` (untracked).

### Changes from the planned verification

- **Real tests only.** Per the user's instruction "TESTS NEED TO BE REAL TESTS, NOT FAKE TESTS":
  - The invented-input unit tests and the synthetic-tone integration tests were replaced by tests that run `ggml-tiny.bin` on real public-domain speech in `tests/fixtures/` (JFK inaugural clip; LibriVox Botchan opening; provenance in `tests/fixtures/SOURCES.md`).
  - These tests cover transcribed words, whole-word and character-group grouping, original-timeline times for a later window, the language round trip, rejections, no-clobber, and `download-model`.
- **Window upper bound dropped.** The planned check "every time within the selected window" was the agent's invention. Whisper sets segment ends from its timestamp tokens, as mlx-whisper did (`transcribe.py:396–404`), so that check was removed.

### Checks run

- `cargo fmt --check` → exit 0.
- `cargo clippy --all-targets -- -D warnings` → exit 0.
- `cargo test` → 71 passed (1 + 2 + 57 + 4 + 7), 0 failed.
- `cargo run -- download-model` → `Model cached: ~/.cache/huggingface/hub/models--ggerganov--whisper.cpp/snapshots/5359861c…/ggml-large-v3-turbo.bin`.
- Default model, `qahwa_1.mp3 --language ar --start 01:00 --end 01:15 --transcribe`:
  - Exit 0; offset 60.0; segments 60.0–75.0. The phrase "طيب / لي بتحبل اهوك / يعني" repeats; the tiny model independently heard the same repetition in this window.
  - The same command with `HF_ENDPOINT=http://127.0.0.1:9` (offline) exits 0 with a byte-identical transcript.
- YouTube `mK-xzqnJmSQ --start 03:00 --end 03:10 --transcribe` → exit 0; offset 180.0; segments 180.0–190.0.
- The grep for `whisper-python|whisper_python|mlx_whisper|mlx-whisper|mlx whisper` over `src tests scripts skills README.md design Cargo.toml .cargo` returns no matches.

## Phase 2 — Setup scripts and README rewrite: not started
