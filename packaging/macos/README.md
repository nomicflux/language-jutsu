# Mac installer

Produces `target/macos/Language-Jutsu-<version>-unsigned.dmg` with a universal native setup app and two prebuilt runtime payloads. Distribution is **unsigned and not notarized**, as requested. The local ad-hoc app signature does not establish publisher trust. [OPENING.txt](OPENING.txt) explains the normal macOS Open Anyway flow; setup does not disable Gatekeeper or remove quarantine attributes.

## Build

Release builders need macOS, Command Line Tools, Rust with the relevant Apple target, CMake, and Python 3.11+. End users need none of these. Dependency versions and SHA-256 hashes are pinned in [dependencies.json](dependencies.json). FFmpeg and LAME are compiled into portable executables; yt-dlp and Deno come from official release artifacts. whisper.cpp's Metal library is embedded by whisper-rs-sys.

On the corresponding native builder:

```sh
python3 packaging/macos/build.py payload --arch arm64
# On the Intel builder:
python3 packaging/macos/build.py payload --arch x86_64
```

Copy both architecture directories into `target/macos/payloads/` on the assembly Mac, preserving symlinks and executable modes. Then:

```sh
python3 packaging/macos/build.py assemble
tar -czf target/macos/dependency-sources.tar.gz -C target/macos sources
```

Cross-compiling both payloads on one Mac is also supported when both Rust targets are installed. The [workflow](../../.github/workflows/macos-installer.yml) uses separate native builders, tests their payloads, then produces one downloadable artifact with the DMG, dependency sources, and opening instructions. It does not publish a GitHub release. Distribute the source archive alongside the DMG and retain the included license notices, including yt-dlp's bundled third-party notices. yt-dlp's standalone distribution contains copyleft components; its own source license alone does not describe the combined binary.

The binary deployment target and installer minimum are macOS 13.0. Packaging rejects binaries with a higher declared deployment minimum, missing CPU slices, or absolute dependencies outside macOS system libraries. Native validation on the oldest supported OS remains necessary before claiming that OS has been tested. `SWIFT_EXEC` can select a compatible Swift compiler without changing the system's toolchain selection.

## Build and publish a new version

1. Update the package version in `Cargo.toml` and the matching `language-jutsu` entry in `Cargo.lock`, then commit and push the version and source changes.
2. Open [Actions → Mac installer](https://github.com/nomicflux/language-jutsu/actions/workflows/macos-installer.yml), choose **Run workflow**, and select the branch to build. The workflow must be present on the default branch to run manually; see [GitHub's instructions](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow).
3. After the run succeeds, download **Language-Jutsu-Mac-installer-unsigned** from its **Artifacts** section. This contains the versioned DMG, `dependency-sources.tar.gz`, and `OPENING.txt`. [Artifact downloads](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts) require signing into GitHub and expire according to repository retention settings.
4. To host downloads for end users, [create a GitHub release](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository) tagged `v<version>` at the commit that was built. Attach the DMG, dependency sources and opening instructions, then publish it. Downloads will appear on the project's [Releases page](https://github.com/nomicflux/language-jutsu/releases). Update the root README's availability statement when the first release is published.

The workflow builds artifacts only; it does not publish a release automatically. The local build commands above produce the same files for manual release upload.

## Installation and updates

The app installs for the current user in `~/Library/Application Support/Language Jutsu`, with no privileged helper or shell-profile changes. It prepares a temporary sibling directory, runs the existing `download-model`, renders and transcribes a short bundled public-domain sample, then publishes the runtime and skill links. A failed preparation leaves the previous runtime in place. A failed publication rolls back the files and links it replaced.

Existing different skills require confirmation and are moved to `.language-jutsu-listening-previous-<id>` beside their original location. Reinstalling this installer's skill requires no replacement confirmation. Lessons are outside the install directory, in `~/Documents/Language Jutsu/listening`. The model keeps the application's existing Hugging Face cache behavior. Reopening the installer retries setup; models already cached are reused.

## Installer-to-CLI boundary

The installer constructs the model request in [InstallerCore.swift:137](InstallerCore.swift#L137):

```swift
try run(cli, ["download-model"], in: stage, log: log, failure: "The speech model could not be downloaded or opened.")
```

The [process wrapper](InstallerCore.swift#L86) passes an executable URL and an argument array directly, with no shell interpolation or JSON protocol:

```swift
process.executableURL = executable
process.arguments = arguments
process.currentDirectoryURL = directory
process.environment = environment
process.standardInput = FileHandle.nullDevice
process.standardOutput = output
process.standardError = output
```

[launcher.sh:3](launcher.sh#L3) sets a private child-process PATH and preserves argument boundaries and the caller's working directory:

```sh
export PATH="$installation/runtime:/usr/bin:/bin:/usr/sbin:/sbin"
exec "$installation/runtime/language-jutsu" "$@"
```

The [CLI parser](../../src/cli.rs#L136) accepts the optional model filename:

```rust
DownloadModel {
    /// Model file in the ggerganov/whisper.cpp Hugging Face repository.
    #[arg(default_value = crate::transcription::DEFAULT_MODEL)]
    model: String,
},
```

`DEFAULT_MODEL` is `ggml-large-v3-turbo.bin` in [transcription.rs](../../src/transcription.rs#L19). [main.rs](../../src/main.rs#L32) parses with `cli::Args::parse()` and constructs the response:

```rust
Some(cli::Command::DownloadModel { model }) => {
    let path = model_cache::fetch_model(&model)?;
    println!("Model cached: {}", path.display());
    Ok(())
}
```

[main.rs](../../src/main.rs#L22) maps `Ok(())` to `ExitCode::SUCCESS`; errors print `error: {error}` to stderr and return `ExitCode::FAILURE`. The [installer](InstallerCore.swift#L105) consumes that process status, not a parsed stdout message:

```swift
guard process.terminationReason == .exit && process.terminationStatus == 0 else {
    throw InstallError.message(failure + " See Details below, then click Retry.")
}
```

Output and diagnostics go to a temporary setup log, displayed when setup fails. Download progress is an indeterminate waiting indicator; no percentage protocol is assumed. The cache request remains implemented in [model_cache.rs](../../src/model_cache.rs#L12), including the pinned model revision and environment-variable precedence.

## Verification

```sh
swiftc -swift-version 5 packaging/macos/InstallerCore.swift packaging/macos/InstallerTests.swift -o target/macos/installer-tests
target/macos/installer-tests
python3 packaging/macos/verify.py target/macos/payloads/arm64
python3 packaging/macos/verify.py target/macos/payloads/x86_64
```

Installer tests use isolated temporary home folders and cover install/update, preserved lessons, model failure and retry, conflict confirmation and backup, cancellation, publication rollback, installed-skill invocation through symlinks, and paths containing spaces. Payload verification invokes the copied launcher with only packaged and macOS tools on PATH, then checks real rendering, decoding, default-model fetching/caching, transcription, and Anki packaging. Add `--youtube-url <public-video-url>` to exercise actual YouTube acquisition and rendering; this check needs external network access.

For UI review, compile `InstallerCore.swift`, `InstallerApp.swift`, and `InstallerUITestMain.swift` into a temporary copy of the app. Place an isolated temporary home path in its `Contents/Resources/test-home.txt`. This test entry point uses the production window and installer; it is never shipped. Verify the initial screen, an error and Retry, conflict confirmation, completion prompts, and opening the lessons folder. Eject the original DMG and remove the test installer copy before invoking the installed skill runner again.

The other two installer requests are constructed at [InstallerCore.swift:142](InstallerCore.swift#L142):

```swift
try run(cli, [sample, "--language", "en", "--start", "0", "--end", "4",
              "--output", check.appendingPathComponent("study.mp3").path], in: check, log: log, failure: "The audio processing check failed.")
try run(cli, [sample, "--language", "en", "--start", "0", "--end", "4", "--transcribe",
              "--output", check.appendingPathComponent("transcript.txt").path], in: check, log: log, failure: "The speech recognition check failed.")
```

The positional `input: Option<PathBuf>` and `--start`/`--end` options are parsed by [cli.rs:24](../../src/cli.rs#L24); required `--language` and `--output` are declared at [cli.rs:47](../../src/cli.rs#L47) and [cli.rs:63](../../src/cli.rs#L63). The transcription switch is declared at [cli.rs:82](../../src/cli.rs#L82):

```rust
/// Transcribe the selected source audio with Whisper, without a playback format.
#[arg(long)]
transcribe: bool,
```

[cli.rs:248](../../src/cli.rs#L248) selects `Mode::Transcribe` when that flag is present and `Mode::Render` when no processing-mode flag is present. The file-producing calls are [app.rs:20](../../src/app.rs#L20) and [app.rs:98](../../src/app.rs#L98):

```rust
transcription::publish(&decoded, &language, &request.whisper, &request.output)?;
// Rendering branch:
render::publish(&decoded, &analysis.units, &sequence, &request.output)?;
```

Both operations return through the same success/error exit-code mapping shown above. Their stdout is diagnostic text; the installer does not parse their audio, transcript, or JSON sidecar files.
