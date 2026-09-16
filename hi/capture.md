---
hi: 1
families: [CAPTURE]
owner: leif
---

# Capture

## Intent

The four seconds between having a thought and losing it is the only scarce
resource here. Every acceptance-criteria tool that died, died at authoring
time: a form appeared and the person went back to Slack. So capturing has to
be a reflex, with no prompts, no wizard, no required fields and no init.

The one thing I want it to be strict about is refusing to quietly clobber
something that already exists.

## Criteria

- **CAPTURE-1**  As a developer, I can write a thought down in one command with no setup.
  - **CAPTURE-1.a**  As a developer, I never have to run an init step before hi is useful.
  - **CAPTURE-1.b**  As a developer, I am never asked a question in the middle of capturing.
  - **CAPTURE-1.c**  As a developer, if I type something hi cannot handle, it says so plainly instead of crashing with a stack trace.
- **CAPTURE-2**  As a developer, I can use a brand new id and it just works.
  - **CAPTURE-2.a**  As a developer, if the family is new, hi starts the file itself rather than asking me where to put it.
  - **CAPTURE-2.b**  As a developer, if I hang a case off something that is not there yet, hi says so and names what is missing.
- **CAPTURE-3**  As a developer, if I reuse an id that already exists it refuses, and tells me the next free one.
- **CAPTURE-4**  As a reader, I find a case directly under its parent, not at the bottom of the file.
  - **CAPTURE-4.a**  As a reader, I find a case in the file where its parent actually lives, not wherever the family happens to be declared.
- **CAPTURE-5**  As a developer, if hi refuses for any reason, nothing is written to disk.
- **CAPTURE-6**  As a developer, hi finds my workspace by what is inside it, not by a directory name, so a folder called hi for something else is left alone.
- **CAPTURE-7**  As a developer, if a file has no criteria section yet, hi makes one instead of appending wherever the file happens to end.
- **CAPTURE-8**  As a developer, I put hi's own options before the id, so nothing in my sentence is mistaken for one.
- **CAPTURE-9**  As a developer, the sentence I type is the sentence that lands in the file, word for word.
- **CAPTURE-10**  As a developer, hi works from any directory inside my repository, and a repository is where it stops looking.
- **CAPTURE-11**  As a developer, every capture tells me which file it landed in, so I never have to go looking.
