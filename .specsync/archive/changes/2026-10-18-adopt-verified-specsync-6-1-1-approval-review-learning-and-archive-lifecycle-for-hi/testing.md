---
change: adopt-verified-specsync-6-1-1-approval-review-learning-and-archive-lifecycle-for-hi
artifact: testing
---

# Testing

## Planned local tests (not run)

- `fledge lanes run verify`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --check`

Run the exact pinned writer: `specsync --version` must report 6.1.1. Run `specsync check --force --strict`, `specsync change audit --strict`, then scoped `specsync change check adopt-verified-specsync-6-1-1-approval-review-learning-and-archive-lifecycle-for-hi --strict` only after definition approval and implementation. The operations no-spec check alone is not product testing. Preserve current coverage thresholds. Run the repository Trust gate where configured and qualified; report missing prerequisites or provenance failures without weakening them.

## Lifecycle regression controls

In disposable fixture repositories, compare an unchanged-policy clean finalized case against uncovered source/test/governance changes, wrong 6.1.0 engine, disabled flags and active/unfinalized workspaces. Each negative control must fail for the intended reason. Verify existing checks still execute and an archive-only follow-up can pass without creating an approval/review record. No human identity is invented for fixtures; controls needing authentic human acceptance remain manual.

## Baseline versus candidate

Baseline command outputs and exit codes live outside this repository in the rollout evidence directory at exact base a65a4a1a28c8dd0c204f6e58be30ecba4f6350ab. They are not implementation validation. Re-run baselines after any base update. New failures belong to this candidate; inherited failures remain owner debt. No product tests have run in this preparation.

## Candidate qualification (2026-10-18)

- `cargo test`: pass with isolated /var/tmp TMPDIR. Initial shared /tmp run fails
  the outside-repository CLI assertion because /tmp/.git exists; retained logs
  distinguish the environmental failure and unchanged-assertion passing retry.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo fmt --check`: pass.
- `fledge lanes run verify`: pass, exact6.1.1 writer, isolated TMPDIR, all eight
  steps complete. Browser step explicitly skips absent Chrome/Chromium; no browser
  validation claimed. Intent/index, nudge7 checks and plugin validation pass.
- Canonical strict check:10 specs pass,100% coverage.
- Lifecycle controls:13 pass; actual6.1.0 writer rejected.
- Scoped verification and strict active audit run separately after recording tasks.
- Archive guard deliberately blocks this active workspace pending real human
  implementation review and finalization. No waiver or fake acceptance.

Logs: rollout/evidence/hi-cargo-test-isolated.log, hi-verify-pinned.log,
hi-clippy.log, hi-fmt.log, hi-controls-final.log and hi-real-old-engine.log.
