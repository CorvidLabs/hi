#!/bin/sh
# Drive the page `hi view` produces in a real browser and assert on what comes
# OUT, not on the HTML that went in.
#
# Every other view test asserts on the generated markup. Two bugs shipped
# anyway, because generating the right markup is not the same as the page
# working:
#
#   - `.row { display: flex }` outranks the user agent's `[hidden]`, so filters
#     updated the count and hid nothing. Shipped in 0.2.0 through 0.2.3.
#   - the role chip rendered with no separator, jammed into the sentence.
#     Shipped in 0.2.0 through 0.2.3.
#
# Both were found by a person taking a screenshot. This script is that person.
set -e

chrome=""
for candidate in \
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
    "/Applications/Chromium.app/Contents/MacOS/Chromium" \
    "$(command -v google-chrome 2>/dev/null)" \
    "$(command -v google-chrome-stable 2>/dev/null)" \
    "$(command -v chromium 2>/dev/null)" \
    "$(command -v chromium-browser 2>/dev/null)"
do
    if [ -n "$candidate" ] && [ -x "$candidate" ]; then chrome="$candidate"; break; fi
done

if [ -z "$chrome" ]; then
    echo "view-behaves: no Chrome or Chromium found, skipping the browser checks." >&2
    echo "view-behaves: install one to run them locally; CI always has one." >&2
    exit 0
fi

root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

cargo build --release --quiet
hi="$PWD/target/release/hi"

# A fixture rather than hi's own criteria, so the assertions do not move every
# time somebody writes a sentence down.
mkdir -p "$root/hi"
cat > "$root/hi/chat.md" <<'MD'
---
hi: 1
families: [SEND]
---

# Chat

## Intent

It should feel like texting.

## Criteria

- **SEND-1**  I hit enter and the message shows up right away.
  - **SEND-1.a**  If I am offline it queues and says so.
- **SEND-2**  It reaches them and the mark changes to sent.

## Retired

- **SEND-9**  My messages auto-delete after a day.
  retired: a different product
MD
cat > "$root/hi/spend.md" <<'MD'
---
hi: 1
families: [SPEND]
---

# Spend

## Intent

Nobody should be surprised by a bill.

## Criteria

- **SPEND-1**  An operator can cap the daily spend.
MD

( cd "$root" && "$hi" view >/dev/null )

# One browser launch runs every check: each mutates the page the way a person
# would, then reads back what is actually visible.
cat > "$root/driver.js" <<'JS'
setTimeout(function () {
  var out = [];
  function check(name, ok, detail) {
    out.push((ok ? "PASS " : "FAIL ") + name + (ok ? "" : ", got: " + detail));
  }
  function visible() {
    return Array.prototype.filter.call(document.querySelectorAll(".row"), function (r) {
      return r.offsetParent !== null || getComputedStyle(r).display !== "none";
    });
  }
  var q = document.getElementById("q");
  var nav = document.querySelectorAll(".navitem");
  var retired = document.getElementById("showretired");
  var count = document.getElementById("count");

  check("controls are revealed once the script runs",
    !document.getElementById("searchwrap").hidden, "searchwrap still hidden");

  var atRest = visible().length;
  check("at rest every active criterion is visible", atRest === 4, "saw " + atRest + " of 4");
  check("retired criteria start hidden",
    !visible().some(function (r) { return r.dataset.retired === "1"; }), "a retired row was visible");

  // The bug that shipped four times: filtering must actually hide rows.
  nav[2].click();
  var filtered = visible();
  check("clicking a feature hides the other feature's rows",
    filtered.length === 1 && filtered[0].dataset.id === "SPEND-1",
    "saw " + filtered.length + " rows: " + filtered.map(function (r) { return r.dataset.id; }).join(","));
  check("the count agrees with what is on screen",
    /1 of 5 shown/.test(count.textContent), "count said: " + count.textContent);

  nav[0].click();
  check("All criteria puts them back", visible().length === 4, "saw " + visible().length);

  q.value = "offline";
  q.dispatchEvent(new Event("input"));
  var searched = visible();
  check("search hides everything that does not match",
    searched.length === 1 && searched[0].dataset.id === "SEND-1.a",
    "saw " + searched.map(function (r) { return r.dataset.id; }).join(","));
  check("the matched words are highlighted where they sit",
    searched.length === 1 && searched[0].querySelector("mark") &&
      searched[0].querySelector("mark").textContent === "offline",
    "no <mark> around the match");

  q.value = "";
  q.dispatchEvent(new Event("input"));
  check("clearing the search restores every row", visible().length === 4, "saw " + visible().length);
  check("clearing the search removes the highlights",
    document.querySelectorAll("mark").length === 0, "a <mark> survived");

  retired.checked = true;
  retired.dispatchEvent(new Event("change"));
  check("the retired toggle reveals them", visible().length === 5, "saw " + visible().length);
  retired.checked = false;
  retired.dispatchEvent(new Event("change"));

  document.dispatchEvent(new KeyboardEvent("keydown", { key: "j", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "j", bubbles: true }));
  var cursor = document.querySelector(".row.cursor");
  check("j moves a cursor through the criteria",
    cursor && cursor.dataset.id === "SEND-1.a", "cursor on " + (cursor ? cursor.dataset.id : "nothing"));

  document.dispatchEvent(new KeyboardEvent("keydown", { key: "/", bubbles: true }));
  check("slash focuses the search box", document.activeElement === q, "focus was elsewhere");

  // A shared link has to land even when a filter would have hidden the row.
  q.blur();
  nav[1].click();
  location.hash = "#SPEND-1";
  window.dispatchEvent(new HashChangeEvent("hashchange"));
  var target = document.getElementById("SPEND-1");
  check("a deep link reveals a row a filter would have hidden",
    target && visible().indexOf(target) !== -1, "SPEND-1 stayed hidden");

  // One line: a multi-line body is easy to half-read with sed and then report
  // a green run off the first result.
  document.body.textContent = "VIEWCHECKS " + out.join(" @@ ");
}, 80);
JS

python3 - "$root" <<'PY'
import sys, pathlib
root = pathlib.Path(sys.argv[1])
page = (root / "intent.html").read_text()
driver = (root / "driver.js").read_text()
(root / "probe.html").write_text(page.replace("</body>", "<script>" + driver + "</script>\n</body>"))
PY

# Chrome is run in the background with a hard deadline and killed if it
# overruns. A browser that never exits would otherwise hang the whole gate,
# which is a worse failure than the bugs this script exists to catch.
dom="$root/dom.html"
# No --user-data-dir: pointing headless Chrome at a fresh profile deadlocks on
# this page, which writes localStorage from the theme toggle. The default
# profile is fine for a read-only DOM dump.
"$chrome" --headless --disable-gpu --no-sandbox --virtual-time-budget=1500 \
    --dump-dom "file://$root/probe.html" > "$dom" 2>/dev/null &
pid=$!

waited=0
while kill -0 "$pid" 2>/dev/null && [ "$waited" -lt 60 ]; do
    sleep 1
    waited=$((waited + 1))
done
if kill -0 "$pid" 2>/dev/null; then
    kill -9 "$pid" 2>/dev/null || true
    echo "view-behaves: the browser did not exit within 60s; killed it." >&2
    exit 1
fi
wait "$pid" 2>/dev/null || true

results="$(tr -d '\n' < "$dom" \
    | sed -n 's/.*VIEWCHECKS //p' | sed 's|</body>.*||' | sed 's/<[^>]*>//g' \
    | sed 's/ @@ /\n/g' | sed 's/^ *//; s/ *$//' | grep -v '^$')"

if [ -z "$results" ]; then
    echo "view-behaves: the page produced no results; the driver did not run." >&2
    exit 1
fi

echo "$results" | sed 's/^/  /'

if echo "$results" | grep -q '^FAIL'; then
    echo >&2
    echo "view-behaves: the page does not behave the way its HTML claims." >&2
    exit 1
fi

passed="$(echo "$results" | grep -c '^PASS')"
echo "view-behaves: $passed browser checks passed."
