---
module: promise
version: 1
status: active
files:
  - src/promise.rs
  - tests/promise.rs

db_tables: []
depends_on:
  - specs/capture/capture.spec.md
  - specs/check/check.spec.md
  - specs/doc/doc.spec.md
  - specs/id/id.spec.md
  - specs/main/main.spec.md
  - specs/workspace/workspace.spec.md
---

# Promise

## Purpose

The property tests of the one promise hi makes: **an id a person has seen is never given to a
second sentence, a criterion hi said it saved is one hi can find again, and a step leaves every
criterion it did not name exactly as it was** (hi: ID-5, ID-5.a, FILE-22.c).

Every other spec in this repository describes a module that ships. This one describes the
adversary. It owns no product behavior and exports nothing; what it owns is a *method*, and the
method is the reason it is specified rather than left as an unremarkable test file. Twelve
confirmed defects in the write path were found by a person sitting down and writing a case. None
were found by hi generating one, because every generator hi had written until then generated files
hi itself would have written (DECISIONS.md §26). So the fixtures here are deliberately files hi
would never produce — bare criterion lines with no bullet, CRLF from Notepad, a fenced example of
the format inside somebody's intent prose, an indented `## Retired`, a stray line below a `# `
heading — and the sequences run over them are random.

The module is in two halves, split by what each half can ask.

`src/promise.rs` is compiled into the binary crate under `#[cfg(test)]`, so it can call
`capture::capture`, `Doc::retire` and `check::run` directly and can ask `Workspace::load` which
*section* a criterion came back in. That is what makes its postcondition a comparison of shapes —
`(id, section, sentence, reason)` — rather than a comparison of ids. A comparison of ids is what
`read_back` used to do, and DECISIONS.md §35 is the nine lines of markdown that walked through it:
every id was still readable afterwards, and a retired criterion had come back to life carrying
somebody else's retirement reason. *A postcondition is only as strong as the thing it compares.*

`tests/promise.rs` runs the release binary in threads, which is the only way to reach the half of
the promise that is about two processes at once: a lock that is really the kernel's, a
read-modify-write that reloads under it, and an id that eight simultaneous writers all ask for
(hi: FILE-19, RETIRE-7). It asserts through the filesystem, because that is all a second process
leaves behind.

Neither half is a unit test of anything. A unit test names a function and an expected answer; these
name a property and let the generator find the input. When one fails, the seed is in the assertion
message and the failing sequence is reproducible by running that seed alone.

## Public API

| Export | Description |
|--------|-------------|
| None. | `src/promise.rs` is a private `#[cfg(test)]` module of the binary crate and declares no `pub` item; `tests/promise.rs` is an integration test binary. Nothing in hi depends on either, and neither is compiled into a release build. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Rng` (private, `src/promise.rs`) | Splitmix64 over a single `u64`. `next` advances and mixes, `pick` chooses from a slice, `below(n)` bounds to `0..n` with `n.max(1)`. Hand-written on purpose: hi's dependency budget is `anyhow`, `clap` and `serde_json`, and a generator is not worth a fourth (DECISIONS.md §26). Deterministic, so a failing seed is a reproduction. |
| `Shape` (private, `src/promise.rs`) | `{ id: String, section: Section, text: String, note: Option<String> }`. One criterion as a *reader of the file* would find it. Derives `PartialEq, Eq, PartialOrd, Ord` so a whole workspace can be compared as a sorted `Vec<Shape>`. The four fields are exactly the four things §35 found a comparison of ids could not see. |
| `Repo` (private, `tests/promise.rs`) | A throwaway repository under `std::env::temp_dir()`. Only `bare` exists here: a `.git` and **no** `hi/`, because the first capture in an adopting repository starts there and a fixture that pre-creates `hi/` cannot see the bootstrap lock defect (DECISIONS.md §33). |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines no traits. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `shapes_of` | `fn shapes_of(workspace: &Workspace) -> Vec<Shape>` | Every criterion in every loaded doc, active and retired alike, as a sorted `Vec<Shape>`. Criteria whose id did not parse are dropped by the `filter_map`, so an unparseable line is not a shape and cannot be compared. |
| `ids_in` | `fn ids_in(workspace: &Workspace) -> BTreeMap<String, Vec<(Section, String)>>` | Every *appearance* of every id, keyed by id. A key with more than one entry is an id written twice, which is the promise's headline failure. |
| `readable_in` | `fn readable_in(workspace: &Workspace, id: &str, section: Section) -> bool` | Whether a freshly loaded workspace can find that id in that section. "Reported saved" is checked against this, never against the return value the verb gave. |
| `unnamed_shapes` | `fn unnamed_shapes(before: &[Shape], named: &BTreeSet<String>) -> Vec<Shape>` | The shapes whose id the step did not name. These must be identical before and after; the named ones are the only ones a verb is allowed to change. |
| `temp_root` | `fn temp_root(name: &str) -> PathBuf` | `<tmp>/hi-promise-<pid>-<name>`, removed and recreated with a `hi/` inside. The pid is not decoration: a fixed scratch path shared by two `cargo test` runs is the bug that made this suite flake and made bulk capture lose writes. |
| `generated_body` | `fn generated_body(kind: u32) -> String` | Six starting files, chosen by `kind % 6`, **none of which capture would have written**. See Behavioral Examples for the list. |
| `live_ids` | `fn live_ids(workspace: &Workspace) -> Vec<Id>` | Parsed ids from `doc.criteria` only, so a step that wants something to retire or to hang a case off does not pick a retired one. |
| `all_parsed_ids` | `fn all_parsed_ids(workspace: &Workspace) -> Vec<Id>` | Parsed ids from `doc.all()`, active and retired, so the "capture a taken id" step can pick a retired one and must still be refused. |
| `next_top_level` | `fn next_top_level(workspace: &Workspace, family: &str) -> String` | `<FAMILY>-<Workspace::next_free>`, the id a person reading the file would type next. |
| `assert_promise` | `fn assert_promise(root, reported: &[(String, Section)], before: &[Shape], named: &BTreeSet<String>, assigned: &BTreeMap<String, String>)` | The postcondition. Reloads the workspace from disk and asserts the four things in Invariants 1 to 4. Called after **every** step, including the ones that refuse. |
| `step` | `fn step(rng: &mut Rng, root: &Path, assigned: &mut BTreeMap<String, String>)` | One random move against the repository: capture a new id, capture a case, capture a taken id, retire, hand-edit a sentence, or hand-add a criterion. Each arm computes what it named and calls `assert_promise`. |
| `typed_file` | `fn typed_file() -> &'static str` (`tests/promise.rs`) | A file a person typed: frontmatter, an `## Intent`, and `WANT-1` and `WANT-2` as **bare** lines. Capture would have written `- **WANT-1**`, and the tests assert that it does not contain that, so a run that only round-trips hi's own output cannot pass. |

## Invariants

1. **An id a verb reported saved is readable, from disk, in the section it was reported in.** Not
   in the value the verb returned: `assert_promise` throws the in-memory workspace away and calls
   `Workspace::load(root)` again. A verb's own report is the claim under test, so it is never also
   the evidence.
2. **Every criterion the step did not name is unchanged in all four of its fields.** `before` and
   `after` are filtered through `unnamed_shapes` and compared with `assert_eq!`. Id, section,
   sentence and retirement reason are all in the comparison, because a criterion that keeps its id
   while changing section, changing its words, or inheriting a neighbour's `retired:` note is a
   criterion that was lost (DECISIONS.md §35, hi: FILE-22.c, ID-5.a).
3. **No id a verb assigned ever carries a second sentence.** `assigned` accumulates
   `id -> sentence` across the whole run, and any later appearance of that id must carry the same
   words. A *person* may write the same id twice — that is a duplicate, and `hi check` reports it —
   but a verb doing it is the promise breaking (hi: ID-5).
4. **`hi check` exiting 0 is a claim, and the claim is checked.** When `check::run(..).ok()` is
   true, `assert_promise` additionally requires that no id appears more than once anywhere, that
   every reported id is readable, and that nothing unnamed moved. This is the assertion that makes
   a silent corruption fail: four of the five id defects in DECISIONS.md §26, §33 and §35 left the
   file wrong and `hi check` exiting 0 both before and after.
5. **A refusal is a step too.** Every arm that can fail calls `assert_promise` on the failure path
   with an empty `reported` and an empty `named`, so a refused capture is held to changing
   *nothing*, not merely to not writing a criterion (hi: CAPTURE-5).
6. **The sequence starts from a file hi did not write.** `generated_body` produces six shapes and
   `a_file_hi_did_not_write_is_where_the_sequence_starts` asserts the starting text does not contain
   `**WANT-1**`. `tests/promise.rs` asserts the same of `typed_file`. This is the guard against the
   failure mode DECISIONS.md §26 names: a generator that round-trips the writer's own output tests
   the writer against itself and finds nothing.
7. **The generator is deterministic and the seeds are fixed.** `[1, 7, 13, 29, 41, 99, 256, 1024]`,
   24 steps each, with the starting file chosen by `seed % 6` so the eight runs do not all begin
   from the same shape. A failure names its seed, and running that seed alone reproduces it. Do not
   make the seed a function of the clock: a property test that cannot be re-run is a bug report with
   no repro.
8. **Nothing here is allowed to need a new dependency.** `Rng` is eleven lines because the
   alternative is a proptest or quickcheck dependency in a crate whose whole dependency list is
   three entries, two of which are clap and anyhow.
9. **The two halves do not overlap and neither is redundant.** `src/promise.rs` can see sections
   and shapes and cannot see a second process; `tests/promise.rs` can see a second process and can
   only see bytes. A concurrency defect that is invisible to a serial sequence, and a section
   defect that is invisible from outside the binary, have both shipped.
10. **The concurrent half asserts on what survived, not on a count.** In
    `concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write` every process that
    *reported* success must be readable in the file, and no process may quietly refuse — an id
    nobody else asked for is free, so a refusal is a failure and its stderr is printed rather than
    counted. The one time that fired it was a lock handoff on Windows, and a count would have said
    nothing about it.
11. **The temp roots carry the pid.** Both halves build their scratch directory from
    `std::process::id()`. Two `cargo test` runs at once otherwise share a fixture, wipe each other's
    files, and the failures read as flakes.

## Behavioral Examples

#### Scenario: The six starting files, none of which hi wrote

- **Given** `generated_body(kind)` for `kind % 6` in `0..6`
- **Then** the sequence starts from, in order: bare criterion lines with no bullet and no emphasis
  (hi: FILE-14); list items with a nested case; a fenced `markdown` example of the format inside
  `## Intent`, plus a real `## Retired` below it (hi: FILE-9, FILE-22.b); two families declared in
  one file; a file saved with CRLF line endings (hi: FILE-10); and a file with a criterion-shaped
  line stranded below a `# Appendix` heading (hi: CHECK-2.e)

#### Scenario: A capture that succeeds

- **Given** any point in a sequence
- **When** `step` captures a new top-level id or a case of a live one and `capture` returns `Ok`
- **Then** the id is recorded in `assigned` with its sentence — panicking immediately if that id was
  already assigned — and `assert_promise` requires it readable in `Section::Criteria`, every other
  criterion unchanged, and, if `hi check` exits 0, no id written twice anywhere

#### Scenario: A capture that refuses

- **Given** a workspace holding `WANT-1`, active or retired
- **When** `step` captures `WANT-1` again with a different sentence
- **Then** the call must return `Err` — `assert!(err.is_err(), "a taken id must refuse")` — and
  `assert_promise` runs with nothing named, so the whole workspace must be shape-identical
  (hi: CAPTURE-3, CAPTURE-5)

#### Scenario: A retire names its descendants

- **Given** a live `WANT-1` with cases under it
- **When** `step` calls `Doc::retire` and saves
- **Then** the ids it returns as taken, plus the target, are the `named` set, every one of them must
  be readable in `Section::Retired`, and every criterion outside that set is unchanged
  (hi: RETIRE-2, ID-5.a)

#### Scenario: A person edits a sentence by hand

- **Given** any criterion in the file
- **When** `step` rewrites its sentence directly on disk
- **Then** only that id is `named`, and every other criterion must be untouched. The replacement
  finds the *line* carrying both the id and the old text before falling back to a substring
  replace, because two hand-typed criteria can share a prefix and a naive `replacen` would edit the
  wrong one — the fixture bug that would have made this step silently test nothing

#### Scenario: A person types a criterion hi did not write

- **Given** a workspace whose next free id in `WANT` is `WANT-5`
- **When** `step` appends `- **WANT-5**  a hand-typed line for WANT-5.` to the file itself, creating
  a `## Criteria` heading if there is none
- **Then** `WANT-5` is `named`, everything else is unchanged, and the *next* capture must not hand
  `WANT-5` out again (hi: FILE-14, CAPTURE-3)

#### Scenario: Eight processes capture eight different ids

- **Given** a bare repository whose `hi/want.md` is `typed_file()`
- **When** eight threads run the real binary for `WANT-3` through `WANT-10` at once
- **Then** every one exits 0, each prints `+WANT-<n>`, every reported id is in the file afterwards,
  and the two sentences the person typed are still there (hi: FILE-19)

#### Scenario: Eight processes ask for the same id

- **Given** the same repository
- **When** eight threads all run `hi WANT-3 "sentence from waiter <i>"`
- **Then** exactly one exits 0 and `WANT-3` appears exactly once in the file. `wins` is asserted
  equal to 1 and the winning stdout is printed on failure, so "two of them won" and "none of them
  won" are different messages (hi: ID-5, FILE-19)

#### Scenario: A retire and a capture at the same time

- **Given** a repository holding `WANT-1`, `WANT-2` and `WANT-3`
- **When** one thread captures `WANT-4` while another retires `WANT-2`
- **Then** both exit 0, `WANT-4` is in the file, `WANT-2` is below `## Retired`, and `WANT-1` —
  which neither named — still carries the words the person typed (hi: RETIRE-7, FILE-22.c)

## Error Cases

This module produces no user-facing errors. Its "error cases" are the failures it is built to
report, and each one names what it saw.

| Condition | Behavior |
|-----------|----------|
| A verb reports an id saved and `Workspace::load` cannot find it in that section | Panic: `<id> was reported saved in <section> and Workspace::load cannot find it there` (hi: FILE-22) |
| A step changes a criterion it did not name | Panic: `a step moved a criterion it did not name`, with the two sorted `Vec<Shape>` in the diff, so the field that changed is visible (hi: ID-5.a) |
| A verb writes a second sentence under an id it already assigned | Panic: `<id> was assigned a second sentence by a verb: first <..>, now <..>` (hi: ID-5) |
| `capture` returns `Ok` twice for one id | Panic from `step` before the postcondition runs: `<id> assigned twice: first <..>, then <..>` |
| `hi check` exits 0 while an id is written twice | Panic: `hi check exited 0 with <id> written twice: <appearances>`. This is the shape every silent corruption in DECISIONS.md §26, §33 and §35 had |
| `hi check` exits 0 while a reported id is unreadable, or an unnamed criterion moved | Panic naming which of the two it was |
| Capturing an id that is already taken returns `Ok` | Panic: `a taken id must refuse, got a write of <id>` |
| A concurrent capture of a distinct id refuses | Panic listing every refusal with its stderr: `every capture asked for an id of its own, and these did not land:` |
| Two concurrent captures of one id both succeed | Panic: `the same id was assigned <n> times: <their stdout>` |
| The starting fixture turns out to be something hi would have written | Panic from the fixture assertion itself, before any step runs |
| `Workspace::load` fails at any point in a sequence | Panic from `.expect("workspace still loads")`. A file hi wrote that hi can no longer read is a failure of the promise, not a skipped test |
| A retire fails mid-sequence | Not a failure. The arm calls `assert_promise` with nothing named and returns: a refusal that writes nothing is allowed, and is still held to writing nothing |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std` | `collections::{BTreeMap, BTreeSet}`, `fs`, `path::{Path, PathBuf}`, `process::{Command, Output}`, `thread`, `env::temp_dir`, `process::id` |
| `crate::capture` | `capture`, and the `Captured` it returns |
| `crate::check` | `run`, `Report::ok` — the conditional in Invariant 4 |
| `crate::doc` | `Section`, and `Doc::retire` / `Doc::save` through the workspace's docs |
| `crate::id` | `Id`, `Id::parse` |
| `crate::workspace` | `Workspace::load`, `Workspace::find_id`, `Workspace::next_free` |
| `CARGO_BIN_EXE_hi` | `tests/promise.rs` only: the release binary under test, resolved by cargo at compile time |

### Consumed By

| Module | What is used |
|--------|-------------|
| None. | `#[cfg(test)]` and an integration target. Nothing in a release build references either file. |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-19 | Claude | Initial specification. Written after `specsync coverage` reported `src/promise.rs` as the repository's one uncovered file (386 LOC), and `fledge atlas` reported `tests/promise.rs` alongside it. Documents the property, the two halves and why they are two, the six fixtures and why none of them is a file hi would write, and the eleven invariants that make a failure here reproducible. |
