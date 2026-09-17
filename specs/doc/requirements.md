---
spec: doc.spec.md
---

## User Stories

- As someone writing down intent, I want my hi file to read as an ordinary markdown document so that
  it is still useful to a person who has never installed the binary (hi: FILE-1, FILE-1.a)
- As the author of the prose in a hi file, I want hi to leave my words exactly as I wrote them, so
  that adding one criterion never produces a diff full of reflowed paragraphs (hi: FILE-4, FILE-4.a)
- As someone capturing a criterion from the terminal, I want the line hi writes to look like a line I
  would have typed by hand, so that the file stays uniform whoever wrote which line (hi: FILE-3)
- As someone who greps and diffs the criteria section, I want one criterion to be one line however
  long the sentence runs (hi: FILE-6)
- As someone who reads the file on GitHub or in a preview rather than in an editor, I want the
  criteria to render as a nested list rather than as one paragraph of run-together sentences
  (hi: FILE-1.b)
- As someone who says ids out loud a year later, I want inserting a criterion to renumber nothing
  around it (hi: ID-1.a)
- As someone whose feature spans several id families, I want one file to hold all of them, each in
  its own readable block (hi: FILE-5)
- As someone documenting the format inside my own intent, I want a fenced code block to stay prose so
  that an example never becomes a real criterion, whether hi is reading my file or writing into it
  (hi: FILE-9, FILE-22.b)
- As someone who is told a criterion was saved, I want it to really be somewhere hi can find again,
  and a refusal rather than a cheerful message when it is not (hi: FILE-22)
- As someone whose file is half written, with a fence I have not closed yet, I want hi to refuse
  rather than save into the middle of my unfinished prose (hi: FILE-22.a)
- As someone whose editor writes YAML its own way, I want hi to read my frontmatter in whichever
  style I wrote it and write it back the same way (hi: FILE-7)
- As someone on Windows, or whose editor adds a byte-order mark, I want my file to work and to come
  back shaped the way it started (hi: FILE-10, FILE-11)
- As someone whose disk can fill up, I want a write that fails partway to leave my file exactly as it
  was (hi: FILE-8)
- As someone capturing into a file I started by hand, I want hi to make the criteria section rather
  than appending wherever my file happens to end (hi: CAPTURE-7)
- As the author of `hi check`, I want a line that is shaped like an id but is not a valid one handed
  to me with its file and line rather than silently skipped, so that I can report it (hi: CHECK-2.d,
  CHECK-3)
- As the author of `hi check`, I also want a criterion that sits outside every section handed to me,
  because nothing reads it where it is (hi: CHECK-2.e)
- As the author of `hi export` and `hi issue`, I want the intent prose, the active criteria and the
  retired criteria available as structured data from one parse (hi: EXPORT-1.a)

## Constraints

- No YAML library. Frontmatter is hand-parsed: `key: value` per line, unknown keys ignored,
  `families` in either of the two styles a person actually writes, whether inline (`families: [A, B]`,
  or `family: A`) or a block list (`families:` followed by indented `- A` lines). Which style the file
  used is remembered, because hi has to write it back the same way (hi: FILE-7).
- The parse is infallible. `Doc::parse` returns a `Doc` for any input, including one with no
  frontmatter, no sections and no valid ids. The only fallible operations are the two filesystem
  calls, the missing-frontmatter refusal inside `insert`, and the read-back refusal of REQ-doc-020.
- No write site decides for itself where a section is. Anything that adds a `## Criteria` or
  `## Retired` heading, or looks for one, goes through the same fence state machine the parser uses,
  and then proves the result by parsing it (REQ-doc-020). A second raw-line scan for a heading is
  how both of the bugs that requirement exists for were written.
- The original lines are the source of truth for serialization. Any future feature that needs to
  rewrite content must do it as a line splice or an in-place single-line replacement, or invariant 1
  of the spec is lost.
- One criterion is one line (hi: FILE-6). There is no wrap width, and adding one would be a format
  decision, not an implementation detail: a wrapped criteria section is harder to grep, harder to
  diff and harder to read as a list. The parser still accepts hand-wrapped criteria, so the
  asymmetry is deliberate.
- Reading is offline and local: this module opens only the path it was given (hi: CHECK-4).
- `Criterion.line`, `Criterion.end_line` and the line index in each `stray` entry are 0-based indices
  into `lines`; `line_no()` is the only 1-based number and exists solely for human-facing messages.
- Markdown structure is only ever read outside a fenced code block. Any future parse rule added to
  `parse_body` has to sit below the fence check or REQ-doc-013 is lost.

## Out of Scope

- Validating ids. Shape, alternation and parse errors belong to `id`; duplicates, missing parents and
  retired-id collisions belong to `check` (hi: CHECK-2.a, CHECK-2.b, CHECK-2.c).
- Deciding whether a capture is permitted, or what the next free number is. That is `capture` and
  `workspace`.
- Finding the `hi/` directory or enumerating which files exist. That is `workspace`.
- Rendering anything for a human or a machine consumer. Listings, issue bodies, the JSON export and
  the `INTENT.md` index are all `out`.
- Any notion of state, lifecycle, evidence or completion. A hi file records intent and identity only.
- Judging, linting or rewriting the English of a sentence.
- Interpreting the inline markdown a sentence may contain. The text is stored and written verbatim;
  rendering it is `hi view`'s business.

### REQ-doc-001

The module SHALL parse a `hi/*.md` file into its frontmatter, title, intent prose, active criteria
and retired criteria without requiring any markup that is not ordinary markdown, beyond the single
`hi:` version line in the frontmatter.

Acceptance Criteria

- `hi:` yields `front.version`, `families: [A, B]` yields `front.families` in declaration order with
  surrounding quotes stripped, and `owner:` yields `front.owner` (hi: FILE-2).
- A `families:` key with an empty value is read as a block list: each following indented `- name`
  line, quotes stripped, appends a family, and the run of them is recorded as the entry's span.
- A singular `family:` key is accepted and appends one family, so a one-family file can say so.
- Unrecognized frontmatter keys are ignored and their lines are left untouched.
- `# Title` sets `title` from the first level-one heading only; `## Intent`, `## Criteria` and
  `## Retired` are matched case-insensitively.
- A second `# ` heading does not replace the title, but it does close whichever section is open, and
  records the append point when the section was `## Criteria`, so criteria written below it are not
  parsed as criteria until the next `## Criteria` or `## Retired` heading. They are recorded as
  strays instead (REQ-doc-014).
- A leading UTF-8 BOM is stripped before any of this, so an editor's BOM cannot hide the frontmatter
  (hi: FILE-11).
- `intent` is the prose of the `## Intent` section with leading and trailing blank lines trimmed and
  interior line breaks preserved.
- A file with no frontmatter, or with an unterminated `---` fence, parses as body-only rather than
  failing (hi: FILE-1.a).

### REQ-doc-002

Serializing a document the module has not modified SHALL reproduce the original text byte for byte.

Acceptance Criteria

- Every original line is retained verbatim in `lines`; `to_text()` is a join, not a re-render
  (hi: FILE-4).
- Indentation, blank lines, heading text and unrecognized content all survive a parse-and-serialize
  round trip.
- The trailing-newline shape is preserved, with one stated exception: content that did not end in a
  newline gains one, because a non-empty hi file always ends in a newline. An empty file stays empty.
- The line ending is preserved (REQ-doc-016), and the two remaining exceptions are stated there and
  in REQ-doc-015: a file of mixed endings comes back in whichever ending was in the majority, and a
  leading BOM is dropped rather than written back.

### REQ-doc-003

A criterion SHALL be recognized as a line, at any indent, whose first whitespace-delimited token is
id-shaped once an optional markdown bullet and any surrounding emphasis are stripped, extended by
the indented non-blank lines that immediately follow it and are not themselves criteria.

Acceptance Criteria

- A leading `- `, `* ` or `+ ` bullet is stripped before the first token is taken, and `*` or `_`
  characters around that token are stripped before it is tested and before it is stored as `raw_id`.
  `- **SEND-1**  x`, `- SEND-1  x`, `  - _SEND-1_  x` and `SEND-1  x` are the same criterion.
- Indentation does not disqualify a line, because hi writes a case indented two spaces under its
  parent (REQ-doc-019). A file whose criteria are a nested list parses as the same criteria as the
  same file written flat.
- Continuation lines are joined into `text` with exactly one space between them, so a wrapped
  sentence reads as one sentence.
- The first blank line, the first unindented line, and the first indented line that is itself
  criterion-shaped all end the criterion, and `end_line` records its last line. Without that third
  rule a nested case would be swallowed as its parent's continuation.
- Prose sitting inside `## Criteria` that does not begin with an id-shaped token is not a criterion
  and is preserved in `lines`. `looks_like_id` requires the first level after the hyphen to be a
  digit, so an ordinary hyphenated word such as `spec-sync` or `well-formed` stays prose.
- A lowercase family is id-shaped, because `looks_like_id` is case-insensitive there. The line
  becomes a criterion whose `Id::parse` failed, so `check` can report it rather than the line reading
  as prose and vanishing (hi: CHECK-2.d).
- Both a leading space and a leading tab count as indentation.
- A line inside a fenced code block is never a criterion, whatever it is shaped like (REQ-doc-013).

### REQ-doc-004

A token that is id-shaped but not a valid id SHALL be recorded as a criterion carrying its parse
error, and SHALL NOT be skipped (hi: CHECK-2.d, ID-4).

Acceptance Criteria

- The criterion has `id == None` and `id_error == Some(..)`, and `raw_id` is the token as written
  with its bullet and emphasis stripped, so the message quotes back the id rather than its markdown
  decoration.
- `line`/`line_no()` point at the real line, so a problem can name the file and the line
  (hi: CHECK-3).
- Parsing continues past the bad line; one malformed id does not cost the rest of the file.
- `used_families()` skips criteria whose id did not parse rather than inventing a family.

### REQ-doc-005

The module SHALL keep active and retired criteria apart, and SHALL treat a `retired:` continuation as
a note rather than part of the sentence.

Acceptance Criteria

- Criteria under `## Criteria` land in `criteria` with `section == Section::Criteria`; those under
  `## Retired` land in `retired` with `section == Section::Retired`.
- A continuation beginning `retired:` sets `note` to the trimmed remainder and contributes nothing to
  `text`.
- `all()` yields active criteria first, then retired, and every `Criterion` carries its `section`, so
  a consumer that wants both still knows which is which. Acting on that distinction belongs to `out`
  (hi: ISSUE-4) and `check` (hi: ID-1.b), not here.

### REQ-doc-006

Inserting a criterion SHALL splice new lines into the file and SHALL NOT alter, reflow or renumber
any existing line (hi: FILE-4.a, ID-1.a).

Acceptance Criteria

- After an insert, the intent prose, headings, blank lines and every pre-existing criterion (down to
  the exact indentation of a `retired:` continuation) are still present character for character.
- No existing id is changed, and no criterion is reordered.
- Every recorded line index (each criterion's `line` and `end_line`, each `stray` entry, plus the
  internal section markers) is shifted to stay consistent with the new `lines`, whether the lines
  were added by the criterion splice, by a created `## Criteria` section or by a frontmatter rewrite
  that changed the number of lines above them.

### REQ-doc-007

A new criterion SHALL be placed inside the block it belongs to: beneath its parent's existing
descendants when it is a case, otherwise at the end of its own family's block, with a new family
opening a block of its own (hi: CAPTURE-4, FILE-5).

Acceptance Criteria

- A case is inserted immediately after the last existing criterion that is its parent or a descendant
  of its parent, and therefore before the next family. The parent itself counts, so the first case of
  a parent that has none yet lands on the line directly below it.
- A criterion with no parent present is inserted after the last criterion of the same family.
- A family the `## Criteria` section has not yet seen is appended after the section's last content
  line, preceded by exactly one blank line. The parser records that append point whenever it leaves
  the section, whether on a `## ` heading, on a level-one `# ` heading, or at the end of the file, so
  the rule holds for all three shapes.
- Into an empty `## Criteria` section the criterion is written directly under the heading, keeping
  the heading's existing blank line and not adding a second.
- No blank line is ever introduced inside a family's block.

### REQ-doc-008

A criterion the module writes SHALL carry the whole sentence on exactly one line, and SHALL NOT be
wrapped at any width (hi: FILE-6, FILE-3).

Acceptance Criteria

- `render_criterion` returns a one-element vector for any input, however long the sentence.
- The line is `{indent}- **{id}**  {sentence}` with exactly two spaces between the id and the
  sentence. The bullet, the emphasis and the indent are REQ-doc-019's business; the one-line rule is
  this requirement's.
- Interior whitespace in the supplied sentence, including newlines from a pasted multi-line thought,
  is collapsed to single spaces so the result is one sentence.
- A whitespace-only sentence still renders one line rather than an empty vector; refusing it belongs
  to `capture`.
- A criterion a person wrapped by hand is still parsed as one criterion, so reading and writing
  differ here on purpose (see REQ-doc-003).

### REQ-doc-009

Inserting a criterion whose family the frontmatter does not declare SHALL add that family to the
`families:` list in place (hi: FILE-5).

Acceptance Criteria

- The recorded span of the `families:` or `family:` entry is replaced, preserving declaration order
  and appending the new family last, in the style the file already used: `families: [A, B, C]` for a
  file that wrote it inline, or `families:` plus one `  - A` line per family for a file that wrote a
  block list (hi: FILE-7). A file that declared `family: SEND` counts as inline and therefore comes
  back saying `families: [SEND, ...]`.
- A frontmatter with neither key gains one inline `families:` line just inside the closing fence.
- Whenever the replacement is a different number of lines than the entry it replaced, every recorded
  index below it (criteria, strays and the section markers) and the frontmatter's own closing line
  are shifted by the difference.
- A family already declared triggers no rewrite at all, so a repeat capture produces no frontmatter
  diff.
- A file with no frontmatter at all is refused with a message naming the file and the three lines to
  add.

### REQ-doc-010

The module SHALL provide the starting text for a new feature file, and that text SHALL parse as a
valid, empty hi document (hi: CAPTURE-2.a, FILE-1).

Acceptance Criteria

- The scaffold contains `hi: 1`, `families: [{family}]`, a `# {title}` heading, an `## Intent`
  section holding one HTML-comment prompt written for a person, and an empty `## Criteria` section.
- Parsing the scaffold yields `version == Some(1)`, the single declared family, and no criteria.
- Inserting into the freshly scaffolded document lands the first criterion under `## Criteria`
  without further setup (hi: CAPTURE-1.a).

### REQ-doc-011

The module SHALL make no change to any file on disk except through `save`, so that a caller which
refuses part-way through leaves the file untouched (hi: CAPTURE-5).

Acceptance Criteria

- `insert` mutates only the in-memory `lines`, `front` and criterion indices.
- `new_file_text` returns a string; writing it is the caller's decision.
- `save` is the module's only write, and it writes exactly `to_text()` to the document's own `path`.
- Every refusal from `insert`, `retire` and `set_retired_reason` restores the in-memory document to
  what it was before the call, so a caller holding one `Doc` for a whole run cannot save a
  half-spliced buffer. Nothing has reached disk at that point either way.

### REQ-doc-012

The module SHALL read a `families` entry written either inline or as a YAML block list, and SHALL
write it back in whichever of the two styles the file already used (hi: FILE-7).

Acceptance Criteria

- `families: [A, B]`, `families: A, B` and `family: A` are all read as an inline entry; `families:`
  with an empty value followed by indented `- A` lines is read as a block entry.
- A block item counts only while it is indented and starts with `-`; the first line that is not ends
  the list, so `owner:` on the next line is still parsed as `owner`.
- `front.families_block` records which style was found, and `front.families_span` records the
  inclusive line span of the whole entry, block items included.
- A rewrite replaces exactly that span and emits the same style, so a block-style file is never
  collapsed to the inline form and an inline file never becomes a block.
- Reading the two styles produces identical `front.families`, so nothing downstream has to branch on
  style.

### REQ-doc-013

The module SHALL treat a fenced code block in the body as prose, and SHALL NOT read any structure
from the lines inside it (hi: FILE-9).

Acceptance Criteria

- A body line whose trimmed form begins with three or more backticks or three or more tildes opens a
  fence; a later run of the same marker, at least as long as the opening run, closes it.
- Between the fences, a `# ` or `## ` line sets no title, opens no section and closes none; an
  id-shaped line becomes neither a criterion nor a stray.
- A fence inside `## Intent` is kept in `intent`, along with everything between its markers, and prose
  written after the fence closes is still intent, so a `# ` comment inside a shell snippet does not
  truncate the section.
- Fence state carries across the whole body, so an unclosed fence makes everything after it opaque.
- The write path reads the same map. `retired_heading` and `retired_end` skip fenced lines, so a
  `## Retired` drawn inside somebody's example is never the section a retirement moves into, and the
  example comes back byte-identical (hi: FILE-22.b).
- `fence_marker` is the one definition of what opens or closes a fence, and it is public so that
  every part of hi that reads markdown gives the same answer. `out::unwrap_soft_breaks` uses it to
  leave the newlines inside a fenced example alone when it renders intent prose into a ticket
  (hi: ISSUE-7.b). It reports the marker and the run length only; the open/close state machine
  belongs to each caller.

### REQ-doc-014

The module SHALL record a criterion-shaped line that sits outside every section, so that it can be
reported rather than silently ignored (hi: CHECK-2.e).

Acceptance Criteria

- A non-blank line outside `## Criteria` and `## Retired`, at any indent, that satisfies
  REQ-doc-003's criterion test is pushed onto `doc.stray` as `(0-based line index, token)`. This
  covers the lines before the first heading, under a `# ` heading, and under a `## ` heading the
  parser does not recognize. Indentation does not exempt a line, because a stray is just as
  invisible indented as it is flush.
- The recorded token is stripped of its bullet but not of its emphasis, because the stray branch
  calls `strip_bullet` alone while `read_criterion` also calls `strip_emphasis`. A stray written
  `  - **SEND-9**  ...` is therefore recorded, and reported by `check`, as `**SEND-9**`.
- A line inside `## Intent` is *not* covered: the intent branch of `parse_body` runs above the stray
  branch, so an id-shaped line written there becomes intent prose and is reported by nothing. The
  requirement is not met in that one region; whether it should be is an open decision recorded in
  `tasks.md`.
- Such a line produces no `Criterion`: it is not in `criteria`, not in `retired`, and not in `all()`.
- Its recorded index is shifted along with everything else when `insert` splices lines above it, so a
  later report still names the right line (hi: CHECK-3).
- A line inside a fenced block is not a stray (REQ-doc-013).

### REQ-doc-015

The module SHALL strip a leading UTF-8 byte-order mark before parsing (hi: FILE-11).

Acceptance Criteria

- A file whose first bytes are `\u{feff}---` is parsed as having frontmatter; the version, families,
  owner and criteria are all read normally.
- The BOM is not carried into `lines` and is not written back by `to_text`, which is the third stated
  exception to REQ-doc-002.

### REQ-doc-016

The module SHALL write a file back with the line ending that file already used (hi: FILE-10).

Acceptance Criteria

- The ending is chosen once at parse time: CRLF when `\r\n` occurrences outnumber bare `\n`
  occurrences, otherwise LF. A file of mixed endings therefore comes back entirely in the majority
  ending, ties going to LF.
- `to_text` joins the lines with that ending and terminates the text with it, so hi introduces no
  bare LF into a Windows file and no CR into a Unix one.
- A criterion inserted into a CRLF file is written with a CRLF ending like every other line.

### REQ-doc-017

`save` SHALL NOT leave the target file truncated or partly written if the write fails (hi: FILE-8).

Acceptance Criteria

- The text is written to a sibling temp file named `.{file name}.hi-tmp` in the target's own
  directory, flushed and synced, and only then renamed over the target.
- A failure to create, write, flush or sync the temp file removes the temp file and returns the
  error, having never opened the target.
- A failure of the rename also removes the temp file and returns the error.
- A successful save leaves no temp file behind.

### REQ-doc-018

Inserting into a file with no `## Criteria` heading SHALL create that section rather than appending
wherever the file happens to end (hi: CAPTURE-7).

Acceptance Criteria

- A blank line, `## Criteria` and a blank line are spliced in after the file's last line with
  content, and every recorded index at or after that point is shifted.
- The criterion then lands under the new heading, keeping the heading's blank line and adding no
  second one, exactly as it would in an existing empty section (REQ-doc-007).
- Everything already in the file, intent prose included, is left character for character
  (REQ-doc-006), and re-parsing the result finds the same intent, the new criterion and no stray.
- When that last content line is inside a fence nobody closed, the created heading would be part of
  the example, so the insert is refused by REQ-doc-020 and the unfinished prose is left alone. An
  unfinished document is an ordinary thing to have; reporting a successful capture into one is not.

### REQ-doc-020

Every write SHALL parse the buffer it is about to save and SHALL refuse it unless the edit is
readable where the verb said it would be (hi: FILE-22).

Acceptance Criteria

- `insert`, `retire` and `set_retired_reason` each parse their proposed text before returning `Ok`.
  The check is: every id the verb named is present in the named section (`## Criteria` for `insert`,
  `## Retired` for the other two, and for every case that went with a retirement); every `raw_id` the
  document made readable before the call is still readable, compared as a multiset so that losing one
  of a duplicated pair counts; and `stray` has not grown.
- A refusal restores the document to its pre-call state and returns an error naming the verb, the id
  and the file. Nothing is written, in memory or on disk (hi: CAPTURE-5).
- When the proposed text contains a fence that is never closed, the refusal adds a hint naming the
  1-based line it opens on and the marker it was opened with (``` or ~~~), because a swallowed
  section is the likeliest cause.
- `insert` discards the verified parse rather than adopting it, so its line-index bookkeeping remains
  the contract with `rewrite_families` and the next insert, and `capture` still reloads the file it
  saved before anything counts (DECISIONS.md §30).
- The check is over what the parser reads, not over the lines that were spliced: a criterion written
  into a fenced region is unreadable however plainly it sits on the page (REQ-doc-013).

### REQ-doc-019

`render_criterion` SHALL emit each criterion as a markdown list item whose id is bold, indented two
spaces per depth level below the first, so that a hi file renders as a nested list rather than a
paragraph of run-together sentences (hi: FILE-1.b).

Acceptance Criteria

- A depth-1 criterion renders as `- **ID**  sentence` at column 0.
- A depth-2 criterion renders as `  - **ID**  sentence`, a depth-3 as `    - **ID**  sentence`, and
  so on. The indent is `"  "` repeated `id.depth() - 1` times.
- The id is wrapped in `**` so it reads as a label rather than as the first two words of the
  sentence wherever the file is rendered.
- The result is still exactly one line however long the sentence runs (hi: FILE-6).
- The parser accepts a criterion written without a bullet, without emphasis, and at any indentation,
  so a file written before this rule or edited by hand is never rejected (REQ-doc-003).
- A bullet may be `-`, `*` or `+` on input; hi always writes `-`. Emphasis may be `*` or `_` on
  input; hi always writes `**`.
