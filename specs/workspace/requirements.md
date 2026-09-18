---
spec: workspace.spec.md
---

## User Stories

- As someone typing `hi SEND-2 "..."` from anywhere inside my repo, I want hi to find the `hi/` directory by itself so that I never have to `cd` to the root or pass a path (hi: CAPTURE-10)
- As someone adding hi to a project that sits inside another repository, I want the walk to stop at my own repository so that I never inherit the outer project's criteria as if they were mine (hi: CAPTURE-10)
- As someone whose repo ships translations, I want a `public/locales/hi/` folder to be left alone so that hi never adopts a Hindi locale bundle as my workspace (hi: CAPTURE-6)
- As someone whose editor writes a byte-order mark, I want my file to still be found and read so that an invisible byte never makes a valid file disappear (hi: FILE-11)
- As someone adopting hi on an existing repo, I want the very first `hi FAMILY-1 "..."` to work before `hi/` exists so that installing hi is one command, not a setup ritual
- As someone reading a hi file with no tool installed, I want the directory to be nothing but markdown (no index, no lockfile, no cache) so that the files survive the binary (hi: FILE-1, FILE-1.a)
- As someone who hand-edits a hi file, I want hi to resolve my family from either the frontmatter or the criteria I actually wrote so that the one machine-facing line stays optional in practice (hi: FILE-2)
- As someone who keeps several features in one file, I want one file to answer for several families so that `hi/chat.md` can hold `SEND`, `RECEIPT` and `OFFLINE` (hi: FILE-5)
- As someone who retired a criterion, I want its id to stay spoken for so that hi never hands the same number to two different intentions
- As someone who parked a retired criterion in a file hi does not read, I want its id to stay spoken for there too, so that being told an id is used and being refused it are the same answer (hi: CAPTURE-14)
- As someone reading `hi check` or piping `hi export` into an agent, I want file paths printed the same way every run, with forward slashes whatever platform I am on, so that output diffs cleanly and a path pasted into a link still works (hi: FILE-12)

## Acceptance Criteria

- Discovery walks up from a starting directory and stops at the first ancestor whose `hi/` actually holds a hi file, or failing that at the first ancestor holding a `.git` entry; it fails with a message that names the problem when the ancestors run out with neither found
- A directory named `hi` that holds no file declaring `hi:` in its frontmatter is not a workspace, whatever else is in it (hi: CAPTURE-6)
- Loading reads only `*.md` files sitting directly inside `<root>/hi`, in sorted path order
- A root with no `hi/` directory loads as a valid, empty workspace rather than an error
- Nothing in this module writes to disk, creates a directory, or mutates a loaded `Doc` (hi: FILE-4)
- Family ownership resolves from frontmatter first and from actual criterion use second, so an undeclared family still finds its file (hi: FILE-2)
- Id lookup, family ownership and next-free numbering all consider retired criteria as well as active ones
- `families()` returns the sorted, deduplicated union of declared and used families across every doc (hi: FILE-5)
- Path rendering is relative to `root`, joined with forward slashes on every platform, total, and never panics on a path outside it (hi: FILE-12)
- The only failures the module can produce are I/O failures; a structurally broken hi file loads and its problems are left to `hi check`

### REQ-workspace-010

The workspace module SHALL read only lowercase-named markdown files in `hi/` as criteria, and SHALL remember the rest rather than discarding them (hi: FILE-20).

Acceptance Criteria

- `Workspace::load` sends every `*.md` directly inside `hi/` through `is_hi_own_file`, which tests whether the first character of the file name is an ASCII uppercase letter, and pushes those onto `Workspace::skipped` instead of parsing them.
- The rule is derived, not arbitrary: `capture::start_file` lowercases every family name, so a criteria file hi wrote is always lowercase and an uppercase name is never one (DECISIONS.md §27).
- `skipped` is kept on the workspace so `check` can look inside those files. Dropping them would make a criterion written into one silently invisible, which is the failure `FILE-20` exists to prevent.
- `holds_hi_files` is unchanged and still requires frontmatter carrying a `hi:` key, so `hi/AGENTS.md` alone never makes a directory look like a workspace.

### REQ-workspace-011

`Workspace::strays` SHALL be the one lookup for an id hi cannot read as structure, covering the
loaded docs and the skipped files together, and `find_stray` SHALL answer from it (hi: CAPTURE-14,
FILE-20).

Acceptance Criteria

- `strays` walks `skipped` first, reading each file and running `doc::criterion_tokens` over it, and records each hit as `StrayPlace::UnreadFile`; then it walks each doc's `Doc::stray` and records each as `StrayPlace::OutsideSection`.
- A `Stray` carries the workspace-relative `file`, the 1-based `line`, the id-shaped `token` and the `place`. The token is whatever the source recorded: `Doc::stray` keeps markdown emphasis (`**SEND-9**`), `criterion_tokens` has already removed it. `check` quotes it back verbatim, so neither is normalized here.
- `find_stray` compares each token with `doc::strip_emphasis` applied and ASCII-uppercased, and returns the first match.
- A skipped file that cannot be read or decoded is passed over and the rest are still scanned, the behavior `check` has had since skipped files were first read. This is a lookup, not a verb: it returns no `Result` and cannot fail.
- `check` builds its `StrayCriterion` problems from the same call, so an id reported as used and an id refused by capture are by construction the same set. They were two scans over two different sets of files, and a retired id in `hi/Archive.md` was reported by one and reissued by the other (DECISIONS.md §32).
- Reading inside hi's own files reserves ids and nothing more: `docs`, `criteria_count`, `families` and the generated feature list are untouched by `strays`, so `hi/AGENTS.md` never becomes a file that holds criteria (DECISIONS.md §27).
- A fenced block in a skipped file is an example rather than structure, exactly as under `## Intent`, so documenting the format in a `hi/README.md` reserves nothing (hi: FILE-9).


## Constraints

- No dependency beyond `std`, `anyhow`, and the sibling `doc` and `id` modules. The `hi/` directory has no manifest format to parse, so nothing else is needed (hi: FILE-1)
- Discovery must terminate: the ancestor walk ends at a `.git` boundary or at the filesystem root, whichever comes first. In the worst case, where the start directory has neither a workspace nor a repository above it, it examines every ancestor before failing
- `Doc::parse` is infallible by contract, so this module may not introduce a parse-failure path of its own. `holds_hi_files` keeps that contract by answering `bool`: every I/O or encoding problem it meets reads as "not a hi file", never as an error
- Recognition reads candidate files off disk during the walk, so it must stay cheap: only files directly inside the candidate `hi/`, only `*.md`, and it stops at the first match
- Indices are returned as `usize` positions in `docs` rather than borrows, because `capture` needs `&mut` on the doc it resolved. An append, which is the only mutation any consumer makes, leaves them valid; a removal or a reorder would not, and nothing does either
- Load cost is linear in the number of hi files and each is read once; the lookups are linear scans, which is correct at the scale hi is for (tens of files, hundreds of criteria)
- `docs` order is only guaranteed sorted at load time: `capture` appends new files to the end

## Out of Scope

- Writing, inserting, formatting or saving anything in a hi file: that is `doc`
- Creating `hi/` or a new feature file, which `capture` does using `dir`
- Judging whether a file is structurally valid: duplicates, orphans, retired collisions and undeclared families are `check`'s findings, not load errors
- Parsing ids or enforcing level alternation: that is `id`
- Rendering criteria as text, JSON, tickets or the `INTENT.md` index, which `out` does
- Any notion of lifecycle, state, evidence or progress. The workspace is a read-at-load snapshot and hi stores nothing between runs (DECISIONS §5)

### REQ-workspace-001

`Workspace::find` SHALL locate the workspace by walking up from a starting directory to the
nearest ancestor whose `hi/` directory actually holds a hi file, and SHALL stop at the nearest
ancestor holding a `.git` entry when it meets one first.

Acceptance Criteria

- The starting path is canonicalized first, so a relative path or a symlink resolves before the walk begins.
- Recognition is tested at each level before the walk moves up, so the *nearest real workspace* wins: a nested project's own `hi/` is preferred over an outer one.
- Within one level `hi/` is tested before `.git`, so a directory that holds both loads its own workspace and the boundary never gets in the way.
- A `.git` entry ends the walk and that directory is loaded, because a repository is where hi anchors: a project nested inside another must not adopt the outer project's criteria (hi: CAPTURE-10). A repo root with no `hi/` yet loads as an empty workspace, which is what lets the first capture create the directory (hi: CAPTURE-1.a).
- The `.git` test is `exists()`, so a `.git` file (a worktree or submodule) is a boundary as well as a `.git` directory.
- An unrecognized `hi/` does not extend the walk past the boundary: the climb continues past that directory, but the `.git` test still applies at that level and every level above it.
- Running out of ancestors with neither found fails with `this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so run it inside one`.
- A starting path that cannot be canonicalized fails with `resolving <path>` and the underlying I/O error.
- The starting path is whatever `main` hands over: the current directory, or the value `peel_root` took out of a *leading* `--root`. `find` walks *up* from it either way, so `--root` is a starting point; the boundary is the `.git` it meets on the way.

### REQ-workspace-002

`Workspace::load` SHALL read every `*.md` file directly inside `<root>/hi` in sorted path order,
and SHALL treat a missing `hi/` directory as an empty workspace rather than an error.

Acceptance Criteria

- Only regular files are loaded, and only those whose extension matches `md` case-insensitively.
- Sub-directories inside `hi/` are not walked.
- Paths are sorted before parsing, so `docs` order and everything derived from it is stable run to run.
- A missing `hi/` yields `docs == []` with `dir` still pointing at `<root>/hi`, so the first capture can create it.
- A directory entry that cannot be stated is skipped; a hi file that cannot be read fails the whole load with `Doc::load`'s own error.
- What `load` yields is whatever `Doc::parse` made of each file. A line inside a fenced code block is prose and never becomes a criterion (hi: FILE-9), and a criterion-shaped line outside every section lands in `Doc::stray`, which `Doc::all()` does not chain. Every lookup in this module therefore looks straight past both; reporting the stray one belongs to `check` (hi: CHECK-2.e).

### REQ-workspace-003

The workspace SHALL be a read-only snapshot taken at load time, and SHALL NOT write, cache or
persist anything.

Acceptance Criteria

- No function in the module writes a file, creates a directory, or mutates a loaded `Doc`.
- The module needs no index, lockfile, manifest or cache beside the hi files and builds none: every answer comes from the markdown it just read (hi: FILE-1, FILE-1.a, FILE-2). The generated feature list `out` keeps in `INTENT.md` is prose for a person and is never read back as state.
- Derived values (counts, families, next-free numbers) are computed on demand from the loaded docs and never stored (DECISIONS §5).
- Because this module writes nothing, prose a human wrote can never be reflowed or reformatted by it (hi: FILE-4, FILE-4.a).

### REQ-workspace-004

`Workspace::doc_for_family` SHALL resolve a family to the file that owns it from the frontmatter
`families:` declaration first, and from actual criterion use second.

Acceptance Criteria

- A doc whose frontmatter names the family is returned even when no criterion uses it yet.
- The declaration is read from `front.families` whichever form the file wrote it in (inline `families: [SEND]`, a YAML block list, or the singular `family:` key), because `doc` normalizes all three into one list. Keeping the file's own style when that list is rewritten belongs to `doc`, not here.
- When no frontmatter declares it, the first doc holding a criterion in that family is returned, counting retired criteria.
- The declaration pass is tried across all docs before the use pass begins, so declaration always beats use.
- When two docs claim the same family, the earlier index wins silently. Nothing in hi reports the overlap: `hi check` compares ids, not the frontmatter `families:` lists of different files.
- An unknown family returns `None`, which capture reads as "create a new file for it", never as an error.

### REQ-workspace-005

`Workspace::find_id` SHALL find a criterion by exact id anywhere in the workspace, searching
retired criteria alongside active ones.

Acceptance Criteria

- The match is on the parsed `Id`, so a criterion whose id failed to parse can never be returned.
- Retired criteria are searched, which is what lets capture refuse to reuse a retired id.
- The doc's index is returned with the criterion so the caller can name the file and reach it mutably.
- An id nothing uses returns `None`.

### REQ-workspace-006

`Workspace::next_free` SHALL return one past the highest top-level number used by a family,
counting retired criteria, and SHALL return `1` for an unused family.

Acceptance Criteria

- Every doc is scanned, so a family split across files still gets one answer.
- Only the first level of each id is considered; cases and steps never advance the top-level number.
- Retired criteria count, so retiring `SEND-3` leaves the next `SEND` at `SEND-4` (DECISIONS §8.3).
- Criteria with unparseable ids are ignored rather than crashing the scan.
- A number that appears only inside a fenced code block, or only on a line stranded outside every section, never counts. `Doc::all()` does not yield either, so neither can advance the next free number (hi: FILE-9, CHECK-2.e).
- An unused family returns `1`.

### REQ-workspace-007

`Workspace::families` SHALL return the deduplicated, sorted union of every declared and every
used family across all docs.

Acceptance Criteria

- Both a doc's frontmatter `families:` list (inline or block form alike, since `doc` normalizes both into `front.families`) and the families its criteria actually use contribute (hi: FILE-2).
- One doc may contribute several families, so a file-per-feature layout still reports each family it holds (hi: FILE-5).
- A family named by two docs appears once.
- The result is sorted, so the check summary and the export scope test read the same way every run.

### REQ-workspace-008

`Workspace::rel` and `Workspace::intent_path` SHALL produce stable, root-relative paths and SHALL
NOT fail.

Acceptance Criteria

- `rel` renders a doc under `root` as a relative path such as `hi/chat.md`, which is the form printed by capture, `hi check` and `hi export`.
- `rel` joins the remaining components with `/` rather than the host separator, so the same string comes back on every platform and stays valid in exported JSON, an issue body and a markdown link (hi: FILE-12).
- `rel` renders a path outside `root` from that path's own components rather than erroring or panicking. The result is a best effort, not a contract: an absolute path outside `root` comes back with a doubled leading separator, and no caller in hi passes one.
- `intent_path` returns `<root>/INTENT.md` whether or not the file exists, leaving "missing" for the caller to interpret.

### REQ-workspace-009

Discovery SHALL recognize a workspace by its contents rather than by its name alone: a `hi/`
directory counts only when it holds at least one markdown file whose frontmatter carries a `hi:`
key (hi: CAPTURE-6).

Acceptance Criteria

- `holds_hi_files` examines only files directly inside the candidate directory whose extension matches `md` case-insensitively, and returns as soon as one qualifies.
- A file qualifies when its first line is `---` and a line before the closing `---` has a key of exactly `hi`, the same one machine-facing line a hi file already carries (hi: FILE-2).
- A leading UTF-8 BOM is stripped before the first line is read, so an editor-written marker cannot hide a file from discovery any more than it can hide the frontmatter from the parser (hi: FILE-11).
- A directory named `hi` that holds only other things, a Hindi locale bundle for instance, does not stop the walk, and nothing is ever created inside it.
- Failure to list the directory, to read a file, or to decode one as UTF-8 reads as "not a hi file". Recognition returns `bool` and never errors.
- Recognition gates **discovery only**. Once a root is chosen, `load` reads every `*.md` directly inside `<root>/hi` whether or not it declares `hi:`, so a file that forgot the line still loads and is still checked.
