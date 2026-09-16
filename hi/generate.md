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

- **ISSUE-1**  As a developer, I can turn a criterion into a ticket without leaving the terminal.
  - **ISSUE-1.a**  As a developer, I get a ticket printed by default, so it works with whatever tracker I actually use.
    - **ISSUE-1.a.1**  As a developer, I can print a ticket with no login, no network, and nothing set up first.
  - **ISSUE-1.b**  As a developer, I can add one flag and open a real GitHub issue instead.
- **ISSUE-2**  As a reader, I find the criterion id on the ticket, so a closed ticket traces back to the intent it served.
- **ISSUE-3**  As a developer, I get the criterion's cases in the body, so the ticket is the whole picture.
  - **ISSUE-3.a**  As a developer, I read a case in the ticket as nested under what it is a case of, not flattened into a list of peers.
- **ISSUE-4**  As a developer, I cannot turn a retired criterion into work.
- **ISSUE-5**  As a developer, I get the feature's intent prose on the ticket, so I know why the work exists and not just what to build.

- **EXPORT-1**  As a developer, I can hand an agent everything it needs to write the spec in one command.
  - **EXPORT-1.a**  As an agent, I get the intent prose and not just the criteria.
- **EXPORT-2**  As a developer, I can export one family, one file, or the whole product.
- **EXPORT-3**  As an agent, I read a smaller export as the same payload with less in it, so I never need a special case.
- **EXPORT-4**  As a developer, I get the retired criteria in the export kept apart from the live ones, so the agent never writes a spec for something we dropped.

- **INDEX-1**  As a developer, I get a root file that shows what features exist without keeping a list by hand.
  - **INDEX-1.a**  As a developer, I can run hi with no root file yet, or no list in it, and it starts one with a place for my own prose rather than refusing until I set it up.
- **INDEX-2**  As a developer, I keep the rest of the root file mine, because hi only ever rewrites the list it generated.
  - **INDEX-2.a**  As a developer, I can quote the comments hi marks its list with, even on a line of their own or inside a code block, and hi does not mistake my example for the real list.
  - **INDEX-2.b**  As a developer, I get a refusal when those comments are broken or unpaired, rather than hi guessing where its list ends.
- **INDEX-3**  As a developer, the product-level file exists from my first capture, so I never have to discover it.
  - **INDEX-3.a**  As a developer, hi keeps reminding me while the product-level why is still unwritten.
