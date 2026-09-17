---
hi: 1
families: [ISSUE, EXPORT, INDEX]
owner: leif
---

# Generate

## Intent

Intent is written once, by a human, and everything downstream is generated from it: tickets to work from, a payload an agent can turn into a spec, and the feature list at the front of the product so nobody keeps it by hand. That is what makes writing it down first pay for itself instead of being one more document to maintain.

Generation must work with no auth, no network, and no integration, because the moment it needs setup it stops being used.

## Criteria

- **ISSUE-1**  I can turn a criterion into a ticket without leaving the terminal.
  - **ISSUE-1.a**  I get a ticket printed by default, so it works with whatever tracker I actually use.
    - **ISSUE-1.a.1**  I can print a ticket with no login, no network, and nothing set up first.
  - **ISSUE-1.b**  I can add one flag and open a real GitHub issue instead.
- **ISSUE-2**  I find the criterion id on the ticket, so a closed ticket traces back to the intent it served.
- **ISSUE-3**  I get the criterion's cases in the body, so the ticket is the whole picture.
  - **ISSUE-3.a**  I read a case in the ticket as nested under what it is a case of, not flattened into a list of peers.
- **ISSUE-4**  I cannot turn a retired criterion into work.
- **ISSUE-5**  I get the feature's intent prose on the ticket, so I know why the work exists and not just what to build.
- **ISSUE-6**  A ticket carries my intent prose, and nothing hi generated into the file around it.
- **ISSUE-7**  I read the intent on a ticket as whole paragraphs, not as a narrow column broken wherever the lines happened to be wrapped in the file.
  - **ISSUE-7.a**  A blank line I left between two thoughts is still a break on the ticket, because that is where I meant one.
  - **ISSUE-7.b**  A list or an example in my intent arrives on the ticket with its own lines intact.

- **EXPORT-1**  I can hand an agent everything it needs to write the spec in one command.
  - **EXPORT-1.a**  I get the intent prose and not just the criteria.
- **EXPORT-2**  I can export one family, one file, or the whole product.
- **EXPORT-3**  I read a smaller export as the same payload with less in it, so I never need a special case.
- **EXPORT-4**  I get the retired criteria in the export kept apart from the live ones, so the agent never writes a spec for something we dropped.
- **EXPORT-5**  The starter prompts hi wrote into a file never reach an agent as if I had written them.

- **INDEX-1**  I get a root file that shows what features exist without keeping a list by hand.
  - **INDEX-1.a**  I can run hi with no root file yet, or no list in it, and it starts one with a place for my own prose rather than refusing until I set it up.
- **INDEX-2**  I keep the rest of the root file mine, because hi only ever rewrites the list it generated.
  - **INDEX-2.a**  I can quote the comments hi marks its list with, even on a line of their own or inside a code block, and hi does not mistake my example for the real list.
  - **INDEX-2.b**  I get a refusal when those comments are broken or unpaired, rather than hi guessing where its list ends.
- **INDEX-3**  The product-level file exists from my first capture, so I never have to discover it.
  - **INDEX-3.a**  hi keeps reminding me while the product-level why is still unwritten.
- **INDEX-4**  The list at the front of my product is true after every command that changes it, without me remembering to refresh it.
  - **INDEX-4.a**  A capture that stored my criterion is never reported as a failure because the list could not be refreshed.
  - **INDEX-4.b**  If I write a criterion into the file by hand, hi tells me the list is behind rather than leaving it wrong.
