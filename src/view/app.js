// Search, navigate, sort, filter and deep-link. Inline on purpose: the page has
// to work from an email attachment with no network (hi: VIEW-2). Everything
// here only hides rows that are already in the document, so with scripting off
// a reader still sees every criterion (hi: VIEW-10).
(function () {
  "use strict";

  var q = document.getElementById("q");
  var searchwrap = document.getElementById("searchwrap");
  var railfoot = document.getElementById("railfoot");
  var sort = document.getElementById("sort");
  var showRetired = document.getElementById("showretired");
  var count = document.getElementById("count");
  var features = document.getElementById("features");
  var flat = document.getElementById("flat");
  var nomatch = document.getElementById("nomatch");
  var toast = document.getElementById("toast");
  var nav = document.getElementById("nav");
  if (!features || !nav || !q) return;

  var rows = Array.prototype.slice.call(features.querySelectorAll(".row"));
  var sections = Array.prototype.slice.call(features.querySelectorAll(".feature"));
  var items = Array.prototype.slice.call(nav.querySelectorAll(".navitem"));
  var clears = Array.prototype.slice.call(document.querySelectorAll(".clear"));
  var home = rows.map(function (row) { return row.parentNode; });
  var lead = document.querySelector(".lead");
  var intents = Array.prototype.slice.call(features.querySelectorAll(".intent"));
  // The sentence exactly as hi rendered it, so highlighting can be undone
  // without ever re-parsing a criterion's own markup (hi: VIEW-3.a).
  var pristine = rows.map(function (row) {
    var text = row.querySelector(".ctext");
    return text ? text.innerHTML : "";
  });

  // Only reveal what the script can actually operate (hi: VIEW-10).
  searchwrap.hidden = false;
  railfoot.hidden = false;

  var file = "";
  var cursor = -1;

  function matches(row, needle) {
    if (row.dataset.retired === "1" && !showRetired.checked) return false;
    if (file && row.dataset.file !== file) return false;
    if (needle && row.dataset.find.indexOf(needle) === -1) return false;
    return true;
  }

  function byId(a, b) {
    return a.dataset.id.localeCompare(b.dataset.id, undefined, { numeric: true });
  }

  // Wrap every occurrence of the needle in <mark>, walking text nodes only, so
  // a criterion's own <code> or <a> is never cut in half (hi: VIEW-13).
  function highlight(row, needle) {
    var host = row.querySelector(".ctext");
    if (!host) return;
    var walker = document.createTreeWalker(host, NodeFilter.SHOW_TEXT, null, false);
    var targets = [];
    var node;
    while ((node = walker.nextNode())) {
      if (node.nodeValue.toLowerCase().indexOf(needle) !== -1) targets.push(node);
    }
    targets.forEach(function (text) {
      var frag = document.createDocumentFragment();
      var rest = text.nodeValue;
      var at = rest.toLowerCase().indexOf(needle);
      while (at !== -1) {
        if (at > 0) frag.appendChild(document.createTextNode(rest.slice(0, at)));
        var hit = document.createElement("mark");
        hit.textContent = rest.slice(at, at + needle.length);
        frag.appendChild(hit);
        rest = rest.slice(at + needle.length);
        at = rest.toLowerCase().indexOf(needle);
      }
      if (rest) frag.appendChild(document.createTextNode(rest));
      text.parentNode.replaceChild(frag, text);
    });
  }

  function apply() {
    var needle = q.value.trim().toLowerCase();
    var mode = sort.value;
    var shown = 0;

    rows.forEach(function (row, i) {
      var ok = matches(row, needle);
      row.hidden = !ok;
      if (ok) shown++;
      var text = row.querySelector(".ctext");
      if (text) {
        text.innerHTML = pristine[i];
        if (ok && needle) highlight(row, needle);
      }
    });

    if (mode === "doc") {
      // Put every row back where it was written, and hide a feature whose rows
      // are all filtered out.
      rows.forEach(function (row, i) {
        if (row.parentNode !== home[i]) home[i].appendChild(row);
      });
      flat.hidden = true;
      features.hidden = false;
      sections.forEach(function (section) {
        var any = Array.prototype.some.call(
          section.querySelectorAll(".row"),
          function (row) { return !row.hidden; }
        );
        section.hidden = !any;
      });
    } else {
      var ordered = rows.slice().sort(function (a, b) {
        if (mode === "id") return byId(a, b);
        var left = a.dataset.family || "";
        var right = b.dataset.family || "";
        return left === right ? byId(a, b) : left.localeCompare(right);
      });
      ordered.forEach(function (row) { flat.appendChild(row); });
      features.hidden = true;
      flat.hidden = false;
    }

    var filtering = needle !== "" || file !== "";
    // Prose earns its place on the page you opened, not on top of the answer
    // you went looking for. The product why goes away as soon as you narrow
    // anything; a feature's own why survives until you search.
    if (lead) lead.hidden = filtering;
    intents.forEach(function (block) { block.hidden = needle !== ""; });
    // `shown` already accounts for the retired toggle, so it is the honest
    // number at rest too: 106 criteria until you ask to see the retired ones.
    count.textContent = filtering
      ? shown + " of " + rows.length + " shown"
      : shown + " criteria";
    clears.forEach(function (button) {
      if (button.id === "clear") button.hidden = !filtering && !showRetired.checked;
    });
    nomatch.hidden = shown !== 0;
    // Only one thing in the rail is ever lit: the feature you filtered to, or,
    // when nothing is filtered, the one you are currently reading.
    items.forEach(function (item) {
      var on = file !== "" && item.dataset.value === file;
      item.setAttribute("aria-pressed", on ? "true" : "false");
    });
    if (cursor !== -1 && rows[cursor] && rows[cursor].hidden) setCursor(-1);
    here();
  }

  // Which feature is being read, as distinct from which one is filtered to
  // (hi: VIEW-12.b).
  function here() {
    if (file || features.hidden) {
      items.forEach(function (item) { item.removeAttribute("data-here"); });
      return;
    }
    var best = null;
    sections.forEach(function (section) {
      if (section.hidden) return;
      if (section.getBoundingClientRect().top - 90 <= 0) best = section.dataset.file;
    });
    if (!best) {
      var first = sections.filter(function (s) { return !s.hidden; })[0];
      best = first ? first.dataset.file : null;
    }
    items.forEach(function (item) {
      if (item.dataset.value && item.dataset.value === best) {
        item.setAttribute("data-here", "true");
      } else {
        item.removeAttribute("data-here");
      }
    });
  }

  function setCursor(next) {
    if (rows[cursor]) rows[cursor].classList.remove("cursor");
    cursor = next;
    if (rows[cursor]) {
      rows[cursor].classList.add("cursor");
      rows[cursor].scrollIntoView({ block: "nearest" });
    }
  }

  function step(delta) {
    var visible = rows.filter(function (row) { return !row.hidden; });
    if (!visible.length) return;
    var at = visible.indexOf(rows[cursor]);
    var next = at === -1 ? (delta > 0 ? 0 : visible.length - 1) : at + delta;
    if (next < 0) next = 0;
    if (next > visible.length - 1) next = visible.length - 1;
    setCursor(rows.indexOf(visible[next]));
  }

  function say(message) {
    toast.textContent = message;
    toast.hidden = false;
    clearTimeout(say.timer);
    say.timer = setTimeout(function () { toast.hidden = true; }, 1600);
  }

  // A link somebody can paste (hi: VIEW-15). Clipboard access is not granted
  // everywhere a self-contained file gets opened, so a refusal still leaves the
  // anchor working and still says which id was clicked.
  function copyLink(id) {
    var url = location.href.split("#")[0] + "#" + id;
    var done = function () { say("Link to " + id + " copied"); };
    // The anchor has already put the link in the address bar, so a refused
    // clipboard is a smaller problem than it sounds. Say which, either way.
    var manual = function () { say(id + " is in the address bar"); };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(url).then(done, manual);
      return;
    }
    try {
      var pad = document.createElement("textarea");
      pad.value = url;
      pad.setAttribute("readonly", "");
      pad.style.position = "fixed";
      pad.style.opacity = "0";
      document.body.appendChild(pad);
      pad.select();
      document.execCommand("copy");
      document.body.removeChild(pad);
      done();
    } catch (err) {
      manual();
    }
  }

  items.forEach(function (item) {
    item.setAttribute("aria-pressed", "false");
    item.addEventListener("click", function () {
      // Clicking the feature you are already filtered to returns you to
      // everything, so the rail is never a dead end (hi: VIEW-7).
      file = file === item.dataset.value ? "" : item.dataset.value;
      setCursor(-1);
      apply();
      window.scrollTo({ top: 0 });
    });
  });

  q.addEventListener("input", apply);
  sort.addEventListener("change", apply);
  showRetired.addEventListener("change", apply);

  function reset() {
    q.value = "";
    showRetired.checked = false;
    file = "";
    setCursor(-1);
    apply();
  }
  clears.forEach(function (button) {
    button.addEventListener("click", function () {
      reset();
      q.focus();
    });
  });

  document.addEventListener("click", function (event) {
    var link = event.target.closest ? event.target.closest(".cid") : null;
    if (!link) return;
    copyLink(link.getAttribute("href").replace(/^#/, ""));
  });

  window.addEventListener("scroll", function () { here(); }, { passive: true });

  // Reading the page from the keyboard (hi: VIEW-14).
  document.addEventListener("keydown", function (event) {
    var typing = document.activeElement === q;
    if (event.key === "Escape") {
      if (typing || q.value || file || showRetired.checked) {
        event.preventDefault();
        reset();
        q.blur();
      }
      return;
    }
    if (event.key === "/" && !typing) {
      event.preventDefault();
      q.focus();
      q.select();
      return;
    }
    if (typing && event.key === "Enter") {
      event.preventDefault();
      step(1);
      return;
    }
    if (typing || event.metaKey || event.ctrlKey || event.altKey) return;
    if (event.key === "j" || event.key === "ArrowDown") {
      event.preventDefault();
      step(1);
    } else if (event.key === "k" || event.key === "ArrowUp") {
      event.preventDefault();
      step(-1);
    } else if (event.key === "Enter" && rows[cursor]) {
      event.preventDefault();
      copyLink(rows[cursor].dataset.id);
    }
  });

  // A link to a criterion has to land on it even when filters would hide it,
  // otherwise a shared link silently shows nothing (hi: VIEW-9.a). Filters are
  // only cleared when they are actually in the way, so clicking an id on a
  // page you have narrowed does not throw that away.
  function reveal() {
    var id = decodeURIComponent(location.hash.replace(/^#/, ""));
    if (!id) return;
    var row = document.getElementById(id);
    if (!row) return;
    var blocked = row.hidden;
    if (row.dataset.retired === "1" && !showRetired.checked) {
      showRetired.checked = true;
      blocked = true;
    }
    if (blocked) {
      q.value = "";
      file = "";
    }
    apply();
    row.scrollIntoView({ block: "center" });
    row.classList.add("flash");
    setCursor(rows.indexOf(row));
    setTimeout(function () { row.classList.remove("flash"); }, 1600);
  }

  window.addEventListener("hashchange", reveal);
  apply();
  reveal();
})();
