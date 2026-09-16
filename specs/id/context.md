---
spec: id.spec.md
---

## Key Decisions

- **Ids are the one strict thing in hi.** DECISIONS.md §4 makes them hand-written, permanent, and
  append-first. This module owns the grammar; it owns nothing about permanence, because permanence
  is a property of a file a human wrote, not of a value at runtime.
- **Letters are cases, numbers are steps, and they alternate strictly by depth.** `SEND-1.a.b` is
  rejected on purpose: a case of a case has to be expressed as a step containing cases. The error
  carries the 1-based depth so the message can name the level the human got wrong.
- **The split is at the first hyphen, not the last.** `SE-ND-1` therefore fails as `BadLevel` on the
  level `ND-1`, not as `BadFamily` on `SE-ND`. This is asserted by `rejects_bad_families` and is the
  single most surprising line in the file.
- **A zero-padded number is refused, not normalized.** `SEND-007` used to parse to `SEND-7`, which
  meant the id a person wrote was not the id hi meant: an exported parent or a captured case would
  point at a line that does not exist. `Id::parse` now returns `IdError::PaddedLevel` before the
  `u32` parse runs, so the padded text is still available to name in the error, and the sentence
  says which spelling to write instead. The guard is `part.len() > 1 && part.starts_with('0')`, and
  the length test is what keeps a bare `0` legal. Do not "simplify" this into a `u32` round trip
  comparison; the point is to refuse, not to correct (ID-1.c).
- **`looks_like_id` is deliberately weaker than `Id::parse`.** It exists so a mistyped id is still
  recognized as an *intended* criterion. Without that gap, `doc` would read `SEND-a.b` as prose and
  `hi check` could never report it (CHECK-2.d). Anything that tightens `looks_like_id` toward
  `parse` silently deletes an error class.
- **`Id`'s fields are public and `parse` is not the only constructor.** `capture` builds
  `Id { family, levels: vec![Level::Number(next_free)] }` directly for the "next free is SEND-3"
  hint. Alternation is therefore a parse invariant, not a type invariant. Do not write code that
  assumes an `Id` in hand came from `parse`.
- **`Level::Letter` holds a `String`, not a `char`.** Multi-letter levels (`SEND-1.aa`) are
  representable and parse cleanly, which is what keeps a family from running out of cases at `z`.
- **`Id` derives `PartialEq`/`Eq`/`Hash` but not `Ord`; `Level` derives `Ord`.** `check` and
  `workspace` compare and hash ids; nothing sorts them. Adding `Ord` to `Id` would need a decision
  about whether `SEND-2` sorts before `SEND-10`.
- **Descendant is structural, never textual.** `SEND-11` must not read as a descendant of `SEND-1`,
  which is why the comparison is over `Vec<Level>` rather than a string prefix. The test
  `knows_its_descendants` pins this.
- **The module is the bottom of the graph.** It imports `std::fmt` and nothing else. Keep it that
  way: everything in hi depends on it, so a dependency added here is a dependency added everywhere.

## Files to Read First

- `src/id.rs` is the whole module, ~330 lines including its
  `#[cfg(test)]` block. The module doc comment states the grammar; the tests state the intent.
- `DECISIONS.md` §4 ("IDs") and §8 (assumption 1: family
  charset, no zero padding). This is the authoritative record for why the grammar is what it is.
- `hi/format.md` holds the ID family criteria (ID-1 through
  ID-4) this module serves, in the human's own words.
- `src/doc.rs` is the heaviest consumer: `looks_like_id` for
  the stray check outside every section (~line 288) and at the line scanner inside one (~line 299),
  `Id::parse` into `Criterion::id_error` (~line 351), and `parent`/equality/`is_descendant_of` in
  `insertion_point` (~line 447).
- `src/capture.rs`, the only place an `Id` is built without
  `parse` (~line 41), and the parent-first file resolution just below it (~line 56).
- `src/main.rs` uses `looks_like_id` as the capture router
  (~line 98), applied to the tail `peel_root` returns.

## Current Status

Fully implemented and stable. All six public entry points (`Id::parse`, `parent`,
`is_descendant_of`, `depth`, `root_number`, `looks_like_id`) plus the three `Display` impls are in
`src/id.rs`, covered by ten inline `#[test]` functions. No known defects and no open work. The one
behavioral change since the grammar was fixed in DECISIONS.md §4 is the seventh `IdError` variant,
`PaddedLevel`: a zero-padded number is now refused rather than normalized (ID-1.c), pinned by
`rejects_a_zero_padded_level` and end to end by
`cli::a_zero_padded_id_is_refused_rather_than_silently_renamed`. Nothing else in the module moved.

Known coverage gaps, all edges rather than paths: `root_number` has no direct unit test (it is
exercised through `capture::refuses_an_id_that_already_exists` and the rendered hint in
`cli::an_existing_id_refuses_with_exit_1_and_writes_nothing`), no test pins the full `Display`
sentence of any `IdError` variant, and no test covers a multi-letter level or a digit level wider
than `u32`.

## Notes

- `IdError::Alternation { expected }` is a `&'static str` that is interpolated straight into the
  user-facing sentence and compared by value in `rejects_two_letters_in_a_row` and
  `rejects_a_letter_where_the_first_level_must_be_a_number`. Changing the wording breaks those
  tests and changes `hi check` output.
- `valid_family` is private and shared by `Id::parse` and `looks_like_id`, which is what keeps the
  two predicates from drifting apart on the family charset. Any charset change belongs there and
  nowhere else.
- `Display` is now byte-preserving for every string `parse` accepts, because the one input that
  broke that (a zero-padded number) is refused outright. DECISIONS.md §8 assumption 1 (the number
  is a plain integer, no padding) is enforced rather than assumed. Files are still written from the
  human's own text, not from `Display`, so nothing in hi reflows a criterion either way (FILE-4).
- A padded id that is already sitting in a file is not silently re-pointed: `looks_like_id` still
  says `SEND-007` is id-shaped, so `doc` stores the `PaddedLevel` error and `check` reports
  `unparseable-id` at that file and line. Tightening `looks_like_id` to reject padding would turn
  that line back into prose and hide the problem.
- **Fences are `doc`'s business, not this module's.** `doc` now treats ``` and `~~~` blocks as
  opaque and never calls `looks_like_id` for a line inside one (FILE-9). Nothing about fences,
  indentation, or line position belongs here: `looks_like_id` judges one token and nothing else.
- `looks_like_id` has a second call site in `doc`, outside every section, feeding `Doc::stray` so
  `check` can report a criterion nothing would read (CHECK-2.e). The two call sites must agree, so
  any change to the predicate changes both what counts as a criterion and what counts as a stray.
- `main` applies `looks_like_id` to the argument tail left after `peel_root` strips `--root`, and
  only to arguments that convert through `OsStr::to_str`. So a flag is never the token tested
  (CAPTURE-8), and a non-UTF-8 first argument is simply not id-shaped rather than a panic
  (CAPTURE-1.c). Neither concern reaches this module: it still takes a plain `&str`.
- A mistyped family is not an error here, or anywhere. DECISIONS.md §6 accepts that `BILLNG-1`
  creates a stray file; it shows up in `hi check` as a family holding one criterion, and the fix is
  a rename.
- The grammar has no depth limit. Alternation is the only structural bound, so `SEND-1.a.1.b.1.c`
  is a legal, if unlikely, id.
- This module never fails a build, never reads a file, and never decides whether a criterion is
  proven. Those absences are deliberate across all of hi. See DECISIONS.md §5.
