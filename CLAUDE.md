# hi (Human Intent)

An acceptance-criteria layer that sits above spec-sync. A human writes plain sentences about what
they want, each with a permanent hand-written id. Tickets and specs are generated from them.

**Read [DECISIONS.md](DECISIONS.md) before changing anything.** It records what was deliberately
left out and why. Most "missing features" in this codebase are decisions, not gaps.

## The gate

```bash
fledge lanes run verify     # fmt + clippy, then tests, hi's own intent, specs, plugin
```

Individual steps: `cargo test` · `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check`
· `cargo run -- check` · `specsync check` · `fledge plugins validate . --strict`

## Architecture

Each module owns one thing, and `main.rs` owns none of them.

| File | Owns |
|---|---|
| `src/id.rs` | The id grammar: family charset, strict number/letter alternation, parent and descendant relations |
| `src/doc.rs` | Parsing and writing `hi/*.md`; surgical insertion that leaves the rest of the file byte-identical; `retire`; `role_of` and `without_role` |
| `src/workspace.rs` | Finding `hi/`, loading docs, and lookups. Writes nothing |
| `src/check.rs` | Structural validation only: six problem kinds, and everything that is deliberately not one |
| `src/capture.rs` | The write path. A new id just works; an existing id refuses. Starts `INTENT.md` when the repo has none |
| `src/out.rs` | `ls`, `issue`, `export`, `index` |
| `src/view.rs` | The HTML page, and the only markdown rendering in the crate |
| `src/main.rs` | clap wiring, routing, exit codes. No domain logic |

Every module has a spec under `specs/<module>/`, and requirements there cite the hi criterion they
serve (`hi: CAPTURE-3`). If you change behavior, update the spec. `specsync check` is in the gate.

## Rules that are easy to break by accident

- **A criterion is exactly one line, and it is a markdown list item.** `render_criterion` never
  wraps, and always emits `- ID  sentence` indented two spaces per depth level. Bare lines look
  fine in the source and render as one run-together paragraph, which is the bug that falsified
  `FILE-1` (DECISIONS.md §10.1 and §12, `hi: FILE-6`, `FILE-1.b`).
- **Criteria are role-play, and nothing enforces it.** Every criterion is written
  `As a <role>, <sentence>`, and `doc::role_of` reads the role back off the front: `As a ` or
  `As an `, then everything up to the first comma, capped at four words so a sentence that merely
  begins "as a" is not mistaken for one. `doc::without_role` returns the remainder. Four verbs
  surface it (`ls`, `export`, `issue`, `view`), and **none of them require it**: a sentence with no
  role passes through exactly as written, and `hi check` says nothing. Do not add a check kind for
  a missing role; that is the sentence grammar DECISIONS.md §9 refused (§14, `hi: FILE-16`).
- **Escape before interpreting markers.** `view::inline_markdown` escapes the whole string first,
  then scans for markers. Reversing that order is a vulnerability, not a refactor.
- **Line-index bookkeeping after `splice`.** `Doc::insert` shifts `line`, `end_line`,
  `criteria_end` and `criteria_heading`. `rewrite_families` shifts them again when it adds a
  frontmatter line. Miss one and the next insert lands in the wrong place.
- **Capture validates before it mutates.** Every refusal path must return before any filesystem
  write (`hi: CAPTURE-5`). There is a test for this; keep it true.
- **`hi check` never fails on unfinished intent.** Only on a structurally broken file
  (`hi: CHECK-1`). Adding a quality gate here would break the whole premise. There are six
  structural problems, and the README says "exactly six", so adding a seventh means editing both.
- **The fledge shim never resolves `hi` from `PATH`.** Another project ships a binary called `hi`
  that is a coding agent with shell access (DECISIONS.md section 13), so `bin/fledge-hi` runs this
  plugin's own build or fails. Do not add a PATH fallback.
- **A `# ` heading closes an open section.** `parse_body` must record `criteria_end` when it does,
  or the append point is lost and a new family lands above the existing block. Anything
  criterion-shaped below such a heading is invisible, which is why `check` reports `stray-criterion`.
- **`looks_like_id` is the contract with `main`.** If it ever accepts a lowercase-initial token,
  argv routing starts shadowing subcommands.
- **Retiring never frees an id.** `Doc::retire` moves the criterion and every descendant together,
  then reparses the whole file rather than patching line indexes in two directions. Capture,
  `hi issue` and `hi check` all treat a retired id as taken forever (`hi: RETIRE-2`, DECISIONS.md
  §4 and §8.3).
- **`INTENT.md` is created by capture, best effort, and only after the criterion is on disk.** A
  capture that succeeded must never be reported as a failure because `INTENT.md` could not be
  written. `check::product_intent_note` then nags until there is prose in it, as a `note:` that
  never touches the exit code (`hi: INDEX-3`, `INDEX-3.a`).

## Dogfooding

hi describes itself in `hi/`. When you change behavior, capture the intent first, in role-play
voice like every other criterion:

```bash
cargo run -- CHECK-6 "As a person running CI, a sentence describing what someone wants"
```

Retire one the same way, rather than editing the markdown by hand:

```bash
cargo run -- retire CHECK-6 "it turned out to be the same as CHECK-2"
```

hi's own roles are thin, and that is a known limit rather than a model to copy. This product has one
audience, so all 96 criteria speak as one of six roles and 61 of them are *a person writing intent*.
A product with an operator on one side and a member on the other has voices that contradict each
other, which is the case hi's own files never exercise. DECISIONS.md §14 explains why that made the
format's biggest gap invisible from inside.

The feature list in `INTENT.md` is generated by `hi index`; the prose around the index block is
human-written and the tool never touches it. `intent.html` is generated by `hi view` and is
gitignored.

## Releasing

v0.1.0 is out: the repo is public, `human-intent` is on crates.io, and the v0.1.0 release carries
binaries for Linux, macOS and Windows. There is no Homebrew formula.

`release.yml` fires on a `v*` tag, so **tagging is the release**. Bump `Cargo.toml`, update
`CHANGELOG.md`, tag, then `cargo publish` separately. The format is not frozen; this is 0.x.

There is unreleased work on `main`: the role-play voice, `hi retire`, `INTENT.md` from the first
capture, and the `hi issue` fixes. `CHANGELOG.md` holds them under Unreleased, with no version
number and no date, because the version is decided at tag time and nowhere else.
