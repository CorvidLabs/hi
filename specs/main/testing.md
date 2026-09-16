---
spec: main.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `tests/cli.rs` | Integration | Drives the built binary in a throwaway repository: argv routing, exit codes, stream discipline, and every verb's happy path. |
| `src/id.rs` (`mod tests`) | Unit | `looks_like_id`, the predicate the capture route depends on. |
| `src/capture.rs` (`mod tests`) | Unit | The refusal paths that produce exit 1. |

`src/main.rs` has no `#[cfg(test)]` module. Everything it owns (argv routing, `peel_root`, exit
codes, stream discipline) needs a real process, so it is covered from `tests/cli.rs` only.

| Requirement | Covered by | Notes |
|---|---|---|
| REQ-main-001 | `an_id_shaped_argument_captures`, `a_new_family_starts_its_own_file`, `a_long_sentence_stays_on_one_line`, `id::tests::recognises_id_shaped_tokens` | Routing to capture, the joined sentence, and the predicate that decides the route. |
| REQ-main-002 | `bare_invocation_prints_help_and_writes_nothing` | Asserts help is printed and that `hi/` is still empty afterwards. |
| REQ-main-003 | `an_existing_id_refuses_with_exit_1_and_writes_nothing`, `a_malformed_id_refuses_without_writing`, `a_zero_padded_id_is_refused_rather_than_silently_renamed`, `check_is_clean_on_unfinished_intent`, `check_fails_on_a_structural_problem`, `check_fails_on_a_duplicate_id`, `an_unknown_subcommand_is_a_usage_error`, `export_rejects_a_scope_that_matches_nothing`, `index_refuses_rather_than_guessing_when_a_marker_is_unclosed`, `outside_a_repository_it_says_so_rather_than_guessing` | All three exit codes: 0 clean, 1 on a refusal or a structural problem, 2 on a usage error. Each refusal also asserts the file came back unchanged. |
| REQ-main-004 | `an_existing_id_refuses_with_exit_1_and_writes_nothing`, `export_stdout_is_parseable_json` | Asserts a refusal prints nothing to stdout and that `hi export` stdout parses as JSON. |
| REQ-main-005 | `check_fails_on_a_structural_problem` | Asserts the file, the problem code, and the line number all appear. |
| REQ-main-006 | No test | Structural; enforced by review, and visible in that `main.rs` imports only module entry points. |
| REQ-main-007 | `root_is_honored_by_capture_and_not_swallowed_into_the_sentence` | Runs capture from an unrelated directory in both `--root PATH` and `--root=PATH` form, then asserts both sentences landed in the target workspace and that no `--root` reached the file. |
| REQ-main-008 | `a_non_utf8_argument_is_reported_not_panicked` | Unix-only (`#[cfg(unix)]`, it builds the argument from raw bytes). Asserts exit 1 rather than 101, no `panicked` in stderr, and the `not valid UTF-8` message. |

## Manual Testing

- [x] Capture hi's own 53 criteria through the built binary, which is how the routing and the
      one-line renderer were first exercised in anger.
- [x] `hi check` against the real repository, clean and with an injected structural problem.
- [ ] Confirm behavior on Windows, where `fs::canonicalize` returns a `\\?\`-prefixed path that
      `Workspace::rel` has never been exercised against.

## Edge Cases & Boundary Conditions

| Case | Expected |
|---|---|
| Bare `hi` | Help, nothing read or written |
| `hi --help`, `hi -V` | clap handles them; no flag is id-shaped so routing never intercepts |
| An id with no sentence | `a criterion needs a sentence`, exit 1 |
| An unknown subcommand | clap usage error, exit 2 |
| Run from outside any repository | `no hi/ directory found`, exit 1 |
| `hi export > file` | File contains only JSON |
| `hi --root PATH SEND-2 "<sentence>"` and `hi --root=PATH SEND-2 "<sentence>"` | Both capture into `PATH`'s workspace; neither writes `--root` into the sentence |
| A `hi/` directory holding no hi file, such as a Hindi locale directory | Walked past; the search continues upward and falls back to the repository root |
| A capture argument that is not valid UTF-8 | `not valid UTF-8`, exit 1, no panic |
| A `--root PATH` value that is not valid UTF-8 | Used as a path; the following argument never goes through `str` |
| `--root=PATH` where the whole token is not valid UTF-8 | Not peeled, because that arm matches on `to_str()`, so it falls through to clap: fine for a subcommand, a usage error ahead of a capture |
| `--root` given twice | The last one wins |
| `hi --root` with nothing after it | Not peeled; clap reports the missing value as a usage error, exit 2 |
| A trailing `--root` after a sentence, as in `hi SEND-2 "text" --root` | Not peeled either, and here nothing catches it: the sentence captured is `text --root`, exit 0 |
| `--root` naming a path that does not exist | `error: resolving <path>` plus the OS error, from `Workspace::find`, exit 1, on both paths |

## Gaps

- Windows is untested. `Workspace::find` canonicalizes, and on Windows that yields a `\\?\` prefix
  that `rel` strips only if the root carries the same prefix. CI builds on Windows but the
  integration tests have never been run there against a nested path.
- `print_report`'s plural forms are asserted only for `1 criterion`; the family and file plurals
  are unverified.
- `--root` is asserted only on the capture path. No test runs `hi --root PATH check` or any other
  subcommand with the flag, so the claim that clap still parses it for the subcommands rests on
  reading `main`, not on a test.
- `peel_root` has no unit test, because `main.rs` has no test module. Its behavior when `--root`
  appears twice, when its value is missing, and when a sentence happens to contain the literal
  token `--root` is untested. The last of those is the one with a visible consequence:
  `hi SEND-2 "text" --root` captures `text --root` and exits 0, because the peel guards on
  `index + 1 < args.len()` and the capture path has no clap to catch what is left over.
- `a_non_utf8_argument_is_reported_not_panicked` is `#[cfg(unix)]`. The same path on Windows,
  where an invalid argument is ill-formed UTF-16 rather than an invalid byte, is untested.
- No test asserts that a non-UTF-8 `--root` value works, and the two forms do not behave alike:
  `--root PATH` carries the bytes through to `canonicalize`, while `--root=PATH` is not recognized
  at all. Nothing pins either.
- No test asserts that a `--root` naming a path that does not exist errors rather than silently
  falling back to the current directory.
