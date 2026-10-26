# Setup dependency table

Every build and runtime dependency of the app, setup and skill, traced per platform. A cell holds the source, what it pulls in, and the evidence (doc quote or command + output). **OPEN** cells are unresolved; **CAN'T VERIFY HERE** cells go to the user as a decision.

## Governing user statements (verbatim)

- U1: "Improve setup - this should work out of the box for a new user as much as possible (have a setup script for a single point of entry, along with downloads of the required transcription backends so that these do not have to be stored on Github)."
- U2: "Homebrew for Mac; we need options for Windows & Linux too."
- U3: "Both" (skill in `~/.claude/skills` and `~/.agents/skills`).
- U6: "If you are installing python, you done fucked up."
- U8: "Native code. A C/C++ engine is not worse than a Python runtime."
- U9: "HF cache + subcommand"
- "THE SETUP SCRIPT SHOULD BE CALLING THE DOWNLOAD-MODEL IN THE APP, SO WHAT THE FUCK IS GOING ON?"
- "REDO FROM SCRATCH. TRACE DEPENDENCIES. WE DO NOT FIGURE THIS OUT IN IMPLEMENTATION, WE FIGURE THIS OUT IN PLANNING, YOU FUCKING IMBECILE."

- Answer to "How should those platforms be verified?": "GitHub Actions (Recommended)" (option text: "Add a workflow file to the repo and push this branch to GitHub; GitHub's hosted Windows x64, Windows ARM, Ubuntu x64, Intel Mac and an Arch container run ./setup.sh / setup.ps1 and the tests. Publishes the branch and adds a CI file.")
- Answer to the Windows ARM question: "Where is this going to happen?"
- Answer to the Debian 12 ARM question: "You are targetted arbitrary builds. I just wanted a simple "a user could use this from a non-mac system""

**Targets (from the last two answers):** macOS with Homebrew; Windows on x64 with winget; Linux = Debian/Ubuntu on x86_64 with apt. Windows ARM, Linux arm64, Debian 12 ARM, Arch and Intel-specific Mac handling are not targeted. CI covers macOS, Windows x64 and Ubuntu x64.

**Why Python is excluded:** setup's Python environment broke on a fresh install. Homebrew's `python3` had moved to 3.15, and `pip install 'mlx-whisper==0.4.3'` failed: "No matching distribution found for torch". Rule applied here: no dependency may install Python as a package or resolve Python packages at setup time. A self-contained executable that installs no Python is not excluded.

## Environment limits (this machine)

- macOS 15 arm64 (M3 Pro) with Homebrew and Xcode Command Line Tools.
- Docker: linux/arm64 native; linux/amd64 emulated (pacman needs `--disable-sandbox` under emulation). **6.2 GB free** in the Docker VM; it fits one Rust build of this app (local `target/debug` is 3.2 GB) at a time. Other projects' images and volumes are not touched.
- Windows: none available. Every Windows cell is **CAN'T VERIFY HERE**.
- Intel Mac: none available.

## A. What the app needs

Traced from `Cargo.lock` / `cargo tree -e normal,build --target <t>` (identical native set for aarch64-apple-darwin, x86_64/aarch64-unknown-linux-gnu, x86_64/aarch64-pc-windows-msvc) and `src/`:

| ID | Dependency | Needed for | Requires |
|---|---|---|---|
| B1 | Rust toolchain (cargo, rustc) | build | — |
| B2 | C/C++ compiler and linker | whisper-rs-sys (C/C++), aws-lc-sys, bzip2-sys, libsqlite3-sys (bundled), all via `cc`/`cmake` crates | per platform below |
| B3 | CMake | whisper-rs-sys builds whisper.cpp with the `cmake` crate | — |
| B4 | whisper.cpp Rust bindings | whisper-rs-sys: bundled `src/bindings.rs` when `WHISPER_DONT_GENERATE_BINDINGS` is set (`.cargo/config.toml`), else bindgen (needs libclang) | per platform below |
| B5 | ggml CPU code generation | whisper.cpp CPU backend; `GGML_NATIVE` detection | per compiler below |
| B6 | TLS crypto (aws-lc-sys 0.46, `prebuilt-nasm`) | hf-hub → reqwest → rustls → aws-lc-rs (model download) | C compiler; Windows ARM64: clang-cl |
| B7 | Metal | `whisper-rs` `metal` feature on macOS | Xcode Command Line Tools |
| R1 | `ffmpeg`, `ffprobe` on PATH | decode, render, transcription resample (`src/media.rs`, `src/render.rs`, `src/transcription.rs`) | options used: `-ac -af -ar -c:a -channel_layout -dn -f -i -map -nostdin -of -q:a -select_streams -show_streams -sn -v -vn -xerror -y`, `libmp3lame` |
| R2 | `yt-dlp` on PATH | `--url` (`src/source.rs:236`) | yt-dlp-ejs + a JS runtime |
| R3 | JS runtime | yt-dlp's YouTube extraction | app passes `--js-runtimes node`; yt-dlp docs: "Only "deno" is enabled by default", `--js-runtimes` = "Additional JavaScript runtime to enable" → Deno ≥ 2.3.0 or Node ≥ 22.0.0 ([EJS wiki](https://github.com/yt-dlp/yt-dlp/wiki/EJS)) |
| R4 | Whisper model in HF cache | `download-model`, `--transcribe` (hf-hub 1.0 with explicit `cache_dir`, `ggerganov/whisper.cpp` pinned to commit `5359861c…`) | network only until cached; symlinks on Unix, blob + snapshot copy on Windows |
| R5 | Skill links | `~/.agents/skills`, `~/.claude/skills` | symlink (Unix), junction (Windows) |
| S1 | Setup-time fetch tools | rustup, Deno installer, yt-dlp download | `curl`, `unzip` (Deno installer: "either unzip or 7z is required") |

## B. Per-platform cells

Platforms: **Mac-arm** (macOS arm64), **Mac-x64** (Intel), **Deb** (Debian 12/13, Ubuntu 22.04/24.04; x86_64 and arm64), **Arch** (x86_64), **Win-x64**, **Win-arm**.

### B1 Rust

| Platform | Source | Evidence |
|---|---|---|
| Mac | Homebrew `rust` | `brew deps rust`: no python |
| Deb, Arch | rustup (`curl … sh.rustup.rs \| sh`) | rustup.sh read; interactive prompt needs a terminal |
| Win | rustup-init.exe (`win.rustup.rs/{x86_64,aarch64}`) | rustup docs: rustup-init "will offer to automatically install the prerequisites", which "installs Visual Studio Community edition". WinGet route from the same page: `winget install --id Microsoft.VisualStudio.Community --source winget --force --override "--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.VC.Tools.ARM64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --addProductLang En-us"`; "replace "Community" with "BuildTools" … if you already have a Visual Studio license" |

### B2 C/C++ compiler

| Platform | Source | Evidence |
|---|---|---|
| Mac | Xcode Command Line Tools (Homebrew requires them) | Phase 1 build on this Mac |
| Deb | `build-essential` | apt, `--no-install-recommends`: no python on all four releases (C.1) |
| Arch | out of scope (`base-devel` pulls python, C.2) | |
| Win-x64 | MSVC from the Visual Studio install that rustup-init offers (B1) | CI (`windows-2025` has Visual Studio Enterprise 2022) |
| Win-arm | MSVC ARM64 **and clang/clang-cl**: ggml `CMakeLists.txt:109` "MSVC is not supported for ARM, use clang"; aws-lc-sys `cc_builder.rs:1130` "Windows ARM64 (aarch64-pc-windows-msvc) requires clang-cl. Install the 'C++ Clang Compiler for Windows' component" | **No supported route found for ggml.** The `cmake` crate passes `-T <toolset>` only when the build script calls `generator_toolset(...)` (`cmake-0.1.58/src/lib.rs:231–236, 692–693`), and whisper-rs-sys 0.15.0 does not. For the Visual Studio generator on ARM64 it passes `-AARM64` with the default (MSVC) toolset (lines 657–672), which ggml rejects. CAN'T VERIFY HERE |

### B3 CMake

| Platform | Source | Evidence |
|---|---|---|
| Mac | Homebrew `cmake` | `brew deps cmake`: no python |
| Deb | apt `cmake` | C.1 |
| Arch | pacman `cmake` | 0 python targets |
| Win | winget `Kitware.CMake` | manifest 4.4.4: portable zip registers alias `cmake`; MSI passes `ADD_CMAKE_TO_PATH=System`; either way `cmake` is on PATH, where the `cmake` crate looks (`cmake-0.1.58/src/lib.rs:899–903`) |

### B4 Bindings

Bundled `bindings.rs` was generated on Linux glibc: it declares `_IO_FILE` with `_old_offset: __off_t` (`c_long`) and asserts `size_of::<_IO_FILE>() - 216usize`. `c_long` is 4 bytes on Windows, so the bundled file cannot compile there (static reading; confirming probe O2).

| Platform | Bundled bindings | bindgen (libclang) |
|---|---|---|
| Mac | builds (Phase 1); type-check `--target aarch64-apple-darwin` exit 0 | **builds** (O8): scratch copy without `.cargo/config.toml`, `cargo build` exit 0; `OUT_DIR/bindings.rs` has `pub struct __sFILE` (2), no `_IO_FILE`, no "Unable to generate". libclang from CLT `/Library/Developer/CommandLineTools/usr/lib/libclang.dylib` |
| Deb | not used | `libclang-dev`, no python with or without recommends (C.1); **builds** (C.4) |
| Arch | out of scope | |
| Win | **fails** (O2): `rustc --target x86_64-pc-windows-msvc --emit=metadata` on `include!(bindings.rs)` → exit 1, `attempt to compute 208_usize - 216_usize` (Size of `_IO_FILE`), `12_usize - 16_usize` (Size of `_G_fpos_t`, `_G_fpos64_t`) | libclang from LLVM; whisper-rs BUILDING.md: "Visual Studio C++, cmake, LLVM(clang)", `setx LIBCLANG_PATH "…\VC\Tools\Llvm\x64\bin"`; CAN'T VERIFY HERE |

**Consequence:** `.cargo/config.toml` `[env] WHISPER_DONT_GENERATE_BINDINGS` applies to every target, and it breaks the Windows build. Cargo `[env]` cannot be set per target. Generating bindings everywhere needs libclang on every platform; the Mac case is shown above.

### B5 ggml CPU build (Linux)

| Compiler | Result |
|---|---|
| GCC 12.2 arm64 (Debian 12) | **fails**: `vec.cpp` `inlining failed in call to 'always_inline' 'vaddq_f16' … target specific option mismatch`; flags `-march=armv8-a+crypto+crc+lse+rcpc+rdma+dotprod+fp16fml+sb+bf16+flagm+pauth+nodotprod+noi8mm+nosve`, `GGML_COMPILER_SUPPORTS_FP16_FORMAT_I3E - Failed`, `HAVE_FP16_VECTOR_ARITHMETIC - Success` (setup run log) |
| GCC 12.2 arm64 (Debian 12), `-DGGML_NATIVE=OFF` | builds (build exit 0); `HAVE_FP16_VECTOR_ARITHMETIC - Failed`, `HAVE_DOTPROD - Failed`, no arch flags → generic armv8-a code |
| GCC 11.4 arm64 (Ubuntu 22.04) | builds; flags `-mcpu=native`; fp16 vector and dotprod not detected |
| GCC 13.3 arm64 (Ubuntu 24.04) | builds; `-march=armv8-a+…+fp16fml+…+dotprod+i8mm+nosve`; fp16 vector and dotprod Success |
| GCC 14.2 arm64 (Debian 13) | builds; `-mcpu=generic-armv8-a+…+fp16fml+…+dotprod+i8mm+nosve+nosme`; fp16 vector and dotprod Success |
| GCC 12.2 x86_64 (Debian 12, emulated) | `c++: internal compiler error: Segmentation fault signal terminated program cc1plus` under amd64 emulation: CAN'T VERIFY HERE |

Cause of the GCC 12 arm64 failure: ggml's "old GCC" branch (`ggml-cpu/CMakeLists.txt:131–133`) takes the `-march=armv8-a+…` string GCC 12 prints for this CPU (Apple M3 under Docker). The feature check says fp16 vector arithmetic is defined, but GCC 12 refuses to inline `arm_neon.h`'s fp16 intrinsics into code built with that arch string. The arch string depends on the CPU, so other ARM CPUs running GCC 12 CAN'T BE VERIFIED HERE. `GGML_NATIVE` cannot be set per target in Cargo `[env]`, and turning it off globally removes native tuning on every platform (x86 would lose AVX).

O1 probe: CMake-only build of the vendored whisper.cpp (`whisper-rs-sys-0.15.0/whisper.cpp`, the defines whisper-rs-sys passes: `BUILD_SHARED_LIBS=OFF`, `WHISPER_BUILD_TESTS=OFF`, `WHISPER_BUILD_EXAMPLES=OFF`, `GGML_METAL=OFF`, `CMAKE_BUILD_TYPE=RelWithDebInfo`), packages `build-essential cmake` with `--no-install-recommends`, one container at a time. Expected before running: Debian 12 arm64 default fails as in setup; Debian 12 arm64 `GGML_NATIVE=OFF` builds (no native flags → `HAVE_FP16_VECTOR_ARITHMETIC` false); others unknown.

### R1 FFmpeg

| Platform | Source | Python? |
|---|---|---|
| Mac | Homebrew `ffmpeg` | `brew deps ffmpeg`: no python |
| Deb | apt `ffmpeg` with `--no-install-recommends` (includes ffprobe) | with recommends: 7 python packages on Debian 12, 9 on Ubuntu 24.04; without: none on all four (C.1) |
| Arch | pacman `ffmpeg` **excluded**: depends on `vapoursynth` → `python`. Static build from [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds) `latest` release: `ffmpeg-n9.0-latest-linux64-gpl-9.0.tar.xz` / `…-linuxarm64-…` (release branch; `bin/ffmpeg`, `bin/ffprobe`) | static binary; no package dependencies |
| Win | winget `yt-dlp.FFmpeg` (dependency of `yt-dlp.yt-dlp`): portable zip from `yt-dlp/FFmpeg-Builds`, commands `ffmpeg`, `ffplay`, `ffprobe`, x86/x64/arm64 | gpl builds enable LAME (`scripts.d/50-libmp3lame.sh`: `--enable-libmp3lame`); VapourSynth is headers only, loaded at run time (`50-vapoursynth.sh`) |

yt-dlp's own fork publishes only `master` Linux builds (`ffmpeg-master-latest-linux{64,arm64}-gpl.tar.xz`). Its upstream BtbN also publishes release branches (`n8.1`, `n9.0`), hence BtbN for Linux.

### R2 yt-dlp

Official standalone executables bundle the interpreter and yt-dlp-ejs: "The standalone release binaries are built with the Python interpreter and the packages marked with * included"; EJS wiki: "`yt-dlp-ejs` is bundled with these executables".

| Platform | Source |
|---|---|
| Mac (arm, x64) | `yt-dlp_macos` ("Universal MacOS (10.15+) standalone executable"). Homebrew `yt-dlp` **excluded**: `brew deps yt-dlp` lists `python@3.14` |
| Linux x86_64 | `yt-dlp_linux` ("glibc 2.17+ standalone x86_64") |
| Linux arm64 | `yt-dlp_linux_aarch64` ("glibc 2.17+ standalone aarch64") |
| Win-x64 | winget `yt-dlp.yt-dlp` 2026.10.21, x64 installer = `…/releases/download/2026.10.21/yt-dlp.exe` (portable), dependencies `DenoLand.Deno`, `yt-dlp.FFmpeg` |
| Win-arm | out of scope |

### R3 JS runtime (Deno ≥ 2.3.0)

| Platform | Source | Evidence |
|---|---|---|
| Mac | Homebrew `deno` | `brew deps deno`: no python; `brew info deno`: stable 2.9.8 |
| Deb, Arch | Deno installer (`deno.land/install.sh`, needs `unzip`); targets x86_64 and aarch64 Linux | install.sh read; Debian 12 arm64 run installed deno 2.9.8 |
| Win | winget `DenoLand.Deno` 2.9.8 (x64, arm64 zips) | manifest |

### R4 Model cache

| Platform | Evidence |
|---|---|
| Mac | `cargo test --test transcription` with an empty `HF_HUB_CACHE`: 8 passed; hf-hub wrote `blobs/be07e048…`, `snapshots/5359861c…/ggml-tiny.bin`, `.locks/…/be07e048….lock`, `.no_exist/5359861c…/ggml-nope.bin`. Cached model used with `HTTPS_PROXY` blocked (test) |
| Linux | same hf-hub Unix path as Mac; verified in CI (`Model cached:` line, transcription tests) |
| Win | Folder: the app passes `cache_dir` = `std::env::home_dir()` (`USERPROFILE`) → `%USERPROFILE%\.cache\huggingface\hub`. hf-hub 1.0.0 alone used `HOME`, else `/tmp`: CI run 40376744569 cached at `/tmp/.cache/huggingface\hub`. hf-hub copies the blob into the snapshot (`cache/storage.rs:274–306`): two copies. The pinned commit makes a cached run return before any request or copy (`download.rs:408–413`). Verified in CI: `Model cached:` path and transcription tests |

### R5 Skill links

| Platform | Evidence |
|---|---|
| Mac | `install-skill.sh` → "Installed" / "Already installed" (setup runs) |
| Linux | same script; verified in CI (two "Installed:" lines) |
| Win | junction; CAN'T VERIFY HERE |

## C. Evidence

### C.1 apt: python packages pulled (`apt-get install -s`)

```
debian:bookworm  curl ffmpeg build-essential cmake unzip | recommends: 7 | no-recommends: (none)
                 libclang-dev | recommends: 0 | no-recommends: (none)
                 clang        | recommends: 10 | no-recommends: (none)
debian:trixie    base set 0 / none; libclang-dev 0 / none; clang 9 / none
ubuntu:22.04     base set 0 / none; libclang-dev 0 / none; clang 10 / none
ubuntu:24.04     base set 9 / none; libclang-dev 0 / none; clang 10 / none
```

### C.2 pacman (archlinux:latest, amd64, `--disable-sandbox -Sp`): count of `python` in targets

```
ffmpeg: 1   base-devel: 1   cmake: 0   unzip: 0   curl: 0   clang: 1   llvm-libs: 0
gcc: 0      make: 0         clang-libs: 0          vapoursynth: 1
ffmpeg Depends On: … vapoursynth …
```

### C.3 Homebrew (`brew deps <formula> | grep -i python`)

```
rust: none   cmake: none   ffmpeg: none   deno: none   llvm: none   yt-dlp: python@3.14
```

### C.4 Linux bindgen build (Ubuntu 24.04 arm64; same packages as x86_64)

`apt-get install --no-install-recommends ca-certificates curl build-essential cmake libclang-dev`, rustup, repo copy **without** `.cargo/config.toml`, `cargo build`:

```
cargo build exit 0
"Unable to generate" count: 0
OUT_DIR/bindings.rs "pub struct _IO_FILE" count: 1   (generated against glibc)
```

### C.5 Windows libclang discovery (clang-sys 1.9.1)

`build/common.rs:186` searches `C:\Program Files*\LLVM\lib`; `common.rs:249–256` also searches the sibling `bin` on Windows ("`libclang.dll` is usually found in the LLVM `bin` directory"). The winget `LLVM.LLVM` MSI installs to `C:\Program Files\LLVM`, so no `LIBCLANG_PATH` is needed.

## E. Settled per target

| Dependency | macOS (Homebrew) | Debian/Ubuntu x86_64 (apt) | Windows x64 (winget) |
|---|---|---|---|
| Rust | `brew install rust` | rustup | rustup-init (offers Visual Studio Community C++) |
| C/C++ compiler | Xcode CLT (Homebrew prerequisite) | `build-essential` | MSVC via rustup-init's Visual Studio install |
| CMake | `brew install cmake` | `cmake` | `Kitware.CMake` |
| libclang (bindings generated on every platform) | Xcode CLT (C.4/O8: builds) | `libclang-dev` (C.4: builds) | `LLVM.LLVM` (C.5); build CAN'T VERIFY HERE → CI |
| FFmpeg/ffprobe | `brew install ffmpeg` | `ffmpeg` | `yt-dlp.FFmpeg` (dependency of `yt-dlp.yt-dlp`) |
| yt-dlp | `yt-dlp_macos` → `/usr/local/bin/yt-dlp` (sudo) | `yt-dlp_linux` → `/usr/local/bin/yt-dlp` (sudo) | `yt-dlp.yt-dlp` (`yt-dlp.exe`) |
| JS runtime | `brew install deno` (2.9.8) | Deno installer (needs `unzip`) | `DenoLand.Deno` (dependency of `yt-dlp.yt-dlp`, 2.9.8) |
| Fetch tools | curl (macOS) | `curl`, `ca-certificates`, `unzip` | PowerShell `Invoke-WebRequest` |
| apt flag | — | `--no-install-recommends` (C.1: no Python) | — |
| Python installed | none (C.3, yt-dlp standalone) | none (C.1) | none (standalone `yt-dlp.exe`) |

Other systems (Linux not on x86_64, Linux without apt, Windows not on x64) stop with a message naming the supported systems.

## D. Open cells

| ID | Status |
|---|---|
| O1 | Settled (B5). Linux arm64 is out of scope. |
| O2 | Settled (B4): bundled bindings fail on Windows. |
| O3 | Settled for x64 (B1, E). Windows ARM is out of scope. |
| O4 | Settled (R4). |
| O5 | Settled (R1, R2, R3). |
| O6 | Arch is out of scope. |
| O7 | Intel Mac is out of scope. |
| O8 | Settled (B4): the Mac builds with generated bindings. |
| CI | The Windows x64 build/run, the Debian/Ubuntu x86_64 fresh-system run, and the full test suite on Linux and Windows are verified on GitHub Actions (user's choice). Hosted macOS and Windows runner images already contain Rust, LLVM and CMake, so their "already installed" path is what runs there. The fresh Debian/Ubuntu path runs in `ubuntu:24.04` and `debian:12` containers on the x64 runner. |
