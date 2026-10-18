---
change: adopt-verified-specsync-6-1-1-approval-review-learning-and-archive-lifecycle-for-hi
artifact: design
---

# Design

## Operations design

Enable both SDD flags for repository delivery only. Add isolated checksum-pinned 6.1.1 canonical/audit/archive CI job, retaining all platform build/test, browser and lint jobs. Cover scripts/, bin/, specs/, governance and manifests. No hi product lifecycle or evidence features; DECISIONS.md explicitly keeps those out of hi.

The guard runs separately from canonical validation and product tasks to avoid recursive verification. It checks exact engine, required flags and native strict active audit, then refuses unfinished active workspaces. It never changes state, signs evidence, or repairs archives. Inherited debt is a blocker rather than a baseline exemption.

Learning stays in native change context/lesson bundles and applicable existing skills. Stronger risk and provenance controls remain unchanged.
