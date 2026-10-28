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

## Phase 2 — Skill automation: complete, 2026-10-28

### Delivered

- Added the iBroadcast setup, backfill, revision and supersession workflow to the listening skill. It uses ordinary actions in the user's signed-in web player; no API, developer app, keys or OAuth.
- Corrected final-audio selection to read the actual lesson Markdown and match its final players to render/sequence reports. Investigation, setup, card and supplemental clips are excluded.
- New lessons render directly to MP3. Final files use `<nn>-<topic>.mp3`, numbered from `01` in lesson order; scratch and superseded audio use other names. Existing final MP3s are reused; only older final WAVs require conversion.
- Reconciled seven existing lessons: 20 final players, seven lesson playlists and four language playlists, newest lesson first. Arabic has 6 tracks, Japanese 6, Spanish 6, Indonesian 2. Exact file mappings and ordered titles are recorded in each lesson's `task.md`; playlist links and setup state are in `listening/ibroadcast.md`.
- Removed nine incorrectly uploaded YuruCamp supplemental focus clips and their generated MP3 copies. Retained its three final lesson players. The Kuwaiti lesson Markdown specifies two practice players followed by two connected players; those four were verified and retained.
- Standardized the 20 final MP3 filenames without re-encoding and updated lesson references. Existing remote copies were reused. Original WAVs and card/support material were preserved.

### Checks run

- Phase 1 format, full test suite (79 tests) and Clippy checks passed during Phase 2; no application code changed afterward. Skill validation and `git diff --check` passed.
- Matched the final MP3 durations to the original render reports, checked title/album/artist/track tags, and confirmed renaming preserved file hashes. Exactly 20 lesson files match the reserved naming scheme; no scratch files match it.
- Verified all seven lesson playlists and all four language playlists in the web player against their ordered titles, albums and languages.
- B1: temporary lesson upload completed with zero errors; completion was first observed 29 seconds after submission. Tracks appeared under the intended album and language.
- B2/new lesson: created a two-player temporary lesson playlist from its album and rebuilt Indonesian with that newer lesson before the existing Jakarta lesson. Adding the older album appended its tracks in lesson order.
- B3/revision: uploaded a replacement temporary player, trashed the old player, and observed its automatic removal from the language playlist. Rebuilt the lesson and language playlists and verified the replacement's position.
- Supersession: trashed the remaining temporary album, deleted its playlist, and rebuilt Indonesian with only the two real Jakarta players. Removed local test copies and staging folders. The nine incorrect clips and three test tracks remain recoverable in iBroadcast Trash.
- B4: user confirmed all four final language playlists on the phone with counts 6, 6, 6 and 2. Computer and phone checks are complete. Temporary copies were used for the lifecycle tests with the user's authorization; no new teaching lesson was authored for those tests.
