---
spec: check.spec.md
---

## Key Decisions

- **`check` is the only verb that can fail on content, and it fails on exactly seven things.** That
  list is the contract, not an implementation detail. 1.0 freezes those seven *and* the policy that
  they are structural only; an eighth is a 2.0 (HI-1.md, DECISIONS.md §39). The design record is
  explicit that hi never fails a build because a criterion is unproven (DECISIONS.md §5, §6); the
  `unfinished_intent_is_never_a_problem` test exists to make that absence visible and to break
  loudly if someone adds a content check.
- **`run` returns `Report`, not `Result<Report>`.** A malformed id is data, not an error, so the
  walk always completes and reports everything it found. This is why `hi check` can list several
  problems in one pass instead of stopping at the first, which is now written down as intent
  (hi: CHECK-5).
- **`UndeclaredFamily` is the fifth kind, and it is recorded in both DECISIONS.md and `hi/`.**
  DECISIONS.md §6 lists the structural errors as exactly the seven variants of `check::Kind`, "a
  family a file never declared" among them, and `hi/check.md` carries CHECK-2.f for it. (An earlier
  revision of this file claimed it was an undocumented sixth wheel; that was true of an older
  DECISIONS.md and is not true now. Duplicate-family is the seventh, DECISIONS.md §39.) It exists because frontmatter is where a family declares its
  home file: `Doc::insert` adds a family to `families:` whenever capture writes a criterion for one
  the frontmatter lacks, so a
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
  and leading token. That is why its problem uses `line + 1` instead of `line_no()`, and why its
  `id` field is a raw token that was never handed to `Id::parse`. A stray takes part in no other
  check. One consequence is visible in output: `doc` runs `strip_bullet` before recording the token
  but not `strip_emphasis`, which `is_criterion_line` used a moment earlier to decide the line was
  id-shaped at all, so a stray written as `- **SEND-9**  ...` is reported as `**SEND-9** sits
  outside any section`, asterisks and all, while a criterion on the same line would have carried
  `raw_id == "SEND-9"`. Verified against the binary. It is cosmetic, but it is the one place where
  `check`'s idea of an id differs from `doc`'s, and a reader comparing the two will notice.
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
  indented or not) becomes intent prose. An unfenced `SEND-9  ...` under `## Intent`
  is therefore not a `StrayCriterion`, even though `section` is `None` there. That is deliberate:
  the intent is prose, and turning a sentence that happens to start with an id-shaped token into a
  finding would make the intent section unwritable. Note the asymmetry with `## Notes` and every
  other unrecognized `## ` heading, which *do* leave a stray, and with a `# ` heading, which clears
  `in_intent` as well as `section`. Nothing tests this; it was verified by hand against the binary.
- **`looks_like_id` decides the whole reachable `IdError` set, and it changed.** A token is
  id-shaped when it splits on a `-` into a family that starts with an ASCII letter and continues in
  letters, digits and underscores, and a remainder whose first character is a digit. Two
  consequences, both verified against the binary:
  - The family test is deliberately case-insensitive, so `send-2` and `Send-3` become criterion
    candidates and `Id::parse` refuses them with `IdError::BadFamily`. That variant is reachable
    from `check`, which it was not before. The point is that a lowercase id is plainly *meant* as a
    criterion, and reading it as prose would lose it silently (hi: CHECK-2.d).
  - The first level must start with a digit, which is what separates an id from an ordinary
    hyphenated word. `spec-sync`, `well-formed` and `co-authored` are prose, and so are `SEND-a` and
    `SE-ND-1` (whose remainder `ND-1` does not start with a digit). `SE-ND-1` used to be this
    module's worked example of `BadLevel`; the live example is now `SEND-1.A` or `SEND-1.a1`.
  The five variants a criterion line can carry are therefore `BadFamily`, `EmptyLevel`, `BadLevel`,
  `PaddedLevel` and `Alternation`. Only `MissingHyphen` (`SEND1`) and `NoLevels` (`SEND-`) are
  filtered out before `check` sees them.
- **Indentation means nothing to either check, because criteria are nested list items now.**
  `render_criterion` writes `- **ID**  sentence` indented two spaces per depth level, so
  `doc::parse_body` matches on the trimmed line. A criterion is recognized at any indent, and so is
  a stray: `  - **SEND-11**  ...` below a `# ` heading is reported exactly as a flush-left line
  would be. What ends a criterion's continuation run is no longer "this line is not indented" but
  "this line is itself id-shaped" (`read_criterion` breaks on `is_criterion_line`). Any spec
  sentence here that says "at column 0" is describing the old parser.

## Files to Read First

- `src/check.rs` is the whole module, one `run` function plus three small types; read the
  `#[cfg(test)]` module first, it is the clearest statement of what is and is not a problem.
- `src/doc.rs` holds `Section`, `Criterion` (`id`, `raw_id`, `id_error`, `section`, `line_no()`),
  `Doc::all()` (active then retired), `Doc::stray`, and above all `parse_body`, which decides in one
  loop what `check` will ever see. The order of its branches is the contract: it skips fenced blocks
  entirely, closes an open section on a `# ` or `## ` heading, swallows the whole of a `## Intent`
  section into the intent prose, and only then treats a line whose leading token is id-shaped as a
  criterion, or (with no section open and no intent being collected) as a stray. Anything that loop
  does not select never reaches `check`. Read `is_criterion_line`, `strip_bullet` and
  `strip_emphasis` next to it: together they are what "id-shaped line" means, and they are why a
  hand-typed criterion is read whether or not it carries a bullet or bold (hi: FILE-14).
- `src/id.rs` defines `Id::parse`, `Id::parent`, the `IdError` variants and their `Display` text
  (which becomes the tail of every `unparseable-id` message, `PaddedLevel` included), and
  `looks_like_id`, which is what makes a line a criterion candidate at all and therefore decides
  which `IdError` variants this module can ever report.
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
workspaces from in-memory strings via `Doc::parse` and need no fixtures. Nine `tests/cli.rs` cases
drive the verb end to end, including the only assertions anywhere on a problem's file and line
(`check_fails_on_a_structural_problem`) and the two that pin the new `looks_like_id` boundary
(`a_wrongly_cased_id_is_reported_rather_than_read_as_prose` and
`an_ordinary_hyphenated_word_is_not_mistaken_for_an_id`).

Known coverage gaps, recorded in `tasks.md`: nothing asserts the file-then-line sort order or reads
`problem.file` / `problem.line` from a `Report` directly (REQ-check-007), nothing asserts the JSON
shape or that `Kind::code()` agrees with the serde kebab-case rename (REQ-check-010), nothing puts a
zero-padded id in a file to see `check` report it (REQ-check-005), nothing pins the `## Intent`
exception to the stray rule (REQ-check-011), and no test anywhere puts a stray at an indent or
checks what a bulleted, bolded stray reports as its id (REQ-check-011).

## Notes

- `Doc::all()` yields active criteria first, then retired ones. Within a document, "first sighting"
  for the duplicate check therefore favours the active line, which is why
  `catches_reuse_of_a_retired_id` reports the retired line as the duplicate.
- The unparseable-id message is `'{raw_id}' is not a valid id: {reason}`, so the separator between
  the two halves is a colon and a space. In full, from the binary:
  `'SEND-1.a.b' is not a valid id: level 3 must be a number, because levels alternate number,
  letter, number, letter`. Changing the separator changes user-visible output.
- `Criterion::line` is 0-based and `line_no()` is 1-based. Every problem must use `line_no()`; a
  bare `.line` would silently produce off-by-one locations.
- There is no `--fix`, and there should not be. `check` reports; repairing an id is a human edit,
  because renumbering would violate the permanence rule in DECISIONS.md §4.
- A mistyped family creates a stray one-criterion file rather than an error (DECISIONS.md §6). That
  is visible in the report as a family holding a single criterion; do not turn it into a problem
  kind. Note the word collision: that is unrelated to `Kind::StrayCriterion`, which is about a line
  outside any section.
- The stray message is two sentences: `{token} sits outside any section. Move it under ## Criteria
  or ## Retired, because nothing reads it where it is`. Note the capital `M`, the full stop before
  it, and the trailing `because` clause that says why it matters rather than only what to do. It is
  built with a `\`-continued format string; the continuation swallows the leading whitespace of the
  next source line, so the space before `because` is the one in `## Retired, ` and there is exactly
  one.
- `check` sees a workspace that `workspace`/`doc` already normalized: a UTF-8 BOM has been stripped
  (both in `Doc::parse` and in workspace discovery), and a CRLF file was split on newlines with the
  line ending remembered in `Doc`'s private `newline` field for writing. Neither affects line
  numbers, so a reported line still matches what the editor shows in a CRLF file.
- `Workspace::find` is stricter at both ends than it was, and both changes move the boundary of what
  `check` is ever handed. It requires a `hi/` directory to actually hold a file with `hi:`
  frontmatter before adopting it, because `hi` is the ISO code for Hindi; and it now stops at a
  `.git` boundary, loading that repository's (possibly empty) `hi/` rather than walking past it into
  an outer repository's criteria. So inside a nested repository with no `hi/`, `hi check` prints
  `0 criteria · 0 families · 0 files` and exits 0, which
  `cli::a_repository_is_a_boundary_for_discovery` pins. The not-a-repository error is reached only
  when neither a qualifying `hi/` nor a `.git` is found all the way up, and its text is now
  `this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so
  run it inside one`. That failure happens before `run` is ever called;
  `cli::outside_a_repository_it_says_so_rather_than_guessing` asserts on `not a repository` and
  shows the exit-1 path belongs to `workspace`, not to a `Report`.
- `check` writes nothing, so `Doc::save`'s atomic write (`write_atomically`: sibling temp file,
  flush, sync, rename) is not this module's concern, but it is why a half-written file can never
  appear in a check run.
