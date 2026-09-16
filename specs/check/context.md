---
spec: check.spec.md
---

## Key Decisions

- **`check` is the only verb that can fail on content, and it fails on exactly six things.** That
  list is the contract, not an implementation detail. The design record is explicit that hi never
  fails a build because a criterion is unproven (DECISIONS.md §5, §6); the `unfinished_intent_is_never_a_problem`
  test exists to make that absence visible and to break loudly if someone adds a content check.
- **`run` returns `Report`, not `Result<Report>`.** A malformed id is data, not an error, so the
  walk always completes and reports everything it found. This is why `hi check` can list several
  problems in one pass instead of stopping at the first.
- **`UndeclaredFamily` is the fifth kind and appears nowhere in DECISIONS.md.** §6 lists the
  structural errors as duplicate id, missing parent, retired collision, and an unparseable line
  (with broken alternation spelled out as a case of the last); `UndeclaredFamily` is not among
  them. It exists because frontmatter is where a family declares its home file: `Doc::insert` adds
  a family to `families:` whenever capture writes a criterion for one the frontmatter lacks, so a
  file that uses a family it does not declare has drifted from the shape capture maintains. Note
  what it is *not*: capture is not blocked by it. `Workspace::doc_for_family` falls back to
  searching by use, so `hi OFFLINE-2 "..."` still lands in the right file and repairs the
  frontmatter on the way. It is a structural error for a mechanical reason, not a style rule.
  Since the fix pass, capture resolves a *case* by the file holding its parent and only falls
  through to `doc_for_family` for an id with no parent (hi: CAPTURE-4.a), which narrows the blast
  radius of an undeclared family without making the finding pointless.
- **The frontmatter `families` list may be inline or a YAML block, and `check` must not care.**
  `Front::families` is the parsed list either way (`Front::families_block` records which style the
  file used, purely so rewriting preserves it). The comparison in the undeclared-family check is
  against `doc.front.families`, so a file written as `families:` / `  - SEND` is not falsely
  reported (hi: FILE-7); `cli::block_style_frontmatter_is_understood_and_preserved` pins that.
- **Retired ids are collected in a separate first pass over every document.** A single pass would
  make the result depend on `workspace.docs` order: an id retired in a file loaded later would be
  missed. Do not fold that loop into the main walk.
- **The retired-collision and undeclared-family checks are gated on `section == Section::Criteria`;
  the duplicate, orphan and unparseable checks are not.** A retired line is still a real id that can
  collide, can be malformed, and can serve as a case's parent. `a_retired_criterion_may_keep_its_case_parent`
  pins the parent half of that.
- **An id that appears in both `## Criteria` and `## Retired` produces two problems, not one.** It
  is a duplicate and a retired collision, and `catches_reuse_of_a_retired_id` asserts both kinds are
  present rather than asserting a count.
- **`continue` after `UnparseableId` is load-bearing.** Without an `Id`, there is no key for the
  duplicate map, no parent, and no family, so the criterion is abandoned after one problem. That is
  also why the problem's `id` field holds the raw token instead of a parsed id.
- **Duplicates are workspace-wide, orphans are file-local.** `seen` spans every document; `present`
  is rebuilt per document. This is deliberate. One family lives in one file, so a case whose parent
  is in a different file is a mistake even if the parent exists somewhere.
- **`Kind::code()` and the serde `rename_all = "kebab-case"` must be kept in lockstep.** They are
  two hand-maintained spellings of the same vocabulary; text output uses the first and `--json` the
  second, and nothing in the compiler ties them together.
- **Sorting happens once, at the end.** Problems are pushed in discovery order and then sorted by
  file then line, so output is diffable and the push sites do not have to care about ordering.
- **`StrayCriterion` is the sixth kind, and it is the only one that does not come from a
  `Criterion`.** It is read straight out of `doc.stray`, a `Vec<(usize, String)>` of 0-based line
  and first token. That is why its problem uses `line + 1` instead of `line_no()`, and why its `id`
  field is a raw token that was never handed to `Id::parse`. A stray takes part in no other check.
- **A stray exists because a `# ` heading closes an open `## Criteria`.** `doc::parse_body` treats
  a level-one heading like a `## ` heading: it records the append point and clears the section.
  Without the stray list, every criterion below such a heading would parse as nothing and disappear
  from `ls`, `export`, `view` and `check` alike (visible to a reader, invisible to the tool). The
  fix is to report it, not to reopen the section.
- **Fence opacity lives in `doc`, and `check` must never re-implement it.** `doc::parse_body`
  tracks an open ``` or `~~~` run and skips every line inside it, so an id-shaped line in a fenced
  example is neither a criterion nor a stray. `check` sees only what `doc` parsed, which is what
  lets a hi file paste a fenced example of the format into its own `## Intent` without generating
  findings (hi: FILE-9). No hi file in this repository does that yet, so the guarantee rests
  entirely on `doc::tests`. Any future "scan the raw lines for missed ids" shortcut in this module
  would undo it.
- **`## Intent` is a second, quieter reason a line never reaches `check`.** `parse_body` checks
  `in_intent` *before* the stray branch, so every line of an intent section (fenced or not,
  indented or not) becomes intent prose. An unfenced `SEND-9  ...` at column 0 under `## Intent`
  is therefore not a `StrayCriterion`, even though `section` is `None` there. That is deliberate:
  the intent is prose, and turning a sentence that happens to start with an id-shaped token into a
  finding would make the intent section unwritable. Note the asymmetry with `## Notes` and every
  other unrecognized `## ` heading, which *do* leave a stray, and with a `# ` heading, which clears
  `in_intent` as well as `section`. Nothing tests this; it was verified by hand against the binary.
- **`IdError::PaddedLevel` is now reachable from `check`.** `looks_like_id("SEND-007")` is true
  (the family is valid and there is something after the hyphen), so the line becomes a criterion
  candidate and `Id::parse` rejects it. It is the fourth `IdError` variant a criterion line can
  carry, alongside `EmptyLevel`, `BadLevel` and `Alternation`.

## Files to Read First

- `src/check.rs` is the whole module, one `run` function plus three small types; read the
  `#[cfg(test)]` module first, it is the clearest statement of what is and is not a problem.
- `src/doc.rs` holds `Section`, `Criterion` (`id`, `raw_id`, `id_error`, `section`, `line_no()`),
  `Doc::all()` (active then retired), `Doc::stray`, and above all `parse_body`, which decides in one
  loop what `check` will ever see. The order of its branches is the contract: it skips fenced blocks
  entirely, closes an open section on a `# ` or `## ` heading, swallows the whole of a `## Intent`
  section into the intent prose, and only then treats a column-0 line with an id-shaped first token
  as a criterion, or (with no section open and no intent being collected) as a stray. Anything
  that loop does not select never reaches `check`.
- `src/id.rs` defines `Id::parse`, `Id::parent`, the `IdError` variants and their `Display` text
  (which becomes the tail of every `unparseable-id` message, `PaddedLevel` included), and
  `looks_like_id`, which is what makes a line a criterion candidate at all.
- `src/workspace.rs` contributes `docs`, `rel`, `criteria_count`, and `families`, which supply the
  report's counts and path strings.
- `src/main.rs` has `run`'s `Command::Check` arm and `print_report`, the only consumer; it maps
  `Report::ok()` to the exit code and prints `  <line>:<code>  <message>` under a file header.
- `hi/check.md` and `DECISIONS.md` §5, §6, §9 carry the human intent this module serves, and the
  list of things that were deliberately not built.

## Current Status

Implemented and covered by ten inline unit tests: a clean workspace, each of the six problem kinds,
prose outside a section staying clean, unfinished intent staying clean, and a retired case keeping
its live parent. The module has no filesystem or network surface, so the tests build their
workspaces from in-memory strings via `Doc::parse` and need no fixtures. Six `tests/cli.rs` cases
exercise the verb end to end, including the only assertions anywhere on a problem's file and line
(`check_fails_on_a_structural_problem`).

Known coverage gaps, recorded in `tasks.md`: nothing asserts the file-then-line sort order or reads
`problem.file` / `problem.line` from a `Report` directly (REQ-check-007), nothing asserts the JSON
shape or that `Kind::code()` agrees with the serde kebab-case rename (REQ-check-010), nothing puts a
zero-padded id in a file to see `check` report it (REQ-check-005), and nothing pins the `## Intent`
exception to the stray rule (REQ-check-011).

## Notes

- `Doc::all()` yields active criteria first, then retired ones. Within a document, "first sighting"
  for the duplicate check therefore favours the active line, which is why
  `catches_reuse_of_a_retired_id` reports the retired line as the duplicate.
- The unparseable-id message writes the quoted token, then a separator, then the `IdError` reason:
  `'SEND-1.a.b' is not a valid id` and `level 3 must be a number ...` joined by that separator.
  Changing the separator changes user-visible output.
- `Criterion::line` is 0-based and `line_no()` is 1-based. Every problem must use `line_no()`; a
  bare `.line` would silently produce off-by-one locations.
- There is no `--fix`, and there should not be. `check` reports; repairing an id is a human edit,
  because renumbering would violate the permanence rule in DECISIONS.md §4.
- A mistyped family creates a stray one-criterion file rather than an error (DECISIONS.md §6). That
  is visible in the report as a family holding a single criterion; do not turn it into a problem
  kind. Note the word collision: that is unrelated to `Kind::StrayCriterion`, which is about a line
  outside any section.
- The stray message is one sentence with a separator and a trailing `because` clause, joining
  `SEND-9 sits outside any section` to `move it under ## Criteria or ## Retired, because nothing
  reads it where it is`. It is built with a `\`-continued format string; the continuation swallows
  the leading whitespace of the next source line, so the rendered message has a single space there.
- `check` sees a workspace that `workspace`/`doc` already normalized: a UTF-8 BOM has been stripped
  (both in `Doc::parse` and in workspace discovery), and a CRLF file was split on newlines with the
  line ending remembered in `Doc`'s private `newline` field for writing. Neither affects line
  numbers, so a reported line still matches what the editor shows in a CRLF file.
- `Workspace::find` now requires a `hi/` directory to actually hold a file with `hi:` frontmatter
  before adopting it, because `hi` is the ISO code for Hindi. That failure happens before `run` is
  ever called; `cli::outside_a_repository_it_says_so_rather_than_guessing` asserts the exit-1 path
  belongs to `workspace`, not to a `Report`.
- `check` writes nothing, so `Doc::save`'s atomic write (`write_atomically`: sibling temp file,
  flush, sync, rename) is not this module's concern, but it is why a half-written file can never
  appear in a check run.
