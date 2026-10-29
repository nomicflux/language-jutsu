#!/usr/bin/env python3
"""Build release payloads on macOS, then assemble the unsigned universal installer."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
WORK = ROOT / "target/macos"
DEPS = json.loads((HERE / "dependencies.json").read_text())
MINIMUM = DEPS["macos_minimum"]


def run(*args, cwd=None, env=None):
    subprocess.run([str(x) for x in args], cwd=cwd, env=env, check=True)


def download(spec):
    path = WORK / "downloads" / spec["url"].rsplit("/", 1)[1]
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists():
        partial = path.with_suffix(path.suffix + ".partial")
        run("curl", "--fail", "--location", "--retry", "3", spec["url"], "-o", partial)
        partial.rename(path)
    if hashlib.sha256(path.read_bytes()).hexdigest() != spec["sha256"]:
        raise RuntimeError(f"Checksum mismatch: {path}. Remove it before retrying.")
    return path


def unpack_source(archive, directory):
    directory.mkdir(parents=True, exist_ok=True)
    # Only checksum-pinned upstream source archives are extracted here.
    with tarfile.open(archive) as source:
        name = source.getnames()[0].split("/")[0]
        if not (directory / name).exists():
            source.extractall(directory)
    return directory / name


def cargo_notices(destination):
    registry = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")) / "registry/src"
    for package in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]:
        if not package.get("source", "").startswith("registry+"):
            continue
        name = f"{package['name']}-{package['version']}"
        candidates = list(registry.glob("*/" + name))
        if not candidates:  # Platform-specific crates not compiled on this builder.
            continue
        source = candidates[0]
        target = destination / name
        target.mkdir(parents=True, exist_ok=True)
        manifest = tomllib.loads((source / "Cargo.toml").read_text())["package"]
        (target / "SOURCE.txt").write_text(f"https://crates.io/crates/{package['name']}/{package['version']}\nLicense: {manifest.get('license', 'see included files')}\n")
        for item in source.iterdir():
            if item.name.lower().startswith(("license", "copying", "notice")):
                if item.is_dir():
                    shutil.copytree(item, target / item.name, dirs_exist_ok=True)
                else:
                    shutil.copy2(item, target / item.name)


def audit_runtime(runtime, architecture):
    """Reject developer-machine library paths and missing architecture slices."""
    for path in runtime.rglob("*"):
        if not path.is_file() or path.is_symlink():
            continue
        kind = subprocess.check_output(["file", "-b", str(path)], text=True)
        if "Mach-O" not in kind:
            continue
        run("lipo", path, "-verify_arch", architecture)
        build_info = subprocess.check_output(["xcrun", "vtool", "-show-build", str(path)], text=True)
        minimums = re.findall(r"\bminos ([0-9.]+)", build_info)
        minimums += re.findall(r"cmd LC_VERSION_MIN_MACOSX\s+cmdsize \d+\s+version ([0-9.]+)", build_info)
        for minimum in minimums:
            if tuple(map(int, (minimum + ".0.0").split(".")[:3])) > tuple(map(int, (MINIMUM + ".0.0").split(".")[:3])):
                raise RuntimeError(f"{path} needs macOS {minimum}, above the installer minimum {MINIMUM}")
        deps = subprocess.check_output(["otool", "-L", str(path)], text=True)
        for line in deps.splitlines():
            if not line.startswith("\t"):
                continue
            library = line.strip().split(" (", 1)[0]
            if library.startswith(("/usr/lib/", "/System/Library/", "@loader_path/", "@executable_path/", "@rpath/")):
                continue
            raise RuntimeError(f"Non-portable dependency in {path}: {library}")


def payload(architecture):
    target = {"arm64": "aarch64-apple-darwin", "x86_64": "x86_64-apple-darwin"}[architecture]
    build = WORK / "build" / architecture
    prefix = build / "prefix"
    runtime = WORK / "payloads" / architecture / "runtime"
    runtime.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, MACOSX_DEPLOYMENT_TARGET=MINIMUM,
               CMAKE_OSX_DEPLOYMENT_TARGET=MINIMUM, CMAKE_OSX_ARCHITECTURES=architecture,
               GGML_NATIVE="OFF", GGML_METAL_EMBED_LIBRARY="ON",
               CFLAGS=f"-O2 -arch {architecture} -mmacosx-version-min={MINIMUM}",
               LDFLAGS=f"-arch {architecture} -mmacosx-version-min={MINIMUM}")
    # GGML's non-native default enables newer x86 instructions; use a portable baseline.
    for flag in ("AVX", "AVX2", "FMA", "F16C", "BMI2"):
        env["GGML_" + flag] = "OFF"
    run("cargo", "build", "--locked", "--release", "--target", target, cwd=ROOT, env=env)
    shutil.copy2(ROOT / "target" / target / "release/language-jutsu", runtime)

    lame_archive = download(DEPS["lame"])
    lame = unpack_source(lame_archive, build)
    if not (prefix / "lib/libmp3lame.a").exists():
        run(lame / "configure", f"--prefix={prefix}", "--disable-shared", "--enable-static",
            "--disable-frontend", f"--host={'aarch64' if architecture == 'arm64' else architecture}-apple-darwin", cwd=lame, env=env)
        run("make", "-j", os.cpu_count() or 2, cwd=lame, env=env)
        run("make", "install", cwd=lame, env=env)
    ffmpeg_archive = download(DEPS["ffmpeg"])
    ffmpeg = unpack_source(ffmpeg_archive, build)
    configure = [str(ffmpeg / "configure"), "--disable-autodetect", "--disable-doc",
                 "--disable-ffplay", "--disable-network", "--disable-shared", "--enable-static",
                 "--enable-libmp3lame", "--disable-x86asm", f"--arch={architecture}",
                 "--target-os=darwin", "--enable-cross-compile",
                 f"--extra-cflags={env['CFLAGS']} -I{prefix}/include",
                 f"--extra-ldflags={env['LDFLAGS']} -L{prefix}/lib"]
    if not (ffmpeg / "ffmpeg").exists():
        run(*configure, cwd=ffmpeg, env=env)
        run("make", "-j", os.cpu_count() or 2, cwd=ffmpeg, env=env)
    for name in ("ffmpeg", "ffprobe"):
        shutil.copy2(ffmpeg / name, runtime / name)
    # ditto preserves the symlinks and executable modes in the official standalone bundle.
    run("ditto", "-x", "-k", download(DEPS["yt-dlp"]), runtime)
    (runtime / "yt-dlp_macos").replace(runtime / "yt-dlp")
    run("ditto", "-x", "-k", download(DEPS["deno"][architecture]), runtime)
    shutil.copytree(ROOT / "formats", runtime / "formats", dirs_exist_ok=True)
    audit_runtime(runtime, architecture)
    licenses = runtime.parent / "licenses"
    licenses.mkdir(exist_ok=True)
    shutil.copy2(ffmpeg / "COPYING.LGPLv2.1", licenses / "FFmpeg-LGPL.txt")
    shutil.copy2(lame / "COPYING", licenses / "LAME-LGPL.txt")
    shutil.copy2(download(DEPS["deno"]["license"]), licenses / "Deno-MIT.txt")
    yt_source = download(DEPS["yt-dlp"]["source"])
    with tarfile.open(yt_source) as archive:
        (licenses / "yt-dlp-UNLICENSE.txt").write_bytes(archive.extractfile("yt-dlp/LICENSE").read())
    shutil.copy2(runtime / "_internal/THIRD_PARTY_LICENSES.txt", licenses / "yt-dlp-THIRD-PARTY.txt")
    shutil.copy2(HERE / "dependencies.json", licenses)
    cargo_notices(licenses / "rust-dependencies")
    # Ship corresponding FFmpeg/LAME sources and build instructions alongside the DMG.
    sources = WORK / "sources"
    sources.mkdir(exist_ok=True)
    for archive in (lame_archive, ffmpeg_archive, yt_source):
        shutil.copy2(archive, sources)
    shutil.copy2(HERE / "build.py", sources)
    shutil.copy2(HERE / "dependencies.json", sources)
    (licenses / "BUILD.txt").write_text("FFmpeg built without GPL/nonfree options.\n" + "\n".join(configure) + "\n")


def assemble():
    for architecture in ("arm64", "x86_64"):
        audit_runtime(WORK / "payloads" / architecture / "runtime", architecture)
        if not (WORK / "payloads" / architecture / "runtime/language-jutsu").exists():
            raise RuntimeError(f"Build the {architecture} payload first")
    app = WORK / "dmg/Install Language Jutsu.app"
    if app.exists():
        shutil.rmtree(app)
    contents = app / "Contents"
    macos = contents / "MacOS"
    resources = contents / "Resources"
    macos.mkdir(parents=True)
    resources.mkdir()
    for architecture in ("arm64", "x86_64"):
        run(os.environ.get("SWIFT_EXEC", "swiftc"), "-swift-version", "5", "-O", "-target", f"{architecture}-apple-macosx{MINIMUM}",
            "-module-cache-path", WORK / "swift-cache", HERE / "InstallerCore.swift", HERE / "InstallerApp.swift", HERE / "InstallerMain.swift",
            "-o", WORK / f"Installer-{architecture}")
    run("lipo", "-create", WORK / "Installer-arm64", WORK / "Installer-x86_64", "-output", macos / "Installer")
    shutil.copytree(WORK / "payloads", resources / "payloads", symlinks=True)
    common = resources / "common"
    (common / "bin").mkdir(parents=True)
    shutil.copy2(HERE / "launcher.sh", common / "bin/language-jutsu")
    (common / "bin/language-jutsu").chmod(0o755)
    shutil.copytree(ROOT / "skills/language-jutsu-listening", common / "skills/language-jutsu-listening",
                    ignore=shutil.ignore_patterns(".DS_Store", "__pycache__"))
    # The skill's optional boundary reference links to these source explanations.
    shutil.copytree(ROOT / "src", common / "src")
    boundaries = common / "skills/language-jutsu-listening/references/boundaries.md"
    boundaries.write_text(boundaries.read_text().replace(
        "The Language Jutsu CLI, native whisper.cpp transcription, source retention, and rendered metadata shapes are traced with code excerpts in [transcription-and-metadata.md](../../../design/transcription-and-metadata.md).",
        "The installed package includes the current [CLI](../../../src/cli.rs), [transcription](../../../src/transcription.rs), [source retention](../../../src/source.rs), and [render metadata](../../../src/metadata.rs) implementation for source-level reference."))
    shutil.copy2(HERE / "INSTALLED.md", common / "README.md")
    shutil.copy2(ROOT / "tests/fixtures/jfk.wav", resources / "sample.wav")
    shutil.copy2(ROOT / "tests/fixtures/SOURCES.md", resources / "SAMPLE-SOURCE.md")
    version = next(line.split('"')[1] for line in (ROOT / "Cargo.toml").read_text().splitlines() if line.startswith("version ="))
    (common / ".language-jutsu-installation").write_text("language-jutsu-macos-v1\n")
    (common / "VERSION").write_text(version + "\n")
    with (contents / "Info.plist").open("wb") as output:
        plistlib.dump({"CFBundleExecutable": "Installer", "CFBundleIdentifier": "org.language-jutsu.installer",
                      "CFBundleName": "Install Language Jutsu", "CFBundlePackageType": "APPL",
                      "CFBundleShortVersionString": version, "CFBundleVersion": version,
                      "LSMinimumSystemVersion": MINIMUM, "NSHighResolutionCapable": True,
                      "NSDocumentsFolderUsageDescription": "Language Jutsu saves your lessons in Documents."}, output)
    # Local ad-hoc signatures are required by Apple Silicon; they do not establish publisher trust.
    run("codesign", "--force", "--sign", "-", app)
    shutil.copy2(HERE / "OPENING.txt", WORK / "dmg/READ ME FIRST.txt")
    image = WORK / f"Language-Jutsu-{version}-unsigned.dmg"
    run("hdiutil", "create", "-ov", "-format", "UDZO", "-volname", "Language Jutsu", "-srcfolder", WORK / "dmg", image)
    print(image)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["payload", "assemble"])
    parser.add_argument("--arch", choices=["arm64", "x86_64"])
    args = parser.parse_args()
    if args.action == "payload":
        if not args.arch:
            parser.error("payload requires --arch")
        payload(args.arch)
    else:
        assemble()
