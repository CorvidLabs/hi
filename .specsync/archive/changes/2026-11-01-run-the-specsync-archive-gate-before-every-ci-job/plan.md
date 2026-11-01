---
change: run-the-specsync-archive-gate-before-every-ci-job
artifact: plan
---

# Plan

## Approach

Add an `archive-gate` job, displayed as `Archive`, at the top of `jobs:` in every workflow that runs on pull_request or push, and make every other job in those workflows need it.

## Out of scope

`release.yml`, which fires only on `v*` tags. Branch protection and the org ruleset requiring Archive. Any change to product code, specs, the existing SpecSync lifecycle job or its scripts.

## Steps

1. In `.github/workflows/ci.yml`, add the `archive-gate` job on ubuntu-latest using `actions/checkout@v4` (head sha, no persisted credentials) and `CorvidLabs/spec-sync@106aaa43d0bcf049f19b173ddb63815a25c3f931` with `archive-gate-only: 'true'`; add `needs: archive-gate` to `check`, `view`, `lint` and `specsync`.
2. In `.github/workflows/pages.yml`, add the same job and `needs: archive-gate` to `build`; `deploy` already needs `build` and so reaches the gate transitively.
3. Confirm both files parse and print each job's needs.
4. Run the archive-readiness script, `specsync change audit --strict` and `specsync check --strict` locally.

## Risks

A pull request carrying an open change cannot get a green CI until the change is archived, which is the intended behaviour. If the pinned commit becomes unreachable the gate fails closed. A skipped job counts as passing, so the merge block depends on the separately configured ruleset requiring Archive.

## Constraints consulted

AGENTS.md: the change goes through the native SpecSync 6.1.1 lifecycle with a real human approver and reviewer (0xLeif); existing product checks and the existing lifecycle gate (`scripts/check-specsync-lifecycle.sh`) are preserved; no credentials, branch protections or deployment state are modified to qualify the change.
