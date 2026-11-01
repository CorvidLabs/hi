---
id: run-the-specsync-archive-gate-before-every-ci-job
state: implementing
type: operations
base_commit: 2d174b05a10d5e73250815acaa7d2af8340fa0e8
---

# Run the SpecSync archive gate before every CI job

## Intent

Run the SpecSync archive gate before every CI job

## Affected Canonical Specs

- None

## Acceptance Criteria

- ci.yml and pages.yml each start with an archive-gate job named Archive that runs CorvidLabs/spec-sync pinned at 106aaa43d0bcf049f19b173ddb63815a25c3f931 with archive-gate-only 'true', and every other job in both workflows needs it directly or transitively, so nothing else runs while a change under .specsync/changes/ is open

## No-spec Rationale

CI workflow ordering only; no canonical spec owns .github/workflows
