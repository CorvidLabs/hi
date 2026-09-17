---
spec: out.spec.md
---

## User Stories

- As someone who just wrote down a criterion, I want to turn it into a ticket without leaving the
  terminal, so the thought becomes work while I still have it (hi: ISSUE-1)
- As someone whose team does not use GitHub Issues, I want `hi issue` to print by default, so it
  works with whatever tracker I actually use and needs no auth (hi: ISSUE-1.a)
- As someone whose team does use GitHub, I want one flag to open the real issue, so I am not
  copying and pasting a body between two windows (hi: ISSUE-1.b)
- As someone reading a closed ticket a year later, I want the criterion id in the body, so I can
  trace the work back to the intent it served (hi: ISSUE-2)
- As a reviewer picking up a ticket, I want the criterion's edge cases in the same body, so the
  ticket is the whole picture rather than one sentence (hi: ISSUE-3)
- As that same reviewer, I want a case to read as nested under what it is a case of, rather than
  flattened into a list of peers (hi: ISSUE-3.a)
- As whoever picks the ticket up, I want the feature's intent prose in the body, so I know why the
  work exists and not only what to build (hi: ISSUE-5)
- As whoever picks the ticket up, I want that prose to read as paragraphs in a wide issue pane
  rather than a narrow column broken after every line, because where the author wrapped their own
  file is not where they wanted a break (hi: ISSUE-7)
- As the author of that prose, I want the blank lines I left, and any list or example I wrote, to
  arrive on the ticket intact, because those newlines are the ones I meant (hi: ISSUE-7.a,
  ISSUE-7.b)
- As somebody adopting hi, I want the files hi writes for me to be one line per paragraph, so the
  first thing I read models the convention rather than an exception to it (hi: FILE-21.a)
- As someone who changed their mind, I want a retired criterion to refuse to become a ticket, so
  a decision we reversed cannot quietly re-enter the backlog (hi: ISSUE-4)
- As someone handing a feature to an agent, I want one command that emits the intent prose and
  every criterion, so the agent can write the spec-sync spec from it (hi: EXPORT-1, EXPORT-1.a)
- As someone working at different granularities, I want to export one family, one file, or the
  whole product (hi: EXPORT-2)
- As the author of whatever consumes the payload, I want the same JSON shape at every scope, so
  nothing downstream has to branch on which scope was asked for (hi: EXPORT-3)
- As someone handing a file to an agent, I want retired criteria to come along kept apart from the
  live ones, so the agent never writes a spec for something we dropped (hi: EXPORT-4)
- As the owner of the repository, I want the root file to show what features exist without me
  maintaining a list by hand (hi: INDEX-1)
- As someone with no root file yet, I want hi to start one with a place for my own prose, rather
  than refusing until I set it up (hi: INDEX-1.a)
- As the author of the prose in `INTENT.md`, I want hi to touch only its own generated block, so
  the paragraphs I wrote stay mine (hi: INDEX-2)
- As someone who wrote *about* the markers in that prose, I want a marker quoted mid-sentence to
  stay a sentence, so documenting how hi works does not make hi eat the documentation
  (hi: INDEX-2.a)
- As someone who edited `INTENT.md` by hand and broke a marker, I want hi to stop and tell me,
  rather than guess where my block ended and take the prose with it (hi: INDEX-2.b)
- As the owner of the repository, I want the list at the front of my product to be true after every
  command that changes it, without me remembering to refresh it (hi: INDEX-4)
- As someone capturing a thought, I want a capture that stored my criterion never to be reported as
  a failure because the list could not be refreshed, because the thought is the thing that mattered
  (hi: INDEX-4.a)
- As someone who typed a criterion straight into a file, I want hi to tell me the list is behind
  rather than leave it wrong, because no verb saw me do it (hi: INDEX-4.b)
- As someone browsing what has been captured, I want a flat listing of every criterion grouped by
  file, so I can see the shape of the intent at a glance

## Acceptance Criteria

### REQ-out-001

`ls` SHALL print every active criterion grouped by its file and indented by id depth, filtered to
one family on request and including retired lines only on request.

Acceptance Criteria

- Files are walked in workspace load order and each file that has something to show is preceded by
  its workspace-relative path.
- A criterion is printed as `<raw id>  <sentence>` indented by two spaces per level of id depth, so
  a case sits visibly under its parent.
- A file with nothing matching the filter is skipped entirely, including its path heading.
- Retired criteria are printed only when `include_retired` is set, and carry a trailing
  `(retired)` marker so they are never mistaken for live intent.
- With a family filter, only criteria whose parsed id belongs to that family are shown.
- The verb returns `()`: there is no failure mode, because an empty or unfinished repository is the
  normal state of intent.

### REQ-out-002

`hi issue` SHALL print the ticket to stdout by default, requiring no authentication, no network,
and no tracker integration (hi: ISSUE-1, ISSUE-1.a).

Acceptance Criteria

- Without `--create`, the command writes `## <title>`, a blank line, and the body to stdout and
  returns success.
- No external process is started and no file is read beyond the workspace already loaded.
- The title is the criterion's sentence with trailing `.` characters removed, so it reads as a
  ticket title rather than a sentence.

### REQ-out-003

The ticket body SHALL carry the criterion's permanent id, its active cases, and the intent prose of
the file it lives in (hi: ISSUE-2, ISSUE-3, ISSUE-5).

Acceptance Criteria

- The body opens with the criterion sentence verbatim, then a line reading `hi: <raw id>`, so a
  closed ticket traces back to the intent it served.
- Every active criterion whose id is a descendant of the subject appears in a `Cases:` bullet list.
- Each case line is `- `, then two spaces per level of depth below the subject, then the raw id, a
  single space, and the sentence. Because the indent is emitted after the list marker rather than
  before it, only the source is offset: rendered as Markdown the whole list is one flat level, so a
  step inside a case reads as a peer of that case. ISSUE-3.a asks for visible nesting and this is
  not it; the shipped behavior is what is written here, and tasks.md carries the decision.
- A sibling criterion is not a case and never appears.
- When the file has non-blank `## Intent` prose, it is appended after a `---` rule under
  `Intent for <file stem>:`, with its soft line breaks unwrapped as REQ-out-015 describes.
- When the file has no intent prose, the rule and the section are both omitted rather than emitted
  empty.

### REQ-out-004

`hi issue --create` SHALL hand the identical title and body to `gh issue create` rather than
re-rendering them, and SHALL target a repository named with `--repo` when one is given
(hi: ISSUE-1.b).

Acceptance Criteria

- The same `issue_markdown` output that would have been printed is passed as `--title` and
  `--body`.
- `--repo <owner/name>` adds `--repo` to the `gh` invocation; without it, `gh` resolves the
  repository itself. `--repo` is read only on the `--create` path, so passing it to a plain
  `hi issue` is accepted and ignored.
- `gh` is spawned with an argument vector and inherited stdio, never through a shell, so a
  criterion sentence cannot be interpreted as shell syntax and `gh`'s own output reaches the user.
- A `gh` that cannot be started, and a `gh` that exits non-zero, are both reported as errors.

### REQ-out-005

`hi issue` SHALL refuse a retired criterion before rendering anything (hi: ISSUE-4).

Acceptance Criteria

- An id found in a file's `## Retired` section is rejected with a message saying it is retired and
  should not become work.
- The refusal happens whether or not `--create` was passed, so no GitHub issue is ever opened for a
  retired id.
- A retired id is still *found* rather than reported missing, so the message explains the real
  situation instead of implying the id never existed.

### REQ-out-006

`export` SHALL emit one envelope shape at every scope, so a downstream consumer never branches on
which scope was requested (hi: EXPORT-3).

Acceptance Criteria

- Every payload carries `hi` (the format version), `scope`, and `files`.
- `scope` is the string the caller asked for, or `repo` when nothing was asked for.
- Every entry in `files` carries `file`, `intent`, `families`, `criteria`, and `retired`, plus
  `title` when the file has an `# H1`.
- Every criterion entry carries `id`, `text`, `depth`, and `parent`, plus `retired` when the line
  carries a `retired:` note. `parent` is `null` for a top-level criterion.
- The `retired` key means "this line has a `retired:` note", not "this criterion is retired": a
  `retired:` continuation written under an active criterion produces the key on an entry sitting in
  `criteria`. A consumer reads retirement from which array the entry is in.
- The only key that varies by scope is `product`, which is the product-level why and belongs to a
  whole-repo export alone.
- `retired` is always present as an array on every file entry, at every scope, so a consumer reads
  what was dropped from the same place every time (hi: EXPORT-4).

### REQ-out-007

`export` SHALL accept a family, a file, or no scope at all, selecting the right files and
criteria for each (hi: EXPORT-2).

Acceptance Criteria

- With no scope, every hi file in the workspace is included with all of its criteria.
- With a family scope, only files that hold at least one criterion of that family are included, and
  within them only that family's criteria, active and retired alike.
- With a file scope, that one file is included with all of its criteria, whatever families they
  belong to.
- A file may be named four ways, all handled by the private `matches_file`: the bare stem (`chat`),
  the file name (`chat.md`), the repository-relative path (`hi/chat.md`), and that path with a
  leading `./`. The third spelling is exactly what `ls` and the payload's own `file` key print, so
  the path hi shows can be pasted back in as a scope.
- The path form is matched as a suffix, not resolved: any string ending in `hi/<stem>.md` selects
  the file, including `/anywhere/hi/chat.md` and `xhi/chat.md`. The comparison is case-sensitive
  and uses the forward slash `Workspace::rel` emits on every platform (hi: FILE-12), so `HI/chat.md`
  and `hi\chat.md` are refused.
- `scope` in the payload echoes whichever spelling was asked for, so three spellings of one file
  give three different `scope` strings and the same `files` array.
- A scope that is both a family and a file stem matches both, not one: the file it names comes
  through whole rather than as one family's slice, *and* every other file holding that family is
  still included, filtered to that family. The stem only widens the file it names.
- A family that frontmatter declares but no criterion uses selects no file, because file selection
  requires a matching criterion in `doc.all()`. Such a scope is refused by REQ-out-009's error path.
- Scope matching is case-sensitive, and exact for the family, stem, and file-name forms; families
  are uppercase and file stems are lowercase, so the two namespaces do not collide in practice.

### REQ-out-008

The export payload SHALL carry human prose, not only criteria, so an agent receives the intent
behind the list (hi: EXPORT-1, EXPORT-1.a).

Acceptance Criteria

- Each file entry carries the full `## Intent` prose of that file.
- A whole-repo export additionally carries the prose of `INTENT.md` in `product`.
- The generated index block is stripped out of `product`, so the agent reads the product-level why
  and not a list it could have derived itself.
- A missing, unreadable, or effectively empty `INTENT.md` omits `product` rather than failing or
  emitting an empty string.
- Only the generated block is stripped. `read_product_intent` does not use `view::strip_comments`,
  so an HTML comment in the prose reaches `product` as written, including the
  `<!-- What is this product for, holistically? ... -->` prompt in a starter file that nobody has
  filled in yet, along with the `# <root>` and `## Features` headings around it.

### REQ-out-009

`export` SHALL refuse a scope that selects no file, and SHALL NOT treat an empty repository as an
error.

Acceptance Criteria

- A stated scope that selects no file exits with exactly
  `nothing matches '<scope>'. Give a family like SEND, a file like chat, or nothing at all for the
  whole repository`, which names the scope and then names the three things a scope can be.
- A family declared only in frontmatter, used by no criterion, selects no file and therefore takes
  this same path. The message does not claim the family does not exist, but it does tell the person
  to give a family when they gave one, and it never says the family is declared and empty.
  `hi ls --family <it>` is quieter: it prints the `no criteria yet` hint and exits 0.
- A whole-repo export of a workspace with no hi files succeeds with an empty `files` list, because
  incomplete intent is the normal state of intent (hi: CHECK-1.b).

### REQ-out-010

`index_block` SHALL generate the feature list for the root file, one line per hi file, so nobody
maintains that list by hand (hi: INDEX-1).

Acceptance Criteria

- Each line is a Markdown bullet linking the file stem to `hi/<stem>.md`, followed by the file's
  families and its active-criterion count.
- Families come from frontmatter when declared and from the ids actually used otherwise; a file
  with neither reads `no families yet`.
- "Declared" means whatever the parser put in `doc.front.families`, which it fills from an inline
  `families: [SEND]` and from a YAML block list alike. Only an empty list falls back to used
  families, so a block-style file lists what it declares, in the index and in `export`'s
  `families` key, rather than the families its ids happen to use.
- The count is pluralized: `1 criterion`, `3 criteria`.
- Retired criteria are not counted, because a reserved id is not a feature.
- A workspace with no files yields the single line `- nothing captured yet` rather than an empty
  block.

### REQ-out-011

`write_index` SHALL rewrite only the span between the `hi:index` markers, and SHALL create the file
or the section when they do not exist yet (hi: INDEX-2, INDEX-1.a).

Acceptance Criteria

- When both markers are present on lines of their own and correctly ordered, everything before the
  opening marker and everything after the closing marker is preserved byte for byte.
- The replaced span ends at the last non-whitespace byte of the closing marker's line, so that
  line's own newline (and the blank line that usually follows the block) survives the rewrite.
- When `INTENT.md` is absent, empty, or whitespace-only, a starter file is written with a heading
  taken from the workspace root directory name, a comment prompting for the product-level why, a
  `## Features` heading, and the generated block.
- When `INTENT.md` has content and no opening marker line at all, the existing text is kept and a
  `## Features` section carrying the block is appended below it. Only the trailing blank lines of
  the existing text are lost, to `trim_end`.
- The function returns the written path relative to the workspace root, so the CLI can report it
  without leaking an absolute path.
- A write failure names the path it was writing, as `writing <path>` wrapping the I/O error.
- The generated block is written with `\n` line endings whatever endings the file already uses. A
  CRLF `INTENT.md` keeps CRLF in its prose and on the closing marker's own line (that newline sits
  outside the replaced span) and gains LF inside the block. Running `hi index` twice over such a
  file still changes nothing the second time.
- The write goes through `doc::write_atomically`, the same sibling-temp-then-rename that
  `Doc::save` uses for a `hi/*.md` file: `.INTENT.md.hi-tmp` is created, written, flushed, fsynced,
  and renamed over the target, and is removed if any of that fails. A write that runs out of space
  therefore leaves `INTENT.md` exactly as it was rather than truncated (hi: FILE-8, INDEX-2).
- Because the replacement is a rename, the directory has to be writable rather than the file: a
  read-only `INTENT.md` is replaced successfully, and comes back with the temporary file's
  permission bits rather than its own.

### REQ-out-012

`write_index` SHALL recognize a `hi:index` marker only when it stands alone on its own line, so a
marker quoted inside a sentence is prose (hi: INDEX-2.a).

Acceptance Criteria

- Both the span search and the unclosed-marker check compare a line's trimmed text to the whole
  marker; neither searches for the marker as a substring.
- A sentence that mentions `<!-- hi:index -->` in the middle of it is copied through untouched, and
  the block replaced is the one whose markers stand alone further down the file.
- Leading or trailing whitespace on a marker's own line does not stop it being recognized.
- The first opening marker line in the file wins; a second one before the close is inside the block
  and is replaced with it.
- The whole-line rule is the whole of the protection, and it is narrower than INDEX-2.a now reads.
  Neither helper tracks fenced code blocks, so a marker pair written on lines of their own inside a
  ``` fence is adopted as the real block: the generated list lands inside the fence and every byte
  from the fenced opening marker to the next closing marker line, prose included, is replaced. The
  same happens to any prose between two bare opening marker lines, since the first one wins. Both
  exit 0. This is the shipped behavior, recorded here because it is what the code does, not because
  it is wanted; tasks.md carries the decision.
- `read_product_intent` strips the block from `product` by the same whole-line rule, so a quoted
  marker does not truncate the product prose either.
- Recognition runs over the raw file text with no BOM stripping, unlike `Doc::parse` and workspace
  discovery. A byte-order mark immediately before an opening marker leaves that line unrecognized,
  since `U+FEFF` is not whitespace and `trim()` keeps it, so the append branch runs instead of the
  splice or the refusal.

### REQ-out-013

`write_index` SHALL refuse a file whose opening marker is never closed, rather than guess where the
generated block ends (hi: INDEX-2.b).

Acceptance Criteria

- An `INTENT.md` holding an opening marker line with no closing marker line after it, including
  the two markers written in the wrong order, exits with
  `<path> has an opening <!-- hi:index --> with no matching <!-- /hi:index -->. Fix the markers
  rather than have hi guess where the block ends`.
- The refusal happens before the write, so the file is left byte for byte as it was and no
  temporary file is created.
- The message names the file, relative to the workspace root.
- A file holding only a closing marker is not this case: with no opening marker line there is
  nothing to guess past, and the append branch runs.

### REQ-out-014

`ls`, `export`, and `index_block` SHALL render only the lines the parser accepted as structure, and
SHALL NOT report on the ones it did not.

Acceptance Criteria

- Every read here goes through `doc.criteria`, `doc.retired`, or `doc.all()`; `Doc::stray` is never
  read by this module.
- An id-shaped line inside a fenced code block was prose to the parser, so it is never listed,
  never exported, and never counted in the index (hi: FILE-9).
- An id-shaped line outside every section is invisible here and is reported by `hi check` instead,
  which is the verb that has an exit code to say it with (hi: CHECK-2.e).
- This module still reports no structural problem of its own: it renders what it was given and
  leaves diagnosis to `check`.

### REQ-out-015

The intent prose a ticket carries SHALL have its soft line breaks unwrapped, and SHALL keep every
newline that means something (hi: ISSUE-7, ISSUE-7.a, ISSUE-7.b).

Acceptance Criteria

- GitHub renders an issue or comment body with hard line breaks on, so a single newline there is a
  `<br>` even though the same bytes in a repository file are not. A person's `## Intent` prose is
  wrapped at their own margin, so without this the ticket is a narrow column down the left of a wide
  pane, broken after every authored line. This is the whole reason the transformation exists.
- A single newline between two lines of one paragraph is replaced by a single space, so the line
  keeps going.
- A blank line is a paragraph break and is preserved (hi: ISSUE-7.a).
- A newline is structural, and is preserved, when the line under it opens a block: a list item
  (`-`, `*`, `+`, or digits then `.` or `)`, each followed by a space or nothing), a block quote,
  an ATX heading of one to six `#`, a table row, a thematic break of three or more `-`, `*` or `_`,
  or a line opening raw HTML. A thematic break is tested before a list item, because `- - -` and
  `* * *` are both.
- Inside a fenced block every newline is the author's and nothing is joined. Fences are recognised
  through `doc::fence_marker`, the same helper the parser uses, so a fence means the same thing
  wherever hi reads markdown (hi: FILE-9, ISSUE-7.b).
- A line that ends in two or more spaces, or in a backslash, is markdown asking for a break on
  purpose. It is not a wrap, nothing is folded onto it, and the marker itself survives into the
  body.
- A wrapped line is joined onto a list item or a block quote as well as onto a paragraph, because a
  lazy continuation belongs to the item above it. It is never joined onto a heading, a table row, a
  rule or an HTML line.
- Indentation is read only where it is the author's: a line that opens its own paragraph is emitted
  exactly as written, so an indented code block survives, while a line reached with a paragraph
  already open is a lazy continuation whatever its indent.
- The transformation applies to `issue_markdown` alone. The printed ticket and the `--create` body
  are the same string, so they cannot diverge (REQ-out-004).

### REQ-out-016

The text hi writes into a file it creates SHALL itself be one line per paragraph (hi: FILE-21.a).

Acceptance Criteria

- `agent_instructions` and `starter_intent` both pass unchanged through the unwrapping of
  REQ-out-015: no paragraph in either is broken by a newline.
- This is a property of the strings, not a runtime check. The first file an adopter reads is the
  one they write the rest of their prose to match, so a hard-wrapped starter file propagates the
  defect REQ-out-015 renders around.
- `agent_instructions` says the convention in one sentence, which is the single narrowing of
  DECISIONS.md §27's rule that the file carries the habit and nothing else. It is admissible there
  because it is how markdown reads a newline rather than anything about hi's format, so it cannot
  go stale with a format that is not frozen (DECISIONS.md §29).
- Nothing here validates, rewrites or reports on the wrapping of a file hi did not write. The
  convention travels as documentation and as the example hi sets; FILE-4 forbids the rewrite and
  CHECK-1 forbids the gate.

### REQ-out-017

`refresh_index` SHALL rewrite the generated list for a verb that has just changed the live count,
and SHALL hand back any reason it could not rather than raising one (hi: INDEX-4, INDEX-4.a).

Acceptance Criteria

- It is `write_index` with the `Result` turned into an `Option<String>`. Every rule REQ-out-011,
  REQ-out-012 and REQ-out-013 state about what is replaced, what is preserved and what is refused
  holds unchanged; only who carries the failure moves.
- `capture` calls it after the criterion is on disk, and `hi retire` calls it after the criterion
  and its cases have been moved and saved. Nothing else calls it. `hi index` still goes through
  `write_index` directly, because there the failure is the whole answer and belongs in the exit
  code.
- INDEX-2.b's refusal to guess at an unpaired marker comes back as `Some(<message>)`. The file is
  still left byte for byte as it was, and the caller still exits 0, because the criterion that
  prompted the refresh is already stored (hi: INDEX-4.a).
- It acquires no lock. `capture` and the `hi retire` arm both hold `lock::acquire` across their
  whole read-modify-write and it is not reentrant, so a lock taken here would deadlock every
  writer (hi: FILE-19).
- The cost is accepted rather than unnoticed: a bulk capture of N criteria rewrites `INTENT.md` N
  times. fledge's adoption was 197 captures. Each rewrite is one atomic replace of a file of a few
  hundred bytes, under a lock that already serializes those captures.
- A person who deleted the generated block from `INTENT.md` gets one back on the next capture,
  through `write_index`'s existing append branch. That is the same thing `hi index` has always done
  and is recorded here because it is now reached without being asked for.

### REQ-out-018

`index_note` SHALL say when the generated list no longer matches the workspace, and SHALL write
nothing and decide nothing (hi: INDEX-4.b).

Acceptance Criteria

- It rebuilds the whole generated block and compares it to the bytes `index_span` found, so it
  answers exactly the question `scripts/index-is-current.sh` answers by regenerating: would running
  `hi index` change anything?
- When they differ it returns `<path>'s feature list is behind what is captured. Run \`hi index\``.
- When there is an opening marker line that `index_span` could not close, it returns
  `<path> has an opening <!-- hi:index --> with no matching <!-- /hi:index -->, so nothing can
  refresh its feature list`. That case is otherwise silent now, because REQ-out-017 swallows the
  refusal on the capture path, and a list nothing can refresh is a list left wrong.
- It returns `None` for a matching block, for an `INTENT.md` that cannot be read, and for one with
  no marker line at all. A file with no generated list is not a list that is behind, and the next
  capture adds one.
- It never writes, never opens a `hi/*.md`, and never produces a `check::Kind`. `check` prints it
  as a `note:` and the exit code does not move (hi: CHECK-1).

## Constraints

- No network access. `export`, `ls`, `index`, and the default `issue` read local files only, and
  `--create` is the sole path that touches anything outside the machine. This is the same offline
  posture CHECK-4 states for `hi check`, held here for the generation verbs; the criterion that
  binds this module directly is ISSUE-1.a.
- No mutation of `hi/*.md`. The only file this module writes is `INTENT.md`, and only inside the
  generated block. Capture owns every edit to a feature file.
- No prose rewriting on disk. Criterion sentences are reproduced verbatim everywhere. Two text
  transformations exist and both are render-time: trimming trailing `.` from a ticket title, and
  unwrapping the soft line breaks in the intent prose a ticket carries (REQ-out-015). Neither ever
  reaches a file. hi does not rewrite, reflow or reformat prose somebody wrote, which is FILE-4 for
  a `hi/*.md` and INDEX-2 for `INTENT.md`, and this module writes only the generated block in the
  latter. Bullets and bold around an id are stripped by the parser before this module sees
  anything, so `ls`, `issue`, and `export` all carry the bare `raw_id` whether the line was written
  `- **SEND-1**  x` or `SEND-1  x`.
- Output must stay stable and diffable: paths are printed relative to the workspace root, files are
  walked in the workspace's sorted load order, and the JSON is `serde_json` pretty-printed.
- Every path this module emits comes from `Workspace::rel`, which joins components with `/` on
  every platform. `ls` headings, the payload's `file` key, `write_index`'s return value, and the
  refusal message all read the same on Windows as on Unix, which is what makes the `hi/<stem>.md`
  form of an `export` scope portable (hi: FILE-12).
- `export` emits a fixed format version (`hi: 1`) so a consumer can pin to a payload shape.
- The module must remain panic-free over malformed input: a criterion whose id failed to parse
  still renders, falling back to depth 1.

## Out of Scope

- Parsing `hi/*.md` and rendering criterion lines, owned by the `doc` module.
- Discovering the `hi/` directory and loading its files, owned by the `workspace` module.
- Parsing, validating, or comparing ids, owned by the `id` module.
- Writing new criteria to a feature file, owned by the `capture` module.
- Structural validation and exit codes for badly-formed files, owned by the `check` module. `out`
  renders whatever the parser produced and reports no structural problems, including the stray
  criteria the parser now records in `Doc::stray`.
- Rendering the same workspace as a browsable HTML page, owned by the `view` module. `hi view` is
  the other read-only verb and it is not implemented here.
- Writing spec-sync specs. `export` is a handoff to an agent; hi has no spec-sync coupling and no
  knowledge of what the agent does with the payload.
- Tracker integration beyond shelling to `gh`. There is no API client, no token handling, and no
  support for any other issue tracker.
- Any notion of state, lifecycle, evidence, or whether a criterion is done. Nothing in this module
  reads or writes such a thing, because hi does not have one.
