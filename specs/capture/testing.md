---
spec: capture.spec.md
---

## Automated Testing

Unit coverage is the inline `#[cfg(test)] mod tests` at the bottom of `src/capture.rs`: eight tests, each against a real temporary directory. Behind it, `tests/cli.rs` drives the release binary and covers what only a process has: argv routing, exit codes, and which stream output lands on. Run them with `cargo test capture::` and `cargo test --test cli` (or `fledge run test` for the full suite).

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/capture.rs` (`capture::tests`) | Unit, real filesystem under `std::env::temp_dir()` | The whole `capture` contract: append, new-family creation, parent-file resolution, and every refusal |
| `src/capture.rs` (`capture::tests::temp_dir`) | Fixture helper | Wipes and recreates `<tmp>/hi-capture-<name>/hi/` so each test starts from a clean tree |
| `src/capture.rs` (`capture::tests::seeded`) | Fixture helper | Writes `hi/chat.md` with `families: [SEND]` and a single `SEND-1  I hit enter.` criterion, then `Workspace::load`s it |
| `tests/cli.rs` | Integration, spawns `CARGO_BIN_EXE_hi` | Capture through the CLI: the id-shaped dispatch, `--root`, a non-UTF-8 argument, exit codes, stdout/stderr split, workspace discovery, and the on-disk result |
| `tests/cli.rs` (`Repo::new`, `Repo::with_chat`) | Fixture helpers | A throwaway `<tmp>/hi-cli-<name>/` with an empty `hi/`, or one seeded with a `hi/chat.md` holding `SEND-1` |

### Requirement Coverage

| Requirement | Covering Tests |
|-------------|----------------|
| REQ-capture-001 (append a new id to its family's file) | `appends_to_an_existing_family`; `cli::an_id_shaped_argument_captures`, `cli::a_long_sentence_stays_on_one_line` |
| REQ-capture-002 (a new family starts its own file, unprompted) | `creates_a_file_for_a_brand_new_family`; `cli::a_new_family_starts_its_own_file` |
| REQ-capture-003 (an existing id refuses with a next-free hint) | `refuses_an_id_that_already_exists`; `cli::an_existing_id_refuses_with_exit_1_and_writes_nothing` |
| REQ-capture-004 (a sub-id with no parent refuses) | `refuses_a_case_with_no_parent` |
| REQ-capture-005 (malformed id and empty sentence are rejected) | `refuses_a_malformed_id_without_writing`, `refuses_an_empty_sentence`; `cli::a_malformed_id_refuses_without_writing`, `cli::a_zero_padded_id_is_refused_rather_than_silently_renamed` (hi: ID-1.c) |
| REQ-capture-006 (a refusal writes nothing) | `refuses_a_malformed_id_without_writing` (asserts neither sentence reaches the file); `refuses_an_id_that_already_exists`, `refuses_a_case_with_no_parent`, `refuses_an_empty_sentence` (return `Err` before any write path is reached); `cli::an_existing_id_refuses_with_exit_1_and_writes_nothing` and `cli::a_zero_padded_id_is_refused_rather_than_silently_renamed` compare the file byte-for-byte against a snapshot taken before the run |
| REQ-capture-007 (a case lands under its parent) | `accepts_a_case_under_an_existing_parent` |
| REQ-capture-008 (no init step required) | **Partly covered.** `creates_a_file_for_a_brand_new_family` exercises `create_file` and the family-file scaffold, but the `temp_dir` fixture always creates `hi/` first, so the `fs::create_dir_all` branch is never taken by any test. `cli::a_hindi_locale_directory_is_not_mistaken_for_a_workspace` covers the discovery side (hi: CAPTURE-6): a `hi/` directory holding no hi file is walked past rather than adopted. The missing-`hi/` path is manual-only today (see Manual Testing, and `tasks.md` → Gaps) |
| REQ-capture-009 (outcome is reported as data) | `appends_to_an_existing_family` (`done.id`, `done.created_file`), `creates_a_file_for_a_brand_new_family` (`done.created_file`, `done.file == "hi/billing.md"`), `a_case_lands_in_the_file_holding_its_parent` (`done.file`); `cli::an_id_shaped_argument_captures` and `cli::a_new_family_starts_its_own_file` assert the printed `+<id>` and `created` lines |
| REQ-capture-010 (a case lands in the file holding its parent) | `a_case_lands_in_the_file_holding_its_parent` |
| REQ-capture-011 (a missing `## Criteria` section is created) | **Not covered here.** No capture test starts from a file without a `## Criteria` heading; the behavior lives in `Doc::insert` and is covered by `specs/doc/` |
| REQ-capture-012 (the write cannot damage the destination) | **Not covered by a failure-injection test.** `write_atomically` is exercised by every passing capture test on its success path; no test simulates a full disk or a failed rename |
| REQ-capture-013 (the file comes back in the person's own style) | `cli::block_style_frontmatter_is_understood_and_preserved` (a block `families:` list survives a capture that has to add a second family, and is not collapsed to inline; hi: FILE-7); line endings and the BOM are covered in `specs/doc/`, and `cli::a_bom_does_not_make_a_valid_file_look_broken` covers the read side only. **Not fully covered:** nothing tests a destination file whose last line has no trailing newline, which gains one (see Edge Cases) |

### What Each Test Asserts

| Test | Assertions |
|------|------------|
| `appends_to_an_existing_family` | `done.id.to_string() == "SEND-2"`, `!done.created_file`, and `hi/chat.md` contains `SEND-2  It reaches them.` |
| `creates_a_file_for_a_brand_new_family` | `done.created_file`, `done.file == "hi/billing.md"`, and the new file contains `families: [BILLING]`, `# Billing`, and `BILLING-1  I can see what I paid.` |
| `refuses_an_id_that_already_exists` | The error message contains `already exists` and `next free is SEND-2` |
| `refuses_a_case_with_no_parent` | Capturing `SEND-4.a` errors with `needs a parent SEND-4` |
| `refuses_a_malformed_id_without_writing` | `SEND-1.a.b` (broken alternation) and `send-1` (lowercase family) both error, and neither sentence appears in `hi/chat.md` afterwards |
| `refuses_an_empty_sentence` | `capture(.., "SEND-2", "   ")` is an error |
| `a_case_lands_in_the_file_holding_its_parent` | With `hi/decl.md` and `hi/real.md` both declaring `families: [SEND]` and only `hi/real.md` holding `SEND-1`, capturing `SEND-1.a` gives `done.file == "hi/real.md"`, `hi/real.md` contains `SEND-1.a`, and `hi/decl.md` does not |
| `accepts_a_case_under_an_existing_parent` | The byte offset of `SEND-1  ` in the file is less than the offset of `SEND-1.a  `, i.e. the case follows its parent |

### What Each Integration Test Asserts

| Test (`tests/cli.rs`) | Assertions |
|------|------------|
| `an_id_shaped_argument_captures` | Exit 0, stdout contains `+SEND-2`, and `hi/chat.md` gains `SEND-2  it reaches them and the mark changes to sent` |
| `a_long_sentence_stays_on_one_line` | The captured line equals `SEND-2  <the whole sentence>` exactly, on one line, with nothing wrapped or truncated (hi: FILE-6) |
| `a_new_family_starts_its_own_file` | Exit 0, stdout contains `created`, and `hi/billing.md` holds `families: [BILLING]` and the criterion |
| `an_existing_id_refuses_with_exit_1_and_writes_nothing` | Exit code 1, stderr contains `already exists` and `next free is SEND-2`, stdout is empty, and `hi/chat.md` equals its pre-run bytes |
| `a_malformed_id_refuses_without_writing` | `SEND-1.a.b` exits 1 and `hi/chat.md` equals its pre-run bytes |
| `a_zero_padded_id_is_refused_rather_than_silently_renamed` | `SEND-007` exits 1, stderr contains `leading zero`, and the file is unchanged (hi: ID-1.c) |
| `block_style_frontmatter_is_understood_and_preserved` | `check` does not report `SEND` as undeclared from a block list; capturing `RECEIPT-2` succeeds, the file still contains `  - SEND` and now `  - RECEIPT`, never `families: [`, and `check` passes afterwards (hi: FILE-7) |
| `a_hindi_locale_directory_is_not_mistaken_for_a_workspace` | Run from `public/locales`, where `public/locales/hi/common.md` has no `hi:` frontmatter: capture succeeds against the real workspace above it and creates nothing inside the locale directory (hi: CAPTURE-6) |
| `root_is_honored_by_capture_and_not_swallowed_into_the_sentence` | Run from another directory, both `--root PATH` and `--root=PATH` capture into the named repository, and the string `--root` never appears in the file (hi: CAPTURE-8) |
| `a_non_utf8_argument_is_reported_not_panicked` | A sentence argument of invalid UTF-8 exits 1, stderr does not contain `panicked`, and stderr contains `not valid UTF-8` (hi: CAPTURE-1.c) |
| `bare_invocation_prints_help_and_writes_nothing` | With no arguments, help is printed and `hi/` is still empty. Nothing is created by an invocation that captures nothing |

## Manual Testing

- [ ] In a repository with no `hi/` directory, run `hi NEWTHING-1 "a first thought"` and confirm `hi/newthing.md` is created and `NEWTHING-1` is in it, with no prompt and no init step (hi: CAPTURE-1, CAPTURE-1.a, CAPTURE-2.a).
- [ ] Run the same capture twice and confirm the second run prints `error: NEWTHING-1 already exists in hi/newthing.md:<line>` plus `hint:  next free is NEWTHING-2`, exits 1, and leaves the file unchanged (hi: CAPTURE-3, CAPTURE-5).
- [ ] Capture `NEWTHING-1.a` and confirm the line lands immediately under `NEWTHING-1` rather than at the end of `## Criteria` (hi: CAPTURE-4).
- [ ] Capture with an unquoted sentence (`hi NEWTHING-2 it reaches them`) and confirm the words are joined into one criterion.
- [ ] Capture a sentence far past terminal width and confirm it is written as a single unwrapped line, with no continuation line, no truncation, and no word changed (hi: FILE-6).
- [ ] Capture a sentence containing runs of spaces or newlines and confirm they collapse to single spaces while every word survives in order.
- [ ] Run `hi check` after a session of captures and confirm it reports zero structural problems. Capture must never author something the checker rejects.
- [ ] Confirm capture never blocks waiting for input: run it with stdin closed (`hi SEND-9 "x" < /dev/null`) (hi: CAPTURE-1.b).
- [ ] Capture into a hand-written `hi/*.md` that has frontmatter and prose but no `## Criteria` heading, and confirm hi opens the section and puts the criterion under it rather than appending to the prose (hi: CAPTURE-7).
- [ ] Capture into a file saved with CRLF line endings and confirm the whole file is still CRLF afterwards and the diff is one line (hi: FILE-10, FILE-4.a).
- [ ] Capture into a file whose `## Intent` shows the hi format inside a fenced code block, and confirm the ids in the fence are neither counted as taken nor reported by `hi check` (hi: FILE-9).
- [ ] Capture into a file saved without a final newline and confirm the only differences are the new line and that added final newline. hi does not currently round-trip a missing trailing newline.
- [ ] Capture the first criterion of a second declared family into a file that has a `# ` heading and prose below its criteria block, and confirm the new block lands at the bottom of `## Criteria` rather than at the top.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Id is malformed and the sentence is also empty | Reported as a bad id, since id validation runs first |
| Id exists in `## Retired` rather than `## Criteria` | Refused as already existing; `Workspace::find_id` walks active and retired alike |
| Rejected id is nested, e.g. `SEND-2.a` already exists | The hint is still a top-level id (`SEND-3`), never `SEND-2.b` |
| Family exists only by use, with no frontmatter declaration | `doc_for_family` finds it by use; the criterion joins that file and `created_file` is `false` |
| `hi/<stem>.md` exists but neither declares nor uses the family | The file is adopted and inserted into, never overwritten; `created_file` is still reported `true` and a duplicate `Doc` for that path is pushed onto `workspace.docs` (harmless for a single capture per process) |
| Family contains underscores, e.g. `TWO_FACTOR` | File is `hi/two-factor.md`, title is `# Two factor` |
| Sentence has leading or trailing whitespace | Trimmed by capture; `render_criterion` then collapses interior whitespace runs to single spaces. No word is altered, added or dropped |
| Sentence is very long, or is one very long word | Written as one unwrapped line regardless of length; `render_criterion` has no wrap width and never truncates (hi: FILE-6) |
| Parent exists, but in a different file from the one `doc_for_family` would resolve to | The case goes to the parent's file. `parent_file` is computed from `Workspace::find_id(&parent)` and takes precedence over `doc_for_family`, so `hi check` sees the parent in the same file and reports nothing. Covered by `a_case_lands_in_the_file_holding_its_parent` (hi: CAPTURE-4.a) |
| Parent exists only in `## Retired` | Accepted, because `Workspace::find_id` spans `Doc::all()`. The destination is that file, and `check`'s per-file `present` also comes from `doc.all()`, so there is no `orphan-case`. `Doc::insert`'s `insertion_point` scans only `self.criteria`, so the new line joins the end of the family's *active* block rather than sitting beneath the retired parent |
| Id appears only inside a fenced code block | Not taken. The parser treats a fence as opaque prose (hi: FILE-9), so the id is in neither `criteria` nor `retired` and capture writes a second, real one |
| Id appears only on a criterion-shaped line outside every section | Not taken. It is in `Doc::stray`, which `Doc::all()` does not chain; `hi check` reports it separately as `stray-criterion` (hi: CHECK-2.e) |
| Destination file has no `## Criteria` heading | `Doc::insert` splices a blank line, `## Criteria` and a blank line in after the last content line and inserts under it (hi: CAPTURE-7) |
| Destination file has a `# `-level heading and prose below its criteria block | The `# ` heading closes the section and records the append point at the end of that block, so a criterion for a family the file declares but has not used opens its block at the bottom of `## Criteria`, above the heading rather than at the top of the block. Covered from `specs/doc/` by `a_level_one_heading_closes_the_criteria_section`; no capture test exercises it |
| Adopted `hi/<stem>.md` has no frontmatter block | `Doc::insert` refuses with `<path> has no frontmatter` and says to add one; the file is left exactly as it was and nothing is saved. The in-memory `Doc` has already been edited by then, but `save` is never reached |
| Destination file uses CRLF, or carries a BOM | `Doc::to_text` rejoins with the detected ending, so a CRLF file stays CRLF (hi: FILE-10). A leading BOM is stripped at parse time and not written back, so the saved file has none (hi: FILE-11), and `holds_hi_files` strips one as well, so a BOM does not hide the workspace from discovery |
| Destination file's last line has no trailing newline | The saved file gains one. `Doc::to_text` appends the detected line ending to any non-empty output, whatever `Doc::trailing_newline` says, so that final byte changes in addition to the inserted line. It is the one documented exception to hi: FILE-4.a's byte-for-byte promise, and nothing guards it |
| Destination file declares `families:` as a YAML block list | The new family joins it as another `  - NAME` item; the list is never collapsed to inline form (hi: FILE-7). Covered by `cli::block_style_frontmatter_is_understood_and_preserved` |
| Id carries a zero-padded number, e.g. `SEND-007` | Refused before anything is read, with `IdError::PaddedLevel`, whose message contains `leading zero` (hi: ID-1.c) |
| `hi/` exists but is empty | `Workspace::find` will not adopt it, because `holds_hi_files` finds no `.md` with `hi:` frontmatter (hi: CAPTURE-6); discovery keeps walking up and falls back to the `.git` root, which is normally the same directory. The first capture then creates the family file inside the empty `hi/` and `created_file` is `true`. With no `.git` anywhere above, discovery fails first with `no hi/ directory found`, followed by `run this inside a repository` |
| First level of the id is not `1` on a brand-new family, e.g. `BILLING-7` | Accepted. Capture does not require families to start at 1, and `next_free` only ever moves upward |
| Two branches append the same next number to one family | Not detectable at capture time on separate branches; the collision surfaces as a duplicate-id error in `hi check` after the merge (DECISIONS.md §4) |
| `Doc::save` fails partway through the write | Nothing is lost: `write_atomically` writes a `.<name>.hi-tmp` sibling, flushes and syncs it, and only then renames over the target, removing the temp file on any failure (hi: FILE-8). No test injects the failure |
| `Doc::insert` or `Doc::save` fails after a new family file was scaffolded | The error propagates; the scaffolded, criterion-less file remains on disk, since `create_file`'s `fs::write` has already run. No test covers this path |
