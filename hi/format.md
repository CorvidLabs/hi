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

- **FILE-1**  As a reader, a hi file reads as an ordinary markdown document with no tool installed.
  - **FILE-1.a**  As a developer, if the binary disappears tomorrow, nothing is lost, because hi keeps nothing of its own anywhere else.
  - **FILE-1.b**  As a reader, on GitHub or in any preview, the criteria are a list, one per line with cases nested, not a paragraph of run-together sentences.
  - **FILE-1.c**  As a reader, the id is bold, so it reads as a label rather than as the first words of the sentence.
- **FILE-2**  As a developer, the machine-facing part of a file is its frontmatter, and nothing below it exists to serve the tool.
- **FILE-3**  As a developer, when hi writes a line, it looks like something I would have typed by hand.
- **FILE-4**  As a developer, hi never rewrites, reflows, or reformats prose that I wrote.
  - **FILE-4.a**  As a reader, after hi adds a criterion, the rest of the file comes back byte for byte identical, apart from the frontmatter line that names the families.
- **FILE-5**  As a developer, one file covers one feature, and it can hold several id families.
- **FILE-6**  As a maintainer, one criterion is one line, however long the sentence runs, so I can grep it and diff it.
- **FILE-7**  As a developer, hi understands my frontmatter whether I list the families inline or one per line, and when it adds a family it keeps the style I chose.
- **FILE-8**  As a developer, if a write fails partway through, my file is left exactly as it was.
- **FILE-9**  As a developer, a fenced code block in my prose is prose, so I can show an example of the format without it becoming real criteria.
- **FILE-10**  As a developer, if my file uses Windows line endings, hi writes them back the same way.
- **FILE-11**  As a developer, a byte-order mark from my editor does not make a valid file look broken.
- **FILE-12**  As a reader, the paths hi prints or exports use forward slashes on every platform, so a link works wherever it is read.
- **FILE-13**  As a developer, a file starts with my own words about why this feature exists, so the first thing anyone reads is the why and not the list.
- **FILE-14**  As a developer, I can type a criterion into the file by hand, bullet or no bullet, bold or plain, and hi still reads it as one.
- **FILE-15**  As a developer, a criterion I changed my mind about stays in the file under Retired, with my reason beside it, so the file remembers what we dropped.
- **FILE-16**  As a reader, I can tell who each criterion speaks for, because every one names the role it is written in.
- **FILE-17**  As a developer, if I cannot put a role in front of a sentence, I learn while typing that I wrote a fact rather than a want.
- **FILE-18**  As a reader, I understand a criterion as what this should be, not as a report of what it currently does.

- **ID-1**  As a reader, an id never moves once written, so I can say it out loud a year later and still mean the same line.
  - **ID-1.a**  As a developer, inserting a criterion never renumbers anything around it.
  - **ID-1.b**  As a maintainer, a retired id stays reserved forever and is never handed out again.
  - **ID-1.c**  As a developer, a zero-padded number is refused, because SEND-007 and SEND-7 must never be two names for one line.
- **ID-2**  As a developer, I choose the id myself, because I am the one who has to say it.
  - **ID-2.a**  As a reader, a family name is in capitals, like SEND or BILLING or SEND_2FA, so an id stands out as a name in the middle of a sentence.
- **ID-3**  As a reader, an id tells me whether it is a case or a step.
  - **ID-3.a**  As a reader, a letter is another case of its parent, like SEND-1.a and SEND-1.b under SEND-1.
  - **ID-3.b**  As a reader, a number is a step or a detail inside its parent, like SEND-1.a.1 inside SEND-1.a.
- **ID-4**  As a developer, if I write an id that breaks the alternation, hi tells me instead of quietly accepting it.
