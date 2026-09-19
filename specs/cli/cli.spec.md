---
module: cli
version: 1
status: active
files:
  - tests/cli.rs

db_tables: []
depends_on:
  - specs/capture/capture.spec.md
  - specs/check/check.spec.md
  - specs/doc/doc.spec.md
  - specs/id/id.spec.md
  - specs/main/main.spec.md
  - specs/out/out.spec.md
  - specs/view/view.spec.md
  - specs/workspace/workspace.spec.md
---

# CLI

## Purpose

The black-box acceptance suite: sixty-seven tests that spawn the real `hi` binary against a real
temporary repository and assert on what a person would see. It is the only place in hi where the
thing under test is a *process*, and that is the whole reason it exists — argv routing, exit codes,
which stream a line lands on, and what is actually on disk afterwards are properties no unit test
can hold, because every one of them is decided outside the module that produced the value.

Three kinds of claim live here and nowhere else.

**What only a process has.** `looks_like_id` routing an id-shaped first argument to capture before
clap parses anything; `peel_root` taking `--root` only while it leads; `args_os` turning a non-UTF-8
argument into a message instead of a panic; exit 0, 1 and clap's 2; stdout carrying the record of
what landed while stderr carries everything else. A unit test can call `peel_root`. Only a process
can prove that `hi SEND-2 the --root docs option should be documented` writes the word `--root`
into the file and `hi --root /tmp/repo SEND-2 "..."` does not.

**What a file on disk has.** Several of hi's rules are about bytes rather than values: a refusal
leaves the destination byte-for-byte unchanged; a capture into a CRLF or BOM'd or block-frontmatter
file returns it in the person's own style; a criterion renders as a markdown list item rather than a
line in a run-together paragraph; a file hi could not decode is never a file hi replaces. The
assertions are `assert_eq!` over file contents and, in the strictest case, over a `listing` of every
path under the repository root with its bytes — so a refusal can be held to changing *nothing*
rather than to changing no criteria.

**The defects that shipped.** `an_id_is_never_handed_out_twice` is five reproduced bugs as five
fixtures in one function, each one a way a write path broke the single promise hi makes
(hi: FILE-13, FILE-20, FILE-22, RETIRE-5, RETIRE-6), and three more have tests of their own for the
concurrent cases (hi: FILE-19, RETIRE-7). These are hand-written counterparts to `specs/promise/`,
which generates its inputs: that module looks for the bugs nobody found, this one pins the bugs that
were. Neither replaces the other, and the comment above each fixture says which defect it is.

Almost every fixture here is a file **hi did not write**. That is a standing rule rather than a
stylistic preference: three id defects shipped because every write-path test asserted over files hi
itself had produced, so the writer was being tested against its own output and agreed with itself
(DECISIONS.md §26). `Repo::with_chat` seeds a bare `SEND-1  I hit enter and it shows up.` line with
no bullet and no emphasis, which is not what capture emits and is what the parser must keep reading
(hi: FILE-14).

One nested module, `mod nudge`, is here rather than in its own file because it is the same kind of
claim about a different executable: `bin/fledge-hi-nudge`, the fledge lifecycle hook. Its one
load-bearing property is that no path through it can exit non-zero, because fledge propagates a
hook's exit code and a failing hook would abort the command that ran it.

## Public API

| Export | Description |
|--------|-------------|
| None. | An integration test target. Cargo builds it as its own binary against `CARGO_BIN_EXE_hi`; nothing in the crate links it and nothing is compiled into a release build. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Repo` (private) | A throwaway repository under `std::env::temp_dir()`, named `hi-cli-<pid>-<name>`. Three constructors, and which one a test picks is load-bearing — see Invariant 3. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines no traits. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `Repo::new` | `fn new(name: &str) -> Repo` | A root with an empty `hi/` directory. The default fixture for a test that does not care how the workspace came to exist. |
| `Repo::bare` | `fn bare(name: &str) -> Repo` | A root with a `.git` and **no** `hi/`: the state every first capture in an adopting repository starts in, and the only state in which the bootstrap lock defect is reachable (DECISIONS.md §33). |
| `Repo::with_chat` | `fn with_chat(name: &str) -> Repo` | `Repo::new` plus a `hi/chat.md` declaring `families: [SEND]` with an `## Intent` and a **bare** `SEND-1  I hit enter and it shows up.` on line 10. The bare form is deliberate (hi: FILE-14) and line 10 is the number the duplicate-id message quotes. |
| `Repo::write` / `Repo::read` | `fn write(&self, rel: &str, body: &str)` / `fn read(&self, rel: &str) -> String` | Repository-relative file access, used both to seed a fixture and to assert on the result. |
| `Repo::run` | `fn run(&self, args: &[&str]) -> Output` | Spawn `CARGO_BIN_EXE_hi` with `current_dir` set to the root. Tests that need to prove discovery or `--root` behavior build their own `Command` with a different working directory instead. |
| `stdout` / `stderr` | `fn stdout(output: &Output) -> String` | `String::from_utf8_lossy` over the captured stream. Lossy on purpose: a test asserting that a non-UTF-8 argument did not panic must still be able to read what was printed. |
| `listing` | `fn listing(root: &Path) -> Vec<String>` | Every path under the root paired with its bytes, sorted. The strongest assertion in the suite: it is what lets `a_file_hi_cannot_read_never_frees_the_id_reserved_in_it` require that a refusal changed nothing at all, `INTENT.md` and `hi/AGENTS.md` included. |
| `future_format` | `fn future_format(name: &str) -> Repo` | A repository whose one criteria file declares `hi: 2`, a format version this binary has never heard of (hi: FILE-25). |
| `nudge::nudge` / `run` / `run_from` / `repo` / `decoy` | (private, `#[cfg(unix)]`) | Helpers for driving `bin/fledge-hi-nudge` as fledge drives it: the moment as argv, the repository as `FLEDGE_REPO_ROOT`, and a working directory that is deliberately *not* the repository. |

## Invariants

1. **Every fixture path carries the process id.** `hi-cli-<pid>-<name>`, `hi-nudge-<pid>-<name>`,
   `hi-nudge-decoy-<pid>`. A fixed scratch path shared between two `cargo test` runs is the defect
   that made this suite flake and made bulk capture lose writes; it is the same mistake
   `doc::write_atomically` had. The one exception is `outside_a_repository_it_says_so_rather_than_guessing`,
   which uses a fixed `hi-cli-norepo` path — see Error Cases.
2. **The tests drive the binary, never the library.** `CARGO_BIN_EXE_hi` is resolved by cargo at
   compile time, so the suite always runs the binary built from the current tree. A stale
   `target/debug/hi` has twice made a passing change look like a code bug; `cargo clean -p human-intent`
   is the answer when a result here disagrees with the release binary.
3. **The fixture constructor is part of the test's meaning.** `Repo::new` pre-creates `hi/`, so a
   test built on it cannot exercise `lock::acquire` creating the directory, and cannot see the
   bootstrap race at all. A write-path test about a *first* capture must use `Repo::bare`.
4. **A refusal is asserted to change nothing, and the strength of "nothing" is chosen per test.**
   Most refusal tests snapshot the destination file and `assert_eq!` afterwards.
   `a_refusal_over_the_format_version_writes_nothing_at_all` additionally asserts that `INTENT.md`,
   `hi/AGENTS.md`, `hi/CLAUDE.md` and `hi/.hi.lock` do not exist and that `hi/` holds exactly one
   entry. `a_file_hi_cannot_read_never_frees_the_id_reserved_in_it` compares a `listing` of the whole
   tree. Every one of them exists because a refusal that wrote something did ship (hi: CAPTURE-5).
5. **stdout and stderr are asserted separately and the split is the contract.** A refusal must print
   nothing to stdout; a note about a feature list that could not be refreshed must be on stderr while
   the command still exits 0; the nudge hook must never write to stdout at all, because
   `fledge work start --json` puts its envelope there (hi: CAPTURE-11, INDEX-4.a).
6. **Exit codes are asserted as numbers, not as success/failure.** `Some(0)`, `Some(1)` and `Some(2)`
   are distinct claims: 1 is hi refusing, 2 is clap reporting a usage error, and a test that accepts
   either cannot tell an id error from an unknown subcommand.
7. **Assertions quote the whole output on failure.** Nearly every `assert!` in the file carries
   `"{}", stderr(&out)` or the file body. A black-box failure with no output attached is a failure
   nobody can diagnose from CI.
8. **Fixtures are files hi did not write.** Bare criterion lines, CRLF, a BOM, block-style
   frontmatter, an indented `## Retired`, a fenced example of the format, an `INTENT.md` whose
   generated count is deliberately wrong, and one file that is not valid UTF-8. Where a test asserts
   a *count* changed, the seeded count is wrong in a way the correct answer is not, so the assertion
   cannot pass by the block never being touched (DECISIONS.md §26).
9. **`an_id_is_never_handed_out_twice` is a register of shipped defects and only grows.** Five blocks
   today, each with the bug it reproduces named in a comment above it. A block is never deleted when
   the bug is fixed; that is the only thing keeping it fixed.
10. **The concurrency tests assert on what survived, and a quiet refusal is a failure.** Eight
    captures of distinct ids must all land; thirty-two of them must land in a repository that had no
    `hi/`; two concurrent retires must both stay retired. Thirty-two rather than eight because eight
    did not reproduce the bootstrap race reliably and thirty-two is the shape adoption actually has.
11. **No path through the fledge hook may exit non-zero.** `no_path_through_the_hook_can_abort_a_push`
    crosses four moments (`start`, `push`, empty, `unknown-moment`) with four roots (present, with a
    `hi/`, non-existent, unset) and requires exit 0 from all sixteen. fledge propagates the code, so
    a non-zero exit here aborts `fledge work push` for a reason that has nothing to do with the push.
12. **The hook's working directory is a decoy.** `decoy()` is a repository with a `.git` and no
    `hi/`, because hi's own repository *has* a `hi/`: a hook that ignored `FLEDGE_REPO_ROOT` and
    guessed from its cwd would fall silent in the crate root and look correct. Mutation testing found
    exactly that, passing for the wrong reason.
13. **The suite runs everywhere `cargo test` does.** Two tests are `#[cfg(unix)]` —
    `a_non_utf8_argument_is_reported_not_panicked`, which needs `OsStrExt`, and `mod nudge`, which
    runs a shell script. Everything else, the concurrency tests included, runs on Windows too.

## Behavioral Examples

#### Scenario: Capture is reached before clap

- **Given** `Repo::with_chat`
- **When** `hi SEND-2 "it reaches them and the mark changes to sent"` runs
- **Then** exit 0, stdout contains `+SEND-2`, and `hi/chat.md` gains
  `**SEND-2**  it reaches them and the mark changes to sent`
  (`an_id_shaped_argument_captures`)

#### Scenario: A sentence is one line however long it is

- **Given** a 150-character sentence
- **When** it is captured
- **Then** the line in the file equals `- **SEND-2**  <the whole sentence>` exactly, unwrapped and
  untruncated (`a_long_sentence_stays_on_one_line`, hi: FILE-6)

#### Scenario: A file renders as a list, not a wall of text

- **Given** a bare repository, and three captures: `SEND-1`, `SEND-1.a`, `SEND-2`
- **Then** the three criterion lines are exactly `- **SEND-1**  I hit enter and it shows up`,
  `  - **SEND-1.a**  If I have no connection it queues` and `- **SEND-2**  It reaches them` — one
  line each, the case indented two spaces — and `hi export` parses the result back to the same three
  (`a_hi_file_renders_as_a_list_not_a_wall_of_text`, hi: FILE-1.b, FILE-1.c)

#### Scenario: `--root` before the id is a flag; after it, a word

- **When** `hi --root <repo> SEND-2 "it reaches them"` runs from elsewhere, and separately
  `hi SEND-2 the --root docs option should be documented` runs inside the repo
- **Then** the first writes `**SEND-2**  it reaches them` and the string `--root` appears nowhere in
  the file; the second writes `**SEND-2**  the --root docs option should be documented`
  (`root_is_honored_by_capture_and_not_swallowed_into_the_sentence`,
  `a_flag_looking_word_inside_a_sentence_stays_a_word`, hi: CAPTURE-8, CAPTURE-9)

#### Scenario: Five ways an id was handed out twice

- **Given** one test, five fixtures
- **Then** (1) retiring into a file whose `## Retired` is not the last section lands the criterion
  *inside* that section rather than at EOF, and the id stays spent; (2) a retirement reason
  containing `\n- **SEND-5**  forged\n` never produces a line that reads as a criterion, and `SEND-5`
  is still free afterwards; (3) a criterion inside a fence under `## Criteria` is unreadable and
  still taken; (4) a properly closed `markdown` example containing `## Retired` and `- **SEND-4**`
  is prose — the retirement goes to the *real* section, `hi ls --retired` finds it, the example is
  untouched, and `SEND-4` is still free because drawing an id does not burn it; (5) a fence the
  person never closed makes capture refuse twice rather than report the same id saved twice, and the
  half-written file is byte-identical afterwards
  (`an_id_is_never_handed_out_twice`, hi: FILE-13, FILE-20, FILE-22, FILE-22.a, FILE-22.b,
  RETIRE-5, RETIRE-6)

#### Scenario: Thirty-two first captures at once

- **Given** `Repo::bare` — a `.git` and no `hi/`
- **When** thirty-two threads each capture a distinct `SEND-<n>`
- **Then** every process that reported success is readable in `hi/send.md`, and **no** process
  refused; each refusal's stderr would be printed rather than counted
  (`concurrent_captures_into_a_repository_with_no_hi_directory_all_land`, hi: FILE-19, CAPTURE-14)

#### Scenario: A file hi cannot decode is never a file hi replaces

- **Given** an `INTENT.md` holding three years of prose and one Latin-1 byte
- **When** a capture runs
- **Then** the capture succeeds, the criterion is on disk, `INTENT.md` is byte-identical, and stderr
  carries `not refreshed` rather than going quiet
  (`a_root_file_hi_cannot_read_survives_a_capture_byte_for_byte`, hi: INDEX-2.c, INDEX-4.a). The same
  file reached by `hi index`, which *is* allowed to install a section, makes the command refuse with
  exit 1 and every byte intact (`hi_index_refuses_a_root_file_it_cannot_read_rather_than_replacing_it`)

#### Scenario: A later format is refused by every verb

- **Given** a `hi/chat.md` declaring `hi: 2`
- **When** each of eleven invocations runs — `check`, `check --json`, `ls`, `ls --retired`,
  `export`, `issue SEND-1`, `index`, `view`, `retire`, a capture, and `seed`
- **Then** every one exits 1, names both `hi/chat.md` and `hi: 2` on stderr, and prints nothing to
  stdout; and the refusal creates no `INTENT.md`, no `hi/AGENTS.md`, no `hi/CLAUDE.md` and no
  `hi/.hi.lock` (`a_file_from_a_later_format_is_refused_by_every_verb`,
  `a_refusal_over_the_format_version_writes_nothing_at_all`, hi: FILE-25, FILE-25.a)

#### Scenario: The fledge hook cannot break a push

- **When** `bin/fledge-hi-nudge` runs for every combination of four moments and four roots
- **Then** all sixteen exit 0; `start` and `push` say different things on stderr in a repository
  with nothing written down; both go quiet once a `hi/` exists; neither ever writes to stdout; and
  none of them speaks when no `FLEDGE_REPO_ROOT` was given (`mod nudge`)

## Error Cases

These are the failure modes of the suite itself — the ways a test here can be wrong or can fail for
a reason that is not the defect it names.

| Condition | Behavior |
|-----------|----------|
| `target/debug/hi` is stale | Not detected. `CARGO_BIN_EXE_hi` makes cargo rebuild, but a result compared by hand against a stale release binary has twice read as a code bug. `cargo clean -p human-intent` when a result disagrees with what the binary does |
| Two `cargo test` runs at once | Safe. Every fixture path carries `std::process::id()` — except `outside_a_repository_it_says_so_rather_than_guessing`, which uses a fixed `<tmp>/hi-cli-norepo` and would collide with a concurrent run of itself |
| A test is built on `Repo::new` when it means a first capture | Passes for the wrong reason. `hi/` already exists, so `lock::acquire` never creates it and the bootstrap path is never taken. Use `Repo::bare` |
| A fixture is "tidied" into the list form capture emits | Passes, and covers less. The bare `SEND-1  ...` lines are what the parser must keep accepting (hi: FILE-14), and a fixture hi wrote tests the writer against itself (DECISIONS.md §26) |
| An assertion about a generated count is seeded with the correct count | Passes even if the block is never touched. The `INTENT.md` fixtures deliberately carry `(7 criteria)` or `(9 criteria)` against a true answer of 2 |
| `assert!(out.status.success())` used where the exit code matters | Cannot distinguish hi's 1 from clap's 2. The suite asserts `out.status.code() == Some(n)` wherever the number is the claim |
| A hook test run from the crate root | `hi`'s own repository has a `hi/`, so a hook that ignored `FLEDGE_REPO_ROOT` would go quiet and look correct. `decoy()` is a repository with no `hi/` for exactly this reason |
| The nudge helpers on Windows | Not run. `mod nudge` is `#[cfg(unix)]`; `scripts/nudge-behaves.sh` covers the same ground in the local gate only |
| A concurrency test that asserts a count of winners rather than which ones | Loses the diagnosis. `concurrent_captures_into_a_repository_with_no_hi_directory_all_land` joins every refusal's stderr into the message, because the one time it fired it was a lock handoff on Windows |
| A criterion fixture whose line number changes | `Repo::with_chat` puts `SEND-1` on line 10 and the duplicate-id message quotes it. Adding a line to that fixture changes a message another test asserts on |
| `hi check` exiting 0 over a broken file | Precisely what several of these tests exist to catch, and why so many assert on `check`'s output *before and after* the operation under test |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std` | `fs`, `path::PathBuf`, `process::{Command, Output}`, `thread`, `env::temp_dir`, `process::id`, and `ffi::OsStr` / `os::unix::ffi::OsStrExt` on unix |
| `serde_json` | `Value` and `from_str`, for the `export` and `check --json` payload assertions |
| `CARGO_BIN_EXE_hi` | The binary under test, resolved by cargo at compile time |
| `CARGO_MANIFEST_DIR` | `mod nudge` only: locating `bin/fledge-hi-nudge` |
| `src/seed/agents_0_5.md` | `include_str!`ed by `seed_rewrites_a_template_hi_has_shipped_and_refuses_an_edit`, so the "a template hi has shipped" fixture is the shipped bytes rather than a copy |

### Consumed By

| Module | What is used |
|--------|-------------|
| None. | An integration test target. Nothing links it. |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-19 | Claude | Initial specification. Written after `fledge atlas` reported `tests/cli.rs` as the largest file in the repository under no spec (1,819 lines). Documents the three kinds of claim only a process can make, the three fixture constructors and why the choice between them is load-bearing, the register of five reproduced id defects, the strength ladder for "a refusal writes nothing", and the nested fledge-hook module. |
