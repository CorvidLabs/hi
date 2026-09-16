---
spec: id.spec.md
---

## Automated Testing

Eleven inline `#[test]` functions live in `src/id.rs` under `mod tests`. Run them with
`cargo test --bin hi id::tests` (11 passed, 70 filtered out), or the whole gate with
`fledge run test`. The end-to-end refusals live in `tests/cli.rs` and run with
`cargo test --test cli`.

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/id.rs` (`id::tests`) | Unit | `parses_a_top_level_id` (REQ-id-001, REQ-id-005, REQ-id-006, REQ-id-008): `SEND-1` gives family `SEND`, levels `[Number(1)]`, renders back to `SEND-1`, `depth() == 1`, `parent().is_none()` |
| `src/id.rs` (`id::tests`) | Unit | `parses_alternating_levels` (REQ-id-001, REQ-id-003, REQ-id-005): `SEND-1.a.1.b` gives `[Number(1), Letter("a"), Number(1), Letter("b")]` and renders back unchanged |
| `src/id.rs` (`id::tests`) | Unit | `rejects_two_letters_in_a_row` (REQ-id-003): `SEND-1.a.b` returns `IdError::Alternation { depth: 3, expected: "a number" }`, pinning both the depth and the wording |
| `src/id.rs` (`id::tests`) | Unit | `rejects_a_letter_where_the_first_level_must_be_a_number` (REQ-id-003): `SEND-a` returns `IdError::Alternation { depth: 1, expected: "a number" }` |
| `src/id.rs` (`id::tests`) | Unit | `rejects_bad_families` (REQ-id-001, REQ-id-002, REQ-id-004): one assertion per variant, namely `send-1`/`1SEND-1` → `BadFamily`, `SE-ND-1` → `BadLevel` (first-hyphen split), `SEND1` → `MissingHyphen`, `SEND-` → `NoLevels`, `SEND-1.` and `SEND-1..a` → `EmptyLevel` |
| `src/id.rs` (`id::tests`) | Unit | `rejects_a_zero_padded_level` (REQ-id-011, REQ-id-005): `SEND-007` and `SEND-1.a.01` return `PaddedLevel` at either depth, while `SEND-0`, `SEND-7`, and `SEND-10` parse, which pins that a bare zero is not padding |
| `src/id.rs` (`id::tests`) | Unit | `allows_underscores_and_digits_in_a_family` (REQ-id-002): `SEND_2FA-1` parses |
| `src/id.rs` (`id::tests`) | Unit | `finds_parents` (REQ-id-005, REQ-id-006): `SEND-1.a.1` → `SEND-1.a` → `SEND-1`, each compared through `Display` |
| `src/id.rs` (`id::tests`) | Unit | `knows_its_descendants` (REQ-id-007): `SEND-1.a` and `SEND-1.a.1` are descendants of `SEND-1`; `SEND-2`, `RECEIPT-1.a`, `SEND-1` itself, and `SEND-11` are not |
| `src/id.rs` (`id::tests`) | Unit | `recognises_id_shaped_tokens` (REQ-id-009): `SEND-1`, `SEND-1.a`, and the wrongly cased `send-2` and `Send-3` are id-shaped; `well-formed`, `spec-sync`, `co-authored`, `SEND-a`, `I`, and `Given` are not. The `SEND-a` assertion is the one that pins the first-level-digit rule, and with it the blind spot in spec invariant 13 |
| `src/id.rs` (`id::tests`) | Unit | `a_wrongly_cased_family_is_rejected_with_a_reason` (REQ-id-002, REQ-id-009): `send-2` and `Send-3` return `BadFamily`, which is the other half of `recognises_id_shaped_tokens`. Together they pin the `looks_like_id`/`parse` gap for case, the case that replaced `SEND-a.b` when the predicate changed |
| `src/capture.rs` (`capture::tests`) | Unit (indirect) | `refuses_an_id_that_already_exists` (REQ-id-008): the "next free is SEND-2" hint is built from `root_number` through `workspace::next_free`, so this is the only unit-level exercise `root_number` gets |
| `src/capture.rs` (`capture::tests`) | Unit (indirect) | `a_case_lands_in_the_file_holding_its_parent` (REQ-id-006): `Id::parent` is what resolves the file, ahead of the declared family, so the parent lookup and the family lookup are proven to be different answers (hi: CAPTURE-4.a) |
| `src/capture.rs` (`capture::tests`) | Unit (indirect) | `refuses_a_case_with_no_parent` and `refuses_a_malformed_id_without_writing` (REQ-id-006, REQ-id-001): the two `Id`-driven refusals, neither of which writes |
| `src/check.rs` (`check::tests`) | Unit (indirect) | `catches_a_malformed_id` (REQ-id-009): proves the `looks_like_id`/`Id::parse` gap reaches the user as a reported problem rather than as silently ignored prose |
| `src/doc.rs` (`doc::tests`) | Unit (indirect) | `inserts_beneath_the_last_descendant_of_a_parent` (REQ-id-006, REQ-id-007): `SEND-1.b` has a parent, so this is the one test that drives both `parent` and `is_descendant_of` through `Doc::insertion_point` |
| `src/doc.rs` (`doc::tests`) | Unit (indirect) | `inserts_after_the_last_criterion_of_the_family` (REQ-id-006): `SEND-2` is depth 1, so it exercises only the `parent() == None` branch of `insertion_point`; `is_descendant_of` is never reached |
| `src/doc.rs` (`doc::tests`) | Unit (indirect) | `records_malformed_ids_rather_than_skipping_them` and `ignores_prose_inside_the_criteria_section` (REQ-id-009): the two halves of the `looks_like_id` gap, one line kept as a broken criterion and one line left as prose |
| `src/doc.rs` (`doc::tests`) | Unit (indirect) | `records_a_criterion_stranded_outside_every_section` (REQ-id-009): `looks_like_id` outside any section feeds `Doc::stray`, so a criterion nothing would read is still noticed (hi: CHECK-2.e) |
| `src/doc.rs` (`doc::tests`) | Unit (indirect) | `a_fenced_block_in_intent_is_not_parsed_as_criteria` and `a_tilde_fence_is_honored_too` (REQ-id-009): an id-shaped line inside a fence never reaches `looks_like_id`, which is why this module needs no notion of fences (hi: FILE-9) |
| `src/check.rs` (`check::tests`) | Unit (indirect) | `catches_a_criterion_stranded_outside_every_section` and `prose_outside_a_section_is_not_a_stray_criterion` (REQ-id-009): the stray path reports an id-shaped line and leaves prose alone, the same gap seen from the other end |
| `tests/cli.rs` | Integration | `a_zero_padded_id_is_refused_rather_than_silently_renamed` (REQ-id-011): `hi SEND-007 "padded seven"` exits 1, says "leading zero", and leaves the file byte identical |
| `tests/cli.rs` | Integration | `a_malformed_id_refuses_without_writing` (REQ-id-003): `hi SEND-1.a.b "two letters in a row"` exits 1 and writes nothing, so the alternation error reaches the user as a refusal |
| `tests/cli.rs` | Integration | `an_id_shaped_argument_captures` (REQ-id-009): `hi SEND-2` followed by a sentence routes to capture with no subcommand, which is `looks_like_id` doing the routing |
| `tests/cli.rs` | Integration | `an_existing_id_refuses_with_exit_1_and_writes_nothing` (REQ-id-008): asserts the rendered "next free is SEND-2" hint, the end-to-end exercise of `root_number` |
| `tests/cli.rs` | Integration | `root_is_honored_by_capture_and_not_swallowed_into_the_sentence` (REQ-id-009): `--root PATH` and `--root=PATH` are peeled before `looks_like_id` sees the first argument, so neither ends up in a sentence (hi: CAPTURE-8) |
| `tests/cli.rs` | Integration | `a_non_utf8_argument_is_reported_not_panicked` (REQ-id-009): an id-shaped first argument routes to capture, then the non-UTF-8 sentence behind it is exit 1 with a sentence rather than a panic (hi: CAPTURE-1.c). The sibling branch (a non-UTF-8 *first* argument, which `to_str` turns into `None` so `looks_like_id` is never asked) is not directly tested |
| `tests/cli.rs` | Integration | `a_wrongly_cased_id_is_reported_rather_than_read_as_prose` (REQ-id-009, REQ-id-002): `- **send-2**  Lowercase, meant as a criterion.` under `## Criteria` makes `hi check` exit 1 with `unparseable-id` naming `send-2`, which is the case half of the `looks_like_id` gap proven end to end |
| `tests/cli.rs` | Integration | `an_ordinary_hyphenated_word_is_not_mistaken_for_an_id` (REQ-id-009): a line of prose reading `spec-sync and well-formed are ordinary words.` leaves `hi check` at exit 0, which is the first-level-digit rule proven end to end |
| `src/doc.rs` (`doc::tests`) | Unit (indirect) | `reads_a_criterion_however_it_was_decorated` (REQ-id-009): `- SEND-1`, bare `SEND-2`, `- **SEND-2.a**`, and `- _SEND-2.b_` all parse, so `doc::is_criterion_line` is proven to strip the bullet and the emphasis before `looks_like_id` sees the token |

## Manual Testing

- [ ] `cargo run --quiet -- SEND-1.a.b "a case of a case"` prints the alternation error naming level
      3 and exits non-zero, writing nothing (REQ-id-003).
- [ ] `cargo run --quiet -- SEND-007 "padded seven"` prints the leading-zero error naming `7` as the
      spelling to use and exits non-zero, writing nothing (REQ-id-011).
- [ ] `cargo run --quiet -- check` on this repo's own `hi/` files exits 0, confirming every
      hand-written id in the project parses (REQ-id-001).
- [ ] Add a line reading `- **send-2**  lowercase` to a scratch hi file and run `hi check`: the
      problem is reported at its file and line as `unparseable-id` rather than skipped as prose
      (REQ-id-009).
- [ ] In the same file add `- **SEND-a.b**  a case of a case` and a paragraph containing `spec-sync`
      and `well-formed`, then run `hi check` and `hi ls`: neither is reported and neither is listed.
      The hyphenated words staying prose is the point; `SEND-a.b` going quiet with them is the
      accepted cost, recorded as spec invariant 13. Confirm the criterion count in the `hi check`
      summary does not include it (REQ-id-009).
- [ ] `cargo run --quiet -- export ID` shows `depth` and `parent` on every criterion, matching the
      dotted ids in `hi/format.md` (REQ-id-008).
- [ ] `fledge run lint` and `fledge run fmt` pass after any edit to `src/id.rs`.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Second hyphen inside the level path (`SE-ND-1`) | Family is `SE`, level is `ND-1`, result is `BadLevel`, not `BadFamily`. Covered by `rejects_bad_families` |
| Empty family (`-1`) | `BadFamily("")`. Not directly asserted; reached through the same `family.is_empty()` guard |
| Trailing or doubled dot (`SEND-1.`, `SEND-1..a`) | `EmptyLevel`. Covered by `rejects_bad_families` |
| Uppercase or mixed level (`SEND-1.A`, `SEND-1a`) | `BadLevel` carrying the level text. Not directly asserted, so this is a gap |
| Multi-letter level (`SEND-1.aa`) | Parses as `Letter("aa")`; a family does not run out of cases at `z`. Not asserted; a gap |
| Zero-padded number (`SEND-007`, `SEND-1.a.01`) | `PaddedLevel` carrying the padded text, at any depth, and never normalized to `SEND-7`. Covered by `rejects_a_zero_padded_level` and end to end by `a_zero_padded_id_is_refused_rather_than_silently_renamed` |
| A bare zero (`SEND-0`) and an unpadded multi-digit number (`SEND-10`) | Both parse; the padding check requires more than one digit. Covered by `rejects_a_zero_padded_level` |
| All-zero level (`SEND-00`) | `PaddedLevel("00")`, correctly refused, but the hint is built with `trim_start_matches('0')`, so the sentence suggests writing it as `''`. Not asserted, another gap |
| A padded id already written in a file (a line that starts `SEND-007`) | `looks_like_id` is `true` and `Id::parse` fails, so `doc` stores the `IdError` and `check` reports `unparseable-id` at that line rather than reading a different criterion. Not directly asserted, a gap. The sibling case for a wrongly cased id *is* asserted, by `cli::a_wrongly_cased_id_is_reported_rather_than_read_as_prose` |
| A wrongly cased id in a file (`send-2`, `Send-3`) | `looks_like_id` is `true`, `Id::parse` returns `BadFamily`, and `check` reports `unparseable-id`. Covered by `recognises_id_shaped_tokens`, `a_wrongly_cased_family_is_rejected_with_a_reason`, and end to end by `cli::a_wrongly_cased_id_is_reported_rather_than_read_as_prose` |
| An ordinary hyphenated word in prose (`spec-sync`, `well-formed`, `co-authored`) | Not id-shaped, because the first level does not start with a digit, so the line stays prose. Covered by `recognises_id_shaped_tokens` and end to end by `cli::an_ordinary_hyphenated_word_is_not_mistaken_for_an_id` |
| A first level that is a letter (`SEND-a`, `SEND-a.b`) written in a file | Not id-shaped, so `Id::parse` is never called and the line is read as prose: no `check` problem, no `ls` entry, and no contribution to the criterion count. `recognises_id_shaped_tokens` asserts `!looks_like_id("SEND-a")`; nothing asserts the file-level consequence, which is the blind spot in spec invariant 13 |
| A family that starts with a digit (`1ST-4`) | Not id-shaped for the same reason the family test requires a leading letter, so the line is prose and `IdError::BadFamily` is unreachable from a file. Reachable only as an argument, where `hi issue 1ST-4` prints the charset sentence. Not asserted, a gap |
| A criterion line decorated with a bullet or emphasis (`- **SEND-1**`, `- _SEND-1_`, bare `SEND-1`) | `doc::is_criterion_line` strips the bullet and the emphasis, so `looks_like_id` always sees a bare token. Nothing about markdown decoration belongs in this module. Covered indirectly by `doc::reads_a_criterion_however_it_was_decorated` |
| Digit level wider than `u32::MAX` | `BadLevel` via the mapped `u32` parse failure, never a panic. Not asserted (gap) |
| `root_number` on an `Id` built without `parse` whose first level is a `Letter`, or with no levels | `None`. Only reachable because `Id`'s fields are public; nothing asserts it, so this is a gap |
| `is_descendant_of` against a longer-numbered sibling (`SEND-11` vs `SEND-1`) | False. Covered by `knows_its_descendants`, and the reason the comparison is over `Level` values rather than text |
| `parent` called repeatedly past depth 1 | Returns `None` and terminates; no panic, no empty-level `Id`. Covered by `parses_a_top_level_id` and `finds_parents` |
| `IdError` `Display` wording for any variant | Interpolated into `hi check` and capture output. No test asserts a full rendered sentence. `Alternation`'s `expected` is pinned only by value equality; `PaddedLevel`'s sentence is pinned only by the substring "leading zero" in `a_zero_padded_id_is_refused_rather_than_silently_renamed`; `BadFamily`'s is pinned only by the substring `send-2` in `cli::a_wrongly_cased_id_is_reported_rather_than_read_as_prose`. The exact bytes, verified against the release binary, are in the spec's Error Cases table. That leaves a gap |
| Digit level at `u32::MAX` boundary (`SEND-99999999999`) | `BadLevel("99999999999")`, verified against the binary: `'SEND-99999999999' is not a valid id: level '99999999999' must be a number or lowercase letters`. Not asserted by a test, a gap |
| An id-shaped line inside a fenced code block | Never reaches `looks_like_id`: `doc` treats the fence as opaque, so this module needs no fence awareness. Covered indirectly by `doc::a_fenced_block_in_intent_is_not_parsed_as_criteria` and `doc::a_tilde_fence_is_honored_too` |
| An id-shaped line outside every section | `looks_like_id` is `true`, `doc` records it in `stray`, and `check` reports it. Covered indirectly by `doc::records_a_criterion_stranded_outside_every_section` and `check::catches_a_criterion_stranded_outside_every_section` |
| Any input at all | No I/O, no panic, no state change. `Id::parse` is total over `&str`: every rejection is an `IdError` (REQ-id-010) |
