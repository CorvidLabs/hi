---
change: adopt-verified-specsync-6-1-1-approval-review-learning-and-archive-lifecycle-for-hi
artifact: plan
---

# Plan

## Approach

Enable both SDD flags for repository delivery only. Add isolated checksum-pinned 6.1.1 canonical/audit/archive CI job, retaining all platform build/test, browser and lint jobs. Cover scripts/, bin/, specs/, governance and manifests. No hi product lifecycle or evidence features; DECISIONS.md explicitly keeps those out of hi.

Exact 6.1.1 writer and action inputs; required meaningful source/test/governance coverage; genuine human definition approval before implementation; separate current scoped human review of the implementation; actual product tests distinct from canonical checks and active-workspace audit; same-PR finalize/archive before merge; read prior lessons and record observed findings with evidence and limits. Preserve all stronger existing checks, risk thresholds, signer policies, archives and unrelated work.

## Out of scope

No product functional behavior, dependency upgrades, credentials, branch protections, main pushes, merges, tags, releases, deployments, VPS access, production services or historical acceptance repairs. No Bot #230 or paused AlgoChat Fledge work. Keep .specsync/version bytes unchanged; it is a lifecycle format stamp.

## Steps

1. Human inspects and approves this exact definition; resolve overlaps before implementation.
2. Pin only declared selectors and regenerate installed agent integrations with verified 6.1.1; inspect generated diff and retain local guidance.
3. Add required meaningful path coverage and a non-mutating lifecycle guard. Preserve existing workflow jobs.
4. Run product gates, canonical check and active audit separately; disclose baseline debt and resolve it only through separately authorized owner work.
5. Publish a tested draft PR only after applicable publication checks pass. Obtain actual current human implementation review.
6. Record only authentic review; finalize/archive on the same PR after evidence and lessons are ready. No merge in this scope.

## Risks

Inherited active records may block a strict global audit/archive guard. Do not accept them retroactively or add exclusions. The rollout stays blocked where stronger enforcement exposes debt. Linux cannot establish macOS or browser visual acceptance.

## Constraints consulted

No open PR at capture. No root AGENTS.md or installed SpecSync skill exists; add bounded delivery instructions without altering hi/ intent or product decisions. Browser prerequisites needed for full lane.

- No archived lesson-bundle.md found; consult repository instructions and current spec context before implementation.
