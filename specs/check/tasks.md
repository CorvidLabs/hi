---
spec: check.spec.md
---

## Tasks

- [ ] Assert a `Problem`'s `file` and `line` directly, and assert the file-then-line sort order over
      a two-file workspace with problems on several lines (REQ-check-007).
- [ ] Assert that `Kind::code()` returns the same six strings the serde kebab-case rename produces,
      so text output and `--json` cannot drift apart (REQ-check-010).
- [ ] Snapshot the serialized `Report` for a workspace with one problem, covering the counts and the
      problem fields (REQ-check-010).
- [ ] Assert that `SEND-007` in a file produces one `UnparseableId` carrying the leading-zero
      message, so `check`'s half of ID-1.c is pinned and not only capture's (REQ-check-005).
- [ ] Assert in `check`'s own tests that a fenced id-shaped line produces no problem, so the
      guarantee is not left resting entirely on `doc`'s tests (REQ-check-012).
- [ ] Assert that an unfenced id-shaped line under `## Intent` produces no problem and
      no criterion, and that the same line under `## Notes` produces one `StrayCriterion`. The
      asymmetry is entirely implicit in the branch order of `doc::parse_body` and nothing anywhere
      pins it (REQ-check-011).
- [ ] Assert that an indented, bulleted, bolded stray (`  - **SEND-11**  ...`) outside every section
      is still exactly one `StrayCriterion`. Criteria render as nested list items now, so a real
      stranded case is indented, and nothing tests that shape (REQ-check-011).
- [ ] Decide and then pin what a stray's `id` should be. Today `doc` strips the bullet but not the
      emphasis before recording the token, so a stray reports `**SEND-9**` while a criterion on the
      same line would report `SEND-9`. Either is defensible; the asymmetry is untested either way,
      and the fix, if it is one, belongs in `doc::parse_body` rather than here (REQ-check-011).
- [ ] Assert in `check`'s own tests that `- **send-2**  ...` yields one `UnparseableId` carrying
      `IdError::BadFamily`. `cli::a_wrongly_cased_id_is_reported_rather_than_read_as_prose` covers it
      end to end, but nothing in this module's suite would notice if the variant stopped being
      reachable (REQ-check-005).

## Gaps

- REQ-check-007 is partly verified: `cli::check_fails_on_a_structural_problem` asserts the path and
  line reach the output, but no test reads `problem.file` or `problem.line` from a `Report`, and no
  test has enough problems to observe the file-then-line sort.
- REQ-check-010 is unverified: nothing exercises serialization, so a renamed field or a changed
  variant spelling would break `hi check --json` silently.
- No test covers an id retired in one file and reused in another, which is the case the separate
  retired-collection pass exists for.
- No test covers a malformed id that would also have been a duplicate or an orphan, which is what
  pins the `continue` after `UnparseableId`.
- REQ-check-012 is covered only indirectly, by `doc`'s fence tests. Nothing in `check`'s own suite
  would notice if this module started re-scanning raw lines.
- Every unit test in this module writes its criteria as bare, flush-left lines (`SEND-1  One.`),
  which is the shape hi itself no longer produces. `render_criterion` writes
  `- **SEND-1**  One.` indented by depth, so the suite exercises only the hand-written half of
  `FILE-14` and never the half the tool emits. The parser treats both identically, verified against
  the binary, but the tests do not say so.
- The `## Intent` exception to REQ-check-011 is covered by no test in any module. It is worth a
  second look from product as well as QA: CHECK-2.e says a criterion sitting outside every section
  is an error "because nothing would read it there", and a criterion typed under `## Intent` is read
  by nothing as a criterion, yet `check` stays silent. The current behavior is defensible (the
  intent is prose) but it is an unwritten carve-out, not a stated one.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
