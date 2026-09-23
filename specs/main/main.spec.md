---
module: main
version: 1
status: active
files:
  - src/main.rs

db_tables: []
depends_on:
  - specs/capture/capture.spec.md
  - specs/check/check.spec.md
  - specs/doc/doc.spec.md
  - specs/id/id.spec.md
  - specs/out/out.spec.md
  - specs/view/view.spec.md
  - specs/workspace/workspace.spec.md
---

# Main

## Purpose

The binary entry point and the whole user-facing command surface. `main` owns argument parsing,
the routing decision between capture and the named subcommands, process exit codes, and the human
rendering of `hi check`'s report. It owns no domain logic: every verb delegates immediately to the
module that implements it.

One routing decision is load-bearing. Capture is the default action, so an id-shaped first
argument is routed to `capture` **before clap parses anything** (`hi SEND-2 "<sentence>"`).
`looks_like_id` splits on the first `-` and asks for two things: a family that is non-empty and
begins with an ASCII letter, and a digit immediately after the hyphen. A flag splits to an empty
family, so `--help`, `-V` and `--json` are never id-shaped; no subcommand name carries a hyphen at
all, so `check`, `ls`, `issue`, `export`, `index` and `view` pass through to clap untouched. This
is what makes writing a criterion one command with no subcommand to remember (hi: CAPTURE-1).

The family test is deliberately case-insensitive, so `hi send-2 "<sentence>"` still routes to
capture and is then refused by `Id::parse` with a reason, rather than being read as a subcommand
(hi: CHECK-2.d). The digit test is what keeps an ordinary hyphenated word such as `spec-sync` or
`well-formed` out of the capture route.

Two details of that pre-parse matter. `--root` is peeled out of argv by `peel_root` *before* the
routing test, and only while it leads: the peel stops at the first argument that is not a `--root`
form, so an option ahead of the id reaches capture as a root, and after the id every token is the
person's sentence (hi: CAPTURE-8). And argv is read as `args_os`, because `env::args()` panics on
a non-UTF-8 argument; a sentence that is not valid UTF-8 becomes a plain error instead of a
backtrace (hi: CAPTURE-1.c).

## Public API

| Export | Description |
|--------|-------------|
| None. | A binary crate. Every item in this file is private; the entry point is `fn main`. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Cli` (private) | clap `Parser` for the non-capture surface: a global `--root` and one required subcommand. `arg_required_else_help` makes bare `hi` print help. |
| `Command` (private) | clap `Subcommand` enum: `Check { json }`, `Ls { family, retired }`, `Issue { id, create, repo }`, `Export { scope }`, `Retire { id, reason }`, `Index`, `View { out }`, `Seed`. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines no traits. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `main` | `fn main() -> ExitCode` | Collects `std::env::args_os()`, peels `--root` with `peel_root`, and routes an id-shaped first remaining argument to capture; otherwise parses `Cli` and dispatches. Converts an error into a printed message and exit 1. |
| `peel_root` (private) | `fn peel_root(args: &[OsString]) -> (Option<PathBuf>, Vec<OsString>)` | Take a *leading* `--root PATH` or `--root=PATH` out of the arguments, returning the path and everything else in order. The loop consumes `--root` forms from the front and `break`s at the first argument that is not one, so options belong before the id and everything from the id onward is the sentence (hi: CAPTURE-8). Among leading occurrences the last wins. Two shapes are not peeled even in front: `--root` with nothing after it, because the arm guards on `index + 1 < args.len()`; and `--root=PATH` whose token is not valid UTF-8, because that arm tests the whole token with `to_str()`. Either then leads the tail, which is not id-shaped, so clap sees it and reports a usage error. |
| `fail` (private) | `fn fail(err: anyhow::Error) -> ExitCode` | Print `error: {err:#}` to stderr and return exit 1. The `:#` carries the whole anyhow context chain. |
| `run_capture` (private) | `fn run_capture(root: Option<&Path>, raw_id: &str, rest: &[String]) -> Result<()>` | Join the remaining argv into a sentence, find the workspace from `root` or the current directory, capture, and print the destination and id. |
| `run` (private) | `fn run(cli: Cli) -> Result<ExitCode>` | Resolve the workspace from `--root` or the current directory, then dispatch one subcommand. |
| `print_report` (private) | `fn print_report(report: &check::Report)` | Render a `check::Report` for a terminal: problems grouped by file, then a one-line summary. |
| `plural` (private) | `fn plural(count: usize, one: &str, many: &str) -> String` | `"1 criterion"` / `"12 criteria"`. |

## Invariants

1. An id-shaped first argument (first *after* a leading `--root` has been peeled) always routes to
   capture, and is never interpreted as a subcommand. `looks_like_id` requires a family that is
   non-empty, begins with an ASCII letter of either case, and is otherwise ASCII alphanumeric or
   `_`; and it requires the character right after the `-` to be a digit. No flag can collide,
   because a flag's family part is empty, and no defined subcommand name can, because none holds a
   hyphen. The digit rule is also what keeps `spec-sync` and `well-formed` off the capture route.
   A token the predicate rejects reaches clap instead, so `hi SEND-a "<sentence>"` is an
   unrecognized subcommand and exit 2, not an id error.
2. Bare `hi` with no arguments prints help and does not touch the filesystem: nothing is asked of
   the person, and nothing is written (hi: CAPTURE-1.b).
3. Exit codes: `0` success, `1` any returned error (a structural problem from `check`, or a refusal
   from capture), and `2` from clap for a usage error.
4. `hi check` exits 1 when and only when `Report::ok()` is false. Every other verb exits 0 on
   success (hi: CHECK-1, CHECK-2).
5. Errors print to stderr; all normal output prints to stdout, so `hi export <scope> > file` is clean.
6. `--root` is global and affects every verb, capture included, **when it comes before the id**.
   `peel_root` removes a leading `--root PATH` or `--root=PATH` from argv before the routing test,
   so it is not joined into a criterion's sentence (hi: CAPTURE-8). Past the id the peel has
   already stopped, so a `--root` inside a sentence stays a word:
   `hi SEND-2 the --root docs option should be documented` captures the sentence whole, and
   `hi SEND-2 "text" --root /elsewhere` captures `text --root /elsewhere` and exits 0.
   Because the non-capture path calls `Cli::parse()`, which reads argv again for itself, the
   peeled copy is used only by capture and clap still parses `--root` for the subcommands.
7. Argv is read as OS strings. A non-UTF-8 argument in a capture sentence is reported as an error
   and exit 1, never a panic (hi: CAPTURE-1.c); on the non-capture path clap's `parse()` reads
   `args_os` itself and reports the same kind of argument as a usage error. A non-UTF-8 `--root`
   value survives the separated form `--root PATH`, because `peel_root` turns the following
   argument straight into a `PathBuf` without going through `str`. The joined form `--root=PATH`
   is different: that arm matches on `to_str()`, so a token that is not valid UTF-8 is not peeled
   at all and is left for clap, which parses it for a subcommand but cannot help a capture.
8. `main` performs no file I/O, no formatting of criteria and no validation of its own. It only
   routes, prints, and maps results onto exit codes. Its only calls into the environment are
   `args_os`, to read argv, and `current_dir`, and the latter only when `--root` was not given.

16. Every verb that performs a read-modify-write holds `lock::acquire` over the whole of it, and
    everything it does inside is inside, `out::refresh_index` included. The lock is not
    reentrant, so nothing under it may take one of its own (hi: FILE-19, INDEX-4).
    There are three: capture, `Retire`, and `Index` through `run_index`. `Index` was the one left
    out, and it is the only path that may *install* a block: unlocked, it reads `INTENT.md`, and a
    capture that finishes before it writes is undone, with both commands printing success
    (hi: INDEX-5). `View` takes no lock, decided rather than overlooked: it never reads the page it
    writes, it regenerates the whole of it from the criteria, its output is derived and gitignored,
    and `lock::acquire` creates `hi/` to live in, which a read verb has no business doing.
    **Both reload the workspace under the lock.** The `Workspace` each arm starts from was read
    before the lock was granted, so another writer may have finished in between; writing from that
    snapshot puts the file back the way it was. `run_capture` has always called `Workspace::find`
    a second time. The `Retire` arm did not, and two concurrent retires therefore both printed
    `retired`, both exited 0, and left one of the two criteria live again with `hi check` reporting
    nothing wrong. It now calls `Workspace::find(&start)` inside the lock and resolves the id from
    that (hi: RETIRE-7, FILE-19, DECISIONS.md §33).
17. stdout is the record of what landed and stderr is everything else. `<file>  +<id>`,
    `<file>  created` and `<file>  <id> retired` go to stdout; the note about a feature list that
    could not be refreshed goes to stderr, because a capture that stored its criterion succeeded
    and nothing parsing stdout should see a line about a file it did not ask about
    (hi: CAPTURE-11, INDEX-4.a).

## Behavioral Examples

#### Scenario: Capture is the default action

- **Given** a repository containing `hi/chat.md` that declares family `SEND`
- **When** the user runs `hi SEND-2 "it reaches them"`
- **Then** argv[1] is id-shaped, capture runs before clap sees anything, and stdout reads
  `hi/chat.md  +SEND-2`

#### Scenario: A new family also prints that a file was started

- **Given** no file declares family `BILLING`
- **When** the user runs `hi BILLING-1 "I can see what I paid"`
- **Then** stdout carries two lines: `hi/billing.md  created` then `hi/billing.md  +BILLING-1`

#### Scenario: `--root` is a flag, not part of the sentence

- **Given** a repository at `/tmp/repo` containing `hi/chat.md`, and a shell sitting somewhere else
- **When** the user runs `hi --root /tmp/repo SEND-2 "it reaches them"`, or the same with
  `--root=/tmp/repo`
- **Then** `peel_root` removes the flag and its value, the remaining first argument `SEND-2` still
  routes to capture, the workspace is found from `/tmp/repo`, and the written line is
  `- **SEND-2**  it reaches them` with no `--root` anywhere in it

#### Scenario: after the id, a flag-shaped word is a word

- **Given** a repository containing `hi/chat.md` that declares family `SEND`
- **When** the user runs `hi SEND-2 the --root docs option should be documented`
- **Then** the peel has already stopped at `SEND-2`, so nothing is taken out of the sentence, and
  the written line is `- **SEND-2**  the --root docs option should be documented` (hi: CAPTURE-8)

#### Scenario: A wrongly cased id is refused rather than read as a subcommand

- **Given** any repository
- **When** the user runs `hi send-2 "it reaches them"`
- **Then** `looks_like_id` still recognizes it, capture runs, and stderr reads
  `error: 'send-2' is not a valid id: family 'send' must start with A-Z and contain only A-Z, 0-9, _`
  with exit 1 and nothing written (hi: CHECK-2.d)

#### Scenario: An argument that is not valid UTF-8

- **Given** any repository
- **When** the user runs `hi SEND-2 $'caf\xe9 works offline'`
- **Then** stderr reads
  `error: that sentence is not valid UTF-8. hi files are text, so it cannot be stored as written`,
  the exit code is 1, and no panic or backtrace is printed

#### Scenario: Bare invocation

- **Given** any directory
- **When** the user runs `hi`
- **Then** clap prints help because of `arg_required_else_help`, and nothing is read or written

#### Scenario: A clean check

- **Given** a workspace with no structural problems
- **When** the user runs `hi check`
- **Then** stdout is a summary like `89 criteria · 8 families · 5 files`, which is what hi's own
  repository reports today, and the exit code is 0

#### Scenario: A structural problem fails

- **Given** a workspace where `SEND-1.a` has no parent
- **When** the user runs `hi check`
- **Then** the file is named, then `  <line>:orphan-case  SEND-1.a has no parent SEND-1`, then the
  summary and a problem count, and the exit code is 1

#### Scenario: A refusal

- **Given** `SEND-1` already exists
- **When** the user runs `hi SEND-1 "something else"`
- **Then** stderr carries `error: SEND-1 already exists in hi/chat.md:9` and the next-free hint,
  stdout is empty, and the exit code is 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No `hi/` directory holding a hi file, and no `.git`, anywhere from the start directory up to the filesystem root | `error: this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so run it inside one`, exit 1. A `hi/` directory that holds no file with `hi:` frontmatter does not count, so a Hindi locale directory is walked past rather than adopted (hi: CAPTURE-6). `Workspace::find` tests both conditions at every level on the way up, so the nearest `.git` stops the walk and an outer repository's criteria are never adopted |
| A capture argument that is not valid UTF-8 | `error: that sentence is not valid UTF-8. hi files are text, so it cannot be stored as written`, exit 1, no panic (hi: CAPTURE-1.c) |
| `--root` naming a path that does not exist | `error: resolving <path>: No such file or directory (os error 2)`, from the `canonicalize` at the top of `Workspace::find`, exit 1. Same on the capture path and the subcommand path |
| Capture given an id that already exists | Error naming the file and line, plus the next free id, exit 1 |
| Capture given a zero-padded number, such as `SEND-007` | Error from `Id::parse` about the leading zero, exit 1, nothing written (hi: ID-1.c) |
| Capture given a malformed id that is still id-shaped, such as `send-2` or `SEND-1.a.b` | `error: '<id>' is not a valid id: <reason>`, exit 1 |
| A first argument that is not id-shaped, such as `SEND-a` or `SEND-`, because the character after the `-` is not a digit | Never routed to capture; clap reports it as an unrecognized subcommand, exit 2 |
| `hi --root SEND-2 "<sentence>"`, a `--root` with its value omitted ahead of an id | `peel_root` takes `SEND-2` as the root, the tail no longer begins with an id, and clap reports `<sentence>` as an unrecognized subcommand, exit 2. Nothing is written |
| Capture given no sentence | `error: a criterion needs a sentence. Say what you actually want`, exit 1 |
| `hi check` finds any structural problem | Problems printed to stdout, exit 1 |
| `hi issue` given an id that does not exist, or a retired one | Error, exit 1 |
| `hi export` given a scope matching no family, file or id | `error: nothing matches '<scope>'. Give a family like SEND, a file like chat, an id like SEND-1, or nothing at all for the whole repository`, exit 1. A family, a bare stem (`chat`), a file name (`chat.md`) and the repo-relative path (`hi/chat.md`) all match |
| `hi index` where `INTENT.md` opens a `hi:index` marker and never closes it | Error from `out::write_index`, exit 1, `INTENT.md` untouched (hi: INDEX-2.b) |
| `hi index` where `INTENT.md` cannot be read for any reason but absence | Error from `out::write_index`, `reading <path>` wrapping the I/O error, exit 1, `INTENT.md` untouched (hi: INDEX-2.c) |
| The same broken `INTENT.md` during a capture or a `hi retire` | Not an error. `out::refresh_index` hands the message back, `main` prints `note: the feature list in INTENT.md was not refreshed: <text>` on stderr, and the exit code is 0 because the criterion is already on disk (hi: INDEX-4.a) |
| `gh` missing or failing during `hi issue --create` | Error with context about the GitHub CLI, exit 1 |
| Unknown subcommand or bad flag | clap usage error, exit 2 |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `clap` (derive) | `Parser`, `Subcommand`, argument parsing, help, version |
| `anyhow` | `Result` and error formatting via `{:#}` |
| `serde_json` | `to_string_pretty` for `hi check --json` |
| `crate::id` | `looks_like_id`, for the capture routing decision |
| `crate::doc` | Nothing is called. `main.rs` declares `mod doc;` because the module tree is rooted here, and every other module reaches it through the crate |
| `crate::workspace` | `Workspace::find`, for every verb |
| `crate::capture` | `capture` |
| `crate::check` | `run`, `Report` |
| `crate::out` | `ls`, `issue`, `export`, `write_index` (with `out::Absent::Install`, because `hi index` was typed to install a section; the refresh after a capture or a retire passes `LeaveAlone`), `refresh_index` |
| `crate::view` | `write` |

### Consumed By

| Module | What is used |
|--------|-------------|
| None. | This is the binary entry point; nothing in the crate depends on it. |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Claude | Reconciled with the bug-fix pass: added `peel_root`, the new `run_capture` signature taking a root, and `args_os` reading. Invariant 6 inverted (`--root` now applies to capture too), and a new invariant 7 covers non-UTF-8 argv. Added scenarios and error rows for `--root`, non-UTF-8 arguments, the `holds_hi_files` workspace rule, padded ids, and an unclosed index marker. |
| 2026-09-16 | Claude | Verification pass. Corrected three claims that the code does not make: an unpeeled `--root` is only a clap usage error off the capture path (after an id it lands in the sentence), a non-UTF-8 `--root` value survives only the separated `--root PATH` form, and `args_os` is an environment call invariant 8 did not name. Added an error row for a `--root` that does not resolve, and `specs/id/id.spec.md` to `depends_on`, which the Consumes table already listed. |
| 2026-09-16 | Claude | Reconciled with the routing and discovery changes, every claim re-checked against `./target/release/hi`. `peel_root` now consumes `--root` only while it leads and stops at the id, so the "scans the whole slice" claim and everything built on it was rewritten. `looks_like_id` is case-insensitive on the family and requires a digit after the hyphen, so invariant 1 and the Purpose no longer claim an uppercase-initial family; `SEND-a` is now a clap usage error rather than an id error, and `send-2` routes to capture and is refused with a reason. Replaced the stale not-found, empty-sentence and export-scope messages with the strings the binary prints. Added `crate::doc` to Consumes and `depends_on`, three behavioral scenarios, and three error rows. |
| 2026-09-17 | Claude | The `Retire` arm calls `out::refresh_index` after `Doc::save`, so retiring a criterion leaves the generated feature list true the way capturing one now does (hi: INDEX-4, DECISIONS.md §30). The failure is printed on stderr as a note and never changes the exit code (hi: INDEX-4.a). Added invariants 16 and 17 and an error row. This pass also added `Retire { id, reason }` to the `Command` enum row, which the subcommand has had since 0.4.0 and this spec had never listed. |
| 2026-09-17 | Claude | The `Retire` arm reloads the workspace under the lock instead of writing from the copy `run` read before it. Two concurrent retires used to both report success and leave one criterion live again, because the second wrote a file it had read before the first one landed (hi: RETIRE-7, FILE-19, DECISIONS.md §33). Extended invariant 16. |
| 2026-09-18 | Claude | The `Index` arm takes the write lock, through a new private `run_index`, and reloads the workspace under it. It was the one read-modify-write outside the lock and the only path that can install a block: unlocked, a capture finishing between its read of `INTENT.md` and its write is undone, with both commands printing success. `hi view` deliberately takes none — it never reads the page it writes, its output is derived and gitignored, and `lock::acquire` creates `hi/`, which a read verb has no business doing. Extended invariant 16 and added REQ-main-009 (hi: INDEX-5, FILE-19, DECISIONS.md §38). |
| 2026-09-18 | Claude | `Command::Seed` takes the write lock, reloads, and dispatches to `capture::seed_agent_files`. Added REQ-main-010 (hi: HABIT-6, DECISIONS.md §39). |
