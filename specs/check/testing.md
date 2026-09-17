---
spec: check.spec.md
---

## Automated Testing

This module's own tests are all inline `#[test]`s in the `#[cfg(test)]` module of `src/check.rs`.
They build a `Workspace` from in-memory strings through the local `workspace()` and `head()` helpers
and `Doc::parse`, so that suite touches no filesystem, which is itself the evidence for
REQ-check-008. The table below also lists the `tests/cli.rs` cases that drive `hi check` as a real
process, and the `src/doc.rs` cases that pin the parser guarantees `check`'s output rests on.

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/check.rs` (`a_clean_workspace_has_no_problems`) | Unit | REQ-check-001, REQ-check-009. A parent and its case, family declared: `report.ok()`, `criteria == 2`, `families == ["SEND"]`. |
| `src/check.rs` (`unfinished_intent_is_never_a_problem`) | Unit | REQ-check-001. One criterion with no spec, test, evidence or downstream work is clean. The comment in the test states the intent explicitly. |
| `src/check.rs` (`catches_duplicate_ids_across_files`) | Unit | REQ-check-002. `SEND-1` in `a.md` and `b.md`: exactly one problem, kind `DuplicateId`. |
| `src/check.rs` (`catches_a_case_with_no_parent`) | Unit | REQ-check-003. `SEND-1.a` with no `SEND-1`: one problem, kind `OrphanCase`, message contains `SEND-1`. |
| `src/check.rs` (`a_retired_criterion_may_keep_its_case_parent`) | Unit | REQ-check-003, REQ-check-004. Live `SEND-1` with `SEND-1.a` under `## Retired` is clean: the parent set includes retired ids, and a retired criterion is exempt from the retired-collision check. |
| `src/check.rs` (`catches_reuse_of_a_retired_id`) | Unit | REQ-check-004, REQ-check-002. `SEND-3` live and retired: the kind list contains both `RetiredCollision` and `DuplicateId`. |
| `src/check.rs` (`catches_a_malformed_id`) | Unit | REQ-check-005. `SEND-1.a.b` (a case of a case): exactly one problem, kind `UnparseableId`. It does not pin the `continue`, because that line is also a duplicate of nothing and an orphan of nothing, so the skip is unobserved; see the gap table below. |
| `src/check.rs` (`catches_an_undeclared_family`) | Unit | REQ-check-006. `OFFLINE-1` in a file declaring only `SEND`: a problem of kind `UndeclaredFamily` is present. |
| `src/check.rs` (`catches_a_criterion_stranded_outside_every_section`) | Unit | REQ-check-011. `SEND-9` below a `# Appendix` heading that closed `## Criteria`: exactly one problem, kind `StrayCriterion`, `id == "SEND-9"`. |
| `src/check.rs` (`prose_outside_a_section_is_not_a_stray_criterion`) | Unit | REQ-check-011. The same shape with ordinary prose below the heading is clean. Only an id-shaped leading token is a stray. |
| `src/check.rs` (whole `tests` module) | Unit | REQ-check-008. No test opens a file, spawns a process, or reaches the network; workspaces are built from strings. |
| `tests/cli.rs` (`check_is_clean_on_unfinished_intent`) | Integration | REQ-check-001. `hi check` on a repo holding one unfinished criterion exits 0 and prints `1 criterion`. |
| `tests/cli.rs` (`check_fails_on_a_structural_problem`) | Integration | REQ-check-003, REQ-check-007, REQ-check-009. An orphan `SEND-1.a` exits 1 and prints `orphan-case`, the relative path `hi/chat.md`, and line `8`. This is the only test that observes a problem's file and line. |
| `tests/cli.rs` (`check_fails_on_a_duplicate_id`) | Integration | REQ-check-002, REQ-check-009. `SEND-1` twice in one file exits 1 and prints `duplicate-id`. |
| `tests/cli.rs` (`block_style_frontmatter_is_understood_and_preserved`) | Integration | REQ-check-006. A YAML block `families:` list is read, so `SEND` is not falsely reported undeclared; after capture adds `RECEIPT`, `hi check` exits 0. |
| `tests/cli.rs` (`a_bom_does_not_make_a_valid_file_look_broken`) | Integration | REQ-check-001, REQ-check-009. A file opening with a UTF-8 BOM checks clean and counts `1 criterion`, rather than looking structurally wrong, since the BOM is stripped upstream in `Doc::parse`, not here. |
| `tests/cli.rs` (`outside_a_repository_it_says_so_rather_than_guessing`) | Integration | REQ-check-009. `hi check` in a directory with no `hi/` and no `.git` above it exits 1 from `Workspace::find`, before `run` is ever called. The test asserts on `not a repository`, from the message `this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so run it inside one`. That exit is not a `Report`. |
| `tests/cli.rs` (`a_repository_is_a_boundary_for_discovery`) | Integration | REQ-check-009. `hi check` inside a repository nested in another one exits 0 and prints `0 criteria`, because `Workspace::find` stops at the inner `.git` instead of adopting the outer repository's criteria. |
| `tests/cli.rs` (`a_wrongly_cased_id_is_reported_rather_than_read_as_prose`) | Integration | REQ-check-005. `- **send-2**  ...` next to a real criterion exits 1 and prints `unparseable-id` and `send-2`. This is the only test anywhere that observes `IdError::BadFamily` reaching `check`. |
| `tests/cli.rs` (`an_ordinary_hyphenated_word_is_not_mistaken_for_an_id`) | Integration | REQ-check-005, REQ-check-011. A line of prose reading `spec-sync and well-formed are ordinary words.` inside `## Criteria` exits 0: `looks_like_id` needs a first level starting with a digit, so a hyphenated word is prose. |
| `src/doc.rs` (`a_fenced_block_in_intent_is_not_parsed_as_criteria`, `a_tilde_fence_is_honored_too`) | Unit | REQ-check-012. A fenced `SEND-9` produces no criterion and leaves `doc.stray` empty, which is the whole mechanism behind the guarantee; `check` has no fence logic to test. |

Run them with `cargo test check::` (or `fledge run test` for the full suite). The integration rows
run under `cargo test --test cli`.

**Not yet covered by an automated test**

| Requirement | Gap |
|-------------|-----|
| REQ-check-005 (the skip half) | No test gives a malformed id that would also have been a duplicate or an orphan, so the `continue` that abandons the criterion after one problem is never observed. |
| REQ-check-005 (`PaddedLevel`) | `a_zero_padded_id_is_refused_rather_than_silently_renamed` covers capture's refusal, not `check`'s. No test puts `SEND-007` in a file and asserts an `unparseable-id` finding. |
| REQ-check-011 (indented strays) | No test anywhere puts a stray at an indent. Since criteria render as nested list items, a real stranded case is indented, which is the shape most likely to appear in a user's file and the one nothing pins. |
| REQ-check-011 (the stray token) | Nothing asserts what a bulleted, bolded stray reports as its `id`. It is `**SEND-9**`, emphasis included, because `doc` strips the bullet but not the emphasis before recording it, unlike `Criterion::raw_id`. |
| REQ-check-007 (sort order) | `check_fails_on_a_structural_problem` asserts the file and line reach the output, but no test has enough problems to observe the file-then-line sort. No unit test reads `problem.file` or `problem.line` directly. |
| REQ-check-010 | Nothing asserts the serialized JSON shape, nor that `Kind::code()` and the serde kebab-case rename produce the same six strings. |
| REQ-check-011 (the `## Intent` exception) | No test anywhere puts an unfenced id-shaped line under `## Intent` and asserts it is neither a criterion nor a stray. `doc::tests::a_fenced_block_in_intent_is_not_parsed_as_criteria` covers only the fenced case, where the fence would have been enough on its own. |

## Manual Testing

- [ ] Run `hi check` in this repository. Expect exit 0 and a summary line of the form
      `N criteria · M families · K files`, with no problem lines.
- [ ] Add `SEND-1.a` to a scratch `hi/*.md` with no `SEND-1`, run `hi check`, and confirm one
      `orphan-case` line under the file header, the summary, `1 problem`, and exit 1.
- [ ] Confirm the printed line number matches what the editor shows for that line (1-based, not 0).
- [ ] Confirm the printed path is relative to the repository root, with no machine-specific prefix.
- [ ] Run `hi check --json` on the same tree and confirm the problem kinds are the kebab-case codes
      and that the JSON lists exactly the problems the text listed.
- [ ] Run `hi check` with the network disabled and confirm it behaves identically (hi: CHECK-4).
- [ ] Add a criterion that is obviously unfinished, untestable, and implemented by nothing; confirm
      `hi check` still exits 0 (hi: CHECK-1).
- [ ] Put a `# Appendix` heading above the last criterion in a scratch file and confirm `hi check`
      reports it as `stray-criterion` rather than silently dropping it (hi: CHECK-2.e).
- [ ] Paste a fenced example of the hi format into a file's `## Intent` and confirm `hi check`
      reports nothing and the criteria count is unchanged (hi: FILE-9).
- [ ] Write `SEND-007` into a scratch file by hand and confirm `hi check` reports `unparseable-id`
      with the leading-zero message (hi: ID-1.c).
- [ ] Write an unfenced `SEND-9  ...` under `## Intent` and confirm `hi check` reports
      nothing: an intent section is prose, so the line is not a stray. Then move the same line under
      a `## Notes` heading and confirm it becomes `stray-criterion`.
- [ ] Indent that same `## Notes` line by two spaces and give it a bullet, `  - **SEND-9**  ...`,
      and confirm it is still one `stray-criterion` and that the id printed is `**SEND-9**`.
- [ ] Write `- **send-2**  ...` into a scratch file and confirm `hi check` reports `unparseable-id`
      with the `family 'send' must start with A-Z` reason, rather than silently reading the line as
      prose (hi: CHECK-2.d).
- [ ] Write a sentence containing `spec-sync` or `well-formed` into `## Criteria` and confirm
      `hi check` says nothing about it.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Workspace with zero documents | `ok()` is true; all counts are 0 and `families` is empty. |
| Document with frontmatter and no `## Criteria` section | No problems; no criteria are counted. |
| Top-level id such as `SEND-1` | `Id::parent()` is `None`, so the orphan check is skipped entirely. |
| Same id three or more times | Reported once per sighting after the first; every message names the same first file and line. |
| Id retired in one file and used actively in another | `RetiredCollision` names the file it was retired in, because the retired pass spans the whole workspace. |
| Duplicate id where one of the two lines is retired | Both problems can apply, but the retired-collision check fires only for the active line. |
| Malformed id that would also be a duplicate or an orphan | Only `UnparseableId` is reported; the `continue` prevents any second finding for that line. |
| Token with no hyphen (`SEND1`), nothing after the hyphen (`SEND-`), or a first level that does not start with a digit (`SEND-a`, `SE-ND-1`, `spec-sync`, `well-formed`) | `doc` never treats the line as a criterion, because `looks_like_id` requires a family that starts with an ASCII letter and continues in letters, digits and underscores, a hyphen, and a remainder beginning with a digit, so it is prose and produces no problem. `IdError::MissingHyphen` and `NoLevels` are therefore unreachable from `check`. |
| Wrongly-cased family (`send-2`, `Send-3`) | `unparseable-id` carrying `IdError::BadFamily`. `looks_like_id` is case-insensitive on the family on purpose, so a line plainly meant as a criterion is refused with a reason rather than read as prose and lost (hi: CHECK-2.d). `BadFamily` is reachable from `check`; it used to be filtered out. |
| Unparseable ids that do reach `check` | `send-2` (`BadFamily`), `SEND-1.` (`EmptyLevel`), `SEND-1.A` and `SEND-1.a1` (`BadLevel`), `SEND-007` (`PaddedLevel`), and `SEND-1.a.b` and `SEND-1.2` (`Alternation`). These five are the only `IdError` variants a criterion line can carry, and each appends its own `Display` text to the message after `': '`. |
| A single-digit `SEND-0` or an unpadded `SEND-7` | Parses fine; only a numeric level longer than one character that starts with `0` is `PaddedLevel`. |
| An id-shaped line inside a fenced code block | No criterion, no stray, no problem: `doc` skips fenced lines, and the fence stays in the intent prose. A tilde fence behaves identically, and a fence closes only on a run of the same character at least as wide as the one that opened it. |
| A `# ` heading between two criteria | The heading closes `## Criteria`, so every criterion below it becomes a `StrayCriterion` rather than silently disappearing. This is the common real-world cause of the kind. |
| An id-shaped line under a `## ` heading `doc` does not recognize (`## Notes`) | `StrayCriterion`. Only `## Criteria` and `## Retired` open a section, so any other `## ` heading leaves the same gap a `# ` heading does. |
| An id-shaped line between the frontmatter and the first heading | `StrayCriterion`. No section has been opened yet, so the line is stranded in the same way. |
| An *indented* id-shaped line outside every section (`  - **SEND-11**  ...`) | `StrayCriterion`, exactly as a flush-left line would be. `doc::parse_body` matches trimmed lines, because criteria render as nested list items and indentation is depth, not structure. |
| A stray written as a bulleted, bolded list item (`- **SEND-9**  ...`) | `StrayCriterion` whose `id` and message carry `**SEND-9**`, asterisks included: `doc` runs `strip_bullet` before recording the token but not `strip_emphasis`, and the token is never handed to `Id::parse`. A criterion on the same line would have carried `raw_id == "SEND-9"`. |
| An id-shaped line under `## Intent` | No criterion, no stray, no problem, at any indent. `Doc::parse_body` pushes every line of an intent section into the intent prose before it reaches the stray branch, so the line stays prose and reappears in `view` and `export`. The heading name is matched case-insensitively. |
| A `# ` heading after `## Intent` | Clears `in_intent` as well as the section, so id-shaped lines below it are strays again. |
| A file opening with a UTF-8 BOM | Checks exactly as if the BOM were absent; `Doc::parse` strips it, so the frontmatter is still found. |
| Retired criterion using an undeclared family | No `UndeclaredFamily`: the check is gated on `Section::Criteria`. |
| Several problems on one line | Each is a separate `Problem` with the same file and line; the sort is stable enough that their relative order follows discovery order. |
| Documents loaded in a different order | The same problems are *found* (the retired pass runs first, and duplicate detection spans the workspace), but a duplicate is reported at whichever sighting comes second, so swapping two files swaps which line is flagged and which the message names. `Workspace::load` sorts the paths it reads, so a real run is deterministic. |
| Continuation lines under a criterion | Never checked; only the id-bearing line is a criterion, and its `line_no()` is the reported location. An indented line ends the run and becomes its own criterion the moment it is itself id-shaped, so a nested case is never swallowed into its parent's sentence. |
| A repository nested inside another, with no `hi/` of its own | `0 criteria · 0 families · 0 files`, exit 0. `Workspace::find` stops at the inner `.git`, so `run` is handed an empty workspace rather than the outer repository's criteria. |

## Added 2026-09-16

| Requirement | Covered by |
|---|---|
| REQ-check-011 | `check::tests::catches_a_criterion_stranded_outside_every_section`, `check::tests::prose_outside_a_section_is_not_a_stray_criterion`, `doc::tests::records_a_criterion_stranded_outside_every_section`, `doc::tests::a_level_one_heading_closes_the_criteria_section` |
| REQ-check-013 (one lookup shared with capture) | `cli::a_criterion_in_a_file_hi_skips_is_reported_rather_than_vanishing` (the report), `cli::a_retired_id_in_a_file_hi_skips_is_never_handed_out_again` (the same id reported here and refused by capture in one test), `workspace::tests::an_id_in_a_file_hi_skips_is_still_taken` |
| REQ-check-012 | `doc::tests::a_fenced_block_in_intent_is_not_parsed_as_criteria`, `doc::tests::a_tilde_fence_is_honored_too`, `doc::tests::a_hash_comment_in_a_fenced_snippet_does_not_truncate_intent` |
| REQ-check-005 (`PaddedLevel` reaching `check`) | Not covered here. `cli::a_zero_padded_id_is_refused_rather_than_silently_renamed` and `id::tests` cover the id grammar; nothing asserts the `unparseable-id` finding. |
| REQ-check-011 (the `## Intent` exception) | Not covered anywhere. Verified by hand against the built binary on 2026-09-16: an unfenced `SEND-9` under `## Intent` produces no problem and no criterion, while the same line under `## Notes` produces `stray-criterion`. |
| REQ-check-005 (`BadFamily` reaching `check`) | `cli::a_wrongly_cased_id_is_reported_rather_than_read_as_prose`, plus `id::tests::recognises_id_shaped_tokens` and `id::tests::a_wrongly_cased_family_is_rejected_with_a_reason` for the grammar half. |
| REQ-check-005 (a hyphenated word is not an id) | `cli::an_ordinary_hyphenated_word_is_not_mistaken_for_an_id`, plus `id::tests::recognises_id_shaped_tokens`. |
| REQ-check-009 (the `.git` boundary) | `cli::a_repository_is_a_boundary_for_discovery`: an inner repository with no `hi/` checks clean at `0 criteria` instead of inheriting the outer one's. |
| REQ-check-011 (an indented stray, and the token's emphasis) | Not covered anywhere. Verified by hand against the built binary on 2026-09-16: `  - **SEND-11**  ...` below a `# ` heading reports one `stray-criterion` with the id `**SEND-11**`. |

The `doc::tests` rows are listed because the guarantee they pin is `check`'s output, but the
mechanism lives entirely in `doc`. `check` must not grow a second copy of it.

## Added 2026-09-17

| Requirement | Covered by |
|---|---|
| REQ-check-014 (the feature list is behind) | `cli::check_says_the_feature_list_is_behind_without_failing`: a criterion typed into `hi/chat.md` by hand makes `hi check` exit 0, print `feature list is behind`, and print no problem; `hi index` then clears the note. The note itself is covered on the `out` side by `out::tests::a_list_that_disagrees_is_a_note`, `a_list_that_matches_is_worth_no_note`, `a_list_hi_can_no_longer_refresh_is_a_note_too` and `a_file_with_no_block_at_all_is_not_a_list_that_is_behind`. |
| REQ-check-014 (it never moves the exit code) | The same test asserts `status.success()` and that stdout carries no `problem`. Checked by reverting: removing the `index_note` push fails it, and mutating `index_note` to nag unconditionally fails it too, so it cannot pass by the note never appearing or by it always appearing. |
