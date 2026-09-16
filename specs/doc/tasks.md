---
spec: doc.spec.md
---

## Tasks

- [ ] Decide whether `insert` should record the `Criterion` it just wrote into `self.criteria`, so a
      second insert on the same in-memory `Doc` can see the first. Today `capture` inserts once per
      process, so this is latent rather than broken.
- [ ] Move the missing-frontmatter check in `insert` above the `lines.splice`, so a refusal cannot
      leave a half-edited document in memory.
- [x] Decide what a level-one `# ` heading after `## Criteria` should mean. Done, and both options
      were taken: the `# ` branch of `parse_body` now records `criteria_end`, so a new family still
      appends below the existing block, and an id-shaped line outside every section is collected on
      `doc.stray` for `check` to report (hi: CHECK-2.e).
- [ ] Decide whether a file carrying both a `families:` and a `family:` entry should be supported.
      Today both append to `front.families` but only the last entry's span is recorded, so a rewrite
      replaces that one and leaves the earlier line duplicating families on the next parse.
- [ ] Decide whether `to_text` should write a stripped BOM back. Today it does not, which is the
      third exception to the byte-identical round trip.
- [ ] Decide whether an id-shaped line inside `## Intent` should be a stray. Today it is not: the
      `in_intent` push in `parse_body` runs above the stray branch, so such a line becomes intent
      prose and neither `doc` nor `check` ever mentions it (the one region CHECK-2.e's fix does not
      reach). Swapping the order would close it, at the cost of flagging an unfenced example id
      someone wrote into their own prose.
- [ ] Decide what to do with `insertion_point`'s final `(self.lines.len(), false)` arm. It became
      unreachable when `insert` started creating a missing `## Criteria` section, since
      `criteria_heading` is always `Some` by the time `insertion_point` runs. It is also the branch
      that used to append a criterion into the intent prose, so deleting it removes the last trace of
      that behavior.

## Gaps

- REQ-doc-011's refusal path has no unit test inside this module. `a_normal_save_leaves_no_temp_file_behind`
  exercises `Doc::load` and `Doc::save` on the success path only. `tests/cli.rs` covers the refusal
  end to end (`an_existing_id_refuses_with_exit_1_and_writes_nothing` and
  `a_malformed_id_refuses_without_writing` both compare the file byte for byte before and after), but
  every one of those refusals happens before `insert` is reached. `insert`'s own post-splice "no
  frontmatter" refusal is untested anywhere; it was confirmed by hand against the built binary, which
  leaves the file on disk untouched.
- REQ-doc-017's failure paths are untested: no test makes the temp-file write or the rename fail, so
  only the happy path and the absence of a leftover temp file are verified.
- No test covers the `to_text()` exception for content that does not end in a newline, nor the empty
  file returning an empty string, nor a file of mixed line endings choosing the majority.
- No test covers the singular `family:` frontmatter key or an unterminated `---` fence; both are
  parse branches with defined behavior. A file carrying two families entries is untested and its
  behavior is a quirk rather than a decision (see Tasks).
- No test covers an unclosed fenced code block swallowing the rest of the body, which is the one
  fence branch the three fence tests do not reach.
- No test covers `used_families()`, `all()` or `name()` directly; they are exercised only through
  sibling modules.
- No test covers a criterion line whose continuation is tab-indented, nor `render_criterion` with a
  whitespace-only sentence.
- No test asserts that a BOM'd file round-trips without the BOM, only that the BOM does not hide the
  frontmatter.
- No test covers an id-shaped line inside `## Intent`. `records_a_criterion_stranded_outside_every_section`
  puts the line under a `# ` heading, and `check::tests::catches_a_criterion_stranded_outside_every_section`
  does the same, so the `in_intent` hole above is invisible to the suite.
- No test reaches `insertion_point`'s `(self.lines.len(), false)` arm, because nothing can.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
