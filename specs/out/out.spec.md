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
`INTENT.md`, and once that file carries a marker pair the write is confined to the generated block
between them (hi: INDEX-2); the starter and append branches, which run only when there is no
opening marker line at all, are the two that add text of their own. The one external process it may
start is `gh`, and only when the caller asked for it with `--create` (hi: ISSUE-1.b). That one
write is now also reached from outside this module: `refresh_index` is what `capture` and
`hi retire` call once the criterion they wrote is on disk, so the generated list is true after every
command that changes it rather than after somebody remembers `hi index` (hi: INDEX-4).

Nothing here fails because intent is incomplete. `ls` over an empty repository prints a hint and
succeeds; a whole-repo `export` of a repository with no criteria emits an empty `files` list and
succeeds. The only intent-shaped errors are requests that cannot be answered: an unparsable id, an
unknown id, a retired id asked to become work, and a scope that selects no file. The one file this
module writes adds a fifth: an `INTENT.md` carrying an opening `hi:index` marker with no matching
close is refused rather than guessed at, because guessing there would eat the prose between the two
(hi: INDEX-2.b). Beyond those, the environment can still fail. `gh` may not start or may exit
non-zero, and the directory holding `INTENT.md` may refuse the sibling temporary file that the
atomic write goes through. Error Cases below lists all eight.

## Public API

| Export | Description |
|--------|-------------|
| `ls` | Print every criterion grouped by file, indented by id depth, optionally filtered to one family and optionally including retired lines. |
| `issue_markdown` | Render one criterion as a `(title, body)` ticket pair carrying its id, its cases, and the file's intent prose with its soft line breaks unwrapped for a medium that renders a newline as a break. |
| `issue` | Resolve an id and print its ticket, or hand the same ticket to `gh issue create` when `create` is set. |
| `export` | Build the agent payload as pretty-printed JSON for a family, a file named by its stem, its file name, or any path ending in `hi/<stem>.md`, or the whole repository. |
| `index_block` | Build the generated feature list: one Markdown bullet per hi file, with its families and active-criterion count. |
| `starter_intent` | The opening of a product-level `INTENT.md`: a title and a prompt for the holistic why, on one line. Shared with `capture`, which creates the file on the first capture so nobody has to discover it (hi: INDEX-3). |
| `agent_instructions` | The text of `hi/AGENTS.md`: the habit an agent follows before building, and nothing else. Shared with `capture`, which writes it on the first capture so an agent finds it without being told (hi: HABIT-1, HABIT-2, HABIT-3). It carries no id grammar, no file format and no list of existing families, because the file is written once and never rewritten, so anything hi could change underneath it would be wrong later with nothing to notice (DECISIONS.md §27). It says one thing about form, which is that prose here is one line per paragraph: that is how markdown reads a newline rather than anything about hi's format, so it cannot go stale with a format that is not frozen (DECISIONS.md §29, hi: FILE-21). The text is itself one line per paragraph (hi: FILE-21.a). |
| `write_index` | Rewrite the generated block inside `INTENT.md`, matching the `hi:index` markers on whole lines only, creating the file or the `## Features` section when they do not exist yet, and refusing when an opening marker has no close; the replacement goes through `doc::write_atomically`, and the path written is returned relative to the workspace root. |
| `refresh_index` | Run `write_index` for a verb that has just changed the live count, handing back the reason it could not be done instead of raising it. Called by `capture` and by `hi retire` after the criterion they wrote is already on disk, so the list at the front of the product stays true without anyone remembering to run `hi index` (hi: INDEX-4, INDEX-4.a). Takes no lock of its own. |
| `index_note` | Read-only. What to say when the generated block in `INTENT.md` no longer matches the workspace, or when a broken marker pair means nothing can refresh it. Used by `check` as a `note:`, never as a problem and never as an exit code (hi: INDEX-4.b, CHECK-1). |

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
| `issue_markdown` | `fn issue_markdown(doc: &Doc, criterion: &Criterion) -> (String, String)` | Title is the criterion sentence with trailing `.` characters trimmed. Body is the sentence, a blank line, `hi: <raw_id>`, an optional `Cases:` bullet list of the criterion's active descendants, each line written as `- `, then two spaces per level below the subject, then `<raw_id> <text>`; the indent is emitted *after* the list marker, so in source a deeper case is offset and in rendered Markdown it is a peer. When `doc.intent` is not blank the body ends with a `---` rule, `Intent for <file stem>:`, and the prose, passed through the private `unwrap_soft_breaks` first (hi: ISSUE-5, ISSUE-7). |
| `issue` | `fn issue(workspace: &Workspace, raw_id: &str, create: bool, repo: Option<&str>) -> Result<()>` | Parses `raw_id`, finds the criterion in the workspace (active or retired), refuses a retired one, then either prints `## <title>` followed by the body or spawns `gh issue create --title <title> --body <body>`, adding `--repo <repo>` when supplied. |
| `export` | `fn export(workspace: &Workspace, scope: Option<&str>) -> Result<String>` | Selects the files the scope reaches, filters their criteria when the scope is a family, and serializes one envelope (`hi`, `scope`, an optional `product`, and `files`) as pretty-printed JSON. File selection goes through the private `matches_file`, which accepts the stem, `<stem>.md`, or any string that ends with `hi/<stem>.md` once a leading `./` is trimmed, so the repository-relative path `ls` and `export` themselves print is accepted back. Errors only when a stated scope selected no file at all, which includes a family declared in frontmatter that no criterion actually uses. |
| `index_block` | `fn index_block(workspace: &Workspace) -> String` | Emits `- [<stem>](hi/<stem>.md): <families> (<n> criteria)` per file, using frontmatter families when declared and used families otherwise, `no families yet` when neither exists, and singular `criterion` at a count of one. An empty workspace yields `- nothing captured yet`. |
| `write_index` | `fn write_index(workspace: &Workspace) -> Result<String>` | Splices the generated block into `INTENT.md` between `<!-- hi:index -->` and `<!-- /hi:index -->` when the private `index_span` finds both on lines of their own, in that order; bails when the private `has_marker_line` still sees an opening marker line that `index_span` could not close; writes a whole starter file when `INTENT.md` is absent or blank (hi: INDEX-1.a); otherwise appends a `## Features` section carrying the block to the existing prose. Whichever branch runs, the result reaches disk through `doc::write_atomically`: a sibling `.INTENT.md.hi-tmp`, flushed and fsynced, then renamed over the target. |
| `refresh_index` | `fn refresh_index(workspace: &Workspace) -> Option<String>` | `write_index(workspace).err().map(|err| format!("{err:#}"))`. Every failure, including INDEX-2.b's refusal to guess at a broken marker pair, comes back as a string for the caller to print rather than as an `Err`. Acquires no lock: `capture` and `hi retire` already hold `lock::acquire` across their whole read-modify-write and it is not reentrant. |
| `index_note` | `fn index_note(workspace: &Workspace) -> Option<String>` | Reads `INTENT.md`. With both marker lines present, compares the bytes between them to `INDEX_OPEN` + `index_block` + `INDEX_CLOSE` and returns `<path>'s feature list is behind what is captured. Run \`hi index\`` when they differ. With an opening marker line that `index_span` could not close, returns `<path> has an opening <!-- hi:index --> with no matching <!-- /hi:index -->, so nothing can refresh its feature list`. Returns `None` for a missing or unreadable `INTENT.md`, for one with no marker line at all, and for a block that already matches. Writes nothing. |

## Invariants

1. Every verb in this module is a pure read of an already-loaded `Workspace`. No `hi/*.md` file is
   opened, parsed, or written here: parsing belongs to `doc`, loading to `workspace`, and writing
   to `capture`. The single file this module writes is `INTENT.md`, and the only other path it
   touches is the sibling `.INTENT.md.hi-tmp` that `doc::write_atomically` creates and renames
   away, or deletes when the write fails.
2. A marker counts only when it is alone on its own line: `index_span` and `has_marker_line` both
   compare `line.trim()` to the whole marker, so a marker quoted inside a sentence is prose and is
   copied through untouched. A substring search would have matched that sentence and rewritten
   everything from there to the real close. Fenced blocks are opaque too: `index_span` and
   `has_marker_line` both track ``` fences and ignore any marker inside one, so a person can show
   an example of the index in their own prose and `hi index` will rewrite the real block below it
   rather than the illustration. That closed in 0.2.0, and INDEX-2.a's promise ("even on a line of
   their own or inside a code block") is delivered.
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
   in, never the presence of that key. Retired criteria travel at every scope, in their own array
   rather than mixed into `criteria` (hi: EXPORT-4).
5. Incomplete intent is never an error. `ls` with nothing to show, an `export` of an empty
   repository, a file with no `## Intent`, and a family with no criteria all succeed.
6. `issue` refuses a retired id before it renders anything, so a criterion the team changed its
   mind about cannot become a ticket (hi: ISSUE-4).
7. Printing is the default and `--create` is opt-in, so `hi issue` works with no auth, no network,
   and no tracker integration (hi: ISSUE-1.a). `gh` is spawned with an argument vector, never
   through a shell, so a criterion sentence is passed as one argument and is never interpreted.
8. `issue_markdown` reproduces the criterion sentence verbatim. Two transformations exist in this
   module and both are render-time, on the way into a ticket that nobody will ever read as a file:
   trailing `.` is trimmed from the title, and the intent prose goes through the private
   `unwrap_soft_breaks`, which folds a newline inside a paragraph into a space and leaves every
   newline that means something — a blank line, a list item, a block quote, a heading, a table row,
   a rule, the inside of a fence, and a line that ends in an explicit hard break. It exists because
   a GitHub issue body is rendered with hard line breaks on, so where a person wrapped their own
   prose becomes a visible break and a wide pane shows a narrow column (hi: ISSUE-7, ISSUE-7.a,
   ISSUE-7.b). The file is never touched. hi does not rewrite, reflow or reformat prose somebody
   wrote (hi: FILE-4), there is no rewrite-on-write anywhere in hi, and `check` has no opinion on
   wrapping (DECISIONS.md §29).
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
12. `INTENT.md` gets most, but not all, of the file hygiene `doc` applies to `hi/*.md`.
   `write_index` and `read_product_intent` read it with `fs::read_to_string` and strip no BOM, so a
   byte-order mark sitting immediately before an opening marker hides that line from both
   (`U+FEFF` is not whitespace, so `line.trim()` keeps it), while `Doc::parse` and workspace
   discovery do strip one. Line endings are not detected either: a marker on a CRLF line is still
   recognized, because `trim()` and `trim_end()` drop the `\r` and the closing marker's `\r\n`
   therefore stays outside the replaced span, but the generated block itself is always written with
   `\n`. The write, however, is now the same sibling-temp-file-and-rename `Doc::save` uses
   (`doc::write_atomically`), so a write that fails partway cannot leave `INTENT.md` truncated
   (hi: INDEX-2, FILE-8). Two consequences follow from the rename: the directory has to be
   writable, not the file, so a read-only `INTENT.md` is replaced rather than refused; and the new
   file carries the temporary file's permission bits rather than the old file's.
13. The text this module hands to `capture` for a file hi creates is itself one line per paragraph,
   because the first file an adopter reads is the one they write the rest of their prose to match
   (hi: FILE-21.a). `agent_instructions` and `starter_intent` both pass through `unwrap_soft_breaks`
   unchanged, and a test asserts that rather than leaving it to a reader's eye. The convention
   travels this way and by documentation, never by hi reformatting anybody's file.
14. `refresh_index` and `index_note` add no new way for `INTENT.md` to be rewritten. `refresh_index`
   is `write_index` with the `Result` turned into an `Option<String>`, so every invariant above
   about what is replaced, what is preserved and what is refused holds unchanged; the only
   difference is who carries the failure. `index_note` writes nothing at all. Neither takes a lock:
   the two verbs that call `refresh_index` already hold `lock::acquire` across their whole
   read-modify-write, and the lock is not reentrant, so taking one here would deadlock every writer
   (hi: FILE-19, INDEX-2, INDEX-4).
15. `index_note` compares whole bytes, not counts. It rebuilds `INDEX_OPEN` + `index_block` +
   `INDEX_CLOSE` and compares it to the span `index_span` found, so it answers the same question
   `scripts/index-is-current.sh` answers by regenerating: would running `hi index` change anything?
   A file whose block still carries CRLF endings inside it therefore reads as behind until it has
   been through `write_index` once, which is true rather than a false positive.

## Behavioral Examples

### Scenario: A ticket carries its id and its cases but not its siblings

- **Given** `hi/chat.md` holds `SEND-1`, its case `SEND-1.a`, and the sibling `SEND-2`, and the file
  has an `## Intent` section
- **When** `issue_markdown` renders `SEND-1`
- **Then** the title is the sentence without its trailing period, the body contains `hi: SEND-1`
  and a `Cases:` list naming `SEND-1.a`, the body does not mention `SEND-2`, and the file's intent
  prose is appended below a `---` rule (hi: ISSUE-2, ISSUE-3, ISSUE-5)
- **And** with a deeper tree, `SEND-1.a.1` is written as `  - **SEND-1.a.1**  <text>`: the depth
  indent sits *before* the bullet, so a renderer nests it under `SEND-1.a` instead of showing a flat
  list of peers. That is ISSUE-3.a, and it landed in 0.2.0

### Scenario: Intent prose a person wrapped at their own margin

- **Given** `hi/host.md` whose `## Intent` holds two paragraphs, each hard-wrapped across several
  lines at about 76 columns, with one blank line between them
- **When** `issue_markdown` renders any criterion from that file
- **Then** each paragraph arrives in the body as a single line, with every authored wrap replaced by
  one space, and the blank line between the two paragraphs is still there (hi: ISSUE-7, ISSUE-7.a)
- **And** `hi/host.md` on disk is byte for byte what it was, because this module never writes a hi
  file at all (hi: FILE-4)
- **And** a list, a fenced example, a block quote, a heading, a table row, a rule and a line ending
  in two spaces all keep their own newlines, because those are the newlines somebody meant
  (hi: ISSUE-7.b)

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
- **And** `chat.md`, `hi/chat.md`, and `./hi/chat.md` all select the same file, so the
  repository-relative path that `ls` and `export` themselves print can be pasted straight back in.
  `scope` in the payload echoes whichever spelling was asked for
- **And** the path arm is a suffix test, not a path resolution: any string ending in `hi/chat.md`
  matches, including `/anywhere/hi/chat.md` and `xhi/chat.md`, while `HI/chat.md` and
  `hi\chat.md` do not, because the comparison is case-sensitive and the separator is the forward
  slash `Workspace::rel` emits on every platform (hi: FILE-12)

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
  bail happens before anything is written and no temporary file is created (hi: INDEX-2.b)

### Scenario: Markers alone on their own lines inside a fenced code block

- **Given** an `INTENT.md` that documents the format by putting `<!-- hi:index -->` and
  `<!-- /hi:index -->` on lines of their own inside a ``` fence, above the real marker pair
- **When** `write_index` runs
- **Then** the fenced pair is skipped and the real pair below it is the block: the illustration is
  left byte-identical and the generated list lands where it belongs. `index_span` tracks ``` fences
  and ignores any marker inside one (hi: INDEX-2.a)

### Scenario: There is no INTENT.md yet

- **Given** a workspace root with no `INTENT.md`, or one that is empty or whitespace-only
- **When** `write_index` runs
- **Then** a starter file is written: an `# <root directory name>` heading, an HTML comment
  prompting for the product-level why, a `## Features` heading, and the generated block
  (hi: INDEX-1.a)
- **And** a whole-repo `export` of that repository carries that starter text in `product`, comment
  and `## Features` heading included: `read_product_intent` strips the generated block and nothing
  else, and it does not use `view::strip_comments`

### Scenario: A verb that changed the live count refreshes the list

- **Given** an `INTENT.md` whose generated block says `(1 criterion)` while the workspace now holds
  two, because a criterion was just captured or retired
- **When** `refresh_index` runs, which `capture` and `hi retire` both do after the criterion they
  wrote is already on disk
- **Then** the block reads `(2 criteria)`, the prose either side of the markers is byte-for-byte
  what it was, and `None` comes back because nothing went wrong (hi: INDEX-4)

### Scenario: A refusal the caller must survive

- **Given** an `INTENT.md` holding an opening marker line with no close, and a capture that has
  just stored its criterion
- **When** `refresh_index` runs
- **Then** `write_index` refuses as it always has, nothing is written, and the refusal comes back
  as `Some(<message>)` rather than as an `Err`, so the caller prints a line and still succeeds
  (hi: INDEX-2.b, INDEX-4.a)

### Scenario: A list that is behind because somebody typed a criterion in

- **Given** a `hi/chat.md` with a second criterion written into it by hand, which `FILE-14` allows
  and no verb can see, and an `INTENT.md` whose block still counts one
- **When** `index_note` runs
- **Then** it returns `INTENT.md's feature list is behind what is captured. Run \`hi index\``,
  `check` prints it as a `note:`, and the exit code is 0 because a note is not a problem
  (hi: INDEX-4.b, CHECK-1)
- **And** the note goes away once `hi index` has been run, and never appears for a block that
  already matches or for an `INTENT.md` with no block in it at all

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
| `export` is given a scope that selects no file | `nothing matches '<scope>'. Give a family like SEND, a file like chat, or nothing at all for the whole repository` |
| `export` is given a family that frontmatter declares but no criterion uses | The same `nothing matches '<scope>'. Give a family like SEND, a file like chat, or nothing at all for the whole repository`. File selection needs a criterion in `doc.all()`, so a declared-but-unused family reaches no file even though `Workspace::families` lists it. The message no longer claims the family does not exist, but it still tells the person to give a family when they gave one, and never says the family is declared and empty |
| `issue` is given `--repo` without `--create` | Not an error: the print path returns before `repo` is read, so the flag is silently ignored |
| `export` is given no scope in a repository with no hi files | Not an error: `files` is an empty list and the payload is still emitted |
| `INTENT.md` is missing or unreadable during `export` | Not an error: `product` is omitted from the payload |
| `INTENT.md` cannot be written by `write_index` | `writing <path>` wrapping the underlying I/O error. The failing operation is the sibling temporary file or the rename, so what has to be writable is the directory; the temporary file is removed on the way out and `INTENT.md` keeps its old bytes |
| `INTENT.md` is read-only but its directory is writable | Not an error: the atomic rename replaces it, and the file comes back carrying the temporary file's permission bits rather than its own |
| `INTENT.md` has an opening marker line that is never closed, including markers written in the wrong order | `<path> has an opening <!-- hi:index --> with no matching <!-- /hi:index -->. Fix the markers rather than have hi guess where the block ends`. Nothing is written (hi: INDEX-2.b) |
| `INTENT.md` holds only the *closing* marker, or holds markers only inside sentences | Not an error: no opening marker line exists, so the existing text is preserved and a fresh `## Features` section carrying the block is appended below it, producing a second `## Features` heading when the file already had one |
| `INTENT.md` begins with a BOM immediately followed by the opening marker | Not an error: the BOM is not whitespace, so that line is not a marker line to either `index_span` or `has_marker_line` and the append branch runs. `read_product_intent` misses the same line, so the stale block reaches `product` |
| `INTENT.md` carries the two markers on lines of their own inside a fenced code block | Not an error: `index_span` tracks ``` fences and skips any marker inside one, so the illustration is left byte-identical and the real pair below it is rewritten. Exit 0 (hi: INDEX-2.a) |
| Any of the above reached through `refresh_index` rather than `hi index` | Not an error at the call site: the message comes back as `Some(<text>)` and `capture` and `hi retire` print it on stderr as `note: the feature list in INTENT.md was not refreshed: <text>` and still exit 0. The criterion is already on disk, and nothing about `INTENT.md` may turn a capture that stored one into a reported failure (hi: INDEX-4.a) |
| `INTENT.md` is missing, unreadable, or holds no marker line at all, during `index_note` | Not an error and not a note: there is no generated list to be behind. `check` already nags about a missing `INTENT.md` through `product_intent_note`, and the next capture appends a block the way `hi index` would |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `anyhow` | `Result`, `bail!`, and `Context` for the fallible verbs |
| `serde` | `Serialize` derive on the private payload types |
| `serde_json` | `to_string_pretty` for the export envelope |
| `std::fs` | `read_to_string` for `INTENT.md`. The index write goes through `doc::write_atomically`, so this module calls no `fs` write function itself |
| `std::process::Command` | Spawning `gh issue create` with inherited stdio |
| `doc` | `Criterion` (`id`, `raw_id`, `text`, `note`, `section`), `Doc` (`criteria`, `retired`, `intent`, `title`, `front.families`, `path`, `all()`, `used_families()`, `name()`), `Section::Retired`, and `write_atomically` for the `INTENT.md` replacement |
| `id` | `Id::parse`, `Id::depth`, `Id::parent`, `Id::is_descendant_of`, `Id::family`, and `Display` for error messages |
| `workspace` | `Workspace` (`docs`, `root`), `Workspace::rel`, `Workspace::families`, `Workspace::find_id`, `Workspace::intent_path` |

### Consumed By

| Module | What is used |
|--------|-------------|
| `main` (CLI) | `out::ls` for `hi ls`, `out::issue` for `hi issue`, `out::export` for `hi export`, `out::write_index` for `hi index`, and `out::refresh_index` after `hi retire` has moved a criterion |
| `capture` | `out::starter_intent` and `out::agent_instructions` for the files a first capture writes, and `out::refresh_index` once the new criterion is on disk (hi: INDEX-4) |
| `check` | `out::index_note`, as a `note:` that never reaches the exit code (hi: INDEX-4.b) |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Verification pass against `src/out.rs`. Corrected the "only three conditions are errors" claim (there are seven), scoped invariant 2 to the splice branch, gave the whole-repo export example the `INTENT.md` its `product` clause assumed, documented that a family/file-stem collision matches both rather than resolving to the file, and added the declared-but-unused family, the ignored `--repo`, and the `retired`-key-is-a-note behaviors. |
| 2026-09-16 | Claude | Reconciled with the bug-fix pass. `write_index` now matches the `hi:index` markers on whole lines via the new private `index_span` / `has_marker_line` (hi: INDEX-2.a) and refuses an unclosed opening marker instead of appending (hi: INDEX-2.b), so the eighth error case is new and the old "only one marker is not an error" row was wrong. Recorded that the closing marker's newline stays outside the replaced span, added the invariant that this module reads only `criteria`/`retired` and never `Doc::stray`, and noted that fenced and stray id-shaped lines never reach any output here (hi: FILE-9, CHECK-2.e). Public API is unchanged at six exports. |
| 2026-09-16 | Claude | Verification pass over the reconciliation. Confirmed the six exports, the marker helpers, and every error case against `src/out.rs`; added what the reconciliation missed. The parser now fills `front.families` from a YAML block list too, so a block-style file's declared families reach `export` and `index_block` (invariant 11). Recorded that `INTENT.md` receives none of the BOM stripping, line-ending detection, or atomic replacement the bug-fix pass gave `hi/*.md` (invariant 12), and added the BOM-before-the-marker row to Error Cases. |
| 2026-09-16 | Claude | Re-verified every claim against `src/out.rs` and `./target/release/hi`. Quoted `export`'s new refusal byte for byte; documented `matches_file`, so a file scope is now the stem, `<stem>.md`, or any path ending in `hi/<stem>.md`; recorded that `write_index` writes through `doc::write_atomically`, which makes invariant 12's no-atomicity claim and the read-only-file error row obsolete and turns a read-only `INTENT.md` into a successful replace. Added two places where the code does not meet a criterion as it now reads: a fenced marker pair is still adopted as the real block (hi: INDEX-2.a), and `issue_markdown` writes the depth indent after the `- `, so a case does not render nested (hi: ISSUE-3.a). Added the ISSUE-5, EXPORT-4, INDEX-1.a, and FILE-12 citations the criteria now support. Public API is unchanged at six exports. |
| 2026-09-17 | Claude | `issue_markdown` unwraps the soft line breaks in the intent prose it carries. A GitHub issue body is rendered with hard line breaks on, so the wrapping a person applied in their own editor arrived as a `<br>` after every line and the ticket read as a narrow column down a wide pane; `CorvidLabs/corvid-bot`'s `hi/host.md` is where it was seen. Added REQ-out-015 for the rule and what it deliberately leaves alone, REQ-out-016 for hi's own written text being one line per paragraph, invariant 13, and a scenario. `unwrap_soft_breaks`, `block_start`, `is_heading`, `is_thematic_break` and `is_list_item` are private; the Public API is unchanged at eight exports. `hi view` was checked and left alone: HTML collapses a newline to a space, so `view::paragraphs` already renders these paragraphs whole. `export` was checked and deliberately left verbatim; the payload is a transport rather than a rendering, no JSON consumer turns a `\n` into a break, and unwrapping there would discard the author's wrapping irreversibly for every downstream reader (DECISIONS.md §29). |
| 2026-09-17 | Claude | `refresh_index` and `index_note` added, at nine and ten exports. The generated list is regenerated by `capture` and `hi retire` themselves rather than waiting for `hi index`, because nothing made anyone run it and three adopter repositories had already drifted (DECISIONS.md §30, hi: INDEX-4). The refresh hands its failure back as an `Option<String>` so a capture that stored a criterion can never be reported as a failure, INDEX-2.b's refusal included (hi: INDEX-4.a). `index_note` is the read-only half, for the hand-written criterion no verb can see (hi: INDEX-4.b). Nothing about what gets rewritten moved: `refresh_index` is `write_index` with the error type changed, so invariants 2, 3 and 12 stand as written. Added REQ-out-017 and REQ-out-018, invariants 14 and 15, three scenarios and two error rows. |
