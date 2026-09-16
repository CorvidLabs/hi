---
spec: view.spec.md
---

## Tasks

- [ ] Assert the rendered page parses as well-formed HTML, so a template mistake fails the build.
- [ ] Add a test for nested markers (`` **a `b` c** ``) pinning the current precedence.
- [ ] Factor the `hi:index` marker constants into one place; they are duplicated in `out.rs` and `view.rs`.
- [ ] Decide whether `view::write` should strip a leading BOM from `INTENT.md`. `Doc::parse` and
      `Workspace::find` both strip one and `str::trim` does not, so today a BOM leaves the H1 in the
      lead prose.
- [ ] Decide whether `strip_index` (and `out`'s marker search with it) should skip fenced blocks in
      `INTENT.md`, now that fences are opaque to `doc`'s parser.
- [ ] Decide whether CI should publish the page to GitHub Pages instead of leaving it a local artifact.

## Gaps

- No well-formedness or snapshot coverage of the output; see `testing.md`.
- Phone-width and blocked-font behavior are verified by hand only.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: pending
- **Dev**: pending
