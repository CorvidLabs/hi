---
spec: doc.spec.md
---

## Key Decisions

- **The line buffer is the document.** `Doc` keeps `lines: Vec<String>` as the authoritative
  representation and treats `criteria`, `retired`, `front`, `title`, `intent` and `stray` as an index
  over it. Serialization is `lines.join(self.newline)`. This is what makes "the rest of the file comes back byte for
  byte identical" (hi: FILE-4.a) a structural property rather than a promise, and it is the reason
  every edit in this module is either a splice or an in-place replacement of one line. A future
  change that rebuilds text from the parsed structure would silently destroy that guarantee.
- **Frontmatter is hand-parsed, no YAML crate, but both YAML styles are read.** `hi` has no
  `serde_yaml` dependency. The parser takes `key: value`, strips `[`/`]` and splits on `,` for an
  inline `families`, and trims matched `"`/`'`. A `families:` key whose value is *empty* switches to
  the block branch and consumes the following indented `- name` lines. Consequences worth knowing: a
  value containing a comma inside quotes is still split; any key other than `hi`, `families`,
  `family` and `owner` is ignored and left alone; and `families:` with nothing under it is an empty
  *block* entry, so a later rewrite of that file writes the block form.
- **The file's frontmatter style is data, not a detail** (hi: FILE-7). `families_block` and
  `families_span` exist so `rewrite_families` can put back what it found: one `families: [..]` line
  for an inline file, `families:` plus one `  - NAME` line per family for a block file. Do not
  simplify `rewrite_families` back to a single-line replacement. A block-style file would collapse
  to inline on the next capture, which is exactly the unasked-for reformatting hi promises not to do.
  The span is also why the rewrite has to `shift_after`: a block list changes the file's line count.
- **Both `families:` and `family:` append.** Neither key clears `front.families`; each one pushes
  what it finds, so a file carrying both ends up with the union in document order. `families_span` is
  overwritten by each entry seen, so it names the *last* one, and that is the entry
  `rewrite_families` replaces. A file with two entries would therefore keep the earlier line and
  duplicate its families on the next parse. No file in the repo does this and nothing guards it.
- **A fenced code block is opaque, and the fence check runs before everything else in the body
  loop** (hi: FILE-9). `parse_body` builds a `fence_map` over the body once and short-circuits on
  `fences.inside[index]` before it looks at `# `, `## `, intent prose or criteria. That order is the
  whole mechanism: moving the heading check above it would make a `## Criteria` inside a
  ```` ```markdown ```` example real again. The state is the marker character and the opening run's
  width, so backticks do not close a tilde fence and a short run does not close a long one. Fence
  state is never reset at a heading, so an unclosed fence deliberately swallows the rest of the body
  rather than guessing where the author meant it to end.
- **The write path asks the same function where the fences are** (hi: FILE-22). `fence_map` is
  private to this module and has three callers: `parse_body`, `retired_heading` and `retired_end`.
  That is deliberate. `retired_heading` used to scan the raw lines, so a `## Retired` drawn inside
  somebody's example was the section `retire` moved into — a heading to the writer and prose to the
  parser, which cost a criterion and freed its id. If a fourth place ever needs to know where a
  section is, it calls this; it does not scan lines. And whatever it does after that, it still reads
  its buffer back (below).
- **A criterion outside every section is recorded rather than ignored** (hi: CHECK-2.e). The
  `let Some(current) = section else { .. }` arm pushes `(index, token)` onto `stray`. This is the
  companion to the `# `-heading fix: a heading can no longer make a criterion vanish quietly,
  because either the section stays open or the line is reported. `check` turns each entry into a
  `Kind::StrayCriterion`, so deleting `stray` would silently remove a whole problem kind.
  **One region is not covered.** `if in_intent { intent_lines.push(line); continue; }` sits *above*
  that arm, so an id-shaped line written under `## Intent` never reaches it: it lands in `doc.intent`
  and nothing reports it. That is the same shape of loss CHECK-2.e is about, in the one region the
  fix does not reach. It is not obviously a bug. An unfenced example id in someone's intent prose is
  exactly the thing that would start being flagged if the order were swapped, so it is recorded as a
  decision in `tasks.md` rather than changed here.
- **Three private helpers decide what a criterion line is, and the order matters.**
  `is_criterion_line(trimmed)` calls `strip_bullet` (a leading `- `, `* ` or `+ `), takes the first
  whitespace-delimited token, calls `strip_emphasis` (`trim_matches` over `*` and `_`), and hands
  the result to `looks_like_id`. `read_criterion` repeats the same three steps to produce `raw_id`,
  which is why `**SEND-1**` is stored as `SEND-1`. None of the three is `pub`; they are the module's
  own grammar and nothing outside `doc` should need them. **The stray branch is the one place that
  does not repeat all three:** it calls `strip_bullet` only, so a stray written `- **SEND-9**  ...`
  is recorded as `**SEND-9**` and `check` prints it that way. It is a cosmetic inconsistency in a
  message, not a parse difference, and it is recorded in `tasks.md` rather than fixed here.
- **A bad id is data, not a parse failure.** `Doc::parse` is infallible. `is_criterion_line` decides
  whether a line *starts* a criterion; `Id::parse` decides whether that criterion's id is *valid*.
  `looks_like_id` is deliberately case-insensitive on the family and deliberately demands a digit as
  the first level, so `send-2` is recognized and then rejected with a reason rather than read as
  prose and lost, while `spec-sync` and `well-formed` stay ordinary words.
  A token that passes the first and fails the second becomes a `Criterion` with `id: None` and
  `id_error: Some(..)`, which is exactly what `check` needs to print a file, a line and a reason
  (hi: CHECK-2.d, CHECK-3). Never "fix" this by skipping the line: the error would disappear from
  `hi check`.
- **Insertion order follows document order, not id order.** `insertion_point` scans `self.criteria`
  in the order they appear in the file and keeps the *last* match, so the result is always "after the
  last thing that belongs with it". It never sorts. This keeps append-first behavior and never
  renumbers (hi: ID-1.a).
- **`doc` is consumed by five modules, `view` included.** `workspace`, `capture`, `check`, `out` and
  `view` all read `Doc`; `view` is the only one that reads `doc.criteria` and `doc.retired` as two
  separate lists rather than through `all()`.
- **Only `## Criteria` is an insertion target.** `insertion_point` looks at `self.criteria` alone.
  Retired criteria are never a landing spot and never shift the chosen point, though their line
  indices are still shifted after the splice.
- **One blank line separates families, none separates a family's own criteria.** This is the whole
  of the "block" convention, implemented by the `blank_before` flag returned from
  `insertion_point`.
- **One criterion is one markdown list item, and hi never wraps.** `render_criterion` is three
  statements: collapse the sentence's whitespace, compute `"  ".repeat(id.depth() - 1)`, then
  `format!("{indent}- **{id}**  {sentence}")`. There is no width constant. The reasoning is in the
  source comment and in DECISIONS.md §3, §10.1 and §12: a wrapped criteria section is harder to grep,
  harder to diff and harder to read as a list, and the list is the point (hi: FILE-6, and FILE-3 for
  "looks like something I would have typed by hand").
- **The bullet is not decoration** (hi: FILE-1.b). Markdown joins consecutive plain lines into one
  paragraph, so a criteria section of bare `SEND-1  ...` lines is one line per criterion in the file
  and an unreadable wall of run-together sentences everywhere it is actually rendered. That
  falsified FILE-1, the first thing the format promises, and it passed every test hi had because
  every test asserted on what the parser recovered rather than on what a person sees. The two-space
  indent per depth level is what makes a case render nested under its parent, and the `**` around the
  id is what stops it reading as the first two words of the sentence. DECISIONS.md §12 is the record.
  Do not "simplify" `render_criterion` back to `format!("{id}  {sentence}")`.
- **`save` is atomic on purpose** (hi: FILE-8). `write_atomically` creates `.{name}.hi-tmp` beside
  the target, writes, `flush`es, `sync_all`s, and only then `rename`s over the real file; every
  failure path deletes the temp file first. `fs::write` truncates the target before writing, so a
  disk that fills up halfway would destroy someone's criteria. Do not "simplify" this back to
  `fs::write`, and do not drop the `sync_all`: the flush and the sync are what make the failure
  surface before the rename rather than after it. The temp file is a sibling, not in `/tmp`, because
  `rename` across filesystems fails.
- **`insert` makes the section it needs** (hi: CAPTURE-7). When `criteria_heading` is `None`,
  `insert` splices `["", "## Criteria", ""]` in after the last content line *before* calling
  `insertion_point`, so the heading branch of that function has something to find. The order matters:
  computing the insertion point first would fall through to "append at `lines.len()`", which is how
  a criterion used to end up inside the intent prose.
- **Every write reads itself back, and that is the last line of defence** (hi: FILE-22). The public
  `insert`, `retire` and `set_retired_reason` are thin wrappers: each clones the document, calls an
  `_inner` that does the work, and passes the proposed text through `Doc::read_back`, which parses it
  and checks that the ids the verb named are readable in the section it named, that no previously
  readable id was lost, that every criterion the verb did *not* name comes back with the same
  section, sentence and reason, and that `stray` did not grow or change. That middle clause is
  newer than the rest and was added because an id-only comparison approved a buffer in which a
  retired criterion was live again (DECISIONS.md §35). Any error, from the inner call or from the
  read-back, restores the clone. Two bugs got past everything above this: one because a heading was
  found where the parser saw prose, one because a heading was appended where the parser saw an
  example. Both printed success. The check is cheap (one reparse of a small file per write) and it
  is the only thing in the module that is stated in terms of what a reader will find rather than in
  terms of what the writer did. Do not remove it because the specific bugs it caught are fixed.
- **Reading and writing are deliberately asymmetric.** `read_criterion` still accepts indented
  continuation lines and joins them, so a file someone wrapped by hand is never rejected
  (hi: FILE-1). `render_criterion` never produces one. The same asymmetry covers decoration: the
  parser takes a bare line, any of three bullets, either emphasis marker and any indent, while the
  writer only ever emits `{indent}- **{id}**  {sentence}`. Do not "fix" the parser to match the
  writer. The leniency is what keeps a hand-edited file, and every file written before the list-item
  rule, valid.
- **An indented criterion line is a criterion, not a continuation.** `read_criterion`'s loop breaks
  on `is_criterion_line(next.trim())` before it treats an indented line as a continuation. That one
  line is what lets a case sit indented under its parent, which is the whole point of the nested
  list, instead of being swallowed into the parent's sentence. It is also why indentation no longer
  disqualifies a stray: `parse_body` trims before testing, everywhere.

## Files to Read First

- `src/doc.rs` is this module, including the `#[cfg(test)]`
  block at the bottom, which is the most precise statement of the insertion rules.
- `src/id.rs` holds `Id`, `Level`, `IdError`, `looks_like_id`,
  `Id::parent`, `Id::is_descendant_of` and `Id::depth`. Everything `insertion_point` does depends on
  the middle two, and `render_criterion`'s indent depends on the last one.
- `DECISIONS.md`, sections 3 (the file), 4 (ids), 5 (no
  state, no lifecycle, no evidence) and 12 (a criterion is a markdown list item). Section 3 is the
  format this module implements and section 12 is the correction that gave criteria their bullets.
- `hi/format.md` has the `FILE` and `ID` criteria this module
  exists to satisfy, in the author's own words.
- `src/capture.rs` is the only caller that mutates a document,
  and the one that decides what is allowed before `insert` is ever reached.

## Current Status

- Implemented and covered by 35 unit tests in `src/doc.rs`. Parsing in both frontmatter styles,
  fenced-block opacity in both directions, retired-section handling, malformed-id recording, stray
  recording, the read-back refusal and the restore that goes with it, the four
  reachable insertion-point branches plus the created-section path, frontmatter family declaration,
  the byte-identical round trip, BOM stripping, CRLF preservation, an atomic save, the one-line rule,
  the nested list-item rule, reading a criterion however it was decorated, and whitespace collapsing
  are each exercised. The end-to-end assertion on the bytes a person actually reads lives in
  `tests/cli.rs::a_hi_file_renders_as_a_list_not_a_wall_of_text`, not here.
- `insertion_point`'s last arm, `(self.lines.len(), false)`, is now unreachable: `insert` guarantees
  `criteria_heading` is `Some` before calling it, so the heading branch always fires first. It is
  dead today rather than wrong, and it is the branch that used to append a criterion into the intent
  prose, so removing it is a decision rather than a tidy-up (see `tasks.md`).
- No known blockers. The module has no `unsafe`, no `unwrap` on user input, and no I/O beyond one
  read in `load` and the temp-file write, sync and rename behind `save`.

## Notes

- **`insert` does not update `self.criteria`.** It splices the rendered lines and shifts every
  recorded index, but never constructs a `Criterion` for the line it just wrote. The parsed view is
  therefore stale after an insert: a second `insert` in the same session will not see the first one,
  so `insert(SEND-1, ..)` followed by `insert(SEND-1.a, ..)` would not find the parent. This is safe
  today because `capture` performs exactly one insert per process and then calls `save`. Anything
  that wants to batch captures must re-parse between them (`Doc::parse(path, &doc.to_text())`) or
  teach `insert` to record what it wrote.
- **`insert` can fail after it has already mutated the buffer.** The `rewrite_families` bail for a
  file with no frontmatter happens after `lines.splice`. The in-memory `Doc` is left half-edited. It
  is correct only because no caller saves a `Doc` whose `insert` returned `Err` (hi: CAPTURE-5). If
  `insert` ever gains a second failure mode, move the frontmatter check above the splice.
- **`to_text` does not perfectly round-trip in three narrow ways.**
  `if self.trailing_newline || !out.is_empty()` means a non-empty document always ends in a newline,
  even if the original did not. A leading BOM is stripped in `parse` and never written back. And
  because `newline` is one string chosen for the whole file, a file of mixed endings comes back in
  whichever ending was in the majority. All three are intentional and all three are stated in the
  spec rather than hidden.
- **The line ending is decided once, by counting** (hi: FILE-10). `crlf = raw.matches("\r\n").count()`
  and `lf = raw.matches('\n').count() - crlf`, then CRLF only when `crlf > lf`; a tie goes to LF.
  This matters because `str::lines()` drops the `\r`, so without `newline` every write would silently
  convert a Windows file to Unix endings and produce a whole-file diff on someone's next commit.
- **The BOM is stripped in `Doc::parse`, not in `Doc::load`** (hi: FILE-11). `parse` does
  `raw.strip_prefix('\u{feff}')`, so text handed in by a test or another module is cleaned the same
  way a file read from disk is. `workspace` strips it again for its own frontmatter sniff, because
  that code never builds a `Doc`; the duplication is deliberate, not a leftover.
- **`parse_front` compares with `trim_end`, not `trim`.** `---` followed by trailing spaces is a
  fence; ` ---` indented by one space is not. A file whose first line is indented gets no
  frontmatter at all and parses entirely as body.
- **Heading detection is `trimmed.strip_prefix("# ")` guarded by `!trimmed.starts_with("##")`.** A
  heading with no space after the hash (`#Chat`) is not a title. A `# ` inside a fenced code block is
  not a heading either, because the fence check above it short-circuits first. See the fence
  decision above, and `a_hash_comment_in_a_fenced_snippet_does_not_truncate_intent`, which is the
  regression test for a `# no config` comment inside a ```` ```bash ```` block ending `## Intent`.
- **`criteria_end` is set when the parser *leaves* the `## Criteria` section**, via
  `last_content_line`, which walks back over trailing blank lines. All three exits set it: a `## `
  heading, a level-one `# ` heading, and the end of the file (handled after the loop). Each gives
  "one past the last line with content", which is where a new family's block begins.
- **The `# ` branch of `parse_body` must keep setting `criteria_end`.** It is three lines that look
  redundant next to the `## ` branch and are not: without them `insertion_point` skips the new-family
  branch and falls through to the heading branch, so a new family's first criterion lands directly
  under `## Criteria`, above the existing block and with no blank line between the families.
  `a_level_one_heading_closes_the_criteria_section` is the regression test and says so in a comment.
  A criterion written below that heading is still not parsed as a criterion. It is recorded on
  `stray` instead, which is the other half of the same fix.
- **`render_criterion` collapses interior whitespace** in the sentence it is handed, via
  `split_whitespace().join(" ")`. That applies only to a sentence arriving from the command line at
  capture time; it never touches prose already in a file, which is what hi: FILE-4 is about. It is
  also what lets a shell-pasted multi-line thought land as one criterion.
- **A sentence may contain inline markdown** (`` `code` ``, `**bold**`, links). This module stores
  and emits it verbatim and has no opinion about it; rendering is `hi view`'s business.
