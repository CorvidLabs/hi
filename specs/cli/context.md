---
spec: cli.spec.md
---

## Key Decisions

- **The suite tests a process, and that is the whole division of labour.** Every module has unit
  tests that call its functions. What cannot be reached that way is argv before clap sees it, an
  exit code, which stream a line landed on, and the bytes of a file after the process ended. If an
  assertion here could be written by calling a function, it belongs in that module instead — and
  several here are deliberate duplicates anyway, because an error string shown to a person is a
  contract and is worth pinning from both sides.
- **Almost every fixture is a file hi did not write.** `Repo::with_chat` seeds a bare
  `SEND-1  I hit enter and it shows up.` with no bullet and no bold id, which is *not* what capture
  emits. Three id defects shipped because every write-path test asserted over files hi had produced
  itself, so the writer was tested against its own output and agreed with itself (DECISIONS.md §26).
  A fixture "tidied" into the list form is a silent weakening of the suite, and it would still pass.
- **Where a count is asserted, the seeded count is wrong on purpose.** `HAND_WRITTEN_INTENT` says
  `(1 criterion)` where the answer is 2, and `a_retire_refreshes_the_feature_list` rewrites it to
  `(9 criteria)` — a number that is neither the count before nor the count after. Seeding the correct
  value would let the assertion pass by the block never being touched at all.
- **The three `Repo` constructors are three different claims.** `Repo::new` gives an empty `hi/`;
  `Repo::with_chat` adds a seeded criteria file; `Repo::bare` gives a `.git` and **no** `hi/`. The
  third is the one that matters: the first capture in an adopting repository has no directory to
  put a lock file in, and `lock::acquire` used to hand back a `Guard` it had not taken when the open
  failed, so every bootstrap capture ran unlocked. A test built on `Repo::new` cannot see any of
  that, because `hi/` already exists (DECISIONS.md §33).
- **Thirty-two concurrent captures, not eight.** `concurrent_captures_into_a_repository_with_no_hi_directory_all_land`
  uses thirty-two because eight did not reproduce the bootstrap race reliably, and because
  thirty-two is the shape adoption actually has — fledge's own adoption was 197 captures. The
  ordinary concurrency test above it uses eight, which is enough for a lock that either exists or
  does not.
- **A quiet refusal in a concurrency test is a failure, and its stderr is printed.** An id nobody
  else asked for is free, so a capture of `SEND-7` refusing while `SEND-8` succeeds is a defect. The
  assertion joins every refusal's stderr into the message rather than counting them, because the one
  time it fired it was a lock handoff on Windows and a count says nothing about that.
- **`an_id_is_never_handed_out_twice` is a register, not a test.** Five fixtures in one function,
  each preceded by a comment naming the shipped bug it reproduces. Keeping them together is
  deliberate: read end to end it is the list of every way hi's own verbs have broken the one thing
  hi promises. A block is never deleted when its bug is fixed — that is the only thing keeping it
  fixed. It grows; it does not shrink.
- **There is a strength ladder for "a refusal writes nothing", and the rung is chosen per test.**
  Most refusal tests snapshot the destination file. `a_refusal_over_the_format_version_writes_nothing_at_all`
  also asserts the *absence* of `INTENT.md`, `hi/AGENTS.md`, `hi/CLAUDE.md` and `hi/.hi.lock`, which
  is how it proves the refusal happened before `lock::acquire` ran.
  `a_file_hi_cannot_read_never_frees_the_id_reserved_in_it` compares a `listing` of every path under
  the root with its bytes. Each rung exists because a refusal that wrote something did ship
  (hi: CAPTURE-5).
- **Exit codes are asserted as numbers.** 1 is hi refusing and 2 is clap reporting a usage error,
  and a test that accepts "not success" cannot tell an id error from an unknown subcommand. That
  distinction is the whole content of `an_unknown_subcommand_is_a_usage_error` and of the
  `SEND-a` row in `specs/main/`.
- **`stdout` and `stderr` use `from_utf8_lossy`.** A test asserting that a non-UTF-8 argument did not
  panic still has to be able to read what was printed.
- **`mod nudge` is here rather than in its own file** because it is the same kind of claim about a
  different executable. Its load-bearing property is exit 0 from every path: fledge propagates a
  hook's exit code, so a non-zero exit aborts the command that ran it, and hi has no business
  blocking somebody's `fledge work push`. `scripts/nudge-behaves.sh` covers the same ground but only
  the local gate runs it; these run wherever `cargo test` does.
- **The hook's working directory is a decoy on purpose.** `decoy()` is a repository with a `.git`
  and no `hi/`. Running the hook from the crate root would hide the bug it guards: hi's own
  repository *has* a `hi/`, so a hook that ignored `FLEDGE_REPO_ROOT` and guessed from its cwd would
  fall silent there and look correct. Mutation testing found exactly that, passing for the wrong
  reason.
- **`Repo::with_chat` puts `SEND-1` on line 10, and that number is load-bearing.** The duplicate-id
  refusal quotes the line number, and `specs/capture/` documents it. Adding a line to that fixture
  changes a message other tests read.
- **The suite keeps running on Windows.** Only two things are `#[cfg(unix)]`:
  `a_non_utf8_argument_is_reported_not_panicked`, which needs `OsStrExt` to build an invalid
  argument, and `mod nudge`, which runs a shell script. The concurrency tests in particular run
  everywhere, which is how the one Windows lock-handoff refusal was ever observed.

## Files to Read First

- `tests/cli.rs` itself, starting with `Repo` at the top and `listing` and `future_format` near the
  bottom. The comment above each test says which defect or which criterion it is about; those
  comments are the index.
- `src/main.rs` for what is being driven: `peel_root`, the `args_os` collection, `looks_like_id`
  routing, and the exit-code mapping. `specs/main/` describes all of it, and the Error Cases table
  there and the assertions here should always agree.
- `DECISIONS.md` §26 for why the fixtures are files hi did not write, §33 for the bootstrap lock and
  the concurrent-retire defect, §35 for the resurrection fixture
  (`a_capture_never_brings_a_retired_criterion_back_to_life`), §32 for the reservation lookup and
  the `INTENT.md` over-write, and §36 for the unreadable-is-not-absent pair.
- `specs/promise/` for the generated counterpart. This suite pins named cases; that module looks for
  unnamed ones. Read them together before adding a test to either.
- `bin/fledge-hi-nudge` and `scripts/nudge-behaves.sh` before touching `mod nudge`.
- `hi/file.md`, `hi/capture.md` and `hi/check.md` for the criteria the assertions cite.

## Current Status

Sixty-seven tests, all passing, run by `cargo test --test cli` and by the `test` step of
`fledge lanes run verify`. Sixty-two at the top level and five inside `#[cfg(unix)] mod nudge`. The
whole suite finishes in under two seconds despite spawning a process per assertion, because the
fixtures are small and the binary does very little.

Coverage by area, roughly: thirteen tests on argv routing, exit codes and the stream split; eight on
discovery, refusals and the format version; eleven on `check`; thirteen on `out`'s four verbs and the
`INTENT.md` rules; seven on id permanence including the five-fixture register and the three
concurrency tests; three on retire; three on `seed` and the agent file; one on the page; and the
five hook tests.

Two things a future agent should know:

1. **`outside_a_repository_it_says_so_rather_than_guessing` uses a fixed path.** It builds
   `<tmp>/hi-cli-norepo/deep/nested` with no pid in it, which is the one place Invariant 1 is not
   honoured. It would collide with a concurrent run of itself. Nothing else in the file does this.
2. **`view_writes_a_self_contained_page` asserts on the HTML going in, not on the page.** It checks
   the criterion text, the dark-mode query, the absence of `src=` and `https://`, and that the search
   input, the sort options, the filter rail and a deep link are present. What the page *does* —
   whether filtering actually hides a row — is `scripts/view-behaves.sh` in headless Chrome, because
   two bugs shipped past HTML-level assertions exactly like these.

## Notes

- `assert!(!page.contains("src="))` in `view_writes_a_self_contained_page` is a substring test over
  the whole document, so it also forbids `src=` inside a criterion's own text. No fixture has one,
  and a future fixture that did would fail for a reason that is not the rule being tested.
- `a_criterion_in_a_file_hi_skips_is_reported_rather_than_vanishing` asserts the word `not lowercase`
  in `check`'s output. That string is `check`'s wording for `StrayPlace::UnreadFile`; changing it is
  a contract change in two places.
- `what_hi_writes_passes_hi_own_check` is the self-consistency test: hi retire wrote the reason after
  the cases while hi check looked for it under the parent, so hi produced files that failed its own
  check. It also asserts the retired *cases* carry no reason of their own, through `hi export`.
- `hi_s_own_files_are_not_counted_as_features` is the counterpart to `hi/AGENTS.md` being written on
  a first capture: those files sit inside `hi/` and are hi's, so they are neither criteria files nor
  lines in the product's feature list (DECISIONS.md §27).
- `seed_rewrites_a_template_hi_has_shipped_and_refuses_an_edit` `include_str!`s
  `../src/seed/agents_0_5.md`, so the "a template hi has shipped" fixture is the shipped bytes rather
  than a copy that could drift.
- The concurrency tests spawn threads that each spawn a process. The thread is only a way to start
  them at once; nothing is asserted about the threads themselves, and no test asserts an ordering
  between processes.
- Several tests run `hi check` *before and after* the operation under test and assert on both. That
  pattern is deliberate: the defects this suite was written for mostly left `hi check` exiting 0 on
  both sides, so the before-value is part of the evidence that the after-value means something.
