---
module: capture
version: 1
status: active
files:
  - src/capture.rs

db_tables: []
depends_on:
  - specs/doc/doc.spec.md
  - specs/id/id.spec.md
  - specs/workspace/workspace.spec.md
---

# Capture

## Purpose

Implements hi's default verb (`hi SEND-2 "it reaches them and the mark changes to sent"`), which
adds exactly one criterion to exactly one file. Capture owns the decision of whether a write may
happen at all and which file receives it; the rendering of the line and the shape of the file
belong to `doc`, and the id grammar belongs to `id`.

The governing rule is that **a new id just works and an existing id refuses** (hi: CAPTURE-2,
CAPTURE-3). Capture never prompts, never opens an editor, never asks where to put a new family, and
never requires an init step first (hi: CAPTURE-1, CAPTURE-1.a, CAPTURE-1.b). The four seconds
between having a thought and losing it is the scarce resource, and a question in the middle of
capturing is what loses it. Every refusal happens before any filesystem write, so a rejected
capture leaves the workspace byte-for-byte unchanged (hi: CAPTURE-5).

A sub-id follows its **parent**, not its family declaration: capture resolves the destination to
the file that actually holds the parent before it falls back to the file that declares or uses the
family, so a case is never stranded in a different file from the criterion it is a case of
(hi: CAPTURE-4.a).

Capture stores no state beyond the criterion sentence itself, records no lifecycle, and binds no
evidence. It has one failure mode that is about content (an id that is already spoken for), and
that failure is the only reason it refuses work that is otherwise well-formed. The words the person
typed are the words that land in the file (hi: CAPTURE-9), and the file capture wrote to is always
named back to them (hi: CAPTURE-11).

## Public API

| Export | Description |
|--------|-------------|
| `Captured` | Report of one successful capture: the parsed id, the repository-relative file it landed in, whether that file had to be started, which product-level and agent-facing files this capture started alongside it, and why the generated feature list could not be refreshed when it could not. |
| `capture` | Add one criterion to the workspace, or return an error without writing anything. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Captured` | `#[derive(Debug)]` struct with six public fields: `id: Id` (the parsed id as written), `file: String` (the destination path relative to the workspace root, e.g. `hi/chat.md`), `created_file: bool` (true when the family was not already held by any loaded doc, so capture had to start or adopt a file for it), `started_intent: Option<String>` (the path of an `INTENT.md` this capture created, hi: INDEX-3), `started_agent: Vec<String>` (the `hi/AGENTS.md` and `hi/CLAUDE.md` this capture wrote, hi: HABIT-1), and `index_error: Option<String>` (why the generated feature list could not be refreshed, hi: INDEX-4.a). The last three are all best effort: each is a line for the caller to print and none of them can turn a successful capture into an error. Returned only on success; the caller prints it. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module exports no traits and implements none beyond the derived `Debug` on `Captured`. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `capture` | `capture(workspace: &mut Workspace, raw_id: &str, sentence: &str) -> Result<Captured>` | Validate `raw_id` against the id grammar, reject an empty sentence, refuse an id that already exists anywhere in the workspace (active or retired) with a next-free hint, refuse a sub-id whose parent is absent, then resolve the destination file (the file holding the parent first, the file that declares or uses the family second, and `hi/<family>.md` created or adopted when neither exists), insert the criterion through `Doc::insert`, save the file atomically through `Doc::save`, reload the saved file so what is in memory matches what is on disk, then do three best-effort things in order: start `INTENT.md` from `out::starter_intent_file` if the repository has none, write `hi/AGENTS.md` and `hi/CLAUDE.md` if they are not there, and refresh the generated feature list through `out::refresh_index`, which rewrites a block that is there and installs none. Reports what happened. Returns `anyhow::Error` on every refusal. |

## Invariants

1. Validation strictly precedes mutation. The id parse, the empty-sentence check, the
   already-exists check and the missing-parent check all run and all return before any path is
   created, any file is written, or any in-memory `Doc` is modified (hi: CAPTURE-5).
2. Checks run in a fixed order: id grammar, sentence, duplicate id, missing parent. The reported
   error for an input that fails several checks is therefore deterministic. A malformed id is
   reported as malformed even when its sentence is also empty.
3. An id that already exists is the one and only content condition that refuses an otherwise
   well-formed capture, and the refusal always names both the existing location as `file:line` and
   the next free top-level id in that family (hi: CAPTURE-3).
4. Existence is checked against active and retired criteria alike, because `Workspace::find_id`
   walks `Doc::all()`. A retired id is still spoken for and cannot be captured again. `find_id` sees
   *parsed* criteria only: `Doc::all()` chains `criteria` and `retired`, so a criterion-shaped line
   inside a fenced code block (the parser treats a fence as opaque prose, hi: FILE-9), one outside
   every section (`Doc::stray`), and one in a file `Workspace::load` skipped are all invisible to it
   and none of them raises `Workspace::next_free`. All three are still refused, by the separate
   `Workspace::find_stray` reservation check below, with a different message: an id hi cannot read
   has still been used (hi: CAPTURE-14, FILE-20).
4a. That reservation check is one lookup shared with `check`, so an id `hi check` reports as sitting
   where nothing reads it is always an id capture refuses to hand out. A retired `SEND-1` in
   `hi/Archive.md` was reported by `check` and reissued by capture for four releases, because the
   two walked different sets of files (DECISIONS.md §32).
5. The next-free hint is always a single-level top-level id, `FAMILY-<n>` where `n` is
   `Workspace::next_free(family)`. It is never a case or a step, even when the rejected id was
   nested.
6. A sub-id may only be captured when its parent already exists **somewhere in the workspace**.
   The check is `Workspace::find_id(&id.parent())`, which spans every loaded file and both
   sections. The refusal names the missing parent rather than just reporting a failure
   (hi: CAPTURE-2.b).
7. **The destination is the parent's file first.** `capture` computes
   `parent_file = id.parent().and_then(|p| workspace.find_id(&p).map(|(index, _)| index))` and only
   falls back to `workspace.doc_for_family(&id.family)` when the id has no parent or the parent was
   not found. This keeps a case beside the criterion it is a case of even when the family is
   declared in a different file's frontmatter (hi: CAPTURE-4.a), and it closes the seam with
   `check`: `src/check.rs` builds `present` per doc from `doc.all()` (active *and* retired), so a
   case written next to its parent is never reported as `orphan-case`.
8. An unknown family creates its own file without asking. The path is `hi/<stem>.md`, where the
   stem is the family lowercased with underscores turned into hyphens, so `TWO_FACTOR` becomes
   `hi/two-factor.md`. The file is seeded by `doc::new_file_text` with `hi: 1`, an inline
   `families: [FAMILY]` frontmatter entry, a title derived from the family (`TWO_FACTOR` becomes
   `Two factor`), an `## Intent` heading with an HTML-comment prompt, and an empty `## Criteria`
   heading (hi: CAPTURE-2.a).
9. A missing `hi/` directory is created on demand, with `fs::create_dir_all`, and there is no init
   step (hi: CAPTURE-1, CAPTURE-1.a). It is normally `lock::acquire` that creates it, before the
   read-modify-write rather than in the middle of one: the lock file lives inside `hi/`, so the
   first capture in a repository could not be locked at all until the directory existed
   (DECISIONS.md §33). `capture::capture` still creates it if it is missing when it comes to save,
   which is the path a direct caller such as a unit test takes. Neither creation is a write a
   refusal can reach: capture's happens after every refusal has returned, and the guard removes a
   `hi/` it made itself when the directory is still empty on release (hi: CAPTURE-5). Capture is handed an already-located
   `Workspace`; `Workspace::find` only treats a `hi/` directory as the workspace when it actually
   holds a file with `hi:` frontmatter (hi: CAPTURE-6), and otherwise anchors to the repository
   root it is standing in, which is what leaves a fresh repository's first capture a single
   command. `holds_hi_files` strips a leading BOM before it looks for that key, so an editor-written marker
   hides a workspace from discovery no more than it hides frontmatter from the parser
   (hi: FILE-11).

   **A repository is where discovery stops.** `Workspace::find` walks up from the start directory,
   and at each level it takes a `hi/` that `holds_hi_files` first and a `.git` beside it second.
   The `.git` test does not merely supply a fallback root: it *ends* the walk, so a repository
   nested inside another one captures into its own root instead of adopting the outer project's
   criteria (hi: CAPTURE-10; `cli::a_repository_is_a_boundary_for_discovery`). When neither a
   qualifying `hi/` nor a `.git` is found anywhere above the start directory, discovery fails
   before capture is reached, with `this is not a repository, and no hi/ directory was found above
   it. hi anchors to a repository, so run it inside one`.
10. Capture reads no input other than its two arguments and the files already on disk. It never
    reads stdin, never blocks on a prompt, and never opens an editor (hi: CAPTURE-1.b).
11. Capture trims the sentence and changes nothing else about it (hi: CAPTURE-9). No word is
    reworded, capitalized, punctuated, added or dropped. `doc::render_criterion` then emits it as
    **exactly one line**, a markdown list item of the form `- **<id>**  <sentence>` indented two
    spaces for every level below the first, so `SEND-1` is written at column 0 and `SEND-1.a` two
    spaces in. Interior whitespace runs collapse to single spaces, so a pasted multi-line thought
    becomes one sentence, and the line is never wrapped however long it runs (hi: FILE-6;
    DECISIONS.md §10.1 and §12). The bullet is not decoration: a block of bare lines is joined into
    a single run-together paragraph by every markdown renderer, so the list item is what makes the
    file read as a list wherever it is actually looked at, and the bold id is what keeps it reading
    as a label rather than as the first words of the sentence (hi: FILE-1.b, FILE-1.c). There is no
    wrap width and no continuation line on write; both a bare line and an indented continuation are
    *parsing* concessions for hand-edited files (hi: FILE-14), which is why a fixture written as
    `SEND-1  I hit enter.` still parses even though capture would have written
    `- **SEND-1**  I hit enter.`.
12. Exactly one criterion is added per successful call, to exactly one file, and that file is
    saved before `capture` returns.
13. A case lands directly beneath its parent rather than at the bottom of the file, because
    placement is delegated to `Doc::insert`, which keeps a family's block contiguous and a parent
    immediately followed by its descendants (hi: CAPTURE-4). When the destination file has no
    `## Criteria` heading at all, `Doc::insert` opens one after the file's last content line and
    inserts there, rather than appending into whatever the file happened to end with. It then reads
    the result back and refuses rather than reporting a capture nothing can find, which is what an
    unfinished document gets (hi: FILE-22, doc's REQ-doc-020)
    (hi: CAPTURE-7). A `# `-level heading below the criteria block closes the section as `## ` does
    and records the append point at the end of that block, so a criterion for a family the file
    declares but has not used yet opens its block at the bottom of `## Criteria`, above the later
    heading, instead of at the top of the block.

    The one shape where placement does not hold is a parent that lives only in `## Retired`:
    `insertion_point` scans `self.criteria` only, so the new line joins the end of the family's
    *active* block while `render_criterion` still indents it by its own depth, and it therefore
    renders as a nested item under whichever active criterion happens to be last. Nothing reports
    it. Recorded in Edge Cases in `testing.md`, with the open decision in `tasks.md`.
14. `Captured.file` is always repository-relative via `Workspace::rel`, so the path printed back is
    stable regardless of where the command was run from and always names the file the criterion
    actually landed in (hi: CAPTURE-11). `rel` joins components with a forward slash on every
    platform rather than the host separator, because the same strings reach `hi export` JSON,
    `hi issue` bodies and markdown links (hi: FILE-12).
15. Capture never renumbers, edits, reorders or deletes an existing criterion. Its only mutations
    are the inserted line, the `## Criteria` heading `Doc::insert` opens when the file had none,
    and the frontmatter `families:` entry `Doc::insert` adds when the file did not declare the
    family yet. That entry is rewritten in whichever style the file already uses, because
    `Doc::rewrite_families` branches on `Front::families_block` and replaces exactly
    `Front::families_span` (hi: FILE-7). Inline `families: [A, B]` stays inline, and a YAML block
    list stays a block list.
16. The write is atomic. `Doc::save` goes through `doc::write_atomically`, which creates a sibling
    `.<name>.hi-tmp`, writes, flushes, `sync_all`s and only then renames over the target, removing
    the temp file on any failure. A capture that fails partway through the write leaves the
    person's file byte-for-byte as it was (hi: FILE-8). The scaffold written by `create_file` is a
    plain `fs::write` to a path that did not exist, so it has nothing to destroy.
17. The destination file's own shape survives the capture: `Doc::to_text` rejoins with the line
    ending the file was read with (hi: FILE-10), and a leading UTF-8 BOM is stripped at parse time
    and not written back (hi: FILE-11). One shape is *not* round-tripped. `to_text` ends every
    non-empty file with that line ending. The guard is `if self.trailing_newline ||
    !out.is_empty()`, so `Doc::trailing_newline` only suppresses it for a file with no lines at
    all, and a destination whose last line carried no newline therefore gains one. That final
    byte is the only thing a capture changes outside the line it inserted and the frontmatter
    entry it may add.

18. Everything after `Doc::save` is best effort and cannot fail the capture. Starting `INTENT.md`,
    writing `hi/AGENTS.md` and refreshing the generated feature list all run only once the
    criterion is on disk, and each reports itself as a field on `Captured` rather than as an
    `Err`. The thought is the thing that mattered, and it is already stored: a capture reported as
    a failure sends somebody looking for a criterion that is in fact there (hi: INDEX-3,
    CAPTURE-1.a, INDEX-4.a).
19. The refresh counts what is on disk, not what is in memory. `Doc::insert` splices the rendered
    line into `Doc::lines` and shifts the surrounding indexes, but does not add the criterion to
    `doc.criteria`, so the in-memory `Doc` still describes the file as it was a moment earlier.
    Capture reloads the file it just saved through `Doc::load` before anything counts, which is why
    `index_block` sees the new criterion. That reload is best effort too: if it fails, the refresh
    writes a list that is one short and `hi check` says so afterwards (hi: INDEX-4, INDEX-4.b).
20. `refresh_index` is called inside the caller's lock and takes none of its own. `main::run_capture`
    holds `lock::acquire` across the whole read-modify-write and the lock is not reentrant, so a
    lock taken here would deadlock every writer (hi: FILE-19).

## Behavioral Examples

#### Scenario: A new id in a known family

- **Given** `hi/chat.md` declares `families: [SEND]` and holds `SEND-1`
- **When** `capture(workspace, "SEND-2", "It reaches them.")` runs
- **Then** `hi/chat.md` gains the line `- **SEND-2**  It reaches them.`, and the returned `Captured`
  has `id == SEND-2`, `file == "hi/chat.md"` and `created_file == false`

#### Scenario: A new family starts its own file, without a question

- **Given** no file in `hi/` declares or uses the family `BILLING`
- **When** `capture(workspace, "BILLING-1", "I can see what I paid.")` runs
- **Then** `hi/billing.md` is created containing `families: [BILLING]`, `# Billing` and
  `- **BILLING-1**  I can see what I paid.`; `created_file` is `true`; nothing was asked of the
  person (hi: CAPTURE-2.a, CAPTURE-1.b)

#### Scenario: A case lands under its parent, and renders as a nested list item

- **Given** `hi/chat.md` holds `SEND-1` followed by other content
- **When** `capture(workspace, "SEND-1.a", "If I have no connection it queues.")` runs
- **Then** the new line appears after `SEND-1` in the file, not at the end of it, and it is written
  as `  - **SEND-1.a**  If I have no connection it queues.`, indented two spaces so it renders
  nested under its parent rather than as a sibling (hi: CAPTURE-4, FILE-1.b)

#### Scenario: A case lands in the file that holds its parent, not the one that declares the family

- **Given** `hi/decl.md` declares `families: [SEND]` but holds no criteria, and `hi/real.md` also
  declares `families: [SEND]` and holds `SEND-1`
- **When** `capture(workspace, "SEND-1.a", "A case.")` runs
- **Then** `SEND-1.a` is written into `hi/real.md` beneath `SEND-1`, `hi/decl.md` is untouched, and
  `Captured.file == "hi/real.md"`, because `find_id(SEND-1)` resolves the destination before
  `doc_for_family("SEND")` is consulted (hi: CAPTURE-4.a)

#### Scenario: An id that is already spoken for

- **Given** `hi/chat.md` as `capture::tests::seeded` writes it, where `SEND-1` sits on line 10
- **When** `capture(workspace, "SEND-1", "Something else.")` runs
- **Then** the call returns an error reading
  `SEND-1 already exists in hi/chat.md:10` followed by `hint:  next free is SEND-2`, and
  `hi/chat.md` is unchanged (hi: CAPTURE-3, CAPTURE-5)

#### Scenario: A refusal writes nothing

- **Given** a workspace holding `SEND-1`
- **When** `capture` is called with `SEND-1.a.b` and again with `send-1`
- **Then** both calls return an error and neither sentence appears anywhere in `hi/chat.md`
  (hi: CAPTURE-5)

#### Scenario: The generated feature list is true after the capture

- **Given** an `INTENT.md` whose generated block says `(7 criteria)` while `hi/chat.md` holds one,
  with hand-written prose above and below the markers
- **When** `capture(workspace, "SEND-2", "It reaches them.")` runs
- **Then** the criterion is in `hi/chat.md`, the block reads `(2 criteria)`, both paragraphs of
  prose are still there, and `Captured.index_error` is `None` (hi: INDEX-4)

#### Scenario: The feature list cannot be refreshed, and the capture succeeds anyway

- **Given** an `INTENT.md` holding an opening `<!-- hi:index -->` line with no close, which
  `write_index` refuses to guess past (hi: INDEX-2.b)
- **When** `capture(workspace, "SEND-2", "It reaches them.")` runs
- **Then** the call returns `Ok`, `hi/chat.md` holds `- **SEND-2**  It reaches them.`, `INTENT.md`
  is byte-for-byte what it was, and `Captured.index_error` carries the refusal for `main` to print
  on stderr as a note (hi: INDEX-4.a, CAPTURE-5)

#### Scenario: Capture into a repository with no `hi/` yet

- **Given** a directory holding a `.git` and no `hi/` at all
- **When** `capture` is called from anywhere inside it with the first id of a new family
- **Then** `Workspace::find` stops at the `.git` and anchors there, `hi/` is created, the family
  file is created inside it, and the criterion is written, with no prior `init` command
  (hi: CAPTURE-1, CAPTURE-1.a, CAPTURE-10)

#### Scenario: Outside a repository, capture says so rather than guessing

- **Given** a directory with no qualifying `hi/` and no `.git` anywhere above it
- **When** any command runs, capture included
- **Then** `Workspace::find` fails before capture is reached with `this is not a repository, and no
  hi/ directory was found above it. hi anchors to a repository, so run it inside one`, `main`
  prints it as `error: <message>`, and the process exits 1 (hi: CAPTURE-10)

#### Scenario: A family whose file already exists but does not declare it

- **Given** `hi/billing.md` exists but neither declares `BILLING` in frontmatter nor uses it
- **When** `capture(workspace, "BILLING-1", "I can see what I paid.")` runs
- **Then** the existing file is loaded and adopted rather than overwritten, the criterion is
  inserted into it, `BILLING` is added to its frontmatter `families:` entry by `Doc::insert` in
  whichever style that entry already uses, and `created_file` is reported as `true`

#### Scenario: A destination file with no `## Criteria` section

- **Given** the destination file has frontmatter and prose but no `## Criteria` heading
- **When** `capture` inserts into it
- **Then** `Doc::insert` splices a blank line, `## Criteria` and a blank line in after the file's
  last content line, records that heading, and places the criterion under it, never appending it
  into the trailing prose (hi: CAPTURE-7)

#### Scenario: A flag-looking word after the id stays a word

- **Given** any workspace
- **When** `hi SEND-2 the --root docs option should be documented` runs
- **Then** `peel_root` stops at the first argument that is not a leading `--root`, so nothing is
  taken out of the sentence, and the file gains
  `- **SEND-2**  the --root docs option should be documented`. `--root` is capture's start
  directory only when it comes *before* the id (hi: CAPTURE-8, CAPTURE-9)

#### Scenario: A zero-padded number

- **Given** any workspace
- **When** `hi SEND-007 "padded seven"` runs
- **Then** `Id::parse` returns `IdError::PaddedLevel`, capture refuses with
  `'SEND-007' is not a valid id`, gives the reason `level '007' has a leading zero`, says to
  `write it as '7', so the id always means the same thing`, and nothing is written
  (hi: ID-1.c, CAPTURE-5)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `raw_id` does not parse as an id (bad family charset, missing hyphen, empty or non-alternating level) | Returns `'<raw_id>' is not a valid id`, followed by the `<IdError>` text; nothing is read further and nothing is written |
| `raw_id` carries a zero-padded numeric level, e.g. `SEND-007` | Same path, with `IdError::PaddedLevel` supplying the reason `level '007' has a leading zero` and the instruction to `write it as '7', so the id always means the same thing`; nothing is written (hi: ID-1.c) |
| Sentence is empty or whitespace-only | Returns `a criterion needs a sentence. Say what you actually want`; nothing is written |
| The id already exists, active or retired, in any file | Returns `<id> already exists in <file>:<line>` plus a second line `hint:  next free is <FAMILY-n>`; nothing is written (hi: CAPTURE-3) |
| The id is a sub-id and its parent does not exist | Returns `<id> needs a parent <parent>, which does not exist yet`, naming the missing parent; nothing is written (hi: CAPTURE-2.b) |
| Discovery never reaches capture, because there is no qualifying `hi/` and no `.git` above the start directory | `Workspace::find` returns `this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so run it inside one`; `capture` is never called (hi: CAPTURE-10) |
| `hi/` cannot be created, or the new family file cannot be written | The underlying `std::io::Error` propagates through `anyhow`; the criterion is not inserted |
| An existing file with the target stem cannot be read while being adopted | `Doc::load`'s error propagates as `reading <path>`; the criterion is not inserted. Nothing in `Doc::parse` can fail, so a malformed file is adopted rather than rejected |
| An adopted file has no frontmatter block | `Doc::insert`'s `rewrite_families` refuses with ``<absolute path> has no frontmatter, so add `---\nhi: 1\n---` at the top`` (the `\n` is literal in the message). Nothing is written. The adopted file is not scaffolded over and not saved. `Doc::insert` restores the in-memory `Doc` before returning the error, so the splice it had already made is undone |
| An adopted file ends inside a fence nobody closed | `Doc::insert` reads its own buffer back, finds the criterion is part of the example rather than part of the file, and refuses with `capturing <id> would put it in <path> where hi cannot read it back, so nothing was written` plus a hint naming the line the fence opens on. Nothing is written and the unfinished prose is untouched. This is a refusal, not an I/O failure, so it sits under hi: CAPTURE-5 like every other one (hi: FILE-22.a) |
| `Doc::save` fails on a full disk, on a quota, or on a rename that cannot complete | `write_atomically` deletes its `.<name>.hi-tmp` sibling and the error propagates as `writing <path>`. The destination file is left byte-for-byte as it was; it is never truncated first (hi: FILE-8) |
| `Doc::insert` or `Doc::save` fails after a *new* family file was written | The error propagates. The scaffolded family file has already been written by `create_file`'s `fs::write` at that point and remains on disk, empty of criteria |
| `out::refresh_index` cannot rewrite `INTENT.md`, for any reason including INDEX-2.b's refusal | Not an error. The message is carried out on `Captured.index_error` and `main` prints `note: the feature list in INTENT.md was not refreshed: <text>` on stderr; the exit code stays 0 and the criterion is stored (hi: INDEX-4.a) |
| `Doc::load` fails when capture reloads the file it just saved | Not an error. The in-memory `Doc` is left as `Doc::insert` made it, so the refreshed list is one criterion short. `hi check` reports the list as behind afterwards (hi: INDEX-4.b) |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std::fs` | `create_dir_all` for a missing `hi/`, `write` for a new family file |
| `anyhow` | `Result` for every fallible path, `bail!` for each refusal message |
| `crate::id` | `Id::parse` for grammar validation, `Id::parent` for the parent check, `Id`'s `Display` for messages, and `Level::Number` to build the next-free hint |
| `crate::doc` | `Doc::load` to adopt an existing file, `Doc::insert` for placement under the parent (and for opening a `## Criteria` section when the file has none), `Doc::save` to persist atomically, `new_file_text` for a new family file's scaffold |
| `crate::workspace` | `Workspace::find_id` (duplicate lookup, parent lookup, and destination-file resolution), `next_free`, `doc_for_family`, `rel`, and the `docs`/`dir` fields |
| `crate::out` | `starter_intent_file` and `agent_instructions` for the files a first capture writes, and `refresh_index` to keep the generated feature list true once the criterion is on disk (hi: INDEX-3, HABIT-1, INDEX-4) |

### Consumed By

| Module | What is used |
|--------|-------------|
| `main` (`src/main.rs`) | `capture::capture` in `run_capture(root, raw_id, rest)`, the default action when the first CLI argument is id-shaped; reads `Captured.created_file`, `Captured.file` and `Captured.id` to print `<file>  created` and `<file>  +<id>` (hi: CAPTURE-11), and `Captured.started_intent`, `Captured.started_agent` and `Captured.index_error` for the best-effort lines beside them; `index_error` goes to stderr so stdout stays the record of what landed. `main` reads `args_os` and peels a **leading** `--root PATH` or `--root=PATH` out with `peel_root` before the id-shaped test, so the flag becomes capture's start directory instead of words in the sentence; `peel_root` breaks at the first argument that is not one of those two forms, so from the id onward every argument, `--root` included, is sentence (hi: CAPTURE-8, CAPTURE-9). A non-UTF-8 argument is reported as `that sentence is not valid UTF-8. hi files are text, so it cannot be stored as written` rather than panicking (hi: CAPTURE-1.c) |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Verification pass against source: corrected the rendering invariant (one line, whitespace collapsed, no wrapping), narrowed the orphan-prevention claim to match `check`'s per-file rule, fixed the duplicate-id example's line number, and added the no-frontmatter adoption error case. |
| 2026-09-16 | Claude | Reconciled with the bug-fix pass: destination now resolves to the parent's file before the declared family (hi: CAPTURE-4.a), which closes the capture/`check` orphan seam the old invariant 6 described; `Doc::insert` opens a `## Criteria` section when the file has none (hi: CAPTURE-7); `Doc::save` is atomic (hi: FILE-8); frontmatter style is preserved on rewrite (hi: FILE-7); fenced and stray criterion-shaped lines are invisible to the duplicate check (hi: FILE-9, CHECK-2.e); zero-padded ids refuse (hi: ID-1.c); workspace discovery requires real hi files (hi: CAPTURE-6); `main` peels `--root` and reads `args_os` (hi: CAPTURE-8, CAPTURE-1.c); line endings and BOM (hi: FILE-10, FILE-11). |
| 2026-09-16 | Claude | Verification pass over that reconciliation: corrected invariant 17, because `Doc::to_text` terminates every non-empty file with the detected line ending, so a destination that had no trailing newline gains one rather than having its shape restored (reproduced against the built binary); recorded that `holds_hi_files` also strips a BOM (invariant 9) and that a `# ` heading fixes the append point at the end of the criteria block (invariant 13). |
| 2026-09-16 | Claude | Re-verified every claim against `src/capture.rs` and the release binary after the crate moved under the specs. Corrected the rendering invariant and every example that still quoted the old bare `ID  sentence` line: `doc::render_criterion` now writes `- **ID**  sentence`, indented two spaces per level (hi: FILE-1.b, FILE-1.c; DECISIONS.md §12), and the bare form survives only as a parsing concession (hi: FILE-14). Replaced the stale not-found message with `this is not a repository, and no hi/ directory was found above it...` and recorded that `Workspace::find` now *stops* at a `.git` rather than walking past it (hi: CAPTURE-10). Recorded that `peel_root` consumes only a leading `--root`, so a flag-looking word after the id stays in the sentence (hi: CAPTURE-8, CAPTURE-9). Fixed the empty-sentence message, which reads `Say what you actually want` with a capital S, and quoted the no-frontmatter refusal in full. Added the citations the module had grown into: CAPTURE-2.b (the refusal names the missing parent), CAPTURE-9, CAPTURE-10, CAPTURE-11, FILE-12 (`Workspace::rel` uses forward slashes on every platform) and FILE-14. Recorded that a case under a retired parent renders as a child of an unrelated active criterion. |
| 2026-09-17 | Claude | `Captured` gains `index_error`, and capture refreshes the generated feature list in `INTENT.md` itself once the criterion is on disk, because nothing made anyone run `hi index` and three adopter repositories had drifted (DECISIONS.md §30, hi: INDEX-4, INDEX-4.a). Added invariants 18 to 20, two scenarios, two error rows and REQ-capture-016. Invariant 19 records the reason the count was wrong at first: `Doc::insert` does not add the criterion to `doc.criteria`, so capture now reloads the file it saved before anything counts. This pass also documented `started_intent` and `started_agent`, which capture has carried since 0.5.0 and this spec had never mentioned; the struct row said three fields and there were five. |
| 2026-09-17 | Claude | Restated invariant 9: `hi/` is now created by `lock::acquire`, before the lock rather than during the write, because the lock file lives inside the directory and the first capture in a repository was therefore never locked (hi: FILE-19, DECISIONS.md §33). `capture` keeps its own `create_dir_all` for a direct caller. Noted that a guard which created the directory removes it again while it is empty, so `CAPTURE-5` still holds for a refusal in a repository that had no `hi/`. |
