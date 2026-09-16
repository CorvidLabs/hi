---
module: check
version: 1
status: active
files:
  - src/check.rs

db_tables: []
depends_on:
  - specs/doc/doc.spec.md
  - specs/id/id.spec.md
  - specs/workspace/workspace.spec.md
---

# Check

## Purpose

Structural validation of an already-loaded workspace, and nothing else. `check` walks every parsed
`hi/*.md` and reports the six ways a file can be structurally wrong: a duplicate id, a case whose
parent does not exist, an active criterion reusing a retired id, a line that is id-shaped but not a
valid id, a family used by a criterion that the file's frontmatter does not declare, and a
criterion-shaped line stranded outside every section, where nothing would read it.

Everything else is deliberately not a problem. `check` has no opinion about a criterion's English,
its length, whether a spec exists for it, whether a test proves it, or whether anything downstream
implements it. Incomplete intent is the normal state of intent (hi: CHECK-1). The module is a pure
function over an in-memory `Workspace`: it opens no files, makes no network call, and mutates
nothing, so `hi check` is safe to add to an existing repo on a Friday afternoon.

Deciding what to print and which exit code to use is `main.rs`'s job; this module only produces the
`Report`.

## Public API

| Export | Description |
|--------|-------------|
| `Kind` | The closed set of six structural problem kinds, serialized in kebab-case for `--json`. |
| `code` | Method on `Kind` returning the stable kebab-case string for one kind, used in text output. |
| `Problem` | One located structural problem: kind, file, 1-based line, id, and human message. |
| `Report` | Everything one check run found: file/criteria/retired counts, families, and problems. |
| `ok` | Method on `Report` reporting whether the run found zero problems; drives the exit code. |
| `run` | Runs every structural check across a workspace and returns the `Report`. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Kind` | `enum` with exactly six variants (`DuplicateId`, `OrphanCase`, `RetiredCollision`, `UnparseableId`, `UndeclaredFamily`, `StrayCriterion`). Derives `Debug, Clone, Copy, PartialEq, Eq, Serialize` with `#[serde(rename_all = "kebab-case")]`, so JSON carries `duplicate-id`, `orphan-case`, `retired-collision`, `unparseable-id`, `undeclared-family`, `stray-criterion`. |
| `Problem` | `struct` with public fields `kind: Kind`, `file: String` (path relative to the workspace root), `line: usize` (1-based), `id: String` (the parsed id, or the raw token when it did not parse, or the stray line's first token), and `message: String` (a complete human sentence). Derives `Debug, Clone, Serialize`. |
| `Report` | `struct` with public fields `files: usize`, `criteria: usize` (active only), `retired: usize`, `families: Vec<String>` (sorted), and `problems: Vec<Problem>` (sorted by file then line). Derives `Debug, Clone, Serialize`. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module declares no trait and hand-writes no trait implementation; `Serialize`, `Debug`, `Clone`, `Copy`, `PartialEq` and `Eq` are all derived. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `run` | `pub fn run(workspace: &Workspace) -> Report` | The whole module. Collects every retired id across the workspace in a first pass, then walks every criterion in every doc applying the per-criterion checks and reporting each doc's stray lines, sorts the problems by file then line, and returns a `Report` with the workspace's counts and families. Takes `&Workspace`, performs no I/O, and cannot fail. There is no `Result`. |
| `code` | `pub fn code(self) -> &'static str` | Method on `Kind`. Maps each variant to its stable kebab-case code (`duplicate-id`, `orphan-case`, `retired-collision`, `unparseable-id`, `undeclared-family`, `stray-criterion`), which `main.rs` prints between the line number and the message. |
| `ok` | `pub fn ok(&self) -> bool` | Method on `Report`. `true` exactly when `problems` is empty. This is the single source of truth for `hi check`'s exit code; counts never affect it. |

## Invariants

1. `run` is a pure function of its argument. It reads no file, opens no socket, consults no
   environment variable, and mutates neither the workspace nor anything on disk. Every document it
   inspects was already parsed by `workspace`/`doc` (hi: CHECK-4).
2. `run` cannot fail. It returns `Report`, not `Result<Report>`, and reports a malformed id as a
   `Problem` rather than as an error. No input shape aborts the walk.
3. Unfinished intent is never a problem. A criterion with no spec, no test, no ticket, no evidence
   and nothing implementing it is structurally clean, and so is a file that is still being written
   (hi: CHECK-1, CHECK-1.a, CHECK-1.b). Nothing in this module inspects a criterion's prose, length,
   grammar, or testability.
4. Exactly six things are problems, and `Kind` is the closed list of them (hi: CHECK-2). Adding a
   seventh kind is a contract change to this spec, not an implementation detail.
5. Every `Problem` names the file and a 1-based line, so a human or an editor can jump straight to
   it (hi: CHECK-3). For a criterion, `line` comes from `Criterion::line_no()`, never from the
   0-based `line` field; for a stray, it is the 0-based index in `Doc::stray` plus one. `file` is
   `Workspace::rel`, so output is relative to the workspace root and stable across machines.
6. `problems` is sorted by `file`, then `line`, so two runs over the same workspace produce byte
   identical output.
7. Retired ids are gathered across the whole workspace in a first pass, before any criterion is
   judged. The order of `workspace.docs` therefore cannot change whether a retired collision is
   found, and a criterion may collide with an id retired in a different file.
8. Duplicate detection is workspace-wide: the second and later sightings of an id are the
   duplicates, and the message names the file and line of the first. Orphan detection is file-local:
   a case's parent must appear in the same document.
9. A retired criterion is exempt from the `RetiredCollision` and `UndeclaredFamily` checks; both
   are gated on `section == Section::Criteria`. It is not exempt from `DuplicateId`, `OrphanCase`,
   or `UnparseableId`, and its ids count toward the set a case may hang off.
10. An id that does not parse yields exactly one `UnparseableId` problem and is then skipped for
    every other check. An unparseable line can never also be reported as a duplicate, an orphan,
    a retired collision, or an undeclared family.
11. One criterion may produce several problems. A reused retired id is both a `DuplicateId` and a
    `RetiredCollision`, and each is reported separately.
12. `Report::ok()` is true exactly when `problems` is empty. The counts (`files`, `criteria`,
    `retired`, `families`) are informational and never influence it, so a workspace with zero
    criteria is clean.
13. `criteria` counts active criteria only (`Workspace::criteria_count`), `retired` sums each doc's
    retired list, and `families` is `Workspace::families()`, which is sorted and drawn from both
    frontmatter declarations and actual use.
14. Every public type is `Serialize`, and `Kind` is renamed to kebab-case, so `hi check --json`
    emits the same finding set a human sees in text.
15. A stray is not a criterion. `Doc::stray` is a separate list of `(0-based line, first token)`
    pairs, so a stray line yields exactly one `StrayCriterion` and takes no part in the duplicate,
    orphan, retired, family or unparseable checks (its token is never parsed as an `Id`), and it is
    not counted in `criteria`.
16. What `check` can see is exactly what `doc` parsed. A fenced code block (``` or `~~~`) is opaque
    to `doc`, so an id-shaped line inside one is neither a criterion nor a stray and produces no
    problem of any kind. A file may document the hi format in its own `## Intent` without the
    example becoming findings (hi: FILE-9).
17. `## Intent` is prose all the way down, not a gap between sections. `doc::parse_body` routes
    every line of an intent section into `Doc::intent` before it reaches the stray branch, so an
    id-shaped line at column 0 under `## Intent` is neither a criterion nor a stray and produces no
    problem, the same outcome a fenced example gets, by a different route. A stray is recorded only
    where no section is open *and* no intent is being collected: before the first heading, under a
    `## ` heading `doc` does not recognize, or below a `# ` heading, which clears `in_intent` as
    well as the section.

## Behavioral Examples

### Scenario: A clean workspace has no problems

- **Given** `hi/chat.md` declares `families: [SEND]` and contains `SEND-1` and its case `SEND-1.a`
- **When** `run` is called on the workspace
- **Then** `report.ok()` is `true`, `report.criteria` is `2`, and `report.families` is `["SEND"]`

### Scenario: Unfinished intent is never a problem

- **Given** a single criterion `SEND-1  It should feel fast.` with no spec, no test, no evidence and
  nothing downstream
- **When** `run` is called
- **Then** `report.ok()` is `true` and `hi check` exits 0 (hi: CHECK-1.a, CHECK-1.b)

### Scenario: The same id in two different files

- **Given** `hi/a.md` and `hi/b.md` both declare `SEND-1`
- **When** `run` is called
- **Then** exactly one problem is reported, its kind is `DuplicateId`, it is located at the second
  sighting, and its message names the first file and line (hi: CHECK-2.a)

### Scenario: A case with no parent

- **Given** `hi/chat.md` contains `SEND-1.a` but no `SEND-1`
- **When** `run` is called
- **Then** exactly one problem is reported, its kind is `OrphanCase`, and its message is
  `SEND-1.a has no parent SEND-1` (hi: CHECK-2.b)

### Scenario: An active criterion reuses a retired id

- **Given** `SEND-3` appears under `## Criteria` and also under `## Retired`
- **When** `run` is called
- **Then** the problem kinds include both `RetiredCollision` ("retired ids stay reserved forever")
  and `DuplicateId` (hi: CHECK-2.c)

### Scenario: A retired criterion may keep its case parent

- **Given** `SEND-1` is live under `## Criteria` and its case `SEND-1.a` sits under `## Retired`
- **When** `run` is called
- **Then** `report.ok()` is `true`: the parent lookup sees every id the file declares, retired
  included, and a retired criterion is not checked against the retired set

### Scenario: A line shaped like an id but not a valid one

- **Given** `hi/chat.md` contains `SEND-1.a.b  Two letters.`, which breaks strict level alternation
- **When** `run` is called
- **Then** exactly one problem is reported, its kind is `UnparseableId`, its `id` is the raw token
  `SEND-1.a.b`, and its message quotes the token and appends the `IdError` reason (hi: CHECK-2.d)

### Scenario: A family the frontmatter does not declare

- **Given** `hi/chat.md` declares `families: [SEND]` but contains an active `OFFLINE-1`
- **When** `run` is called
- **Then** a problem of kind `UndeclaredFamily` is reported, naming `OFFLINE`, because frontmatter
  is where a family declares its home file: `Workspace::doc_for_family` consults `families` before
  falling back to a search by use, and `Doc::insert` adds a family to `families` whenever capture
  writes a criterion for one the frontmatter lacks. Capture only reaches `doc_for_family` when the
  new id has no existing parent, because a case follows the file its parent lives in
  (hi: CAPTURE-4.a), so an undeclared family is exactly the case where the weaker fall-back has to
  carry the lookup

### Scenario: A criterion stranded below a `# ` heading

- **Given** `hi/chat.md` holds `SEND-1` under `## Criteria`, then a `# Appendix` heading, then
  `SEND-9  Invisible.`
- **When** `run` is called
- **Then** exactly one problem is reported, its kind is `StrayCriterion`, and its `id` is `SEND-9`:
  the `# ` heading closed the section, so `doc` recorded the line in `Doc::stray` rather than
  parsing it as a criterion, and `check` says so instead of letting it vanish (hi: CHECK-2.e)

### Scenario: Prose outside a section is not a stray

- **Given** the same shape but with `Just prose about the feature.` below the `# Notes` heading
- **When** `run` is called
- **Then** `report.ok()` is `true`. Only a line whose first token satisfies `looks_like_id` is
  recorded as a stray, so ordinary prose below a heading is silent

### Scenario: An id written into the intent prose

- **Given** `hi/chat.md` holds `SEND-9  a line in the intent prose` at column 0 under `## Intent`
- **When** `run` is called
- **Then** `report.ok()` is `true` and nothing is reported. An intent section is collected as prose
  before `doc` ever looks for a stray, so the line is neither a criterion nor a stray. "Outside
  every section" in `Kind::StrayCriterion` means outside every section `doc` recognizes, and
  `## Intent` is one it does

### Scenario: The format documented inside a hi file

- **Given** a file whose `## Intent` prose contains a fenced block (``` or `~~~`) holding a line
  such as `SEND-9  inside a fence`
- **When** `run` is called
- **Then** `report.ok()` is `true` and `report.criteria` does not count the fenced line. `doc`
  treats the fence as opaque, so the example reaches `check` neither as a criterion nor as a stray
  (hi: FILE-9)

### Scenario: Locating a problem

- **Given** any reported problem
- **When** it is read off the `Report`
- **Then** `problem.file` is the workspace-relative path, `problem.line` is the 1-based line the id
  sits on, and `problem.kind.code()` is the stable kebab-case code: enough for a human or an
  editor to jump straight to it (hi: CHECK-3). `main.rs` renders that as a file header followed by
  `  <line>:<code>  <message>`, but the format is main's contract, not this module's

## Error Cases

| Condition | Behavior |
|-----------|----------|
| The same id declared twice, in one file or across files | `DuplicateId` at the second and every later sighting; the message names the first file and line. Not reported at the first sighting. |
| A case or step whose parent id is absent from the same file | `OrphanCase` naming the missing parent. A top-level id (`Id::parent()` is `None`) is never an orphan. |
| An active criterion whose id appears under any file's `## Retired` | `RetiredCollision` naming the file it was retired in. Always also a `DuplicateId`, because the retired line and the active line are two sightings of one id. |
| An id-shaped token that `Id::parse` rejects, meaning an empty level (`SEND-1.`), a level that is neither a number nor lowercase letters (`SE-ND-1`), a zero-padded number (`SEND-007`), or broken alternation (`SEND-1.a.b`) | `UnparseableId` carrying the raw token and the `IdError` text; the criterion is skipped for all other checks. `IdError::PaddedLevel` reports that level `'007'` has a leading zero and asks for `'7'` instead, so the id always means the same thing, because `SEND-007` and `SEND-7` must never be two names for one line (hi: ID-1.c). |
| A token with no hyphen, nothing after the hyphen, or a family outside `[A-Z][A-Z0-9_]*` (`SEND1`, `SEND-`, `Send-1`) | Not reported at all. `looks_like_id` rejects the token, so `doc` treats the line as prose and `IdError::MissingHyphen`, `NoLevels` and `BadFamily` never reach this module. |
| An active criterion whose family is not in its file's frontmatter `families` list | `UndeclaredFamily` naming the family. Retired criteria are exempt. The comparison is against `Front::families` as parsed, so an inline list (`families: [SEND]`) and a YAML block list (`families:` then `  - SEND`) behave identically and neither is falsely reported (hi: FILE-7). |
| A criterion-shaped line at column 0 outside `## Criteria` and `## Retired` (typically below a `# ` heading that closed the section, or under a `## ` heading `doc` does not recognize) | `StrayCriterion` naming the token. Nothing parses the line as a criterion, so without this it would silently vanish (hi: CHECK-2.e). |
| An id-shaped line at column 0 under `## Intent` | Not a problem, and not a stray. `doc` collects the whole intent section as prose before the stray branch is reached, so an id written into the intent is prose. The line survives in `Doc::intent` for `view` and `export` rather than becoming a finding. |
| A criterion with no spec, ticket, test, evidence, or downstream work | Not a problem. Never reported, in any form. |
| A file with prose, headings, or blank lines that are not criteria | Not a problem. `doc` only offers lines whose first token is id-shaped, so prose is never judged, whether inside a section or outside one. |
| An id-shaped line inside a fenced code block (``` or `~~~`), whether inside `## Criteria`, inside `## Intent`, or outside every section | Not a problem, and not a criterion. `doc` tracks the fence across the whole body and treats it as opaque, so the line never reaches `check` as either a criterion or a stray (hi: FILE-9). |
| A workspace with no docs, or docs with no criteria | Not a problem. `ok()` is `true` and `criteria` is `0`; `files` and `families` still report what was loaded, since `families` includes frontmatter declarations. |
| A retired criterion using an undeclared family, or a retired id duplicated in `## Retired` | The family check is skipped; the duplicate check still applies. |
| Any I/O or parse failure | Out of scope. `run` is handed already-parsed docs and returns no `Result`. Loading failures surface from `workspace`/`doc`. |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std::collections::HashMap` | `seen` (id to first file and line) and `retired` (id to the file that retired it). |
| `serde` | `Serialize` derive on `Kind`, `Problem`, and `Report`, plus `#[serde(rename_all = "kebab-case")]` on `Kind`, for `hi check --json`. |
| `doc` | `Section::Criteria` / `Section::Retired`, `Doc::all()`, `Doc::path`, `Doc::front.families` (already parsed from either the inline or the YAML block frontmatter style), `Doc::retired`, `Doc::stray` (the `(0-based line, token)` pairs behind `StrayCriterion`), and the `Criterion` fields `id`, `raw_id`, `id_error`, `section` plus `Criterion::line_no()`. |
| `workspace` | `Workspace::docs`, `Workspace::rel()`, `Workspace::criteria_count()`, `Workspace::families()`. |
| `id` | `Id::parent()`, `Id::family`, `Id`'s `Display`, and `IdError`'s `Display` for the unparseable-id reason. |

### Consumed By

| Module | What is used |
|--------|-------------|
| `main` (`src/main.rs`) | `check::run` for the `hi check` subcommand, `Report::ok()` for the exit code (`0` clean, `1` structural error), `Report`'s counts for the summary line, and `Problem`'s fields plus `Kind::code()` for `print_report`. Serializes `Report` with `serde_json` for `hi check --json`. |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Added `StrayCriterion`, the sixth kind: a `# ` heading closes an open section, so a criterion below one was invisible to every verb. Reported now instead (hi: CHECK-2.e). |
| 2026-09-16 | Leif | Review pass against the source: corrected the reachable `IdError` set (`MissingHyphen`, `NoLevels` and `BadFamily` are filtered out by `looks_like_id` before `check` sees them), corrected the counts a criteria-free workspace reports, corrected the `UndeclaredFamily` rationale (capture is not blocked by it), and re-anchored the "Locating a problem" scenario to the `Report` rather than to `main.rs`'s print format. |
| 2026-09-16 | Leif | Reconciled with the bug-fix pass. Purpose now names all six kinds rather than five; invariant 4 says a *seventh* kind would be the contract change; invariant 5 records that a stray's line is `Doc::stray`'s 0-based index plus one, not `line_no()`; added invariant 15 (a stray is never parsed as an `Id` and never counted) and invariant 16 (fences are opaque, so a fenced id-shaped line is neither criterion nor stray, hi: FILE-9). Added scenarios for the stray, for prose outside a section, and for the format documented inside a fence. `IdError::PaddedLevel` (`SEND-007`) joined the reachable unparseable set (hi: ID-1.c), `Doc::stray` joined the consumed surface, and the `UndeclaredFamily` rationale now notes that capture reaches `doc_for_family` only when the id has no existing parent (hi: CAPTURE-4.a). |
| 2026-09-16 | Leif | Verification pass against the source, run against the built binary. Added invariant 17 and a scenario for the `## Intent` exception: `doc` collects an intent section as prose before the stray branch, so an id-shaped line there is neither criterion nor stray. The stray error case previously claimed every column-0 id-shaped line outside `## Criteria`/`## Retired` was reported. Recorded that the `UndeclaredFamily` comparison is against `Front::families` as parsed, so inline and YAML block styles behave identically (hi: FILE-7), and narrowed the fence error case from "anywhere in the file" to the body sections it actually covers. |
