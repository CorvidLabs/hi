#!/usr/bin/env bash
set -euo pipefail
source_root=$(cd "$(dirname "$0")/.." && pwd)
engine=${SPECSYNC_BIN:-$(command -v specsync)}
engine=$(realpath "$engine")
scratch=$(mktemp -d)
fixture=$scratch/repo
export BASELINE=$scratch/baseline.json
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$fixture/.specsync" "$fixture/scripts"
cp "$source_root/.specsync/sdd.json" "$fixture/.specsync/sdd.json"
cp "$source_root/scripts/check-specsync-lifecycle.sh" "$fixture/scripts/"
cd "$fixture"
git init -q
git -c user.name=fixture -c user.email=fixture@example.invalid add .
git -c user.name=fixture -c user.email=fixture@example.invalid commit -qm 'clean policy fixture (no human approval records)'
pass() { SPECSYNC_BIN="$engine" bash scripts/check-specsync-lifecycle.sh >"$scratch/result.log" 2>&1 || { cat "$scratch/result.log"; exit 1; }; echo "PASS $1"; }
fail() {
  if SPECSYNC_BIN="$engine" bash scripts/check-specsync-lifecycle.sh >"$scratch/result.log" 2>&1; then echo "unexpected pass: $1" >&2; exit 1; fi
  grep -q "$2" "$scratch/result.log" || { cat "$scratch/result.log"; exit 1; }
  echo "PASS negative: $1"
}
git update-ref refs/remotes/origin/main HEAD
pass 'clean no-active-workspace fixture'
cp .specsync/sdd.json "$BASELINE"
for flag in enabled require_change_for_meaningful_files; do
  python3 - "$flag" <<'PY'
import json,sys,os
p=json.load(open(os.environ['BASELINE'])); p[sys.argv[1]]=False
open('.specsync/sdd.json','w').write(json.dumps(p))
PY
  fail "$flag disabled" 'Required policy disabled'
done
for path in src/ tests/ AGENTS.md; do
  python3 - "$path" <<'PY'
import json,sys,os
p=json.load(open(os.environ['BASELINE']));p['meaningful_paths']=[v for v in p['meaningful_paths'] if v not in (sys.argv[1], '.')]
open('.specsync/sdd.json','w').write(json.dumps(p))
PY
  fail "$path uncovered" 'Uncovered meaningful path'
done
cp "$BASELINE" .specsync/sdd.json
python3 - <<'PY'
import json,os
p=json.load(open(os.environ['BASELINE']));p['ignored_paths'].append('src/')
open('.specsync/sdd.json','w').write(json.dumps(p))
PY
fail 'ignored source prefix' 'Unapproved meaningful-path exclusion'
for omitted in src/uncovered.ts src/nested/; do
  python3 - "$omitted" <<'PYCONTROL'
import json,os,sys
p=json.load(open(os.environ['BASELINE']));p['ignored_paths'].append(sys.argv[1])
open('.specsync/sdd.json','w').write(json.dumps(p))
PYCONTROL
  fail "ignored descendant $omitted" 'Unapproved meaningful-path exclusion'
done
cp "$BASELINE" .specsync/sdd.json
for path in src/uncovered.rs tests/uncovered.rs AGENTS.md; do
  mkdir -p "$(dirname "$path")"
  printf 'uncovered negative fixture\n' > "$path"
  git add "$path"
  git -c user.name=fixture -c user.email=fixture@example.invalid commit -qm 'uncovered change'
  fail "native committed $path coverage" 'meaningful changed paths are not covered'
  git reset --hard -q refs/remotes/origin/main
done
mkdir -p .specsync/changes/unfinalized
fail 'unfinished workspace' 'Unfinished active workspaces'
rmdir .specsync/changes/unfinalized
printf '#!/bin/sh\nprintf "specsync 6.1.0\\n"\n' > "$scratch/wrong-engine"
chmod +x "$scratch/wrong-engine"
real_engine=$engine
engine=$scratch/wrong-engine
fail 'wrong version response (stub)' 'Expected specsync 6.1.1'
engine=$real_engine
pass 'read-only archive-ready follow-up'
echo 'Human-reviewed finalization fixture remains manual; no acceptance was invented.'
