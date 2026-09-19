---
spec: promise.spec.md
---

## Automated Testing

This module *is* tests, so "automated testing" here means what runs and what each run asserts,
rather than a separate suite testing a separate implementation. Five `#[test]` functions across
two files, all under plain `cargo test` with no feature flag and no environment variable. Run them
with `cargo test promise` (both halves), `cargo test --bin hi promise::` (the serial half) or
`cargo test --test promise` (the concurrent half). `fledge lanes run verify` runs all of them
through its `test` step.

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/promise.rs` | Property, in-crate, real filesystem under `std::env::temp_dir()` | Random capture / retire / hand-edit sequences over files hi did not write, asserted on shape and section after every step |
| `src/promise.rs` (`temp_root`) | Fixture helper | `<tmp>/hi-promise-<pid>-<name>`, wiped and recreated with a `hi/` inside. The pid is what keeps two `cargo test` runs from sharing a scratch directory |
| `src/promise.rs` (`generated_body`) | Fixture helper | Six starting files, none of which capture would have written: bare lines, nested list items, a fenced example beside a real `## Retired`, two families in one file, CRLF, and a stray criterion below a `# ` heading |
| `src/promise.rs` (`assert_promise`) | Postcondition | The whole property. Reloads from disk, then asserts reported-is-readable, unnamed-is-unchanged, no-second-sentence, and the `hi check` conditional |
| `tests/promise.rs` | Property, integration, spawns `CARGO_BIN_EXE_hi` in threads | The concurrent half: two processes writing the same file, an id eight processes all want, and a retire racing a capture |
| `tests/promise.rs` (`Repo::bare`, `typed_file`) | Fixture helpers | A `.git` and no `hi/`, seeded with a file whose criteria are bare lines. Both halves of the fixture are deliberate: `bare` is the only state the bootstrap lock defect is reachable from, and `typed_file` is asserted not to contain `**WANT-1**` |

### Requirement Coverage

| Requirement | Covering Tests |
|-------------|----------------|
| REQ-promise-001 (reported saved means readable, from disk, in that section) | `assert_promise`'s `readable_in` loop, reached by every step of both `random_sequences_over_files_hi_did_not_write_keep_the_promise` and `a_file_hi_did_not_write_is_where_the_sequence_starts`. Through the binary: `concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write` re-reads the file and requires every `+WANT-<n>` it saw on stdout to be in it |
| REQ-promise-002 (unnamed criteria are unchanged in all four fields) | `assert_promise`'s `assert_eq!(after_unnamed, before_unnamed)`, after every step. Through the binary: `concurrent_retire_and_capture_keep_every_unnamed_criterion` asserts `WANT-1` still carries the sentence the person typed while a capture and a retire both landed |
| REQ-promise-003 (no id carries a second sentence) | `step`'s `assigned` panic on a repeat `Ok`, and `assert_promise`'s walk over `ids_in` for ids in `assigned`. Through the binary: `concurrent_captures_of_the_same_id_assign_it_once` asserts `wins == 1` and one occurrence of `WANT-3` in the file |
| REQ-promise-004 (a clean `hi check` is checked) | The `check::run(..).ok()` branch of `assert_promise`, reached whenever the workspace is structurally clean. Through the binary: the `check.status.success()` branch of `concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write`, which then requires the saved ids to be unique |
| REQ-promise-005 (a refusal writes nothing at all) | The `Err` arms of `step` kinds 0/1 and 3, and kind 2 in full: `assert!(err.is_err(), "a taken id must refuse")` followed by a postcondition with nothing named. **Partly covered:** the assertion is over shapes, so a refusal that created `hi/`, wrote `INTENT.md` or left a lock file behind is not caught here. `cli::a_file_hi_cannot_read_never_frees_the_id_reserved_in_it` covers that with a byte-level `listing` of the whole tree |
| REQ-promise-006 (the sequence starts from a file hi did not write) | `a_file_hi_did_not_write_is_where_the_sequence_starts` asserts `!body.contains("**WANT-1**")` and then that the typed `WANT-1` is still findable after eight steps; `concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write` makes the same assertion about `typed_file()` |
| REQ-promise-007 (deterministic, reproducible from the printed seed) | Structural: `Rng` is splitmix64 over a fixed seed list, `temp_root` names the directory `serial-<seed>`, and the `assigned.keys()` assertion prints `seed {seed}: {id} was saved and is gone`. **Not covered by a test** — nothing asserts that two runs of one seed produce the same sequence |
| REQ-promise-008 (the concurrent half, through the binary, from a bare repository) | All three tests in `tests/promise.rs`. `Repo::bare` is the fixture in every one |
| REQ-promise-009 (no shared scratch path) | Structural: `std::process::id()` in both `temp_root` and `Repo::bare`. **Not covered by a test** — the failure mode is two concurrent `cargo test` runs, which the suite cannot stage against itself |

### What Each Test Asserts

| Test | Assertions |
|------|------------|
| `random_sequences_over_files_hi_did_not_write_keep_the_promise` | For each of eight seeds: a temp root seeded with `generated_body(seed % 6)`, 24 random steps, and `assert_promise` after every one. Then, at the end of the seed, every id in `assigned` is still present in the workspace — `seed {seed}: {id} was saved and is gone` |
| `a_file_hi_did_not_write_is_where_the_sequence_starts` | The fixture does not contain `**WANT-1**`; after eight steps from seed 3, `workspace.find_id(Id::parse("WANT-1"))` is still `Some`. The criterion the person typed survives a sequence of hi's own writes |
| `assert_promise` (every call) | Four things: each `(id, section)` in `reported` is `readable_in` a freshly loaded workspace; `unnamed_shapes(after) == unnamed_shapes(before)`; every appearance of an id in `assigned` carries that id's first sentence; and, when `check::run(..).ok()`, no id appears twice, every reported id is readable, and nothing unnamed moved |
| `concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write` | The starting file does not contain `**WANT-1**`. Eight threads capture `WANT-3`..`WANT-10`; each success prints `+WANT-<n>`; any failure panics with its stderr. Afterwards every saved id is in the file, and both sentences the person typed are still there. If `hi check` exits 0, the saved ids are unique |
| `concurrent_captures_of_the_same_id_assign_it_once` | Eight threads all capture `WANT-3` with different sentences. Exactly one exits 0 — `the same id was assigned {wins} times: {sentences:?}` — `WANT-3` appears exactly once in the file, and `WANT-1`'s typed sentence survives |
| `concurrent_retire_and_capture_keep_every_unnamed_criterion` | After seeding `WANT-3`, one thread captures `WANT-4` and another retires `WANT-2`. Both exit 0. `WANT-1` still carries `The first thought they typed.`; `WANT-4` is in the file; `WANT-2` appears *after* the `## Retired` heading |

## Manual Testing

- [ ] Run a single seed on its own (`cargo test --bin hi promise::random_sequences -- --nocapture` after narrowing the seed list) and read the sequence it performs. A property test whose steps nobody has ever watched is a property test that may be doing nothing.
- [ ] Widen the seed list to a few hundred and run it once before a release. The committed list is sized for the gate, not for a search.
- [ ] Run `cargo test` twice at once, in two terminals, and confirm neither run's fixtures disappear. This is the one thing REQ-promise-009 is about and the suite cannot stage it against itself.
- [ ] Revert the `Shape` comparison to a comparison of ids and confirm the suite still passes over the DECISIONS.md §35 fixture. It should, which is the point: only the four-field comparison catches that bug.
- [ ] Rewrite one `generated_body` fixture into the list form capture emits, and confirm the suite still passes. It will. That is the failure mode DECISIONS.md §26 records, and it is invisible unless somebody goes looking.
- [ ] On Windows, run `cargo test --test promise` several times and confirm no capture of a distinct id ever refuses. The one observed refusal was a lock handoff there.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| A step captures a case whose letter collides with an existing case | The capture is refused, the `Err` arm runs, and the whole workspace must be shape-identical. Arm 1 picks a letter from `a..h` and is allowed to collide on purpose |
| A step captures a case of a criterion that is retired between load and write | Accepted by capture, because `find_id` spans `Doc::all()`. The line joins the family's active block, which `specs/capture/` documents as a known departure. The postcondition does not object: the id is readable in `Section::Criteria`, which is what was reported |
| `live_ids` names an id that `find_id` cannot resolve | Arm 3 returns without asserting. Correct, but it means a sequence can perform fewer than 24 steps and nothing counts them |
| The hand-edit step's target shares a sentence prefix with another criterion | The line-level replacement finds the line carrying both the id and the old text first. Without it the step would edit a criterion it did not name, and the assertion would fire on the wrong one |
| A hand-added criterion is an orphan, or breaks alternation | `hi check` exits non-zero and Invariant 4's conditional simply does not run. The first three assertions still do |
| A criterion's id does not parse | It is not a `Shape` at all, so it is outside the before/after comparison entirely. No arm currently produces one |
| A retire returns `Err` | Not a failure. The arm asserts the empty postcondition and returns without saving |
| Two concurrent processes both refuse | `concurrent_captures_of_the_same_id_assign_it_once` fails with `wins == 0`, which is a different message from `wins == 2`. Both are defects and the message says which |
| A concurrent capture of a *distinct* id refuses | A failure, and every refusal's stderr is printed. An id nobody else asked for is free |
| The order in which a racing capture and retire land | Unspecified and deliberately unasserted. Only the state afterwards is asserted |
| The sequence leaves the workspace structurally broken at the end | Fine. No test asserts a clean `hi check` at the end of a sequence; a hand-added orphan is a legitimate thing for a person to have typed |
| Two `cargo test` runs at once | Each has its own pid and therefore its own scratch tree. This is the whole of REQ-promise-009 |
