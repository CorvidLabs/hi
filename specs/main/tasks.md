---
spec: main.spec.md
---

## Tasks

- [ ] Run the integration tests on Windows and confirm `Workspace::rel`'s `strip_prefix` behaves with canonicalized `\\?\` paths. The separator half is done and unit-tested.
- [x] Assert `--root` is honored by capture and not swallowed into the sentence, covered by `root_is_honored_by_capture_and_not_swallowed_into_the_sentence`.
- [x] Assert a flag-shaped word after the id stays part of the sentence, covered by `a_flag_looking_word_inside_a_sentence_stays_a_word`.
- [ ] Assert `--root` is honored by every non-capture subcommand; only the capture path is covered.
- [ ] Assert the family and file plural forms in the check summary.
- [ ] Cover the non-UTF-8 argument path on Windows, where the test is currently `#[cfg(unix)]`.
- [ ] Pin `peel_root`'s remaining argument shapes: `--root` twice ahead of the id, a leading `--root` with no value, `hi --root SEND-2 "<sentence>"` where the id is peeled as the root, and a `--root` value that is not valid UTF-8 in each of its two forms.
- [ ] Pin the routing predicate from argv rather than only through `hi check`: no test runs `hi spec-sync ...` or `hi SEND-a ...` and asserts the exit 2 that `looks_like_id` now produces for them.

## Gaps

- Windows path behavior is unverified; see `testing.md`.
- `peel_root` has no direct test, because `main.rs` carries no test module. The two leading forms
  are covered by `root_is_honored_by_capture_and_not_swallowed_into_the_sentence`, and the stop at
  the id by `a_flag_looking_word_inside_a_sentence_stays_a_word`. The malformed shapes are not.
- `looks_like_id`'s two halves are unit-tested in `src/id.rs` and exercised through `hi check`, but
  never through argv, so nothing asserts that a token the predicate rejects becomes a clap usage
  error rather than a capture refusal.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
