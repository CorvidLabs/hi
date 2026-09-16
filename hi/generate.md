---
hi: 1
families: [ISSUE, EXPORT, INDEX]
owner: leif
---

# Generate

## Intent

Intent is written once, by a human, and everything downstream is generated
from it: tickets to work from, and a payload an agent can turn into a spec.
That is what makes writing it down first pay for itself instead of being one
more document to maintain.

Generation must work with no auth, no network, and no integration, because
the moment it needs setup it stops being used.

## Criteria

- **ISSUE-1**  I can turn a criterion into a ticket without leaving the terminal.
  - **ISSUE-1.a**  By default it prints, so it works with whatever tracker I actually use.
  - **ISSUE-1.b**  With one flag it opens a real GitHub issue instead.
- **ISSUE-2**  The ticket carries the criterion id, so a closed ticket traces back to the intent it served.
- **ISSUE-3**  A criterion's cases come along in the body, so the ticket is the whole picture.
- **ISSUE-4**  A retired criterion never becomes work.

- **EXPORT-1**  I can hand an agent everything it needs to write the spec in one command.
  - **EXPORT-1.a**  The payload carries the intent prose, not just the criteria.
- **EXPORT-2**  I can export one family, one file, or the whole product.
- **EXPORT-3**  The shape is the same whatever scope I ask for, so nothing downstream has to branch.

- **INDEX-1**  The root file shows what features exist without me maintaining a list by hand.
- **INDEX-2**  hi only ever touches the generated block, and the prose above it stays mine.
  - **INDEX-2.a**  A marker quoted in my prose is not mistaken for the generated block.
  - **INDEX-2.b**  If the markers are broken, hi refuses rather than guessing where the block ends.
