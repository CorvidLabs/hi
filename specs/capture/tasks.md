---
spec: capture.spec.md
---

## Tasks

- [x] Refuse an id that already exists and name the next free one. Evidence: `refuses_an_id_that_already_exists`.
- [x] Start a file for an unknown family with no prompt. Evidence: `creates_a_file_for_a_brand_new_family`.
- [x] Place a case directly under its parent. Evidence: `accepts_a_case_under_an_existing_parent`.
- [x] Refuse without writing on a malformed id. Evidence: `refuses_a_malformed_id_without_writing`.
- [x] Reconcile the parent check with `check`'s orphan rule, resolved on capture's side: the destination is now the file holding the parent (`parent_file.or_else(|| doc_for_family(..))`), so the line is always written where `check`'s per-file `present` will find its parent (hi: CAPTURE-4.a). Evidence: `a_case_lands_in_the_file_holding_its_parent`.
- [ ] Add a test for adopting an existing `hi/<stem>.md` that does not declare the family, asserting the file's prior contents survive.
- [ ] Add a capture-level test for adopting a file with no `## Criteria` heading, asserting the section is opened and the criterion lands under it (hi: CAPTURE-7). The behavior is tested in `specs/doc/`, not through `capture`.
- [ ] Decide whether `created_file` should be false when an existing file is adopted rather than written, and whether `create_file` should stop pushing a second `Doc` for a path `Workspace::load` already loaded.
- [ ] Decide whether a destination file that had no trailing newline should keep it that way. `Doc::to_text` ends every non-empty file with the detected line ending, so a capture adds one; the fix, if it is wanted, is a `doc` change (`self.trailing_newline || !out.is_empty()`), not a capture one.
- [ ] Decide what a case of a **retired** parent should do. Capture accepts it (the parent check spans `Doc::all()`), `Doc::insert`'s `insertion_point` does not find the parent (it scans `self.criteria` only), and `render_criterion` indents by depth regardless, so the line renders as a case of whichever active criterion is last and `hi check` reports the file clean. Either refuse it in `capture`, or teach `insertion_point` about `self.retired` in `doc`. Reproduced against the release binary; there is no criterion covering the choice either way.

## Gaps

- No test covers the adopt-an-existing-file branch of `create_file`, which is the only branch that can push a duplicate `Doc` onto `workspace.docs`.
- No *unit* test covers capturing into a workspace whose `hi/` directory does not exist; `capture::tests::temp_dir` and `cli::Repo::new` both create it, so `capture`'s own `fs::create_dir_all` is never exercised. The path itself is covered through the binary by `cli::concurrent_captures_into_a_repository_with_no_hi_directory_all_land`, where `lock::acquire` creates the directory.
- No test covers a retired id being refused as already taken; only an active one is.
- No test asserts that a refused capture leaves a *new family file* unwritten; `refuses_a_malformed_id_without_writing` only checks the seeded file's contents.
- No test covers the `Doc::insert` / `Doc::save` failure path after a family file has been scaffolded.
- No test covers adopting a `hi/<stem>.md` that has no frontmatter, where `Doc::insert` refuses with `has no frontmatter` and the capture fails after the file was already adopted into `workspace.docs`.
- No test injects a write failure, so the `write_atomically` recovery path (temp file removed, original intact; hi: FILE-8) is only exercised on its success path.
- No capture test starts from a file without a `## Criteria` heading, so `Doc::insert`'s section-creation branch (hi: CAPTURE-7) is covered only from `specs/doc/`.
- No test captures a criterion into a file whose parent lives in `## Retired`, so nothing catches the line rendering as a case of an unrelated active criterion. The behavior is reproduced by hand in `testing.md` → Edge Cases, and the open decision is in Tasks above.
- No test captures into a CRLF file, so line-ending preservation through a capture (hi: FILE-10) is covered only at the `doc` level.
- No test asserts that an id written only inside a fenced code block is not treated as taken (hi: FILE-9), nor that a stray criterion-shaped line is not (hi: CHECK-2.e).
- No test captures into a file that has no trailing newline, so nothing guards the one place a capture changes a byte outside the line it inserted: `Doc::to_text` adding that newline.
- No capture test exercises a destination file with a `# ` heading below its criteria block, so the append point that heading now records (a new family's block landing at the bottom of `## Criteria` rather than the top) is covered only from `specs/doc/`.
- No *unit* test asserts the rendered list form. `appends_to_an_existing_family` and `creates_a_file_for_a_brand_new_family` assert on `**ID**  sentence`, which would still pass if the bullet or the depth indent were dropped. Only `cli::a_hi_file_renders_as_a_list_not_a_wall_of_text` compares a whole line, and only for depths 1 and 2 (hi: FILE-1.b, FILE-1.c).

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
