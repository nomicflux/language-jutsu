# Language Jutsu for Mac

This folder contains the installed program, its runtime tools, formats, and listening skill.

Invoke `bin/language-jutsu` by its absolute path. It forwards all arguments and finds the bundled tools without changing your working directory or shell settings. Use `--help` for command options. The skill's `scripts/jutsu` runner invokes this same launcher.

Guided lessons run in Codex or Claude Code. Invoke `$language-jutsu-listening` in Codex or `/language-jutsu-listening` in Claude Code. Restart the assistant if the skill does not appear.

Lessons are saved in `~/Documents/Language Jutsu/listening/` unless you choose another location. The default speech model is downloaded automatically into the Hugging Face cache, normally `~/.cache/huggingface/hub/`. The program also honors `HF_HUB_CACHE`, `HF_HOME`, and `XDG_CACHE_HOME` when present in its environment.

Run a newer installer to update this folder. Lessons and cached models are kept separately. Third-party notices are in `licenses/` and `runtime/_internal/THIRD_PARTY_LICENSES.txt`; the distribution includes a companion source archive.
