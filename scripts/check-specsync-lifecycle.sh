#!/usr/bin/env bash
# Read-only qualification: no approvals, repairs, signing or archival.
set -euo pipefail
engine=${SPECSYNC_BIN:-specsync}
[[ "$("$engine" --version)" == "specsync 6.1.1" ]] || { echo 'Expected specsync 6.1.1' >&2; exit 1; }
python3 - <<'POLICY'
import json
from pathlib import Path
policy = json.loads(Path('.specsync/sdd.json').read_text())
for flag in ('enabled', 'require_change_for_meaningful_files'):
    if policy.get(flag) is not True:
        raise SystemExit('Required policy disabled: ' + flag)
required = ['src/', 'tests/', 'site/', '.github/', 'Cargo.toml', 'Cargo.lock', 'action.yml', 'package.json', 'bun.lock', 'package-lock.json', 'pnpm-lock.yaml', 'yarn.lock', 'Package.swift', 'Package.resolved', 'go.mod', 'go.sum', 'pyproject.toml', 'uv.lock', 'requirements.txt', '.specsync/sdd.json', '.specsync/config.toml', '.specsync/config.json', '.specsync/registry.toml', '.specsync/version', 'AGENTS.md', 'CLAUDE.md', '.agents/', '.codex/', '.claude/', 'scripts/', 'fledge.toml', '.trust.toml', '.augur.toml', '.attest.json', 'specs/', 'bin/', 'docs/', 'README.md', 'DECISIONS.md']
paths = policy.get('meaningful_paths', [])
ignored = policy.get('ignored_paths', [])
allowed_ignores = {'.specsync/changes/', '.specsync/archive/', '.specsync/hashes.json', '.specsync/change.lock', '.specsync/change-sequence.json', '.specsync/adoption-report.json'}
if any(path not in allowed_ignores for path in ignored):
    raise SystemExit('Unapproved meaningful-path exclusion')
for path in required:
    if not any(p == '.' or path == p or (p.endswith('/') and path.startswith(p)) for p in paths):
        raise SystemExit('Uncovered meaningful path: ' + path)
    if any(p == '.' or path == p or (p.endswith('/') and path.startswith(p)) for p in ignored):
        raise SystemExit('Ignored meaningful path: ' + path)
POLICY
"$engine" change audit --strict
python3 - <<'WORKSPACES'
from pathlib import Path
root = Path('.specsync/changes')
active = sorted(p.name for p in root.iterdir() if p.is_dir() and p.name != 'archive') if root.exists() else []
if active:
    raise SystemExit('Unfinished active workspaces: ' + ', '.join(active))
print('SpecSync lifecycle and archive readiness passed')
WORKSPACES
