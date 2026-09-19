---
spec: promise.spec.md
---

## Tasks

- [x] Compare shapes rather than ids, so a criterion that changes section, sentence or retirement reason is caught. Evidence: `Shape`, `unnamed_shapes`, and the `a step moved a criterion it did not name` assertion (DECISIONS.md §35).
- [x] Reload from disk inside the postcondition, so a verb's report is never its own evidence. Evidence: `assert_promise`'s `Workspace::load(root)`.
- [x] Make a clean `hi check` an assertion rather than a convenience. Evidence: the `check::run(..).ok()` branch in `assert_promise`.
- [x] Start every sequence from a file hi did not write. Evidence: `generated_body`'s six shapes and `a_file_hi_did_not_write_is_where_the_sequence_starts`.
- [x] Hold a refusal to writing nothing at all. Evidence: the `Err` arms calling `assert_promise` with nothing named.
- [x] Cover the concurrent half through the real binary, from a repository that has never run hi. Evidence: the three tests in `tests/promise.rs` over `Repo::bare`.
- [ ] Remove the dead loop at the end of `random_sequences_over_files_hi_did_not_write_keep_the_promise`. The `if appearances.len() == 1 { let _ = (first, text); }` body asserts nothing and reads as though it does; the surviving assertion is the `assigned.keys()` loop below it.
- [ ] Count the steps that actually ran. `step` arm 3 returns early when `find_id` misses, so a 24-step sequence can perform fewer and nothing notices. A counter asserted at the end would make a sequence that degenerated into no-ops visible.
- [ ] Decide whether a `Shape` should exist for a criterion whose id did not parse. `shapes_of`'s `filter_map` drops them, so an unparseable line is outside Invariant 2 entirely.
- [ ] Add a step that hand-edits a *retirement reason* rather than a sentence. `Shape::note` is compared but no arm changes one, so the field DECISIONS.md §35 was actually about is only exercised through `Doc::retire`.
- [ ] Add a step that runs `hi index` or `hi seed` between writes. Both take the lock and both touch files, and neither is in any sequence.
- [ ] Consider a longer soak behind an explicit command rather than a feature flag, so the seed list can be widened without slowing `cargo test`.

## Gaps

- **No shrinking.** A failure gives a seed and a starting fixture, not a minimal counterexample. Reproducing means running that seed and reading the sequence.
- **No coverage of `hi issue`, `hi export`, `hi ls` or `hi view` inside a sequence.** They only read, so they cannot break the promise directly, but none of them is exercised against a file a sequence mutated.
- **The concurrent half never mixes a hand edit into a race.** A person saving the file in an editor while two hi processes write is the shape most likely to be met in practice and is not covered anywhere.
- **No Windows-specific assertion.** `tests/promise.rs` runs wherever `cargo test` does, and the one refusal ever observed in `concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write` was a lock handoff on Windows. The test prints the stderr it saw; nothing pins the platform behavior.
- **Nothing asserts that the two halves stay on their own side of the line.** A future serial test added to `tests/promise.rs` would quietly lose access to `Section`, and a future concurrent test added to `src/promise.rs` cannot fork at all.
- **The seed list is fixed at eight.** Widening it is a one-line edit nobody is prompted to make, and there is no record of how much of the step space eight seeds × 24 steps actually reaches.
