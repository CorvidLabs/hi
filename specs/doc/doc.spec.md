---
module: doc
version: 1
status: active
files:
  - src/doc.rs

db_tables: []
depends_on:
  - specs/id/id.spec.md
---

# Doc

## Purpose

Reads and writes `hi/*.md`. This module owns the file format: the frontmatter block, the `# Title`,
`## Intent`, `## Criteria` and `## Retired` sections, the criterion line grammar (a markdown list
item whose first token is an id, indented two spaces per depth level, optionally extended by
indented continuations), and the surgical insertion that splices new lines into a file while leaving
every line it did not write byte-identical.

A hi file is markdown a person could have typed by hand, and must still read as a document with no
tool installed (hi: FILE-1, FILE-1.a). Because every markdown renderer joins consecutive plain lines
into one paragraph, a criterion hi writes is a list item rather than a bare line, so the criteria
render as a nested list instead of a wall of run-together sentences (hi: FILE-1.b). The parser is
looser than the writer: it strips an optional bullet and any emphasis around the id, and accepts a
criterion at any indent, so a file written by hand or written before this rule is never rejected.
The parser keeps the original lines rather than an abstract tree, so serialization is a join rather
than a re-render, and hi never reflows prose it did not write (hi: FILE-4, FILE-4.a). Because the
file is markdown, a fenced code block in it is prose:
the parser reads nothing structural between fences, so a person can show an example of the format
inside their own intent (hi: FILE-9). The file's own byte shape (its line ending, its trailing
newline, its frontmatter style) is carried through a write rather than normalized (hi: FILE-7,
FILE-10), and the write itself is atomic so a failure cannot leave a truncated file (hi: FILE-8).

This module is pure structure. It does not validate ids beyond recording why one failed to parse,
does not decide whether a capture is allowed, does not enumerate files on disk, and does not render
any output for a human. Those belong to `id`, `capture`, `workspace`, and `out`/`view` respectively.

## Public API

| Export | Description |
|--------|-------------|
| `Section` | Which section of the file a criterion was found in: `Criteria` or `Retired`. |
| `Criterion` | One criterion, holding the parsed id (or the parse error), raw id text, sentence, optional `retired:` note, its line range, and its section. |
| `line_no` | `Criterion` method returning the 1-based line number of the criterion's id line, for human-facing messages. |
| `Front` | The hand-parsed frontmatter: `hi:` version, declared `families`, `owner`, the inclusive line range of the `---` block, the inclusive line span of the `families` entry, and whether that entry was written as a YAML block list. |
| `Doc` | One parsed `hi/*.md`: path, frontmatter, title, intent prose, active and retired criteria, criterion-shaped lines stranded outside every section (`## Intent` excepted, where such a line stays prose), and every original line. |
| `load` | `Doc` constructor that reads a path from disk and parses it. |
| `parse` | `Doc` constructor over already-loaded text, so callers and tests need no filesystem. Strips a leading BOM and remembers the file's line ending. |
| `all` | Iterator over every criterion in the file, active criteria first, then retired. |
| `used_families` | The families actually used by criteria, in first-seen order, ignoring what the frontmatter declares. |
| `insert` | Splices a rendered criterion into the `## Criteria` section at the right place, creating that section when the file has none, and declares its family in the frontmatter if it is new. |
| `to_text` | Serializes the line buffer back to a string, joining with the file's own line ending and preserving the original trailing-newline shape. |
| `save` | Writes `to_text()` back to the document's own path, atomically: a sibling temp file, flushed and synced, then renamed over the target. |
| `name` | The file stem, used as the document's display name (`hi/chat.md` is `chat`). |
| `render_criterion` | Renders an id and a sentence as one markdown list item, `<indent>- **ID**<2 spaces>sentence`, indented two spaces per depth level, with the id in bold and all interior whitespace collapsed to single spaces. Never wraps. |
| `write_atomically` | Write a string to a path without ever leaving the target truncated: sibling temp file, flush, fsync, rename. Public so `out::write_index` can give `INTENT.md` the same protection `Doc::save` gives `hi/*.md` (hi: FILE-8). |
| `new_file_text` | The starting text for a brand-new feature file, already parseable as an empty hi document. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Section` | `enum { Criteria, Retired }`. `Copy` + `Eq`, so callers filter on it directly. |
| `Criterion` | Fields: `id: Option<Id>`, `raw_id: String`, `id_error: Option<IdError>`, `text: String`, `note: Option<String>`, `line: usize` (0-based), `end_line: usize` (0-based, inclusive), `section: Section`. `id` is `None` exactly when `id_error` is `Some`. |
| `Front` | Fields: `version: Option<u32>`, `families: Vec<String>`, `owner: Option<String>`, `range: Option<(usize, usize)>` (indices of the opening and closing `---`), `families_span: Option<(usize, usize)>` (inclusive line span of the `families`/`family` entry, covering every `- item` line of a block list), `families_block: bool` (true when that entry was written as a YAML block list). `Default` yields an empty frontmatter with `range: None`, `families_span: None` and `families_block: false`. |
| `Doc` | Public fields: `path: PathBuf`, `front: Front`, `title: Option<String>`, `intent: String`, `criteria: Vec<Criterion>`, `retired: Vec<Criterion>`, `lines: Vec<String>`, `stray: Vec<(usize, String)>` (0-based line index and the id-shaped token, for criterion-shaped lines found outside every section; see invariant 13 for the `## Intent` exception). Four private fields track where new criteria append (`criteria_end`), where the `## Criteria` heading sits (`criteria_heading`), whether the file ended with a newline (`trailing_newline`), and which line ending it uses (`newline`, `"\n"` or `"\r\n"`). |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines no traits; it derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` and `Default` where noted above. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `load` | `Doc::load(path: &Path) -> Result<Doc>` | Reads the file and parses it. Fails only when the read fails, with the path in the context. |
| `parse` | `Doc::parse(path: PathBuf, raw: &str) -> Doc` | Infallible. Strips a leading UTF-8 BOM, records whether the text ended in a newline, picks the line ending the file uses (CRLF when `\r\n` outnumbers bare `\n`, otherwise LF), splits `raw` into lines, parses frontmatter, then the body. Malformed input becomes recorded structure, never an error. |
| `line_no` | `Criterion::line_no(&self) -> usize` | `self.line + 1`. |
| `all` | `Doc::all(&self) -> impl Iterator<Item = &Criterion>` | `criteria.iter().chain(retired.iter())`. |
| `used_families` | `Doc::used_families(&self) -> Vec<String>` | Deduplicated families of every criterion whose id parsed, in document order. Criteria with unparseable ids contribute nothing. |
| `insert` | `Doc::insert(&mut self, id: &Id, text: &str) -> Result<()>` | Creates a `## Criteria` section when the file has none, chooses an insertion point, renders the criterion, splices it into `lines`, shifts every recorded line index at or after the splice (criteria, retired, `criteria_end`, `criteria_heading` and `stray`), and appends the family to the frontmatter if it is not already declared. In-memory only. |
| `to_text` | `Doc::to_text(&self) -> String` | `lines.join(self.newline)` plus one `self.newline` whenever the result is non-empty or the original had one. |
| `save` | `Doc::save(&self) -> Result<()>` | `write_atomically(self.path, self.to_text())`, with the path in the error context. The only function in this module that writes to disk. |
| `name` | `Doc::name(&self) -> String` | The path's file stem, falling back to the whole displayed path when there is no stem. |
| `render_criterion` | `render_criterion(id: &Id, text: &str) -> Vec<String>` | Returns exactly one line, `{indent}- **{id}**  {sentence}`, where `indent` is `"  "` repeated `id.depth() - 1` times and `sentence` is `text` split on whitespace and rejoined with single spaces. The bullet makes the criteria render as a list and the bold makes the id read as a label rather than the first two words of the sentence. The `Vec` return exists because `insert` splices a slice of lines, not because more than one is ever produced. |
| `new_file_text` | `new_file_text(title: &str, family: &str) -> String` | Returns frontmatter (`hi: 1`, `families: [{family}]`), `# {title}`, an `## Intent` section holding one HTML-comment prompt, and an empty `## Criteria` section. |

## Invariants

1. **A file hi did not change comes back byte for byte** (hi: FILE-4, FILE-4.a). Parsing stores the
   original `lines` verbatim and `to_text()` joins them with the line ending the file itself used;
   nothing in the parse path rewrites a line. Three documented exceptions, all of them normalizing
   toward "what a hi file looks like": content that did not end in a newline gains one, because a
   non-empty document always ends in a newline; a leading UTF-8 BOM is dropped rather than written
   back; and a file with mixed line endings comes back entirely in the ending that was in the
   majority, ties going to LF. A wholly empty file stays empty.
2. **Insertion is a splice, never a rewrite** (hi: ID-1.a). `insert` only ever calls
   `lines.splice(..)`, once for the rendered criterion, once more for a `## Criteria` section when
   the file has none, and once more to replace the `families` entry when the family is new. No
   existing criterion line, prose line, blank line or heading is touched, and no id is renumbered.
3. **A criterion is a line whose first token is id-shaped, inside a section and outside a fence**
   (invariants 12 and 13 cover the two exclusions). The line is trimmed, a leading `- `, `* ` or
   `+ ` bullet is dropped, and any `*` or `_` around the first whitespace-delimited token is dropped
   before `looks_like_id` decides. Indentation does not disqualify a line, because a case is written
   indented under its parent. A line that is blank, or whose first token fails `looks_like_id` after
   that stripping, is not a criterion, so prose may sit inside `## Criteria` without becoming one:
   `looks_like_id` requires a first level that is a digit, so an ordinary hyphenated word like
   `spec-sync` or `well-formed` stays prose. `raw_id` is stored stripped, so `**SEND-1**` and
   `SEND-1` are the same criterion.
4. **An id-shaped token that will not parse is recorded, not dropped** (hi: CHECK-2.d, ID-4). The
   `Criterion` carries `id: None`, `id_error: Some(..)` and `raw_id`, the token as written with its
   bullet and emphasis stripped, so `check` can quote it back against a file and a line.
   `id.is_some()` and `id_error.is_none()` are exact complements.
5. **Continuations belong to the criterion above them.** A line that is indented, non-blank, and
   not itself criterion-shaped continues the criterion; the first blank line, the first unindented
   line, and the first indented line that is itself criterion-shaped all end it. That last exclusion
   is what makes a nested case its own criterion rather than a continuation of its parent.
   Continuations join the sentence with a single space, except a continuation beginning `retired:`
   which becomes `note` and is excluded from `text`. Continuations are read but never written: hi
   accepts a criterion a person wrapped by hand, and emits one line per criterion itself
   (invariant 11).
6. **A family's criteria stay in one contiguous block, and a case sits under its parent**
   (hi: CAPTURE-4, FILE-5). `insert` prefers the line after the lowest criterion in the file that is
   either the id's parent itself or a descendant of that parent (so a first case lands directly under
   its parent), then the line after the last criterion of the same family, and only opens a new
   blank-line-separated block for a family the `## Criteria` section has not seen. That last branch
   needs a recorded `criteria_end`, which the parser sets whenever it leaves the section (on a
   `## ` heading, on a level-one `# ` heading, or at the end of the file), so a new family appends
   below the existing block in every one of those shapes. Only `insert` consults `self.criteria`;
   retired criteria are never an insertion target.
7. **Every line index in a `Doc` stays consistent with `lines` across an insert.** `insert`, the
   created `## Criteria` section and the frontmatter rewrite shift `line`, `end_line`,
   `criteria_end`, `criteria_heading` and every `stray` index by the number of lines added at or
   before them.
8. **The frontmatter declares every family the file holds, in the style the file already used**
   (hi: FILE-5, FILE-7). After `insert` returns `Ok`, `front.families` contains `id.family`. When the
   family was not already declared, the recorded `families_span` is replaced (by one
   `families: [A, B]` line when the file wrote the entry inline, or by a `families:` line followed by
   one `  - A` line per family when it wrote a block list), and a frontmatter with no families entry
   at all gains an inline one just inside the closing fence. When the family was already declared
   nothing is rewritten, so a file that says `family: SEND` keeps saying exactly that.
9. **Only the machine-readable version line exists for the tool** (hi: FILE-2). Frontmatter carries
   `hi`, `families`, `family` and `owner`; every other key is ignored and preserved untouched.
10. **Nothing reaches disk without `save`.** `insert` mutates only the in-memory buffer, and
    `new_file_text` returns text rather than writing it, so a caller that refuses part-way through
    leaves the file on disk exactly as it was (hi: CAPTURE-5).
11. **One criterion hi writes is exactly one markdown list item** (hi: FILE-6, FILE-3, FILE-1.b).
    Two spaces of indent per depth level below the first, then `- `, then the id in `**bold**`, then
    two spaces, then the whole sentence on that line however long, with interior whitespace
    collapsed to single spaces. The bullet is what makes the file render as a nested list instead of
    one run-together paragraph; the criteria section stays greppable, diffable and readable as a
    list, which is the point of the list. `render_criterion` therefore always returns a one-element
    vector, and it is the only place in the module that writes a criterion line.
12. **A fenced code block is opaque to the parser** (hi: FILE-9). A body line whose trimmed form
    starts with three or more backticks or tildes opens a fence, and a run of the same marker at
    least as long closes it. Between the two, nothing is structure: no heading sets the title or
    closes a section, no id-shaped line becomes a criterion, and none becomes a `stray` either.
    Fence lines and their contents still land in `intent` when the fence sits inside `## Intent`, so
    an example of the format shown inside a person's own intent stays prose. Fence tracking spans
    the whole body: an unclosed fence makes the rest of the file opaque.
13. **A criterion-shaped line outside every section is recorded, not dropped** (hi: CHECK-2.e).
    A non-blank line that satisfies invariant 3's criterion test, at any indent, which sits outside a
    fence and under no `## Criteria` or `## Retired` heading, is pushed onto `stray` as
    `(line index, token)` and is parsed no further. The token recorded is bullet-stripped but *not*
    emphasis-stripped, because the stray branch calls `strip_bullet` alone where `read_criterion`
    calls `strip_emphasis` as well: a stray written `  - **SEND-9**  ...` is recorded, and reported
    by `check`, as `**SEND-9**`. Nothing reads it as a criterion, so `check`
    reports it as a `StrayCriterion` rather than letting the line vanish because a heading moved
    above it. One region is excluded, and the exclusion is load-bearing to know about: the intent
    branch of `parse_body` runs above the stray branch, so an id-shaped line written under
    `## Intent` becomes part of `intent` and is neither a criterion nor a stray. Nothing reports it.
    Whether that is right is an open question, recorded in `tasks.md`.
14. **The file's line ending survives a write** (hi: FILE-10). `parse` picks CRLF only when `\r\n`
    outnumbers bare `\n`, and `to_text` joins with that ending, so a Windows file is written back
    with Windows endings and hi introduces no bare LF into it.
15. **A leading byte-order mark never hides the frontmatter** (hi: FILE-11). `parse` strips a
    `\u{feff}` prefix before anything else, so an editor-written BOM cannot push `---` off the first
    line and make a valid file look like it has no frontmatter. The BOM is not written back.
16. **A failed write leaves the file exactly as it was** (hi: FILE-8). `save` writes a sibling temp
    file named `.{file name}.hi-tmp`, flushes and `sync_all`s it, and only then renames it over the
    target; a failure at any step removes the temp file and returns the error, and the original is
    never truncated. A successful save leaves no temp file behind.
17. **`insert` always has a section to land in** (hi: CAPTURE-7). A file with no `## Criteria`
    heading gets a blank line, `## Criteria` and a blank line spliced in after its last line with
    content, before the insertion point is chosen. A criterion is therefore never appended into
    intent prose or below a heading where nothing would read it.

## Behavioral Examples

Several scenarios below are **Given** a bare, unbulleted `SEND-1  ...` line. That is deliberate and
matches the unit tests' `SAMPLE` fixture: it is what the parser must keep accepting from a
hand-edited file, not what hi writes. Every line hi writes is the list item of invariant 11, and the
**Then** clauses quote it in that form.

#### Scenario: Parsing a complete file

- **Given** a file with `hi: 1`, `families: [SEND, RECEIPT]`, `owner: leif`, `# Chat`, an
  `## Intent` paragraph, three criteria and a `## Retired` block
- **When** `Doc::parse` runs
- **Then** `front.version` is `Some(1)`, `front.families` is `["SEND", "RECEIPT"]`, `front.owner` is
  `Some("leif")`, `title` is `Some("Chat")`, `intent` is the prose with surrounding blank lines
  trimmed, `criteria` holds 3 entries and `retired` holds 1

#### Scenario: A wrapped criterion is one criterion

- **Given** `SEND-1  I hit enter and the message shows up` followed by an indented
  `right away, marked as sending.`
- **When** the file is parsed
- **Then** one `Criterion` results, its `text` is the two lines joined by a single space, its
  `line_no()` is the id line, and its `end_line` is the continuation

#### Scenario: Prose inside the criteria section

- **Given** `## Criteria` containing the line `Just a note to self.` above `SEND-1  A real one.`
- **When** the file is parsed
- **Then** exactly one criterion is produced, `SEND-1`, and the note line survives in `lines`
  untouched

#### Scenario: A retired criterion keeps its reason out of the sentence

- **Given** `SEND-3  Messages auto-delete after 24 hours.` under `## Retired`, with an indented
  `retired: different product`
- **When** the file is parsed
- **Then** the criterion lands in `retired` with `section == Section::Retired`, `text` is the
  sentence alone, and `note` is `Some("different product")`

#### Scenario: A malformed id is kept as evidence

- **Given** `SEND-1.a.b  Two letters in a row.`
- **When** the file is parsed
- **Then** one criterion is produced with `id == None`, `id_error == Some(..)` and
  `raw_id == "SEND-1.a.b"`, so `check` can report it against a real line number

#### Scenario: An example of the format inside the intent stays prose

- **Given** an `## Intent` section containing a fenced ```` ```markdown ```` block that itself shows
  a `## Criteria` heading and a `SEND-9` line, and a real `## Criteria` section below it
- **When** the file is parsed
- **Then** only the real criterion is produced, `stray` is empty, and the fence and its contents are
  still part of `intent`; a `~~~` fence and a `# ` comment inside a ```` ```bash ```` block behave
  the same way, and prose written after the fence closes is still intent

#### Scenario: A criterion stranded outside every section

- **Given** `## Criteria` holding `SEND-1  Seen.`, then `# Appendix`, then `SEND-9  Invisible.`
- **When** the file is parsed
- **Then** `criteria` holds only `SEND-1` and `stray` holds `(line of SEND-9, "SEND-9")`, so `check`
  can report the line rather than the criterion silently disappearing

#### Scenario: Block-style frontmatter is read and written back in its own style

- **Given** frontmatter whose families are `families:` followed by `  - SEND`
- **When** the file is parsed and `insert(RECEIPT-1, "Two.")` runs
- **Then** `front.families` is `["SEND"]` with `families_block` true before the insert, and after it
  the frontmatter reads `families:\n  - SEND\n  - RECEIPT`, never collapsed to the inline form,
  while every criterion's recorded `line` still points at its own line despite the block growing

#### Scenario: A file with Windows line endings stays a Windows file

- **Given** a CRLF file holding `SEND-1  One.`
- **When** `insert(SEND-2, "Two.")` runs and the document is serialized
- **Then** the new line is `- **SEND-2**  Two.\r\n` and the text contains no bare LF at all

#### Scenario: A byte-order mark does not hide the frontmatter

- **Given** a file whose first bytes are a BOM followed by `---`
- **When** it is parsed
- **Then** `front.version` is `Some(1)`, `front.families` is read normally, and the criteria parse:
  the file is not treated as having no frontmatter

#### Scenario: A new case lands under its parent

- **Given** a file whose criteria run `SEND-1`, `SEND-1.a`, `RECEIPT-1`
- **When** `insert(SEND-1.b, "...")` runs
- **Then** the new line sits immediately after `SEND-1.a` and before `RECEIPT-1`, and no blank line
  is introduced inside the `SEND` block

#### Scenario: A new family opens its own block

- **Given** a `## Criteria` section holding only the hand-written line `SEND-1  One.`
- **When** `insert(RECEIPT-1, "Two.")` runs
- **Then** the text contains `SEND-1  One.\n\n- **RECEIPT-1**  Two.` (one blank line separates the
  families) and the frontmatter becomes `families: [SEND, RECEIPT]`

#### Scenario: The first criterion in an empty section

- **Given** a file ending in `## Criteria` and one blank line
- **When** `insert(SEND-1, "I hit enter.")` runs
- **Then** the result contains `## Criteria\n\n- **SEND-1**  I hit enter.`; the heading keeps its
  blank line and the criterion does not gain a second one

#### Scenario: An insert leaves everything else alone

- **Given** the sample file with intent prose and a retired block
- **When** `insert(SEND-2, "It reaches them.")` runs and the document is serialized
- **Then** the intent prose, the `## Retired` heading and the exact `        retired: different
  product` continuation are all still present character for character

#### Scenario: A new family appends below a section a `# ` heading closed

- **Given** `## Criteria` holding `SEND-1  One.`, followed by `# Appendix` and some notes
- **When** `insert(RECEIPT-1, "Two.")` runs
- **Then** the new line sits after `SEND-1` and before `# Appendix`. The level-one heading recorded
  the append point, so the new family lands at the bottom of the criteria block rather than above it

#### Scenario: A file with no criteria section gets one

- **Given** a file with frontmatter, a `# Chat` title and an `## Intent` paragraph, and no
  `## Criteria` heading at all
- **When** `insert(SEND-1, "One.")` runs
- **Then** a `## Criteria` heading is added after the last line with content and the criterion lands
  under it; re-parsing the result yields the same intent prose, one criterion and no strays

#### Scenario: A save leaves no temp file behind

- **Given** a document loaded from disk and inserted into
- **When** `save()` succeeds
- **Then** the target file holds the new criterion and no `.{name}.hi-tmp` sibling remains in the
  directory

#### Scenario: A long sentence stays on one line

- **Given** a sentence of roughly 125 characters and the id `SEND-1`
- **When** `render_criterion` runs
- **Then** exactly one line is returned, and it is `- **SEND-1**  ` followed by the sentence
  unchanged. No wrapping, however long the sentence

#### Scenario: A case is rendered as a nested list item

- **Given** the ids `SEND-1`, `SEND-1.a` and `SEND-1.a.1`, each with a short sentence
- **When** `render_criterion` runs on each
- **Then** the lines are `- **SEND-1**  One.`, `  - **SEND-1.a**  A case.` and
  `    - **SEND-1.a.1**  A step.`: two spaces of indent per depth level below the first, so the
  file renders as a nested list rather than one paragraph (hi: FILE-1.b)

#### Scenario: A criterion is read however it was decorated

- **Given** a `## Criteria` section holding `- SEND-1  A plain bullet.`, `SEND-2  No bullet at
  all.`, `  - **SEND-2.a**  Bulleted and bold.` and `  - _SEND-2.b_  Italic, because someone will.`
- **When** the file is parsed
- **Then** four criteria result, and each `raw_id` is the id as it means rather than as it was
  decorated: `SEND-1`, `SEND-2`, `SEND-2.a`, `SEND-2.b`, with `text` carrying no bullet or emphasis
  from the id

#### Scenario: A pasted multi-line thought collapses to a sentence

- **Given** the text `"I hit enter\n   and it   shows up"` and the id `SEND-1`
- **When** `render_criterion` runs
- **Then** the result is the single line `- **SEND-1**  I hit enter and it shows up`

#### Scenario: A scaffolded file is already a valid document

- **Given** `new_file_text("Billing", "BILLING")`
- **When** the result is parsed
- **Then** `front.version` is `Some(1)`, `front.families` is `["BILLING"]`, and `criteria` is empty

## Error Cases

| Condition | Behavior |
|-----------|----------|
| The file cannot be read | `Doc::load` returns the `io::Error` wrapped with `reading {path}` |
| The file cannot be written | `Doc::save` returns the `io::Error` wrapped with `writing {path}`. The failure happens against the temp file or the rename, so the target on disk is unchanged (hi: FILE-8) |
| The temp file is created but the write, flush or sync fails | The temp file is removed and the error is returned; the target is never opened for writing |
| The rename over the target fails | The temp file is removed and the rename's error is returned |
| `insert` adds a family to a file with no frontmatter | Returns ``{path} has no frontmatter, so add `---\nhi: 1\n---` at the top``, in which each `\n` is a literal backslash and `n` rather than a real line break. The line splice has already happened in memory, so the caller must drop the `Doc` rather than `save` it |
| An id-shaped token will not parse | Recorded on the `Criterion` as `id_error`; parsing continues. Never an error here. Reporting is `check`'s job (hi: CHECK-2.d). A zero-padded level arrives as `IdError::PaddedLevel` like any other parse failure (hi: ID-1.c) |
| An id-shaped line sits outside every section | Recorded on `doc.stray`; parsing continues. Reporting it as a `StrayCriterion` is `check`'s job (hi: CHECK-2.e) |
| An id-shaped line sits inside `## Intent` | Not a criterion and not a stray. It is kept as intent prose, because the intent branch of `parse_body` runs above the stray branch, so nothing reports it (see `tasks.md`) |
| The frontmatter fence is opened but never closed | Not treated as frontmatter. `front` stays `Default` with `range: None` and the whole file is parsed as body |
| The first line is not `---` | Same as above: there is no frontmatter and the body starts at line 0. A leading BOM is stripped first, so a BOM before `---` is still frontmatter (hi: FILE-11) |
| A fenced code block is never closed | The rest of the body is opaque: no further heading, criterion or stray is recognized (hi: FILE-9) |
| The file has no `## Criteria` heading at all | `insert` creates one after the file's last line with content and lands the criterion under it (hi: CAPTURE-7) |
| `render_criterion` is given whitespace-only text | Returns one line, the indent, the bullet and `**{id}**` followed by the two separating spaces and nothing else. Rejecting an empty sentence is `capture`'s job |
| A criterion line's family is lowercase, as in `send-2  ...` | `looks_like_id` is case-insensitive on the family, so the line is a criterion. `Id::parse` then fails with `IdError::BadFamily`, and it is recorded like any other malformed id rather than read as prose and lost (hi: CHECK-2.d) |
| A line begins with an ordinary hyphenated word, as in `spec-sync is fine` | Not a criterion and not a stray. `looks_like_id` requires the first level after the hyphen to be a digit, so a hyphenated word is prose |
| The sentence is longer than any sensible terminal width | Still one line. Wrapping is not a failure mode here because it never happens |
| A duplicate id, a missing parent, or a retired-id collision | Not detected here. `doc` reports structure; `check` reports problems (hi: CHECK-2.a, CHECK-2.b, CHECK-2.c) |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std::fs` | `read_to_string` in `load`; `File::create`, `rename` and `remove_file` in the atomic write behind `save` |
| `std::io` | the `Write` trait for `write_all` and `flush` on the temp file, and `io::Result` as `write_atomically`'s return type. `sync_all` is `fs::File`'s own method, not part of `Write` |
| `std::path` | `Path`, `PathBuf` for the document's own location, the temp file's directory, and `name()` |
| `anyhow` | `Result`, `Context` for path-carrying io errors, `bail!` for the missing-frontmatter refusal |
| `id` | `Id` (`family`, `Id::parse`, `parent`, `is_descendant_of`, `depth` for the list item's indent, `Display`), `IdError` for the recorded parse failure, `looks_like_id` for deciding whether a line starts a criterion. `Id::levels` and `root_number()` are not used here |

### Consumed By

| Module | What is used |
|--------|-------------|
| `workspace` | `Doc` and `Doc::load` to hold every loaded file; `doc.front.families` and `doc.used_families()` to resolve a family to a file and to list every family; `doc.all()` to find an id or iterate the repo; `doc.criteria.len()` for the active count. It does not read `doc.path`; the modules that print a file name do |
| `capture` | `Doc::load`, `Doc::insert`, `Doc::save` to append a criterion; `new_file_text` to start a file for a new family; `doc.path` to name the file it wrote |
| `check` | `Section` to tell active from retired; `doc.path`, `doc.all()`, `Criterion::line_no`, `raw_id`, `id`, `id_error`, `section` and `front.families` to report structural problems against a file and line; `doc.stray` to report a criterion that sits outside every section (hi: CHECK-2.e) |
| `out` | `Doc`, `Criterion` and `Section` to list criteria, build issue bodies, build the export payload and regenerate the `INTENT.md` index; `doc.path`, `doc.name()`, `doc.title`, `doc.intent`, `doc.front.families`, `doc.used_families()`, `doc.criteria.len()`, `criterion.note`, and `criterion.id` for the depth it indents a listing with and exports as `depth`. `out::write_index` also calls `write_atomically` directly, so `INTENT.md` gets the same protection as a hi file (hi: FILE-8) |
| `view` | `Doc` and `Criterion` to build the HTML page: `doc.title`, `doc.name()`, `doc.intent`, and `doc.criteria` / `doc.retired` as separate lists; `criterion.raw_id`, `criterion.text` and `criterion.id` (for nesting depth, capped at four levels) |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Verification pass against source: cited hi: FILE-6 for the one-line rule, added `view` to Consumed By, narrowed the `Id` surface actually used, recorded the `criteria_end` gap when a `# ` heading closes `## Criteria`, and corrected the frontmatter and no-heading claims. |
| 2026-09-16 | Claude | Reconciled with the bug-fix pass: fenced blocks are opaque (FILE-9), a `# ` heading now records the append point, `stray` is recorded for criteria outside every section (CHECK-2.e), block-style frontmatter is parsed and preserved via `families_span`/`families_block` (FILE-7), a BOM is stripped (FILE-11), the file's line ending is kept in `newline` (FILE-10), `save` is atomic (FILE-8), and `insert` creates a missing `## Criteria` section (CAPTURE-7). |
| 2026-09-16 | Claude | Verification pass against `src/doc.rs`: recorded that `## Intent` is excluded from `stray` because the intent branch runs first, restated the parent-insertion rule to match `insertion_point` (the parent itself counts, not only its descendants), corrected the no-frontmatter message's literal `\n`, and corrected the `std::io` surface (`sync_all` is `fs::File`'s, not `Write`'s). |
| 2026-09-16 | Claude | Reconciled with the list-item rule and the new `looks_like_id`: every quoted render is now `- **ID**  sentence` indented two spaces per depth level; the criterion-line grammar no longer claims column 0, because `strip_bullet`, `strip_emphasis` and `is_criterion_line` accept a bullet, emphasis and any indent; an indented line is now a stray like any other, and the recorded token keeps its emphasis; a nested criterion line ends the continuation run above it; added the lowercase-family and hyphenated-word error cases; recorded `Id::depth` as consumed and `out`'s use of `write_atomically`, `criterion.note` and `criterion.id`. |
