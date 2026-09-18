---
spec: main.spec.md
---

## User Stories

- As someone mid-thought, I want writing a criterion to be one command with no subcommand to remember, so that capture is a reflex (hi: CAPTURE-1)
- As someone running hi in CI, I want an exit code I can rely on, so that a broken file fails and unfinished intent does not (hi: CHECK-1, CHECK-2)
- As someone reading a failure, I want the file and line named, so that I can go straight to it (hi: CHECK-3)
- As an agent, I want `--json` on check and a clean stdout on export, so that I can consume hi's output without parsing prose
- As someone who types a flag before the id, I want it treated as a flag, so that `--root` works even though capture never reaches clap; and as someone who types one inside a sentence, I want it left alone, so that my words survive (hi: CAPTURE-8)
- As someone who types something hi cannot handle, I want a plain sentence back rather than a stack trace (hi: CAPTURE-1.c)
- As someone refreshing the list at the front of my product while an agent is capturing into the same repository, I want neither command to undo the other (hi: INDEX-5)

## Acceptance Criteria

### REQ-main-001

The entry point SHALL treat an id-shaped first argument as a capture, routing it before clap parses arguments (hi: CAPTURE-1).

Acceptance Criteria

- `hi SEND-2 "it reaches them"` captures and prints `hi/chat.md  +SEND-2`.
- The remaining arguments are joined with single spaces to form the sentence.
- No flag or subcommand name is id-shaped, so `--help`, `-V`, `check`, `ls`, `issue`, `export`, `index` and `view` all reach clap unchanged. A flag splits to an empty family part, and no subcommand name holds a hyphen.
- An ordinary hyphenated word is not id-shaped, because the character after the `-` must be a digit, so `spec-sync` and `well-formed` reach clap rather than capture.
- A wrongly cased id is id-shaped, because the family test ignores case, so `hi send-2 "<sentence>"` routes to capture and is refused there with a reason rather than read as a subcommand (hi: CHECK-2.d).
- The routing test runs on the arguments left after a leading `--root` has been peeled, so `hi --root PATH SEND-2 "<sentence>"` still captures.

### REQ-main-002

The entry point SHALL print help and touch nothing when invoked with no arguments.

Acceptance Criteria

- Bare `hi` prints the help text via clap's `arg_required_else_help`.
- No file is read, created, or written.
- The exit code is 2, which is clap's own code for "a required argument is missing". Nothing in `main` chooses it, and `bare_invocation_prints_help_and_writes_nothing` asserts on the help text and the untouched `hi/`, not on the code.

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

The entry point SHALL remove a leading `--root` from the arguments before routing, so that it is honored by capture and never becomes part of a criterion's sentence, and SHALL leave everything from the id onward untouched, so that a flag-shaped word in a sentence stays a word (hi: CAPTURE-8).

Acceptance Criteria

- Both `--root PATH` and `--root=PATH` are recognized when they come before the id.
- `hi --root PATH SEND-2 "it reaches them"` writes `- **SEND-2**  it reaches them` into `PATH`'s workspace, and the written line contains no `--root`.
- Capture resolves its workspace from that root when one was given, and from the current directory otherwise.
- The peel stops at the first argument that is not a `--root` form, so `hi SEND-2 the --root docs option should be documented` captures the whole sentence, `--root docs` included.
- Among leading occurrences the last wins.
- A leading `--root` with nothing after it is not peeled, because the arm guards on `index + 1 < args.len()`. It then leads the tail, which is therefore not id-shaped, so clap reports the missing value as a usage error, exit 2.
- A `--root` typed after the id is never honored, because the peel has already stopped: `hi SEND-2 "text" --root /elsewhere` captures `text --root /elsewhere` into the current workspace and exits 0.
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

### REQ-main-009

`hi index` SHALL hold the write lock across its read-modify-write, and `hi view` SHALL not take one
(hi: INDEX-5, FILE-19).

Acceptance Criteria

- The `Index` arm calls `run_index`, which takes `lock::acquire` on `<root>/hi`, reloads the
  workspace under it with `Workspace::find(&start)`, and only then calls
  `out::write_index(.., Absent::Install)`. The reload is for the reason `run_capture` and the
  `Retire` arm reload: the workspace the arm started from was read before the lock was granted.
- `hi index` is a read-modify-write — read `INTENT.md`, splice the generated block into it, write the
  rest back — and it is the one path that may *install* a block. Unlocked, a capture that finished
  between the read and the write is undone: the list goes back without that criterion in it, any
  prose saved in between goes with it, and both commands print success.
- `hi view` takes no lock, decided rather than overlooked. It never reads the page it is about to
  write; it regenerates the whole file from the criteria, so there is no window in which it could
  put back a stale version of somebody else's work, and `fs::write` of a fully rendered page is not
  a read-modify-write. Three further reasons: `lock::acquire` creates `hi/` to live in, so a read
  verb would start writing into the repository; the page is derived and gitignored, so the worst a
  race can do is publish a page one criterion out of date, which the next run fixes and which no id
  depends on; and `hi view` in CI would queue behind a bulk capture for nothing.
- The lock is not reentrant, so nothing under `run_index` may take one of its own. `write_index`
  does not, and `refresh_index` — which the two writing verbs call inside their own lock — must not
  either (REQ-out-017).
- The read verbs `ls`, `issue`, `export` and `check` take no lock and write nothing, so there is
  nothing for one to protect.

### REQ-main-010

`hi seed` SHALL hold the write lock, reload under it, and dispatch to `capture::seed_agent_files`
(hi: HABIT-6, FILE-19).

Acceptance Criteria

- The `Seed` arm takes `lock::acquire` on `<root>/hi`, reloads with `Workspace::find`, then calls
  `capture::seed_agent_files`. The lock is the same one capture, retire and `hi index` take, so a
  seed cannot interleave with a capture rewriting the same file.
- Missing → print each created path. Prior template → print `updated to the current instruction`.
  Already current → print `already current`. Edited → the error from `seed_agent_files`, exit 1,
  nothing written (REQ-capture-019).
- A file declaring an unknown `hi:` version is refused by `Workspace::find` before the lock, like
  every other verb (REQ-workspace-013). `tests/cli.rs` includes `seed` in the future-format verb list.


## Constraints

- Capture must stay the default action; requiring a subcommand to write a criterion would cost the reflex that the whole design is built around.
- The routing pre-parse must never shadow a flag or a subcommand. What guarantees that is `looks_like_id` demanding a non-empty family before the hyphen and a digit after it: a flag has no family, and no subcommand name here holds a hyphen.
- `peel_root` reads only the leading flags, so `--root` is honored only ahead of the id. That is the cost of leaving a person's sentence exactly as typed on a path that never reaches clap.
- No verb may prompt, open an editor, or read stdin.
- No verb opens a network connection except `hi issue --create`, which shells out to `gh` only when explicitly asked.

## Out of Scope

- Parsing, validating, rendering and writing hi files, which `doc`, `id`, `check`, `capture`, `out` and `view` own.
- Shell completions and man pages.
- Any configuration file. hi has none.
