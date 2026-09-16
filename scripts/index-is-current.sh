#!/bin/sh
# Fail if INTENT.md's generated feature list is out of date.
#
# The list is generated, so it can fall behind the criteria it counts without
# anyone noticing. It did: the committed index claimed 114 criteria against an
# actual 119, across three published surfaces that each said a different number.
#
# `git diff` against HEAD is the wrong check, because it also fails on an
# INTENT.md you are legitimately part-way through editing. The question is
# narrower: does regenerating change anything? If it does, the file on disk was
# stale whether or not it was committed.
set -e

before="$(mktemp)"
trap 'rm -f "$before"' EXIT
cp INTENT.md "$before"

cargo run --quiet -- index >/dev/null

if ! diff -q "$before" INTENT.md >/dev/null 2>&1; then
    echo "INTENT.md's index was stale; hi index has just rewritten it:" >&2
    diff "$before" INTENT.md >&2 || true
    echo >&2
    echo "Commit the regenerated INTENT.md." >&2
    exit 1
fi
