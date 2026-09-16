---
hi: 1
families: [ISSUE, EXPORT, INDEX]
owner: leif
---

# Generate

## Intent

Intent is written once, by a human, and everything downstream is generated
from it: tickets to work from, a payload an agent can turn into a spec, and
the feature list at the front of the product so nobody keeps it by hand.
That is what makes writing it down first pay for itself instead of being one
more document to maintain.

Generation must work with no auth, no network, and no integration, because
the moment it needs setup it stops being used.

## Criteria

- **ISSUE-1**  I can turn a criterion into a ticket without leaving the terminal.
  - **ISSUE-1.a**  By default it prints, so it works with whatever tracker I actually use.
    - **ISSUE-1.a.1**  Printing a ticket needs no login, no network, and nothing set up first.
  - **ISSUE-1.b**  With one flag it opens a real GitHub issue instead.
- **ISSUE-2**  The ticket carries the criterion id, so a closed ticket traces back to the intent it served.
- **ISSUE-3**  A criterion's cases come along in the body, so the ticket is the whole picture.
  - **ISSUE-3.a**  In the ticket, a case reads as nested under what it is a case of, not flattened into a list of peers.
- **ISSUE-4**  A retired criterion never becomes work.
- **ISSUE-5**  The ticket carries the feature's intent prose, so whoever picks it up knows why the work exists and not just what to build.

- **EXPORT-1**  I can hand an agent everything it needs to write the spec in one command.
  - **EXPORT-1.a**  The payload carries the intent prose, not just the criteria.
- **EXPORT-2**  I can export one family, one file, or the whole product.
- **EXPORT-3**  A smaller export is the same payload with less in it, so the agent reading it never needs a special case.
- **EXPORT-4**  Retired criteria come along, kept apart from the live ones, so the agent never writes a spec for something we dropped.

- **INDEX-1**  The root file shows what features exist without me maintaining a list by hand.
  - **INDEX-1.a**  If there is no root file yet, or no list in it, hi starts one with a place for my own prose, rather than refusing until I set it up.
- **INDEX-2**  hi only ever rewrites the list it generated, and the rest of the root file stays mine.
  - **INDEX-2.a**  If I quote the comments hi marks its list with, even on a line of their own or inside a code block, hi does not mistake my example for the real list.
  - **INDEX-2.b**  If those comments are broken or unpaired, hi refuses rather than guessing where its list ends.
