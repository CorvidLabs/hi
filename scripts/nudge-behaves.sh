#!/bin/sh
# Drives bin/fledge-hi-nudge the way fledge drives it, and asserts on what a
# person would see.
#
# The thing worth testing is not the wording, it is that every path exits 0. A
# fledge lifecycle hook that exits non-zero aborts the command that ran it, so a
# bug in here would block `fledge work push` (hi: HABIT-4.a).
set -eu

NUDGE="$(cd "$(dirname "$0")/.." && pwd)/bin/fledge-hi-nudge"
TMP="${TMPDIR:-/tmp}/hi-nudge-$$"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/repo/.git"

fail=0
pass() { echo "  PASS $1"; }
bad() { echo "  FAIL $1"; fail=1; }

# Exit status first, because it is the one that can break somebody's push.
for moment in start push; do
  for env_root in "$TMP/repo" "" "/nope/does/not/exist"; do
    FLEDGE_REPO_ROOT="$env_root" "$NUDGE" "$moment" >/dev/null 2>&1
    [ $? -eq 0 ] || bad "$moment with root '$env_root' must exit 0"
  done
done
pass "every path exits 0, so a hook can never abort the command"

out=$(FLEDGE_REPO_ROOT="$TMP/repo" "$NUDGE" start 2>&1)
case "$out" in *"No hi/ here"*) pass "a repo with nothing written down is told so at work start";;
  *) bad "expected the start nudge, got: $out";; esac

out=$(FLEDGE_REPO_ROOT="$TMP/repo" "$NUDGE" push 2>&1)
case "$out" in *"before it ships"*) pass "and told again, differently, before it ships";;
  *) bad "expected the push nudge, got: $out";; esac

# Nothing may reach stdout: `fledge work start --json` puts a JSON envelope
# there and a hook writing into it would corrupt the envelope.
out=$(FLEDGE_REPO_ROOT="$TMP/repo" "$NUDGE" start 2>/dev/null)
[ -z "$out" ] && pass "says nothing on stdout, so a --json envelope stays valid" \
  || bad "wrote to stdout: $out"

mkdir -p "$TMP/repo/hi"
out=$(FLEDGE_REPO_ROOT="$TMP/repo" "$NUDGE" start 2>&1)
[ -z "$out" ] && pass "goes quiet once something is written down (HABIT-4.b)" \
  || bad "should be silent with a hi/ present, got: $out"

rm -rf "$TMP/repo/hi" "$TMP/repo/.git"
out=$(FLEDGE_REPO_ROOT="$TMP/repo" "$NUDGE" start 2>&1)
[ -z "$out" ] && pass "says nothing outside a repository" || bad "spoke outside a repo: $out"

out=$(FLEDGE_REPO_ROOT="" "$NUDGE" start 2>&1)
[ -z "$out" ] && pass "says nothing on a fledge too old to name the repository" \
  || bad "guessed at a repository it was not told about: $out"

[ "$fail" -eq 0 ] || { echo "nudge-behaves: FAILED"; exit 1; }
echo "nudge-behaves: 7 checks passed."
