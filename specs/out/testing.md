---
spec: out.spec.md
---

## Automated Testing

Automated coverage for this module lives in the `#[cfg(test)] mod tests` block at the bottom of
`src/out.rs`, plus the process-level tests in `tests/cli.rs` that drive the real binary. Run them
with `cargo test out::` and `cargo test --test cli` (or `fledge run test` for the whole suite). The shared
fixture is the local `workspace(files: &[(&str, &str)])` helper, which builds a `Workspace` from
`Doc::parse` with `root: /r` and `dir: /r/hi` so no filesystem is touched, and the `CHAT` constant,
a one-file document declaring `families: [SEND]` with `SEND-1`, its case `SEND-1.a`, `SEND-2`, and
the intent sentence `It should feel like texting.`

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/out.rs::issue_body_carries_the_id_and_its_cases` | Unit | REQ-out-002, REQ-out-003. Asserts the title is the sentence with its trailing period trimmed, the body contains `hi: SEND-1`, the case `SEND-1.a` is present, the sibling `SEND-2` is absent, and the file's intent prose is carried into the body. |
| `src/out.rs::a_ticket_unwraps_prose_the_author_only_wrapped` | Unit | REQ-out-003, REQ-out-015. Over the `WRAPPED` fixture, whose `## Intent` is two paragraphs hard-wrapped at about 76 columns with a blank line between them: asserts the first paragraph arrives as one line, that no authored wrap survives as a newline, and that the blank line between the paragraphs is still there. This is the regression test for the reported defect and it fails without the `unwrap_soft_breaks` call in `issue_markdown`. |
| `src/out.rs::only_a_wrapped_line_is_joined` | Unit | REQ-out-015. Eleven cases straight against `unwrap_soft_breaks`: a wrapped paragraph joins; a blank line separates; a list, an ordered list, a backtick fence, a tilde fence, a heading, a table, a block quote and a thematic break all keep their own newlines; a wrapped list item is joined into the item it belongs to; and a line ending in two spaces keeps both the break and the marker. |
| `src/out.rs::the_files_hi_writes_are_one_line_per_paragraph` | Unit | REQ-out-016. Asserts `agent_instructions()` and `starter_intent()` pass through `unwrap_soft_breaks` unchanged, so hi's own starter text cannot drift back into the wrapping it renders around. |
| `src/out.rs::export_of_the_whole_repo_includes_every_file` | Unit | REQ-out-006, REQ-out-007, REQ-out-008. Asserts `scope == "repo"`, that all three criteria are present, that `criteria[1].parent == "SEND-1"`, and that the file's `intent` is the `## Intent` prose. |
| `src/out.rs::export_scoped_to_a_family_keeps_only_that_family` | Unit | REQ-out-007. Two files (`chat.md` owning `SEND`, `billing.md` owning `BILLING`); scope `SEND` yields exactly one file entry, `hi/chat.md`. |
| `src/out.rs::export_scoped_to_a_file_keeps_that_file` | Unit | REQ-out-006, REQ-out-007. Scope `chat` (the file stem without `.md`) yields `scope == "chat"` and exactly one file entry. |
| `src/out.rs::export_rejects_an_unknown_scope` | Unit | REQ-out-009. Scope `NOPE` returns `Err`. |
| `src/out.rs::export_scoped_to_an_id_keeps_it_with_its_cases_and_its_intent` | Unit | REQ-out-022. Scope `SEND-1` over two files yields one file entry, its intent, `criteria` of `SEND-1`, `SEND-1.a`, `SEND-1.a.1`, `SEND-1.b` in file order, no `SEND-2`, an empty `retired`, and no `product`. |
| `src/out.rs::export_of_a_case_brings_every_criterion_it_sits_under` | Unit | REQ-out-022. Scope `SEND-1.a.1` yields `SEND-1`, `SEND-1.a`, `SEND-1.a.1` and asserts every `parent` names an entry in the payload (hi: EXPORT-7.a). |
| `src/out.rs::export_of_an_orphan_case_names_the_parent_it_does_not_have` | Unit | REQ-out-022. Over a hand-edited file holding `SEND-2.a` and no `SEND-2`, scope `SEND-2.a` yields only that case, whose `parent` is still `"SEND-2"`: export neither invents the parent nor drops the case (hi: CHECK-2.b). |
| `src/out.rs::an_id_wins_over_a_family_frontmatter_declared_by_the_same_name` | Unit | REQ-out-022. A file declaring `families: [SEND, SEND-1]` still exports `SEND-1` as an id, with `SEND-1` and `SEND-1.a`, rather than as an empty family slice. |
| `src/out.rs::a_retired_case_comes_with_the_live_criterion_it_was_a_case_of` | Unit | REQ-out-022. Scope `SEND-1.b`, retired under a live `SEND-1`, yields `criteria` of `SEND-1` and `retired` of `SEND-1.b` (hi: EXPORT-4, EXPORT-7.a). |
| `src/out.rs::an_id_shaped_scope_that_is_not_one_says_why` | Unit | REQ-out-022. Scope `SEND-01` is refused with `is not a valid id` and the leading-zero reason. |
| `src/out.rs::export_of_a_retired_id_keeps_it_apart` | Unit | REQ-out-022. Scope `SEND-3`, which is retired, yields an empty `criteria` and `retired` of `SEND-3` (hi: EXPORT-4). |
| `src/out.rs::export_of_an_id_is_the_same_envelope_with_less_in_it` | Unit | REQ-out-022, REQ-out-006. The top-level and per-file key sets for scope `SEND-1.a` equal those for scope `SEND` (hi: EXPORT-3). |
| `src/out.rs::export_rejects_an_id_nobody_wrote` | Unit | REQ-out-022, REQ-out-009. Scope `SEND-9` returns `Err`, and the message names an id as a scope. |
| `src/out.rs::a_marker_quoted_in_prose_is_not_the_generated_block` | Unit | REQ-out-012. Over an `INTENT.md` whose prose quotes `<!-- hi:index -->` mid-sentence above the real marker pair, `index_span` returns the real block: the sentence sits before `span.start`, `stale` is inside the span, and `span.end` leaves the closing line's newline outside so the blank line after the block survives. |
| `src/out.rs::an_unclosed_marker_has_no_span` | Unit | REQ-out-013. `index_span` is `None` for an opening marker with no close, while `has_marker_line` still sees the opening marker, the exact pair of conditions that makes `write_index` bail. |
| `src/out.rs::index_lists_each_file_with_its_families_and_count` | Unit | REQ-out-010. The generated block contains `[chat](hi/chat.md)`, the family `SEND`, and `(3 criteria)`. |
| `tests/cli.rs::export_stdout_is_parseable_json` | Integration | REQ-out-006. Runs the real binary in a temp repo: stdout parses as JSON, `hi == 1`, `scope == "repo"`, and the first criterion id is `SEND-1`. |
| `tests/cli.rs::export_rejects_a_scope_that_matches_nothing` | Integration | REQ-out-009. `hi export NOPE` exits 1 and writes nothing to stdout, so a failed export cannot be piped into a consumer. |
| `tests/cli.rs::export_accepts_the_path_it_prints` | Integration | REQ-out-007. Runs `hi export` three times over one repo with the scopes `chat`, `chat.md`, and `hi/chat.md`, asserting all three succeed, so the repository-relative path the tool prints is accepted back as a scope. It asserts on the exit status only, not on the payload. |
| `tests/cli.rs::issue_prints_a_ticket_carrying_the_id` | Integration | REQ-out-002, REQ-out-003. `hi issue SEND-1` succeeds and stdout carries `hi: SEND-1` and the file's intent prose. |
| `tests/cli.rs::a_ticket_unwraps_prose_and_leaves_the_file_alone` | Integration | REQ-out-015, and FILE-4 from the other side. Drives the real binary over a `hi/host.md` whose intent is hard-wrapped: stdout carries each paragraph whole with the blank line between them intact, and the file on disk is asserted byte-identical to what was written. |
| `tests/cli.rs::index_rewrites_only_the_generated_block` | Integration | REQ-out-011. `hi index` over an `INTENT.md` with prose and a stale block keeps the prose, drops the stale text, and writes the new bullet. |
| `tests/cli.rs::index_refuses_rather_than_guessing_when_a_marker_is_unclosed` | Integration | REQ-out-013. An `INTENT.md` with an opening marker and no close makes `hi index` exit 1, and the file is asserted byte-identical to what it was before the run. |
| `tests/cli.rs::index_leaves_a_marker_quoted_in_prose_alone` | Integration | REQ-out-012. The quoted sentence survives a real `hi index`, the stale block is replaced, and `<!-- /hi:index -->\n\n` shows the blank line after the block survived. |
| `src/out.rs::a_list_that_matches_is_worth_no_note` | Unit | REQ-out-018. Over a real temp directory (the `on_disk` helper, pid in its name), an `INTENT.md` whose block already matches yields `None`. The negative half: it fails under a mutation that makes `index_note` always nag. |
| `src/out.rs::a_list_that_disagrees_is_a_note` | Unit | REQ-out-018. The same file with `(3 criteria)` changed to `(1 criterion)` yields a note naming `INTENT.md` and saying the feature list is behind. Fails when `index_note` is mutated to return `None`. |
| `src/out.rs::a_list_hi_can_no_longer_refresh_is_a_note_too` | Unit | REQ-out-018. An unpaired opening marker yields the note that nothing can refresh the list, which is the only thing left that says so once REQ-out-017 swallows the refusal on the capture path. |
| `src/out.rs::a_file_with_no_block_at_all_is_not_a_list_that_is_behind` | Unit | REQ-out-018. Prose with no marker line yields `None`. The second negative half. |
| `src/out.rs::refresh_hands_back_a_refusal_instead_of_raising_it` | Unit | REQ-out-017. Over an unpaired opening marker, `refresh_index` returns `Some` carrying INDEX-2.b's message and the file is asserted byte-identical afterwards. Fails when `refresh_index` is mutated to discard the reason. |
| `tests/cli.rs::a_capture_refreshes_the_feature_list` | Integration | REQ-out-017. Over a hand-written `INTENT.md` whose block says `(1 criterion)`, a real capture leaves it saying `(2 criteria)` while the prose above and below the markers is asserted byte-identical with `starts_with` and `ends_with`. Fails both when the refresh is removed from `capture` and when `capture` stops reloading the file it just saved. |
| `tests/cli.rs::a_retire_refreshes_the_feature_list` | Integration | REQ-out-017. Three live criteria, a block hand-set to `(9 criteria)` so the assertion cannot pass by the block never moving, then a real `hi retire`: the block reads `(2 criteria)`. |
| `tests/cli.rs::a_capture_survives_an_index_it_cannot_refresh` | Integration | REQ-out-017. An `INTENT.md` with an unpaired opening marker: the capture exits 0, the criterion is in `hi/chat.md`, `INTENT.md` is byte-identical, and stderr says the list was not refreshed. |
| `tests/cli.rs::check_says_the_feature_list_is_behind_without_failing` | Integration | REQ-out-018. A criterion typed into `hi/chat.md` by hand: `hi check` exits 0, says the feature list is behind, and prints no problem. Running `hi index` clears the note. |

### Requirement Coverage Map

| Requirement | Covering `#[test]` functions |
|-------------|------------------------------|
| REQ-out-001 (`ls`) | None. Uncovered. See Gaps. |
| REQ-out-002 (`issue` prints by default) | `issue_body_carries_the_id_and_its_cases`, `issue_prints_a_ticket_carrying_the_id` (the stdout path itself) |
| REQ-out-003 (id, cases, intent in the body) | `issue_body_carries_the_id_and_its_cases`, `issue_prints_a_ticket_carrying_the_id`, `a_ticket_unwraps_prose_the_author_only_wrapped` |
| REQ-out-004 (`--create` shells to `gh`) | None. Uncovered. See Gaps. |
| REQ-out-005 (retired never becomes work) | None. Uncovered. See Gaps. |
| REQ-out-006 (one envelope at every scope) | `export_of_the_whole_repo_includes_every_file`, `export_scoped_to_a_file_keeps_that_file`, `export_stdout_is_parseable_json` |
| REQ-out-007 (family / file / repo scopes) | `export_of_the_whole_repo_includes_every_file`, `export_scoped_to_a_family_keeps_only_that_family`, `export_scoped_to_a_file_keeps_that_file`, `export_accepts_the_path_it_prints` (the stem, file-name, and `hi/<stem>.md` spellings). The family/file-stem collision is still uncovered. See Gaps |
| REQ-out-008 (prose in the payload) | `export_of_the_whole_repo_includes_every_file` (file `intent`; `product` is uncovered) |
| REQ-out-009 (unknown scope refuses) | `export_rejects_an_unknown_scope`, `export_rejects_a_scope_that_matches_nothing` (exit code and empty stdout) |
| REQ-out-010 (`index_block`) | `index_lists_each_file_with_its_families_and_count` |
| REQ-out-011 (`write_index` splice) | `index_rewrites_only_the_generated_block` (prose survives, stale block replaced), `out::tests::a_block_that_is_there_is_still_refreshed` (the splice runs under `LeaveAlone` too), `out::tests::hi_index_still_installs_a_block_because_it_was_asked_for` (the append branch under `Install`), `out::tests::a_starter_root_file_is_born_with_its_feature_list` and `the_first_capture_starts_the_product_intent_file` (the starter branch). |
| REQ-out-019 (only `NotFound` may create) | `out::tests::a_root_file_hi_cannot_decode_is_left_alone` (the refresh hands the reason back and the bytes survive), `a_root_file_hi_cannot_read_survives_a_capture_byte_for_byte` (end to end, exit 0, warned), `hi_index_refuses_a_root_file_it_cannot_read_rather_than_replacing_it` (end to end, exit 1, bytes survive). Each fails without the fix: with `unwrap_or_default` restored, `hi index` prints `INTENT.md  index updated` and replaces the file. |
| REQ-out-017's install split | `out::tests::a_refresh_puts_back_no_block_that_was_removed`, `a_feature_list_i_deleted_stays_deleted` (capture leaves it, `hi index` installs it). Guarded against over-correction by `a_capture_refreshes_the_feature_list`, `a_retire_refreshes_the_feature_list` and `a_block_that_is_there_is_still_refreshed`, which fail if a block that exists stops being refreshed. |
| REQ-out-012 (markers matched on whole lines) | `a_marker_quoted_in_prose_is_not_the_generated_block`, `index_leaves_a_marker_quoted_in_prose_alone` |
| REQ-out-013 (unclosed marker refuses) | `an_unclosed_marker_has_no_span`, `index_refuses_rather_than_guessing_when_a_marker_is_unclosed` |
| REQ-out-015 (soft breaks unwrapped into a ticket) | `a_ticket_unwraps_prose_the_author_only_wrapped`, `only_a_wrapped_line_is_joined`, `a_ticket_unwraps_prose_and_leaves_the_file_alone` |
| REQ-out-016 (hi's own text is one line per paragraph) | `the_files_hi_writes_are_one_line_per_paragraph` |
| REQ-out-017 (`refresh_index`) | `refresh_hands_back_a_refusal_instead_of_raising_it`, `a_capture_refreshes_the_feature_list`, `a_retire_refreshes_the_feature_list`, `a_capture_survives_an_index_it_cannot_refresh`, and `capture::tests::a_capture_leaves_the_generated_list_true` / `an_index_that_cannot_be_refreshed_is_not_a_failed_capture` in `specs/capture` |
| REQ-out-018 (`index_note`) | `a_list_that_matches_is_worth_no_note`, `a_list_that_disagrees_is_a_note`, `a_list_hi_can_no_longer_refresh_is_a_note_too`, `a_file_with_no_block_at_all_is_not_a_list_that_is_behind`, `check_says_the_feature_list_is_behind_without_failing` |
| REQ-out-014 (renders only parsed structure) | None here. The parser side is covered in `specs/doc`; nothing asserts that a fenced or stray id is absent from this module's output. See Gaps. |
| REQ-out-022 (one id as a scope) | `export_scoped_to_an_id_keeps_it_with_its_cases_and_its_intent`, `export_of_a_case_brings_every_criterion_it_sits_under`, `export_of_an_orphan_case_names_the_parent_it_does_not_have`, `an_id_wins_over_a_family_frontmatter_declared_by_the_same_name`, `a_retired_case_comes_with_the_live_criterion_it_was_a_case_of`, `an_id_shaped_scope_that_is_not_one_says_why`, `export_of_a_retired_id_keeps_it_apart`, `export_of_an_id_is_the_same_envelope_with_less_in_it`, `export_rejects_an_id_nobody_wrote`, `export_takes_one_id_from_the_command_line` |
| `tests/cli.rs` (`export_says_which_shape_it_is_apart_from_which_format_it_read`) | Integration | REQ-out-020. `hi export` carries `"hi": 1` for the format the files are in and `"export": 1` for the shape of the payload, as two separate fields. |
| `tests/cli.rs` (`the_agent_file_says_to_check_the_ids_after_a_merge`) | Integration | REQ-out-016. The `hi/AGENTS.md` a first capture writes names `hi check`, a merge, and two branches choosing the same id (hi: HABIT-5). |
| `tests/cli.rs` (`export_takes_one_id_from_the_command_line`) | Integration | REQ-out-022. `hi export SEND-1.a` through the real binary exits 0 with `scope == "SEND-1.a"` and `criteria` of `SEND-1` and `SEND-1.a`, and not `SEND-2`, over list-item criteria. |

## Manual Testing

These flows run against this repository's own `hi/` directory, which is the module's live fixture.

- [ ] `./target/release/hi ls`: every file is listed flush left, criteria are indented two spaces
      per depth level, and `SEND-1.a`-shaped cases sit visibly under their parents (REQ-out-001).
- [ ] `./target/release/hi ls --family EXPORT`: only `hi/generate.md` appears, and only its
      `EXPORT` lines; files with no `EXPORT` criteria print no heading at all (REQ-out-001).
- [ ] `./target/release/hi ls --retired`: retired lines appear with a trailing `(retired)` marker
      (REQ-out-001). This repository currently holds no retired criteria, so the flag changes
      nothing here; run it in a scratch repo with a `## Retired` section to see the marker.
- [ ] `./target/release/hi issue EXPORT-1` prints `## I can hand an agent everything it needs to
      write the spec in one command`, then a body containing `hi: EXPORT-1` and the `Generate`
      intent prose (REQ-out-002, REQ-out-003).
- [ ] `./target/release/hi issue ISSUE-1`: the body carries a `Cases:` list holding `ISSUE-1.a`,
      `ISSUE-1.a.1`, and `ISSUE-1.b`, and no other family's criteria (REQ-out-003). Note the
      depth-3 line reads `-   ISSUE-1.a.1 ...`, with the indent after the bullet, so a Markdown
      renderer shows it level with `ISSUE-1.a` rather than inside it (REQ-out-003, hi: ISSUE-3.a).
- [ ] `./target/release/hi issue not-an-id` exits 1 with `'not-an-id' is not a valid id: family
      'not' must start with A-Z and contain only A-Z, 0-9, _` (Error Cases).
- [ ] `./target/release/hi issue NOPE-1` and `./target/release/hi issue EXPORT-9` both print
      `<id> does not exist` and exit 1. There are only two distinct messages on the lookup path: an
      unknown family and an unknown number inside a known family are not told apart (Error Cases).
- [ ] `./target/release/hi issue EXPORT-1 --repo owner/name` without `--create` prints the
      ticket exactly as the bare command does; the flag is accepted and ignored (Error Cases).
- [ ] `./target/release/hi issue EXPORT-1 --create` in a repository with `gh` authenticated: a real
      issue opens carrying the same title and body the print path produced (REQ-out-004). Verify
      once, then close the issue.
- [ ] `PATH= ./target/release/hi issue EXPORT-1 --create` reports that `gh` could not be run and
      names the GitHub CLI (REQ-out-004, Error Cases).
- [ ] `./target/release/hi export | jq '.scope, (.files | length), (.product != null)'` and the same
      for `export EXPORT` and `export generate`: the key set is identical across all three except
      `product`, which is present only at repo scope (REQ-out-006, REQ-out-008).
- [ ] `./target/release/hi export generate`, `export generate.md`, and `export hi/generate.md` all
      succeed and return the same one-file payload; only `.scope` differs, echoing what was typed.
      `export HI/generate.md` is refused, because the match is case-sensitive (REQ-out-007).
- [ ] `./target/release/hi export NOPE` exits 1 with `nothing matches 'NOPE'. Give a family like
      SEND, a file like chat, an id like SEND-1, or nothing at all for the whole repository`
      (REQ-out-009).
- [ ] `cp INTENT.md /tmp/intent.before && ./target/release/hi index && diff /tmp/intent.before
      INTENT.md`: the only changed lines are inside the `hi:index` markers (REQ-out-011,
      hi: INDEX-2).
- [ ] In a scratch directory with a `hi/` folder and no `INTENT.md`: `hi index` creates the starter
      file, its `#` heading is the directory name, and a second `hi index` is idempotent
      (REQ-out-011).
- [ ] In a scratch `INTENT.md`, delete the `<!-- /hi:index -->` line and run `hi index`. It exits 1
      naming both markers, and the file is unchanged (REQ-out-013, hi: INDEX-2.b).
- [ ] In a scratch `INTENT.md`, write a sentence quoting `<!-- hi:index -->` above the real marker
      pair and run `hi index`: the sentence survives and only the real block is rewritten
      (REQ-out-012, hi: INDEX-2.a).
- [ ] In a scratch repository, save `INTENT.md` with CRLF endings around a real marker pair and run
      `hi index` twice. The prose and the closing marker keep their `\r\n`, the block inside the
      markers is written with LF, and the second run leaves the file byte-identical (REQ-out-011).
- [ ] In a scratch repository, declare a hi file's families as a YAML block list (`families:` then
      `  - SEND`) and run `hi index` and `hi export`: both report `SEND` from the declaration
      rather than falling back to the families the ids happen to use (REQ-out-010).
- [ ] Put a fenced code block in a hi file containing an id-shaped line, then run
      `./target/release/hi ls`, `hi export`, and `hi index`. The fenced line appears in none of
      them and does not move the criterion count (REQ-out-014, hi: FILE-9).
- [ ] In a scratch `INTENT.md`, put `<!-- hi:index -->` and `<!-- /hi:index -->` on lines of their
      own inside a ``` fence, above a real marker pair, and run `hi index`. It exits 0 and writes
      the generated list *inside the fence*, leaving the real block stale and replacing any prose
      between the two pairs. That is today's behavior and it contradicts hi: INDEX-2.a; see
      tasks.md before treating this checkbox as a pass (REQ-out-012).
- [ ] In a scratch repository, `chmod 444 INTENT.md` and run `hi index`. It succeeds, because the
      atomic rename needs a writable directory rather than a writable file, and the replacement
      comes back at the temporary file's mode (REQ-out-011).

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| `ls` over a workspace with no criteria | Prints `no criteria yet. Capture one with \`hi SEND-1 "..."\``, exits 0 |
| `ls --family X` where no file holds `X` | Prints only the hint line; no file headings and no blank lines |
| `ls` over a criterion whose id failed to parse | Rendered at depth 1 using its raw id; invisible under any `--family` filter |
| `issue` on a criterion with no cases | No `Cases:` section is emitted rather than an empty one |
| `issue` on a criterion whose cases go more than one level deep | Every descendant is listed with the depth indent before the bullet, so the source reads `  - **SEND-1.a.1**  ...` and a Markdown renderer nests it under its parent (hi: ISSUE-3.a) |
| `issue` on a criterion in a file with no `## Intent` | No `---` rule and no intent section |
| `issue` on a criterion whose sentence ends in `...` | All trailing periods are trimmed from the title; the body keeps the sentence verbatim |
| `issue` on a retired id | `<id> is retired, so it should not become work`, exits 1, and nothing is sent to `gh` even with `--create` |
| `issue --create` where `gh` exits non-zero | `gh issue create failed`; `gh`'s own diagnostics have already been printed through inherited stdio |
| `export` with a scope that is both a family and a file stem | Both match. The named file comes through whole, not just that family's slice, and every other file holding that family is also included, filtered to it |
| `export` with a scope naming a family that frontmatter declares but no criterion uses | Refused with `nothing matches '<scope>'. Give a family like SEND, a file like chat, an id like SEND-1, or nothing at all for the whole repository`; no file is selected, so a declared-but-unused family is invisible to `export` |
| `export` with a scope spelled as a path: `hi/chat.md`, `./hi/chat.md`, or `chat.md` | All select `hi/chat.md`, the same as the bare stem. `scope` in the payload is whatever was typed |
| `export` with a path that merely ends in `hi/<stem>.md`, such as `/anywhere/hi/chat.md` or `xhi/chat.md` | Selected: the path arm is a suffix test, not a path resolution. `HI/chat.md` and `hi\chat.md` are refused, because it is case-sensitive and forward-slash only |
| `export` where an active criterion carries a `retired:` continuation line | The entry stays in `criteria` and additionally carries a `retired` key holding the note; the key is not a retirement signal |
| `issue --repo owner/name` without `--create` | The flag is read only on the `--create` path, so it is silently ignored and the ticket is printed |
| `export` with a family scope over a file holding only that family's retired criteria | The file is included with an empty `criteria` list and a populated `retired` list |
| `export` with no scope over an empty workspace | `files: []`, exit 0, because incomplete intent is never an error |
| `export` with no scope where `INTENT.md` is missing or holds only the generated block | `product` is omitted entirely; no error |
| `index_block` for a file with exactly one criterion | `(1 criterion)`, singular |
| `index_block` for a file with no frontmatter families | Falls back to the families its ids actually use, then to `no families yet` |
| `write_index` where `INTENT.md` has only the opening marker, or the markers reversed | Refused: `<path> has an opening <!-- hi:index --> with no matching <!-- /hi:index -->. Fix the markers rather than have hi guess where the block ends`, exit 1, file untouched (hi: INDEX-2.b) |
| `write_index` where `INTENT.md` has only the *closing* marker | Not an error: no opening marker line exists, so the append branch runs and a `## Features` section carrying the block goes below the existing text, a duplicate heading if the file already had one |
| `write_index` where a marker appears inside a sentence rather than alone on a line | The sentence is prose. If a real marker pair exists further down it is the block; if not, the append branch runs (hi: INDEX-2.a) |
| `write_index` where the block is followed by a blank line and more prose | The blank line survives: the replaced span stops at the end of the closing marker's text, leaving that line's newline in the file |
| `write_index` over an `INTENT.md` written with CRLF endings | The markers are still recognized (`trim()` and `trim_end()` drop the `\r`), the prose and the closing marker's `\r\n` keep CRLF, and the generated block is written with LF. A second `hi index` changes nothing |
| `write_index` where a BOM sits immediately before the opening marker | The line is not a marker line (`U+FEFF` is not whitespace), so there is no splice and no refusal: the append branch runs and a second `## Features` appears. `read_product_intent` misses the same line, so the stale block reaches `product` |
| `index_block` or `export` over a file declaring its families as a YAML block list | Identical to the inline form: the parser fills `doc.front.families` from either, so the declared families are what the index line and the payload's `families` key carry |
| `ls`, `export`, or `index` over a file with an id-shaped line inside a fenced code block | The line is prose to the parser and appears nowhere in this module's output, and is not counted by `index_block` (hi: FILE-9) |
| `ls`, `export`, or `index` over a file with an id-shaped line outside every section | Invisible here: this module never reads `Doc::stray`. `hi check` reports it as `stray-criterion` (hi: CHECK-2.e) |
| `issue` given a zero-padded id such as `SEND-007` | Rejected at parse: `'SEND-007' is not a valid id: level '007' has a leading zero; write it as '7', so the id always means the same thing` (hi: ID-1.c) |
| `write_index` where `INTENT.md` is whitespace-only | Treated as absent: a full starter file is written |
| `refresh_index` where `write_index` would refuse or fail | `Some(<message>)` comes back, `INTENT.md` is untouched, and the caller exits 0 (hi: INDEX-4.a) |
| `refresh_index` where `INTENT.md` has no generated block at all | Nothing is written, `None` comes back and nothing is said. `Absent::LeaveAlone` skips the append branch, so a block somebody deleted stays deleted and `hi index` is how one is asked for (hi: INDEX-4.c) |
| `write_index` or `refresh_index` where `INTENT.md` cannot be read for any reason but absence | `reading <path>` wrapping the I/O error, nothing written, every byte of the file still there. Through `hi index` that is exit 1; through a capture it is a printed line and exit 0 (hi: INDEX-2.c, INDEX-4.a) |
| `index_note` over an `INTENT.md` whose block still carries CRLF inside it | Read as behind, because `hi index` would in fact rewrite it to LF. True rather than a false positive |
| `index_note` where `INTENT.md` is missing, unreadable, or has no marker line | `None`. There is no generated list to be behind; `check::product_intent_note` covers the missing file |
| `write_index` where `INTENT.md` is read-only but its directory is writable | Succeeds. `doc::write_atomically` renames a sibling temp over it, so the mode of the old file never matters; the replacement carries the temp file's permission bits |
| `write_index` where the directory holding `INTENT.md` is not writable, or the disk is full | Fails with `writing <path>` wrapping the I/O error, the temp file is removed, and `INTENT.md` keeps every byte it had |
| `write_index` where both markers sit alone on lines inside a fenced code block | The fenced pair is skipped and the real block below it is rewritten; the illustration is left byte-identical. Exit 0 (hi: INDEX-2.a) |
| `write_index` where two bare opening marker lines precede one closing marker | The first opening marker wins, so the prose between the two opening markers is inside the replaced span and is lost. Exit 0 |
| `export` at repo scope where `INTENT.md` is a freshly generated starter file | `product` is present and carries the `# <root>` heading, the unanswered `<!-- What is this product for ... -->` prompt, and the `## Features` heading. Only the generated block is stripped; `view::strip_comments` is not used here |
