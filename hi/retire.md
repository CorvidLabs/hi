---
hi: 1
families: [RETIRE]
---

# Changing your mind

## Intent

Changing your mind is normal, and the format has always had a place for it.
For a while the tool did not: `## Retired` existed in the file and no command
put anything there, so the only way to retire something was to hand-edit the
markdown, in a tool whose whole pitch is that you do not hand-edit.

Someone using hi cold on a real product cut seven criteria by deleting lines
and never found the section at all. Their sentences are gone and the ids they
used are not written down anywhere.

Retiring should cost one command, keep the id spoken for forever, and keep the
reason next to the thing it explains.

## Criteria

- **RETIRE-1**  As a person writing intent, I can retire a criterion with a command instead of hand-editing the file.
  - **RETIRE-1.a**  As a person writing intent, its cases go with it, so nothing is left orphaned behind it.
  - **RETIRE-1.b**  As a person writing intent, I can say why I changed my mind, and the reason stays next to what I retired.
  - **RETIRE-1.c**  As a person writing intent, I can come back later and say why, without editing the file by hand.
  - **RETIRE-1.d**  As a person writing intent, I am told which cases went with a criterion I retired, in case one belonged to something else.
- **RETIRE-2**  As a reader, a retired id is still spoken for, so it is never handed out to something else.
- **RETIRE-3**  As a reader, I am told when a retired criterion never says why it was retired.
- **RETIRE-4**  As a reader, I can learn the standard a team retires things by, because every retirement says why.
