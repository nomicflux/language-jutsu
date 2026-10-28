# Two daily reminders and their lesson handoff

Activate schedules only when the user requests setup/activation. Development or installation does not restore previously deleted notifications. The established requested times are **17:00 to prepare tomorrow** and **12:00 to start today**, independently editable, in **America/Chicago**. Noon is provisional. These are two separate same-chat heartbeat automations.

## Set up through the desktop tool

Discover `automation_update` and read its current schema. Inspect existing automations as its instructions require; update a matching reminder instead of duplicating it. Use the current chat and a local-calendar recurrence with America/Chicago, not a fixed UTC offset. Use the tool's actual supported input; do not invent serialization or write raw automation files/directives.

The saved prompts explicitly invoke `$language-jutsu-listening` and contain the **absolute durable lesson-root path**, so a later run can find the same files. Substitute that path in these prompt templates before saving them:

**Preparation event:**

> Use $language-jutsu-listening with lesson root [resolved absolute path]. This is the daily preparation reminder. Resolve tomorrow's calendar date in America/Chicago. Invite me to choose tomorrow's conversation, using the established instruction language. Do not infer content from documentation examples or silently reuse today's request. Record the pending preparation date so my reply prepares that date's lesson. If tomorrow is already prepared, identify it briefly and invite any change. Keep this reminder short; prepare the full interactive lesson after I supply the request. Preserve today's unfinished lesson.

**Start event:**

> Use $language-jutsu-listening with lesson root [resolved absolute path]. This is the daily start reminder. Resolve today's calendar date in America/Chicago and read that date's task.md. Invite me to begin the prepared lesson using its purpose, approximate session time and first activity; if already started, resume the pending learner turn. Use the established instruction language for directions and the target language for practice conversations. If absent or unready, state the actual missing preparation and elicit what is needed without inventing content or presenting a failed lesson. Do not rerun completed preparation merely to remind me.

These are requested daily reminders, so each run produces its reminder even if files are unchanged. Configure notification preferences through the tool's supported fields; do not put notification-policy controls in the saved prompt. Preserve later user changes independently for each reminder. Verify the two returned next occurrences in Chicago time and report any timezone or tool limitation instead of silently changing the schedule.

## Keep date handoff concrete

At the preparation event, write `listening/pending-preparation-date.txt` with tomorrow's ISO date. The reply to that invitation uses that saved date unless the user names another; a delayed reply must not silently shift the task to a different day. Once the reply's literal request and assigned date are saved in `listening/YYYY-MM-DD/task.md`, clear the pending invitation file. Later unrelated requests must not inherit a stale invitation date. Preparation keeps durable media/evidence for that dated task. Today's event reads today's record, never an arbitrary latest file. Mark failed preparation accurately with its specific unresolved cause; a rendered file alone does not make the task ready.

A completed task can be acknowledged with its prepared material still available; do not reset it to a new quiz. A missing task prompts a content request. A new preparation event does not overwrite current comprehension progress.

## Verify computer notification delivery

Desktop schedules that need local files require the computer on and the app running. Turn-completion alerts must be enabled in the app and permitted by the OS; completion may follow the scheduled start. Check the existing settings and, during authorized setup, arrange the app/OS controls needed for the requested banners. If the OS requires the user's own permission click, explain that exact remaining action. Do not claim banner delivery from successful schedule creation alone; confirm the first actual notification when possible.

The actual scheduling/notification internals are not available as source code here. Their request shape comes from the live tool schema; OS delivery remains unverified until observed. Current official references: [scheduled tasks](https://learn.chatgpt.com/docs/automations?surface=app), [desktop notifications](https://learn.chatgpt.com/docs/notifications?surface=app).
