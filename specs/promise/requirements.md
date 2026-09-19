---
spec: promise.spec.md
---

## User Stories

- As someone who has written a hundred criteria, I want an id I have seen never to be given to a second sentence, whatever order I captured, retired and hand-edited in, so that the one thing hi promises is the one thing I never have to verify myself (hi: ID-5)
- As someone who keeps a file open in an editor while hi writes to it, I want every criterion to keep its id, its section and its words across a step that did not name it, so that a command about one criterion is a command about one criterion (hi: ID-5.a, FILE-22.c)
- As someone hi has just told "saved", I want that to mean hi can find it again, so that success on stdout is not a different claim from the file on disk (hi: FILE-22)
- As two people on one repository, I want two hi processes writing at once to both land, or one of them to refuse, so that a race is never resolved by silently losing somebody's thought (hi: FILE-19, RETIRE-7)
- As the author of the next change to the write path, I want the test that catches me to start from a file I did not write, so that a bug that only appears in somebody else's markdown is not invisible to the suite (DECISIONS.md §26)
- As the author of a failing build, I want the seed printed with the failure, so that a property test is a reproduction and not a rumour

## Acceptance Criteria

### REQ-promise-001

The promise tests SHALL assert, after every step of every sequence, that each id a verb reported as saved is readable from a freshly loaded workspace in the section it was reported in (hi: FILE-22, ID-5.a).

Acceptance Criteria

- `assert_promise` discards the in-memory `Workspace` the step used and calls `Workspace::load(root)` again, so the verb's own report is never also the evidence for it.
- Each `(id, section)` in `reported` is checked with `readable_in`, which scans `doc.all()` across every loaded doc and compares both the rendered id and the `Section`.
- The failure message names the id and the section: `<id> was reported saved in <section> and Workspace::load cannot find it there`.
- A capture reports `Section::Criteria`; a retire reports `Section::Retired` for the target and for every descendant `Doc::retire` returned as taken.

### REQ-promise-002

The promise tests SHALL assert that every criterion a step did not name is unchanged in its id, its section, its sentence and its retirement reason (hi: ID-5.a, FILE-22.c).

Acceptance Criteria

- `Shape` carries all four fields and derives `Ord`, and the comparison is `assert_eq!` over two sorted `Vec<Shape>` filtered by `unnamed_shapes`.
- The comparison is *not* over ids. A criterion that keeps its id while moving from `## Retired` to `## Criteria`, losing its words, or acquiring a neighbour's `retired:` note must fail (DECISIONS.md §35).
- `named` is computed per step: the captured id; the retired id plus every id `Doc::retire` reported taken; the hand-edited id; the hand-added id. A refusal names nothing.
- The failure message is `a step moved a criterion it did not name`, and the assertion prints both vectors so the changed field is visible in the diff.

### REQ-promise-003

The promise tests SHALL assert that no id a verb assigned ever carries a second sentence (hi: ID-5).

Acceptance Criteria

- `assigned: BTreeMap<String, String>` accumulates across the whole sequence and is never cleared.
- `step` panics with `<id> assigned twice` if `capture` returns `Ok` for an id already in the map, before the postcondition runs.
- `assert_promise` walks every id with more than one appearance and, when that id is in `assigned`, requires every appearance to carry the first sentence.
- A duplicate written by a *person* is permitted here and is `hi check`'s business (`duplicate-id`); only a verb assigning one twice is a failure of this property.

### REQ-promise-004

The promise tests SHALL treat a clean `hi check` as a claim and check it (hi: CHECK-1, ID-5).

Acceptance Criteria

- When `check::run(&workspace).expect("check is operational").ok()` is true, three further assertions run: no id appears more than once anywhere in the workspace, every reported id is readable in its section, and the unnamed shapes are unchanged.
- The failure message for the first is `hi check exited 0 with <id> written twice: <appearances>`.
- This is the assertion that fails on a *silent* corruption. Four of the five id defects on record left the file wrong with `hi check` exiting 0 both before and after (DECISIONS.md §26, §33, §35).
- `check::run` returning `Err` is a panic, not a skip: a workspace hi cannot check is already a failure.

### REQ-promise-005

The promise tests SHALL hold a refusal to writing nothing at all, not merely to writing no criterion (hi: CAPTURE-5).

Acceptance Criteria

- The `Err` arm of the capture step calls `assert_promise` with an empty `reported` and an empty `named`, so every shape in the workspace is compared.
- The taken-id step asserts the call failed before it asserts anything else: `a taken id must refuse, got a write of <id>`.
- A `Doc::retire` that returns `Err` runs the same empty postcondition and returns without saving.

### REQ-promise-006

The promise tests SHALL start every sequence from a file hi itself would not have written (DECISIONS.md §26).

Acceptance Criteria

- `generated_body` returns six distinct bodies chosen by `kind % 6`: bare lines with no bullet; list items with a nested case; a fenced `markdown` example plus a real `## Retired`; two families in one file; CRLF throughout; and a criterion-shaped line stranded below a `# Appendix` heading.
- `a_file_hi_did_not_write_is_where_the_sequence_starts` asserts `!body.contains("**WANT-1**")` before writing the fixture, so a fixture quietly "fixed" into capture's own list form fails the test rather than weakening it.
- `tests/promise.rs` makes the same assertion about `typed_file()` after writing it.
- The seeded sequence picks its body with `seed % 6`, so the eight runs do not all begin from the same shape.

### REQ-promise-007

The promise tests SHALL be deterministic and reproducible from the value printed with a failure.

Acceptance Criteria

- `Rng` is splitmix64 over one `u64` and takes its seed from the fixed list `[1, 7, 13, 29, 41, 99, 256, 1024]`.
- Each seed runs 24 steps against its own temp root, named `serial-<seed>`, so a failure names the seed in the path and in `seed {seed}: ...` assertions.
- No step reads the clock, the environment, or the network.
- The generator adds no dependency to the crate.

### REQ-promise-008

The promise tests SHALL exercise the concurrent half through the real binary, in a repository that has never run hi (hi: FILE-19, RETIRE-7, CAPTURE-5).

Acceptance Criteria

- `tests/promise.rs` spawns `CARGO_BIN_EXE_hi` in threads and passes `--root` rather than relying on a shared working directory.
- `Repo::bare` creates a `.git` and no `hi/`, which is the state a first capture starts in and the only state in which the bootstrap lock defect is reachable (DECISIONS.md §33).
- `concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write` requires every reported success to be readable afterwards and requires that nothing refused, printing each refusal's stderr rather than counting them.
- `concurrent_captures_of_the_same_id_assign_it_once` requires exactly one winner and exactly one occurrence of `WANT-3` in the file.
- `concurrent_retire_and_capture_keep_every_unnamed_criterion` requires both commands to exit 0, `WANT-4` present, `WANT-2` below `## Retired`, and `WANT-1` — which neither named — still carrying the person's words.

### REQ-promise-009

The promise tests SHALL NOT share a scratch path between processes.

Acceptance Criteria

- Both `promise::temp_root` and `tests::promise::Repo::bare` build their directory from `std::process::id()`.
- Each root is `remove_dir_all`ed before it is created, so a sequence never inherits a previous run's file.
- This is the same defect `doc::write_atomically` had, and it is why the integration suite flaked and why bulk capture lost writes.

## Constraints

- This module ships nothing. It must never be reachable from a release build: `src/promise.rs` is behind `#[cfg(test)]` in `src/main.rs` and `tests/promise.rs` is an integration target.
- No new dependency. `Rng` exists because hi's dependency list is `anyhow`, `clap` and `serde_json`, and a property-testing crate would be a fourth for a generator that fits in eleven lines.
- The postcondition compares shapes, never ids. Weakening it back to a comparison of ids reopens DECISIONS.md §35, where every id was still readable and a retired criterion had come back to life.
- The fixtures are files hi did not write, and they stay that way. A fixture rewritten into the form capture emits turns the suite into a round-trip of the writer against itself, which is the failure DECISIONS.md §26 records: twelve defects found by hand, none by a generator.
- The serial half must keep asking `Workspace::load` about `Section`, which is why it lives inside the crate rather than in `tests/`. The concurrent half must keep spawning processes, which is why it does not.
- A failing seed must remain runnable on its own. Do not seed from the clock and do not shuffle the seed list.

## Out of Scope

- What a criterion, an id, a section or a retirement *is*: `specs/id/`, `specs/doc/`.
- The behavior being asserted. Capture's refusals live in `specs/capture/`, retire and the section rules in `specs/doc/`, the seven structural problems in `specs/check/`, the lock in `specs/workspace/`.
- Argv routing, exit codes and the stdout/stderr split, which `tests/cli.rs` covers (`specs/cli/`). The concurrent half here reads stdout only to confirm which id a process claimed.
- Coverage measurement. Whether a line of hi is executed is `fledge coverage`'s question; whether the promise holds is this one's.
- Performance. The sequences are sized to run inside `cargo test` without a flag, not to search the space exhaustively.
