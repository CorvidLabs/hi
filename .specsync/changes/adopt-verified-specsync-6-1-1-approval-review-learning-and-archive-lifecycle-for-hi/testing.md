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
