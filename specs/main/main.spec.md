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
argument is routed to `capture` **before clap parses anything** (`hi SEND-2 "<sentence>"`). Flags
never look like ids, because `looks_like_id` requires an uppercase-initial family and a hyphen,
so `--help`, `-V` and every subcommand name pass through to clap untouched. This is what makes
writing a criterion one command with no subcommand to remember (hi: CAPTURE-1).

Two details of that pre-parse matter. `--root` is peeled out of argv by `peel_root` *before* the
routing test, so the flag reaches capture as a root rather than being joined into the criterion's
sentence (hi: CAPTURE-8). And argv is read as `args_os`, because `env::args()` panics on a
non-UTF-8 argument; a sentence that is not valid UTF-8 becomes a plain error instead of a
backtrace (hi: CAPTURE-1.c).

## Public API

| Export | Description |
|--------|-------------|
| None. | A binary crate. Every item in this file is private; the entry point is `fn main`. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Cli` (private) | clap `Parser` for the non-capture surface: a global `--root` and one required subcommand. `arg_required_else_help` makes bare `hi` print help. |
| `Command` (private) | clap `Subcommand` enum: `Check { json }`, `Ls { family, retired }`, `Issue { id, create, repo }`, `Export { scope }`, `Index`, `View { out }`. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines no traits. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `main` | `fn main() -> ExitCode` | Collects `std::env::args_os()`, peels `--root` with `peel_root`, and routes an id-shaped first remaining argument to capture; otherwise parses `Cli` and dispatches. Converts an error into a printed message and exit 1. |
| `peel_root` (private) | `fn peel_root(args: &[OsString]) -> (Option<PathBuf>, Vec<OsString>)` | Take an optional `--root PATH` or `--root=PATH` out of the arguments, returning the path and everything else in order. Scans the whole slice, and the last occurrence wins. Two shapes fall through rather than being peeled: `--root` with nothing after it, because the arm guards on `index + 1 < args.len()`; and `--root=PATH` whose token is not valid UTF-8, because that arm tests the whole token with `to_str()`. An unpeeled `--root` reaches whichever path the routing test then picks: clap reports it as a usage error, but if an id came first it stays in the captured sentence. |
| `fail` (private) | `fn fail(err: anyhow::Error) -> ExitCode` | Print `error: {err:#}` to stderr and return exit 1. The `:#` carries the whole anyhow context chain. |
| `run_capture` (private) | `fn run_capture(root: Option<&Path>, raw_id: &str, rest: &[String]) -> Result<()>` | Join the remaining argv into a sentence, find the workspace from `root` or the current directory, capture, and print the destination and id. |
| `run` (private) | `fn run(cli: Cli) -> Result<ExitCode>` | Resolve the workspace from `--root` or the current directory, then dispatch one subcommand. |
| `print_report` (private) | `fn print_report(report: &check::Report)` | Render a `check::Report` for a terminal: problems grouped by file, then a one-line summary. |
| `plural` (private) | `fn plural(count: usize, one: &str, many: &str) -> String` | `"1 criterion"` / `"12 criteria"`. |

## Invariants

1. An id-shaped first argument (first *after* `--root` has been peeled) always routes to capture,
   and is never interpreted as a subcommand. `looks_like_id` requires an uppercase-initial family
   followed by `-` and a non-empty remainder, so no flag and no defined subcommand name can collide
   with it.
2. Bare `hi` with no arguments prints help and does not touch the filesystem: nothing is asked of
   the person, and nothing is written (hi: CAPTURE-1.b).
3. Exit codes: `0` success, `1` any returned error (a structural problem from `check`, or a refusal
   from capture), and `2` from clap for a usage error.
4. `hi check` exits 1 when and only when `Report::ok()` is false. Every other verb exits 0 on
   success (hi: CHECK-1, CHECK-2).
5. Errors print to stderr; all normal output prints to stdout, so `hi export <scope> > file` is clean.
6. `--root` is global and affects every verb, capture included. `peel_root` removes it from argv
   before the routing test, so a well-formed `--root PATH` or `--root=PATH` is not joined into a
   criterion's sentence (hi: CAPTURE-8). A `--root` the peel does not recognize still can be. See
   the `peel_root` row above for the two shapes.
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
  `SEND-2  it reaches them` with no `--root` anywhere in it

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
- **Then** stdout is a summary like `53 criteria · 8 families · 5 files` and the exit code is 0

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
| No `hi/` directory holding a hi file, and no `.git`, above the start directory | `error: no hi/ directory found. Run this inside a repository`, exit 1. A `hi/` directory that holds no file with `hi:` frontmatter does not count, so a Hindi locale directory is walked past rather than adopted (hi: CAPTURE-6) |
| A capture argument that is not valid UTF-8 | `error: that sentence is not valid UTF-8. hi files are text, so it cannot be stored as written`, exit 1, no panic (hi: CAPTURE-1.c) |
| `--root` naming a path that does not exist | `error: resolving <path>: No such file or directory (os error 2)`, from the `canonicalize` at the top of `Workspace::find`, exit 1. Same on the capture path and the subcommand path |
| Capture given an id that already exists | Error naming the file and line, plus the next free id, exit 1 |
| Capture given a zero-padded number, such as `SEND-007` | Error from `Id::parse` about the leading zero, exit 1, nothing written (hi: ID-1.c) |
| Capture given a malformed id | `error: '<id>' is not a valid id: <reason>`, exit 1 |
| Capture given no sentence | `error: a criterion needs a sentence, so say what you actually want`, exit 1 |
| `hi check` finds any structural problem | Problems printed to stdout, exit 1 |
| `hi issue` given an id that does not exist, or a retired one | Error, exit 1 |
| `hi export` given a scope matching no family or file | Error beginning `nothing matches '<scope>'`, exit 1 |
| `hi index` where `INTENT.md` opens a `hi:index` marker and never closes it | Error from `out::write_index`, exit 1, `INTENT.md` untouched (hi: INDEX-2.b) |
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
| `crate::workspace` | `Workspace::find`, for every verb |
| `crate::capture` | `capture` |
| `crate::check` | `run`, `Report` |
| `crate::out` | `ls`, `issue`, `export`, `write_index` |
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
