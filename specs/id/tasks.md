---
spec: id.spec.md
---

## Tasks

- [x] Specify the id grammar, the six public entry points, and the three `Display` impls. Evidence: `specs/id/id.spec.md`, `status: active`.
- [x] Map every REQ to real inline tests. Evidence: `specs/id/testing.md`, `cargo test --bin hi id::tests` (11 passing).
- [x] Recognize a wrongly cased id so it is refused with a reason instead of read as prose (REQ-id-009, hi: CHECK-2.d), and require the first level to be a digit so `spec-sync` and `well-formed` stay ordinary words. Evidence: the inline family test in `looks_like_id` in `src/id.rs`, `recognises_id_shaped_tokens`, `a_wrongly_cased_family_is_rejected_with_a_reason`, `tests/cli.rs::a_wrongly_cased_id_is_reported_rather_than_read_as_prose`, `tests/cli.rs::an_ordinary_hyphenated_word_is_not_mistaken_for_an_id`.
- [x] Refuse a zero-padded numeric level instead of normalizing it (REQ-id-011, hi: ID-1.c). Evidence: `IdError::PaddedLevel` in `src/id.rs`, `rejects_a_zero_padded_level`, `tests/cli.rs::a_zero_padded_id_is_refused_rather_than_silently_renamed`.
- [ ] Add a direct unit test for `root_number`, including the `None` branch on an `Id` built without `parse` (REQ-id-008).
- [ ] Add a test pinning the rendered `Display` sentence of each `IdError` variant, since those strings are user-facing `hi check` output (REQ-id-004, REQ-id-011).
- [ ] Add cases for a multi-letter level (`SEND-1.aa`), an uppercase level (`SEND-1.A`), and a digit level wider than `u32::MAX` (REQ-id-004, REQ-id-005). All three were confirmed by hand against the release binary: `SEND-1.aa` captures, `SEND-1.A` is `BadLevel`, `SEND-99999999999` is `BadLevel`.
- [ ] Decide whether spec invariant 13 is acceptable as shipped, and pin the answer with a test either way. Today `- **SEND-a.b**  ...` and `- **1ST-4**  ...` under `## Criteria` are counted by nothing and reported by nothing, because `looks_like_id` requires a leading letter in the family and a digit in the first level. That is silence, which DECISIONS.md §11 names as the failure mode worth fearing, and it sits against ID-4 and CHECK-2.d at depth 1. It is also the direct price of keeping `spec-sync` and `well-formed` as prose, so it is a product call, not a bug fix (REQ-id-009).

## Gaps

- `root_number` has no direct test; it is only exercised through `capture::refuses_an_id_that_already_exists` building the "next free is SEND-2" hint and the rendered hint in `tests/cli.rs`.
- No test asserts the full `Display` text of `IdError`. `Alternation`'s `depth` and `expected` are compared by value rather than through the rendered sentence, and `PaddedLevel`'s sentence is pinned only by the substring "leading zero" in `tests/cli.rs`.
- Untested but reachable inputs: a multi-letter level, an uppercase or mixed level, and a digit level that overflows `u32`.
- Nothing asserts that a zero-padded id already written into a file surfaces as a `check` problem rather than resolving to the unpadded criterion; only the capture path is covered.
- Nothing tests an `Id` built directly through its public fields, which is the one way to hold an id that does not satisfy the alternation rule (`capture` does this deliberately).
- Nothing asserts the file-level consequence of `looks_like_id`'s first-level-digit rule. `recognises_id_shaped_tokens` pins `!looks_like_id("SEND-a")`, but no test reads a file containing `- **SEND-a.b**` or `- **1ST-4**` and asserts that `hi check` stays quiet and the criterion count excludes it. That is the assertion that would make spec invariant 13 visible to anyone who changes the predicate.
- `valid_family` and `looks_like_id`'s inline family test are now separate code with no test tying them together, so a charset change made in one and not the other would pass the suite.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: not applicable (no user-facing surface beyond error text)
- **Dev**: pending
