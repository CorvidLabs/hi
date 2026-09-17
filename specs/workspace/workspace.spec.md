---
module: workspace
version: 1
status: active
files:
  - src/workspace.rs
  - src/lock.rs

db_tables: []
depends_on:
  - specs/doc/doc.spec.md
  - specs/id/id.spec.md
---

# Workspace

## Purpose

Finds the `hi/` directory for the repository you are standing in, loads every `hi/*.md` into
memory as a `Vec<Doc>`, and answers the four questions every verb needs to ask of that
collection: which file owns a family, where an id already lives, what the next free number in a
family is, and which families exist at all.

Discovery recognizes a workspace by what is **inside** the directory, not by its name alone: `hi`
is also the ISO 639-1 code for Hindi, so `public/locales/hi/` is a real directory in real
repositories. A `hi/` directory counts only when it holds a markdown file whose frontmatter
carries a `hi:` key; otherwise the walk climbs past it (hi: CAPTURE-6). The climb is bounded by
the repository: a directory holding `.git` is where the walk stops, so adding hi to a project
nested inside another never adopts the outer project's criteria as if they were yours
(hi: CAPTURE-10).

This module has **no hi family of its own**. It is infrastructure: `main`, `capture`, `check`,
`out` and `view` all sit on top of it, and none of hi's human-facing criteria describe it
directly. What it does have is an obligation to the `FILE` family: it must make the `hi/`
directory work as nothing but a folder of ordinary markdown. Discovery is a directory-name
convention confirmed by the one machine-facing line already in the file, resolution is
frontmatter plus a scan of what is actually used, and hi keeps no machine-readable index,
lockfile, cache or database that it reads back as state (hi: FILE-1, FILE-1.a, FILE-2). The generated feature list in `INTENT.md` is prose for a person, never an
input to this module. The module itself is strictly read-only: it opens files for reading only,
and the writing lives in `doc` (saving a hi file), `capture` (creating `hi/` and a new feature
file), `out` (rewriting `INTENT.md`) and `view` (writing the HTML page) (hi: FILE-4, FILE-4.a).

## Public API

| Export | Description |
|--------|-------------|
| `Workspace` | The loaded view of one repository's `hi/` directory: `root`, `dir`, and every parsed `Doc`. |
| `find` | Walk up from a starting directory to the nearest `hi/` that actually holds hi files, stopping at the first `.git` boundary, which is the repository root and the right place to make one. |
| `load` | Read every `*.md` directly inside `<root>/hi` into a `Workspace`. |
| `intent_path` | The path to `<root>/INTENT.md`, existing or not. |
| `criteria_count` | Count of active criteria across every doc, retired excluded. |
| `doc_for_family` | Index of the doc that owns a family, by frontmatter declaration or by actual use. |
| `find_id` | Look up one criterion by exact id, active or retired, returning its doc index with it. |
| `acquire` | Take the one-writer lock for a repository's `hi/` directory, creating that directory when the repository has never run hi, waiting for another writer, and returning a guard that releases on drop. Fails rather than returning a guard it did not take. |
| `Guard` | The held lock, and only ever a lock that was really taken: the constructor is private and takes the file that was exclusively created. Releasing on drop means a refusal never leaves the lock behind. |
| `find_stray` | The file and 1-based line of a criterion-shaped line hi could not read but which spoke for this id, matched after stripping emphasis. Capture refuses such an id: a line hi cannot parse has still used it, and handing it out again puts two identical ids with different sentences in one file (hi: CAPTURE-14, FILE-20). |
| `next_free` | One past the highest top-level number a family uses; retired numbers count, so they are never reissued. |
| `families` | Every family in the workspace, deduplicated and sorted. |
| `rel` | Render a path relative to `root`, joined with forward slashes on every platform, for stable output such as `hi/chat.md`. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Workspace` | Every hi file in one repository plus where they live. Public fields: `root: PathBuf` (the directory containing `hi/`, the base for `rel` and `INTENT.md`), `dir: PathBuf` (the `hi/` directory itself, which may not exist yet), and `docs: Vec<Doc>` (one parsed file each, in sorted-path order at load time). Constructed only by `find` and `load`, and by test code that builds the fields directly. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines and exports no traits. |

### Functions

Every exported function is an inherent method on `Workspace`. One private free function,
`holds_hi_files`, backs discovery.

| Function | Signature | Description |
|----------|-----------|-------------|
| `find` | `fn find(start: &Path) -> Result<Workspace>` | Canonicalizes `start`, then walks up the ancestor chain. At each level it asks `holds_hi_files(<dir>/hi)` first and returns `load(dir)` for the first ancestor that answers yes, so the *nearest real workspace* wins. Failing that it tests `<dir>/.git` at the same level, and a hit returns `load(dir)` as well: a repository is a boundary, so a project nested inside another gets its own empty workspace rather than the outer one's criteria, and the first capture in a fresh repo has a root to create `hi/` in (hi: CAPTURE-10, CAPTURE-1.a). Errors only when the ancestors run out with neither found. |
| `load` | `fn load(root: &Path) -> Result<Workspace>` | Reads every `*.md` file directly inside `<root>/hi`, in sorted path order, into `docs`. A missing `hi/` directory is not an error. It yields a workspace with zero docs and `dir` pointing at the path where it would go. |
| `intent_path` | `fn intent_path(&self) -> PathBuf` | `<root>/INTENT.md`. Returns the path whether or not the file exists; the caller decides what a missing file means. |
| `criteria_count` | `fn criteria_count(&self) -> usize` | Total active criteria across every doc. Retired criteria are not counted. |
| `doc_for_family` | `fn doc_for_family(&self, family: &str) -> Option<usize>` | Index into `docs` of the file that owns `family`. Prefers the first doc whose frontmatter `families:` list names it; falls back to the first doc where some criterion, active or retired, actually uses it. `None` means no file owns the family yet, which is capture's signal to create one. |
| `find_id` | `fn find_id(&self, id: &Id) -> Option<(usize, &crate::doc::Criterion)>` | Looks up one criterion by exact id across every doc, searching active and retired criteria alike. Returns the doc's index alongside the criterion. Criteria whose id failed to parse can never match. |
| `next_free` | `fn next_free(&self, family: &str) -> u32` | The highest top-level number used by `family` anywhere in the workspace, plus one; `1` when the family is unused. Retired criteria count, so a retired number is never handed back out. |
| `families` | `fn families(&self) -> Vec<String>` | Every family in the workspace, deduplicated and sorted: the union of each doc's declared `families:` and the families its criteria actually use. |
| `rel` | `fn rel(&self, path: &Path) -> String` | Renders `path` relative to `root` for stable, machine-comparable output such as `hi/chat.md`. The remaining components are joined with `/` rather than the host separator, so a Windows run prints `hi/chat.md` too and the string is safe in exported JSON, an issue body and a markdown link (hi: FILE-12). A path that is not under `root` is rendered from its own components rather than failing. |
| `holds_hi_files` (private) | `fn holds_hi_files(dir: &Path) -> bool` | True when `dir` holds at least one file this tool would recognize: a `*.md` (extension compared case-insensitively) whose opening `---` frontmatter block contains a key of exactly `hi`. Strips a leading UTF-8 BOM before looking, so an editor-written marker cannot hide a file from discovery the way it once hid the frontmatter from the parser (hi: FILE-11). A directory it cannot list, a file it cannot read, or a file that is not valid UTF-8 simply does not count; nothing here errors. |
| `acquire` | `lock::acquire(hi_dir: &Path) -> Result<Guard>` | `create_dir_all(hi_dir)`, then exclusive-create `<hi_dir>/.hi.lock` and write the holder's pid into it. Retries every 20ms for 30 seconds and then gives up with a message naming the file to delete. The directory comes first because the lock lives inside it: before `hi/` exists there is nothing to create a lock file in, and that failure used to be handed back as a guard, so every bootstrap capture ran unlocked (DECISIONS.md §31). Any failure other than "already taken" is an error, never a guard. |
| `acquire_within` (private) | `fn acquire_within(hi_dir: &Path, patience: Duration, abandoned: Duration) -> Result<Guard>` | The body of `acquire`, with both waits named so a test can wait in milliseconds instead of seconds. `acquire` passes `PATIENCE` (30s) and `ABANDONED` (5s). |
| `Guard::held` (private) | `fn held(file: File, path: PathBuf, dir: PathBuf, created_dir: bool) -> Guard` | The only constructor, and it takes the `File` that was exclusively created, so a guard that does not hold the lock cannot be built. Writes the pid and starts the heartbeat thread. |
| `Guard::drop` | `fn drop(&mut self)` | Ends the heartbeat (drops the sender, joins the thread), removes the lock file, and, when acquiring had to create `hi/` itself, removes that directory as well — `remove_dir` refuses a directory with anything in it, so this only ever takes back an empty one and a refusal leaves nothing behind (hi: CAPTURE-5). Held across the whole read-modify-write in `capture` and `retire`, not just the write: `doc::write_atomically` makes one write atomic and does nothing about two processes each reading the same original and writing over the other. Eight concurrent captures used to land two, and thirty-two into a repository with no `hi/` yet used to land twenty-three (hi: FILE-19). |
| heartbeat (private thread) | spawned by `Guard::held` | Every `HEARTBEAT` (250ms) it reopens the lock file and rewrites the same pid, which moves its mtime. It never creates the file, so it can never put back a lock the guard has released, and the guard stops it before removing the file. This is what lets a waiter tell a working holder from a dead one (hi: FILE-22). |

## Invariants

1. `src/workspace.rs` never writes to disk. It opens files for reading and nothing else: no
   `fs::write`, no `create_dir_all`, no file handle opened for writing anywhere in it,
   `holds_hi_files` included, which only reads. The one exception in this spec is `src/lock.rs`,
   whose whole job is a file: it creates `hi/` when the repository has never run hi, creates and
   refreshes `<hi>/.hi.lock`, and removes both again on release. Every edit to an existing hi file goes through `doc`'s insert/save
   path, whose temp-file-then-rename lives in the public `doc::write_atomically` (hi: FILE-8),
   the same helper `out::write_index` uses for `INTENT.md`, and the only hi file written from
   scratch is the one `capture::create_file` creates (hi: FILE-4).
2. A `Workspace` is a snapshot taken at load time. Nothing here re-reads the filesystem, watches
   for changes, or persists derived state between runs, so no file can be stale relative to
   something hi remembered (DECISIONS §5: state is derived, never written).
3. `load` considers only files sitting **directly** inside `<root>/hi` whose extension matches
   `md` case-insensitively. Sub-directories are not walked and non-file entries are skipped.
4. `docs` is ordered by sorted path at load time, so `families()`, `criteria_count()`, the check
   report and the export payload are all stable from run to run. Capture appends to `docs` after
   load, and appended docs sit at the end rather than in sorted position.
5. A directory with no `hi/` inside it loads successfully with zero docs. Emptiness is a valid
   workspace, never an error; `hi FAMILY-1 "..."` has to work on a repo that has never run hi.
6. `find` only ever moves up, and it stops at the first ancestor whose `hi/` actually holds a hi
   file, so a nested project's workspace wins over an outer one. Failing that it stops at the
   first ancestor holding a `.git` entry and loads that root, so discovery never leaves the
   repository it started in (hi: CAPTURE-10). Within one level `hi/` is tested before `.git`, so
   a directory that has both loads its own workspace. A directory named `hi` that holds something
   else (a Hindi locale bundle, say) never captures the walk (hi: CAPTURE-6); it is climbed past
   like any other level, and the `.git` boundary still applies at that level and above it.
7. `doc_for_family`, `find_id` and `next_free` all search retired criteria as well as active ones
   (via `Doc::all()`). A retired id stays spoken for forever and its number is never reissued
   (DECISIONS §4, §8.3).
8. Ownership of a family is first-wins, not exclusive. If two docs declare the same family,
   `doc_for_family` returns the earlier one and nothing anywhere complains: `check` compares
   ids, not frontmatter `families:` lists across files, so a shared declaration is not a
   structural problem today. Only a genuinely duplicated id is reported (`Kind::DuplicateId`).
9. Frontmatter is a shortcut, not the authority. A file that declares nothing still resolves by
   what its criteria use, so a hi file remains a plain markdown document rather than a registry
   entry (hi: FILE-2). The declaration is also style-blind here: `doc` fills `front.families`
   from an inline `families: [SEND, RECEIPT]`, from a YAML block list, or from the singular
   `family:` key alike, so the form a file uses changes nothing this module does with it.
   Preserving that form when the list is rewritten is `doc`'s job, not this module's.
10. One doc may own several families and `families()` unions them, so the file-per-feature,
    many-families-per-file layout holds (hi: FILE-5).
11. Every index this module returns is a position in `self.docs`, not a borrow. The only mutation
    any consumer makes is an append (`capture` pushes a newly created doc and keeps using the
    index it was handed), and an append never moves an existing element, so held indices stay
    valid. Removing or reordering `docs` would invalidate them; nothing in hi does either.
12. `rel` and `intent_path` are total: neither can fail. On a path outside `root`, `rel` falls
    back to rendering that path's own components joined with `/` rather than erroring. That
    fallback is a safety net, not an output format: an absolute path outside `root` comes back
    with a doubled leading separator, because its root component renders as `/` and is then
    joined with `/` again. Nothing in hi calls `rel` with such a path.
13. Parsing is infallible, so the only failures this module can produce are I/O failures. A hi
    file that is malformed loads successfully and its problems surface in `hi check`.
14. The recognition test gates **discovery only, never loading**. `holds_hi_files` decides only
    whether a `hi/` directory may stop the walk, and the `.git` boundary can stop it regardless;
    once a root is chosen either way, `load` reads every `*.md` directly inside `<root>/hi` no
    matter what its frontmatter says. A `.git` root whose `hi/` holds one frontmatter-less
    markdown file reports `0 criteria` and `1 file`, which is that split exactly. A hand-written file that forgot the
    `hi:` line still loads and is still checked. It just cannot, by itself, make a directory the
    workspace.
15. Every derived answer here comes from criteria `doc` actually parsed. Lines inside a fenced
    code block are prose and never become criteria (hi: FILE-9), and criterion-shaped lines
    outside every section land in `Doc::stray`, which `Doc::all()` does not chain. So a fenced or
    stray line is invisible to `doc_for_family`, `find_id`, `next_free`, `families` and
    `criteria_count`; it can never claim an id or advance a number. `check` is the module that
    reports it (`Kind::StrayCriterion`, hi: CHECK-2.e).

16. **A `Guard` is always a lock that was really taken.** `acquire` creates `hi/` before it tries
    the lock file, so the first capture in a repository locks like every later one, and any
    failure other than `AlreadyExists` returns an error rather than a guard. The constructor is
    private and takes the exclusively created `File`, so "a guard that did not acquire" is not a
    value that can exist; releasing one therefore cannot remove a lock somebody else holds
    (hi: FILE-19, DECISIONS.md §31).
17. **A lock is broken because nobody is holding it, never because it is old.** The holder
    refreshes the file four times a second from a thread of its own. A waiter remembers the mtime
    it first saw and the `Instant` it saw it, and breaks the lock only after that mtime has stood
    still for `ABANDONED` of the *waiter's own* elapsed time, re-reading it once more immediately
    before removing it so that a lock another waiter has just taken is never removed. Nothing
    compares this machine's clock to the file's, so clock skew on a shared filesystem cannot make
    a live lock look ancient (hi: FILE-19, FILE-22).
18. Every writer waits on the same path, `<root>/hi/.hi.lock`, and `root` comes from a
    `Workspace::find` that canonicalized its starting directory, so two processes reaching the
    same repository by different symlinked paths still contend for one lock.

## Behavioral Examples

### Scenario: Discovery from a nested directory

- **Given** a repository at `/repo` with `/repo/hi/chat.md` carrying `hi: 1` in its frontmatter,
  and the process standing in `/repo/src/deep`
- **When** `Workspace::find` is called with `/repo/src/deep`
- **Then** the walk climbs to `/repo`, `holds_hi_files(/repo/hi)` answers yes, and it returns a
  workspace with `root = /repo`, `dir = /repo/hi`, and one doc

### Scenario: A repository that has never run hi

- **Given** `/repo/.git` exists and `/repo/hi` does not
- **When** `Workspace::find` is called from a directory under `/repo` that has no `hi/` or `.git`
  of its own
- **Then** the walk climbs to `/repo`, finds no recognized `hi/` there but does find `.git`, and
  stops: `root = /repo`, `dir = /repo/hi`, `docs` empty, so capture can create the directory and
  the first file

### Scenario: A repository nested inside another

- **Given** `/outer/hi/chat.md` is a real workspace, and `/outer/child/.git` exists while
  `/outer/child/hi` does not
- **When** `Workspace::find` is called from `/outer/child`
- **Then** `holds_hi_files(/outer/child/hi)` answers no, `.git` answers yes, and the walk stops
  there: `root = /outer/child`, `docs` empty. `hi check` in the inner project reports
  `0 criteria` instead of the outer project's (hi: CAPTURE-10)

### Scenario: A Hindi locale directory is not a workspace

- **Given** `/repo/hi/chat.md` is the real workspace and `/repo/public/locales/hi/common.md` is a
  page of Hindi strings with no frontmatter, and the process is standing in
  `/repo/public/locales`
- **When** `Workspace::find` is called from there
- **Then** `holds_hi_files(/repo/public/locales/hi)` answers no because nothing in it declares
  `hi:`, the walk climbs past it, and with no `.git` in between it lands on `/repo`, so a capture
  writes to `hi/chat.md` and nothing is ever created inside the locale directory (hi: CAPTURE-6)

### Scenario: A hi file that starts with a byte-order mark

- **Given** `/repo/hi/chat.md` whose first bytes are a UTF-8 BOM followed by `---`
- **When** `Workspace::find` is called from `/repo`
- **Then** `holds_hi_files` strips the BOM before reading the first line, sees `---`, finds the
  `hi:` key, and the directory is recognized: the file is neither hidden from discovery nor
  made to look broken (hi: FILE-11)

### Scenario: Family resolved by use rather than declaration

- **Given** `hi/chat.md` whose frontmatter declares no families but whose criteria include
  `OFFLINE-1`
- **When** `doc_for_family("OFFLINE")` is called
- **Then** the frontmatter pass finds nothing, the fallback pass matches the criterion, and the
  index of `hi/chat.md` is returned

### Scenario: Next free number after a retirement

- **Given** a family whose active criteria stop at `SEND-2` and whose `## Retired` section holds
  `SEND-3`
- **When** `next_free("SEND")` is called
- **Then** it returns `4`, because retired ids are counted and their numbers are never reused

### Scenario: Looking up a retired id

- **Given** `SEND-3` sitting under `## Retired` in `hi/chat.md`
- **When** `find_id` is called with `SEND-3`
- **Then** the criterion is returned with its doc index, which is what lets capture refuse to
  reuse a retired id

### Scenario: Families across several files

- **Given** `hi/billing.md` declaring `[BILLING]` and `hi/chat.md` declaring `[SEND, RECEIPT]`
  while also using `OFFLINE`
- **When** `families()` is called
- **Then** it returns `["BILLING", "OFFLINE", "RECEIPT", "SEND"]`: union, deduplicated, sorted

### Scenario: Stable display paths

- **Given** a doc loaded from `/repo/hi/chat.md` in a workspace rooted at `/repo`
- **When** `rel` is called with that path
- **Then** it returns `hi/chat.md`, with forward slashes on every platform, which is the form
  that appears in check problems, capture output and the export payload (hi: FILE-12)

### Scenario: The first two captures in a repository that has never run hi

- **Given** `/repo/.git` exists, `/repo/hi` does not, and two `hi` processes start at once
- **When** both call `lock::acquire(/repo/hi)`
- **Then** both `create_dir_all` the directory (the loser of that race is told it already exists,
  which is success), one exclusive-creates `/repo/hi/.hi.lock` and the other waits for it. Each
  reloads the workspace under the lock, so the second one sees the file the first one wrote and
  appends to it instead of writing its own copy over it (hi: FILE-19)

### Scenario: A capture that refuses before `hi/` existed

- **Given** the same fresh repository, and `hi SEND-1.a "a case with no parent"`
- **When** capture refuses because `SEND-1` does not exist
- **Then** the guard releases: the lock file goes, and because acquiring had created `hi/`, the
  now-empty directory goes with it. Nothing at all was written (hi: CAPTURE-5)

### Scenario: A holder that is slower than the abandonment window

- **Given** a capture that holds the lock for a minute on a slow filesystem
- **When** another `hi` waits for it
- **Then** the waiter sees the lock's mtime move every 250ms, restarts its watch each time, and
  never breaks the lock. It waits out `PATIENCE` and says so instead. The old rule broke any lock
  older than sixty seconds, which said the holder was slow and not that it was dead

### Scenario: A hi that was killed holding the lock

- **Given** `hi/.hi.lock` left by a process that no longer exists, so nothing is refreshing it
- **When** the next capture runs
- **Then** its mtime stands still for `ABANDONED`, the waiter re-reads it, finds it unchanged,
  removes it and takes its own. The repository recovers without anybody deleting a file by hand
  (hi: FILE-22)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `start` cannot be canonicalized because it does not exist, or because a symlink in it is broken | `find` fails with `resolving <path>` wrapping the underlying I/O error |
| No ancestor holds a `hi/` directory with a hi file in it, and none holds a `.git` entry | `find` fails with `this is not a repository, and no hi/ directory was found above it. hi anchors to a repository, so run it inside one` |
| An ancestor holds a `hi/` directory but nothing inside it declares `hi:` in frontmatter | Not a workspace. The walk climbs past it and uses a farther real `hi/` instead, or the first `.git` boundary at or above that level (hi: CAPTURE-6) |
| A file inside a candidate `hi/` cannot be read, or is not valid UTF-8 | It simply does not count toward recognition; the other files in the directory are still examined. `holds_hi_files` never errors |
| A candidate `hi/` cannot be listed, or `hi` is a file rather than a directory | `read_dir` fails, `holds_hi_files` returns `false`, and the walk moves up |
| `<root>/hi` exists but cannot be listed, for example on a permission error | `load` fails with `reading <dir>` wrapping the underlying I/O error |
| A `*.md` file inside `hi/` cannot be read | `Doc::load` propagates its own `reading <path>` error and the whole load fails; the workspace is never returned half-populated |
| A directory entry inside `hi/` cannot be read from the iterator | That entry is silently skipped by the `filter_map(entry.ok())`; the remaining files still load |
| An entry is listed but cannot be stated | `path.is_file()` returns `false` on the metadata error, so the entry is skipped the same way a directory would be; the remaining files still load |
| A hi file is structurally broken by a malformed id, a missing parent, or a duplicate | Not an error here. The file loads, the criterion carries its parse error, and `hi check` reports it |
| `doc_for_family` is asked about an unknown family | `None`, which capture reads as "start a new file", never as a failure |
| `find_id` is asked about an id nothing uses | `None` |
| `next_free` is asked about a family with no criteria | `1` |
| `hi/` cannot be created, because the repository root is not writable | `acquire` fails with `creating <dir>` wrapping the underlying I/O error, before anything else happens |
| The lock file cannot be created for any reason other than another writer holding it | `acquire` fails with `could not take the write lock <path>` and a hint naming the directory hi has to be able to write in. It never returns a guard it did not take, and the file on disk is untouched |
| Another writer holds the lock and keeps holding it | `acquire` retries for `PATIENCE` and fails with `another hi is writing to <dir> and has not finished`, plus the file to delete if nothing is running |
| The heartbeat cannot reopen the lock file | The thread stops. The guard still removes the file on release; a lock that stops being refreshed is recoverable by the next waiter, which is the same path a killed process takes |
| `rel` is given a path that is not under `root` | Returns that path's own components joined with `/`, so an absolute one comes back with a doubled leading separator. No caller in hi does this |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std::fs` | `canonicalize`, `read_dir`, `read_to_string` (the frontmatter sniff in `holds_hi_files`) |
| `std::path` | `Path`, `PathBuf` |
| `std::fs` (lock) | `create_dir_all`, `OpenOptions::create_new` for the exclusive create, `metadata`/`modified` for the abandonment watch, `remove_file`, `remove_dir` |
| `std::sync::mpsc` | `channel`, `recv_timeout`: the heartbeat's timer, and how the guard ends it |
| `std::thread` | `spawn` for the heartbeat, `sleep` for the retry |
| `std::time` | `Duration`, `Instant` (the waiter's own elapsed time), `SystemTime` (the lock's mtime, compared only against itself) |
| `anyhow` | `Result`, `Context::with_context` for I/O context, `bail!` for the no-workspace error |
| `doc` | `Doc`, `Doc::load`, `Doc::all`, `Doc::used_families`, `Doc::criteria`, `Doc::front.families`, `Criterion` |
| `id` | `Id` for `find_id` lookups, `Id::family` and `Id::root_number` for family and numbering queries |

### Consumed By

| Module | What is used |
|--------|-------------|
| `main` | `Workspace::find` once per invocation, before capture or any subcommand; `run_capture` and `run` are its only two call sites in the crate. Each hands over the current directory, or `--root` when it was given. Capture gets that value from `peel_root`, which lifts a *leading* `--root` out of argv before routing so it cannot be joined into the criterion's sentence; once the id appears `peel_root` stops looking, and every word after it is the sentence (hi: CAPTURE-8, CAPTURE-9). The clap path gets its own copy from `cli.root`. Either way what `find` receives is a *starting* directory it walks up from, not a boundary. The boundary is the `.git` entry `find` meets on the way up |
| `capture` | `find_id` (refuse a taken id, and resolve the **parent's own file** first so a case lands where its parent lives; hi: CAPTURE-4.a), `next_free` (the hint), `doc_for_family` (the fallback when the parent gives no file), `dir` (create `hi/` and place a new file), `docs` (mutate and append), `rel` (output paths) |
| `check` | `docs` (iterate and count), `criteria_count`, `families`, `rel` (problem locations) |
| `out` | `docs` (`ls`, `export`, `index`), `find_id` (`issue`), `families` (resolve an export scope), `intent_path` (read and rewrite `INTENT.md`), `rel` (file names in output) |
| `view` | `docs` and `criteria_count` (render the page), `intent_path` (read the product prose), `root` (both the page's title, from its file name, and the base for the output path), `rel` (report where the page was written) |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Verification pass against `src/workspace.rs`: corrected `find` to a per-level walk (a nearer `.git` beats a farther `hi/`), removed the claim that `check` reports two files declaring the same family, corrected the index-invalidation invariant (an append keeps indices valid), narrowed the "everything hi writes" claim to the four modules that actually write, restated `next_free` as one past the highest number, and re-attributed the `rel` and `intent_path` test coverage. |
| 2026-09-16 | Leif | Verified the reconciliation against `src/workspace.rs`, `src/main.rs` and the full test suite: the discovery rewrite checks out line for line. Corrected three things it left behind: the declaration pass now reads a `front.families` that `doc` fills from inline, block or singular-`family:` form alike (invariant 9); `main`'s two call sites get `--root` from `peel_root` and from `cli.root` respectively, not both from `peel_root`; and `intent_path` is in fact asserted end to end by the `hi index` tests in `tests/cli.rs`, not merely executed. Note for a future reader: the `///` comment on `find` in the source still describes the old `.git` stop and contradicts the code below it. |
| 2026-09-16 | Leif | Reconciled with the bug-fix pass: `find` now recognizes a workspace by content via the new private `holds_hi_files` (a `*.md` carrying a `hi:` key, BOM stripped) instead of by the directory name, and `.git` became a remembered fallback rather than a stop, so a farther real `hi/` now beats a nearer repo root, reversing the previous rule. Added the Hindi-locale and BOM scenarios, the recognition-gates-discovery-not-loading invariant, the fenced/stray-lines-are-invisible invariant, and the `--root`/parent-first notes for `main` and `capture`. |
| 2026-09-16 | Leif | Drift pass against the shipped binary. `.git` is a **stop** again, not a remembered fallback: there is no `git_root`, the walk returns `load(dir)` at the first `.git` it meets, and a repo nested inside another no longer adopts the outer one's criteria (hi: CAPTURE-10, verified with `hi check` in a nested `.git` fixture reporting `0 criteria`). Replaced the not-found message with the one the binary prints. Corrected `rel`: it now joins components with `/` on every platform (hi: FILE-12), and its outside-`root` fallback is no longer the path unchanged. Noted that `out::write_index` shares the public `doc::write_atomically`, that `peel_root` only consumes a `--root` that precedes the id, and that `view` takes the page title from `root`. Added the nested-repository scenario. |
| 2026-09-17 | Claude | `lock::acquire` now creates `hi/` before taking the lock and fails closed on anything but a lock another writer holds, so the first capture in a repository is locked like every later one; `Guard`'s constructor is private and takes the exclusively created file, so a guard that never acquired cannot exist to remove somebody else's lock on drop. Staleness is no longer age: the holder heartbeats and a waiter breaks a lock only after watching its mtime stand still for `ABANDONED` of its own elapsed time. `PATIENCE` 5s → 30s, `STALE` 60s → `ABANDONED` 5s. Rewrote the `acquire`, `Guard` and `Guard::drop` rows, narrowed invariant 1 to `src/workspace.rs`, added invariants 16–18, four scenarios, four error rows and the new `std` dependencies (hi: FILE-19, FILE-22, CAPTURE-5, DECISIONS.md §31). |
