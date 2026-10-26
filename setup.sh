#!/bin/sh
# Install Language Jutsu's tools, Whisper model and listening skill
# (macOS with Homebrew; Debian/Ubuntu on x86_64).
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
cd "$repo"
export PATH="$HOME/.cargo/bin:$HOME/.deno/bin:$PATH"

unsupported() {
    echo 'setup.sh supports macOS with Homebrew and Debian/Ubuntu on x86_64' >&2
    exit 1
}

# Print the package for each absent command; arguments are command=package pairs.
missing_packages() {
    for pair in "$@"; do
        command -v "${pair%%=*}" >/dev/null || printf '%s ' "${pair#*=}"
    done
}

# Install the official standalone yt-dlp executable named by $1.
install_yt_dlp() {
    sudo mkdir -p /usr/local/bin
    sudo curl -fL "https://github.com/yt-dlp/yt-dlp/releases/latest/download/$1" -o /usr/local/bin/yt-dlp
    sudo chmod 755 /usr/local/bin/yt-dlp
}

install_mac_tools() {
    command -v brew >/dev/null || unsupported
    packages=$(missing_packages cargo=rust cmake=cmake ffmpeg=ffmpeg deno=deno)
    [ -z "$packages" ] || brew install $packages
    command -v yt-dlp >/dev/null || install_yt_dlp yt-dlp_macos
}

install_linux_tools() {
    if ! command -v apt-get >/dev/null || [ "$(uname -m)" != x86_64 ]; then unsupported; fi
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends ca-certificates curl unzip build-essential cmake libclang-dev ffmpeg
    command -v cargo >/dev/null || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    command -v deno >/dev/null || curl -fsSL https://deno.land/install.sh | sh -s -- -y
    command -v yt-dlp >/dev/null || install_yt_dlp yt-dlp_linux
}

case "$(uname -s)" in
    Darwin) install_mac_tools ;;
    Linux) install_linux_tools ;;
    *) unsupported ;;
esac
cargo build
echo 'Downloading the Whisper model (about 1.5 GiB on first run)...'
cargo run --quiet -- download-model
scripts/install-skill.sh "$HOME/.agents/skills"
scripts/install-skill.sh "$HOME/.claude/skills"
echo 'Setup complete.'
