---
spec: cli.spec.md
---

## User Stories

- As someone typing `hi SEND-2 "..."`, I want the id-shaped first argument to reach capture rather than clap, so that writing a criterion is one command with no subcommand to remember (hi: CAPTURE-1)
- As someone putting hi's own options before the id, I want them peeled off and the rest of the line kept whole, so that a sentence containing a flag-shaped word is still my sentence (hi: CAPTURE-8, CAPTURE-9)
- As someone piping `hi export` into a file, I want results on stdout and everything else on stderr, so that a note about a feature list never lands in my JSON (hi: CAPTURE-11, INDEX-4.a)
- As someone whose script checks `$?`, I want 1 to mean hi refused and 2 to mean I typed the command wrong, so that the two are distinguishable (hi: CHECK-1)
- As someone whose file hi refused to write to, I want the file byte-for-byte as it was, including the files hi normally starts on a first capture, so that a refusal is never a half-applied command (hi: CAPTURE-5)
- As someone who typed their own markdown, I want hi to read it and hand it back in my style, bare lines, CRLF, BOM, block frontmatter and all, so that adopting hi is not a reformatting of what I already have (hi: FILE-4, FILE-7, FILE-10, FILE-11, FILE-14)
- As someone who has been handed an id, I want it never handed out again, whichever verb wrote it and wherever in the file it ended up (hi: ID-5, FILE-22)
- As someone running `fledge work push`, I want hi's lifecycle hook never to abort my command, whatever state my repository is in
- As the author of the next fix, I want each reproduced defect to keep its fixture and its comment, so that a bug that shipped once cannot ship twice

## Acceptance Criteria

### REQ-cli-001

The integration suite SHALL drive the binary as a process, from a throwaway repository, and SHALL assert on exit code, on each output stream separately, and on the bytes on disk.

Acceptance Criteria

- Every test spawns `CARGO_BIN_EXE_hi`, so the binary under test is built from the current tree.
- `Repo` creates its root under `std::env::temp_dir()` named `hi-cli-<pid>-<name>` and removes it first, so two `cargo test` runs never share a fixture.
- Exit codes are asserted as `out.status.code() == Some(n)` wherever the number is the claim: 0 for success, 1 for a refusal, 2 for a clap usage error.
- `stdout` and `stderr` are separate helpers and are asserted separately. A refusal asserts `stdout(&out).is_empty()`.
- Assertions carry the output or the file body in their failure message.

### REQ-cli-002

The integration suite SHALL cover the routing decision that no unit test can reach (hi: CAPTURE-1, CAPTURE-8, CAPTURE-9, CAPTURE-1.c, CHECK-2.d).

Acceptance Criteria

- `an_id_shaped_argument_captures` proves an id-shaped first argument reaches capture rather than clap.
- `an_unknown_subcommand_is_a_usage_error` proves a non-id first argument reaches clap, exit 2.
- `a_wrongly_cased_id_is_reported_rather_than_read_as_prose` and `an_ordinary_hyphenated_word_is_not_mistaken_for_an_id` pin the two halves of `looks_like_id`: case-insensitive on the family, and requiring a digit after the hyphen.
- `root_is_honored_by_capture_and_not_swallowed_into_the_sentence` runs from a different working directory, in both the `--root PATH` and `--root=PATH` forms, and asserts the string `--root` appears nowhere in the file.
- `a_flag_looking_word_inside_a_sentence_stays_a_word` is the mirror: the same token after the id is written into the criterion intact.
- `a_non_utf8_argument_is_reported_not_panicked` (unix) asserts exit 1, `not valid UTF-8` on stderr, and the absence of `panicked`.
- `bare_invocation_prints_help_and_writes_nothing` asserts help is printed and `hi/` is still empty.

### REQ-cli-003

The integration suite SHALL hold every refusal to writing nothing, at a strength chosen to match what that refusal could have written (hi: CAPTURE-5).

Acceptance Criteria

- `an_existing_id_refuses_with_exit_1_and_writes_nothing`, `a_malformed_id_refuses_without_writing`, `a_zero_padded_id_is_refused_rather_than_silently_renamed`, `index_refuses_rather_than_guessing_when_a_marker_is_unclosed` and `capturing_into_a_family_two_files_claim_refuses_and_writes_nothing` each snapshot the file before the run and `assert_eq!` after it.
- `a_refusal_over_the_format_version_writes_nothing_at_all` additionally asserts that `INTENT.md`, `hi/AGENTS.md`, `hi/CLAUDE.md` and `hi/.hi.lock` do not exist, and that `hi/` holds exactly the one entry that was already there.
- `a_file_hi_cannot_read_never_frees_the_id_reserved_in_it` compares a `listing` of every path under the root with its bytes, before and after, so the refusal is held to changing nothing at all.
- Each of these strengths exists because a refusal that wrote something shipped.

### REQ-cli-004

The integration suite SHALL keep a register of the reproduced ways an id was handed out twice, one fixture per defect, each naming the defect it reproduces (hi: ID-5, FILE-13, FILE-20, FILE-22, RETIRE-5, RETIRE-6).

Acceptance Criteria

- `an_id_is_never_handed_out_twice` holds five fixtures in one function, each preceded by a comment naming the bug: a `## Retired` that is not the last section; a retirement reason containing newlines; a criterion inside a fence under `## Criteria`; a properly closed fenced example containing both `## Retired` and a criterion; and a fence the person never closed.
- Each block asserts both halves of the rule where the rule has two: an id hi cannot read is still taken, *and* an id nobody wrote is still free. Fixture 4 asserts that the documented `SEND-4` is still capturable and that the example prose is untouched.
- Fixture 5 asserts capture refuses **twice** for the same id and that the half-written file is byte-identical, because the defect was reporting the same id saved twice with both lines invisible.
- A block is never removed when its bug is fixed.

### REQ-cli-005

The integration suite SHALL cover the concurrent write paths, asserting on what survived rather than on a count (hi: FILE-19, RETIRE-7, CAPTURE-14).

Acceptance Criteria

- `concurrent_captures_all_land` runs eight captures of distinct ids against a seeded repository and requires all eight in the file.
- `concurrent_captures_into_a_repository_with_no_hi_directory_all_land` runs thirty-two against `Repo::bare`, requires every reported success to be readable, and requires that nothing refused — printing each refusal's stderr rather than counting them. Thirty-two rather than eight because eight did not reproduce the bootstrap race reliably.
- `concurrent_retires_never_bring_a_criterion_back` runs two retires at once and requires both ids to sit below `## Retired` afterwards, with the third criterion untouched.
- None of these asserts an ordering between processes.

### REQ-cli-006

The integration suite SHALL assert that hi returns a person's file in the person's own style, over fixtures hi did not write (hi: FILE-4, FILE-7, FILE-10, FILE-11, FILE-14, FILE-1.b, FILE-1.c, FILE-6).

Acceptance Criteria

- `Repo::with_chat` seeds a **bare** `SEND-1  I hit enter and it shows up.` line: no bullet, no emphasis, and not what capture emits.
- `block_style_frontmatter_is_understood_and_preserved` asserts a YAML block `families:` list is read, gains the new family as another `  - NAME` item, and is never collapsed to `families: [`.
- `a_bom_does_not_make_a_valid_file_look_broken` asserts a BOM'd file checks clean.
- `a_hi_file_renders_as_a_list_not_a_wall_of_text` asserts the three written lines exactly, including the two-space indent on the case, and then round-trips them through `hi export`.
- `a_long_sentence_stays_on_one_line` asserts the captured line equals `- **SEND-2**  <sentence>` for a sentence far past terminal width.
- `a_ticket_unwraps_prose_and_leaves_the_file_alone` asserts a wrapped paragraph arrives whole in the ticket, that a blank line stays a break, and that the source file is byte-identical afterwards.

### REQ-cli-007

The integration suite SHALL assert that a generated feature list is true after every verb that changes the count, and that hi never writes prose a person removed (hi: INDEX-2, INDEX-4, INDEX-4.a, INDEX-4.c, INDEX-2.b, INDEX-2.c).

Acceptance Criteria

- The `HAND_WRITTEN_INTENT` fixture is prose either side of a block whose count is *already wrong*, so an assertion that the count is now right cannot pass by the block never being touched.
- `a_capture_refreshes_the_feature_list` and `a_retire_refreshes_the_feature_list` assert the corrected count and that the prose above and below the markers is byte-identical.
- `a_capture_survives_an_index_it_cannot_refresh` asserts exit 0, the criterion on disk, `INTENT.md` unchanged, and `not refreshed` on stderr.
- `a_feature_list_i_deleted_stays_deleted` asserts a capture leaves a marker-less `INTENT.md` untouched with nothing on stderr, and that `hi index` is how a person asks for one back.
- `a_root_file_hi_cannot_read_survives_a_capture_byte_for_byte` and `hi_index_refuses_a_root_file_it_cannot_read_rather_than_replacing_it` cover the undecodable file on both paths: a note and exit 0 from a capture, a refusal and exit 1 from `hi index`, every byte intact in both.
- `index_rewrites_only_the_generated_block` and `index_leaves_a_marker_quoted_in_prose_alone` assert the person's sentences, including one quoting the marker itself, survive.
- `the_first_capture_starts_the_product_intent_file` asserts the starter file is born with a true list.

### REQ-cli-008

The integration suite SHALL assert that `hi check` fails on structure and only on structure, and that its notes are data (hi: CHECK-1, CHECK-2, CHECK-6).

Acceptance Criteria

- `check_is_clean_on_unfinished_intent` asserts exit 0 over a criterion with nothing implementing it.
- `check_fails_on_a_structural_problem`, `check_fails_on_a_duplicate_id`, `check_fails_when_two_files_claim_the_same_family` and `a_criterion_in_a_file_hi_skips_is_reported_rather_than_vanishing` each assert exit 1 and the problem kind by name, with the file and, for the orphan, the line number.
- `check_says_the_feature_list_is_behind_without_failing` asserts exit 0, the note present, and the word `problem` absent — a note is not a seventh problem.
- `check_json_carries_its_notes_as_a_list_with_codes` asserts `notes` is an array, each entry has a `kind`, no message contains a newline, and the old joined `note` string is `null`.

### REQ-cli-009

The integration suite SHALL assert that a file declaring a later format version is refused by every verb, and that a file declaring none is still this format (hi: FILE-25, FILE-25.a).

Acceptance Criteria

- `a_file_from_a_later_format_is_refused_by_every_verb` loops over eleven invocations covering every subcommand and the capture path, and requires exit 1, both `hi/chat.md` and `hi: 2` on stderr, and empty stdout from each.
- `a_refusal_over_the_format_version_writes_nothing_at_all` asserts the refusal happens before `lock::acquire`, by requiring that no `hi/.hi.lock` was ever created.
- `a_file_that_declares_nothing_is_still_this_format` asserts a file written before `hi:` existed still works, found through the repository boundary rather than through the key it does not have.

### REQ-cli-010

The integration suite SHALL assert that no path through the fledge lifecycle hook can abort the command that ran it.

Acceptance Criteria

- `mod nudge` is `#[cfg(unix)]` and runs `bin/fledge-hi-nudge` as a process, passing the moment as argv and the repository as `FLEDGE_REPO_ROOT`, the way fledge does.
- `no_path_through_the_hook_can_abort_a_push` requires exit 0 from all sixteen combinations of four moments (`start`, `push`, empty, `unknown-moment`) and four roots (a repository, a repository with a `hi/`, a path that does not exist, and no variable at all).
- `nothing_reaches_stdout_so_a_json_envelope_stays_valid` requires stdout empty and stderr non-empty at both speaking moments.
- `a_repository_with_nothing_written_down_is_told_so_at_both_moments` requires the two moments to say different things.
- `it_goes_quiet_once_something_is_written_down` requires silence once a `hi/` exists.
- `it_never_guesses_at_a_repository_it_was_not_told_about` runs with the hook's working directory set to a repository that has **no** `hi/`, so a hook guessing from cwd would speak and be caught.

## Constraints

- The suite tests a process. Anything assertable by calling a function belongs in that module's own unit tests; what lives here is what needs argv, an exit code, a stream or a byte on disk.
- Fixtures are files hi did not write, and they stay that way. Tidying a bare criterion line into capture's list form makes the suite a round-trip of the writer against its own output, which is the failure DECISIONS.md §26 records.
- Where a test asserts that a generated value was corrected, the seeded value must be wrong in a way the correct answer is not.
- Every scratch path carries the process id.
- `Repo::new` pre-creates `hi/` and therefore cannot exercise a first capture. A write-path test about a repository that has never run hi must use `Repo::bare`.
- Error strings asserted here are part of hi's contract with a person, and duplicate the assertions in the module unit tests on purpose: `already exists`, `next free is SEND-2`, `leading zero`, `not valid UTF-8`, `not a repository`, `not refreshed`, `it is yours`, `lowercase`, `feature list is behind`.
- The suite must keep running on Windows. Only the two things that cannot — `OsStrExt` and a shell script — are behind `#[cfg(unix)]`.

## Out of Scope

- The behavior being asserted. Each verb's rules live in its own spec; this suite only observes them from outside.
- Generated sequences. Random capture / retire / hand-edit orderings are `specs/promise/`; this suite pins named, reproduced cases.
- The page's behavior in a browser. `view_writes_a_self_contained_page` asserts on the HTML going in; what the page *does* is `scripts/view-behaves.sh`, driven in headless Chrome, because two bugs shipped past HTML-level assertions.
- `hi check`'s own coverage of the seven problem kinds, which lives in `specs/check/`; only the exit code, the kind names and the JSON shape are asserted here.
- The fledge plugin manifest, validated by `fledge plugins validate . --strict` in the gate rather than by a test.
