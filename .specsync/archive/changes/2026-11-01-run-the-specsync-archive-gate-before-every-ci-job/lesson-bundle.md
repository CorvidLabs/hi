# Lesson bundle — run-the-specsync-archive-gate-before-every-ci-job

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Run the SpecSync archive gate before every CI job
- **Kind**: Operations
- **Paths**: .github/workflows/ci.yml, .github/workflows/pages.yml
- **Acceptance**: ci.yml and pages.yml each start with an archive-gate job named Archive that runs CorvidLabs/spec-sync pinned at 106aaa43d0bcf049f19b173ddb63815a25c3f931 with archive-gate-only 'true', and every other job in both workflows needs it directly or transitively, so nothing else runs while a change under .specsync/changes/ is open

## Evidence

- Verification commit: `e94b08e7d3e5a93e540cbf28c677b005d4e7b3ec`
- Base commit: `2d174b05a10d5e73250815acaa7d2af8340fa0e8`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

This is part of the org-wide rollout of CorvidLabs/spec-sync#832, approved by 0xLeif. That pull request (merged as 106aaa43d0bcf049f19b173ddb63815a25c3f931) added an archive gate to the spec-sync GitHub Action: with `archive-gate-only: 'true'` the action runs only its archive-readiness script, needs only bash and python3, downloads no binary, and fails while any change under `.specsync/changes/` is still open. The goal is that every SpecSync change is archived before a pull request merges and that CI runs nothing else until it is.

Published @v6.1.1 and older ignore the new input, so the step pins the commit 106aaa43d0bcf049f19b173ddb63815a25c3f931; it should move to the first release that carries the gate. Branch protection, an org ruleset requiring a check named Archive, is configured separately and is not part of this change. Because a skipped job counts as passing, merge blocking relies on that ruleset requiring Archive itself.

The existing `specsync` job in ci.yml, including `scripts/check-specsync-lifecycle.sh`, is kept unchanged; it now also needs the gate. `release.yml` is tag-only and is deliberately left alone.

## From the change's testing.md

# Testing

This is an operations change with no spec in scope, so it proves no product tests.

- Both edited workflows parse with Psych.safe_load, and the printed job graph shows `archive-gate` with no needs, every ci.yml job (`check`, `view`, `lint`, `specsync`) needing `archive-gate`, pages.yml `build` needing `archive-gate` and `deploy` needing `build`.
- `bash .github/scripts/archive-readiness.sh .` from CorvidLabs/spec-sync at the pinned commit fails while this change is open and passes once it is archived.
- `specsync change audit --strict` and `specsync check --strict` (6.1.1) run locally.
- On the pull request, the Archive check runs first and the other ci.yml jobs wait for it.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
