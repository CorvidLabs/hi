---
spec: view.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/view.rs` (`mod tests`) | Unit | Eleven tests: the inline renderer, escaping, link allow-list, unmatched markers, comment stripping, the empty-prose block, the no-network guarantee, copy fidelity, page assembly, retired folding, index stripping. |
| `tests/cli.rs` | Integration | `view_writes_a_self_contained_page` runs the binary end to end and asserts the written `intent.html` carries a criterion sentence and the `prefers-color-scheme: dark` block, and no `<script`. It does not assert the `:root[data-theme="dark"]` block, and it does not check for external references. |

| Requirement | Covered by | Notes |
|---|---|---|
| REQ-view-001 | `page_carries_intent_prose_and_every_criterion` | Asserts product prose, feature prose, the H2 title and a nested criterion all reach the page. |
| REQ-view-002 | `page_carries_intent_prose_and_every_criterion` | Asserts `class="d2"` is emitted for a depth-2 id. |
| REQ-view-003 | `the_page_fetches_nothing`, `page_carries_intent_prose_and_every_criterion`, `view_writes_a_self_contained_page` (`tests/cli.rs`) | `the_page_fetches_nothing` renders a page and asserts it contains none of `http://`, `https://`, `//fonts.`, `<script`, `<iframe` or `@import`, which pins the whole no-network claim. The other two assert only the `prefers-color-scheme: dark` block; the `:root[data-theme="dark"]` block is unasserted. The integration test also asserts the written file carries no `<script`. |
| REQ-view-004 | `renders_inline_code_bold_italic_and_links`, `leaves_an_unclosed_marker_alone` | All four constructs, plus the literal-text behavior of unmatched markers. |
| REQ-view-005 | `escapes_markup_in_a_sentence`, `refuses_a_javascript_link_and_leaves_it_as_text` | Script tags, quotes, and a hostile URL scheme. |
| REQ-view-006 | `retired_criteria_are_tucked_away_but_present` | Asserts `<details>`, the summary count, and that the retired id is still present. |
| REQ-view-007 | `strips_the_generated_index_from_product_prose` | Index, H1 and `## Features` are removed from lead prose. No test writes a hi file, because the module has no write path to them. |
| REQ-view-008 | `html_comments_never_reach_the_page`, `a_feature_whose_prose_is_only_the_template_comment_renders_no_empty_block` | The first pins `strip_comments` directly, including the unterminated case; the second renders a whole page from a file whose prose is only the template prompt and asserts neither the prompt nor an empty `<div class="intent">` appears, while the criterion still does. |
| REQ-view-009 | `copied_text_keeps_the_id_and_sentence_apart` | Asserts the newline between the `cid` and `ctext` spans, and that `</span><span` appears nowhere on the page. One `<li>` per criterion is structural and unasserted. |
| REQ-view-010 | None | Manual only. The layout rules are read off the stylesheet; no test renders at a width. |

## Manual Testing

- [x] Render hi's own intent and read the page top to bottom for a reader who has not seen the repo.
- [x] View in both light and dark mode and confirm contrast and legibility in each.
- [ ] View at 375px width and confirm the id/sentence stacking and the absence of horizontal scroll (hi: VIEW-2.a).
- [ ] Open the page with the network off and watch the browser's network panel stay empty.
- [ ] Copy the criteria list into a plain-text field and confirm one criterion per line, id and sentence apart (hi: VIEW-1.d).
- [ ] Open the page in Safari and Firefox, not only Chrome.

## Edge Cases & Boundary Conditions

| Case | Expected |
|---|---|
| Sentence containing `<script>` | Escaped to text; no element produced |
| Sentence containing a double quote | `&quot;`; cannot terminate an attribute |
| `[click](javascript:alert(1))` | No anchor; renders as literal text |
| `[click](JavaScript:alert(1))` | No anchor; the allow-list is prefix-matched, so any non-allowed scheme falls through |
| Unmatched `` ` `` or `*` | Literal text, unchanged |
| `2 * 3 is 6` | Unchanged; no italic |
| Nested markers, e.g. `` **a `b` c** `` | No recursion: whichever marker opens first takes everything up to its close verbatim. `` **a `b` c** `` renders `<strong>a `b` c</strong>` with the backticks literal, and `` `x **y** z` `` renders `<code>x **y** z</code>` with the asterisks literal. Confirmed against the binary, not asserted |
| Feature file with no criteria | `Nothing written down yet.` |
| Feature file with no `## Intent` | Section renders criteria with no prose block |
| Prose containing `<!-- note -->` | The span is removed; the text around it renders normally |
| Prose that is only a comment | No prose block at all, not an empty `<div>` |
| Prose with an unterminated `<!--` | Everything from the marker to the end of that prose is dropped |
| `<!--` inside a criterion sentence | Escaped to visible text; only prose is comment-stripped |
| Fenced block kept in `## Intent` prose by `doc` | Rendered as ordinary paragraph text with the inline rules applied; `view` has no block-level markdown, so it does not become a code block. Reasoned about, not asserted |
| `<!-- hi:index -->` mentioned inside a sentence | Not a marker; the comparison is a trimmed whole line, so stripping does not start. Not asserted |
| `<!-- hi:index -->`, an H1 or `## Features` alone on a line inside a fenced example in `INTENT.md` | Stripped as if real; `strip_index` is not fence-aware. A fenced opening marker starts stripping there and drops everything down to the next real closing marker. Confirmed against the binary, not asserted |
| `INTENT.md` starting with a UTF-8 BOM | The BOM survives, so the H1 line is not recognised and renders in the lead prose. Confirmed against the binary, not asserted by a test |
| Missing `INTENT.md` | Lead section omitted; page still renders |
| Id that failed to parse | Renders at depth 1 with its raw text, escaped |
| Id deeper than four levels, e.g. `SEND-1.a.1.a.1` | Rendered at `d4`; the depth class is capped, not the id |
| `--out` naming a directory that does not exist | `error: writing <path>: No such file or directory (os error 2)`, exit 1. No directory is created |
| `--out` that is absolute or climbs out of the repository | Written there. The printed path is `Workspace::rel` of a path it cannot relativize: `../outside.html`, or a doubled leading slash for an absolute path. Confirmed against the binary, not asserted |

## Gaps

- No assertion that the generated HTML is well-formed. A parse check over the output would catch a
  template-level mistake that the current substring assertions would miss.
- Nested-marker behavior (`` **a `b` c** ``) is confirmed against the binary but nothing asserts it,
  so the no-recursion rule could change unnoticed.
- A fenced block that `doc` keeps in `## Intent` prose has no rendering test; `view` sees it as plain prose.
- `strip_index` has one test, over a well-formed block. The unterminated-marker, fenced-marker and
  BOM cases have nothing pinning them; the fenced and BOM cases were confirmed by hand against the
  binary rather than in the suite.
- No snapshot test of the whole page, so a layout regression is invisible to `cargo test`.
- Phone-width rendering (REQ-view-010) has no automated coverage at all; the stylesheet is read, not exercised.
- `copied_text_keeps_the_id_and_sentence_apart` pins the span whitespace but nothing asserts that each criterion is its own `<li>`, which is the other half of hi: VIEW-1.d.
- Nothing asserts the `--out` behavior: neither the failure message on an unwritable path nor the paths that escape the repository root.
