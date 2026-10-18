# Lesson bundle — adopt-verified-specsync-6-1-1-approval-review-learning-and-archive-lifecycle-for-hi

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Adopt verified SpecSync 6.1.1 approval review learning and archive lifecycle for hi
- **Kind**: Operations
- **Paths**: .github/workflows/ci.yml, CLAUDE.md, .specsync/sdd.json, AGENTS.md, scripts/check-specsync-lifecycle.sh, scripts/test-specsync-lifecycle.sh, .codex/skills/spec-sync/SKILL.md
- **Acceptance**: Exact 6.1.1 writer and action inputs; required meaningful source/test/governance coverage; genuine human definition approval before implementation; separate current scoped human review of the implementation; actual product tests distinct from canonical checks and active-workspace audit; same-PR finalize/archive before merge; read prior lessons and record observed findings with evidence and limits. Preserve all stronger existing checks, risk thresholds, signer policies, archives and unrelated work. A non-mutating lifecycle guard fails for a wrong engine, disabled policy, uncovered meaningful path or unfinished active workspace and passes a clean finalized fixture. No fabricated approvals or inherited-history repairs. Human review and finalization remain pending until they actually happen.

## Evidence

- Verification commit: `47660ebb0842a34fe860ace2a390cea429f0ac1d`
- Base commit: `a65a4a1a28c8dd0c204f6e58be30ecba4f6350ab`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

## Status

Prepared by agent:codex using the authenticated corvid-agent GitHub account. Proposed interview answers are agent drafting from the user request, not a new human decision. Definition approval: pending. Implementation acceptance/review: pending. No approve, review, check, finalize, push or PR publication was performed for this change.

Exact base: `a65a4a1a28c8dd0c204f6e58be30ecba4f6350ab` on `main`; isolated branch `codex/specsync-611-definition`.

## Ownership

No open PR at capture. No root AGENTS.md or installed SpecSync skill exists; add bounded delivery instructions without altering hi/ intent or product decisions. Browser prerequisites needed for full lane.

Host-Mac _notes #69 comment 6245284491 owns Bot policy; Raven #418 is merged and excluded. Site #537 is closed unmerged. The 197-repository audit is not this target list.

## Evidence-backed observations

The isolated official Linux aarch64 6.1.1 archive passed its published SHA-256 check and reports specsync 6.1.1. The global PATH writer is 6.1.0 and must not write these records. Arcsite Trust 1.2.0 action.yml supports specsync-version. Broad rollout permission does not approve these newly written definitions.

## Learning plan

Read the existing instruction/spec context and cited lessons before implementing. Record observed failure, fix, command/evidence, applicability limits and remaining question in this change context. Promote only reusable supported findings; do not claim a new product lesson from an unrun test. Preserve historical lesson bundles.

- No archived lesson-bundle.md found; consult repository instructions and current spec context before implementation.

## Resume

Use the isolated 6.1.1 binary with `change status adopt-verified-specsync-6-1-1-approval-review-learning-and-archive-lifecycle-for-hi`. Definition approval and implementation review remain separate human decisions.

## Runtime observations (2026-10-18)

Definition approval was genuinely recorded for 0xLeif based on the relayed 14:07 UTC
user response. No human implementation review or finalization has occurred.
The root agent implemented only the approved paths; independent local qualification
preserves product source, tests, hi intent and DECISIONS.md. No historic lessons
were available; existing repository intent, delivery instructions and decisions
were read rather than manufacturing a product lesson.

The initial CLI outside-repository test failed because host /tmp/.git exists,
causing the fixture to find a Git boundary as designed. An isolated TMPDIR under
/var/tmp outside Git ancestors fixes the environment without weakening the test;
all cargo tests then pass. Do not delete shared /tmp/.git or modify product code
to hide this environment problem. This isolation technique applies only when the
outside-repository assumption is independently checked.

The exact6.1.1 verify lane passes, but its browser script explicitly skips because
Chrome/Chromium is absent. This is not browser acceptance; hosted browser CI must
supply that evidence. Fresh main remains a65a4a1a28c8dd0c204f6e58be30ecba4f6350ab,
with no open PR at recheck. Human review and same-PR finalize/archive stay pending.

## From the change's design.md

# Design

## Operations design

Enable both SDD flags for repository delivery only. Add isolated checksum-pinned 6.1.1 canonical/audit/archive CI job, retaining all platform build/test, browser and lint jobs. Cover scripts/, bin/, specs/, governance and manifests. No hi product lifecycle or evidence features; DECISIONS.md explicitly keeps those out of hi.

The guard runs separately from canonical validation and product tasks to avoid recursive verification. It checks exact engine, required flags and native strict active audit, then refuses unfinished active workspaces. It never changes state, signs evidence, or repairs archives. Inherited debt is a blocker rather than a baseline exemption.

Learning stays in native change context/lesson bundles and applicable existing skills. Stronger risk and provenance controls remain unchanged.

## From the change's testing.md

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

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
