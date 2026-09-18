---
spec: doc.spec.md
---

## Automated Testing

This module's unit tests live in the `#[cfg(test)] mod tests` block at the bottom of `src/doc.rs` and
run with `cargo test doc::` (35 of them). A second, repo-level suite in `tests/cli.rs` drives the
real binary and covers several of this module's requirements end to end; it is not owned by this
spec, but the rows below name the tests in it that a change to `doc` would break.

The unit tests use a `doc(raw: &str)` helper that calls `Doc::parse` against the fixed path
`hi/chat.md`, so all but one of them touch no filesystem. Most parse assertions run against one
`SAMPLE` constant: frontmatter with `hi: 1`, `families: [SEND, RECEIPT]` and `owner: leif`, a
`# Chat` title, a two-line `## Intent`, three criteria (`SEND-1` with a continuation, `SEND-1.a`,
`RECEIPT-1`), and a `## Retired` block holding `SEND-3` with a `retired:` note. The exception is
`a_normal_save_leaves_no_temp_file_behind`, which writes into `std::env::temp_dir()/hi-doc-save`
because it has to exercise `Doc::load` and `Doc::save` against a real directory.

The three read-back tests use a second constant, `DOCUMENTED`: a file hi did not write, hand-typed,
with a properly closed example of the format inside its own `## Intent` holding `## Criteria`,
`## Retired` and an id-shaped line. Every write-path test used to assert over files hi itself had
produced, and that is exactly how the two bugs REQ-doc-020 exists for got through
(DECISIONS.md §26, §31). `EXAMPLE` is that fenced block, quoted, so a test can assert it came back
byte-identical.

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/doc.rs` (`mod tests`) | Unit | Frontmatter in both YAML styles, intent prose, fenced-block opacity, criterion and continuation parsing, criteria read however they were decorated, retired handling, malformed-id recording, stray recording, all insertion-point branches, creating a missing `## Criteria` section, frontmatter family declaration, byte-identical round trip, BOM stripping, CRLF preservation, an atomic save that leaves no temp file, the one-line and nested-list-item render rules, and the new-file scaffold |
| `tests/cli.rs` | Integration (not owned here) | Drives the built binary against a throwaway repo. Relevant to `doc`: the captured line is one list item and a case is a nested one, a new family scaffolds a parseable file, a block-style frontmatter is understood and written back in its own style, a BOM'd file checks clean, a wrongly cased id is reported rather than read as prose, an ordinary hyphenated word is not mistaken for an id, and a refusal leaves the file byte-identical on disk |

### Requirement Coverage

| Requirement | Tests |
|-------------|-------|
| REQ-doc-001 (parse as ordinary markdown) | `parses_frontmatter`, `parses_intent_prose`, `reads_block_style_families`, `a_byte_order_mark_does_not_hide_the_frontmatter`, `new_file_text_parses_as_an_empty_doc` |
| REQ-doc-002 (byte-identical round trip) | `round_trips_a_file_it_does_not_change` |
| REQ-doc-003 (criterion line grammar) | `joins_continuation_lines`, `ignores_prose_inside_the_criteria_section`, `reads_a_criterion_however_it_was_decorated` (a bare line, a plain bullet, a bulleted bold id and an italic id all parse to the same `raw_id`); end to end, `cli::a_wrongly_cased_id_is_reported_rather_than_read_as_prose` and `cli::an_ordinary_hyphenated_word_is_not_mistaken_for_an_id` pin the two edges of `looks_like_id` |
| REQ-doc-004 (malformed ids recorded) | `records_malformed_ids_rather_than_skipping_them` |
| REQ-doc-005 (criteria vs retired, `retired:` note) | `separates_retired_criteria` |
| REQ-doc-006 (insert is a splice) | `insert_keeps_the_rest_of_the_file_byte_identical`, `inserts_after_the_last_criterion_of_the_family`, `block_style_insert_keeps_line_positions_correct` (re-parses the written text and asserts every criterion's recorded line still holds its own id); end to end, `cli::an_id_shaped_argument_captures` |
| REQ-doc-007 (placement within blocks) | `inserts_beneath_the_last_descendant_of_a_parent`, `inserts_after_the_last_criterion_of_the_family`, `opens_a_new_block_for_a_new_family`, `keeps_one_family_in_one_block`, `appends_into_an_empty_criteria_section`, `keeps_the_blank_line_under_an_empty_criteria_heading`, `a_level_one_heading_closes_the_criteria_section` |
| REQ-doc-008 (one criterion is one line, hi: FILE-6) | `a_criterion_is_always_exactly_one_line`, `collapses_pasted_whitespace_into_one_sentence`; end to end, `cli::a_long_sentence_stays_on_one_line`, which asserts the written line equals `- **SEND-2**  {sentence}` exactly |
| REQ-doc-009 (declare a new family in frontmatter) | `declares_a_new_family_in_frontmatter_on_insert`, `appends_into_an_empty_criteria_section`, `keeps_block_style_when_adding_a_family`; end to end, `cli::a_new_family_starts_its_own_file`, `cli::block_style_frontmatter_is_understood_and_preserved` |
| REQ-doc-010 (new-file scaffold) | `new_file_text_parses_as_an_empty_doc`; end to end, `cli::a_new_family_starts_its_own_file` |
| REQ-doc-011 (nothing on disk without `save`) | `a_capture_into_an_unfinished_fence_is_refused_rather_than_lost` asserts `to_text()` is byte-identical to the input after a refused `insert`, which is the restore-on-refusal half; end to end, `cli::an_id_is_never_handed_out_twice` case 5 reads the file back. `a_normal_save_leaves_no_temp_file_behind` is the only unit test that reaches disk, and it covers the success path rather than a refusal. `cli::an_existing_id_refuses_with_exit_1_and_writes_nothing` and `cli::a_malformed_id_refuses_without_writing` read the file back and assert it is byte-identical after a refusal, and `capture::tests::refuses_a_malformed_id_without_writing` checks the sentence is absent. The other `capture` refusal tests assert on the error message only. Every one of them refuses *before* `insert` is reached, so `insert`'s own post-splice "no frontmatter" refusal is still untested |
| REQ-doc-012 (both frontmatter styles, hi: FILE-7) | `reads_block_style_families`, `keeps_block_style_when_adding_a_family`, `block_style_insert_keeps_line_positions_correct`; end to end, `cli::block_style_frontmatter_is_understood_and_preserved`, which also asserts the file is never collapsed to `families: [` |
| REQ-doc-013 (fenced blocks are prose, hi: FILE-9) | `a_fenced_block_in_intent_is_not_parsed_as_criteria`, `a_tilde_fence_is_honored_too`, `a_hash_comment_in_a_fenced_snippet_does_not_truncate_intent`; for the write half, `a_fenced_retired_example_is_not_the_retired_section` and `a_documented_example_is_left_completely_alone_by_a_capture`, which also asserts the id drawn in the example is still free to capture |
| REQ-doc-020 (a write is read back, hi: FILE-22) | `a_fenced_retired_example_is_not_the_retired_section` (retiring past a documented `## Retired` must land under a real one and keep the id reserved), `a_capture_into_an_unfinished_fence_is_refused_rather_than_lost` (the refusal, the hint, the file left byte-identical, and that closing the fence is all it takes), `a_documented_example_is_left_completely_alone_by_a_capture` (the guard against over-correcting into treating a fence as structure); end to end, `cli::an_id_is_never_handed_out_twice` cases 4 and 5 drive both reproductions through the real binary. Each was run against the unfixed code and fails there: with only the `retired_heading` fix reverted the retirement is refused instead of landing, and with the read-back reverted as well the criterion is stranded, `insert` returns `Ok`, and `hi SEND-1` hands the id out again. The unaffected-criteria half is covered by `a_capture_above_an_indented_retired_heading_does_not_swallow_it` (the legitimate case still succeeds and SEND-1 stays retired), `a_write_that_would_bring_an_unrelated_criterion_back_is_refused` and `a_write_may_not_reword_a_criterion_it_was_not_about` (which call `read_back` with a hand-made buffer, so they hold whether or not the parser agrees), and `cli::a_capture_never_brings_a_retired_criterion_back_to_life` end to end. Reverted separately: with only the `is_heading_line` break removed the two `read_back` tests still pass and the legitimate capture is *refused* instead of lost, which is the postcondition doing its job; with only the shape comparison removed both `read_back` tests return `Ok` on a buffer in which a retired criterion is live again |
| REQ-doc-014 (stray criteria recorded, hi: CHECK-2.e) | `records_a_criterion_stranded_outside_every_section`; `a_file_with_no_criteria_heading_gets_one` and `a_fenced_block_in_intent_is_not_parsed_as_criteria` both assert `stray` stays empty when it should. Reporting is covered in `check`. Nothing tests an id-shaped line inside `## Intent`, which the code does not record at all |
| REQ-doc-015 (BOM stripped, hi: FILE-11) | `a_byte_order_mark_does_not_hide_the_frontmatter`; end to end, `cli::a_bom_does_not_make_a_valid_file_look_broken` |
| REQ-doc-016 (line endings preserved, hi: FILE-10) | `crlf_line_endings_survive_a_write`, which asserts every `\n` in the written text is part of a `\r\n` |
| REQ-doc-017 (atomic save, hi: FILE-8) | `a_normal_save_leaves_no_temp_file_behind` covers the success path and the absence of a leftover `.chat.md.hi-tmp`. The failure paths (a temp file that cannot be written, a rename that fails) have no test |
| REQ-doc-018 (insert creates a missing section, hi: CAPTURE-7) | `a_file_with_no_criteria_heading_gets_one`, which also re-parses the result and asserts the intent prose survived; `a_capture_into_an_unfinished_fence_is_refused_rather_than_lost` for the case where there is nowhere to put the section |
| REQ-doc-019 (the list-item rule, hi: FILE-1.b) | `a_case_is_rendered_as_a_nested_list_item`, which pins all three indents; `a_criterion_is_always_exactly_one_line` and `collapses_pasted_whitespace_into_one_sentence`, which both assert the `- **ID**  ` prefix; `reads_a_criterion_however_it_was_decorated` for the reading half; end to end, `cli::a_hi_file_renders_as_a_list_not_a_wall_of_text` |
| `src/doc.rs` (`only_an_undeclared_version_and_this_one_are_this_format`) | Unit | REQ-doc-021. No `hi:` key, an empty `hi:` value and `hi: 1` are all HI/1; `2`, `0`, `10`, `1.1`, `one` and `HI/1` all come back as the declared text, quoted exactly as the file wrote it. |

## Manual Testing

- [ ] Capture into an existing family (`hi SEND-2 "..."`), then `git diff hi/chat.md`: the diff must
      be the added lines and nothing else, no reflowed prose and no whitespace churn (hi: FILE-4.a).
- [ ] Capture a case (`hi SEND-1.b "..."`) and confirm the line lands directly under `SEND-1.a`
      rather than at the bottom of the file, indented two spaces deeper than `SEND-1`
      (hi: CAPTURE-4, FILE-1.b).
- [ ] Capture into a family the file does not declare and confirm the `families:` line is the only
      frontmatter line that changed.
- [ ] Capture a very long sentence and confirm it lands as exactly one line, then `grep` the
      criteria section and confirm every criterion is one grep hit (hi: FILE-6, FILE-3).
- [ ] Capture a sentence pasted with embedded newlines and confirm it collapses to one sentence.
- [ ] Capture into a file whose frontmatter uses a block-style `families:` list and confirm the list
      is still a block list afterwards, with the new family as one more `  - NAME` line (hi: FILE-7).
- [ ] Capture into a file saved with Windows line endings, then check the file in a hex viewer or
      with `file`: every line must still end `\r\n` (hi: FILE-10).
- [ ] Write a hi file whose `## Intent` contains a fenced block showing an example criterion, run
      `hi export`, and confirm the example is not among the criteria (hi: FILE-9).
- [ ] Put an id-shaped line below a second `# ` heading and confirm `hi check` names it rather than
      passing silently (hi: CHECK-2.e).
- [ ] Capture into a hand-written file that has no `## Criteria` heading and confirm hi adds the
      section rather than appending into the prose (hi: CAPTURE-7).
- [ ] Open a hi file in a plain markdown viewer with no tooling and confirm it reads as a document,
      and specifically that the criteria render as a nested list with one item per criterion rather
      than as a paragraph of run-together sentences (hi: FILE-1, FILE-1.a, FILE-1.b).
- [ ] Hand-edit a file so its criteria are flat, unbulleted lines, run `hi check` and `hi ls`, and
      confirm both still read every criterion: the parser's leniency is what keeps a hand-edited
      file valid.
- [ ] Run `hi check` after each of the above and confirm no new structural problems appear.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| File with no frontmatter | Parses as body-only; `front` is `Default` with `range: None` |
| Frontmatter fence opened but never closed | Not frontmatter; the whole file is body |
| First line is ` ---` (indented) | Not a fence; no frontmatter |
| Leading UTF-8 BOM before `---` | Stripped before parsing, so the frontmatter is found; the BOM is not written back |
| `---` with trailing spaces | Still a fence; `trim_end` is used for the comparison |
| Block-style `families:` YAML list | Parsed: each indented `- name` line appends a family, `families_block` is true, and a rewrite keeps the block style |
| Block list whose next line is not an indented `-` | Ends the list there; `owner:` on the following line is still parsed |
| `families:` with an empty value and no items under it | An empty block entry: no families, `families_block` is true, so a later rewrite writes the block form |
| `families: []` | Inline: no families, `families_block` stays false, so a rewrite writes `families: [NEW]` |
| A file carrying both `families:` and `family:` | Both append to `front.families`; the span recorded is the **last** entry seen, so a rewrite replaces that one and leaves the earlier line in place |
| `family:` singular key | Appends one family, so a one-family file can use it |
| Empty `owner:` value | `front.owner` stays `None` |
| Unknown frontmatter key | Ignored on parse and preserved on write |
| Prose line inside `## Criteria` | Not a criterion; retained in `lines` |
| Tab-indented continuation | Treated as a continuation, same as space indentation |
| Blank line between a criterion and its would-be continuation | Ends the criterion; the following text is not joined |
| Heading written as `#Chat` (no space) | Not recognized as the title |
| Second `# ` heading in a file | The title is not replaced: the first one wins, but the heading closes the open section and records the append point, so a criterion below it is recorded on `stray` rather than parsed, until the next `## Criteria` or `## Retired` heading |
| New family inserted into a file whose `## Criteria` is closed by a `# ` heading | The `# ` branch records `criteria_end`, so the new family appends below the existing block, after one blank line, and still above the `# ` heading |
| Fenced block containing a heading or an id-shaped line | Opaque: no title, no section change, no criterion, no stray. A ```` ``` ```` fence closes only on backticks and a `~~~` fence only on tildes, and the closing run must be at least as long as the opening one |
| Fenced block opened and never closed | The rest of the body is opaque; nothing after it is parsed as structure, and a capture into such a file is refused rather than written (REQ-doc-020) |
| `## Retired` written only inside a fenced example | Not a section: `retired_heading` skips fenced lines, so `retire` creates a real one at the end of the file and the example is untouched |
| Fenced block inside `## Intent` | The fence lines and everything between them stay in `intent`, and prose after the close is still intent |
| Id-shaped line outside every section | Recorded on `stray` with its 0-based line index; it is in neither `criteria` nor `retired` |
| Indented id-shaped line outside every section | Also a stray. Indentation does not disqualify a line, because a case is written indented under its parent |
| Bulleted or emphasized stray, `  - **SEND-9**  ...` | A stray, but the recorded token keeps its emphasis: the stray branch calls `strip_bullet` alone, so `check` prints `**SEND-9**` rather than `SEND-9` |
| Criterion written with a `*` or `+` bullet, or with `_italics_` around the id | Parsed; `raw_id` is the id alone. hi always writes `- ` and `**` |
| Criterion indented under its parent, as hi writes it | A criterion of its own, not a continuation of the line above: `read_criterion` stops its continuation run at any indented line that is itself criterion-shaped |
| Line starting with a lowercase family, `send-2  ...` | Id-shaped, so it is a criterion with `id: None` and `IdError::BadFamily`; `check` reports it rather than the line reading as prose and vanishing |
| Line starting with an ordinary hyphenated word, `spec-sync ...` | Not a criterion and not a stray: `looks_like_id` requires a digit as the first level |
| Id-shaped line inside `## Intent` | Neither a criterion nor a stray: the intent branch runs above the stray branch, so the line is kept as intent prose and nothing reports it. Untested, and an open decision in `tasks.md` |
| `## Criteria` / `## Retired` in any letter case | Matched; comparison is lower-cased |
| Insert into a file with no `## Criteria` heading at all | A blank line, `## Criteria` and a blank line are spliced in after the last content line, and the criterion lands under them |
| Insert of a case whose parent is not in the file | Falls through to the family rule, landing after the family's last criterion; refusing this is `capture`'s job |
| Insert into a file with no frontmatter, new family | `insert` returns the "has no frontmatter" error and restores the document, so the splice is undone and nothing is saved |
| Two inserts on one in-memory `Doc` | The second does not see the first, because `insert` does not update `self.criteria`. Re-parse between captures |
| Whitespace-only sentence to `render_criterion` | One line: the indent, the bullet, the bold id and its two separating spaces, with nothing after them |
| Sentence of any length | One line. `render_criterion` never returns more than one element |
| Sentence containing newlines, tabs or runs of spaces | Collapsed to single spaces before rendering |
| Sentence containing inline markdown | Stored and written verbatim; this module has no opinion about it |
| Hand-wrapped criterion already in a file | Parsed as one criterion and left exactly as written; hi does not re-render it to one line |
| File with content but no final newline | Gains one on `to_text()`; one of the three documented exceptions to the byte-identical round trip |
| Completely empty file | `to_text()` returns an empty string, not a lone newline |
| File written entirely with CRLF | `newline` is `"\r\n"`; every line written back, inserted ones included, ends `\r\n` |
| File with mixed line endings | The majority ending wins and the whole file is written in it; an exact tie goes to LF |
| Path with no file stem | `name()` falls back to the full displayed path |
| Path with no file name at all | `file_name()` is `None`, so the name is empty and the temp file is `..hi-tmp` in the parent directory; a path with no parent uses `.` |
| A stale `.{name}.hi-tmp` already sitting beside the target | Overwritten (the temp file is created with `File::create`) and then renamed away |
| `save` fails partway | The temp file is removed and the error is returned; the target file is byte-for-byte what it was (hi: FILE-8) |

## Added 2026-09-16: the list-item rule

REQ-doc-019 is covered by `doc::tests::a_case_is_rendered_as_a_nested_list_item`,
`doc::tests::reads_a_criterion_however_it_was_decorated` and
`cli::a_hi_file_renders_as_a_list_not_a_wall_of_text`.

The last of those is the one that would have caught the original defect: it captures three criteria
through the real binary and then asserts on the exact bytes a person reads
(`- **SEND-1**  ...`, `  - **SEND-1.a**  ...`, `- **SEND-2**  ...`), not on what the parser
recovers. Every earlier test passed while the file rendered as a wall of run-together sentences,
because every earlier test asserted on parse output.
