// Search, sort, filter and deep-link. Inline on purpose: the page has to work
// from an email attachment with no network (hi: VIEW-2). Everything here only
// hides rows that are already in the document, so with scripting off a reader
// still sees every criterion.
(function () {
  "use strict";

  var controls = document.getElementById("controls");
  var q = document.getElementById("q");
  var sort = document.getElementById("sort");
  var showRetired = document.getElementById("showretired");
  var clear = document.getElementById("clear");
  var count = document.getElementById("count");
  var features = document.getElementById("features");
  var flat = document.getElementById("flat");
  var nomatch = document.getElementById("nomatch");
  if (!controls || !features) return;

  var rows = Array.prototype.slice.call(features.querySelectorAll(".row"));
  var sections = Array.prototype.slice.call(features.querySelectorAll(".feature"));
  var chips = Array.prototype.slice.call(document.querySelectorAll(".chip"));
  var home = rows.map(function (row) { return row.parentNode; });

  controls.hidden = false;

  var active = { role: [], file: [] };

  function toggle(list, value) {
    var at = list.indexOf(value);
    if (at === -1) list.push(value); else list.splice(at, 1);
  }

  function matches(row, needle) {
    if (row.dataset.retired === "1" && !showRetired.checked) return false;
    if (active.role.length && active.role.indexOf(row.dataset.role) === -1) return false;
    if (active.file.length && active.file.indexOf(row.dataset.file) === -1) return false;
    if (needle && row.dataset.find.indexOf(needle) === -1) return false;
    return true;
  }

  function byId(a, b) {
    return a.dataset.id.localeCompare(b.dataset.id, undefined, { numeric: true });
  }

  function apply() {
    var needle = q.value.trim().toLowerCase();
    var mode = sort.value;
    var shown = 0;

    rows.forEach(function (row) {
      var ok = matches(row, needle);
      row.hidden = !ok;
      if (ok) shown++;
    });

    if (mode === "doc") {
      // Put every row back where it was written, and hide a feature whose
      // rows are all filtered out.
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
        var key = mode === "role" ? "role" : "family";
        var left = a.dataset[key] || "";
        var right = b.dataset[key] || "";
        return left === right ? byId(a, b) : left.localeCompare(right);
      });
      ordered.forEach(function (row) { flat.appendChild(row); });
      features.hidden = true;
      flat.hidden = false;
    }

    var filtering = needle !== "" || active.role.length > 0 || active.file.length > 0;
    count.textContent = filtering
      ? shown + " of " + rows.length + " shown"
      : rows.length + " criteria";
    clear.hidden = !filtering && !showRetired.checked;
    nomatch.hidden = shown !== 0;
  }

  chips.forEach(function (chip) {
    chip.setAttribute("aria-pressed", "false");
    chip.addEventListener("click", function () {
      toggle(active[chip.dataset.filter], chip.dataset.value);
      chip.setAttribute(
        "aria-pressed",
        active[chip.dataset.filter].indexOf(chip.dataset.value) === -1 ? "false" : "true"
      );
      apply();
    });
  });

  q.addEventListener("input", apply);
  sort.addEventListener("change", apply);
  showRetired.addEventListener("change", apply);

  clear.addEventListener("click", function () {
    q.value = "";
    showRetired.checked = false;
    active.role = [];
    active.file = [];
    chips.forEach(function (chip) { chip.setAttribute("aria-pressed", "false"); });
    apply();
    q.focus();
  });

  // Typing / from anywhere jumps to the search box, the way a document reader
  // expects. Escape clears it.
  document.addEventListener("keydown", function (event) {
    if (event.key === "/" && document.activeElement !== q) {
      event.preventDefault();
      q.focus();
      q.select();
    } else if (event.key === "Escape" && document.activeElement === q) {
      q.value = "";
      apply();
    }
  });

  // A link to a criterion has to land on it even when filters would hide it,
  // otherwise a shared link silently shows nothing.
  function reveal() {
    var id = decodeURIComponent(location.hash.replace(/^#/, ""));
    if (!id) return;
    var row = document.getElementById(id);
    if (!row) return;
    if (row.dataset.retired === "1") showRetired.checked = true;
    q.value = "";
    active.role = [];
    active.file = [];
    chips.forEach(function (chip) { chip.setAttribute("aria-pressed", "false"); });
    apply();
    row.scrollIntoView({ block: "center" });
    row.classList.add("flash");
    setTimeout(function () { row.classList.remove("flash"); }, 1600);
  }

  window.addEventListener("hashchange", reveal);
  apply();
  reveal();
})();
