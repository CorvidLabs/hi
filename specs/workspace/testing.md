---
spec: workspace.spec.md
---

## Automated Testing

`src/workspace.rs` has **no inline `#[cfg(test)]` module**. Its behavior is exercised from two
directions. The sibling modules' tests cover the lookups: `capture::tests` is the only unit place
that builds a workspace from a real directory via `Workspace::load`, while `check::tests`,
`out::tests` and `view::tests` construct `Workspace` directly from literal fields. `tests/cli.rs`
drives the real binary and is the only thing that executes `Workspace::find` at all, and because
its `Repo` helper creates `hi/` but never a `.git`, every subcommand it runs depends on
`holds_hi_files` recognizing the directory, so discovery is load-bearing in most of that file.

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `tests/cli.rs` | Integration, real binary | `Workspace::find`'s discovery walk. `a_hindi_locale_directory_is_not_mistaken_for_a_workspace` is the direct test of REQ-workspace-009: it runs from `public/locales`, where `public/locales/hi/common.md` has no frontmatter, and asserts the capture landed in the repo's real `hi/chat.md` and that nothing was created inside the locale directory. `a_bom_does_not_make_a_valid_file_look_broken` covers the BOM strip in `holds_hi_files`. The repo has no `.git`, so a BOM that defeated recognition would fail the run with `no hi/ directory found` instead of reporting `1 criterion`. `outside_a_repository_it_says_so_rather_than_guessing` covers the exhausted walk: a temp directory with neither `hi/` nor `.git`, exit 1, `no hi/ directory found`. `root_is_honored_by_capture_and_not_swallowed_into_the_sentence` runs from an unrelated directory and covers `--root` reaching `find` as a starting point through both `--root PATH` and `--root=PATH` |
| `tests/cli.rs` | Integration, real binary | `Workspace::load` on a real directory, indirectly but in every test that runs a subcommand: `check_is_clean_on_unfinished_intent`, `export_stdout_is_parseable_json`, `issue_prints_a_ticket_carrying_the_id`, `view_writes_a_self_contained_page`, `index_rewrites_only_the_generated_block`, `check_fails_on_a_duplicate_id`, `check_fails_on_a_structural_problem`, `block_style_frontmatter_is_understood_and_preserved`, and the capture tests such as `an_id_shaped_argument_captures` and `a_new_family_starts_its_own_file`. `bare_invocation_prints_help_and_writes_nothing` and `an_unknown_subcommand_is_a_usage_error` do **not**: clap exits before `find` is reached |
| `src/capture.rs` (`capture::tests`) | Unit, real filesystem | The `seeded()` helper calls `Workspace::load` on a temp directory containing `hi/chat.md`, so every test in this module exercises REQ-workspace-002: `appends_to_an_existing_family`, `creates_a_file_for_a_brand_new_family`, `refuses_an_id_that_already_exists`, `refuses_a_case_with_no_parent`, `refuses_a_malformed_id_without_writing`, `refuses_an_empty_sentence`, `accepts_a_case_under_an_existing_parent` |
| `src/capture.rs` (`capture::tests`) | Unit | `doc_for_family` both ways: `appends_to_an_existing_family` (declared family resolves to `hi/chat.md`), `creates_a_file_for_a_brand_new_family` (unknown family returns `None`, so a file is created) |
| `src/capture.rs` (`capture::tests`) | Unit | `find_id` and `next_free`: `refuses_an_id_that_already_exists` asserts both the `already exists` message and `next free is SEND-2`; `refuses_a_case_with_no_parent` and `accepts_a_case_under_an_existing_parent` cover the parent lookup. `a_case_lands_in_the_file_holding_its_parent` covers `find_id` used to pick the destination *file*, with two files sharing a family and the case following its parent rather than the declaration (hi: CAPTURE-4.a) |
| `src/capture.rs` (`capture::tests`) | Unit | `rel`: `creates_a_file_for_a_brand_new_family` asserts `done.file == "hi/billing.md"`, the root-relative form |
| `src/check.rs` (`check::tests`) | Unit, in-memory workspace | `criteria_count` and `families` through the report: `a_clean_workspace_has_no_problems` asserts `report.criteria == 2` and `report.families == ["SEND"]`; `catches_duplicate_ids_across_files` runs the same lookups across two docs |
| `src/out.rs` (`out::tests`) | Unit, in-memory workspace | `find_id` and `rel`: `issue_body_carries_the_id_and_its_cases` resolves `SEND-1` through `workspace.find_id`; `export_scoped_to_a_family_keeps_only_that_family` is the one export test that asserts the value `rel` produced (`value["files"][0]["file"] == "hi/chat.md"`). `export_of_the_whole_repo_includes_every_file` and `export_scoped_to_a_file_keeps_that_file` run `rel` but assert only counts, criteria and scope, so they execute it without checking it |
| `tests/cli.rs` | Integration, real binary | `doc_for_family`'s resolve-by-use fallback and a block-style declaration: `block_style_frontmatter_is_understood_and_preserved` writes a file whose `families:` is a YAML block list naming only `SEND` while `RECEIPT-1` is used but undeclared, then captures `RECEIPT-2`. The declaration pass misses, the use pass matches, and the criterion lands in `hi/chat.md` rather than in a newly created `hi/receipt.md` (hi: FILE-2) |
| `src/out.rs` (`out::tests`) | Unit | `families` as a scope resolver: `export_scoped_to_a_family_keeps_only_that_family` (a name in `families()` is treated as a family) and `export_rejects_an_unknown_scope` (a name in neither `families()` nor the file stems is an error) |
| `src/view.rs` (`view::tests`) | Unit, in-memory workspace | `docs` and `criteria_count` driving the rendered page: `page_carries_intent_prose_and_every_criterion` and `retired_criteria_are_tucked_away_but_present` both call `render` over a literal `Workspace` |

### Requirement Coverage

| Requirement | Covering Tests | Status |
|-------------|----------------|--------|
| REQ-workspace-001 (`find`: ancestor walk, `.git` fallback, not-found error) | `tests/cli.rs`: `a_hindi_locale_directory_is_not_mistaken_for_a_workspace` (the walk climbs past an unrecognized `hi/`), `outside_a_repository_it_says_so_rather_than_guessing` (exhausted walk, exact message, exit 1), `root_is_honored_by_capture_and_not_swallowed_into_the_sentence` (`--root` as the starting point), plus every other cli test as a smoke test of the found-it path | Partial. The walk and the error message are covered end to end. Untested: the `.git` fallback itself (no cli fixture creates a `.git`, so the first-capture-in-a-fresh-repo path ships unverified), nearest-`hi/`-wins across two nested workspaces, and a `.git` file rather than a directory. No unit test calls `Workspace::find` directly. |
| REQ-workspace-002 (`load`: `hi/*.md`, sorted, empty is valid) | `appends_to_an_existing_family`, `creates_a_file_for_a_brand_new_family`, `refuses_an_id_that_already_exists`, `refuses_a_case_with_no_parent`, `refuses_a_malformed_id_without_writing`, `refuses_an_empty_sentence`, `accepts_a_case_under_an_existing_parent` (all via `seeded()`), plus `a_case_lands_in_the_file_holding_its_parent`, which calls `Workspace::load` on a two-file `hi/` | Partial: every fixture but one holds a single file; the two-file case executes the sort without asserting it (`decl.md` and `real.md` are already in sorted order, and the test passes either way), and non-`.md` skipping, sub-directories and a missing `hi/` are untested |
| REQ-workspace-003 (read-only snapshot) | `refuses_a_malformed_id_without_writing` (re-reads `hi/chat.md` and asserts the refused sentences are absent); `tests/cli.rs`: `a_zero_padded_id_is_refused_rather_than_silently_renamed` and `an_existing_id_refuses_with_exit_1_and_writes_nothing` compare the file byte for byte before and after a refused run | Partial, asserted at the capture level, not against this module directly. `refuses_an_empty_sentence` asserts only that the call errors, not that the file is untouched |
| REQ-workspace-004 (`doc_for_family`) | `appends_to_an_existing_family`, `creates_a_file_for_a_brand_new_family`; `tests/cli.rs`: `block_style_frontmatter_is_understood_and_preserved` | Covered: the frontmatter path, the `None` path and the resolve-by-use fallback. The block test is the one that reaches the fallback: `hi/chat.md` declares only `SEND` (as a YAML block list) while `RECEIPT-1` is used but undeclared, so capturing `RECEIPT-2` resolves through the use pass and lands in `chat.md` instead of creating `hi/receipt.md`. No *unit* test reaches the fallback |
| REQ-workspace-005 (`find_id`, active and retired) | `refuses_an_id_that_already_exists`, `refuses_a_case_with_no_parent`, `accepts_a_case_under_an_existing_parent`, `a_case_lands_in_the_file_holding_its_parent`, `issue_body_carries_the_id_and_its_cases` | Partial. Retired-id lookup is not covered here; `check::tests::catches_reuse_of_a_retired_id` covers the equivalent rule inside `check` |
| REQ-workspace-006 (`next_free`) | `refuses_an_id_that_already_exists` (asserts `next free is SEND-2`) | Partial: the retired-number case and the unused-family `1` case are untested |
| REQ-workspace-007 (`families`) | `a_clean_workspace_has_no_problems`, `export_scoped_to_a_family_keeps_only_that_family`, `export_rejects_an_unknown_scope` | Partial. The multi-doc union and sort order are not asserted, and every fixture that asserts a family list declares it inline, so a block-list declaration reaching `families()` is unverified (`block_style_frontmatter_is_understood_and_preserved` proves `front.families` is filled from a block list, but asserts it through `check`'s per-doc test and through capture, never through `families()`) |
| REQ-workspace-008 (`rel`, `intent_path`) | `creates_a_file_for_a_brand_new_family` (`done.file == "hi/billing.md"`), `export_scoped_to_a_family_keeps_only_that_family` (`files[0].file == "hi/chat.md"`); `tests/cli.rs`: `index_rewrites_only_the_generated_block`, `index_leaves_a_marker_quoted_in_prose_alone`, `index_refuses_rather_than_guessing_when_a_marker_is_unclosed` | Covered. Those first two assert `rel`'s output, and the three index tests assert `intent_path` end to end: `out::write_index` reads and rewrites exactly the path it returns, so each test writing `INTENT.md` at the repo root and then reading its rewritten body proves the path resolved to `<root>/INTENT.md`. The unit-level export tests only *execute* it. `export_of_the_whole_repo_includes_every_file` reaches it through `read_product_intent`, and against the fake `/r` root the read fails, so `product` is `None` and nothing is asserted there |
| REQ-workspace-009 (recognition by `hi:` frontmatter, BOM stripped) | `tests/cli.rs`: `a_hindi_locale_directory_is_not_mistaken_for_a_workspace`, `a_bom_does_not_make_a_valid_file_look_broken` | Partial: the two headline cases are covered through the binary. `holds_hi_files` has no direct unit test, so the unreadable-file, non-UTF-8 and `hi`-is-a-file branches are unverified, as is the block-style `hi:` key inside a longer frontmatter |

### Commands

| Scope | Command |
|-------|---------|
| Everything that touches this module | `cargo test` |
| The discovery walk, end to end through the binary | `cargo test --test cli` |
| The load path on a real directory | `cargo test capture::tests` |
| The counting and family lookups | `cargo test check::tests` |
| The id lookup and path rendering | `cargo test out::tests` |
| The doc iteration and criteria count behind the HTML page | `cargo test view::tests` |

## Manual Testing

- [ ] From a deep subdirectory of this repo, run `./target/release/hi check` and confirm it resolves the repo's `hi/` rather than failing (REQ-workspace-001).
- [ ] In a temp directory that is a git repo with no `hi/`, run `hi DEMO-1 "a first criterion"` and confirm `hi/demo.md` is created. This is the `.git` fallback working after the walk finds nothing (REQ-workspace-001, REQ-workspace-002).
- [ ] In a temp directory that is neither a repo nor inside one, run `hi check` and confirm the error reads `no hi/ directory found; run this inside a repository` and the exit code is 1.
- [ ] Make a `hi/` directory holding only a markdown file with no frontmatter, run `hi check` from beside it, and confirm hi ignores it and resolves the real workspace above, or fails rather than adopting it (REQ-workspace-009, hi: CAPTURE-6).
- [ ] Prepend a UTF-8 BOM to a hi file that is the only file in its `hi/`, run `hi check`, and confirm the workspace is still found and the criteria still counted (REQ-workspace-009, hi: FILE-11).
- [ ] Inside a git repo whose parent directory holds the real `hi/`, run `hi check` and confirm it resolves the parent's workspace; a `.git` entry no longer bounds the walk (REQ-workspace-001).
- [ ] Add a `hi/notes.txt` and a `hi/sub/extra.md` to a scratch workspace, run `hi check`, and confirm neither is counted (REQ-workspace-002).
- [ ] Remove the `families:` line from a hi file, run `hi ls` and `hi export <FAMILY>`, and confirm the family still resolves from the criteria themselves (REQ-workspace-004, REQ-workspace-007, hi: FILE-2).
- [ ] Run `hi export` twice and diff the output; the file order and family order must be byte-identical (REQ-workspace-002, REQ-workspace-007).
- [ ] Run `hi check` before and after, and confirm no file under `hi/` changed. This module writes nothing (REQ-workspace-003, hi: FILE-4.a).

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Starting directory does not exist or contains a broken symlink | `find` fails with `resolving <path>` and the underlying I/O error |
| No recognized `hi/` and no `.git` anywhere up to the filesystem root | `find` fails with `no hi/ directory found; run this inside a repository` |
| A nested project has its own `hi/` inside an outer repo | The nested `hi/` wins; the walk stops at the first ancestor holding a recognized one |
| A nearer ancestor has `.git` but no `hi/`, while a farther ancestor has a recognized `hi/` | The **farther `hi/` wins**. `.git` is remembered, not obeyed, and the walk continues past it. (This reverses the pre-fix behavior; do not re-introduce an early `return` on `.git`.) |
| One directory holds both a recognized `hi/` and `.git` | `hi/` is tested first within the level, so that directory loads its own `hi/` and the `.git` fallback is never consulted |
| Repo root has `.git` but no `hi/` | The walk runs to the filesystem root, then falls back: loads successfully with `docs == []` and `dir == <root>/hi`, so the first capture can create it |
| `.git` is a file rather than a directory (a git worktree or submodule) | Still marks a fallback root, because the check is `exists()`, not `is_dir()` |
| A directory named `hi` holding only non-hi markdown, such as a Hindi locale bundle | Not recognized. The walk climbs past it and nothing is ever written inside it (hi: CAPTURE-6) |
| A `hi/` holding one file whose frontmatter starts with a UTF-8 BOM | Recognized. `holds_hi_files` strips the BOM before reading the first line (hi: FILE-11) |
| A `hi/` holding one file that is not valid UTF-8, or that cannot be read | It does not count toward recognition, and the walk continues as if the directory were empty. Note that if some *other* file in that directory does qualify, the unreadable one still reaches `load`, where `Doc::load` fails the whole run |
| `hi` exists but is a file, not a directory | `read_dir` fails, recognition is `false`, and the walk moves up |
| `hi/` exists but is empty | Not recognized by `find`, so the walk climbs past it. `load` on that root is still a valid, empty workspace (zero docs, `criteria_count() == 0`, `families() == []`), which is what the `.git` fallback relies on |
| `hi/` contains non-markdown files or sub-directories | They are skipped; only regular `*.md` files directly inside are loaded |
| A file named `CHAT.MD` | Loaded, since the extension test is `eq_ignore_ascii_case` |
| `hi/` cannot be listed (permissions) | `load` fails with `reading <dir>` and the underlying error |
| One hi file cannot be read | The whole load fails with `Doc::load`'s `reading <path>` error; no partial workspace is returned |
| A directory entry cannot be read from the iterator | Skipped by `filter_map(entry.ok())`; the other files still load |
| An entry is listed but its metadata cannot be read | `path.is_file()` is `false` on the error, so it is skipped like a directory; the other files still load |
| A hi file with a malformed id, a duplicate, or an orphan case | Loads without error; the problems belong to `hi check`, not to loading |
| A hi file that shows the criterion format inside a fenced code block | The fenced lines are prose, so they never become criteria. `families()`, `next_free()` and `find_id` cannot see them (hi: FILE-9) |
| A criterion-shaped line sitting outside every section | It lands in `Doc::stray`, which `Doc::all()` does not chain, so no lookup here sees it. `check` reports it as `Kind::StrayCriterion` (hi: CHECK-2.e) |
| A file inside `hi/` with no `hi:` frontmatter line at all | Still loaded and still checked: recognition gates discovery, not loading |
| Two docs declare the same family | `doc_for_family` returns the earlier index and nothing reports the overlap. `check` compares ids, not the `families:` lists of different files |
| A family used by criteria but declared in no frontmatter | Resolves through the use fallback and appears in `families()` |
| A file whose `families:` is a YAML block list, or that uses the singular `family:` key | Resolves exactly as the inline form does. `doc` normalizes all three into `front.families`, so the declaration pass and `families()` never see which style the file used |
| A family whose only criteria are retired | Still resolves, still counts toward `next_free`, and is excluded from `criteria_count()` |
| `next_free` on a family whose highest id is retired | Returns that number plus one; a retired number is never reissued |
| `next_free` on an unused family | Returns `1` |
| `rel` on a path outside `root` | Returns the path's own display form, unchanged |
| An index held across a `docs.push` (capture creating a file) | Still valid: an append moves nothing, and `capture` uses the index `create_file` returned straight afterwards. Only a removal or a reorder would invalidate an index, and nothing in hi does either |
