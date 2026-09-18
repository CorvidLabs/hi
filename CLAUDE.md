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
| `src/doc.rs` | Parsing and writing `hi/*.md`; surgical insertion that leaves the rest of the file byte-identical; `retire` |
| `src/workspace.rs` | Finding `hi/`, loading docs, and lookups. Writes nothing |
| `src/check.rs` | Structural validation only: seven problem kinds, and everything that is deliberately not one |
| `src/capture.rs` | The write path. A new id just works; an existing id refuses. Starts `INTENT.md` when the repo has none |
| `src/out.rs` | `ls`, `issue`, `export`, `index` |
| `src/view.rs` | The HTML page, and the only markdown rendering in the crate. `view/style.css`, `view/app.js` and the three `view/theme*`/`view/*.html` files are `include_str!`-ed, never built by `format!` |
| `src/main.rs` | clap wiring, routing, exit codes. No domain logic |

Every module has a spec under `specs/<module>/`, and requirements there cite the hi criterion they
serve (`hi: CAPTURE-3`). If you change behavior, update the spec. `specsync check` is in the gate.

## Rules that are easy to break by accident

- **A criterion is exactly one line, and it is a markdown list item.** `render_criterion` never
  wraps, and always emits `- ID  sentence` indented two spaces per depth level. Bare lines look
  fine in the source and render as one run-together paragraph, which is the bug that falsified
  `FILE-1` (DECISIONS.md §10.1 and §12, `hi: FILE-6`, `FILE-1.b`).
- **A paragraph of prose is one line, and hi never reflows anybody else's.** A newline inside a
  paragraph is a `<br>` in a GitHub issue but not in a file, so `hi issue` folds it
  (`out::unwrap_soft_breaks`) and keeps blank lines, lists, quotes, headings, tables, rules, fences
  and explicit hard breaks. Everything hi *writes* is one line per paragraph — `agent_instructions`,
  `starter_intent`, this repository's own `hi/*.md` intent blocks, the README's example — because
  the file an adopter reads first is the one they copy. `hi view` and `hi export` were checked and
  deliberately left alone. Do not make capture rewrap prose and do not add a check kind for
  wrapping: `FILE-4` and `CHECK-1` both say no (DECISIONS.md §29, `hi: ISSUE-7`, `FILE-21`).
- **A criterion is a plain sentence, and hi never parses it.** No prefix, no fields, no subject
  read off the front. Every verb (`ls`, `export`, `issue`, `view`) prints the author's words
  through untouched. 0.2.0 through 0.2.5 required an `As a <role>,` opening and read it back with
  a four-word heuristic; that is removed. Do not reintroduce a role, a `speaker:` field or any
  other slot in front of the sentence, and do not add a check kind for one: that is the sentence
  grammar DECISIONS.md §9 refused, arriving by the back door (§14 and §24, `hi: FILE-16`,
  `FILE-17`, both retired).
- **The brand tokens are imported, never edited here.** The top of
  `src/view/style.css` is copied verbatim out of
  `_CorvidLabs/design-system/assets/tokens.css` (Brand Kit v1.3), and so are the sun/moon toggle,
  its pre-paint snippet and `theme.js`. If a value looks wrong, it is wrong in the kit; fix it
  there and re-copy. The one divergence is the webfonts: the kit loads them from Google, and this
  page must fetch nothing (`hi: VIEW-2`), so both faces are named first in the stack with system
  fallbacks (DECISIONS.md §25, `hi: VIEW-17`, `VIEW-17.a`).
- **`[hidden]` needs `display: none !important`.** An author `display` rule outranks the user
  agent's, so `.row { display: flex }` silently defeats `row.hidden` and filtering changes the
  count while changing nothing on screen. That shipped for four releases. There is a test; keep it
  (DECISIONS.md §25, `hi: VIEW-6`, `VIEW-7`).
- **The page is checked by something that opens it.** `scripts/view-behaves.sh` drives a generated
  page in headless Chrome and asserts on what is visible, because every other view test asserts on
  the HTML going in and two bugs shipped anyway: the roles rendering with no separator, and the
  filters hiding nothing. Add a check there when you change the page's behavior. Do not pass
  `--user-data-dir`: a fresh profile deadlocks headless Chrome on a page that writes localStorage,
  which this one does from the theme toggle.
- **Escape before interpreting markers.** `view::inline_markdown` escapes the whole string first,
  then scans for markers. Reversing that order is a vulnerability, not a refactor.
- **Line-index bookkeeping after `splice`.** `Doc::insert` shifts `line`, `end_line`,
  `criteria_end` and `criteria_heading`. `rewrite_families` shifts them again when it adds a
  frontmatter line. Miss one and the next insert lands in the wrong place.
- **An id is the only promise. Guard it.** Five of hi's own verbs could break it, four
  silently. Anything that writes a `hi/*.md` must not be able to strand a criterion, forge one,
  hide one, or lose one to a concurrent write. `capture` and `retire` hold `lock::acquire` across
  the whole read-modify-write, and every string hi writes into a file goes through `doc::one_line`
  (DECISIONS.md §26, `hi: FILE-19`, `FILE-20`, `RETIRE-5`, `RETIRE-6`, `CAPTURE-14`).
- **A write reads itself back, and a fence is not structure to the writer either.** `insert`,
  `retire` and `set_retired_reason` each parse the buffer they are about to save and refuse it
  unless the id is readable in the section the verb named and nothing previously readable was lost.
  That check exists because two verbs decided where a section was by scanning raw lines: `hi retire`
  found the `## Retired` inside somebody's fenced example and freed a real id, and `insert` appended
  a `## Criteria` heading inside an unclosed fence and reported the same id saved twice. Anything
  that needs to know where a section is calls `doc::fence_map`, the same one `parse_body` uses, and
  then still reads its result back (DECISIONS.md §31, `hi: FILE-22`, `FILE-22.a`, `FILE-22.b`).
- **`unwrap_or_default` on a read of somebody's file is a bug.** `write_index` had it, so every
  read error read as "no file here" and the starter scaffold was written over an `INTENT.md`
  holding a person's prose and one invalid byte. Only `NotFound` may create; every other read
  error preserves the file and is reported. Since 0.7.0 that path runs on every capture, which is
  how a latent bug became a constant one (DECISIONS.md §32, `hi: INDEX-2.c`).
- **One reservation lookup, not two.** `Workspace::strays` covers the docs hi loads and the
  uppercase files it skips, and `check` reports from it while `capture` refuses from it. They were
  two scans over two different sets of files, so a retired id in `hi/Archive.md` was reported as
  taken and handed out again (`hi: CAPTURE-14`, DECISIONS.md §32). Reading inside hi's own files
  reserves ids and nothing else: they are still not docs, not counted, not in the feature list.
- **Unreadable is not absent.** `strays` returns a `Result`, and a file it cannot read or decode is
  that failure rather than an empty list. Swallowing the error made the shared lookup say "free"
  about an id it had not been able to look for, so `check` and `capture` agreed on the same wrong
  answer and a reserved id was handed out. `let Ok(x) = read(..) else { continue }` is the same
  shape as `unwrap_or_default` on a read: distrust both wherever the answer decides whether
  something exists. The failure is operational and never a check kind (`hi: CAPTURE-15`,
  DECISIONS.md §36).
- **The automatic refresh rewrites a block and installs none.** `refresh_index` passes
  `Absent::LeaveAlone` so its whole effect on disk is a span replacement between two markers.
  `hi index` passes `Absent::Install`. A `## Features` heading is prose, and a person who deleted
  the block gets to keep it deleted (`hi: INDEX-4.c`, DECISIONS.md §32).
- **Read inside the lock, or you are writing from before it.** Both write verbs call
  `Workspace::find` again once the lock is granted. `retire` did not, and two concurrent retires
  both reported success while one criterion came back to life. Holding a lock around a write you
  decided on before you held it protects nothing (DECISIONS.md §33, `hi: RETIRE-7`).
- **The lock must be a lock in a repository that has never run hi.** `hi/.hi.lock` lives inside
  `hi/`, so `lock::acquire` creates the directory first and fails closed on anything else. It used
  to hand back a `Guard` when the open failed, so every concurrent first capture ran unlocked and
  releasing one deleted a real holder's lock. `Guard` has no public constructor for that reason,
  and it takes back an empty `hi/` it made so a refusal still writes nothing. A write-path test
  that starts from a repository which already has a `hi/` cannot see any of this: use
  `cli::Repo::bare` (DECISIONS.md §33, `hi: FILE-19`, `CAPTURE-5`).
- **hi never breaks a lock, because the lock is the kernel's.** `flock` on unix, `LockFileEx` on
  Windows, both declared in `lock`'s own three-line `extern` blocks rather than added as a fifth
  dependency. Age was wrong, and so was the heartbeat that replaced it: a waiter that decides a
  lock is abandoned and then removes it has already approved the removal by the time the holder
  changes, and no extra check closes that. Do not reintroduce any rule that lets one process
  decide another is finished. The one `remove_file` of `.hi.lock` is `Guard::drop`, and it runs
  *before* the close, while the lock is still held. On unix, `still_at` must stay: the kernel can
  grant a queued waiter a lock on an inode the pathname no longer names (DECISIONS.md §34,
  `hi: FILE-23`, `FILE-24`).
- **Never write a fixed temp or fixture path.** `write_atomically` and the integration-test
  fixtures both used one, so two processes shared a scratch file. That is why the suite flaked and
  why bulk capture lost writes. Include the pid.
- **Rebuild before you trust an integration test.** `target/debug/hi` went stale twice in one
  session and both times the failure looked like a code bug. `cargo clean -p human-intent` when a
  result does not match what the release binary does.
- **Capture validates before it mutates.** Every refusal path must return before any filesystem
  write (`hi: CAPTURE-5`). There is a test for this; keep it true.
- **`hi check` never fails on unfinished intent.** Only on a structurally broken file
  (`hi: CHECK-1`). Adding a quality gate here would break the whole premise. There are seven
  structural problems, listed in [HI-1.md](HI-1.md). 1.0 freezes those seven and the policy
  that they are structural only; an eighth is a 2.0. Do not add a kind for wrapping, for
  English, or for unfinished intent (DECISIONS.md §39).
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
- **A verb that changes the live count refreshes `INTENT.md`'s generated list.** `capture` and the
  `hi retire` arm both call `out::refresh_index` after the criterion is on disk, because nothing
  made anyone run `hi index` and three adopter repositories had drifted. It is best effort: every
  failure, `INDEX-2.b`'s refusal to guess at broken markers included, comes back as a string and is
  printed on stderr, and the command still exits 0. Do not let it raise, do not move it above
  `Doc::save`, and do not give it a lock: `capture` and `retire` already hold `lock::acquire` and
  it is not reentrant (DECISIONS.md §30, `hi: INDEX-4`, `INDEX-4.a`).
- **`hi check`'s note about that list is a note, not a problem.** `out::index_note` is
  pushed onto `Report::note`, which `Report::ok` never reads, so the exit code cannot move. It is
  the first thing `check` nags about that hi itself maintains, and it is only admissible because
  capture and retire keep the list current, leaving one cause: a criterion typed in by hand, which
  `FILE-14` allows (DECISIONS.md §30, `hi: INDEX-4.b`, `CHECK-1`).
- **`Doc::insert` does not add the criterion to `doc.criteria`.** It splices the line into
  `Doc::lines` and shifts the indexes, and that is all. Anything that counts after an insert must
  reload the saved file first, which `capture` does. Do not "fix" this by reparsing at the end of
  `insert` the way `retire` does: the index shifting is the contract with `rewrite_families` and
  with the next insert, and `doc::tests::block_style_insert_keeps_line_positions_correct` stops
  being able to fail.
- **`INTENT.md` is created by capture, best effort, and only after the criterion is on disk.** A
  capture that succeeded must never be reported as a failure because `INTENT.md` could not be
  written. `check::product_intent_note` then nags until there is prose in it, as a `note:` that
  never touches the exit code (`hi: INDEX-3`, `INDEX-3.a`).

## Dogfooding

hi describes itself in `hi/`. When you change behavior, capture the intent first, as one plain
sentence like every other criterion:

```bash
cargo run -- CHECK-6 "a sentence describing what someone wants"
```

Retire one the same way, rather than editing the markdown by hand:

```bash
cargo run -- retire CHECK-6 "it turned out to be the same as CHECK-2"
```

hi has one audience and one voice, which is a known blind spot rather than a model to copy. A
product with an operator on one side and a member on the other has voices that contradict each
other, and hi's own files never exercise that case. DECISIONS.md §14 explains how that hid the
format's biggest gap from inside, and §24 explains why the first fix we shipped for it was wrong.

Before you write a criterion, try putting *As a ___,* in front of it. If you cannot finish it, you
wrote a fact about the system rather than something somebody wants. Then leave the words out of the
file; the test is for you, not for the format.

The feature list in `INTENT.md` is generated by `hi index`; the prose around the index block is
human-written and the tool never touches it. `intent.html` is generated by `hi view` and is
gitignored.

## Releasing

v0.7.0 is out: the repo is public, `human-intent` is on crates.io, `corvidlabs/tap/hi` is in the
Homebrew tap, and every tagged release carries binaries for Linux and macOS (both architectures
each) and Windows. The docs are at corvidlabs.xyz/hi, and corvidlabs.github.io/hi publishes this
repository's own `hi view` output on every push to `main`.

**v0.2.4 and v0.2.5 are tagged on GitHub but were never published to crates.io.** Both `cargo
publish` runs failed on a dirty tree and the failure was not noticed. 0.3.0 closed the gap, and the
lesson is the rule below: read what `cargo publish` actually printed, and check the registry.

`release.yml` fires on a `v*` tag, so **tagging is the release**. Bump `Cargo.toml`, update
`CHANGELOG.md`, commit, push, tag, then `cargo publish` separately. The format is not frozen; this
is 0.x.

**Publish from a clean tree, and verify the registry afterwards.** `cargo publish` refuses a dirty
working tree, and that refusal is easy to miss in a wall of output. Run `cargo publish --dry-run`
first, and confirm the version really landed:

```bash
curl -s -A "release-check" https://crates.io/api/v1/crates/human-intent | \
  python3 -c "import json,sys; print(json.load(sys.stdin)['crate']['max_version'])"
```

Unreleased work sits under an `Unreleased` heading in `CHANGELOG.md` with no version number and no
date, because the version is decided at tag time and nowhere else.
