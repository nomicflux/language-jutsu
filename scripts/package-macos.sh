#!/bin/bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/package-macos.sh [--publish]

Build an unsigned Mac installer from this checkout for Apple Silicon and Intel.
Uses the version in Cargo.toml; writes the DMG, dependency sources and opening
instructions to target/macos/.

Requires macOS, Command Line Tools, Rust/rustup, CMake and Python 3.11+.
Missing Rust targets are installed automatically. SWIFT_EXEC may select swiftc.

--publish also uploads the files to a new GitHub release and marks it Latest.
          First update Cargo.toml and Cargo.lock to a new version, then commit
          and push. Requires GitHub CLI (gh) authenticated with write access.
          Existing release tags are never replaced.
EOF
}

fail() { printf '%s\n' "$*" >&2; exit 1; }

publish=false
[[ $# -le 1 ]] || { usage >&2; exit 1; }
case "${1-}" in
    '') ;;
    --publish) publish=true ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; exit 1 ;;
esac
[[ $(uname -s) == Darwin ]] || fail 'Build the Mac installer on macOS.'

repo=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd -P)
cd "$repo"
for tool in python3 cargo rustup cmake "${SWIFT_EXEC:-swiftc}"; do
    command -v "$tool" >/dev/null || fail "Missing build tool: $tool (see --help)."
done
python3 -c 'import tomllib' 2>/dev/null || fail 'Python 3.11 or later is required.'
version=$(python3 - <<'PY'
import tomllib
from pathlib import Path
package = tomllib.loads(Path("Cargo.toml").read_text())["package"]
locked = tomllib.loads(Path("Cargo.lock").read_text())["package"]
assert any(p["name"] == package["name"] and p["version"] == package["version"]
           and "source" not in p for p in locked), "Update the package version in Cargo.lock too."
print(package["version"])
PY
)
tag="v$version"

if "$publish"; then
    command -v gh >/dev/null || fail 'Install GitHub CLI, then run gh auth login.'
    gh auth status
    git diff --quiet HEAD || fail 'Commit your changes before publishing.'
    untracked=$(git ls-files --others --exclude-standard -- scripts packaging/macos src formats skills/language-jutsu-listening tests/fixtures)
    [[ -z "$untracked" ]] || fail "Commit these packaging inputs before publishing:
$untracked"
    commit=$(git rev-parse HEAD)
    branch=$(git symbolic-ref --short HEAD) || fail 'Check out a branch before publishing.'
    remote_refs=$(git ls-remote origin)
    if printf '%s\n' "$remote_refs" | awk -v ref="refs/tags/$tag" '$2 == ref { found = 1 } END { exit !found }'; then
        fail "$tag already exists. Choose a new version in Cargo.toml and Cargo.lock."
    fi
    printf '%s\n' "$remote_refs" | awk -v commit="$commit" -v ref="refs/heads/$branch" '$1 == commit && $2 == ref { found = 1 } END { exit !found }' || fail 'Push this commit to origin before publishing.'
fi

rustup target add aarch64-apple-darwin x86_64-apple-darwin
work="$repo/target/macos"
# Recreate distributable contents so deleted files cannot survive an update.
rm -rf "$work/payloads" "$work/dmg" "$work/sources"
for arch in arm64 x86_64; do
    python3 packaging/macos/build.py payload --arch "$arch"
done
python3 packaging/macos/build.py assemble
dmg="$work/Language-Jutsu-$version-unsigned.dmg"
hdiutil verify "$dmg"
tar -czf "$work/dependency-sources.tar.gz" -C "$work" sources
cp packaging/macos/OPENING.txt "$work/OPENING.txt"

printf '\nPackaged %s:\n%s\n%s\n%s\n' "$version" "$dmg" "$work/dependency-sources.tar.gz" "$work/OPENING.txt"
if "$publish"; then
    gh release create "$tag" "$dmg" "$work/dependency-sources.tar.gz" "$work/OPENING.txt" \
        --repo nomicflux/language-jutsu --target "$commit" \
        --title "Language Jutsu $version" --notes-file "$work/OPENING.txt" --draft
    gh release edit "$tag" --repo nomicflux/language-jutsu --draft=false --latest
    printf '\nPublished: https://github.com/nomicflux/language-jutsu/releases/tag/%s\n' "$tag"
fi
