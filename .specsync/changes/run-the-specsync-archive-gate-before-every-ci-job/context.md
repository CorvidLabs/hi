---
change: run-the-specsync-archive-gate-before-every-ci-job
artifact: context
---

# Context

This is part of the org-wide rollout of CorvidLabs/spec-sync#832, approved by 0xLeif. That pull request (merged as 106aaa43d0bcf049f19b173ddb63815a25c3f931) added an archive gate to the spec-sync GitHub Action: with `archive-gate-only: 'true'` the action runs only its archive-readiness script, needs only bash and python3, downloads no binary, and fails while any change under `.specsync/changes/` is still open. The goal is that every SpecSync change is archived before a pull request merges and that CI runs nothing else until it is.

Published @v6.1.1 and older ignore the new input, so the step pins the commit 106aaa43d0bcf049f19b173ddb63815a25c3f931; it should move to the first release that carries the gate. Branch protection, an org ruleset requiring a check named Archive, is configured separately and is not part of this change. Because a skipped job counts as passing, merge blocking relies on that ruleset requiring Archive itself.

The existing `specsync` job in ci.yml, including `scripts/check-specsync-lifecycle.sh`, is kept unchanged; it now also needs the gate. `release.yml` is tag-only and is deliberately left alone.
