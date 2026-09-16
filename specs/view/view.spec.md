---
module: view
version: 1
status: active
files:
  - src/view.rs
  - src/view/style.css
  - src/view/app.js
  - src/view/theme.js
  - src/view/prepaint.html
  - src/view/toggle.html

db_tables: []
depends_on:
  - specs/doc/doc.spec.md
  - specs/workspace/workspace.spec.md
---

# View

## Purpose

Renders the whole workspace as one self-contained HTML page for people who do not read markdown.
The raw `hi/*.md` files are written by whoever holds the intent; this is the artifact everyone else
reads: the product why first, each feature's prose, then its criteria as a plain list with ids
present but visually quiet, and retired criteria folded away (hi: VIEW-1, VIEW-1.a, VIEW-4). Each
criterion is its own list item and its id is a separate element from its sentence, with whitespace
between them, so text copied off the page keeps the two apart (hi: VIEW-1.d).

It also owns the only markdown rendering in the crate. A criterion sentence may carry inline
`` `code` ``, `**bold**`, `*italic*` and `[links](url)`; everything is HTML-escaped **before** any
marker is interpreted, so a sentence can never inject markup (hi: VIEW-3, VIEW-3.a).

Prose is the writer's scratch space, so HTML comment spans are removed before it is rendered. The
template hi writes into every new file carries a prompt comment, and neither it nor any other note
reaches the page. Prose that is empty once the comments are gone produces no block at all, rather
than an empty one (hi: VIEW-1.c).

The page is one file with no build step, no bundler and no network access whatsoever. Every byte of
CSS is inline, there is no `<link>`, `<script>`, `@import` or absolute URL anywhere in the document,
and the type is a system font stack, so opening the page discloses the reader to nobody
(hi: VIEW-2). It is a single centered column that reads at phone width (hi: VIEW-2.a). Colors come
from the CorvidLabs brand tokens and are defined for light and dark in all three theme states.

## Public API

| Export | Description |
|--------|-------------|
| `inline_markdown` | Escape a sentence and render the inline markdown it is allowed to carry. |
| `strip_comments` | Remove `<!-- ... -->` spans from prose, so hi's own starter prompt never surfaces as something a person wrote. |
| `strip_index` | Drop the generated index block, the `# ` title and the generated `## Features` heading from `INTENT.md` prose. |
| `render` | Build the complete HTML page from a workspace, optional product-level prose, and the product's own name. |
| `write` | Render and write the page, defaulting to `intent.html` at the repository root. |

### Structs & Enums

| Type | Description |
|------|-------------|
| None. | This module exports no types; it is three functions over `Workspace` and `Doc`. |

### Traits

| Trait | Description |
|-------|-------------|
| None. | This module defines no traits. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `inline_markdown` | `inline_markdown(raw: &str) -> String` | HTML-escape `raw`, then render `` `code` ``, `**bold**`, `*italic*` and `[text](url)`. Unmatched markers are left as literal text, and markers do not nest: the first one to open takes everything up to its close verbatim. Only `http://`, `https://` and root-relative URLs become links. |
| `render` | `render(workspace: &Workspace, product_intent: Option<&str>, name: Option<&str>) -> String` | Build the full document: head, tokens, the navigation rail, lead prose, one section per doc, footer with counts. `name` titles the page and falls back to the workspace directory name when it is `None`. |
| `write` | `write(workspace: &Workspace, out: Option<&str>) -> Result<String>` | Read `INTENT.md` for both the product's name (its `# ` heading) and the lead prose, strip its generated index, render, write to `out` or `intent.html`, and return the path as `Workspace::rel` prints it: repository-relative, forward slashes on every platform. |
| `escape` (private) | `escape(raw: &str) -> String` | Escape `&`, `<`, `>`, `"` and `'`. |
| `find_from` / `find_pair` (private) | `(&[char], usize, char) -> Option<usize>` / `(&[char], usize) -> Option<usize>` | Locate a closing single marker, or a closing `**`. |
| `strip_comments` (private) | `strip_comments(raw: &str) -> String` | Remove every HTML comment span, from `<!--` to `-->`. An unterminated `<!--` drops the remainder of the prose, as a browser would. |
| `strip_comments` | `strip_comments(raw: &str) -> String` | Remove every `<!-- ... -->` span, including one left unterminated, the way a browser would. Public because `out::export` and `out::issue` hand the same prose to an agent and to a tracker, and a starter prompt is not intent wherever it surfaces (hi: VIEW-1.c, EXPORT-5, ISSUE-6). |
| `strip_index` | `strip_index(raw: String) -> String` | Drop the `<!-- hi:index -->` block, any `# ` heading and a `## Features` heading, returning what a person actually wrote. `view::write` reads the heading separately for the page title before this runs (hi: VIEW-11, EXPORT-5). |
| `paragraphs` (private) | `paragraphs(raw: &str) -> String` | Strip comments, split what is left on blank lines, drop blocks that are now empty, join each block's lines with a space, and render it as one `<p>`. Returns an empty string when nothing survives. |
| `criteria_list` / `feature_section` (private) | `(&[Criterion]) -> String` / `(&Doc) -> String` | Render one list of criteria, and one feature's whole section. Each criterion is one `<li class="d<depth>">` holding a `cid` span and a `ctext` span separated by a newline (hi: VIEW-1.d). |
| `strip_index` (private) | `strip_index(raw: String) -> String` | Remove the `hi:index` block, the H1 and the `## Features` heading from `INTENT.md`. |

## Invariants

1. Every string that reaches the page passes through `escape` before any marker is interpreted, so
   no criterion sentence, title, family or file name can emit markup (hi: VIEW-3.a).
2. Because escaping runs first, `inline_markdown` operates on already-escaped text, and a `<` in
   the source is `&lt;` by the time markers are scanned. A marker can never reconstruct a tag.
3. A link is only emitted for a URL beginning `http://`, `https://` or `/`. Anything else,
   `javascript:` included, renders as literal text.
4. An unmatched marker is literal. `2 * 3 is 6` and ``a `b`` render unchanged.
5. `**` is tried before `*`, so bold never renders as an italic wrapping a stray asterisk.
6. Markers do not nest. The scan is single-pass and the first marker to open consumes everything up
   to its close without re-interpreting it, so `` **a `b` c** `` renders `<strong>a `b` c</strong>`
   with the backticks visible, and `` `x **y** z` `` renders `<code>x **y** z</code>` with the
   asterisks visible.
7. The page is self-contained and fetches nothing. All CSS is inline; the document contains no
   `<script>`, no `<link>`, no `<iframe>`, no `@import` and no `http://` or `https://` reference at
   all. The font stacks name a branded family first, but only as one the reader may already have
   installed locally; nothing is ever downloaded (hi: VIEW-2).
8. Colors are defined on bare `:root` for light, and redefined under both
   `@media (prefers-color-scheme: dark)` guarded by `:root:not([data-theme="light"])` and
   `:root[data-theme="dark"]`, so the page is correct in all three theme states.
9. Case depth is rendered as indentation, capped at four levels so a deep id cannot push text off
   a narrow screen (hi: VIEW-1.b).
10. Retired criteria are written into the page in their own section but hidden until the reader
    ticks "Show retired", present but not noise (hi: VIEW-4).
11. Prose passes through `strip_comments` before it is split into paragraphs, so no HTML comment
    span, including the prompt in hi's own new-file template, ever reaches the page. An
    unterminated `<!--` drops everything after it, the way a browser would (hi: VIEW-1.c).
12. A prose block that is empty once comments are stripped emits nothing: no `<div class="intent">`
    for a feature and no `<div class="lead">` for the product. The page never carries an empty
    wrapper element.
13. `view` never rewrites, reorders or reformats a `hi/*.md` file: it reads the workspace and writes
    exactly one file, the output page. That path is the caller's; `out` is joined to the
    repository root and is not otherwise constrained, so an absolute path or one with `..` writes
    outside it.
14. The page shows no status, because hi stores none. It renders intent, not progress (hi: VIEW-5).
15. A criterion's id and its sentence are two sibling spans with a newline between them, inside one
    `<li>` per criterion. No two spans anywhere on the page are adjacent with no whitespace, so
    copied text and any html-to-text conversion read `SEND-1 I hit enter`, never `SEND-1I hit
    enter`, and one criterion never runs into the next. Flex layout ignores that whitespace, so the
    rendered line is unchanged (hi: VIEW-1.d).
16. The page reads at phone width. Above 900px it is two columns, a 244px navigation rail beside
    the document; below that it is one column and the rail becomes a bar across the top. Headings
    are sized with `clamp()` so they shrink rather than overflow, and below 560px the id stacks
    above its sentence. Every edge-to-edge row cancels the page gutter with the `--gutter` token
    rather than a repeated literal, `html, body` clip horizontal overflow, and the feature list
    scrolls inside itself, so the page itself never scrolls sideways at any width. Nothing wraps an
    unbroken token longer than the column (hi: VIEW-2.a).
17. The navigation rail is the page's table of contents: the product's name, the search box, one
    entry per feature carrying that feature's criterion count, and the sort, retired and reset
    controls. It sticks while the document scrolls, and on a phone only the search box and the
    feature list stay stuck, because a header that eats a third of the viewport is not navigation
    (hi: VIEW-12, VIEW-12.a).
18. Exactly one entry in the rail is ever lit: the feature filtered to, or, when nothing is
    filtered, the one currently under the top of the viewport (hi: VIEW-12.b).
19. The page's title is the `# ` heading of `INTENT.md`, which is the one place a person named
    their product. It falls back to the workspace directory name, and nothing generic is printed
    above the author's own prose (hi: VIEW-11, VIEW-16).
20. The stylesheet neutralizes `[hidden]` with `display: none !important`. Author rules outrank the
    user agent, so without it `.row { display: flex }` keeps every filtered-out criterion on screen
    while the count claims it was hidden (hi: VIEW-6, VIEW-7).
21. A search match is wrapped in `<mark>` by walking text nodes only, never by re-parsing rendered
    HTML, so a criterion's own `<code>` or `<a>` is never cut in half (hi: VIEW-13, VIEW-3.a).
22. `hi export` and `hi issue` read prose through the same `strip_comments` this module uses for
    the page, and `read_product_intent` also runs `strip_index`. A day-one repository, where every
    `## Intent` still holds hi's question and `INTENT.md` still holds the generated `## Features`
    heading, therefore exports `product: null` and an empty per-file `intent`, and its tickets carry
    no intent section at all, rather than quoting the prompt back (hi: EXPORT-5, ISSUE-6).
23. The token block at the top of `view/style.css` is a verbatim copy of
    `_CorvidLabs/design-system/assets/tokens.css` (Brand Kit v1.3), as are `view/theme.js` and the
    sun/moon toggle's markup and pre-paint snippet. Nothing here re-derives a brand value. The one
    divergence is the webfonts: the kit loads Schibsted Grotesk and Spline Sans Mono from Google,
    and this page must reference no other server, so both are named first in the font stack with
    system fallbacks (hi: VIEW-17, VIEW-17.a).
24. The page honours `?theme=light|dark`, `data-theme` on `<html>` and `prefers-color-scheme`, in
    that order, and the toggle persists a choice to `localStorage` under `corvid-theme`. The
    pre-paint snippet runs before the stylesheet so a stored choice never flashes (hi: VIEW-18).
25. Prose yields to results: the product's lead prose is hidden while anything is filtered, and a
    feature's own prose is hidden while a search is running (hi: VIEW-6).

## Behavioral Examples

#### Scenario: A product person opens the page

- **Given** a workspace with `INTENT.md` prose and three feature files
- **When** `render(workspace, Some(prose))` runs
- **Then** the product prose appears above the first feature, each feature's `## Intent` appears
  above its criteria, and every id is rendered in a muted monospace span beside its sentence

#### Scenario: Inline markdown in a sentence

- **Given** `VIEW-3` in hi's own `hi/view.md`, whose sentence carries a `**bold**` span, an
  `*italic*` span, a backtick-quoted `code` span and a link
- **When** it is rendered
- **Then** the output carries `<strong>bold</strong>`, `<em>italic</em>` and `<code>code</code>`,
  and the surrounding text is unchanged

#### Scenario: A sentence that looks like markup

- **Given** a sentence containing `<script>alert(1)</script>`
- **When** it is rendered
- **Then** the output carries `&lt;script&gt;alert(1)&lt;/script&gt;` and no executable element

#### Scenario: A hostile link

- **Given** the sentence `[click](javascript:alert(1))`
- **When** it is rendered
- **Then** no `<a href` is emitted and the text renders literally

#### Scenario: A note left in the prose

- **Given** a feature whose `## Intent` holds nothing but a comment, as every file `hi` creates
  does until someone writes in it
- **When** the page is rendered
- **Then** the prompt text is absent and the section carries no intent block at all (not an empty
  one), while the feature's criteria still render

#### Scenario: Retired criteria

- **Given** a file with one active and one retired criterion
- **When** the page is rendered
- **Then** the active criterion is in the main list and the retired one is inside a
  `<details>` summarised as `1 retired`

#### Scenario: The page is pasted into an email

- **Given** a rendered page with several criteria
- **When** a reader selects the list and copies it into a plain-text field
- **Then** each criterion arrives on its own line with its id separated from its sentence, because
  every `</span>` is followed by a newline before the next `<span>`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `INTENT.md` is absent or unreadable | The lead section is omitted; the page still renders |
| `INTENT.md` prose is empty once the index, H1 and comments are removed | No `<div class="lead">` is emitted |
| A feature file has no `## Intent` prose | That section renders its criteria with no prose block |
| A feature's prose is only an HTML comment | Same as no prose: no intent block, criteria unaffected |
| A comment in prose is never closed | Everything from `<!--` to the end of that prose is dropped |
| `INTENT.md` has an opening `hi:index` marker with no close | `strip_index` drops the rest of the file from the lead prose. `view` only reads, so unlike `out::write_index` it has nothing to refuse |
| `INTENT.md` holds a fenced example containing a marker, an H1 or `## Features` on its own line | Treated as the real thing and stripped. An H1 or `## Features` loses only its own line; an opening `hi:index` marker starts stripping there, so everything down to the next real closing marker leaves the lead prose. `strip_index` is not fence-aware, and neither is `out`'s marker search; only `doc`'s parser treats a fence as opaque |
| `INTENT.md` begins with a UTF-8 BOM | It is not stripped here. `str::trim` does not remove U+FEFF, so the first line is not recognised as the H1 and renders as lead prose. `Doc::parse` and workspace discovery strip a BOM; this read does not |
| A feature file has no criteria | The section renders `Nothing written down yet.` |
| The output path cannot be written, for instance `--out nope/page.html` where `nope/` does not exist | `Err` with context `writing <absolute path>`, printed as `error: writing <path>: No such file or directory (os error 2)`; exit 1 from `main`. The module creates no directories |
| `--out` is absolute, or climbs out of the repository with `..` | `Path::join` takes it as given and the page is written there. The printed path comes back from `Workspace::rel`, which cannot relativize it: `--out ../outside.html` prints `../outside.html` and an absolute path prints with a doubled leading slash, for example `//tmp/page.html` |
| A criterion's id failed to parse | It renders at depth 1 with its raw id text, escaped |

## Dependencies

### Consumes

| Crate/Module | What is used |
|-------------|-------------|
| `std::fs` | Reading `INTENT.md`, writing the page |
| `anyhow` | `Result` and `Context` on the write |
| `crate::doc` | `Criterion`, `Doc`, and each doc's title, intent, criteria and retired lists |
| `crate::workspace` | `Workspace`, `criteria_count`, `intent_path`, `rel`, and the doc list |

### Consumed By

| Module | What is used |
|--------|-------------|
| `main` | `view::write`, for the `hi view` subcommand |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Claude | Reconciled with the bug-fix pass: documented `strip_comments`, the comment-free prose guarantee (hi: VIEW-1.c), and the omission of empty lead/intent blocks; added the matching invariants, scenario, error rows, REQ-view-008 and the two new tests. |
| 2026-09-16 | Claude | Verification pass: recorded that `strip_index` is not fence-aware and that `INTENT.md` is read without stripping a BOM; narrowed the read-only invariant to what the code enforces; corrected the phone-width indentation bullet and the dark-theme assertion note in `testing.md`. |
| 2026-09-16 | Claude | Drift pass against the shipped binary: the page no longer links Google Fonts and fetches nothing, so the self-containment claims in the purpose, invariant 7, REQ-view-003, the notes and the manual checks were rewritten; added invariants and requirements for copy fidelity (hi: VIEW-1.d) and phone reading (hi: VIEW-2.a) and cited hi: VIEW-5 on the no-status invariant; recorded the `--out` paths that escape the root; corrected the count and roster of unit tests; corrected the nested-marker claim, which said code wins inside bold when in fact markers do not nest at all; noted that `doc::write_atomically` is now public, so it is no longer the reason `write` uses a plain `fs::write`. |
