---
module: out
version: 1
status: active
files:
  - src/out.rs

db_tables: []
depends_on:
  - specs/doc/doc.spec.md
  - specs/id/id.spec.md
  - specs/workspace/workspace.spec.md
---

# Out

## Purpose

Implements the four generating hi verbs: `ls`, `issue`, `export`, and `index`. These are the
generation half of the pipeline: intent is written once by a human and tickets, agent payloads,
and the root feature index are derived from it (hi: ISSUE-1, EXPORT-1, INDEX-1). `hi view` also
only reads, but it lives in the `view` module.

The module is deliberately narrow. It renders an already-parsed `Workspace` and never parses,
never mutates a `hi/*.md` file, and never consults the network. The one write it performs is
`INTENT.md`, and that write is confined to the generated block between the `hi:index` markers
(hi: INDEX-2). The one external process it may start is `gh`, and only when the caller asked for
it with `--create` (hi: ISSUE-1.b).

Nothing here fails because intent is incomplete. `ls` over an empty repository prints a hint and
succeeds; a whole-repo `export` of a repository with no criteria emits an empty `files` list and
succeeds. The only intent-shaped errors are requests that cannot be answered: an unparsable id, an
unknown id, a retired id asked to become work, and a scope that selects no file. The one file this
module writes adds a fifth: an `INTENT.md` carrying an opening `hi:index` marker with no matching
close is refused rather than guessed at, because guessing there would eat the prose between the two
(hi: INDEX-2.b). Beyond those, the environment can still fail. `gh` may not start or may exit
non-zero, and `INTENT.md` may not be writable. Error Cases below lists all eight.

## Public API

| Export | Description |
|--------|-------------|
| `ls` | Print every criterion grouped by file, indented by id depth, optionally filtered to one family and optionally including retired lines. |
| `issue_markdown` | Render one criterion as a `(title, body)` ticket pair carrying its id, its cases, and the file's intent prose. |
| `issue` | Resolve an id and print its ticket, or hand the same ticket to `gh issue create` when `create` is set. |
| `export` | Build the agent payload as pretty-printed JSON for a family, a file stem, or the whole repository. |
| `index_block` | Build the generated feature list: one Markdown bullet per hi file, with its families and active-criterion count. |
| `write_index` | Rewrite the generated block inside `INTENT.md`, matching the `hi:index` markers on whole lines only, creating the file or the `## Features` section when they do not exist yet, and refusing when an opening marker has no close, then return the path written relative to the workspace root. |

### Structs & Enums

| Type | Description |
|------|-------------|
| None. | The JSON payload types (`Export`, `ExportFile`, `ExportCriterion`) are private; the payload is a contract expressed as JSON, not as Rust types, so nothing outside this module constructs them. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines no traits. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `ls` | `fn ls(workspace: &Workspace, family: Option<&str>, include_retired: bool)` | Walks `workspace.docs` in load order. Prints the workspace-relative path of each file that has something to show, then each criterion as `<2×depth spaces><raw_id>  <text>`; retired lines gain a trailing `  (retired)`. A file with nothing matching is skipped entirely, including its heading. When no criterion at all was printed, prints the capture hint instead. Returns `()` and cannot fail. |
| `issue_markdown` | `fn issue_markdown(doc: &Doc, criterion: &Criterion) -> (String, String)` | Title is the criterion sentence with trailing `.` characters trimmed. Body is the sentence, a blank line, `hi: <raw_id>`, an optional `Cases:` bullet list of the criterion's active descendants indented relative to the parent, and, when `doc.intent` is not blank, a `---` rule followed by `Intent for <file stem>:` and the prose. |
| `issue` | `fn issue(workspace: &Workspace, raw_id: &str, create: bool, repo: Option<&str>) -> Result<()>` | Parses `raw_id`, finds the criterion in the workspace (active or retired), refuses a retired one, then either prints `## <title>` followed by the body or spawns `gh issue create --title <title> --body <body>`, adding `--repo <repo>` when supplied. |
| `export` | `fn export(workspace: &Workspace, scope: Option<&str>) -> Result<String>` | Selects the files the scope reaches, filters their criteria when the scope is a family, and serializes one envelope (`hi`, `scope`, an optional `product`, and `files`) as pretty-printed JSON. Errors only when a stated scope selected no file at all, which includes a family declared in frontmatter that no criterion actually uses. |
| `index_block` | `fn index_block(workspace: &Workspace) -> String` | Emits `- [<stem>](hi/<stem>.md): <families> (<n> criteria)` per file, using frontmatter families when declared and used families otherwise, `no families yet` when neither exists, and singular `criterion` at a count of one. An empty workspace yields `- nothing captured yet`. |
| `write_index` | `fn write_index(workspace: &Workspace) -> Result<String>` | Splices the generated block into `INTENT.md` between `<!-- hi:index -->` and `<!-- /hi:index -->` when the private `index_span` finds both on lines of their own, in that order; bails when the private `has_marker_line` still sees an opening marker line that `index_span` could not close; writes a whole starter file when `INTENT.md` is absent or blank; otherwise appends a `## Features` section carrying the block to the existing prose. |

## Invariants

1. Every verb in this module is a pure read of an already-loaded `Workspace`. No `hi/*.md` file is
   opened, parsed, or written here: parsing belongs to `doc`, loading to `workspace`, and writing
   to `capture`. The single file this module writes is `INTENT.md`.
2. A marker counts only when it is alone on its own line: `index_span` and `has_marker_line` both
   compare `line.trim()` to the whole marker, so a marker quoted inside a sentence is prose and is
   copied through untouched (hi: INDEX-2.a). A substring search would have matched that sentence
   and rewritten everything from there to the real close.
3. When `INTENT.md` holds both marker lines in order, `write_index` replaces exactly the span from
   the start of the opening marker's line through the end of the closing marker's text. The closing
   line's own newline is deliberately left outside the span, so the blank line that follows the
   block survives every rewrite. Bytes before the opening marker and after the closing marker are
   copied through unchanged, so the human prose in `INTENT.md` is never rewritten, reflowed, or
   reordered (hi: INDEX-2). The remaining branches have no block to replace: an opening marker line
   with no close is refused and nothing is written (hi: INDEX-2.b), an absent or blank file becomes
   a starter file, and a file with no opening marker line at all keeps its text (`trim_end`ed, so
   trailing blank lines are the one thing that can be lost) and gains the block below it. No
   branch edits human prose in place.
4. `export` emits the same envelope at every scope: `hi`, `scope`, `files`, and per file
   `file`, optional `title`, `intent`, `families`, `criteria`, `retired`. `scope` is the string the
   caller asked for, or `"repo"` when they asked for nothing. A consumer never branches on which
   scope was requested (hi: EXPORT-3). `product` is the one key that varies. It is the
   product-level why from `INTENT.md` and belongs only to a whole-repo export. Inside `criteria`
   and `retired` alike, an entry carries `id`, `text`, `depth`, `parent`, and `retired`; that last
   key is the criterion's `retired:` note, which `doc` records wherever it was written, so it can
   appear on an entry sitting in `criteria`. Whether a criterion is retired is which array it is
   in, never the presence of that key.
5. Incomplete intent is never an error. `ls` with nothing to show, an `export` of an empty
   repository, a file with no `## Intent`, and a family with no criteria all succeed.
6. `issue` refuses a retired id before it renders anything, so a criterion the team changed its
   mind about cannot become a ticket (hi: ISSUE-4).
7. Printing is the default and `--create` is opt-in, so `hi issue` works with no auth, no network,
   and no tracker integration (hi: ISSUE-1.a). `gh` is spawned with an argument vector, never
   through a shell, so a criterion sentence is passed as one argument and is never interpreted.
8. `issue_markdown` reproduces the criterion sentence verbatim; the only transformation applied
   anywhere in this module is trimming trailing `.` from the ticket title. hi never rewrites human
   prose (hi: FILE-4).
9. `index_block` counts active criteria only. Retired ids stay reserved but are not features, so
   they never appear in the `INTENT.md` index.
10. `ls` and `issue_markdown` derive indentation from `Id::depth()`, falling back to depth 1 when a
   criterion's id failed to parse, so a malformed line still renders rather than panicking.
11. This module renders only what the parser accepted as structure. It reads `doc.criteria`,
   `doc.retired`, and `doc.all()` (which is those two chained) and never `doc.stray`. So an
   id-shaped line inside a fenced code block is prose and never appears anywhere here (hi: FILE-9),
   and an id-shaped line outside every section is invisible to `ls`, absent from `export`, and
   uncounted by `index_block`; `hi check` is the verb that reports it (hi: CHECK-2.e). The same
   holds for the family lists in `export` and `index_block`: they are `doc.front.families` exactly
   as the parser recorded it, filled from an inline `families: [A, B]` or from a YAML block list
   alike, so a block-style declaration reaches both rather than falling back to used families.
12. `INTENT.md` gets none of the file hygiene `doc` applies to `hi/*.md`. `write_index` and
   `read_product_intent` read it with `fs::read_to_string` and strip no BOM, so a byte-order mark
   sitting immediately before an opening marker hides that line from both (`U+FEFF` is not
   whitespace, so `line.trim()` keeps it), while `Doc::parse` and workspace discovery do strip one.
   Line endings are not detected either: a marker on a CRLF line is still recognized, because
   `trim()` and `trim_end()` drop the `\r` and the closing marker's `\r\n` therefore stays outside
   the replaced span, but the generated block itself is always written with `\n`. And the write is
   one `fs::write` of the whole file, not the sibling-temp-file-and-rename of `Doc::save`, so
   `INTENT.md` carries no crash-atomicity guarantee.

## Behavioral Examples

### Scenario: A ticket carries its id and its cases but not its siblings

- **Given** `hi/chat.md` holds `SEND-1`, its case `SEND-1.a`, and the sibling `SEND-2`, and the file
  has an `## Intent` section
- **When** `issue_markdown` renders `SEND-1`
- **Then** the title is the sentence without its trailing period, the body contains `hi: SEND-1`
  and a `Cases:` list naming `SEND-1.a`, the body does not mention `SEND-2`, and the file's intent
  prose is appended below a `---` rule

### Scenario: Whole-repo export

- **Given** a workspace holding `hi/chat.md` with `SEND-1`, `SEND-1.a`, and `SEND-2`, and an
  `INTENT.md` at the root carrying hand-written prose around the generated index block
- **When** `export` is called with no scope
- **Then** the JSON has `scope: "repo"`, one entry in `files` carrying all three criteria, the
  second criterion's `parent` is `SEND-1`, the file's `intent` is the prose from `## Intent`, and
  the prose from `INTENT.md`, with the generated index stripped out, appears in `product`
- **And** with no `INTENT.md`, or one that is empty once the index block is stripped, the `product`
  key is simply absent; every other key is identical

### Scenario: Export scoped to a family

- **Given** `hi/chat.md` owning `SEND` and `hi/billing.md` owning `BILLING`
- **When** `export` is called with scope `SEND`
- **Then** `files` holds exactly one entry, `hi/chat.md`, and only `SEND` criteria are inside it

### Scenario: Export scoped to a file

- **Given** the same workspace
- **When** `export` is called with scope `chat` (the file stem, without `.md`)
- **Then** `scope` is `"chat"`, `files` holds exactly one entry, and every criterion in that file
  is kept regardless of which family it belongs to

### Scenario: A scope that is both a file stem and a family matches both

- **Given** `hi/SEND.md` holding only `ALPHA-1`, and `hi/other.md` holding `SEND-1` and `BETA-1`
- **When** `export` is called with scope `SEND`
- **Then** `files` holds two entries: `hi/SEND.md` complete, because the stem matched it, and
  `hi/other.md` filtered to `SEND-1`, because the family matched it. The stem match does not
  suppress the family match elsewhere; it only widens the file it names
- **And** in practice this does not arise, because families are uppercase and file stems are
  lowercase

### Scenario: The feature index

- **Given** `hi/chat.md` declaring `families: [SEND]` with three active criteria
- **When** `index_block` runs
- **Then** the block contains `[chat](hi/chat.md)`, the family name `SEND`, and `(3 criteria)`

### Scenario: Rewriting the index leaves the prose alone

- **Given** an `INTENT.md` whose hand-written paragraphs sit above a `## Features` section
  containing the two `hi:index` markers
- **When** `write_index` runs
- **Then** only the bytes between the markers change, the paragraphs above and anything below the
  closing marker are byte-for-byte identical, including the blank line straight after the closing
  marker, whose newline is left outside the replaced span, and the returned string is the path
  relative to the workspace root

### Scenario: A marker quoted in prose is not the generated block

- **Given** an `INTENT.md` whose prose contains the sentence
  `hi rewrites everything between <!-- hi:index --> and the close.` above a `## Features` section
  holding the real marker pair
- **When** `write_index` runs
- **Then** the span replaced starts at the real opening marker line, the sentence survives
  word for word, and only the stale block between the real markers is replaced (hi: INDEX-2.a)

### Scenario: An opening marker with no close

- **Given** an `INTENT.md` holding `<!-- hi:index -->` on its own line and no `<!-- /hi:index -->`
  line anywhere after it, including the case where the two markers appear in the wrong order
- **When** `write_index` runs
- **Then** it fails with a message naming both markers and telling the person to fix them rather
  than have hi guess where the block ends, and `INTENT.md` is left exactly as it was, because the
  bail happens before the single `fs::write` (hi: INDEX-2.b)

### Scenario: There is no INTENT.md yet

- **Given** a workspace root with no `INTENT.md`, or one that is empty or whitespace-only
- **When** `write_index` runs
- **Then** a starter file is written: an `# <root directory name>` heading, an HTML comment
  prompting for the product-level why, a `## Features` heading, and the generated block

### Scenario: Nothing captured yet

- **Given** a workspace whose `hi/` directory holds no criteria
- **When** `ls` runs
- **Then** it prints `no criteria yet. Capture one with \`hi SEND-1 "..."\`` and returns normally

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `issue` is given a string that is not a valid id | `'<raw>' is not a valid id: <reason>`, carrying whatever `IdError` `Id::parse` returned, including `IdError::PaddedLevel` for a zero-padded number such as `SEND-007` (hi: ID-1.c) |
| `issue` is given a well-formed id that no file holds | `<id> does not exist` |
| `issue` is given a retired id | `<id> is retired, so it should not become work` (hi: ISSUE-4) |
| `issue --create` cannot start `gh` | `running \`gh\` (is the GitHub CLI installed and authenticated?)` wrapping the spawn error |
| `gh issue create` exits non-zero | `gh issue create failed`; `gh`'s own diagnostics have already reached the terminal because its stdio is inherited |
| `export` is given a scope that selects no file | `nothing matches '<scope>', which is not a family or a file in hi/` |
| `export` is given a family that frontmatter declares but no criterion uses | The same `nothing matches '<scope>', which is not a family or a file in hi/`. File selection needs a criterion in `doc.all()`, so a declared-but-unused family reaches no file even though `Workspace::families` lists it, and the message contradicts the frontmatter |
| `issue` is given `--repo` without `--create` | Not an error: the print path returns before `repo` is read, so the flag is silently ignored |
| `export` is given no scope in a repository with no hi files | Not an error: `files` is an empty list and the payload is still emitted |
| `INTENT.md` is missing or unreadable during `export` | Not an error: `product` is omitted from the payload |
| `INTENT.md` cannot be written by `write_index` | `writing <path>` wrapping the underlying I/O error |
| `INTENT.md` has an opening marker line that is never closed, including markers written in the wrong order | `<path> has an opening <!-- hi:index --> with no matching <!-- /hi:index -->. Fix the markers rather than have hi guess where the block ends`. Nothing is written (hi: INDEX-2.b) |
| `INTENT.md` holds only the *closing* marker, or holds markers only inside sentences | Not an error: no opening marker line exists, so the existing text is preserved and a fresh `## Features` section carrying the block is appended below it, producing a second `## Features` heading when the file already had one |
| `INTENT.md` begins with a BOM immediately followed by the opening marker | Not an error: the BOM is not whitespace, so that line is not a marker line to either `index_span` or `has_marker_line` and the append branch runs. `read_product_intent` misses the same line, so the stale block reaches `product` |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `anyhow` | `Result`, `bail!`, and `Context` for the fallible verbs |
| `serde` | `Serialize` derive on the private payload types |
| `serde_json` | `to_string_pretty` for the export envelope |
| `std::fs` | `read_to_string` for `INTENT.md`, `write` for the index |
| `std::process::Command` | Spawning `gh issue create` with inherited stdio |
| `doc` | `Criterion` (`id`, `raw_id`, `text`, `note`, `section`), `Doc` (`criteria`, `retired`, `intent`, `title`, `front.families`, `path`, `all()`, `used_families()`, `name()`), `Section::Retired` |
| `id` | `Id::parse`, `Id::depth`, `Id::parent`, `Id::is_descendant_of`, `Id::family`, and `Display` for error messages |
| `workspace` | `Workspace` (`docs`, `root`), `Workspace::rel`, `Workspace::families`, `Workspace::find_id`, `Workspace::intent_path` |

### Consumed By

| Module | What is used |
|--------|-------------|
| `main` (CLI) | `out::ls` for `hi ls`, `out::issue` for `hi issue`, `out::export` for `hi export`, `out::write_index` for `hi index` |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Verification pass against `src/out.rs`. Corrected the "only three conditions are errors" claim (there are seven), scoped invariant 2 to the splice branch, gave the whole-repo export example the `INTENT.md` its `product` clause assumed, documented that a family/file-stem collision matches both rather than resolving to the file, and added the declared-but-unused family, the ignored `--repo`, and the `retired`-key-is-a-note behaviors. |
| 2026-09-16 | Claude | Reconciled with the bug-fix pass. `write_index` now matches the `hi:index` markers on whole lines via the new private `index_span` / `has_marker_line` (hi: INDEX-2.a) and refuses an unclosed opening marker instead of appending (hi: INDEX-2.b), so the eighth error case is new and the old "only one marker is not an error" row was wrong. Recorded that the closing marker's newline stays outside the replaced span, added the invariant that this module reads only `criteria`/`retired` and never `Doc::stray`, and noted that fenced and stray id-shaped lines never reach any output here (hi: FILE-9, CHECK-2.e). Public API is unchanged at six exports. |
| 2026-09-16 | Claude | Verification pass over the reconciliation. Confirmed the six exports, the marker helpers, and every error case against `src/out.rs`; added what the reconciliation missed. The parser now fills `front.families` from a YAML block list too, so a block-style file's declared families reach `export` and `index_block` (invariant 11). Recorded that `INTENT.md` receives none of the BOM stripping, line-ending detection, or atomic replacement the bug-fix pass gave `hi/*.md` (invariant 12), and added the BOM-before-the-marker row to Error Cases. |
