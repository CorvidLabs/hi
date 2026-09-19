---
spec: cli.spec.md
---

## Tasks

- [x] Cover the routing decision a unit test cannot reach: an id-shaped first argument, a wrongly cased one, an ordinary hyphenated word, and an unknown subcommand. Evidence: `an_id_shaped_argument_captures`, `a_wrongly_cased_id_is_reported_rather_than_read_as_prose`, `an_ordinary_hyphenated_word_is_not_mistaken_for_an_id`, `an_unknown_subcommand_is_a_usage_error`.
- [x] Cover `--root` in both directions, from a different working directory. Evidence: `root_is_honored_by_capture_and_not_swallowed_into_the_sentence`, `a_flag_looking_word_inside_a_sentence_stays_a_word`.
- [x] Keep a register of the reproduced ways an id was handed out twice. Evidence: `an_id_is_never_handed_out_twice`, five fixtures.
- [x] Cover the concurrent write paths from a repository that has never run hi. Evidence: `concurrent_captures_into_a_repository_with_no_hi_directory_all_land` (32 processes), `concurrent_retires_never_bring_a_criterion_back`.
- [x] Hold the strongest refusals to a byte-level listing of the whole tree. Evidence: `listing`, used by `a_file_hi_cannot_read_never_frees_the_id_reserved_in_it`.
- [x] Prove no path through the fledge hook can abort a push, from a working directory that is not the repository. Evidence: `nudge::no_path_through_the_hook_can_abort_a_push`, `nudge::it_never_guesses_at_a_repository_it_was_not_told_about`.
- [ ] Give `outside_a_repository_it_says_so_rather_than_guessing` a pid-qualified path. It is the one fixture in the file that uses a fixed `<tmp>/hi-cli-norepo`, and it would collide with a concurrent run of itself.
- [ ] Narrow `view_writes_a_self_contained_page`'s `src=` and `https://` assertions to the document's own markup. As substring tests over the whole page they would also fail on a criterion whose *text* contained either, which is a fixture hazard rather than a rule.
- [ ] Add a test for a single capture into a repository with no `hi/`. The path is covered only by the two tests that run many at once and assert on what survived; the plain case is in `specs/capture/`'s Manual Testing and nowhere else.
- [ ] Add a test for capturing into a file with no trailing newline, which gains one. It is the one documented exception to hi: FILE-4.a's byte-for-byte promise and nothing guards it.
- [ ] Add a test for a capture into a CRLF file through the binary. Line-ending preservation is covered at the `doc` level only (hi: FILE-10).
- [ ] Add a test for `hi issue --create`, which shells out to `gh`. Nothing exercises the `--create` path or its error message.
- [ ] Consider splitting the file. At 1,819 lines it is the largest in the repository, and the seven areas in Current Status are natural seams — but the register in `an_id_is_never_handed_out_twice` should stay in one piece.

## Gaps

- **No coverage of `hi issue --create`.** The `gh` invocation, its arguments and its failure message are untested end to end.
- **No coverage of a `--root` that does not exist.** `specs/main/` documents `error: resolving <path>` from `canonicalize`; no test asserts it.
- **No coverage of `hi view --out <path>`.** Only the default output path is exercised.
- **No coverage of a single bootstrap capture.** Every test that starts from `Repo::bare` runs several processes at once, so the ordinary first-capture-in-a-new-repository path is only ever observed under contention.
- **Nothing asserts the page's behavior.** `view_writes_a_self_contained_page` asserts on the HTML; `scripts/view-behaves.sh` is the only thing that opens it, and only the local gate and CI run that.
- **`mod nudge` does not run on Windows.** Coverage of the hook there is `scripts/nudge-behaves.sh` in the local gate, which is also unix.
- **No test asserts that stdout stays machine-readable across a verb that also writes a note.** `a_capture_survives_an_index_it_cannot_refresh` asserts the note is on stderr, but nothing parses stdout as a record in the presence of one.
- **No negative test for the fixture rule itself.** Nothing fails if a future fixture is written in capture's own list form, which is the weakening DECISIONS.md §26 is about.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
