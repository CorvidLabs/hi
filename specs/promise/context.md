---
spec: promise.spec.md
---

## Key Decisions

- **The property is the specification, and the generator is the adversary.** Everything else in
  `specs/` describes behavior a person asked for. This describes the one invariant that spans every
  verb: an id, once seen, belongs to one sentence forever. It is the only thing hi promises
  (DECISIONS.md §4), so it is the only thing that gets a test whose input hi chooses.
- **Twelve defects were found by a person writing a case; none by hi generating one.** That is the
  sentence the module doc opens with and it is the reason the fixtures look the way they do. Every
  generator this repository had written before produced files hi itself would have written, so the
  write path was tested against its own output and agreed with itself. The six `generated_body`
  shapes are all things a person types and hi never emits.
- **The postcondition compares shapes, not ids, and that distinction cost a release.**
  `read_back` used to compare a sorted multiset of ids before and after a write. DECISIONS.md §35
  is the nine lines of markdown that walked through it: an indented `## Retired`, a capture above
  it, and afterwards two active criteria and none retired — SEND-1 live again, still carrying
  `retired: Dropped.`. Every id was still readable. *A postcondition is only as strong as the thing
  it compares.* `Shape` is `(id, section, text, note)` for exactly that reason, and the four fields
  are not negotiable.
- **A verb's report is the claim, so it is never the evidence.** `assert_promise` throws away the
  `Workspace` the step was holding and reloads from disk. Capture returning `Ok` with an id in it
  is precisely what is under test; checking that report against itself would have passed through
  every one of the five id defects on record.
- **A clean `hi check` is an assertion, not a convenience.** Invariant 4 is the one that catches
  silent corruption, and it is written as a conditional because `check` failing is a legitimate
  state mid-sequence: a hand-added criterion can be an orphan, a hand-edit can break alternation.
  What is not legitimate is `check` exiting 0 over a file where an id is written twice. Four of the
  five defects had exactly that shape, before and after.
- **Refusals are steps.** A refusal must leave the workspace shape-identical, not merely
  criterion-identical (hi: CAPTURE-5). The `Err` arms all run the postcondition with nothing named,
  which is how the suite would catch a refusal that had already created `hi/`, written `INTENT.md`,
  or left a lock file behind.
- **The split between the two halves is about what each can ask, and it is not redundancy.**
  `src/promise.rs` is `#[cfg(test)] mod promise;` inside the binary crate, so it can call
  `capture::capture` and `check::run` and can ask `Workspace::load` which *section* a criterion came
  back in. It cannot fork. `tests/promise.rs` can spawn eight processes and can only see bytes on
  disk. A concurrency defect invisible to a serial sequence, and a section defect invisible from
  outside the binary, have both shipped. Moving either half to the other side loses one of them.
- **`Repo::bare` and not `Repo::new`.** The concurrent fixtures create a `.git` and no `hi/`,
  because the first capture in an adopting repository has no directory to put a lock file in, and
  `lock::acquire` used to hand back a `Guard` when the open failed. Every bootstrap capture then ran
  unlocked. A fixture that pre-creates `hi/` cannot reach that path at all (DECISIONS.md §33).
- **Eight threads for the same-id case, thirty-two for the bootstrap case.** The number in
  `tests/cli.rs` is thirty-two because eight did not reproduce the bootstrap race reliably and
  because thirty-two is the shape adoption actually has. The numbers here are eight, which is enough
  for a lock that either exists or does not.
- **A refusal in the concurrent capture test is a failure, and its stderr is printed.** An id nobody
  else asked for is free, so `WANT-7` refusing while `WANT-8` succeeds is a defect. The assertion
  joins every refusal's stderr into the message rather than counting them, because the one time it
  fired it was a lock handoff on Windows and a count says nothing about that.
- **Splitmix64 by hand, eleven lines.** hi depends on `anyhow`, `clap` and `serde_json`. A
  property-testing crate would be a fourth dependency for a generator, in a project whose stated
  rule is minimal dependencies. The tradeoff accepted is that there is no shrinking: a failure gives
  a seed and a step count, not a minimal counterexample.
- **The seeds are fixed and the fixture is chosen by the seed.** `[1, 7, 13, 29, 41, 99, 256, 1024]`
  with `seed % 6` picking the starting body, so the eight runs do not all start from the same shape.
  A clock-seeded generator would find more over time and would make every failure unreproducible,
  which is a worse trade for a gate that has to be green or red.
- **The hand-edit step replaces a line, not a substring.** `step`'s arm 4 finds the line containing
  both the target id and the old sentence before falling back to `body.replacen(&old_text, ..)`.
  Two hand-typed criteria can share a prefix, and a naive substring replace would edit a different
  criterion than the one it named — which would make the step *pass* while testing nothing, because
  the criterion it actually changed would be in the unnamed set and the assertion would fire on the
  wrong thing.

## Files to Read First

- `src/promise.rs`, top to bottom. It is 386 lines and the order is: `Rng`, `Shape` and the four
  accessors over a `Workspace`, `temp_root`, `generated_body`, the id helpers, `assert_promise`,
  `step`, and the two `#[test]` functions. `assert_promise` is the whole contract in 60 lines; read
  it before changing any arm of `step`.
- `tests/promise.rs`, all 220 lines. `typed_file()` is the fixture; the three tests are the
  concurrent half.
- `DECISIONS.md` §26 "The one promise, and the four ways hi broke it" for why a generated test
  exists at all, §33 for the fifth way and the bootstrap lock, §35 for why the postcondition
  compares shapes, and §36 for the reservation lookup the taken-id step leans on.
- `hi/id.md` for ID-5 and ID-5.a, which are the two sentences this module exists to hold true, and
  `hi/file.md` for FILE-19, FILE-22 and FILE-22.c.
- `src/capture.rs` and `src/doc.rs`'s `retire` / `insert` / `read_back`, which are the write paths
  under test. `specs/capture/` and `specs/doc/` describe them; this module only asserts about them.
- `tests/cli.rs`'s `an_id_is_never_handed_out_twice` (`specs/cli/`), which is the hand-written
  counterpart: five named, reproduced defects as five fixed fixtures. The two are complements —
  that test pins the bugs that were found, this module looks for the ones that were not.

## Current Status

Complete and passing. Two `#[test]` functions in `src/promise.rs` (eight seeds, 24 steps each, plus
an eight-step run that pins the fixture) and three in `tests/promise.rs` (eight threads, eight
threads, and a capture racing a retire). All five run under plain `cargo test` with no feature flag
and no environment variable, which is deliberate: a property test behind a flag is a property test
nobody runs.

The suite is a regression net, not a search. It is sized to finish inside an ordinary `cargo test`,
so a long soak is something a person runs by editing the seed list, not something CI does.

Two things a future agent should know:

1. **`step` arm 3 can return early without asserting anything.** If `workspace.find_id(target)`
   comes back `None` — which can happen because `live_ids` was computed from an earlier load — the
   arm returns. That is correct but it means a sequence can silently do fewer steps than 24. Nothing
   counts the steps that actually ran.
2. **The tail of `random_sequences_over_files_hi_did_not_write_keep_the_promise` is nearly a no-op.**
   The loop over `seen` with the `if appearances.len() == 1 { let _ = (first, text); }` body asserts
   nothing; the real assertion after it is that every id in `assigned` is still present. The dead
   loop is harmless and reads as though it checks something. It is listed in `tasks.md`.

## Notes

- `check::run(&workspace).expect("check is operational")` panics rather than skipping if `check`
  itself errors. That is intended: a workspace hi cannot check is already a failure of the promise.
- `shapes_of` drops criteria whose id did not parse, through the `filter_map` on `c.id.as_ref()?`.
  So an unparseable line is not a `Shape` and cannot be compared before-and-after. The hand-edit
  step never produces one, but a future arm that does would be invisible to Invariant 2.
- The `note` field of `Shape` is the retirement reason. It is the field DECISIONS.md §35 found had
  moved to a criterion that did not own it, and it is the reason the comparison is four-field.
- `next_top_level` asks `Workspace::next_free`, so the generated ids follow the same rule a person
  reading the file would. It does not attempt to guess a case's letter; arm 1 picks a letter from
  `a..h` directly and is allowed to collide, because a collision must be refused.
- `tests/promise.rs` asserts `stdout` contains `+WANT-<n>` rather than parsing it. The exact line
  format is `specs/main/`'s contract; this only needs to know which id a process claimed.
- Neither half asserts anything about wall-clock time or ordering between processes. Whether the
  capture or the retire lands first in `concurrent_retire_and_capture_keep_every_unnamed_criterion`
  is not specified and must not be asserted; what is asserted is that both are true afterwards.
