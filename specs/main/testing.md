---
spec: main.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `tests/cli.rs` | Integration | Drives the built binary in a throwaway repository: argv routing, exit codes, stream discipline, and every verb's happy path. It also pins the discovery `main` depends on through `Workspace::find`, in `outside_a_repository_it_says_so_rather_than_guessing`, `a_hindi_locale_directory_is_not_mistaken_for_a_workspace` and `a_repository_is_a_boundary_for_discovery`. |
| `tests/cli.rs` (concurrency) | Integration, many real processes | The two write verbs under contention: `concurrent_captures_into_a_repository_with_no_hi_directory_all_land` (32 captures into a repository with no `hi/` yet), `concurrent_captures_all_land` (8 into an existing one) and `concurrent_retires_never_bring_a_criterion_back` (two retires at once, neither of which may come back live). The last of these covers the reload inside the lock in the `Retire` arm; reverting that reload fails it on every run (hi: RETIRE-7, FILE-19). |
| `src/id.rs` (`mod tests`) | Unit | `looks_like_id`, the predicate the capture route depends on. |
| `src/capture.rs` (`mod tests`) | Unit | The refusal paths that produce exit 1. |
| `src/main.rs` (`index_will_not_write_while_another_writer_holds_the_lock`) | Unit | REQ-main-009. Takes the repository's write lock, then runs `run_index` on a thread. `INTENT.md` is unchanged three hundred milliseconds later — an unlocked index finishes in under one — and once the lock is dropped the thread completes and the file changes. The assertion is deliberately not "the two eventually agree": two writers racing agree often enough that a test on the result would pass while the bug was live. |

`src/main.rs` has one `#[cfg(test)]` test, and only because the thing it pins cannot be seen from
outside: proving `hi index` *waits* means holding the lock while it runs, and a second process
cannot hold the lock and observe the first one at the same time without a helper binary. Everything
else `main` owns (argv routing, `peel_root`, exit codes, stream discipline) needs a real process and
is covered from `tests/cli.rs` only.

| Requirement | Covered by | Notes |
|---|---|---|
| REQ-main-001 | `an_id_shaped_argument_captures`, `a_new_family_starts_its_own_file`, `a_long_sentence_stays_on_one_line`, `id::tests::recognises_id_shaped_tokens`, `id::tests::a_wrongly_cased_family_is_rejected_with_a_reason`, `a_wrongly_cased_id_is_reported_rather_than_read_as_prose`, `an_ordinary_hyphenated_word_is_not_mistaken_for_an_id` | Routing to capture, the joined sentence, and the predicate that decides the route. `recognises_id_shaped_tokens` pins both halves of the rule directly: `send-2` and `Send-3` are id-shaped, `spec-sync`, `well-formed` and `SEND-a` are not. The two `tests/cli.rs` cases exercise the same predicate through `hi check` rather than through argv, because that is where a wrongly cased id has a visible consequence. |
| REQ-main-002 | `bare_invocation_prints_help_and_writes_nothing` | Asserts help is printed and that `hi/` is still empty afterwards. It does not assert the exit code, which is clap's 2. |
| REQ-main-003 | `an_existing_id_refuses_with_exit_1_and_writes_nothing`, `a_malformed_id_refuses_without_writing`, `a_zero_padded_id_is_refused_rather_than_silently_renamed`, `check_is_clean_on_unfinished_intent`, `check_fails_on_a_structural_problem`, `check_fails_on_a_duplicate_id`, `an_unknown_subcommand_is_a_usage_error`, `export_rejects_a_scope_that_matches_nothing`, `index_refuses_rather_than_guessing_when_a_marker_is_unclosed`, `outside_a_repository_it_says_so_rather_than_guessing` | All three exit codes: 0 clean, 1 on a refusal or a structural problem, 2 on a usage error. Each refusal also asserts the file came back unchanged. |
| REQ-main-004 | `an_existing_id_refuses_with_exit_1_and_writes_nothing`, `export_stdout_is_parseable_json` | Asserts a refusal prints nothing to stdout and that `hi export` stdout parses as JSON. |
| REQ-main-005 | `check_fails_on_a_structural_problem` | Asserts the file, the problem code, and the line number all appear. |
| REQ-main-006 | No test | Structural; enforced by review, and visible in that `main.rs` imports only module entry points. |
| REQ-main-007 | `root_is_honored_by_capture_and_not_swallowed_into_the_sentence`, `a_flag_looking_word_inside_a_sentence_stays_a_word` | The first runs capture from an unrelated directory in both `--root PATH` and `--root=PATH` form, then asserts both sentences landed in the target workspace and that no `--root` reached the file. The second is the other half: it captures `the --root docs option should be documented` after the id and asserts the sentence survives intact, which is what the `break` in `peel_root` buys. |
| REQ-main-008 | `a_non_utf8_argument_is_reported_not_panicked` | Unix-only (`#[cfg(unix)]`, it builds the argument from raw bytes). Asserts exit 1 rather than 101, no `panicked` in stderr, and the `not valid UTF-8` message. |
| REQ-main-009 | `main::tests::index_will_not_write_while_another_writer_holds_the_lock` | The `hi index` half. The `hi view` half is a decision not to take a lock, so there is nothing to observe; what pins it is that `view::write` reads no page before writing one. |

## Manual Testing

- [x] Capture hi's own criteria through the built binary, which is how the routing and the
      one-line renderer were first exercised in anger. `hi check` on this repository reports
      `89 criteria · 8 families · 5 files` today.
- [x] `hi check` against the real repository, clean and with an injected structural problem.
- [ ] Confirm behavior on Windows, where `fs::canonicalize` returns a `\\?\`-prefixed path that
      `Workspace::rel`'s `strip_prefix` has never been exercised against. `rel`'s separator
      behavior is settled and unit-tested; only the prefix is open.

## Edge Cases & Boundary Conditions

| Case | Expected |
|---|---|
| Bare `hi` | Help, nothing read or written, exit 2 from clap |
| `hi --help`, `hi -V` | clap handles them; a flag's family part is empty so routing never intercepts |
| An id with no sentence | `a criterion needs a sentence. Say what you actually want`, exit 1 |
| An unknown subcommand | clap usage error, exit 2 |
| A first argument shaped like a word rather than an id, such as `spec-sync` | Not id-shaped, because the character after the `-` is not a digit; it reaches clap as an unrecognized subcommand, exit 2 |
| A wrongly cased id, `hi send-2 "<sentence>"` | Id-shaped, so it routes to capture and is refused there: `error: 'send-2' is not a valid id: family 'send' must start with A-Z and contain only A-Z, 0-9, _`, exit 1 |
| `hi SEND-a "<sentence>"` | Not id-shaped either, so it is an unrecognized subcommand at exit 2 rather than an alternation error at exit 1 |
| Run from outside any repository | `this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so run it inside one`, exit 1 |
| `hi export > file` | File contains only JSON |
| Two `hi retire` runs at once against the same file | Both take the lock in turn and each reloads inside it, so both retirements are in the file afterwards. Writing from the workspace `run` loaded before the lock used to put the file back as it was and leave the first-retired criterion live again, with both commands reporting success (hi: RETIRE-7) |
| `hi --root PATH SEND-2 "<sentence>"` and `hi --root=PATH SEND-2 "<sentence>"` | Both capture into `PATH`'s workspace; neither writes `--root` into the sentence |
| A `hi/` directory holding no hi file, such as a Hindi locale directory | Walked past; the search continues upward to the nearest qualifying `hi/` or `.git` |
| A repository nested inside another repository | The inner `.git` stops the walk, so the outer repository's criteria are not adopted; `hi check` in the child reports `0 criteria`, exit 0 |
| A capture argument that is not valid UTF-8 | `not valid UTF-8`, exit 1, no panic |
| A `--root PATH` value that is not valid UTF-8 | Used as a path; the following argument never goes through `str`, so the bytes reach `canonicalize` |
| `--root=PATH` where the whole token is not valid UTF-8 | Not peeled, because that arm matches on `to_str()`, so it falls through to clap: fine for a subcommand, and ahead of an id it makes the id look like an unrecognized subcommand, exit 2 |
| `--root` given twice ahead of the id | The last one wins |
| `hi --root` with nothing after it | Not peeled; it leads the tail, which is therefore not id-shaped, so clap reports the missing value as a usage error, exit 2 |
| `hi --root SEND-2 "<sentence>"`, a `--root` whose value was forgotten ahead of an id | The id is peeled as the root, so the sentence leads the tail and clap reports it as an unrecognized subcommand, exit 2 |
| A trailing `--root` after a sentence, as in `hi SEND-2 "text" --root /elsewhere` | The peel stopped at the id, so the whole thing is prose: `text --root /elsewhere` is captured into the current workspace, exit 0 |
| A flag-shaped word inside a sentence, as in `hi SEND-2 the --root docs option should be documented` | Captured whole; nothing is taken out (hi: CAPTURE-8) |
| `--root` naming a path that does not exist | `error: resolving <path>` plus the OS error, from `Workspace::find`, exit 1, on both paths |

## Gaps

- Windows is untested. The separator half of this is settled: `Workspace::rel` now joins
  components with `/` on every platform, and `workspace::tests::relative_paths_always_use_forward_slashes`
  pins it. What is still open is the prefix: `Workspace::find` canonicalizes, and on Windows that
  yields a `\\?\` prefix that `rel`'s `strip_prefix` removes only if the root carries the same
  prefix. CI builds on Windows but the integration tests have never been run there against a
  nested path.
- `print_report`'s plural forms are asserted only for `1 criterion`; the family and file plurals
  are unverified.
- `--root` is asserted only on the capture path. No test runs `hi --root PATH check` or any other
  subcommand with the flag, so the claim that clap still parses it for the subcommands rests on
  reading `main`, not on a test.
- `peel_root` has no unit test, because `main.rs` has no test module. A sentence containing the
  literal token `--root` is now covered end to end by
  `a_flag_looking_word_inside_a_sentence_stays_a_word`. What is still unpinned is `--root` given
  twice ahead of the id, a leading `--root` whose value is missing, and
  `hi --root SEND-2 "<sentence>"`, where the id is peeled as the root and the sentence becomes an
  unrecognized subcommand. Each of those exits 2 or picks the last root, and nothing asserts it.
- `a_non_utf8_argument_is_reported_not_panicked` is `#[cfg(unix)]`. The same path on Windows,
  where an invalid argument is ill-formed UTF-16 rather than an invalid byte, is untested.
- No test asserts that a non-UTF-8 `--root` value works, and the two forms do not behave alike:
  `--root PATH` carries the bytes through to `canonicalize`, while `--root=PATH` is not recognized
  at all. Nothing pins either.
- No test asserts that a `--root` naming a path that does not exist errors rather than silently
  falling back to the current directory.
