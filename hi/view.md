---
hi: 1
families: [VIEW]
---

# The view

## Intent

The people who decide what we build mostly do not want to read a markdown
file full of identifiers. If the only way to see what we agreed to is to open
`hi/chat.md` in an editor, then the intent is written for engineers and the
product side goes back to arguing from memory.

So there has to be a view that is just the sentences. The why first, the
criteria as a readable list, and the ids present but quiet. One file I can
send to somebody.

## Criteria

- **VIEW-1**  As a reader, I can see what we agreed to without reading markdown or being handed a file full of ids.
  - **VIEW-1.a**  As a reader, I get the intent prose first, and the ids stay small and out of the way.
  - **VIEW-1.b**  As a reader, I see cases visually nested under what they are cases of.
  - **VIEW-1.c**  As a developer, I can leave comments in my intent prose and they stay out of the page.
  - **VIEW-1.d**  As a reader, I can copy text off the page and the id and the sentence stay apart, with each criterion on its own line.
- **VIEW-2**  As a developer, I have one self-contained file I can send to anyone.
  - **VIEW-2.a**  As a developer, I know the page reads properly on a phone, because that is where people open what I send them.
- **VIEW-3**  As a developer, I can use **bold**, *italic*, `code`, and links in a criterion, and they render properly.
  - **VIEW-3.a**  As a reader, anything that looks like markup in a sentence is escaped, never executed.
- **VIEW-4**  As a reader, retired criteria are on the page but folded away, so the history is there without being noise.
- **VIEW-5**  As a developer, nothing on the page says whether anything is done, so nobody can read it as a progress report.
