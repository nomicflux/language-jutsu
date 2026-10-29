# Language Jutsu

Language Jutsu turns foreign-language recordings and YouTube videos into study audio for repeated listening practice. Practice short passages at slower speeds, repeat them with gaps to respond, and build up to longer stretches of conversation. It can also transcribe recordings and prepare lesson flashcards for Anki.

Use Language Jutsu in Codex or Claude Code to prepare and teach interactive lessons around that audio, with vocabulary, grammar explanations and practice at your level.

## Example lessons

In Claude Code:

```text
/language-jutsu-listening Shami Arabic, basic conversation level, 30 minutes.
Use https://www.youtube.com/watch?v=mK-xzqnJmSQ from 03:00 to 05:00.
I lose track of who is replying to what. Short phrases, lots of repetition.
```

In Codex:

```text
$language-jutsu-listening Japanese, JLPT N3. A scene from a slice-of-life anime
about ordering food. 20 minutes, I'll type my answers. Include readings for kanji,
and make a new Anki deck from the lesson.
```

## Install on Mac

**[Download Language Jutsu for Mac](https://github.com/nomicflux/language-jutsu/releases/download/v0.1.0/Language-Jutsu-0.1.0-unsigned.dmg)**

Requires macOS 13 or later, on an Apple Silicon or Intel Mac. Guided lessons run in [Codex](https://developers.openai.com/codex/app/) or [Claude Code](https://code.claude.com/docs/en/desktop-quickstart); install one and sign in if you have not already.

1. Download the installer using the link above.
2. Open the downloaded `.dmg`, then double-click **Install Language Jutsu**.
3. If macOS blocks it, open **System Settings → Privacy & Security**, find the blocked-app message, and choose **Open Anyway**. Confirm when prompted. The installer is unsigned.
4. Click **Install**. Keep your internet connection available while the speech model downloads automatically (about 1.5 GiB on the first installation). You do not need to choose a model or install programming tools.
5. When setup finishes, click **Copy Codex Prompt** or **Copy Claude Code Prompt**, open that assistant, and paste the prompt to start your first lesson.
6. Eject the Language Jutsu disk image when you are done.

If the listening skill does not appear, restart Codex or Claude Code. If setup fails, check your internet connection and click **Retry**.

## Shape your lesson

Tell the assistant:

- The language and dialect you want to practise.
- Your level and how much time you have for the whole lesson.
- A recording or video to use, or a topic and type of scene to find.
- What is difficult: hearing familiar words, understanding unfamiliar expressions, or following the conversation.
- Whether you want short, slow, repeated passages or more continuous speech.
- Whether you will type or speak your answers, and which language to use for explanations.

Vocabulary and grammar notes are included unless you decline them. You can also request transcripts, pronunciation help, readings for unfamiliar writing systems, or flashcards.

## Anki flashcards

Ask for flashcards and say whether to **create a new deck** or **add to an existing deck**. For an existing deck, name it and provide an export or access.

The lesson's vocabulary, grammar and listening cards come in an `.apkg` file. In Anki, choose **File → Import** and select that file.

## Listen on your phone

You can copy the finished MP3 files to your usual audio player, or use iBroadcast playlists.

For iBroadcast (experimental):

1. Sign in to the [iBroadcast web player](https://media.ibroadcast.com/) in a browser your assistant can control, including file uploads.
2. Install the iBroadcast phone app and sign in to the same account.
3. Ask: “Set up iBroadcast playlists for my lessons, including existing lessons.”

You get one playlist per lesson and one per language, with newer lessons first. If a playlist is missing on your phone, use **Profile → Settings → Library → Refresh Library Now**. If browser control is unavailable, the assistant can give you the MP3 files and manual upload instructions.

## Reminders

Ask the listening skill to set up daily preparation and lesson reminders. The defaults are 17:00 to prepare tomorrow's lesson and 12:00 to start today's, in America/Chicago; you can change either time. Installation does not activate reminders.

```text
/language-jutsu-listening Set up my daily preparation and start reminders.
```

In Codex, use `$language-jutsu-listening` instead of `/language-jutsu-listening`.

## Your lessons and updates

Your lessons are saved in **Documents → Language Jutsu → listening**, unless you choose another location. The installer also provides an **Open Lessons Folder** button.

To update Language Jutsu, download a newer installer from [Releases](https://github.com/nomicflux/language-jutsu/releases) and run it. Your saved lessons and downloaded speech model are retained.
