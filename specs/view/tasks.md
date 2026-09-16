---
spec: view.spec.md
---

## Tasks

- [ ] Assert the rendered page parses as well-formed HTML, so a template mistake fails the build.
- [ ] Add a test for nested markers (`` **a `b` c** ``) pinning the current rule, which is that
      there is no nesting: the first marker to open takes its contents verbatim, so the inner
      backticks stay visible. Confirmed against the binary; nothing asserts it.
- [ ] Factor the `hi:index` marker constants into one place; they are duplicated in `out.rs` and `view.rs`.
- [ ] Decide whether `view::write` should strip a leading BOM from `INTENT.md`. `Doc::parse` and
      `Workspace::find` both strip one and `str::trim` does not, so today a BOM leaves the H1 in the
      lead prose.
- [ ] Make `strip_index` (and `out`'s marker search with it) skip fenced blocks in `INTENT.md`.
      This is no longer only a judgement call: hi: INDEX-2.a now says a marker quoted *"inside a
      code block"* is not the real list, and neither module honours that. `view` only drops prose
      from the page, so the cost here is cosmetic; `out` rewrites the file, so it is the side that
      matters. Fix both together or the two stop agreeing about where the block is.
- [ ] Add a phone-width check to the suite, or accept REQ-view-010 (hi: VIEW-2.a) as manual
      forever and say so. Today nothing automated touches it.
- [ ] Assert that each criterion is its own `<li>`. `copied_text_keeps_the_id_and_sentence_apart`
      pins the span whitespace but not the per-criterion line, which is the other half of
      hi: VIEW-1.d.
- [ ] Decide whether `--out` should be confined to the repository root, and whether `Workspace::rel`
      should print an unrelativizable path as-is rather than with a doubled leading slash.
- [ ] Decide whether CI should publish the page to GitHub Pages instead of leaving it a local artifact.

## Gaps

- No well-formedness or snapshot coverage of the output; see `testing.md`.
- Phone-width behavior is verified by hand only.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: pending
- **Dev**: pending
