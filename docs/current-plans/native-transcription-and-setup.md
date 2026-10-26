# Native transcription, setup, and README

## 1. User requirements and definitions — verbatim

**U1** (user, 2026-10-26 12:50):
> I want to take this repo and make it useable for others.
>
> 1. Go over the README. Rewrite with how to set up the app, example usage (from the command line for given files & YouTube URLs, with different formats; and also from the SKILL, with the various options users can specify.)
> 2. Improve setup - this should work out of the box for a new user as much as possible (have a setup script for a single point of entry, along with downloads of the required transcription backends so that these do not have to be stored on Github). Update README.

**U2** (user's answer to "How should the setup script handle system tools (Rust/Cargo, FFmpeg/ffprobe, yt-dlp + Deno)?"):
> Homebrew for Mac; we need options for Windows & Linux too.

**U3** (user's answer to "Where should setup install the language-jutsu-listening skill?"; selected option text: "~/.claude/skills (Claude Code) and ~/.agents/skills (Codex), via the existing install-skill.sh."):
> Both

**U4** (user's answer to "Which models should setup download?"):
> Whichever is actually used. If the scripts are part of actual flow, include both; otherwise, just Whisper.

**U5** (user's answer to "Where should the downloaded Whisper model live?"; selected option text: "Pre-download into the standard HF cache, so the CLI's existing default `--whisper-model mlx-community/whisper-large-v3-turbo` works without a flag. No settings file written."):
> Hugging Face cache

**U6** (user, 13:03):
> If you are installing python, you done fucked up.

**U7** (user, after the post mortem):
> " Replace the transcription backend with one that doesn't need Python. " Does the existing one ACTUALLY need Python, or was that a hack in the first place.

**U8** (user):
> Native code. A C/C++ engine is not worse than a Python runtime.

**U9** (user's answer to the model-fetch question; selected option text: "The binary uses the hf-hub crate to fetch the file into the standard HF cache. A new `language-jutsu download-model` subcommand lets setup pre-fetch it; --transcribe uses the cached file by default."):
> HF cache + subcommand

**U10** (user's answer to "How should transcription be tested once the fake-Python helper is gone?"; options were "1. Real tiny model — Integration tests run real whisper.cpp with ggml-tiny.bin (75 MiB, fetched once and cached), checking source-timeline mapping, outputs and no-clobber. Tests need network on first run." and "2. Pure functions only — Unit tests cover the pure parts (token-to-word grouping, timestamp mapping, document building) with no model; the whisper.cpp call itself is untested."):
> 1 & 2

**U11** (user's answer to "May I delete scripts/mlx_whisper_helper.py, tests/helpers/whisper_python.py, the --whisper-python option, and passing a local MLX model directory to --whisper-model?"):
> Yes, delete all four

**U12** (user's answer to "May I collapse that `if` [src/anki.rs:325] in Phase 1?"):
> Yes, fix it

**U13** (user, after the plan was presented; the plan's note read "your own `listening/settings.md` still names `.venv-whisper` and the MLX model folder"):
> Remove stale setting. Proceed.

**Later user statements (verbatim):**
> TESTS NEED TO BE REAL TESTS, NOT FAKE TESTS.
>
> EVERY FAKE TEST IS FUCKING ROT.

> JUST MAKE IT WORK WITH REAL TESTS!!!!! STOP ASKING PERMISSION FOR WHAT YOU'VE BEEN TOLD TO DO, WHILE YOU SIMULTANEOUSLY CREATE SHITTY WORK YOU WEREN'T TOLD TO DO.

> THE SETUP SCRIPT SHOULD BE CALLING THE DOWNLOAD-MODEL IN THE APP, SO WHAT THE FUCK IS GOING ON?

> AND YOU HAD AN RM IN THERE, AND YOU WERE DOWNLOADING IT MANUALLY.

> Yes, proceed. Without extra bullshit.

> REDO FROM SCRATCH. TRACE DEPENDENCIES.
>
> WE DO NOT FIGURE THIS OUT IN IMPLEMENTATION, WE FIGURE THIS OUT IN PLANNING, YOU FUCKING IMBECILE.

> ". Examples: does a standalone yt-dlp with its own Python inside count as installing Python,"
>
> You have the context. What the fuck sort of question is this. YOU HAVE THE CONVERSATION AND _WHY_ WE DECIDED NOT TO INSTALL PYTHON, IT IS NOT A BLACK-BOX COMMAND.

After the Windows CI test failed on hf-hub's "Cache lock timed out" (the agent's report said "On Windows the download took longer than 10 seconds"):
> "On Windows the download took longer than 10 seconds" Uh huh, yes? So? We then wait longer. You fucked up the tests if this is a thing. OF COURSE it could take an indeterminate amount of time to download a model.

> You are still not addressing the root problem.

> "CAN'T VERIFY HERE"
>
> YOU _RECOGNIZED_ THAT YOU HAD A FLAW AND DID NOT SEARCH IT OUT.

> YES, MAKE IT WORK, AND MAKE IT WORK BY _CONSTRUCTION_. NO MORE ARBITRARY TIMEOUT MANAGEMENT OR HOPING AND PRAYING. PLAN SO THIS FUCKING _WORKS BY CONSTRUCTION AND RESEARCH_, THEN IMPLEMENT>

("YES" answers the agent's statement: model fetching must meet "HF cache + subcommand" on all three systems; on Windows the model goes in the user's real Hugging Face cache, in one copy, without being recopied on each run; a run that finds another download in progress waits for it, however long it takes, then uses that file instead of downloading again; the tests then download the model once and only read it after that.)

Answers on 2026-10-26 (question text in [setup-dependencies.md](setup-dependencies.md)):
- Verification of platforms this Mac cannot run: "GitHub Actions (Recommended)" (option: "Add a workflow file to the repo and push this branch to GitHub; … Publishes the branch and adds a CI file.")
- Windows ARM: "Where is this going to happen?"
- Debian 12 ARM: "You are targetted arbitrary builds. I just wanted a simple "a user could use this from a non-mac system""

### Source requirements (repository text, not the user)

- **README.md:109–111 (current):** "No playback format is loaded or accepted for transcription. File and YouTube inputs, source retention, and original-timeline windows work in this mode." "Segment and word times are estimated **original-source seconds**, shifted by the selected start frame divided by the source sample rate. The helper always uses `task="transcribe"`, never English translation. Use a language code supported by Whisper; an unsupported code fails explicitly."
- **README.md:55, 70:** "Existing output files are never replaced."
- **skills/language-jutsu-listening/references/preparation.md:38:** "The text file has a structured `.json` companion: `text`, `language`, `segments` (with `words`), `source`, `backend`, `settings`, `timestamp_reference`, and `source_offset_seconds`. Segment/word `start` and `end` are already original-source seconds."
- **skills/language-jutsu-listening/references/evidence.md:25:** "The existing `scripts/compare_audio.py` is an optional Qwen2-Audio comparison … Do not … set up another model as a routine lesson stage." (Basis for U4 → Whisper only.)
- **research/technical-options.md:27:** whisper-rs/whisper.cpp "is the leading candidate **if local transcription is desired**."

## 2. Investigation results

### Current behavior

- `src/transcription.rs` embeds `scripts/mlx_whisper_helper.py` (`include_str!`, line 14). `publish` checks that neither output exists, converts the selected PCM to WAV with ffmpeg, runs `Command::new(&options.python).args(["-c", HELPER])` (line 110), validates the helper's JSON response, shifts timestamps by `selected_range.start_frame / sample_rate`, and persists `transcript.txt` and `transcript.txt.json` without clobbering.
- `src/cli.rs`: `--whisper-python` (default `python3`) and `--whisper-model` (default `mlx-community/whisper-large-v3-turbo`) build `transcription::Options { python, model }`. `Command` has a single subcommand, `Anki`.
- `src/main.rs` dispatches `Command::Anki` with `if let`; otherwise `app::run`.
- `src/app.rs:15–23`: the transcription branch acquires the source, decodes the window, and calls `transcription::publish`, before any format is loaded.
- `tests/transcription.rs` (`#![cfg(unix)]`) drives the CLI with a fake Python interpreter (`tests/helpers/whisper_python.py`): language-name round trip for 9 languages, timeline mapping, 5 helper failure modes, and no-clobber.
- Python references outside transcription: `skills/.../scripts/compare_audio.py` and its uses in `evidence.md` and `boundaries.md:105+` (the optional Qwen comparison, untouched per U4). `SKILL.md:94` tells the skill to store "Python and model paths". `preparation.md:3,17,29,34` uses `$whisper_python`. `boundaries.md:5` and `design/transcription-and-metadata.md` describe the "Rust/Python transcription bridge".

### Baseline (2026-10-26, `main` at e3a6e04 plus untracked files)

- `cargo fmt --check` → exit 0.
- `cargo test` → 68 passed (1 + 2 + 57 + 4 + 4), 0 failed.
- `cargo clippy --all-targets -- -D warnings` → **exit 101**: `src/anki.rs:325` `clippy::collapsible_if` (fixed in Phase 1 per U12).

### External facts established (docs, source, probes)

Probe crate: whisper-rs 0.16.0 (`metal`) and hf-hub 1.0.0 (`blocking`), built and run on this Mac (Apple M3 Pro).

- **hf-hub 1.0.0 (used in Phase 1, replaced in Phase 2; source read 2026-10-26 after the Windows CI failure):**
  - Cache folder: `HF_HUB_CACHE` → `HUGGINGFACE_HUB_CACHE` → `HF_HOME` → `XDG_CACHE_HOME` → `$HOME/.cache/huggingface/hub`; with no `HOME` it uses `/tmp` (`constants.rs:27–56`). Windows has no `HOME`, so CI cached the model at `/tmp/.cache/huggingface\hub\…`.
  - A run waiting for another run's download gives up after a fixed, private 10 s (`constants.rs:22`, `cache/storage.rs:12–29`). After getting the lock it does not recheck the blob, so it downloads again (`repository/download.rs:522–529, 549–578`).
  - With revision `main`, every call makes a network request even when cached (`download.rs:408–458`). On Windows, every call that reaches the network deletes the snapshot file and copies the blob again (`cache/storage.rs:274–306`), and the model is stored twice (blob + copy).
  - Interrupted downloads restart from zero (`File::create` on `.incomplete`, `download.rs:976–1018`). A transient network error falls back to the cache (`download.rs:419–428`).
- **Hub cache specification** ([Hub Local Cache](https://huggingface.co/docs/hub/local-cache), "a reference for reimplementing the cache system in any language"):
  - Location: `HF_HUB_CACHE` (priority), else `$HF_HOME/hub`, else `~/.cache/huggingface/hub`. The huggingface_hub docs add: "If `HF_HOME` is not set, the default home will be `"$XDG_CACHE_HOME/huggingface"`".
  - Layout: `<cache>/models--<org>--<repo>/{blobs/<etag>, refs/<revision> (commit hash), snapshots/<commit>/<file> -> ../../blobs/<etag>}`. Locks: `<cache>/.locks/<repo_folder>/<blob_hash>.lock`.
  - File resolution: read `refs/<revision>`, then `snapshots/<commit>/<file>` exists → cached, no network needed.
  - Windows: "On Windows systems where symlinks are not available … actual file copies are placed directly in `snapshots/` instead of symlinks. The `blobs/` directory … [is] not used in this mode."
- **Hub download protocol (probe 2026-10-26, `curl -sI …/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin`):** `HTTP/2 302`, `x-repo-commit: 5359861c739e955e79d9a303bcbc70fb988958b1`, `x-linked-etag: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21"`, `x-linked-size: 77691713`, absolute `location` on `us.aws.cdn.hf.co`. A plain `curl -fL` GET produced 77,691,713 bytes with SHA-256 `be07e048…1e21`, the `x-linked-etag`. The etag of a stored large file is its SHA-256, so every download can be verified against it.
- **`std::fs::File::lock` (stable 1.89; toolchain 1.97.1):** "Blocks until the lock can be acquired … No other file handle to this file, in this or any other process, may acquire another lock … The lock will be released when this file … is closed". `flock(LOCK_EX)` on Unix, `LockFileEx(LOCKFILE_EXCLUSIVE_LOCK)` on Windows. The OS releases it when the process exits, including on a crash.
- **`std::env::home_dir`:** Windows: "Returns the value of the 'USERPROFILE' environment variable if it is set … Otherwise, `GetUserProfileDirectory`"; Unix: `HOME`, else `getpwuid_r`.
- **reqwest 0.13.5 blocking:** default features = rustls with aws-lc-rs (the TLS stack already traced in setup-dependencies.md B6). `ClientBuilder::timeout` "Default is 30 seconds … Pass `None` to disable timeout"; it wraps the request and each body `read` (`blocking/response.rs:440`). Redirects: "Default will follow redirects up to a maximum of 10"; `redirect::Policy::none()` disables them. The system proxy reads `HTTPS_PROXY` (`hyper-util` `matcher.rs:233`).
- **whisper-rs 0.16.0 API:** `WhisperContext::new_with_params(path, WhisperContextParameters::default())`, `ctx.create_state()`, `FullParams::new(SamplingStrategy::Greedy { best_of: 1 })` with `set_language`, `set_translate`, `set_temperature`, `set_temperature_inc`, `set_no_context`, `set_token_timestamps`, `set_print_*`; `state.full(params, &[f32])` (16 kHz mono); `state.as_iter()` → `WhisperSegment` (`start_timestamp`/`end_timestamp` in centiseconds, `no_speech_probability`, `to_str_lossy`, `n_tokens`, `get_token`); `WhisperToken::token_data()` → `{ id, p, t0, t1, … }` (centiseconds), `to_bytes()`; `state.full_lang_id_from_state()`; `get_lang_id(code) -> Option<i32>`; `ctx.token_eot()`, `ctx.token_lang(id)`, `ctx.token_translate()`; `WHISPER_CPP_VERSION`; `install_logging_hooks()`.
- **Probe token output** (tiny, Arabic clip): text tokens have `id < token_eot()` (50257); `[_BEG_]` = 50364 and `[_TT_n]` ≥ 50364 are special. Example segment `200..500`: tokens `" ل" 201–225 p.213`, `"اب" 225–263`, `"ت"`, `"ح"`, `"ب"`, `" ب"`, `"لا"`, … `[_TT_250]`. `state.full_lang_id_from_state()` = 13 (`ar`).
- **Languages:** whisper.cpp `g_lang` has 100 entries and includes `yue` (99), `sw`, `fa`, `hi`; it lacks `zu`, `xh`, `st` (the same support as the current README table). `get_lang_id("zu")` returns -1 → `None`.
- **Language vs loaded model:** `whisper_token_lang = token_sot + 1 + lang_id` (`whisper.cpp:4234`); language tokens sit below `token_translate`. Tiny has `n_langs = 99` (probe log), so `token_lang(99) = token_translate`. Rule: a loaded model supports `id` iff `ctx.token_lang(id) < ctx.token_translate()`.
- **mlx-whisper word rule being replaced** (`tokenizer.py:279–320`, `timing.py:173–181`): split tokens at points where the decoded bytes are complete Unicode; for `zh, ja, th, lo, my, yue` each such piece is a word; otherwise a piece starting with a space (or the first piece) starts a new word and other pieces join the current word. Word probability = mean of token probabilities.
- **Build:**
  - The probe built with Metal using cmake (Homebrew) and Xcode Command Line Tools clang; no `.cargo/config` rustflags were needed.
  - whisper-rs BUILDING.md: Linux needs cmake + a C/C++ toolchain (plus clang for bindgen); Windows MSVC needs Visual Studio C++, CMake, and LLVM/`LIBCLANG_PATH` for bindgen.
  - `WHISPER_DONT_GENERATE_BINDINGS` (`build.rs:119`) copies the bundled bindings instead of running bindgen. Without libclang, bindgen panics (`bindgen-0.72.1/lib.rs:616`).
  - Probe built with `WHISPER_DONT_GENERATE_BINDINGS=1` and Metal, and ran correctly.
  - With `[env] WHISPER_DONT_GENERATE_BINDINGS = "1"` in the crate's `.cargo/config.toml`, a later `cargo build --manifest-path` from `/` did not rebuild (`Finished … in 0.31s`); whisper-rs-sys has no `rerun-if-env-changed` for that variable.
- **Debug builds:** cmake-rs maps opt-level 0 to `CMAKE_BUILD_TYPE=Debug` (`cmake-0.1.58/src/lib.rs:146–176`). The skill and README use `cargo run` (dev profile), so whisper.cpp would be an unoptimized build. `[profile.dev.package.whisper-rs-sys] opt-level = 3` gives `RelWithDebInfo` (`build.rs:270`).
- **Logging:** whisper.cpp writes ~60 diagnostic lines to stderr per run (probe `run.err`). `install_logging_hooks()` without the `log_backend`/`tracing_backend` features routes them to a no-op (`whisper_logging_hook.rs`); failures still return `WhisperError`.
- **Setup sources:**
  - yt-dlp: the official PyInstaller standalone `yt-dlp_linux` bundles EJS (yt-dlp EJS wiki) and resolves (HTTP 200 at `releases/latest/download/yt-dlp_linux`); it needs no Python.
  - Windows: winget `yt-dlp.yt-dlp` x64/arm64 is the official `yt-dlp.exe` and depends on `DenoLand.Deno` and `yt-dlp.FFmpeg`, which provides `ffmpeg`/`ffprobe`. winget has `Kitware.CMake`.
  - Rust: rustup-init.exe (`https://win.rustup.rs/{x86_64|aarch64}`) offers to install the Visual Studio C++ prerequisites. Unix: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
  - Deno: `curl -fsSL https://deno.land/install.sh | sh` (needs `unzip`).
  - Homebrew: the `yt-dlp` formula depends on `deno` **and `python@3.14`** (`brew deps yt-dlp`), so it is excluded. The complete, current per-platform trace is [setup-dependencies.md](setup-dependencies.md); it supersedes the setup bullets in this list.

## 3. Data model and behavior

### Model identity

- `WhisperModel` = a file name in the Hugging Face repository `ggerganov/whisper.cpp` (e.g. `ggml-large-v3-turbo.bin`, `ggml-tiny.bin`). Default: `ggml-large-v3-turbo.bin` (the same model family as the current default).
- One resolver, `fetch_model(name) -> PathBuf` (`src/model_cache.rs` from Phase 2; hf-hub in Phase 1), returns the cached path, or downloads into the HF cache per "Model cache by construction" in Phase 2. It has two callers:
  - the new `download-model [MODEL]` subcommand, which prints `Model cached: <path>`;
  - `--transcribe --whisper-model MODEL`.
- No local-directory or Python options remain (U11).

### Transcription pipeline (`--transcribe`)

1. Fail if `OUTPUT` or `OUTPUT.json` exists (unchanged; nothing is fetched or replaced).
2. Language: `get_lang_id(code)` is `None` → error "Whisper does not support language code {code}".
3. `fetch_model(name)`, load the `WhisperContext` (logging hooks installed). If `token_lang(id) >= token_translate()` → error "{model} does not support language {code}".
4. Audio: ffmpeg converts the selected window's f32le PCM (source rate and channels) to 16 kHz mono f32le, writing to a temp file (as today, to avoid pipe deadlock), then reads it into `Vec<f32>`.
5. Run `full` with: language `code`, translate `false`, temperature `0.0`, temperature_inc `0.0` (no fallback), no_context `true`, token timestamps `true`, Greedy `best_of: 1`, all `print_*` false, no initial prompt.
6. Check `full_lang_id_from_state() == id`, else "whisper.cpp returned unexpected language".
7. Collect segments. Text pieces are tokens with `id < token_eot()`, as `Piece { bytes, start_cs, end_cs, probability }`. `words(&pieces, code)` applies the mlx-whisper rule above, with word `start` = first piece `t0`/100, `end` = last piece `t1`/100, and `probability` = mean.
8. `map_timestamps` (unchanged pure function): finite values, `0 ≤ start ≤ end`, probability in `[0, 1]`, plus offset.
9. Write the document; persistence is unchanged (no-clobber; text removed if the sidecar fails).

### Saved document shape

The top-level shape is unchanged (preparation.md:38 fields retained):

```text
Document { source, backend, settings, timestamp_reference: "original_source_seconds",
           source_offset_seconds, text, language, segments }
Backend  { name: "whisper.cpp", version: WHISPER_CPP_VERSION, model: <file name> }
Settings { language, task: "transcribe", word_timestamps: true, temperature: 0.0,
           condition_on_previous_text: false, initial_prompt: null }
Segment  { id, start, end, text, words: [Word], no_speech_prob }
Word     { word, start, end, probability }
```

**Information change:** `Segment.temperature`, `avg_logprob`, and `compression_ratio` are no longer written; whisper.cpp does not report them. No reader in the app or skill consumes them (preparation.md:38 lists the consumed fields). Existing saved transcripts are untouched.

### Worked cases

| # | Start | Operation | Result | Governing |
|---|---|---|---|---|
| 1 | Fresh Mac with Homebrew | `./setup.sh` | brew installs whichever of `rust cmake ffmpeg deno` are missing; the official `yt-dlp_macos` goes to `/usr/local/bin/yt-dlp` (sudo) if `yt-dlp` is absent; build (bindings generated with the Command Line Tools' libclang); `cargo run -- download-model` puts `ggml-large-v3-turbo.bin` in `~/.cache/huggingface/hub`; skill linked in `~/.agents/skills` and `~/.claude/skills`; no Python installed | U1.2, U2, U3, U5, U6, U9 |
| 2 | After case 1, offline | `cargo run -- rec.mp3 --language ja --start 0.3 --end 20.3 --transcribe --output t.txt` | model resolved from cache; `t.txt` + `t.txt.json`, times in original-source seconds (offset 0.3) | U8, U9, README:111 |
| 3 | Any | `--language zulu --transcribe` | error "Whisper does not support language code zu"; no outputs; no download | README:111 |
| 4 | Tiny model | `--language cantonese --whisper-model ggml-tiny.bin --transcribe` | error "ggml-tiny.bin does not support language yue"; no outputs | README:111 |
| 5 | Any | `--whisper-model ggml-nope.bin --transcribe` | hf-hub not-found error; no outputs | README:111 |
| 6 | `t.txt` exists | `--transcribe --output t.txt` | "already exists"; nothing replaced or downloaded | README:55 |
| 7 | Arabic pieces `" ل" "اب" "ت" "ح" "ب" " ب" "لا"` | `words(…, "ar")` | `" لابتحب"` (start = t0 of `" ل"`, end = t1 of `"ب"`, p = mean of 5), `" بلا"` | mlx rule (preserved behavior) |
| 8 | Japanese pieces where one character's UTF-8 bytes span two tokens | `words(…, "ja")` | those two tokens form one word; every complete character group is its own word | mlx rule |
| 9 | Fresh Debian/Ubuntu on x86_64 | `./setup.sh` | `sudo apt-get install -y --no-install-recommends ca-certificates curl unzip build-essential cmake libclang-dev ffmpeg`; rustup (`-y`) if no `cargo`; Deno installer (`-y`) if no `deno`; `yt-dlp_linux` → `/usr/local/bin/yt-dlp` (sudo) if absent; build; download; skills; no Python installed; transcription runs on CPU | U2, U6, U8 |
| 10 | Windows on x64 | `setup.ps1` | winget `yt-dlp.yt-dlp` (brings `DenoLand.Deno`, `yt-dlp.FFmpeg`) if no `yt-dlp`; `Kitware.CMake` if no `cmake`; `LLVM.LLVM` if `%ProgramFiles%\LLVM\bin\libclang.dll` is absent; rustup-init if no `cargo` (offers Visual Studio C++); build; download; skill junctions in both dirs | U2, U3, U6 |
| 11 | Linux not on x86_64, or without `apt-get` | `./setup.sh` | exits 1 before installing anything: "setup.sh supports macOS with Homebrew and Debian/Ubuntu on x86_64" | targets answer |
| 12 | Windows not on x64 | `setup.ps1` | throws before installing anything: "setup.ps1 supports Windows on x64" | targets answer |

## 4. Phases

Work happens on branch `native-transcription-setup` (the repo is on `main`). Each phase commits only its intended files; the untracked media, `listening/`, and design drafts stay uncommitted unless listed.

### Phase 1 — Native transcription

**Outcome:** `--transcribe` runs whisper.cpp inside the binary; `download-model` pre-fetches models into the HF cache; no Python anywhere in the transcription path; docs that describe transcription match.

**Change inventory**

| File | Change | Author |
|---|---|---|
| `Cargo.toml` | add `whisper-rs = "0.16"`, `hf-hub = { version = "1", features = ["blocking"] }`; `[target.'cfg(target_os = "macos")'.dependencies] whisper-rs = { version = "0.16", features = ["metal"] }`; `[profile.dev.package.whisper-rs-sys] opt-level = 3` | me |
| `.cargo/config.toml` (create) | `[env] WHISPER_DONT_GENERATE_BINDINGS = "1"` | me |
| `src/transcription.rs` | replace the helper bridge with the pipeline in §3. `Options { model }` (python removed). New: `DEFAULT_MODEL`, `fetch_model`, model/language checks, 16 kHz conversion (replaces `selected_wav`), whisper run, piece collection, `words`, segment building. Keep `map_timestamps`; split `publish` under 20 lines. Remove `HELPER`, `HelperRequest`, `HelperResponse`, `validate_response`. `#[cfg(test)] mod tests` covers the pure functions | me |
| `src/cli.rs` | remove `whisper_python`; `--whisper-model` doc and default `DEFAULT_MODEL`; add `Command::DownloadModel { model }` with default `DEFAULT_MODEL` | me |
| `src/main.rs` | `match` on `Command::Anki` / `Command::DownloadModel` | me |
| `src/anki.rs:325` | collapse nested `if` (U12) | me |
| `scripts/mlx_whisper_helper.py` | delete (U11) | me |
| `tests/helpers/whisper_python.py` | delete (U11) | me |
| `tests/transcription.rs` | rewrite against real `ggml-tiny.bin`; drop `#![cfg(unix)]` | me |
| `README.md` "Transcribe source audio" + languages column header | describe whisper.cpp, `download-model`, `--whisper-model` file names; remove Python setup (full rewrite in Phase 2) | me |
| `skills/.../SKILL.md:94` | "Python and model paths" → "model" | me |
| `skills/.../references/preparation.md:3,17,29,34` | remove `whisper_python`; `whisper_model` is a model file name; replace the MLX/Python paragraph with whisper.cpp + `download-model` | me |
| `skills/.../references/boundaries.md:5` | "Rust/Python transcription bridge" → native transcription | me |
| `design/transcription-and-metadata.md` | replace helper request/response excerpts with the native pipeline and the new Segment/Backend shapes | me |
| `listening/settings.md` (untracked user data) | U13: remove the Whisper Python and MLX Whisper model entries and `mlx-whisper 0.4.3` from Runtime; keep the `.venv-whisper` Python as the audio-comparison Python (compare_audio.py uses it) | me (done before Phase 1) |

**Verification**

- Unit tests (`src/transcription.rs`):
  - `words`: case 7; case 8 using real multibyte splits (e.g. `"お"` = `E3 81 8A` split across two pieces); a space-language first piece without a leading space; probability mean.
  - `map_timestamps`: offset applied; negative or reversed rejected; probability > 1 rejected.
  - Centisecond conversion.
- Integration tests (`tests/transcription.rs`, real `ggml-tiny.bin`, synthetic 16 kHz ramp, window `0.10001–0.40001`):
  - (a) Success: both outputs exist; `timestamp_reference`; `source_offset_seconds == 1601/16000`; every segment and word time within `[offset, offset + 0.3]`; `backend.name == "whisper.cpp"`; `backend.model == "ggml-tiny.bin"`; `settings.task == "transcribe"`; `selected_range` 1601..6401.
  - (b) Name round trip for fr, it, de, zh, ko, fa, hi, sw (code in `source.language.code`, `settings.language`, `language`).
  - (c) Cantonese → error naming `yue`, no outputs (case 4).
  - (d) `zulu` → error, no outputs (case 3).
  - (e) `ggml-nope.bin` → error, no outputs (case 5).
  - (f) No-clobber (case 6, unchanged test).
  - (g) `download-model ggml-tiny.bin` prints an existing path under the HF cache.
- Commands: `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- Artifact runs:
  - `cargo run -- download-model` (large-v3-turbo).
  - Real transcription of `qahwa_1.mp3 --language ar --start 01:00 --end 01:15`, with output inspected.
  - The same command with `HF_ENDPOINT=http://127.0.0.1:9` (offline, case 2).
  - A YouTube `--url … --transcribe --start --end` run.
  - `grep -rn "whisper-python\|whisper_python\|mlx_whisper\|mlx-whisper"` over `src tests scripts skills README.md design` → only historical research text.

**Boundary state:** the CLI transcribes natively; all tests and lint are green; no Python reference remains in the app, tests, skill instructions, or README transcription section. The README's other sections and the setup scripts are unchanged; the Phase 2 inputs are ready.

### Phase 2 — Setup scripts and README rewrite

**Basis:** every dependency below is traced in [setup-dependencies.md](setup-dependencies.md) (sections B, C, E). Targets: macOS with Homebrew, Debian/Ubuntu on x86_64, Windows on x64.

**Outcome:** on each target, one entry point installs only what is missing, builds, runs the app's `download-model`, and installs the skill in both locations. Nothing installs Python. GitHub Actions runs setup and the tests on all three targets. The README explains setup and usage.

**Data model / behavior changes**

- whisper.cpp bindings are generated by bindgen on every platform; `.cargo/config.toml` goes. Why: the bundled bindings fail to compile for Windows (O2: `208_usize - 216_usize`, Size of `_IO_FILE`), and Cargo `[env]` cannot be set per target. Generated bindings build on macOS (O8) and Debian/Ubuntu with `libclang-dev` (C.4). Consequence: libclang is a build requirement everywhere (Command Line Tools / `libclang-dev` / `LLVM.LLVM`).
- yt-dlp is always the official standalone executable, never a Python package (setup-dependencies.md "Why Python is excluded").

**Model cache by construction** (user: "MAKE IT WORK BY _CONSTRUCTION_. NO MORE ARBITRARY TIMEOUT MANAGEMENT OR HOPING AND PRAYING."). hf-hub is removed; `src/model_cache.rs` implements the Hub cache specification (§2) for the one repository the app uses.

`fetch_model(name)`:
1. **Cache folder:** `HF_HUB_CACHE`, else `$HF_HOME/hub`, else `$XDG_CACHE_HOME/huggingface/hub`, else `home_dir()/.cache/huggingface/hub` (Windows: `%USERPROFILE%\.cache\huggingface\hub`). A pure function of the variables and the home folder.
2. **Cached:** if `refs/main` names commit C and `snapshots/C/<name>` exists, return it. No network request, nothing copied.
3. **Describe:** `HEAD https://huggingface.co/ggerganov/whisper.cpp/resolve/main/<name>` without following redirects. A status that is neither success nor redirection is an error (e.g. 404 for `ggml-nope.bin`). `x-repo-commit` gives the commit; `x-linked-etag` (quotes removed) gives the etag, which is the file's SHA-256. A missing header is an error.
4. **Lock:** `File::lock` on `.locks/models--ggerganov--whisper.cpp/<etag>.lock`. It blocks until free, with no time limit. hf-hub and huggingface_hub use the same lock file for the same blob, so their downloads are waited for too.
5. **Place, under the lock,** only if `snapshots/<commit>/<name>` is still absent:
   - Unix: if `blobs/<etag>` is absent, download into it; then symlink the snapshot to `../../blobs/<etag>`.
   - Windows: download straight into the snapshot path. This is the spec's no-symlink mode: one copy, `blobs/` unused.
   - Download: GET the resolve URL (redirects followed, `timeout(None)`), streamed into `blobs/<etag>.incomplete` (`File::create` truncates any earlier partial) while computing SHA-256. If the hash ≠ etag, it is an error and nothing is published. Otherwise rename to the target.
6. Write `refs/main` = commit and return the snapshot path. The lock is released when the handle drops.

Properties this construction guarantees:
- A snapshot path exists only for a complete, SHA-256-verified file: it is created only by renaming a verified file, or by symlinking to such a blob. `refs/main` is written after it, so the lock-free check in step 2 never sees a partial file.
- At most one process downloads a blob at a time. A waiter wakes holding the lock, finds the snapshot in step 5, and does not download again.
- Waiting has no time limit and cannot be stranded: the OS releases the lock when the holder exits or crashes. The next holder truncates the `.incomplete` file and downloads again.
- The app sets no timeout anywhere (`timeout(None)`). A stalled connection keeps the run waiting until the user interrupts it, and interrupting is safe by the points above.
- A cached model needs no network. Consequence: a later re-upload of the same file name on the Hub is not fetched while the cached copy exists. Phase 1 (hf-hub) checked the Hub on every run.

Callers: `transcription::load_model` and `main.rs` `DownloadModel` call `model_cache::fetch_model`; error text stays `cannot fetch Whisper model {name}: …`.

**Change inventory**

| File | Change | Author |
|---|---|---|
| `.cargo/config.toml` | **deleted** (user: "Ok. Delete."); deletion staged, to be committed with Phase 2 | me |
| `setup.sh` (rewrite of the uncommitted draft) | `cd "$repo"`; PATH gets `~/.cargo/bin` and `~/.deno/bin` for this run. **Darwin:** Homebrew required; `brew install` whichever of `cargo=rust cmake=cmake ffmpeg=ffmpeg deno=deno` are missing; if `yt-dlp` is absent: `sudo mkdir -p /usr/local/bin`, `sudo curl -fL …/releases/latest/download/yt-dlp_macos -o /usr/local/bin/yt-dlp`, `sudo chmod 755`. **Linux:** requires `apt-get` and `uname -m` = `x86_64`, else exit 1 with the supported-systems message; `sudo apt-get update`; `sudo apt-get install -y --no-install-recommends ca-certificates curl unzip build-essential cmake libclang-dev ffmpeg`; `curl … sh.rustup.rs \| sh -s -- -y` if no `cargo`; `curl … deno.land/install.sh \| sh -s -- -y` if no `deno` (`-y` adds Deno to the shell configs; without a terminal the installer skips that step); `yt-dlp_linux` → `/usr/local/bin/yt-dlp` as above if absent. **Other systems:** exit 1 with the message. **All:** `cargo build`; `cargo run --quiet -- download-model`; `scripts/install-skill.sh "$HOME/.agents/skills"` and `"$HOME/.claude/skills"`. No `python` anywhere | me |
| `setup.ps1` (rewrite of the uncommitted draft) | Throws unless `$env:PROCESSOR_ARCHITECTURE -eq 'AMD64'`. `winget install --source winget --exact --id <id> --accept-source-agreements --accept-package-agreements` (throws on a nonzero exit) for `yt-dlp.yt-dlp` if no `yt-dlp`, `Kitware.CMake` if no `cmake`, `LLVM.LLVM` if `$env:ProgramFiles\LLVM\bin\libclang.dll` is absent; rustup-init (`win.rustup.rs/x86_64`) if no `cargo`; refresh PATH (Machine + User + `~\.cargo\bin`); `cargo build`; `cargo run --quiet -- download-model`; skill junctions in both dirs (refuse to replace a different target) | me |
| `README.md` | Keep the rewritten README (sections: what it does; Setup; Command-line usage; Format files; Guided listening skill; Languages; Validation status; Design and research; Development checks). Rewrite the Setup section from table E: supported systems; what each script installs, including libclang and `/usr/local/bin/yt-dlp`; manual equivalents; unsupported systems stop with a message. Remove "`setup.ps1` has not yet been run on a Windows machine" once CI runs it. Model cache text (Setup step 2 and "Transcribe source audio"): cache folder including Windows `%USERPROFILE%\.cache\huggingface\hub`; a run needing a model another run is downloading waits for it and uses it; a cached model is used with no network | me |
| `.github/workflows/setup.yml` (create) | Trigger: `push` to `native-transcription-setup`, and `workflow_dispatch`. **macos** (`macos-15`): `./setup.sh`, `cargo test`, `cargo run -- tests/fixtures/jfk.wav --language en --transcribe --output jfk.txt`. **linux** (`ubuntu-24.04`, matrix `ubuntu:24.04`, `debian:12`): in a fresh container, as a non-root user with passwordless sudo, `./setup.sh`, `command -v python3` must fail, then `cargo test` and the jfk transcription. **windows** (`windows-2025`): Microsoft's documented winget bootstrap (`Install-Module Microsoft.WinGet.Client -Force`, `Repair-WinGetPackageManager -AllUsers`; the runner image lists no winget), `./setup.ps1`, `cargo test`, the jfk transcription. Each job also runs `yt-dlp --version` and `deno --version` | me |
| `Cargo.toml` | remove `hf-hub`; add `reqwest = { version = "0.13", features = ["blocking"] }` (default features: rustls + aws-lc-rs, as before) | me |
| `src/model_cache.rs` (create) | "Model cache by construction": `fetch_model`, `cache_dir_from` (pure), cached lookup, HEAD description, lock, place (Unix/Windows), verified download, ref write; unit test for `cache_dir_from` order | me |
| `src/transcription.rs` | remove `fetch_model` and the `hf_hub` import; `load_model` calls `model_cache::fetch_model` | me |
| `src/main.rs` | `mod model_cache`; `DownloadModel` calls `model_cache::fetch_model` | me |
| `tests/transcription.rs` | add `concurrent_downloads_into_a_fresh_cache_download_once`: 4 simultaneous `download-model ggml-tiny.bin` with `HF_HUB_CACHE` = a new temp folder; all exit 0 and print the same path under it; that file's SHA-256 = `be07e048…1e21`; the folder holds exactly one 77,691,713-byte file (symlinks not followed) and no `.incomplete`. Add `cached_model_is_used_without_network`: after `download-model ggml-tiny.bin`, with `HTTPS_PROXY`/`HTTP_PROXY` = `http://127.0.0.1:9`, `download-model ggml-nope.bin` fails with "cannot fetch Whisper model" (the network is blocked) and the jfk transcription with `ggml-tiny.bin` succeeds | me |
| `design/transcription-and-metadata.md` (untracked user draft) | replace the hf-hub `fetch_model` excerpt with `model_cache::fetch_model` | me |
| `docs/current-plans/setup-dependencies.md` | B6 source: reqwest (model download); R4: the model-cache construction, Windows `%USERPROFILE%` folder and one copy | me |
| `docs/current-plans/NATIVE_TRANSCRIPTION_IMPLEMENTATION_STATUS.md` | Phase 2 results | me |

**Dependencies:** Phase 1 (`download-model`, native transcription). Pushing the branch to `origin` (github.com/nomicflux/language-jutsu) is authorized by the GitHub Actions answer: "Publishes the branch and adds a CI file."

**Verification**

- Local (Mac):
  - `sh -n setup.sh` and `shellcheck setup.sh` (expected notices: SC1007 `CDPATH=`, SC2086 package-list word split).
  - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, all with generated bindings after the deletion.
  - `./setup.sh`, with the output recorded. `brew list` before and after shows no new Python formula. The model is under `~/.cache/huggingface/hub/models--ggerganov--whisper.cpp`; both skill links exist.
  - Every README CLI example, run as written in a scratch copy.
- CI: all four jobs (macos, linux × 2, windows) green, with each run URL and its key log lines (setup completion, `Model cached:`, the test result lines, the transcription output, the `python3` check on Linux) recorded in the status file. On Windows, `Model cached:` must be under `C:\Users\runneradmin\.cache\huggingface\hub`.
- `cargo tree -i aws-lc-sys` still resolves (TLS stack unchanged); `cargo tree -i hf-hub` finds nothing.
- YouTube URL runs are verified locally on the Mac (done earlier and repeated in the README examples). YouTube URL runs from GitHub's datacenter runners are not part of the CI gate, because YouTube may demand sign-in from those addresses; CI verifies that `yt-dlp` and `deno` are installed and on PATH.

**Boundary state:** a new user on macOS (Homebrew), Debian/Ubuntu x86_64 or Windows x64 runs one script and has a working CLI, model and skill, with no Python installed by setup. Other systems get a message naming the supported ones. CI proves each target on every push.

## 5. Requirement → behavior → phase → check

| Req | Behavior | Phase | Check that fails if absent |
|---|---|---|---|
| U1.1 README rewrite: setup, CLI (files, URLs, formats), skill options | README sections in Phase 2 | 2 | README examples executed; section review against SKILL.md list |
| U1.2 setup script, single entry, downloads backends, not stored on GitHub | `setup.sh` / `setup.ps1`; whisper.cpp built from crates; model in HF cache (outside repo) | 2 (1 supplies `download-model`) | Mac setup run; CI jobs; `git status` shows no model files |
| U2 Homebrew for Mac; Windows and Linux options | brew (Mac), apt (Debian/Ubuntu x86_64), winget (Windows x64); scope per "a user could use this from a non-mac system" | 2 | Mac run; CI macos, linux (ubuntu:24.04, debian:12), windows jobs |
| U3 Both | install-skill to `~/.agents/skills` and `~/.claude/skills`; ps1 junctions to both | 2 | links present after Mac run; CI setup output "Installed:" twice |
| U4 Whisper only | setup downloads only the Whisper model | 2 | script contents; setup output |
| U5 HF cache | hf-hub default cache | 1, 2 | test (g); `Model cached:` path under the HF cache in every run |
| U6 no Python install | standalone yt-dlp everywhere; `--no-install-recommends`; no Homebrew `yt-dlp` | 2 | Linux CI `command -v python3` fails after setup; Mac `brew list` diff; script contents |
| "THE SETUP SCRIPT SHOULD BE CALLING THE DOWNLOAD-MODEL IN THE APP" | `cargo run --quiet -- download-model` in both scripts | 2 | `Model cached:` in every setup run |
| "TRACE DEPENDENCIES" / "WE FIGURE THIS OUT IN PLANNING" | setup-dependencies.md (every build/runtime dependency per target, with evidence) | plan | table complete before Phase 2 starts |
| GitHub Actions answer | `.github/workflows/setup.yml`, branch pushed | 2 | four green jobs |
| U8 native code | whisper-rs in-process | 1 | integration tests (a)–(e) |
| U9 HF cache + subcommand | `download-model`; `--transcribe` uses the cache by default | 1 | test (g); offline artifact run |
| U10 tests 1 & 2 | unit and real-tiny integration tests | 1 | `cargo test` |
| U11 deletions | four removals | 1 | files absent; `--whisper-python` rejected by clap |
| U12 lint fix | anki.rs collapse | 1 | clippy exit 0 |
| README:111 original-source seconds, transcribe only, unsupported fails | §3 steps 2, 3, 5, 8 | 1 | tests (a), (c), (d) |
| README:55 no replace | step 1 | 1 | test (f) |
| preparation.md:38 fields | document shape | 1 | test (a) field assertions |

### Implementation choices (agent-authored, not requirements)

- **Word timing uses whisper.cpp token `t0`/`t1`, not DTW.** research/technical-options.md:25 records that whisper.cpp word timestamps are "explicitly experimental"; mlx-whisper's were also estimates. This is the same class of output, stated in the README as estimates.
- **whisper.cpp diagnostics are silenced**; failures still surface as errors.
- **The model download shows no progress bar.** Setup prints a line before the 1.5 GiB download.

## 6. Validation gate

Run `/verify-plan` on this file. Phase 1 starts only after you review that report and authorize it.
