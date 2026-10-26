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

## Phase 2 — Setup scripts and README rewrite: in progress, CI pending (2026-10-26)

### Second attempt (the replanned Phase 2)

- `.cargo/config.toml` deleted; whisper.cpp bindings are generated on every platform.
- `setup.sh`: macOS with Homebrew (`rust cmake ffmpeg deno` when missing; standalone `yt-dlp_macos` → `/usr/local/bin/yt-dlp`), Debian/Ubuntu x86_64 (`apt-get install --no-install-recommends ca-certificates curl unzip build-essential cmake libclang-dev ffmpeg`, rustup, Deno installer, `yt-dlp_linux` → `/usr/local/bin/yt-dlp`); other systems exit 1 with "setup.sh supports macOS with Homebrew and Debian/Ubuntu on x86_64".
- `setup.ps1`: throws "setup.ps1 supports Windows on x64" unless AMD64; winget `yt-dlp.yt-dlp`, `Kitware.CMake`, `LLVM.LLVM` when missing (throws on a nonzero exit); rustup-init `x86_64`.
- README Setup section rewritten per target.
- `.github/workflows/setup.yml`: macos-15; ubuntu:24.04 and debian:12 containers as a sudo user with the `python3` check; windows-2025 with the winget bootstrap.

Local checks (Mac):

- `sh -n setup.sh` exit 0; `shellcheck setup.sh` reports only SC1007 (`CDPATH=`) and SC2086 (`brew install $packages`).
- After `cargo clean -p whisper-rs-sys` (the build script reruns only when `wrapper.h` changes): `cargo fmt --check` 0, `cargo clippy --all-targets -- -D warnings` 0, `cargo test` 71 passed (1 + 2 + 57 + 4 + 7). The generated `bindings.rs` contains `struct __sFILE`.
- `./setup.sh` exit 0: `Model cached: ~/.cache/huggingface/hub/models--ggerganov--whisper.cpp/snapshots/5359861c…/ggml-large-v3-turbo.bin`; "Already installed" for both skill links; `brew list --formula` identical before and after.
- Every README CLI example, run as written in a fresh copy of the tracked files (no `.cargo/config.toml`), exited 0. The 17 runs: local default, YouTube default, joining-then-timing, YouTube shorter, the chunk override, `--save-source`, `--check`, sequence, units, metadata, `download-model`, file transcription, URL transcription, the decoder, the ffmpeg WAV conversion, `anki`, and the standalone binary.

### First attempt

The failures below were planning omissions. Phase 2 was replanned from a full dependency trace ([setup-dependencies.md](setup-dependencies.md)); the new Phase 2 is in [native-transcription-and-setup.md](native-transcription-and-setup.md).

### Done

- `setup.sh` and `setup.ps1` rewritten (no Python steps); `README.md` rewritten. Linux `yt-dlp` uses `yt-dlp_linux_aarch64` on aarch64 (`yt-dlp_linux` is x86_64 only); apt/pacman lists add `curl`, which the rustup, Deno and yt-dlp downloads need.
- `sh -n setup.sh` exit 0; `shellcheck` reports only SC1007 and SC2086 (expected).
- macOS `./setup.sh`: exit 0, with all brew tools already present. The model was cached and the skill installed in `~/.agents/skills` and `~/.claude/skills`. A rerun reports "Already installed".
- Every README CLI example ran in a scratch copy and exited 0: file and URL with each format, the chunk override, `--save-source`, `--check`, sequence, units, metadata, `download-model`, file and URL transcription, the decoder plus ffmpeg, `anki`, and the standalone binary.
- `cargo fmt --check` 0, `cargo clippy --all-targets -- -D warnings` 0, `cargo test` 71 passed.

### Failures

1. **Homebrew `yt-dlp` installs Python.** `brew deps yt-dlp` lists `python@3.14`. A fresh Mac running `setup.sh` would install Python (U6).
2. **Debian apt installs Python.** In the Debian bookworm container run, `apt-get install` pulled `python3` through ffmpeg's Recommends. Simulation: with Recommends, 4 python3 packages; with `--no-install-recommends`, 0.
3. **The whisper.cpp build fails on aarch64 Linux (GCC 12.2, Debian bookworm).** `ggml-cpu/vec.cpp`: `inlining failed in call to 'always_inline' 'vaddq_f16' … target specific option mismatch`. The variant flags were `-march=armv8-a+…+fp16fml…` and `GGML_COMPILER_SUPPORTS_FP16_FORMAT_I3E - Failed`. whisper-rs-sys passes `GGML_*`/`CMAKE_*` environment variables to CMake (`build.rs:279–285`). x86_64 Linux has not been run.

Windows: not executable here. Not committed.
