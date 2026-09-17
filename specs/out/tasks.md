---
spec: out.spec.md
---

## Tasks

- [x] Document the four read verbs against source. Evidence: `specs/out/out.spec.md` v1.
- [x] Unwrap the soft line breaks in the intent prose a ticket carries (REQ-out-015). Evidence:
      `unwrap_soft_breaks` in `src/out.rs`, pinned by
      `src/out.rs::a_ticket_unwraps_prose_the_author_only_wrapped` (which fails without the call),
      `src/out.rs::only_a_wrapped_line_is_joined`, and
      `tests/cli.rs::a_ticket_unwraps_prose_and_leaves_the_file_alone`.
- [x] Decide whether `hi view` and `hi export` need the same treatment (REQ-out-015). Evidence:
      neither does, and neither was changed. `view::paragraphs` already joins a paragraph's lines
      with spaces, and HTML collapses whitespace anyway, so the page has always rendered these
      paragraphs whole. `export` is a transport and not a rendering, and unwrapping there would
      throw the author's wrap points away for every downstream consumer irreversibly; see Gaps.
      `ls` and `index_block` emit no intent prose at all.
- [x] Make the text hi writes into a file it creates one line per paragraph (REQ-out-016).
      Evidence: `agent_instructions` and `starter_intent` in `src/out.rs`, pinned by
      `src/out.rs::the_files_hi_writes_are_one_line_per_paragraph`.
- [ ] Add a unit test for `issue` refusing a retired id (REQ-out-005). The `CHAT` fixture needs a
      `## Retired` section, then assert the error message names "retired".
- [x] Add a unit test for `write_index` splicing between the markers and leaving surrounding prose
      byte-for-byte identical (REQ-out-011). Evidence: covered end to end by
      `tests/cli.rs::index_rewrites_only_the_generated_block` and
      `index_leaves_a_marker_quoted_in_prose_alone`, plus the inline
      `a_marker_quoted_in_prose_is_not_the_generated_block` on `index_span` itself.
- [x] Pin the whole-line marker rule and the refusal on a broken marker pair (REQ-out-012,
      REQ-out-013). Evidence: `an_unclosed_marker_has_no_span` inline and
      `tests/cli.rs::index_refuses_rather_than_guessing_when_a_marker_is_unclosed`, which asserts
      the file is byte-identical after the refusal.
- [ ] Add a unit test for `write_index` creating the starter file when `INTENT.md` is absent or
      whitespace-only (REQ-out-011).
- [ ] Add a unit test asserting `product` is present at repo scope and absent at family and file
      scope (REQ-out-006, REQ-out-008): the one-shape rule is the export contract and nothing
      currently pins it.
- [x] Decide whether a family scope should carry that family's retired criteria. Evidence: it does,
      and EXPORT-4 now says so ("Retired criteria come along, kept apart from the live ones, so the
      agent never writes a spec for something we dropped"). Still untested; see Gaps.
- [ ] Decide what `export <FAMILY>` should do when the family is declared in frontmatter but used
      by no criterion. The message no longer claims the family does not exist, but it still answers
      a correct family name with "Give a family like SEND" and never says the family is declared and
      empty (REQ-out-009). Either name that case, or let a declaration select the file.
- [ ] Decide whether `INTENT.md` should get the remaining `hi/*.md` hygiene: a stripped BOM and a
      preserved line ending. The atomic replace is done (`write_index` routes through
      `doc::write_atomically`), but the block is still written with `\n` into a CRLF file, and a
      BOM'd opening marker still quietly takes the append branch (REQ-out-011, REQ-out-012).
- [ ] Add a unit test pinning the family/file-stem collision: a scope that is both selects the
      named file whole *and* every other file holding the family, filtered (REQ-out-007). It is the
      one scope rule nothing asserts and the easiest to "fix" into a regression. Verified by hand
      against the binary: `hi/SEND.md` holding `ALPHA-1` beside `hi/other.md` holding `SEND-1` and
      `BETA-1`, scope `SEND`, gives `hi/SEND.md` with `ALPHA-1` and `hi/other.md` with `SEND-1`.
- [ ] Strengthen `tests/cli.rs::export_accepts_the_path_it_prints`, which today asserts only that
      `chat`, `chat.md`, and `hi/chat.md` exit 0 (REQ-out-007). Nothing asserts they select the same
      file, that `scope` echoes the spelling, or that the path arm is a suffix test, so
      `/anywhere/hi/chat.md` matching is unpinned in either direction.

## Gaps

- `ls` has no automated coverage at all (REQ-out-001). It only prints, so covering it means either
  extracting a string-returning renderer or capturing stdout.
- `issue --create` has no automated coverage (REQ-out-004). It spawns `gh`; covering it needs a
  stub binary on `PATH` or an injected command runner.
- `issue` has three unasserted error paths: invalid id, unknown id, and retired id (REQ-out-005).
- `write_index`'s starter-file and append branches have no coverage (REQ-out-011). The splice and
  refusal branches are covered by `tests/cli.rs`; nothing exercises an absent, blank, or
  marker-less `INTENT.md`.
- Nothing asserts that a fenced or stray id-shaped line is absent from this module's output
  (REQ-out-014). The parser side is covered in `specs/doc`; here the consequence for `ls`,
  `export`, and the `index_block` count is only checked by hand.
- `read_product_intent` is unasserted: neither the index-stripping nor the `None` fallbacks are
  covered (REQ-out-008).
- Nothing covers `INTENT.md`'s encoding edges (REQ-out-011, REQ-out-012): a CRLF file through
  `write_index`, or a BOM immediately before the opening marker. Both are reachable by hand-editing
  and are only verified by reading the code and by hand.
- The `no criteria yet` hint and the `nothing captured yet` index line are both unasserted.
- Nothing asserts what a person *sees* on either surface this module renders to. The two
  quoted-marker tests both put the marker mid-sentence, so the fenced-marker loss is uncovered, and
  no test renders an `issue` body to check a depth-3 case against its parent. Both are the failure
  shape DECISIONS.md §12 named: the tests assert on what hi parses, not on what the reader gets.
- Nothing covers `write_index`'s atomicity now that it has some (REQ-out-011). There is no
  regression test alongside the capture one, and nothing pins that a read-only `INTENT.md` is
  replaced rather than refused.
- `export` deliberately carries the intent prose verbatim, wrapping and all, while `issue_markdown`
  unwraps it (REQ-out-008, REQ-out-015). The payload is a transport rather than a rendering: no JSON
  consumer turns a `\n` into a visible break the way a GitHub issue body does, a consumer that wants
  it unwrapped can unwrap it, and one that wants the source form cannot get it back once hi has
  thrown the wrap points away. Decided rather than overlooked; revisit only if an agent is found
  propagating the wrapping into something a person reads.
- The wrapping convention has no test outside this repository and no enforcement anywhere
  (REQ-out-016). hi writes one-line paragraphs, hi's own files are one-line paragraphs, `hi/AGENTS.md`
  says so and the README says so, and that is all there is. Whether adopters' files follow is not
  measured, and measuring it is not the same as gating on it: FILE-4 forbids rewriting their prose
  and CHECK-1 forbids a seventh structural problem (DECISIONS.md §29).

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
