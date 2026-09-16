---
spec: out.spec.md
---

## Tasks

- [x] Document the four read verbs against source. Evidence: `specs/out/out.spec.md` v1.
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
- [ ] Decide whether a family scope should carry that family's retired criteria. It does today; it
      is unstated in DECISIONS.md and untested either way.
- [ ] Decide what `export <FAMILY>` should do when the family is declared in frontmatter but used
      by no criterion. Today it bails with "it is not a family or a file in hi/", which contradicts
      the frontmatter `Workspace::families` read it from (REQ-out-009). Either the message should
      say the family is declared but empty, or the file-selection test should accept a declaration.
- [ ] Decide whether `INTENT.md` should get the same treatment `hi/*.md` now gets: a stripped BOM,
      a preserved line ending, and an atomic replace. `write_index` does none of the three today
      (plain `fs::write`, `\n` inside the generated block, no BOM strip), which is invisible until
      someone hand-edits the file on Windows or with an editor that writes a BOM, and then a BOM'd
      opening marker quietly takes the append branch (REQ-out-011, REQ-out-012).
- [ ] Add a unit test pinning the family/file-stem collision: a scope that is both selects the
      named file whole *and* every other file holding the family, filtered (REQ-out-007). It is the
      one scope rule nothing asserts and the easiest to "fix" into a regression.

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

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
