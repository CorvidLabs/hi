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

- **CAPTURE-1**  Writing down a thought takes one command and no setup.
  - **CAPTURE-1.a**  There is no init step before hi is useful.
  - **CAPTURE-1.b**  I am never asked a question in the middle of capturing.
  - **CAPTURE-1.c**  If I type something hi cannot handle, it says so plainly instead of crashing with a stack trace.
- **CAPTURE-2**  A brand new id just works.
  - **CAPTURE-2.a**  If the family is new, hi starts the file itself rather than asking me where to put it.
  - **CAPTURE-2.b**  If I hang a case off something that is not there yet, hi says so and names what is missing.
- **CAPTURE-3**  An id that already exists refuses, and tells me the next free one.
- **CAPTURE-4**  A case lands directly under its parent, not at the bottom of the file.
  - **CAPTURE-4.a**  A case lands in the file where its parent actually lives, not wherever the family happens to be declared.
- **CAPTURE-5**  If hi refuses for any reason, nothing is written to disk.
- **CAPTURE-6**  hi finds my workspace by what is inside it, not by a directory name, so a folder called hi for something else is left alone.
- **CAPTURE-7**  If a file has no criteria section yet, hi makes one instead of appending wherever the file happens to end.
- **CAPTURE-8**  I put hi's own options before the id, so nothing in my sentence is mistaken for one.
- **CAPTURE-9**  The sentence I type is the sentence that lands in the file, word for word.
- **CAPTURE-10**  hi works from any directory inside my repository, and a repository is where it stops looking.
- **CAPTURE-11**  Every capture tells me which file it landed in, so I never have to go looking.
