---
spec: main.spec.md
---

## Tasks

- [ ] Run the integration tests on Windows and confirm `Workspace::rel` behaves with canonicalized `\\?\` paths.
- [x] Assert `--root` is honored by capture and not swallowed into the sentence, covered by `root_is_honored_by_capture_and_not_swallowed_into_the_sentence`.
- [ ] Assert `--root` is honored by every non-capture subcommand; only the capture path is covered.
- [ ] Assert the family and file plural forms in the check summary.
- [ ] Cover the non-UTF-8 argument path on Windows, where the test is currently `#[cfg(unix)]`.
- [ ] Pin `peel_root`'s remaining argument shapes: `--root` twice, a trailing `--root` with no value on the capture path, and a `--root` value that is not valid UTF-8 in each of its two forms.

## Gaps

- Windows path behavior is unverified; see `testing.md`.
- `peel_root` has no direct test, because `main.rs` carries no test module. Only the two happy-path
  forms are covered, by `root_is_honored_by_capture_and_not_swallowed_into_the_sentence`.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
