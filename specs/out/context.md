---
spec: out.spec.md
---

## Key Decisions

- `out` is no longer only reached by the read verbs. `refresh_index` is `write_index` with its
  `Result` turned into an `Option<String>`, and `capture` and the `hi retire` arm both call it once
  the criterion they wrote is on disk. The generated list was drifting in real repositories because
  nothing made anyone run `hi index`, and hi's own repo only escaped it because
  `scripts/index-is-current.sh` is in its gate, which no adopter has (DECISIONS.md §30,
  hi: INDEX-4). Nothing about what gets rewritten changed; only who carries the failure.
- `index_note` is the read-only half, for `check`. It is a `note:` and not a `check::Kind`,
  and it must stay one: `hi check` fails on a structurally broken file and nothing else
  (hi: INDEX-4.b, CHECK-1, HI-1.md).
- `out` is the read half of hi. It takes an already-loaded `Workspace` and renders it. It does not
  open, parse, or write any `hi/*.md` file; those belong to `doc`, `workspace`, and `capture`.
  `INTENT.md` is the single file this module writes, through `doc::write_atomically`, which also
  creates and renames away a sibling `.INTENT.md.hi-tmp`.
- `hi issue` prints by default and only shells to `gh` behind `--create`. That is deliberate: hi
  must work with no auth, no network, and no tracker integration, because the moment generation
  needs setup it stops being used (hi: ISSUE-1.a, ISSUE-1.b, and DECISIONS.md §6 "Generation").
- `gh` is invoked with `Command::new("gh").args([...])`, an argv, never a shell string. Criterion
  sentences are arbitrary human prose and will contain quotes, backticks, and `$`. Keep it argv.
- `status()` rather than `output()`: `gh`'s stdio is inherited so its own errors and the URL of the
  created issue reach the user directly. A non-zero exit becomes the terse `gh issue create failed`
  precisely because `gh` has already said the useful part.
- The export envelope is fixed at every scope (hi: EXPORT-3). `Export` always carries `hi`, `scope`
  and `files`, and every `ExportFile` always carries the same keys. `product` is the one
  scope-dependent key, and it is `skip_serializing_if = "Option::is_none"` because the
  product-level why only belongs to a whole-repo export. Do not add another conditional key without
  re-reading EXPORT-3.
- `scope` in the payload is the literal string the caller asked for, falling back to `"repo"`. A
  consumer reads it as a label, not as a discriminator.
- Ticket titles get `trim_end_matches('.')`. This removes *every* trailing period, so
  `"it just works..."` becomes `"it just works"`. That is the only text transformation anywhere in
  this module; everything else reproduces human prose verbatim (hi: FILE-4).
- The `Cases:` list in `issue_markdown` is `format!("- {indent}{} {}\n", ...)`: the depth indent is
  emitted **after** the list marker. The `indent` local exists only to nest, and in Markdown it
  cannot, because a list marker must be preceded by its indent. So `SEND-1.a.1` renders as a peer
  of `SEND-1.a`, and five spaces (depth 4) turn the sentence into an indented code block. ISSUE-3.a
  asks for exactly the nesting this loses. Anyone moving the indent in front of the bullet should
  do it together with a test that renders the body, not one that greps it: this is the FILE-1.b
  lesson from DECISIONS.md §12 on a second surface.
- A retired criterion is *found* and then refused, not reported as missing. `Workspace::find_id`
  searches both sections precisely so the message can say "is retired" rather than "does not exist"
  (hi: ISSUE-4).
- `write_index` is a splice, not a regeneration: it copies `existing[..span.start]`, writes the
  generated block, and copies `existing[span.end..]`. Prose outside the markers is preserved byte
  for byte (hi: INDEX-2).
- Marker matching is **whole-line**, in `index_span` and `has_marker_line` alike: `line.trim()`
  compared to the whole marker, over `split_inclusive('\n')`. Do not put this back to `find()` or
  `contains()`. A substring search matches a marker quoted in a sentence, and the splice then
  rewrites everything from that sentence to the real close, which is exactly the prose INDEX-2
  exists to protect. The doc comment on `index_span` says so; leave it there.
  on a line inside a fenced code block. `index_span` tracks ``` fences and ignores any marker
  inside one, so a person can document the format in their own prose and `hi index` still finds
  the real block below it (hi: INDEX-2.a, closed in 0.2.0).

## Files to Read First

- `src/out.rs`: the whole module, in four marked sections
  (`ls`, `issue`, `export`, `index`) plus its `#[cfg(test)]` module.
- `DECISIONS.md`: §6 "Generation" explains why `issue`
  prints by default and why `export` is a handoff rather than a spec writer; §8 assumption 5 is the
  one-shape-at-every-scope rule; §5 explains the absences (no state, no lifecycle, no evidence).
- `hi/generate.md`: the ISSUE, EXPORT, and INDEX criteria
  this module exists to serve. `./target/release/hi export generate` prints it as the payload.
- `src/workspace.rs`: `Workspace::rel`, `find_id`,
  `families`, `intent_path`, and the sorted load order that makes `ls` and `index` output stable.
- `src/doc.rs`: `Doc`, `Criterion`, `Section`, the
  `all()` / `used_families()` / `name()` helpers this module leans on, and `write_atomically`,
  which `write_index` calls.
- `src/id.rs`: `Id::depth`, `parent`, `is_descendant_of`,
  which drive indentation, the `parent` field in the payload, and the `Cases:` list.
- `src/main.rs`: the clap subcommands that call in here,
  and the flag names (`--family`, `--retired`, `--create`, `--repo`) that map to the parameters.

## Current Status

Fully implemented. All four verbs are wired into `main.rs` and work end to end against this
repository's own `hi/` directory. `./target/release/hi export` and `./target/release/hi index`
are how `INTENT.md` stays current.

Inline test coverage is real but uneven. Of the eight `#[test]` functions, five cover
`issue_markdown` and `export`, one covers `index_block`, and two cover the marker helpers
(`a_marker_quoted_in_prose_is_not_the_generated_block`, `an_unclosed_marker_has_no_span`). Those
two test `index_span` and `has_marker_line` directly, over string literals, which is why they need
no temporary directory. `write_index` itself is still uncovered inline because it touches the real
filesystem; `tests/cli.rs` covers it end to end instead, with `index_rewrites_only_the_generated_block`,
`index_refuses_rather_than_guessing_when_a_marker_is_unclosed`, and
`index_leaves_a_marker_quoted_in_prose_alone`. `tests/cli.rs` also carries
`export_stdout_is_parseable_json`, `export_rejects_a_scope_that_matches_nothing`,
`export_accepts_the_path_it_prints` (the stem, file-name, and `hi/<stem>.md` spellings, exit status
only), and `issue_prints_a_ticket_carrying_the_id`. `ls` has no coverage anywhere: it only prints.
The unit-test helper builds a `Workspace` by hand from `Doc::parse`, with `root: /r` and
`dir: /r/hi`, so no temporary directory is needed there. Its `CHAT` fixture is still written in the
bare, bulletless line form, which is deliberate: it also pins that the parser accepts a
hand-written file, which is what `strip_bullet` and `strip_emphasis` exist for.

No known blockers.

## Notes

- Non-obvious in `export`: a scope that is both a family name and a file stem is not a tie-break;
  both matches apply. `is_this_file` short-circuits the skip condition and the `keep` closure for
  the file it names, so that file comes through whole; every *other* file holding the family still
  satisfies the `is_family` arm of the skip condition and comes through filtered to that family.
  Verified with `hi/SEND.md` (holding `ALPHA-1`) beside `hi/other.md` (holding `SEND-1`, `BETA-1`)
  and scope `SEND`: two file entries, `hi/SEND.md` with `ALPHA-1` and `hi/other.md` with `SEND-1`.
  In practice families are uppercase and stems lowercase, so the collision does not arise.
- Non-obvious in `export`: `is_family` comes from `Workspace::families`, which includes frontmatter
  declarations, but file selection additionally requires `doc.all()` to hold a criterion of that
  family. A family declared in frontmatter and used by nothing therefore selects no file, and the
  export bails with `nothing matches '<scope>'. Give a family like SEND, a file like chat, or
  nothing at all for the whole repository`. That message no longer asserts the family does not
  exist, but it still answers a correct family name by listing the three things a scope can be, and
  it never says the family is declared and empty. `hi ls --family <it>` is quieter about the same
  situation: the `no criteria yet` hint, exit 0.
- Non-obvious in `export`: `matches_file` has three arms, `scope == stem`, `scope == "<stem>.md"`,
  and `scope.trim_start_matches("./").ends_with("hi/<stem>.md")`. The third is a **suffix test on a
  string**, not a path resolution, so `/anywhere/hi/chat.md` and even `xhi/chat.md` match, while
  `hi\chat.md` does not. The point of the arm is that `Workspace::rel` prints `hi/chat.md` with
  forward slashes on every platform (hi: FILE-12), so the path in `ls` output and in the payload's
  `file` key can be pasted straight back in as a scope. If this ever grows a fourth arm, remember
  that `scope` in the payload echoes the raw string, so one file already has four names.
- Non-obvious in `export`: `ExportCriterion.retired` is `Criterion::note`, and `doc` records a
  `retired:` continuation line wherever it is written. An active criterion with a stray `retired:`
  line therefore ships inside `criteria` carrying a `retired` key, and `hi issue` will still make a
  ticket of it, because `issue` gates on `Section::Retired` rather than on the note. Retirement is
  which array the entry is in.
- Non-obvious in `export`: a family scope keeps the family's *retired* criteria too. `keep` is
  applied to both `doc.criteria` and `doc.retired`, and file selection uses `doc.all()`. So a file
  that holds only retired criteria of the scoped family is still included, with an empty `criteria`
  list and a populated `retired` one. EXPORT-4 now states this outright ("Retired criteria come
  along, kept apart from the live ones"), so it is settled intent rather than an accident.
- Non-obvious in `ls`: the family filter uses `is_none_or` over `c.id`, so a criterion whose id
  failed to parse is invisible under any family filter but visible in an unfiltered listing. That
  is fine, because `hi check` is where malformed ids get reported.
- Non-obvious in `ls`: top-level criteria are indented two spaces, not zero, because indentation is
  `2 * depth` and a top-level id has depth 1. The file path is the only flush-left line.
- Non-obvious in `index_block`: the link is built as `hi/{stem}.md` from `doc.name()`, not from
  `workspace.rel(&doc.path)`. Correct today because `Workspace::load` only ever reads `<root>/hi`;
  it would silently go wrong if hi files were ever loaded from elsewhere.
- Non-obvious in `write_index`: a file containing only the opening marker, or the markers in the
  wrong order, is now refused rather than appended to (both produce `index_span() == None` with
  `has_marker_line(INDEX_OPEN) == true`). The append branch is still reachable (an `INTENT.md`
  with no opening marker line at all), and it remains the one place bytes outside the block change:
  it writes `existing.trim_end()`, so trailing blank lines are dropped and a file that already had
  a `## Features` heading gains a second one. That is still not an error and still not reported.
- Non-obvious in `write_index` and `read_product_intent`: `INTENT.md` gets the atomic write `doc`
  gained but not the rest of its hygiene. No BOM is stripped, so a byte-order mark immediately
  before an opening marker hides that line from both (`U+FEFF` is not whitespace, so `trim()` keeps
  it) and the append branch runs silently. `Doc::parse` and `holds_hi_files` both strip one. No
  line ending is detected, so the generated block is always LF; a CRLF file keeps CRLF in its prose
  and on the closing marker's own line, because that newline is outside the span, and `hi index`
  stays idempotent over it (verified twice over a CRLF fixture). Both are current behavior, not a
  decision recorded anywhere; see tasks.md before "fixing" one in isolation.
- Non-obvious in `read_product_intent`: it strips the generated block and nothing else. It does not
  call `view::strip_comments`, so an HTML comment in `INTENT.md` prose lands in `product` verbatim,
  including the `<!-- What is this product for, holistically? ... -->` prompt that `write_index`
  puts in a starter file. A repository where nobody has answered that prompt therefore hands an
  agent a `product` string consisting of a heading, the prompt, and `## Features`. It is not empty,
  so the `None` fallback does not catch it.
- Non-obvious in `index_block` and `export`: the families they report are `doc.front.families`, and
  the parser now fills that from a YAML block list as well as an inline `families: [A, B]`. A
  block-style file therefore lists its *declared* families in both outputs; the `used_families()`
  fallback only runs when the declaration is absent or empty. Verified with `families:` / `  - SEND`
  / `  - RECEIPT` over a file also using `BETA-1`: the index line reads `SEND, RECEIPT`.
- Non-obvious in `read_product_intent`: marker detection is `line.trim() == marker` on a whole
  line, the same rule `index_span` uses, so a marker wrapped in other text on the same line is
  not recognized and the index block would leak into `product`. `write_index` always emits the
  markers on their own lines, so this only matters for a hand-edited `INTENT.md`. Note the two
  functions agree on the rule but do not share code; change one and you have to change the other.
- Non-obvious across the whole module: it reads `doc.criteria`, `doc.retired`, and `doc.all()`
  (which is just those two chained) and never `doc.stray`. The parser now treats fenced code blocks
  as opaque and records criterion-shaped lines found outside every section in `Doc::stray`. So a
  documented example of the format inside a ``` fence is prose here, neither listed nor exported
  nor counted by `index_block` (hi: FILE-9), and a stray criterion is likewise
  invisible. Neither is a bug to fix here: `hi check` reports the stray one as `stray-criterion`
  (hi: CHECK-2.e), and this module has no exit code to report anything with.
- `read_product_intent` returns `None` when the file is unreadable *or* when what is left after
  stripping the block is empty. Both are silent: a whole-repo export of a repository with a
  placeholder `INTENT.md` simply has no `product` key.
- `INDEX_OPEN` / `INDEX_CLOSE` are private consts in this module. `check` does not know about them;
  nothing validates that `INTENT.md` still contains them.
