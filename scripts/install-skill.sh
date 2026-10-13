#!/bin/sh
set -eu

if [ "$#" -gt 1 ]; then
    printf 'Usage: %s [skills-directory]\n' "$0" >&2
    exit 1
fi

if [ "${1-}" = '--help' ] || [ "${1-}" = '-h' ]; then
    printf 'Usage: %s [skills-directory]\nDefault: ~/.agents/skills\n' "$0"
    exit 0
fi

repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
skill="$repo/skills/language-jutsu-listening"
skills_dir=${1:-"$HOME/.agents/skills"}
destination="$skills_dir/language-jutsu-listening"

if [ ! -f "$skill/SKILL.md" ]; then
    printf 'Bundled skill not found: %s\n' "$skill" >&2
    exit 1
fi

if [ -L "$destination" ] && [ "$(readlink "$destination")" = "$skill" ]; then
    printf 'Already installed: %s\n' "$destination"
elif [ -e "$destination" ] || [ -L "$destination" ]; then
    printf 'Cannot install: %s already exists. Nothing was replaced.\n' "$destination" >&2
    exit 1
else
    mkdir -p "$skills_dir"
    ln -s "$skill" "$destination"
    printf 'Installed: %s -> %s\n' "$destination" "$skill"
fi

printf 'In Codex, invoke $language-jutsu-listening. Restart Codex if it does not appear.\n'
