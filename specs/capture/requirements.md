---
spec: capture.spec.md
---

## User Stories

- As someone mid-thought, I want to write a criterion down with one command and no setup, so that the idea is on disk before I lose it (hi: CAPTURE-1, CAPTURE-1.a)
- As someone mid-thought, I want capture to never stop and ask me a question, so that answering a prompt is not what costs me the thought (hi: CAPTURE-1.b)
- As someone naming a new part of the product, I want a brand-new family to start its own file by itself, so that I do not have to decide on a filename before I can say what I want (hi: CAPTURE-2, CAPTURE-2.a)
- As someone appending to a family from two branches, I want an id that is already taken to refuse and tell me the next free number, so that I never quietly clobber an existing criterion (hi: CAPTURE-3)
- As a reader of a hi file, I want a case to sit directly under the criterion it is a case of, so that the file reads as an outline rather than an append log (hi: CAPTURE-4)
- As someone whose family is declared in one file and written in another, I want a case to land in the file where its parent actually lives, so that it is not stranded somewhere nothing reads it as a case (hi: CAPTURE-4.a)
- As someone who typed something wrong, I want a refusal to leave the file exactly as it was, so that a rejected capture is never a half-applied one (hi: CAPTURE-5)
- As someone capturing into a file I started by hand, I want hi to make a criteria section rather than appending wherever my file happens to end, so that the criterion is somewhere that will be read back (hi: CAPTURE-7)
- As someone whose disk can fill up, I want a failed write to leave my file exactly as it was, so that a capture can never destroy criteria I already had (hi: FILE-8)
- As someone with my own YAML habits, line endings, and editor, I want hi to write my file back in my style, so that a one-line capture is a one-line diff (hi: FILE-7, FILE-10, FILE-11)
- As someone who typed a sentence, I want every word of it to reach the file unchanged, including a word that happens to look like one of hi's own options, so that capture is never quietly editing me (hi: CAPTURE-8, CAPTURE-9)
- As someone reading the file on GitHub rather than in an editor, I want each criterion to be its own list item with its cases nested under it, so that what I captured reads as a list and not as a paragraph of run-together sentences (hi: FILE-1.b, FILE-1.c)
- As someone working deep inside a repository, and sometimes inside a repository that sits inside another one, I want capture to anchor to *my* repository, so that I never append to somebody else's criteria (hi: CAPTURE-10)
- As someone who just captured something, I want to be told which file it landed in, so that I never have to go looking for it (hi: CAPTURE-11)
- As the author of `hi check`, I want capture to refuse a case whose parent I have not written yet and to name what is missing, so that the ordinary way of mistyping a sub-id does not author the structural error the checker then reports (hi: CAPTURE-2.b)

- As the owner of the repository, I want the feature list at the front of my product to be true after
  a capture, without me remembering to run `hi index` (hi: INDEX-4)
- As someone capturing a thought, I want a capture that stored my criterion never to be reported as a
  failure because that list could not be refreshed (hi: INDEX-4.a)

## Acceptance Criteria

### REQ-capture-001

The capture module SHALL append a well-formed, previously unused criterion id and its sentence to the file that owns the id's family, and SHALL report the id, the repository-relative file, and whether that file was newly started (hi: CAPTURE-2).

Acceptance Criteria

- `capture(workspace, "SEND-2", "It reaches them.")` against a workspace that already holds `SEND-1` in `hi/chat.md` writes the line `- **SEND-2**  It reaches them.` into that file.
- The line is a markdown list item with the id in bold, indented two spaces for every level below the first, so a top-level criterion starts at column 0 and `SEND-2.a` would start at column 2 (hi: FILE-1.b, FILE-1.c).
- The returned `Captured` carries `id` rendering as `SEND-2`, `file == "hi/chat.md"`, and `created_file == false`.
- The file is saved before the call returns; the caller never has to flush anything.
- Exactly one criterion is added, to exactly one file, per successful call.

### REQ-capture-002

The capture module SHALL start `hi/<family>.md` itself when the id's family is not held by any loaded file, without asking the person where to put it (hi: CAPTURE-2.a, CAPTURE-1.b).

Acceptance Criteria

- `capture(workspace, "BILLING-1", "I can see what I paid.")` creates `hi/billing.md` when no file declares or uses `BILLING`.
- The new file contains `families: [BILLING]` frontmatter, a `# Billing` title, an `## Intent` heading followed by the HTML-comment prompt `<!-- What is this for, and what should it feel like? Write it as a person. -->`, a `## Criteria` heading, and the criterion line `- **BILLING-1**  I can see what I paid.`.
- The file stem is the family lowercased with underscores replaced by hyphens; the title is the family lowercased with underscores replaced by spaces and the first letter capitalized.
- `created_file` is `true` so the caller can print the extra `created` line.
- No prompt, wizard, editor, or stdin read occurs at any point.

### REQ-capture-003

The capture module SHALL refuse an id that already exists anywhere in the workspace, naming where it exists and the next free top-level id in its family, and SHALL NOT write anything (hi: CAPTURE-3, CAPTURE-5).

Acceptance Criteria

- The error message contains `already exists`, the repository-relative file, and the 1-based line number of the existing criterion.
- The message carries a second line of the form `hint:  next free is SEND-2`, taken from `Workspace::next_free` for that family.
- The hint is always a single-level top-level id, even when the rejected id was a case or a step.
- `Workspace::find_id` walks active and retired criteria alike, so a retired id counts as existing and a retired number is never handed back out.
- Only parsed criteria count *here*. `Doc::all()` chains `criteria` and `retired` and nothing else, so an id written inside a fenced code block (prose to the parser, hi: FILE-9), on a criterion-shaped line outside every section, or in a file `load` skipped, is not what this refusal sees and does not raise `next_free`. It is still refused, by the separate reservation check REQ-capture-017 describes, with a different message.
- The destination file is byte-for-byte unchanged.

### REQ-capture-004

The capture module SHALL refuse a sub-id whose parent criterion does not exist, naming the missing parent, and SHALL NOT write anything (hi: CAPTURE-2.b, CAPTURE-5).

Acceptance Criteria

- `capture(workspace, "SEND-4.a", "An orphan.")` with no `SEND-4` present returns the error `SEND-4.a needs a parent SEND-4, which does not exist yet`, which names the id that is missing rather than only reporting that something is (hi: CAPTURE-2.b).
- The check runs after the already-exists check and before any file is resolved or created, so no family file is scaffolded for a refused orphan.
- Existence is judged workspace-wide by `Workspace::find_id`, across every loaded file and both `## Criteria` and `## Retired`.
- That workspace-wide scope no longer disagrees with `check`'s per-file orphan rule, because the accepted sub-id is then written into the very file `find_id` found its parent in (REQ-capture-010). `src/check.rs` builds `present` from that file's `doc.all()`, so the parent is present there by construction.

### REQ-capture-005

The capture module SHALL reject a malformed id and an empty sentence before considering the workspace at all.

Acceptance Criteria

- An id failing the grammar in `id::Id::parse` (bad family charset, missing hyphen, empty level, or broken number/letter alternation) returns `'<raw_id>' is not a valid id` followed by `<reason>`, carrying the underlying `IdError` text.
- `SEND-1.a.b` and `send-1` are both rejected.
- A zero-padded numeric level is rejected with `IdError::PaddedLevel`, so `SEND-007` refuses rather than becoming a second spelling of `SEND-7` (hi: ID-1.c).
- A sentence that is empty or whitespace-only returns `a criterion needs a sentence. Say what you actually want`.
- The sentence is trimmed before the emptiness test, so `"   "` is empty. A sentence of nothing but a newline or tabs is empty for the same reason.
- Id validation precedes sentence validation, so an input that fails both is reported as a bad id.

### REQ-capture-006

The capture module SHALL complete every validation before performing any filesystem mutation, so that a refusal leaves the workspace unchanged (hi: CAPTURE-5).

Acceptance Criteria

- Id parsing, sentence emptiness, duplicate detection, and the parent check all execute before `hi/` is created, before a family file is written, and before any in-memory `Doc` is touched.
- After a rejected capture, neither the rejected sentence nor the rejected id appears in any file.
- No partially written or truncated file is produced by a refusal.

### REQ-capture-007

The capture module SHALL place a new case or step directly beneath its parent criterion rather than at the end of the file (hi: CAPTURE-4).

Acceptance Criteria

- After capturing `SEND-1.a` into a file holding `SEND-1`, the offset of `SEND-1.a` in the file text is greater than the offset of `SEND-1 ` and no unrelated criterion separates them.
- The case is written two spaces further in than its parent, so it renders as a nested list item rather than a sibling (hi: FILE-1.b).
- Placement is delegated to `Doc::insert`; capture itself computes no line numbers.
- A top-level id with no parent joins its family's block rather than starting a new one, when that family already has a block.
- The one departure: when the parent exists only in `## Retired`, `Doc::insert`'s `insertion_point` does not find it (it scans `self.criteria` only), so the line joins the end of the family's active block while still carrying its own depth's indent, and renders as a case of whichever active criterion is last. Reproduced against the release binary; see Edge Cases in `testing.md`.

### REQ-capture-008

The capture module SHALL require no initialization step, creating the `hi/` directory on demand, and SHALL anchor to the repository it is run inside (hi: CAPTURE-1, CAPTURE-1.a, CAPTURE-10).

Acceptance Criteria

- When `workspace.dir` does not exist, it is created with `fs::create_dir_all` as part of starting the first family file.
- A first capture in a repository with no `hi/` directory succeeds in one command. `Workspace::find` only accepts a `hi/` directory that `holds_hi_files`, meaning one holding a `.md` file with a `hi:` frontmatter key (hi: CAPTURE-6), so an absent or still-empty `hi/` is reached through the `.git` branch, and `capture` then creates the directory and the file.
- That `.git` branch is a boundary, not just a fallback: `Workspace::find` returns at the first directory holding one, so a repository nested inside another captures into its own root and never adopts the outer project's criteria (hi: CAPTURE-10, covered by `cli::a_repository_is_a_boundary_for_discovery`).
- Capture works from any directory inside the repository, because `find` walks up from the start directory (or from `--root`, when one was given).
- When neither a qualifying `hi/` nor a `.git` is found anywhere above the start directory, `Workspace::find` fails before `capture` is called, with `this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so run it inside one`, and the process exits 1.
- No configuration file, lockfile, cache, or state file is created, read, or required.

### REQ-capture-009

The capture module SHALL report the outcome of a successful capture as data, including the file it landed in, leaving all rendering to the caller (hi: CAPTURE-11).

Acceptance Criteria

- `Captured` exposes `id`, `file`, and `created_file` as public fields and derives `Debug`.
- `file` is produced through `Workspace::rel`, so it is relative to the workspace root and identical no matter which directory the command ran from, and it names the file the line actually landed in rather than the one the family is declared in.
- `Workspace::rel` joins path components with a forward slash on every platform, because the same string is printed, exported as JSON and used in markdown links (hi: FILE-12).
- `capture` prints nothing itself; `src/main.rs` prints `<file>  created` and `<file>  +<id>` from the returned value, so every capture ends by naming its destination (hi: CAPTURE-11).

### REQ-capture-010

The capture module SHALL send a sub-id to the file that actually holds its parent, consulting the declared family only when the id has no parent or the parent was not found (hi: CAPTURE-4.a).

Acceptance Criteria

- `capture` computes `parent_file` from `id.parent().and_then(|p| workspace.find_id(&p).map(|(index, _)| index))` and passes it to `parent_file.or_else(|| workspace.doc_for_family(&id.family))`; the family lookup runs only as the fallback.
- Given `hi/decl.md` and `hi/real.md` both declaring `families: [SEND]` but only `hi/real.md` holding `SEND-1`, capturing `SEND-1.a` writes into `hi/real.md` and leaves `hi/decl.md` untouched.
- `Captured.file` names the file the line actually landed in, so the printed path is the one to open.
- A top-level id is unaffected: `Id::parent()` is `None` for a single-level id, so resolution goes straight to `doc_for_family`.

### REQ-capture-011

The capture module SHALL cause a `## Criteria` section to exist in the destination file rather than appending a criterion wherever the file happens to end (hi: CAPTURE-7).

Acceptance Criteria

- When the destination `Doc` has no `criteria_heading`, `Doc::insert` splices a blank line, `## Criteria`, and a blank line in after `last_content_line`, records the new heading, and shifts every tracked position past it.
- The criterion is then placed under that heading, not inside the file's intent prose or below an unrelated heading.
- This applies to the adoption path in `create_file`, where an existing `hi/<stem>.md` may be any markdown file the person wrote.
- A file that already has a `## Criteria` heading gains no second one.

### REQ-capture-012

The capture module SHALL NOT be able to damage the destination file by failing partway through its write (hi: FILE-8).

Acceptance Criteria

- `Doc::save` calls `doc::write_atomically`, which writes a sibling `.<name>.hi-tmp`, `write_all`s, `flush`es, `sync_all`s, and only then `fs::rename`s over the target. The target is never opened for truncation.
- Any failure before the rename removes the temp file and returns the `std::io::Error`, wrapped by `save` as `writing <path>`; the original file is untouched.
- A failed rename likewise removes the temp file.
- `create_file`'s scaffold is a plain `fs::write` to a path that has just been shown not to exist, so it has no prior contents to lose.

### REQ-capture-013

The capture module SHALL return the destination file in the shape the person wrote it, changing only the criterion it added (hi: FILE-4.a, FILE-7, FILE-10, FILE-11).

Acceptance Criteria

- When `Doc::insert` has to declare the family, `rewrite_families` replaces exactly `Front::families_span` and branches on `Front::families_block`: an inline `families: [A, B]` entry is written back inline, a YAML block list is written back as `families:` followed by `  - A` items. Neither style is converted to the other.
- When the file has no `families` entry at all, one is added just inside the closing `---`.
- `Doc::to_text` rejoins the lines with `Doc::newline`, the ending detected at parse time, so a CRLF file stays CRLF.
- A leading UTF-8 BOM is stripped by `Doc::parse` and is not written back, so the saved file has no BOM. `Workspace::find`'s `holds_hi_files` strips one too, so a BOM does not hide the workspace from discovery either.
- The one shape that is not round-tripped: `Doc::to_text` ends every non-empty file with that same line ending (`if self.trailing_newline || !out.is_empty()`), so a destination file whose last line carried no newline gains one. `Doc::trailing_newline` only suppresses the ending for a file with no lines at all. Reproduced against the built binary; recorded here rather than claimed as preservation.

### REQ-capture-014

The capture module SHALL leave the working habit where an agent will read it, without being asked and without an init step (hi: HABIT-1, HABIT-2, HABIT-3).

Acceptance Criteria

- `start_agent_files` writes `hi/AGENTS.md` from `out::agent_instructions` when no entry of that name exists, and `hi/CLAUDE.md` beside it pointing at the same text, and returns what it started so `main` can name each file.
- `link_to_agents` makes `hi/CLAUDE.md` a symlink to `AGENTS.md` where the platform allows one, and falls back to a one-line `See @AGENTS.md` file where it does not. A symlink keeps one truth; a committed symlink on a checkout with `core.symlinks` false would otherwise arrive as a text file holding the literal target, which an agent reads as the whole instruction (DECISIONS.md §27).
- Presence is tested with `symlink_metadata`, so an existing symlink, including a broken one, is left alone rather than treated as absent.
- `hi/CLAUDE.md` is written only when `hi/AGENTS.md` is a real file, so the pointer never dangles.
- Both are written only after the criterion is on disk, and every failure is swallowed: a capture that stored its criterion is never reported as a failure because these could not be written (hi: CAPTURE-1.a; the `INDEX-3` pattern).
- Neither file is rewritten on a later capture. hi writes them once and they belong to the repository afterwards.

### REQ-capture-015

The capture module SHALL refuse a family whose file would be one hi keeps for itself, identically on every platform (hi: CAPTURE-5).

Acceptance Criteria

- `RESERVED_FAMILIES` is `AGENTS` and `CLAUDE`, compared with `eq_ignore_ascii_case`, and is exactly the set of files `start_agent_files` writes.
- The refusal happens after id and sentence validation and before any filesystem write, and names the file that family would want.
- The refusal does not depend on the filesystem folding case. `start_file` lowercases a family name, so `AGENTS` wants `hi/agents.md`: the same path as `hi/AGENTS.md` on macOS or Windows, a confusing neighbour on Linux. Refusing on both is what keeps the behaviour one behaviour.


### REQ-capture-016

The capture module SHALL leave the generated feature list in `INTENT.md` true, and SHALL never report a capture that stored its criterion as a failure for doing so (hi: INDEX-4, INDEX-4.a).

Acceptance Criteria

- `out::refresh_index` is called after `Doc::save`, so the criterion is on disk before anything else is attempted. Every refusal path still returns before any filesystem write (REQ-capture-006, hi: CAPTURE-5).
- The refresh counts what is on disk. `Doc::insert` splices the rendered line into `Doc::lines` without adding the criterion to `doc.criteria`, so capture reloads the file it just saved through `Doc::load` first. Without that reload the generated list comes out one criterion short.
- Any reason the refresh could not happen, including `write_index`'s refusal to guess past a broken marker pair (hi: INDEX-2.b), is carried out on `Captured.index_error` for `main` to print. The call still returns `Ok` and the process still exits 0.
- This is the same rule `start_product_intent` and `start_agent_files` already follow, and for the same reason: the thought is what mattered and it is already stored (hi: INDEX-3, HABIT-1, CAPTURE-1.a).
- The refresh runs inside the lock `main::run_capture` already holds and takes none of its own, because `lock::acquire` is not reentrant (hi: FILE-19).
- The accepted cost is that a bulk capture rewrites `INTENT.md` once per criterion. fledge's adoption was 197 captures. That is 197 atomic replaces of a few hundred bytes, serialized by a lock those captures already contend for.
- The refresh rewrites the generated block and installs none. `refresh_index` passes `out::Absent::LeaveAlone`, so an `INTENT.md` with no marker pair is left exactly as it is and nothing is reported; installing a `## Features` section is `hi index`'s act, because there it was asked for (hi: INDEX-4.c, DECISIONS.md §32).
- `start_product_intent` therefore writes `out::starter_intent_file`, which carries the `## Features` heading and the generated block, rather than the prose prompt alone. A first capture still leaves a complete `INTENT.md`; it is written once rather than appended to by the refresh that follows (hi: INDEX-1.a, INDEX-3).

### REQ-capture-017

The capture module SHALL refuse an id written anywhere hi cannot read it as a criterion, wherever that is, and SHALL NOT write anything (hi: CAPTURE-14, FILE-20, CAPTURE-5).

Acceptance Criteria

- After the `find_id` refusal and before any filesystem write, `capture` calls `Workspace::find_stray`, the one reservation lookup, which covers criterion-shaped lines in a criteria file outside every section, inside a fence, and in a file `load` skipped because its name is not lowercase.
- The message is `<id> is already written at <file>:<line>, where hi cannot read it.`, followed by a `hint:` line. The hint for `StrayPlace::OutsideSection` says to move the line under `## Criteria` or `## Retired`; for `StrayPlace::UnreadFile` it says to move it into a lowercase file, because renaming `hi/AGENTS.md` would turn hi's own instruction file into a criteria file.
- The refusal and `check`'s `stray-criterion` report come from the same call, so an id `hi check` names as used is always an id `hi` refuses to reissue. They were two scans over two different sets of files, and a retired `SEND-1` in `hi/Archive.md` was reported by one and handed out again by the other (DECISIONS.md §32).
- An id that no line anywhere speaks for is still free, so this reserves rather than blocks.
- hi's own files are read for reservation and for nothing else. Neither `hi/AGENTS.md` nor `hi/CLAUDE.md` becomes a doc, a family, a counted criterion or a line in the generated feature list, and the prose hi writes into them contains no criterion-shaped line (DECISIONS.md §27).

## Constraints

- hi holds intent and identity only. Capture stores no lifecycle, status, checkbox, timestamp, author, or evidence link alongside the criterion. The line is an id and a sentence and nothing else.
- Capture never rewords, judges, capitalizes or punctuates the person's English, and never drops or adds a word (hi: CAPTURE-9). It trims the sentence; `doc::render_criterion` then writes it as exactly one line however long it runs, collapsing interior whitespace runs to single spaces so a pasted multi-line thought becomes one sentence (hi: FILE-6; DECISIONS.md §10.1). Nothing wraps, so there is no continuation line to author.
- That line is a markdown list item, `- **<id>**  <sentence>`, indented two spaces per level below the first. The bullet and the bold id are structure, not decoration: without them a block of criteria is joined into one run-together paragraph by every markdown renderer, which is what DECISIONS.md §12 records as having falsified FILE-1 (hi: FILE-1.b, FILE-1.c). The parser accepts the bare form, and a bullet with no emphasis, so a hand-edited file is still read (hi: FILE-14); capture itself always writes the list form.
- Nothing typed after the id is ever treated as an option. `peel_root` consumes a `--root` only while it still leads the argument vector, so a sentence mentioning `--root` keeps it (hi: CAPTURE-8, CAPTURE-9).
- Ids are permanent and append-first. Capture never renumbers, reorders, edits, or deletes an existing criterion.
- The only content condition that makes capture fail is an id that is already spoken for. Incomplete intent (a criterion with no spec, no ticket, and no test) is never an error here.
- Interactivity is forbidden. No prompt, no wizard, no required field, no editor, no stdin (hi: CAPTURE-1.b).
- Capture depends only on `std`, `anyhow`, and the sibling `id`, `doc`, and `workspace` modules. It shells out to nothing and reaches the network never.
- Error text is part of the contract: `refuses_an_id_that_already_exists` asserts on `already exists` and `next free is SEND-2`, `refuses_a_case_with_no_parent` asserts on `needs a parent SEND-4`, and `tests/cli.rs` asserts the same two strings plus `leading zero`, `not valid UTF-8` and `not a repository` through the real binary on stderr with exit 1.

## Out of Scope

- The id grammar, alternation rule, parent derivation, the padded-level rule, `IdError` text, and `looks_like_id` (its case-insensitive family test and its leading-digit rule for the first level) all live in `src/id.rs` (`specs/id/`).
- Line rendering and the list-item form, whitespace collapsing, bullet and emphasis stripping on the way back in, insertion-point arithmetic, section creation, fence opacity, BOM and line-ending handling, frontmatter parsing and rewriting, and atomic serialization belong to `src/doc.rs` (`specs/doc/`).
- Locating the repository, the `holds_hi_files` test, the `.git` boundary and the message when neither is found, loading every `hi/*.md`, family lookup, next-free computation, and relative-path display (including its forward slashes) sit in `src/workspace.rs` (`specs/workspace/`).
- Detecting duplicates, orphans, retired-id collisions, broken alternation, and stray criteria across the whole repository, and the exit code for them, are `src/check.rs`'s job (`specs/check/`).
- Listing, ticket generation, JSON export, and the `INTENT.md` index come from `src/out.rs` (`specs/out/`).
- CLI argument routing, the id-shaped-first-argument dispatch, `peel_root`'s leading-only handling of `--root`, the `args_os` non-UTF-8 guard, and process exit codes are handled in `src/main.rs` (hi: CAPTURE-8, CAPTURE-1.c).
- Retiring a criterion. Capture only ever adds; moving a criterion to `## Retired` is a hand edit.
