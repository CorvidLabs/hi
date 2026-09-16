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

- **VIEW-1**  Someone who does not read markdown can see what we agreed to without opening a file.
  - **VIEW-1.a**  The intent prose comes first, and the ids stay small and out of the way.
  - **VIEW-1.b**  Cases are visually nested under what they are cases of.
  - **VIEW-1.c**  Comments I leave in my prose stay out of the page.
  - **VIEW-1.d**  Text copied off the page keeps the id and the sentence apart, and each criterion on its own line.
- **VIEW-2**  The page is one self-contained file I can send to anyone.
- **VIEW-3**  A criterion can use **bold**, `code`, and links, and they render properly.
  - **VIEW-3.a**  Anything that looks like markup in a sentence is escaped, never executed.
- **VIEW-4**  Retired criteria are on the page but folded away, so the history is there without being noise.
