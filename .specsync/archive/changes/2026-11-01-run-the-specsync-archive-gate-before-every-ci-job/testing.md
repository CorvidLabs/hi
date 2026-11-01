---
change: run-the-specsync-archive-gate-before-every-ci-job
artifact: testing
---

# Testing

This is an operations change with no spec in scope, so it proves no product tests.

- Both edited workflows parse with Psych.safe_load, and the printed job graph shows `archive-gate` with no needs, every ci.yml job (`check`, `view`, `lint`, `specsync`) needing `archive-gate`, pages.yml `build` needing `archive-gate` and `deploy` needing `build`.
- `bash .github/scripts/archive-readiness.sh .` from CorvidLabs/spec-sync at the pinned commit fails while this change is open and passes once it is archived.
- `specsync change audit --strict` and `specsync check --strict` (6.1.1) run locally.
- On the pull request, the Archive check runs first and the other ci.yml jobs wait for it.
