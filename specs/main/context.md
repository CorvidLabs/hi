---
spec: main.spec.md
---

## Decisions

- **Capture is the default action, implemented as a pre-parse of argv.** clap cannot express "any
  unrecognized first token that looks like an id is an argument to the default subcommand" without
  `allow_external_subcommands`, which would also swallow genuine typos of real subcommands. The
  explicit `looks_like_id(&args[1])` check is three lines and never surprises anyone.
- **`arg_required_else_help` rather than a dashboard on bare `hi`.** Decided in the design
  interview: bare invocation prints help. A status dashboard would need criteria mapped to paths,
  which nothing in this design does.
- **Exit 1 for both a structural problem and a refusal.** hi never gates on intent, but a file it
  cannot parse and a write it refused are both real failures a script should notice.
- **`--root` is peeled out of argv by hand, before routing.** Capture never reaches clap, so
  without `peel_root` the flag and its value would be joined into the criterion's sentence. Do not
  "simplify" this by moving the peel after the `looks_like_id` test: the test runs on the peeled
  tail so that `hi --root PATH SEND-2 "<sentence>"` still routes to capture.
- **The peeled `root` is used only by capture.** The non-capture path calls `Cli::parse()`, which
  reads argv again for itself, so `--root` is genuinely parsed twice. That is deliberate and
  cheap; do not try to hand the peeled tail to clap instead, because clap would then be parsing a
  different argv than the one it reports in usage errors.
- **Argv is read with `args_os`, not `args`.** `std::env::args()` panics on a non-UTF-8 argument.
  Reading OS strings lets a bad sentence become an `error: that sentence is not valid UTF-8` line
  and exit 1 instead of a backtrace and exit 101 (hi: CAPTURE-1.c). The UTF-8 conversion is done
  once, over the whole capture tail, with `collect()` into an `Option<Vec<String>>`: if any argument
  is bad the whole capture is refused before `Workspace::find` runs, so nothing is read or written.

## Constraints

- `looks_like_id` is the contract between this module and `id`. If its rules ever loosen to accept
  a lowercase-initial token, the routing here would start shadowing subcommands.
- `--root` is global on `Cli` *and* applies to capture, through `peel_root`. The two paths must
  stay in agreement: if a second global flag is ever added, capture will not see it unless it is
  peeled here too.
- `peel_root` scans the whole argument list, not just the leading flags, so a sentence containing
  the bare token `--root` followed by another word loses both. Accepted cost; `--root` is not a
  phrase anyone writes in a criterion.
- `peel_root` matches on `to_str()`, so its two forms are not symmetric about encoding. `--root
  PATH` peels whatever OS string follows, valid UTF-8 or not, and hands it to `PathBuf`.
  `--root=PATH` has to read the whole token as `str` to split it, so a value that is not valid
  UTF-8 leaves the flag unpeeled and it falls through to clap.
- The `index + 1 < args.len()` guard means a `--root` at the very end is never peeled. Off the
  capture path clap catches the missing value. On the capture path nothing does, so
  `hi SEND-2 "text" --root` writes `SEND-2  text --root`. If that is ever worth refusing, it has
  to be refused in `main`, because by the time `capture` sees the sentence it is just words.
- `args[1..]` assumes argv carries a program name. Every normal launch does.

## Files the next agent needs

- `src/main.rs` holds routing, `peel_root`, dispatch, and report rendering.
- `src/id.rs` defines `looks_like_id`, which decides the route.
- `src/workspace.rs` has `Workspace::find`, the upward search. It no longer accepts any directory
  named `hi/`: `holds_hi_files` requires one to actually contain a file with `hi:` frontmatter,
  because `hi` is the ISO code for Hindi and `public/locales/hi/` is a real directory. When
  nothing qualifies it falls back to the `.git` root rather than failing, which is what keeps the
  first capture in a repository a single command.
- `src/check.rs` defines `Report` and `Kind::code()`, which `print_report` renders. `Kind` has six
  variants now; `stray-criterion` was added in the same pass. `print_report` never matches on
  `Kind`, so it needed no change and must not grow a match.
- `tests/cli.rs` carries the only tests this module has.
