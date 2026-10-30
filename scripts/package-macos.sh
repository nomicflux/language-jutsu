#!/bin/bash
set -euo pipefail
repo=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd -P)
exec python3 "$repo/scripts/package_macos.py" "$@"
