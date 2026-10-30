#!/usr/bin/env python3
"""Rebuild, package and publish a Mac release, including its version and Git updates."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / "target/macos"
REPOSITORY = "nomicflux/language-jutsu"
LATEST = f"https://github.com/{REPOSITORY}/releases/latest"
# These are the app, its bundled skill, and the files used to build and install it.
# Personal lessons, recordings, and untracked research are not release inputs.
INPUTS = ["Cargo.toml", "Cargo.lock", "README.md", "src", "formats",
          "skills/language-jutsu-listening", "packaging/macos", "scripts",
          "tests", "setup.sh", "setup.ps1", ".github"]


def run(*args, **kwargs):
    return subprocess.run([str(arg) for arg in args], cwd=ROOT, text=True, check=True, **kwargs)


def read(*args):
    return run(*args, stdout=subprocess.PIPE).stdout.strip()


def version_tuple(value):
    if not re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", value):
        raise ValueError(f"Use a version such as 0.1.1, not {value!r}.")
    return tuple(map(int, value.split(".")))


def releases():
    return {release["tagName"]: release for release in json.loads(read(
        "gh", "release", "list", "--repo", REPOSITORY, "--limit", "1000",
        "--json", "tagName,isDraft"))}


def source_snapshot():
    paths = read("git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *INPUTS)
    snapshot = {}
    for name in filter(None, paths.split("\0")):
        path = ROOT / name
        if path.is_symlink():
            snapshot[name] = ("link", os.readlink(path))
        elif path.is_file():
            snapshot[name] = (path.stat().st_mode, hashlib.sha256(path.read_bytes()).hexdigest())
        else:
            snapshot[name] = None
    return snapshot


def update_version(version):
    replacements = {
        "Cargo.toml": r'(?ms)(^\[package\]\n.*?^version\s*=\s*")[^"]+(")',
        "Cargo.lock": r'(?m)(^name = "language-jutsu"\nversion = ")[^"]+(")',
    }
    updates = {}
    for filename, pattern in replacements.items():
        path = ROOT / filename
        text, count = re.subn(pattern, lambda match: match[1] + version + match[2], path.read_text())
        if count != 1:
            raise ValueError(f"Could not identify the package version in {filename}.")
        updates[path] = text
    readme = ROOT / "README.md"
    text, count = re.subn(r"(\[Download Language Jutsu for Mac\]\()[^)]+(\))",
                          lambda match: match[1] + LATEST + match[2], readme.read_text())
    if count != 1:
        raise ValueError("Could not identify the Mac download link in README.md.")
    updates[readme] = text
    for path, text in updates.items():
        path.write_text(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__, prog="scripts/package-macos.sh")
    parser.add_argument("version", nargs="?", help="Release version, e.g. 0.1.1. Omit to choose the next patch release (or retry an unfinished release).")
    args = parser.parse_args()
    if platform.system() != "Darwin":
        raise ValueError("Run this script on a Mac.")
    for tool in ("git", "gh", "cargo", "rustup", "cmake", os.environ.get("SWIFT_EXEC", "swiftc")):
        if not shutil.which(tool):
            raise ValueError(f"Missing release tool: {tool}.")
    auth = subprocess.run(["gh", "auth", "status"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if auth.returncode:
        run("gh", "auth", "login")
    branch = read("git", "symbolic-ref", "--short", "HEAD")
    remote_tags = {}
    for line in read("git", "ls-remote", "--tags", "origin").splitlines():
        commit, ref = line.split("\t")
        tag = ref.removeprefix("refs/tags/").removesuffix("^{}")
        remote_tags[tag] = commit
    known_releases = releases()
    current = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    current_parts = version_tuple(current)
    published = [version_tuple(tag[1:]) for tag in remote_tags
                 if re.fullmatch(r"v\d+\.\d+\.\d+", tag) and not known_releases.get(tag, {}).get("isDraft")]
    latest = max(published, default=(-1, -1, -1))
    if args.version:
        version = args.version
    elif current_parts > latest:
        version = current
    else:
        major, minor, patch = max(current_parts, latest)
        version = f"{major}.{minor}.{patch + 1}"
    if version_tuple(version) <= latest:
        raise ValueError(f"Version {version} has already been released or is older than an existing release.")
    tag = "v" + version
    if tag in known_releases and not known_releases[tag]["isDraft"]:
        raise ValueError(f"{tag} is already published; it will not be replaced.")

    print(f"Releasing {tag}: updating versions, rebuilding and publishing.", flush=True)
    update_version(version)
    snapshot = source_snapshot()
    run("rustup", "target", "add", "aarch64-apple-darwin", "x86_64-apple-darwin")
    for name in ("payloads", "dmg", "sources"):
        shutil.rmtree(WORK / name, ignore_errors=True)
    for arch in ("arm64", "x86_64"):
        run(sys.executable, "packaging/macos/build.py", "payload", "--arch", arch)
    run(sys.executable, "packaging/macos/build.py", "assemble")
    dmg = WORK / f"Language-Jutsu-{version}-unsigned.dmg"
    run("hdiutil", "verify", dmg)
    sources = WORK / "dependency-sources.tar.gz"
    with tarfile.open(sources, "w:gz") as archive:
        archive.add(WORK / "sources", arcname="sources")
    opening = WORK / "OPENING.txt"
    shutil.copy2(ROOT / "packaging/macos/OPENING.txt", opening)
    if source_snapshot() != snapshot:
        raise ValueError("Release inputs changed during the build. Run the script again to build those changes.")

    run("git", "add", "-A", "--", *INPUTS)
    changed = subprocess.run(["git", "diff", "--cached", "--quiet", "--", *INPUTS], cwd=ROOT)
    if changed.returncode == 1:
        run("git", "commit", "--only", "-m", f"Release {tag}", "--", *INPUTS)
    else:
        changed.check_returncode()
    commit = read("git", "rev-parse", "HEAD")
    if tag in remote_tags and remote_tags[tag] != commit:
        raise ValueError(f"{tag} already points to different source. Run with a new version.")
    run("git", "push", "origin", f"HEAD:refs/heads/{branch}")
    existing = releases().get(tag)
    if existing and not existing["isDraft"]:
        raise ValueError(f"{tag} was published while building; it will not be replaced.")
    if existing:
        run("gh", "release", "edit", tag, "--repo", REPOSITORY, "--target", commit,
            "--title", f"Language Jutsu {version}", "--notes-file", opening)
    else:
        run("gh", "release", "create", tag, "--repo", REPOSITORY, "--target", commit,
            "--title", f"Language Jutsu {version}", "--notes-file", opening, "--draft")
    run("gh", "release", "upload", tag, dmg, sources, opening, "--repo", REPOSITORY, "--clobber")
    run("gh", "release", "edit", tag, "--repo", REPOSITORY, "--draft=false", "--latest")
    print(f"Published: https://github.com/{REPOSITORY}/releases/tag/{tag}\nInstaller: {dmg}")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(f"Release stopped: {error}")
