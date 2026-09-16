---
hi: 1
families: [FILE, ID]
owner: leif
---

# The format

## Intent

A hi file has to be something a person would write anyway. If someone deletes
the binary tomorrow, the files should still read perfectly as a document.
That is the whole test. Nothing below the frontmatter exists to serve the
machine, and nothing the tool writes should look different from what a
careful person would have typed by hand.

Ids are the one thing I am strict about, because an id that moves is worse
than no id at all. You should be able to say "SEND-1.a is wrong" out loud in
a standup a year from now and have it still mean the same line.

## Criteria

- **FILE-1**  A hi file reads as an ordinary markdown document with no tool installed.
  - **FILE-1.a**  If the binary disappears tomorrow, nothing is lost, because hi keeps nothing of its own anywhere else.
  - **FILE-1.b**  On GitHub or in any preview, the criteria are a list, one per line with cases nested, not a paragraph of run-together sentences.
  - **FILE-1.c**  The id is bold, so it reads as a label rather than as the first words of the sentence.
- **FILE-2**  The machine-facing part of a file is its frontmatter, and nothing below it exists to serve the tool.
- **FILE-3**  When hi writes a line, it looks like something I would have typed by hand.
- **FILE-4**  hi never rewrites, reflows, or reformats prose that I wrote.
  - **FILE-4.a**  After hi adds a criterion, the rest of the file comes back byte for byte identical, apart from the frontmatter line that names the families.
- **FILE-5**  One file covers one feature, and it can hold several id families.
- **FILE-6**  One criterion is one line, however long the sentence runs, so I can grep it and diff it.
- **FILE-7**  hi understands my frontmatter whether I list the families inline or one per line, and when it adds a family it keeps the style I chose.
- **FILE-8**  If a write fails partway through, my file is left exactly as it was.
- **FILE-9**  A fenced code block in my prose is prose, so I can show an example of the format without it becoming real criteria.
- **FILE-10**  If my file uses Windows line endings, hi writes them back the same way.
- **FILE-11**  A byte-order mark from my editor does not make a valid file look broken.
- **FILE-12**  The paths hi prints or exports use forward slashes on every platform, so a link works wherever it is read.
- **FILE-13**  A file starts with my own words about why this feature exists, so the first thing anyone reads is the why and not the list.
- **FILE-14**  I can type a criterion into the file by hand, bullet or no bullet, bold or plain, and hi still reads it as one.
- **FILE-15**  A criterion I changed my mind about stays in the file under Retired, with my reason beside it, so the file remembers what we dropped.
- **FILE-18**  I understand a criterion as what this should be, not as a report of what it currently does.
- **FILE-19**  Two captures running at the same time both land, instead of one quietly overwriting the other.
- **FILE-20**  A criterion hi cannot see is never silently invisible; it is reported rather than ignored.

- **ID-1**  An id never moves once written, so I can say it out loud a year later and still mean the same line.
  - **ID-1.a**  Inserting a criterion never renumbers anything around it.
  - **ID-1.b**  A retired id stays reserved forever and is never handed out again.
  - **ID-1.c**  A zero-padded number is refused, because SEND-007 and SEND-7 must never be two names for one line.
- **ID-2**  I choose the id myself, because I am the one who has to say it.
  - **ID-2.a**  A family name is in capitals, like SEND or BILLING or SEND_2FA, so an id stands out as a name in the middle of a sentence.
- **ID-3**  An id tells me whether it is a case or a step.
  - **ID-3.a**  A letter is another case of its parent, like SEND-1.a and SEND-1.b under SEND-1.
  - **ID-3.b**  A number is a step or a detail inside its parent, like SEND-1.a.1 inside SEND-1.a.
- **ID-4**  If I write an id that breaks the alternation, hi tells me instead of quietly accepting it.

## Retired

- **FILE-16**  I can tell who each criterion speaks for, because every one names the role it is written in.
  retired: roles were removed from the format; the plain sentence is the criterion
- **FILE-17**  If I cannot put a role in front of a sentence, I learn while typing that I wrote a fact rather than a want.
  retired: roles were removed from the format; the test survives as advice in DECISIONS, not as a rule
