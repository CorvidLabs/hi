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
that failure is the only reason it refuses work that is otherwise well-formed.

## Public API

| Export | Description |
|--------|-------------|
| `Captured` | Report of one successful capture: the parsed id, the repository-relative file it landed in, and whether that file had to be started. |
| `capture` | Add one criterion to the workspace, or return an error without writing anything. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Captured` | `#[derive(Debug)]` struct with three public fields: `id: Id` (the parsed id as written), `file: String` (the destination path relative to the workspace root, e.g. `hi/chat.md`), and `created_file: bool` (true when the family was not already held by any loaded doc, so capture had to start or adopt a file for it). Returned only on success; the caller prints it. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module exports no traits and implements none beyond the derived `Debug` on `Captured`. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `capture` | `capture(workspace: &mut Workspace, raw_id: &str, sentence: &str) -> Result<Captured>` | Validate `raw_id` against the id grammar, reject an empty sentence, refuse an id that already exists anywhere in the workspace (active or retired) with a next-free hint, refuse a sub-id whose parent is absent, then resolve the destination file (the file holding the parent first, the file that declares or uses the family second, and `hi/<family>.md` created or adopted when neither exists), insert the criterion through `Doc::insert`, save the file atomically through `Doc::save`, and report what happened. Returns `anyhow::Error` on every refusal. |

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
   walks `Doc::all()`. A retired id is still spoken for and cannot be captured again. It is checked
   against *parsed* criteria only: `Doc::all()` chains `criteria` and `retired`, so a criterion-shaped
   line inside a fenced code block (the parser treats a fence as opaque prose, hi: FILE-9) and a
   criterion-shaped line outside every section (`Doc::stray`) are both invisible to capture. Neither
   makes an id taken, and neither raises `Workspace::next_free`.
5. The next-free hint is always a single-level top-level id, `FAMILY-<n>` where `n` is
   `Workspace::next_free(family)`. It is never a case or a step, even when the rejected id was
   nested.
6. A sub-id may only be captured when its parent already exists **somewhere in the workspace**.
   The check is `Workspace::find_id(&id.parent())`, which spans every loaded file and both
   sections.
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
9. A missing `hi/` directory is created on demand by `create_file`, with `fs::create_dir_all`.
   There is no init step (hi: CAPTURE-1, CAPTURE-1.a). Capture is handed an already-located
   `Workspace`; `Workspace::find` only treats a `hi/` directory as the workspace when it actually
   holds a file with `hi:` frontmatter (hi: CAPTURE-6), and otherwise falls back to the repository
   root, which is what leaves a fresh repository's first capture a single command.
   `holds_hi_files` strips a leading BOM before it looks for that key, so an editor-written marker
   hides a workspace from discovery no more than it hides frontmatter from the parser
   (hi: FILE-11).
10. Capture reads no input other than its two arguments and the files already on disk. It never
    reads stdin, never blocks on a prompt, and never opens an editor (hi: CAPTURE-1.b).
11. Capture trims the sentence and changes nothing else about it. `doc::render_criterion` then
    emits it as **exactly one line**, `<id>  <sentence>`, however long the sentence runs, with
    interior whitespace runs collapsed to single spaces so a pasted multi-line thought becomes one
    sentence (hi: FILE-6; DECISIONS.md §10.1). There is no wrapping, no wrap width and no
    continuation line on write; indented continuation lines are a *parsing* concession for
    hand-edited files. No word is reworded, capitalized, punctuated, added or dropped.
12. Exactly one criterion is added per successful call, to exactly one file, and that file is
    saved before `capture` returns.
13. A case lands directly beneath its parent rather than at the bottom of the file, because
    placement is delegated to `Doc::insert`, which keeps a family's block contiguous and a parent
    immediately followed by its descendants (hi: CAPTURE-4). When the destination file has no
    `## Criteria` heading at all, `Doc::insert` opens one after the file's last content line and
    inserts there, rather than appending into whatever the file happened to end with
    (hi: CAPTURE-7). A `# `-level heading below the criteria block closes the section as `## ` does
    and records the append point at the end of that block, so a criterion for a family the file
    declares but has not used yet opens its block at the bottom of `## Criteria`, above the later
    heading, instead of at the top of the block.
14. `Captured.file` is always repository-relative via `Workspace::rel`, so output is stable
    regardless of where the command was run from.
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

## Behavioral Examples

#### Scenario: A new id in a known family

- **Given** `hi/chat.md` declares `families: [SEND]` and holds `SEND-1`
- **When** `capture(workspace, "SEND-2", "It reaches them.")` runs
- **Then** `hi/chat.md` gains the line `SEND-2  It reaches them.`, and the returned `Captured` has
  `id == SEND-2`, `file == "hi/chat.md"` and `created_file == false`

#### Scenario: A new family starts its own file, without a question

- **Given** no file in `hi/` declares or uses the family `BILLING`
- **When** `capture(workspace, "BILLING-1", "I can see what I paid.")` runs
- **Then** `hi/billing.md` is created containing `families: [BILLING]`, `# Billing` and
  `BILLING-1  I can see what I paid.`; `created_file` is `true`; nothing was asked of the person
  (hi: CAPTURE-2.a, CAPTURE-1.b)

#### Scenario: A case lands under its parent

- **Given** `hi/chat.md` holds `SEND-1` followed by other content
- **When** `capture(workspace, "SEND-1.a", "If I have no connection it queues.")` runs
- **Then** the new `SEND-1.a` line appears after `SEND-1` in the file, not at the end of it
  (hi: CAPTURE-4)

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

#### Scenario: Capture into a repository with no `hi/` yet

- **Given** a workspace whose `hi/` directory does not exist
- **When** `capture` is called with the first id of a new family
- **Then** `hi/` is created, the family file is created inside it, and the criterion is written,
  with no prior `init` command (hi: CAPTURE-1, CAPTURE-1.a)

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
| Sentence is empty or whitespace-only | Returns `a criterion needs a sentence`, followed by `say what you actually want`; nothing is written |
| The id already exists, active or retired, in any file | Returns `<id> already exists in <file>:<line>` plus a second line `hint:  next free is <FAMILY-n>`; nothing is written (hi: CAPTURE-3) |
| The id is a sub-id and its parent does not exist | Returns `<id> needs a parent <parent>, which does not exist yet`; nothing is written |
| `hi/` cannot be created, or the new family file cannot be written | The underlying `std::io::Error` propagates through `anyhow`; the criterion is not inserted |
| An existing file with the target stem cannot be read while being adopted | `Doc::load`'s error propagates as `reading <path>`; the criterion is not inserted. Nothing in `Doc::parse` can fail, so a malformed file is adopted rather than rejected |
| An adopted file has no frontmatter block | `Doc::insert`'s `rewrite_families` refuses with `<absolute path> has no frontmatter`, followed by an instruction to add a `---`, `hi: 1`, `---` block at the top. Nothing is written. The adopted file is not scaffolded over and not saved. Note the in-memory `Doc` has already been mutated (a `## Criteria` section may have been opened and the criterion line spliced in) at that point, but nothing reaches disk because `save` is never reached |
| `Doc::save` fails on a full disk, on a quota, or on a rename that cannot complete | `write_atomically` deletes its `.<name>.hi-tmp` sibling and the error propagates as `writing <path>`. The destination file is left byte-for-byte as it was; it is never truncated first (hi: FILE-8) |
| `Doc::insert` or `Doc::save` fails after a *new* family file was written | The error propagates. The scaffolded family file has already been written by `create_file`'s `fs::write` at that point and remains on disk, empty of criteria |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std::fs` | `create_dir_all` for a missing `hi/`, `write` for a new family file |
| `anyhow` | `Result` for every fallible path, `bail!` for each refusal message |
| `crate::id` | `Id::parse` for grammar validation, `Id::parent` for the parent check, `Id`'s `Display` for messages, and `Level::Number` to build the next-free hint |
| `crate::doc` | `Doc::load` to adopt an existing file, `Doc::insert` for placement under the parent (and for opening a `## Criteria` section when the file has none), `Doc::save` to persist atomically, `new_file_text` for a new family file's scaffold |
| `crate::workspace` | `Workspace::find_id` (duplicate lookup, parent lookup, and destination-file resolution), `next_free`, `doc_for_family`, `rel`, and the `docs`/`dir` fields |

### Consumed By

| Module | What is used |
|--------|-------------|
| `main` (`src/main.rs`) | `capture::capture` in `run_capture(root, raw_id, rest)`, the default action when the first CLI argument is id-shaped; reads `Captured.created_file`, `Captured.file` and `Captured.id` to print `<file>  created` and `<file>  +<id>`. `main` reads `args_os` and peels `--root`/`--root=PATH` out with `peel_root` before the id-shaped test, so the flag becomes capture's start directory instead of words in the sentence (hi: CAPTURE-8), and a non-UTF-8 argument is reported with a message beginning `that sentence is not valid UTF-8` rather than panicking (hi: CAPTURE-1.c) |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Verification pass against source: corrected the rendering invariant (one line, whitespace collapsed, no wrapping), narrowed the orphan-prevention claim to match `check`'s per-file rule, fixed the duplicate-id example's line number, and added the no-frontmatter adoption error case. |
| 2026-09-16 | Claude | Reconciled with the bug-fix pass: destination now resolves to the parent's file before the declared family (hi: CAPTURE-4.a), which closes the capture/`check` orphan seam the old invariant 6 described; `Doc::insert` opens a `## Criteria` section when the file has none (hi: CAPTURE-7); `Doc::save` is atomic (hi: FILE-8); frontmatter style is preserved on rewrite (hi: FILE-7); fenced and stray criterion-shaped lines are invisible to the duplicate check (hi: FILE-9, CHECK-2.e); zero-padded ids refuse (hi: ID-1.c); workspace discovery requires real hi files (hi: CAPTURE-6); `main` peels `--root` and reads `args_os` (hi: CAPTURE-8, CAPTURE-1.c); line endings and BOM (hi: FILE-10, FILE-11). |
| 2026-09-16 | Claude | Verification pass over that reconciliation: corrected invariant 17, because `Doc::to_text` terminates every non-empty file with the detected line ending, so a destination that had no trailing newline gains one rather than having its shape restored (reproduced against the built binary); recorded that `holds_hi_files` also strips a BOM (invariant 9) and that a `# ` heading fixes the append point at the end of the criteria block (invariant 13). |
