---
hi: 1
families: [FILE, ID]
owner: leif
---

# The format

## Intent

A hi file has to be something a person would write anyway. If someone deletes
the binary tomorrow, the files should still read perfectly as a document.
That is the whole test. Nothing in a hi file exists to serve the machine
except one version line, and nothing the tool writes should look different
from what a careful person would have typed by hand.

Ids are the one thing I am strict about, because an id that moves is worse
than no id at all. You should be able to say "SEND-1.a is wrong" out loud in
a standup a year from now and have it still mean the same line.

## Criteria

- **FILE-1**  A hi file reads as an ordinary markdown document with no tool installed.
  - **FILE-1.a**  If the binary disappears tomorrow, the files still make sense to a person.
  - **FILE-1.b**  Read on GitHub or in any preview, the criteria are a list, one per line with cases nested, not a paragraph of run-together sentences.
  - **FILE-1.c**  The id is bold, so it reads as a label rather than as the first words of the sentence.
- **FILE-2**  The machine-facing part of a file is its frontmatter, and nothing below it exists to serve the tool.
- **FILE-3**  When hi writes a line, it looks like something I would have typed by hand.
- **FILE-4**  hi never rewrites, reflows, or reformats prose that I wrote.
  - **FILE-4.a**  After hi adds a criterion, the rest of the file comes back byte for byte identical.
- **FILE-5**  One file covers one feature, and it can hold several id families.
- **FILE-6**  One criterion is one line, however long the sentence runs, so I can grep it and diff it.
- **FILE-7**  hi reads my frontmatter in whichever YAML style I wrote it, and writes it back in that same style.
- **FILE-8**  If a write fails partway through, my file is left exactly as it was.
- **FILE-9**  A fenced code block in my prose is prose, so I can show an example of the format without it becoming real criteria.
- **FILE-10**  If my file uses Windows line endings, hi writes them back the same way.
- **FILE-11**  A byte-order mark from my editor does not make a valid file look broken.
- **FILE-12**  Paths hi prints or exports use forward slashes on every platform, so a link works wherever it is read.

- **ID-1**  An id never moves once written, so I can say it out loud a year later and still mean the same line.
  - **ID-1.a**  Inserting a criterion never renumbers anything around it.
  - **ID-1.b**  A retired id stays reserved forever and is never handed out again.
  - **ID-1.c**  A zero-padded number is refused, because SEND-007 and SEND-7 must never be two names for one line.
- **ID-2**  I choose the id myself, because I am the one who has to say it.
- **ID-3**  Reading an id tells me whether it is a case or a step.
  - **ID-3.a**  A letter is another case of its parent.
  - **ID-3.b**  A number is a step or a detail inside its parent.
- **ID-4**  If I write an id that breaks the alternation, hi tells me instead of quietly accepting it.
