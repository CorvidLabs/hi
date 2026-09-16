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
- **`looks_like_id` is deliberately weaker than `Id::parse` on case and on level text.** It exists
  so a mistyped id is still recognized as an *intended* criterion. Without that gap, `doc` would
  read `send-2` as prose and `hi check` could never report it (CHECK-2.d). Anything that tightens
  `looks_like_id` toward `parse` silently deletes an error class.
- **It is also deliberately stricter than `parse` on one thing: the first level must start with a
  digit.** That is the entire test that tells `SEND-1` from `spec-sync`, `well-formed` and
  `co-authored`, which are ordinary words a hi file will contain. The rule is load-bearing, and it
  has a known cost: `SEND-a`, `SEND-a.b` and `1ST-4` are not id-shaped, so a criterion written that
  way in a file is read as prose and reported by nothing. `IdError::Alternation { depth: 1 }`, and
  the `IdError::BadFamily` raised by a family starting with a digit, are unreachable from file
  parsing and fire only when the id is an argument to a subcommand, as in `hi issue SEND-a`. As a
  bare first argument they are not even routed to capture: `hi SEND-a "..."` is clap's "unrecognized
  subcommand" at exit 2. The `BadFamily` raised by a *wrongly cased* family is the exception and is
  reachable from a file, which is the whole point of the case-insensitive family test. That trade
  was taken knowingly; it is spec invariant 13 and it is the first thing to reconsider if the
  predicate is ever revisited.
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

- `src/id.rs` is the whole module, 356 lines including its `#[cfg(test)]` block. The module doc
  comment states the grammar; the tests state the intent.
- `DECISIONS.md` §4 ("IDs") and §8 (assumption 1: family charset, no zero padding). This is the
  authoritative record for why the grammar is what it is.
- `hi/format.md` holds the ID family criteria (ID-1 through ID-4) this module serves, in the human's
  own words.
- `src/doc.rs` is the heaviest consumer. It never calls `looks_like_id` directly: `is_criterion_line`
  (~line 605) strips a bullet and any emphasis and calls it on the first token, and that helper has
  three call sites, the stray check outside every section (~line 284), the line scanner inside one
  (~line 296), and the continuation scan, where an indented criterion line ends the one above it
  (~line 338). `Id::parse` feeds `Criterion::id_error` (~line 354), and
  `parent`/equality/`is_descendant_of` drive `insertion_point` (~line 450).
- `src/capture.rs`, the only place an `Id` is built without `parse` (~line 39), and the parent-first
  file resolution just below it (~line 57).
- `src/main.rs` uses `looks_like_id` as the capture router (~line 98), applied to the tail
  `peel_root` returns.

## Current Status

Fully implemented and stable. All six public entry points (`Id::parse`, `parent`,
`is_descendant_of`, `depth`, `root_number`, `looks_like_id`) plus the three `Display` impls are in
`src/id.rs`, covered by eleven inline `#[test]` functions. No known defects and no open work.

Two behavioral changes since the grammar was fixed in DECISIONS.md §4:

1. The seventh `IdError` variant, `PaddedLevel`: a zero-padded number is refused rather than
   normalized (ID-1.c), pinned by `rejects_a_zero_padded_level` and end to end by
   `cli::a_zero_padded_id_is_refused_rather_than_silently_renamed`.
2. `looks_like_id` was rewritten. The family test is now case-insensitive and inline rather than a
   call to `valid_family`, so a wrongly cased id is recognized and then refused with a reason
   instead of vanishing (CHECK-2.d); and the first level must now start with a digit, so an ordinary
   hyphenated word is prose. Pinned by `recognises_id_shaped_tokens`,
   `a_wrongly_cased_family_is_rejected_with_a_reason`, and end to end by
   `cli::a_wrongly_cased_id_is_reported_rather_than_read_as_prose` and
   `cli::an_ordinary_hyphenated_word_is_not_mistaken_for_an_id`.

`Id::parse` itself has not moved since `PaddedLevel` landed.

Known coverage gaps, all edges rather than paths: `root_number` has no direct unit test (it is
exercised through `capture::refuses_an_id_that_already_exists` and the rendered hint in
`cli::an_existing_id_refuses_with_exit_1_and_writes_nothing`), no test pins the full `Display`
sentence of any `IdError` variant, no test covers a multi-letter level or a digit level wider than
`u32`, and nothing asserts the file-level consequence of invariant 13 (that `SEND-a.b` and `1ST-4`
under `## Criteria` are counted and reported by nothing).

## Notes

- `IdError::Alternation { expected }` is a `&'static str` that is interpolated straight into the
  user-facing sentence and compared by value in `rejects_two_letters_in_a_row` and
  `rejects_a_letter_where_the_first_level_must_be_a_number`. Changing the wording breaks those
  tests and changes `hi check` output.
- `valid_family` is private and is called by `Id::parse` **only**. `looks_like_id` no longer uses
  it: it writes its own family test, which accepts either case (`is_ascii_alphabetic` then
  `is_ascii_alphanumeric || '_'`) where `valid_family` demands uppercase. The two are meant to
  differ on case and meant to agree on everything else, and nothing in the code enforces that
  agreement any more, so a charset change has to be made in both places by hand. That is the one
  duplication in the file and it is worth a comment if it is ever touched.
- `Display` is now byte-preserving for every string `parse` accepts, because the one input that
  broke that (a zero-padded number) is refused outright. DECISIONS.md §8 assumption 1 (the number
  is a plain integer, no padding) is enforced rather than assumed. Files are still written from the
  human's own text, not from `Display`, so nothing in hi reflows a criterion either way (FILE-4).
- A padded id that is already sitting in a file is not silently re-pointed: `looks_like_id` still
  says `SEND-007` is id-shaped, so `doc` stores the `PaddedLevel` error and `check` reports
  `unparseable-id` at that file and line. Tightening `looks_like_id` to reject padding would turn
  that line back into prose and hide the problem.
- **Markdown decoration and fences are `doc`'s business, not this module's.** `doc` treats ``` and
  `~~~` blocks as opaque and never calls `looks_like_id` for a line inside one (FILE-9), and
  `doc::is_criterion_line` strips a leading bullet (`- `, `* `, `+ `) through `strip_bullet` and any
  surrounding `*`/`_` through `strip_emphasis` before handing over the first token. So
  `- **SEND-1**` arrives here as `SEND-1`. Nothing about bullets, emphasis, fences, indentation, or
  line position belongs here: `looks_like_id` judges one bare token and nothing else.
- `is_criterion_line` has three call sites in `doc`: the line scanner inside a section, the stray
  check outside every section feeding `Doc::stray` so `check` can report a criterion nothing would
  read (CHECK-2.e), and the continuation scan, where an indented line that is itself a criterion
  ends the one above it. All three must agree, so any change to the predicate changes what counts as
  a criterion, what counts as a stray, and where a criterion's text stops.
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
