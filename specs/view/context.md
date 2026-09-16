---
spec: view.spec.md
---

## Key Decisions

- **Escape first, then interpret markers.** `inline_markdown` runs `escape` over the whole string
  before scanning for `` ` ``, `*` or `[`. By the time a marker is interpreted, every `<` is already
  `&lt;`, so no combination of markers can reconstruct a tag. This ordering is the entire security
  argument for the module. Reversing it would be a vulnerability, not a refactor.
- **A hand-written inline subset rather than a markdown crate.** The supported set is four
  constructs. A full markdown library would pull in a dependency, would render block constructs
  nobody asked for, and, most importantly, would make the escaping guarantee something we assert
  about someone else's code rather than something readable in forty lines here.
- **Link schemes are allow-listed, not deny-listed.** Only `http://`, `https://` and a leading `/`
  produce an anchor. A deny-list of `javascript:` would miss `JavaScript:`, `java\tscript:`, `data:`
  and entity-encoded variants.
- **Comments are stripped from prose, before paragraphs are split.** `paragraphs` runs
  `strip_comments` first. Doing it after the split would leave a comment that straddles a blank
  line half-rendered. An unterminated `<!--` deliberately swallows the rest of the prose, matching
  what a browser does with the same input. That `return out` in the `None` arm is the behavior,
  not a missing case (hi: VIEW-1.c).
- **Empty prose emits nothing, not an empty element.** Both the feature intent and the product lead
  are only wrapped in a `<div>` when the rendered paragraphs are non-empty. This is what keeps hi's
  own new-file template (`doc::new_file_text` puts a single prompt comment under `## Intent`) from
  leaving a hollow block on the page. Removing either `is_empty` guard reintroduces it.
- **Comment stripping is a prose-only rule.** Criterion sentences go through `escape` instead, so a
  `<!--` in a sentence is visible text. That asymmetry is intended: prose is where a person thinks
  out loud, a criterion is the thing being quoted.
- **No script element at all.** The page has no interactivity beyond the native `<details>` for
  retired criteria, so there is nothing to execute and the file is trivially safe to email.
- **A system font stack, not a webfont.** The page linked Google Fonts once and no longer does. The
  promise is one file you can send to anyone, and a stylesheet link breaks it twice: the page is
  then not self-contained, and opening it tells a font host the reader's address and when they
  read it. The branded families stay first in each stack for anyone who has them installed
  locally, with a real system fallback behind them, and nothing is ever fetched (hi: VIEW-2). The
  `the_page_fetches_nothing` test holds the line by probing the rendered output for `http://`,
  `https://`, `//fonts.`, `<script`, `<iframe` and `@import`.
- **A newline between the id span and the sentence span.** `criteria_list` writes a literal newline
  between `</span>` and the next `<span>`. Flex layout ignores it, so nothing moves on screen, but
  without it a copy off the page, or any html-to-text conversion, yields `SEND-1I hit enter`. It
  is load-bearing whitespace; the comment in the source says so, and `copied_text_keeps_the_id_and_sentence_apart`
  also asserts that no two spans anywhere on the page are adjacent (hi: VIEW-1.d).
- **Brand tokens, not a new palette.** Colors, fonts and the iridescence rule come from the
  CorvidLabs Brand Kit, defined for light on bare `:root` and redefined for both dark states.
- **The page shows no status.** hi stores none (see `DECISIONS.md` §5). A progress indicator here
  would be inventing state the format deliberately does not carry (hi: VIEW-5).

## Files to Read First

- `src/view.rs` is the whole module: escaping, the inline renderer, section assembly, the template.
- `src/doc.rs` defines `Doc` and `Criterion`, whose `title`, `intent`, `criteria` and `retired`
  fields this module reads.
- `src/workspace.rs` provides `intent_path`, `criteria_count` and `rel`.
- The CorvidLabs brand tokens. Colors and type here were copied from the brand kit, not
  re-derived; keep them in step with it rather than inventing new values.
- `DECISIONS.md` §10.3 explains why this module exists at all, and §5 why it shows no status.

## Current Status

Implemented and in use: `hi view` writes `intent.html` at the repository root, and hi's own intent
renders through it. Eleven unit tests cover the inline renderer, the escaping guarantees, the link
allow-list, unmatched markers, comment stripping, the empty-prose block, the no-network guarantee
(`the_page_fetches_nothing`), copy fidelity (`copied_text_keeps_the_id_and_sentence_apart`),
section assembly, the retired `<details>`, and index stripping;
`tests/cli.rs::view_writes_a_self_contained_page` covers the command end to end.

Known limits, all deliberate: the inline subset is four constructs and will not grow into a
markdown implementation; there is no pagination, search or filtering; and the page is regenerated
wholesale rather than diffed, so it is a build artifact and is gitignored.

Known limits that are not deliberate, and are open in `tasks.md`: phone rendering (hi: VIEW-2.a)
has no automated coverage, and `strip_index` is not fence-aware even though hi: INDEX-2.a now says
in so many words that a marker inside a code block is not the real list.

## Notes

- `intent.html` is in `.gitignore`. It is generated, and committing it would put a large diff in
  every pull request that touches a criterion. If the page should be browsable without running the
  tool, publish it to GitHub Pages from CI rather than committing it.
- The page has no external reference of any kind. It once linked a Google Fonts stylesheet; that
  link is gone. `"Schibsted Grotesk"` and `"Spline Sans Mono"` still lead the two stacks, so a
  reader who has them installed sees them, and everyone else falls through to
  `ui-sans-serif`/`system-ui` and `ui-monospace`. Either way the page fetches nothing.
- `strip_index` removes the `hi:index` block, the H1 and `## Features` from `INTENT.md` prose. If
  the index markers in `out.rs` ever change, they must change here too: `out.rs` holds them as
  `INDEX_OPEN`/`INDEX_CLOSE` while `view.rs` still spells them as literals. Both sides compare a
  trimmed whole line, never a substring, so a marker quoted mid-sentence is left alone; keep that
  whole-line comparison if the constants are ever unified.
- That whole-line comparison is only half of hi: INDEX-2.a, which now reads *"If I quote the
  comments hi marks its list with, even on a line of their own or inside a code block, hi does not
  mistake my example for the real list."* `strip_index` works line by line with no fence tracking,
  so a fenced example in `INTENT.md` that shows a marker, an H1 or `## Features` on its own line is
  treated as the real thing: an H1 or `## Features` loses its line, and a fenced opening marker
  starts stripping there and swallows everything down to the next real closing marker. Verified
  against the binary. This is not the `doc` behavior: fences are opaque to `doc::parse_body` only.
  `out`'s `index_span` and `has_marker_line` are equally fence-blind, so the two sides still agree
  about where the block is, and they fall short of INDEX-2.a together. See `tasks.md`.
- `write` reads `INTENT.md` with a plain `fs::read_to_string` and does not strip a byte-order mark,
  though `Doc::parse` and `Workspace::find` both do. `str::trim` leaves U+FEFF in place, so a BOM
  keeps the H1 line from matching and it renders as prose. Nothing in the module depends on that;
  it is simply the one file read in the crate that does not strip a BOM.
- CRLF never reaches `paragraphs`, whose `split("\n\n")` would not find a blank line otherwise:
  `doc` builds `intent` from `raw.lines()` and joins with `"\n"` (its detected `newline` is used
  only when writing a file back), and `strip_index` normalizes the lead prose the same way.
- `write` uses a plain `fs::write`. `Doc::save`'s temp-file-and-rename dance exists because a hi
  file is a person's source of truth; `intent.html` is a regenerated artifact, and a torn write of
  it is fixed by running the command again. `doc::write_atomically` is `pub` now and `out::write_index`
  calls it, so availability is no longer the reason. If the page is ever published straight out of
  a working tree, reach for it here too.
- `write` creates no directory, and `workspace.root.join(out)` does not confine the output to the
  repository: an absolute `--out`, or one with `..`, is taken as given. The returned string comes
  from `Workspace::rel`, which falls back to the unmodified path when `strip_prefix` fails, so an
  absolute path is printed with a doubled leading slash, for example `//tmp/page.html`. It is
  cosmetic and it is what ships; the file lands where the caller asked.
- `doc` keeps fenced code blocks inside `## Intent` in the prose it hands over, since a fence is
  opaque to its parser rather than dropped. `view` has no block-level markdown, so such a block
  arrives here as ordinary paragraph text. If a real code block on the page is ever wanted, it is a
  new feature in `paragraphs`, not a parser change in `doc`.
