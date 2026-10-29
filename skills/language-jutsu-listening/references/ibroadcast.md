# iBroadcast lesson playlists

Use the user's own signed-in [iBroadcast web player](https://media.ibroadcast.com/) through browser control. Each user has their own skill, CLI, account and playlists. Perform the web-player actions below; do not register a developer app, obtain keys, use OAuth, or call iBroadcast APIs.

## One-time setup

When the user requests setup:

1. Have them create a free iBroadcast account if needed and sign in at `media.ibroadcast.com` in the browser the agent can control.
2. Have them install the iBroadcast phone app from the App Store or Google Play and sign in to the same account. On the computer, use the web player; on macOS, Safari's File → Add to Dock is an optional shortcut. See the [account help](https://help.ibroadcast.com/en/account).
3. Write `listening/ibroadcast.md`: playlists enabled, browser used, and whether sign-in was confirmed. Keep credentials in the browser, not this file. Its existence enables sync for subsequent lessons; check the current browser session when syncing.
4. Backfill every existing, non-superseded lesson using the final-audio selection and sync below. Read the dated and per-lesson records together so supersession notices and references to lesson subdirectories are respected.
5. Confirm the resulting playlists in the computer web player and ask the user to confirm them in the phone app. If the phone's library is stale, use Profile → Settings → Library → Refresh Library Now. Record which device checks actually succeeded.

## Select the actual final lesson audio first

Final lesson audio uses the reserved `<nn>-<topic>.mp3` names from SKILL.md. Select the recorded final files and order them numerically by `nn`. Before uploading, match them to the actual finished lesson and its render/sequence reports, applying the user's latest corrections. Use each complete player unchanged, including its rendered practice sequence. For older lessons without these names, first identify the actual final players from the lesson and its revisions, then rename the final MP3s to this scheme and update their lesson references; never promote a scratch clip by renaming it.

Exclude investigation and setup clips, source excerpts, checks, card audio, supplemental focus clips, drafts and superseded outputs. The numbered final files must agree with the lesson's final-audio list. Resolve conflicts before syncing; an old filename, broad progression listing or previous upload does not establish that a file is final lesson audio.

Use the existing final audio; do not rerender a lesson for sync.

## Files and names

| Item | Value |
| --- | --- |
| Local MP3 | The final lesson file `<nn>-<topic>.mp3`; for an older final WAV, its converted copy with this name |
| Title tag | The player's lesson label |
| Album tag and lesson playlist | `YYYY-MM-DD <lesson title>` |
| Artist tag and language playlist | Language name without dialect, e.g. `Arabic` for both Kuwaiti and Levantine |
| Track tag | Numeric segment position (`01` → `1`) |

Use the lesson's saved date and title, not today's sync date. Keep lesson album names unique; if two lessons have the same date and title, resolve their names before uploading. Only manage albums and playlists created for these lessons and recorded in their `task.md` files. A matching name alone does not establish ownership; resolve a collision with an existing personal playlist before changing it.

## Sync final players

1. Use the final files identified above. Compare them with the previous `iBroadcast` record and the actual library to identify new, unchanged, replaced, incorrectly included and superseded tracks. Previous sync records describe what happened; they do not validate the source selection. Reuse current final MP3s directly. Only for an older final WAV without a current MP3 copy, use the existing CLI:

   ```sh
   "$jutsu" mp3-copy '<player>.wav' --title '<player label>' \
     --album 'YYYY-MM-DD <lesson title>' --artist '<language>' \
     --track 1 --output '01-<topic>.mp3'
   ```

   Supply the actual paths, labels and position. The command never replaces an existing output. Reuse a converted copy only when it belongs to the verified current final player. If a revision reuses the same WAV pathname, move its previously recorded MP3 aside before making the new copy. Do not convert an existing final MP3 again merely to add tags.

2. Upload the selected MP3s that are not already present: Add Music → Upload Music Now, select only those files, and submit the upload. The [upload help](https://help.ibroadcast.com/en/uploading-music) also documents dragging files into the web player. If the picker accepts folders, use a temporary folder containing only the intended MP3s, retaining separate lesson folders; never select the whole lesson directory. Wait for processing to finish. Set missing or incorrect title, album, artist and track number through right-click → Edit Track → Save, using the table above and the [metadata editing instructions](https://help.ibroadcast.com/en/editing-library#edit-track-metadata). After an interruption, inspect the library before retrying an uncertain upload.

3. Move replaced and incorrectly included tracks from this sync to Trash, retaining the current final players. For a revision, first confirm the new track is present; iBroadcast may already have replaced a duplicate. For a superseded lesson, move its recorded album to Trash and delete its lesson playlist. Leave unrelated library items alone. [Track and album removal help](https://help.ibroadcast.com/en/editing-library).

4. Confirm that each affected album contains exactly the selected final lesson audio with the correct titles, language and track numbers before rebuilding its playlist. Delete its recorded previous playlist if present. Right-click the album → Add to Playlist → Add to New Playlist; enter the album name and create it. Check the resulting files and order against the lesson's final-audio list. Deleting a playlist leaves its library tracks intact. [Playlist and sorting help](https://help.ibroadcast.com/en/organizing-library).

5. Rebuild the affected language playlist from its remaining lesson albums, newest lesson first. Delete its recorded previous playlist. Use the newest album's tracks to create the language playlist, then add each older album in turn through Add to Playlist → Add to End. Include all dialects of that language. Check the resulting sequence, including album boundaries. If no lessons remain, remove the managed language playlist.

6. Update an `iBroadcast` section in each affected lesson's `task.md`: exact final MP3 paths (plus WAV-to-MP3 mappings for converted older players), ordered titles, album/lesson playlist name, language playlist name, confirmed uploads, removed/superseded items, and sync date. Record any unfinished step so continuation checks actual remote state instead of treating an incomplete sync as complete. A superseded lesson's record must say its album and playlist were removed. Device visibility confirms access, not that the correct source audio was selected.

If browser control is unavailable, identify the same final MP3s, convert only older final WAVs as above, and give the user their paths, exact playlist names and ordered upload/metadata/playlist steps. Record uploads and device checks only after the user confirms them; do not mark manual work as done.
