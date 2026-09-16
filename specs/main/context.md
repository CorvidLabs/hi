---
spec: main.spec.md
---

## Decisions

- **Capture is the default action, implemented as a pre-parse of argv.** clap cannot express "any
  unrecognized first token that looks like an id is an argument to the default subcommand" without
  `allow_external_subcommands`, which would also swallow genuine typos of real subcommands. The
  explicit test is four lines, `tail.first().and_then(|arg| arg.to_str()).is_some_and(id::looks_like_id)`,
  run on the tail `peel_root` returns rather than on `args[1]` directly. Its one surprise is that a
  token the predicate rejects never reaches capture at all, so `hi SEND-a "<sentence>"` is a clap
  usage error and not an id error.
- **`arg_required_else_help` rather than a dashboard on bare `hi`.** Decided in the design
  interview: bare invocation prints help. A status dashboard would need criteria mapped to paths,
  which nothing in this design does.
- **Exit 1 for both a structural problem and a refusal.** hi never gates on intent, but a file it
  cannot parse and a write it refused are both real failures a script should notice.
- **`--root` is peeled out of argv by hand, before routing.** Capture never reaches clap, so
  without `peel_root` the flag and its value would be joined into the criterion's sentence. Do not
  "simplify" this by moving the peel after the `looks_like_id` test: the test runs on the peeled
  tail so that `hi --root PATH SEND-2 "<sentence>"` still routes to capture.
- **The peel consumes only a *leading* `--root`, and stops at the first argument that is not
  one.** Options come before the id; from the id onward every token belongs to the person. An
  earlier version scanned the whole argument vector, which quietly ate two words out of
  `hi DOC-2 the --root docs option should be documented`. The `break` in the loop is the fix, and
  `a_flag_looking_word_inside_a_sentence_stays_a_word` pins it. The cost is the opposite mistake:
  a `--root` typed after the id is stored as prose instead of honored, and nothing warns.
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

- `looks_like_id` is the contract between this module and `id`. It accepts a lowercase-initial
  family on purpose, so `hi send-2 "<sentence>"` is refused with a reason rather than read as a
  subcommand (hi: CHECK-2.d). What keeps it from shadowing anything is the other half of the rule:
  the character after the `-` must be a digit, and no subcommand name here holds a hyphen at all.
  If that digit rule ever loosens, the routing would start swallowing ordinary hyphenated words,
  and any future hyphenated subcommand would have to avoid a digit in that position.
- `--root` is global on `Cli` *and* applies to capture, through `peel_root`. The two paths must
  stay in agreement: if a second global flag is ever added, capture will not see it unless it is
  peeled here too, and it will only be honored ahead of the id.
- `peel_root` reads only the leading flags, so a `--root` typed after the id is part of the
  sentence and is never honored. Accepted cost, and the safer half of the trade: losing two words
  out of someone's criterion is worse than ignoring a misplaced flag.
- `peel_root` matches on `to_str()`, so its two forms are not symmetric about encoding. `--root
  PATH` peels whatever OS string follows, valid UTF-8 or not, and hands it to `PathBuf`.
  `--root=PATH` has to read the whole token as `str` to split it, so a value that is not valid
  UTF-8 leaves the flag unpeeled and it falls through to clap.
- The `index + 1 < args.len()` guard means a leading `--root` with nothing after it is never
  peeled; it stays at the front of the tail, which is therefore not id-shaped, and clap reports
  the missing value as a usage error. The guard also means `hi --root SEND-2 "<sentence>"` peels
  the *id* as the root value, after which clap reports the sentence as an unrecognized subcommand.
  Both exit 2 and write nothing.
- `args[1..]` assumes argv carries a program name. Every normal launch does.

## Files the next agent needs

- `src/main.rs` holds routing, `peel_root`, dispatch, and report rendering.
- `src/id.rs` defines `looks_like_id`, which decides the route.
- `src/workspace.rs` has `Workspace::find`, the upward search. It no longer accepts any directory
  named `hi/`: `holds_hi_files` requires one to actually contain a file with `hi:` frontmatter,
  because `hi` is the ISO code for Hindi and `public/locales/hi/` is a real directory. At each
  level it asks two questions in order, a qualifying `hi/` first and then a `.git`, so the walk
  stops at the *nearest* repository. That is what keeps the first capture in a repository a single
  command, and what stops a project nested inside another from adopting the outer project's
  criteria. When neither ever answers, the search reaches the filesystem root and fails with
  `this is not a repository, and no hi/ directory was found above it`.
- `src/check.rs` defines `Report` and `Kind::code()`, which `print_report` renders. `Kind` has six
  variants now; `stray-criterion` was added in the same pass. `print_report` never matches on
  `Kind`, so it needed no change and must not grow a match.
- `tests/cli.rs` carries the only tests this module has.
