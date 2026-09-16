---
spec: main.spec.md
---

## User Stories

- As someone mid-thought, I want writing a criterion to be one command with no subcommand to remember, so that capture is a reflex (hi: CAPTURE-1)
- As someone running hi in CI, I want an exit code I can rely on, so that a broken file fails and unfinished intent does not (hi: CHECK-1, CHECK-2)
- As someone reading a failure, I want the file and line named, so that I can go straight to it (hi: CHECK-3)
- As an agent, I want `--json` on check and a clean stdout on export, so that I can consume hi's output without parsing prose
- As someone who types a flag, I want it treated as a flag and not swallowed into my sentence, so that `--root` works even though capture never reaches clap (hi: CAPTURE-8)
- As someone who types something hi cannot handle, I want a plain sentence back rather than a stack trace (hi: CAPTURE-1.c)

## Acceptance Criteria

### REQ-main-001

The entry point SHALL treat an id-shaped first argument as a capture, routing it before clap parses arguments (hi: CAPTURE-1).

Acceptance Criteria

- `hi SEND-2 "it reaches them"` captures and prints `hi/chat.md  +SEND-2`.
- The remaining arguments are joined with single spaces to form the sentence.
- No flag or subcommand name is id-shaped, so `--help`, `-V`, `check`, `ls`, `issue`, `export`, `index` and `view` all reach clap unchanged.
- The routing test runs on the arguments left after `--root` has been peeled, so `hi --root PATH SEND-2 "<sentence>"` still captures.

### REQ-main-002

The entry point SHALL print help and touch nothing when invoked with no arguments.

Acceptance Criteria

- Bare `hi` prints the help text via clap's `arg_required_else_help`.
- No file is read, created, or written.

### REQ-main-003

The entry point SHALL exit 0 on success, 1 on any returned error, and 2 on a usage error.

Acceptance Criteria

- `hi check` exits 1 when and only when the report carries at least one problem (hi: CHECK-2).
- A capture refusal exits 1 and writes nothing (hi: CAPTURE-5).
- An unknown subcommand or bad flag exits 2, which is clap's own behavior.
- Every other successful verb exits 0.

### REQ-main-004

The entry point SHALL send errors to stderr and normal output to stdout.

Acceptance Criteria

- Lines that begin with `error:`, formatted with anyhow's `{:#}` so the whole context chain is shown, go to stderr.
- `hi export > file` produces a file containing only JSON.
- A refusal produces no stdout at all.

### REQ-main-005

The check report SHALL name the file and line of every structural problem, then summarize (hi: CHECK-3).

Acceptance Criteria

- Problems are grouped under their file, printed once per file.
- Each problem line carries the 1-based line number, the problem's kebab-case code, and the message.
- The summary counts criteria, families and files, adding retired only when there are any.
- Counts are singular or plural correctly: `1 criterion`, `12 criteria`, `1 family`, `8 families`.

### REQ-main-006

The entry point SHALL contain no domain logic, delegating every verb to its module.

Acceptance Criteria

- `main.rs` performs no file I/O of its own beyond resolving the current directory.
- It parses no hi file, validates no id, and renders no criterion.
- Adding a verb means adding a `Command` variant and one dispatch arm.

### REQ-main-007

The entry point SHALL remove `--root` from the arguments before routing, so that it is honored by capture and never becomes part of a criterion's sentence (hi: CAPTURE-8).

Acceptance Criteria

- Both `--root PATH` and `--root=PATH` are recognized, wherever they appear in the arguments.
- `hi --root PATH SEND-2 "it reaches them"` writes `SEND-2  it reaches them` into `PATH`'s workspace, and the written line contains no `--root`.
- Capture resolves its workspace from that root when one was given, and from the current directory otherwise.
- The last `--root` seen wins.
- A `--root` with nothing after it is not peeled, because the arm guards on `index + 1 < args.len()`. Off the capture path clap then reports it as a usage error; after an id it stays in the arguments and is joined into the sentence, so `hi SEND-2 "text" --root` captures `text --root` and exits 0.
- A `--root` naming a path that does not exist is an error from `Workspace::find`'s `canonicalize`, exit 1, on both paths.
- The non-capture path is unaffected: `Cli::parse()` reads argv for itself, so clap still parses `--root` for every subcommand.

### REQ-main-008

The entry point SHALL report a non-UTF-8 argument as an error rather than panicking (hi: CAPTURE-1.c).

Acceptance Criteria

- Argv is read with `std::env::args_os()`, which does not panic the way `std::env::args()` does.
- A capture argument that is not valid UTF-8 produces `error: that sentence is not valid UTF-8. hi files are text, so it cannot be stored as written` on stderr and exit 1.
- The exit code is 1, not 101, and stderr carries no panic message or backtrace.
- A `--root` value that is not valid UTF-8 is still usable in the separated form `--root PATH`, because `peel_root` turns the following argument into a `PathBuf` without going through `str`.
- The joined form `--root=PATH` is matched with `to_str()`, so a token that is not valid UTF-8 is not peeled and falls through to clap. That leaves it usable for a subcommand, but not for a capture.

## Constraints

- Capture must stay the default action; requiring a subcommand to write a criterion would cost the reflex that the whole design is built around.
- The routing pre-parse must never shadow a flag or a subcommand, which is why `looks_like_id` demands an uppercase family and a hyphen.
- `peel_root` scans the whole argument list rather than only the leading flags, so a criterion sentence containing the bare token `--root` followed by another word loses both. That is the cost of honoring `--root` on a path that never reaches clap.
- No verb may prompt, open an editor, or read stdin.
- No verb opens a network connection except `hi issue --create`, which shells out to `gh` only when explicitly asked.

## Out of Scope

- Parsing, validating, rendering and writing hi files, which `doc`, `id`, `check`, `capture`, `out` and `view` own.
- Shell completions and man pages.
- Any configuration file. hi has none.
