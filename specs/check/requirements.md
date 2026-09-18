---
spec: check.spec.md
---

## User Stories

- As someone adopting hi mid-project, I want `hi check` to pass on a repo full of half-written
  intent so that installing it on a Friday afternoon never turns anyone's build red (hi: CHECK-1)
- As an author of a criterion, I want nothing to complain that my sentence has no spec, no test and
  nothing implementing it, because that is what an unfinished criterion looks like (hi: CHECK-1.a,
  CHECK-1.b)
- As a maintainer of a hi file, I want the one thing that actually rots (ids) checked hard, so
  that a duplicate, an orphaned case, or a resurrected retired id is caught the day it lands
  (hi: CHECK-2)
- As someone reading a failing check, I want every problem to name the file and the line so that I
  can jump straight to it instead of grepping (hi: CHECK-3)
- As a developer on a plane or behind a corporate proxy, I want checking to read nothing but my own
  files so that it works offline and leaks nothing (hi: CHECK-4)
- As a tool integrator, I want `hi check --json` to carry the same findings the text output shows,
  each with a stable kind code, so that an editor or CI script can consume them without parsing
  prose
- As someone documenting the hi format inside a hi file, I want a fenced example to stay an example
  so that showing what a criterion looks like does not fill my report with findings (hi: FILE-9)
- As someone who moved a heading, I want a criterion that ended up outside every section to be
  reported rather than silently ignored, because nothing downstream reads it there (hi: CHECK-2.e)
- As someone reading `hi check --json` from a script, I want each note on its own, under a name that
  stays the same when somebody rewrites the sentence (hi: CHECK-6)

## Acceptance Criteria

- Notes are not problems. `Report::note` carries the product-level why, the unexplained
  retirements, and the generated feature list being behind; `Report::ok` reads none of them and the
  exit code never moves for any of them (hi: CHECK-1, INDEX-3.a, RETIRE-3, INDEX-4.b)
- `run` reports exactly six kinds of problem and nothing else: `duplicate-id`, `orphan-case`,
  `retired-collision`, `unparseable-id`, `undeclared-family`, `stray-criterion`
- Absence of downstream work (no spec, no ticket, no test, no evidence) is never reported in any
  form, and neither is the content, length, or phrasing of a criterion
- Every problem carries the workspace-relative file, a 1-based line number, the id (raw token when
  it did not parse), and a complete human message
- `problems` is sorted by file then line, so repeated runs are byte identical
- `Report::ok()` is true exactly when `problems` is empty, and is the only input to the exit code
- `run` takes an already-parsed `&Workspace` and mutates nothing. Its one read of the filesystem is
  `Workspace::strays`, which opens the files hi skips so that an id written into one is still
  reported (REQ-workspace-011), and that read is why `run` returns `Result<Report>`: a file hi
  cannot read is an operational failure and is never a finding. Nothing about the *structure* of a
  file can make `run` fail, so one run still reports every problem in every file it could read
  (hi: CHECK-5, CAPTURE-15)
- `Kind`, `Problem`, and `Report` all derive `Serialize`, with `Kind` in kebab-case, so JSON output
  and text output describe the same findings
- Findings are limited to what `doc` parsed: an id-shaped line inside a fenced code block, or
  anywhere inside `## Intent`, is not a criterion, not a stray, and not a problem

### REQ-check-013

The check module SHALL report a criterion-shaped line written into a file it does not read as criteria (hi: FILE-20, CHECK-1).

Acceptance Criteria

- Both kinds of stray come from one call to `Workspace::strays`, which reads every path in `Workspace::skipped` through `doc::criterion_tokens` and every doc's `Doc::stray`, and each entry becomes a `stray-criterion` problem. A `StrayPlace::UnreadFile` entry names the file, the line, the token, and the fact that the name is not lowercase; a `StrayPlace::OutsideSection` entry carries REQ-check-011's message. Neither message changed.
- `capture` refuses an id from the same call, so an id this reports as used and an id capture refuses are the same set by construction. They were two scans over two different sets of files, and a retired id in `hi/Archive.md` was reported here and handed out again by capture (hi: CAPTURE-14, DECISIONS.md §32).
- The kind is reused rather than added to. A criterion in a skipped file is the same failure `stray-criterion` already describes — nothing reads it where it is — and the README promises exactly six structural problems.
- A fenced block in such a file is an example rather than structure, exactly as it is under `## Intent` (hi: FILE-9), so documenting the format in a `hi/README.md` does not report a loss.
- A file that cannot be read is skipped rather than reported, because an unreadable file is not evidence of a criterion.


### REQ-check-014

The check module SHALL say when the generated feature list in `INTENT.md` no longer matches what is captured, as a note that never reaches the exit code (hi: INDEX-4.b, CHECK-1).

Acceptance Criteria

- The note comes from `out::index_note`, which rebuilds the block and compares bytes. It is pushed onto the same `notes` vector as the product-level why and the unexplained retirements, and `Report::ok` reads none of them.
- It is not a seventh `Kind`. `run` still reports exactly six structural problems, `hi check` still exits 1 only on a structurally broken file, and the README's "exactly six" stands.
- It is the first note about something hi itself maintains; the other two are about words only a person can write. It is admissible because capture and retire keep the list current themselves (REQ-capture-016, REQ-out-017), so the note fires for one cause: a criterion typed straight into a file, which FILE-14 allows and no verb can see.
- The note names the file and the verb that fixes it, and it goes away once that verb has been run.
- No `INTENT.md`, no markers in it, or a block that already matches: no note. A missing `INTENT.md` is already covered by `product_intent_note`.

## Constraints

- Structural only. This module must never gain a content check, a prose linter, a readability score,
  or a testability heuristic. A 59%-precision requirements-smell detector is how a tool gets
  disabled in week one (DECISIONS.md §9)
- No state, no lifecycle, no evidence binding. There is nothing to read back and nothing to write,
  so `run` must stay a pure function over the in-memory workspace (DECISIONS.md §5)
- `Kind` is a closed set. Adding a variant changes the `--json` contract and every consumer's
  switch, so it is a spec change, not an implementation detail
- Kind codes are stable strings. `Kind::code()` and the serde kebab-case rename must agree, because
  text output and JSON output are the same vocabulary
- Line numbers are 1-based in every `Problem`, while `Criterion::line` is 0-based. `line_no()` is
  the only correct source
- File paths must go through `Workspace::rel`, never `Path::display` on the absolute path, so that
  output does not embed a machine-specific prefix and the separator is a forward slash on every
  platform (hi: FILE-12)
- Whether a problem is *found* must stay independent of document order, which is why retired ids
  are collected in a pass of their own before any criterion is judged. Which of two duplicate
  sightings is *reported* does follow `workspace.docs` order, and that order is fixed by
  `Workspace::load` sorting the paths it reads

## Out of Scope

- Printing, formatting, colouring, pluralizing, and choosing the process exit code: `src/main.rs`
  owns all of it, and this module only builds the `Report`
- Reading, parsing, or writing `hi/*.md`, since `doc` owns parsing and `workspace` owns discovery
- Deciding what a valid id is. `id` owns the grammar and the `IdError` text; this module only
  reports that parsing failed
- Judging a criterion's English, its testability, or whether it is a good idea (DECISIONS.md §9)
- Any check that would require network, git history, a test run, or a spec-sync lookup
- Fixing anything. `check` never edits a file; repairing an id is a human edit or a capture

### REQ-check-001

`hi check` SHALL NOT report a criterion as a problem for being unfinished, unimplemented, or
unproven (hi: CHECK-1, CHECK-1.a, CHECK-1.b).

Acceptance Criteria

- A workspace whose only content is one well-formed criterion with nothing downstream produces
  `report.ok() == true`.
- No code path inspects a criterion's `text`, its length, or its wording.
- A workspace with no docs, or docs with no criteria, is clean rather than empty-and-suspicious.
- Adding hi to an existing repository cannot change that repository's build result unless a file is
  structurally wrong.

### REQ-check-002

`run` SHALL report a `DuplicateId` problem for the second and every later declaration of an id
anywhere in the workspace (hi: CHECK-2.a).

Acceptance Criteria

- The first sighting of an id is recorded and never reported.
- A later sighting in the same file or a different file is reported at its own file and line.
- The message names the file and 1-based line of the first declaration.
- Retired criteria participate: a retired line is both a possible first sighting and a possible
  duplicate.

### REQ-check-003

`run` SHALL report an `OrphanCase` problem for any id with a parent that the same document does not
declare (hi: CHECK-2.b).

Acceptance Criteria

- The parent is `Id::parent()`; a top-level id has none and is never an orphan.
- The parent set is every id the document declares, active and retired, so a live parent may carry a
  retired case and a retired case may hang off a live parent.
- The message is `<id> has no parent <parent>`.
- The check is file-local: a parent declared in a different file does not satisfy it.

### REQ-check-004

`run` SHALL report a `RetiredCollision` problem when an active criterion reuses an id that any
document has retired (hi: CHECK-2.c).

Acceptance Criteria

- Every retired id in the workspace is collected before any criterion is judged, so document order
  cannot change the outcome.
- Only criteria in `## Criteria` are checked; a criterion in `## Retired` is exempt.
- The message names the file the id was retired in and states that retired ids stay reserved
  forever.
- The same line may also be reported as a `DuplicateId`; the two findings are independent.

### REQ-check-005

`run` SHALL report an `UnparseableId` problem for a line whose leading token is id-shaped but is not
a valid id, and SHALL skip that criterion for every other check (hi: CHECK-2.d).

Acceptance Criteria

- The problem's `id` field carries the raw token as it means, not as it was decorated: `doc` strips
  a bullet marker and markdown emphasis, so `- **send-2**  ...` reports the id `send-2`. Nothing
  else is normalized, and the case is left exactly as written.
- The message is `'<token>' is not a valid id: ` followed by the `IdError` reason, or by
  `not a valid id` when no reason is available.
- `SEND-1.a.b` (a case of a case) is reported, since levels alternate number, letter, number. So is
  `SEND-1.2`, a number where a letter belongs.
- `SEND-007` is reported, carrying `IdError::PaddedLevel`: a zero-padded number would parse to a
  different spelling than it was written, so `SEND-007` and `SEND-7` must never be two names for one
  line (hi: ID-1.c).
- `send-2` and `Send-3` are reported, carrying `IdError::BadFamily`. `looks_like_id` is
  case-insensitive on the family so a line plainly meant as a criterion is refused with a reason
  rather than read as prose and lost (hi: CHECK-2.d).
- Exactly one problem is produced for that line: it is never also a duplicate, orphan, retired
  collision, or undeclared family.
- Prose is unaffected, because `doc` only offers a line whose leading token, once a bullet marker
  and markdown emphasis are stripped, satisfies `looks_like_id`: a family that starts with an ASCII letter and continues in letters, digits and underscores, a
  hyphen, and a first level beginning with a digit. `spec-sync`, `well-formed` and `SEND-a` are
  ordinary words and reach this module as nothing at all.

### REQ-check-006

`run` SHALL report an `UndeclaredFamily` problem when an active criterion uses a family that its
file's frontmatter `families` list does not declare (hi: CHECK-2.f).

Acceptance Criteria

- The comparison is exact and case-sensitive against `doc.front.families`, which `doc` has already
  parsed from whichever frontmatter style the file uses. An inline `families: [SEND]` and a YAML
  block `families:` / `  - SEND` produce the same list, so neither is falsely reported (hi: FILE-7).
- Only criteria in `## Criteria` are checked; retired criteria are exempt.
- The message is `family <FAMILY> is not in this file's frontmatter families list`.
- The problem exists because frontmatter is the declared home of a family: `Doc::insert` adds a
  family to `families` whenever capture writes a criterion for one the frontmatter lacks, so a file
  that uses a family without declaring it has drifted from the shape capture maintains, and
  `Workspace::doc_for_family` is left resolving it through its weaker fall-back search by use.
- Capture consults `doc_for_family` only for an id with no existing parent; a case is routed to the
  file its parent actually lives in (hi: CAPTURE-4.a). An undeclared family is therefore precisely
  the case where the fall-back search has to decide, which is why the drift is worth reporting.

### REQ-check-007

Every problem SHALL name its file and a 1-based line, and problems SHALL be ordered by file then
line (hi: CHECK-3).

Acceptance Criteria

- `file` is produced by `Workspace::rel`, so it is relative to the workspace root.
- `line` is produced by `Criterion::line_no()`, so it is 1-based and matches what an editor shows.
- `problems` is sorted by `file`, then `line`, before the report is returned.
- Two runs over the same workspace produce identical output, so the result is diffable.

### REQ-check-008

`run` SHALL be offline and SHALL change nothing, and no *structural* shape of a file SHALL be able
to abort it (hi: CHECK-4).

Acceptance Criteria

- No filesystem write, no network access, and no environment lookup occurs in this module.
- The only read is the one `Workspace::strays` makes over the files hi skips, which is the
  reservation lookup `capture` refuses from. It is not a finding and it does not vary with the
  content of a criterion.
- The workspace is taken by shared reference and is not mutated.
- `run` returns `Result<Report>`. The error arm exists only for a file hi could not read at all
  (hi: CAPTURE-15); no id, sentence, section, family or malformed line can reach it, and every
  structural problem is a `Problem` in the `Report`.
- Every unit test constructs its workspace from in-memory strings, with no filesystem fixture; the
  unreadable-file case is covered in `workspace` and end to end, where a file on disk is the point.

### REQ-check-009

The `Report` SHALL carry the workspace totals, and `Report::ok()` SHALL be the single source of
truth for whether the run is clean.

Acceptance Criteria

- `files` is the number of documents, `criteria` counts active criteria only, `retired` sums each
  document's retired list, and `families` is the sorted workspace family list.
- `ok()` is `true` exactly when `problems` is empty.
- No count influences `ok()`, so a workspace with zero criteria is clean.
- `ok()` is the only thing this module contributes to the exit code: `main.rs` maps `true` to
  `ExitCode::SUCCESS` and `false` to `1`. A failure to find or load the workspace exits `1` before
  `run` is ever called, and is `workspace`'s error path, not a `Report`.
- `Workspace::find` stops at a `.git` boundary, so inside a repository that has no `hi/` directory
  it loads an empty workspace rather than adopting an outer repository's criteria. `hi check` there
  reports `0 criteria · 0 families · 0 files` and exits `0`; the not-a-repository error path is
  reached only when no `hi/` and no `.git` is found all the way up (hi: CHECK-1).

### REQ-check-010

The report SHALL be serializable so that machine output carries the same findings as text output.

Acceptance Criteria

- `Problem`, `Note` and `Report` derive `Serialize`. `Kind` and `NoteKind` implement it by hand,
  writing `self.code()`, so the code is defined once rather than once in `code()` and once in a
  serde rename that happens to agree with it today (DECISIONS.md §31).
- `Kind` serializes as `duplicate-id`, `orphan-case`, `retired-collision`, `unparseable-id`,
  `undeclared-family`, `stray-criterion`.
- `hi check --json` emits the whole `Report`, including the counts and every problem, not a
  summary.
- A consumer reading JSON sees neither more nor fewer findings than a human reading the text.

### REQ-check-011

`run` SHALL report a `StrayCriterion` problem for a criterion-shaped line that sits outside both
`## Criteria` and `## Retired`, because nothing reads it where it is (hi: CHECK-2.e).

Acceptance Criteria

- A line whose leading token satisfies `looks_like_id`, appearing where `Doc` has no open section
  and is not collecting `## Intent` prose, is reported with its 1-based line number and the token as
  the id.
- The common cause is a `# ` heading closing the section above it; `Doc::parse_body` records these
  in `Doc::stray` as it parses. A `## ` heading `doc` does not recognize (`## Notes`, say) leaves
  the same gap.
- A line under `## Intent` is never a stray. `Doc::parse_body` routes every line of an intent
  section into `Doc::intent` before the stray branch is reached, so an id written into the intent
  stays prose and survives for `view` and `export`.
- Ordinary prose outside a section is not reported. Only a genuinely id-shaped token is, which an
  ordinary hyphenated word such as `spec-sync` or `well-formed` is not.
- Indentation is immaterial. `Doc::parse_body` tests every line trimmed, so a nested list item such
  as `  - **SEND-11**  ...` outside every section is reported exactly as a flush-left one would be.
  Criteria are rendered as nested list items, so indentation cannot be the thing that distinguishes
  a criterion from a continuation; the id-shaped leading token is.
- The recorded token has a leading `- `, `* ` or `+ ` bullet removed but keeps any markdown
  emphasis, so `- **SEND-9**  ...` is reported with the id `**SEND-9**`. A stray is never handed to
  `Id::parse`, so nothing normalizes it the way `Criterion::raw_id` is normalized.
- The problem is reported once per stray line, and the stray line is not counted as a criterion.
- The stray token is never handed to `Id::parse`, so a stray takes no part in the duplicate, orphan,
  retired-collision, unparseable or undeclared-family checks.
- The reported line is `Doc::stray`'s 0-based index plus one, which is the same 1-based number
  `Criterion::line_no()` would give.

### REQ-check-012

`run` SHALL report nothing for an id-shaped line that sits inside a fenced code block, so that a hi
file can document the hi format without the example becoming findings (hi: FILE-9).

Acceptance Criteria

- A ``` or `~~~` run of three or more characters opens a fenced block, and a run of at least the
  same width in the same character closes it; `doc` skips everything between.
- A line inside a fence is neither parsed as a criterion nor recorded in `Doc::stray`, so it can
  produce no problem of any kind and is not counted in `report.criteria`.
- This module holds no fence logic of its own and must not gain any: `check` reports exactly what
  `doc` parsed, and re-scanning the raw lines would reintroduce the bug.
- A fenced example inside `## Intent` still survives in the intent prose, so `view` and `export`
  keep showing it.

### REQ-check-015

The report SHALL carry its notes as a list, each under a stable code, and no note SHALL carry
presentation or reach the exit code (hi: CHECK-6, CHECK-1).

Acceptance Criteria

- `Report.notes` is a `Vec<Note>`, one entry per thing worth saying. It replaced a single
  `Option<String>` holding every note joined with `\n      `: six spaces of terminal indentation
  inside the data, which is why `--json` could not carry the notes apart and a reader had to split
  on whitespace to get them back.
- A `Note` is a `NoteKind` and a `message`. The message carries no leading `note:`, no indentation
  and no embedded newline. How the notes are laid out belongs to whoever prints them, and `main`
  prints one `note: <message>` line each.
- `NoteKind` has four codes: `no-product-why`, `index-behind`, `index-markers`,
  `unexplained-retirement`. They are chosen for what the note is about, so rewriting a sentence
  does not move one.
- A missing `INTENT.md` and an `INTENT.md` with no human prose are both `no-product-why`, because
  the remedy is the same sentence. A list that is behind and an unpaired marker pair are separate
  codes, because one is fixed by running `hi index` and the other by repairing the markers by hand.
- The code is not printed in the terminal. A problem prints its kind because a person navigates by
  it; a note is already a sentence saying what to do, and the code exists for the reader that is
  not a person.
- No `NoteKind` is a `Kind` and none of them may become one. `Report::ok` does not read `notes`, so
  nothing in them moves the exit code, and the six structural problems stay six (hi: CHECK-1).
- `out::index_note` returns a `check::Note`, so the two index codes are decided beside the other
  two rather than in a second place (hi: INDEX-4.b).
