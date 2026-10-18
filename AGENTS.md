# Repository delivery instructions

## Required SpecSync 6.1.1 delivery lifecycle

Use a checksum-verified `specsync` 6.1.1 writer; check `specsync --version` before changing lifecycle records. CI pins the engine separately from action code. `.specsync/version` is an on-disk lifecycle format stamp, never an engine selector.

Every meaningful source, test, public documentation or governance change requires a native change definition. Read applicable intent, specs and prior lessons, finish the interview and artifacts, and obtain actual human definition approval before implementing. Record the real approving person through `change approve --actor`; never infer implementation acceptance from that scope approval.

Run product checks separately from canonical `specsync check --force --strict`, scoped `specsync change check <id> --strict --commit`, and global `specsync change audit --strict`. An operations check with no spec in scope proves no product tests. `bash scripts/check-specsync-lifecycle.sh` is the non-mutating CI archive-readiness gate; it fails while any active workspace remains. Inherited failures stay visible and require their owners, never exclusions or fabricated history.

Have a human inspect the current implementation, change package, applicable canonical delta and evidence before recording `change review --reviewer` with their real identity. Capture observed lessons, actual evidence, applicability limits and remaining questions in the native change context and existing lesson surfaces. Finalize/archive on the same PR only after genuine review and verification. Do not commit between review and finalization. A definition approval or agent review is not human implementation review.

Preserve existing product checks and stronger risk/provenance policy. Missing prerequisites or skipped tests remain explicit limitations. Never modify credentials, branch protections, production services or deployment state to qualify a change.
