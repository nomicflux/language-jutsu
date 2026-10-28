# iBroadcast playlists — implementation status

## Phase 1 — CLI `mp3-copy`: complete

### Delivered

- `language-jutsu mp3-copy <input> --title T --album A --artist R --track N --output X.mp3` writes a tagged MP3 copy (`-map_metadata -1`, libmp3lame `-q:a 2`). The output must end in lowercase `.mp3`; an existing file is never replaced.
- `src/output.rs::reserve` is the one shared implementation of the wrong-type check, the no-replace check and temp-file creation. `render::publish`, `anki::build` and `mp3_copy::copy` use it with their existing error messages.
- Extensions are matched lowercase only. `render` no longer accepts `.WAV`/`.MP3`, and `anki` still rejects `.APKG`.
- Checks run before output: extension, then existing file.
- `render::publish` and `anki::build` were split into functions under 20 lines.
- README documents the command.

### Checks run

- `cargo fmt --check` → exit 0.
- `cargo test` → 0 failed (4 unit, 3 + 60 + 4 + 8 integration, including 3 new `mp3_copy_*` tests in `tests/cli.rs` and 2 `output` unit tests).
- `cargo clippy --all-targets` → exit 0.
- Real run on `listening/2026-10-28/jakarta-savings/savings-ex1-are-you-a-saver.wav`: ffprobe reported `format_name=mp3`, title "Are you a saver?", album `2026-10-28 Jakarta street talk: "Can you save money?"`, artist Indonesian, track 1. The resulting MP3 was deleted afterwards.
