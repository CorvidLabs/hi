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

- **VIEW-1**  I can see what we agreed to without reading markdown or being handed a file full of ids.
  - **VIEW-1.a**  I get the intent prose first, and the ids stay small and out of the way.
  - **VIEW-1.b**  I see cases visually nested under what they are cases of.
  - **VIEW-1.c**  I can leave comments in my intent prose and they stay out of the page.
  - **VIEW-1.d**  I can copy text off the page and the id and the sentence stay apart, with each criterion on its own line.
- **VIEW-2**  I have one self-contained file I can send to anyone.
  - **VIEW-2.a**  I know the page reads properly on a phone, because that is where people open what I send them.
- **VIEW-3**  I can use **bold**, *italic*, `code`, and links in a criterion, and they render properly.
  - **VIEW-3.a**  Anything that looks like markup in a sentence is escaped, never executed.
- **VIEW-4**  Retired criteria are on the page but folded away, so the history is there without being noise.
- **VIEW-5**  Nothing on the page says whether anything is done, so nobody can read it as a progress report.
- **VIEW-6**  I can search the whole page by id or wording and see only what matches.
- **VIEW-7**  I can narrow the page to one feature by clicking it.
- **VIEW-8**  I can sort everything by id or by family instead of reading it grouped.
- **VIEW-9**  I can click any id to get a link straight to that criterion, and that link works when I send it to someone.
  - **VIEW-9.a**  A link to a criterion lands on it even when a filter would have hidden it.
- **VIEW-10**  I still see every criterion with scripting turned off, and I am not shown a search box that cannot search.
- **VIEW-11**  The page is named after my product, taking the name from the heading I wrote in INTENT.md.
- **VIEW-12**  I can reach any feature from a list that stays on screen while I scroll, instead of scrolling to look for it.
  - **VIEW-12.a**  The list tells me how many criteria are in each feature before I go there.
  - **VIEW-12.b**  The list shows me which feature I am currently reading.
- **VIEW-13**  When I search, the words that matched are highlighted where they sit, so I can see why a line came back.
- **VIEW-14**  I can move through criteria from the keyboard, without reaching for the mouse.
- **VIEW-15**  Clicking an id copies a link I can paste to someone, and tells me it did.
- **VIEW-16**  Nothing generic sits above my own words; the page opens with what I wrote, not with boilerplate about the tool.
- **VIEW-17**  The page wears the CorvidLabs brand, using the kit's own tokens rather than colours invented here.
  - **VIEW-17.a**  The two brand faces are named first and the page falls back to the system's own, because it still has to open with no network.
- **VIEW-18**  I can switch the page between light and dark myself, and it remembers which I chose.
