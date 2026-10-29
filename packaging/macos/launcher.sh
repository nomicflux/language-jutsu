#!/bin/sh
set -eu
installation=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
export PATH="$installation/runtime:/usr/bin:/bin:/usr/sbin:/sbin"
exec "$installation/runtime/language-jutsu" "$@"
