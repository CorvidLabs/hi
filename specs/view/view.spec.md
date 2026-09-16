---
module: view
version: 1
status: active
files:
  - src/view.rs

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
present but visually quiet, and retired criteria folded away (hi: VIEW-1, VIEW-1.a, VIEW-4).

It also owns the only markdown rendering in the crate. A criterion sentence may carry inline
`` `code` ``, `**bold**`, `*italic*` and `[links](url)`; everything is HTML-escaped **before** any
marker is interpreted, so a sentence can never inject markup (hi: VIEW-3, VIEW-3.a).

Prose is the writer's scratch space, so HTML comment spans are removed before it is rendered. The
template hi writes into every new file carries a prompt comment, and neither it nor any other note
reaches the page. Prose that is empty once the comments are gone produces no block at all, rather
than an empty one (hi: VIEW-1.c).

The page is one file with no build step, no bundler, and no network dependency beyond an optional
Google Fonts stylesheet, and it renders correctly without it (hi: VIEW-2). Colors come from the
CorvidLabs brand tokens and are defined for light and dark in all three theme states.

## Public API

| Export | Description |
|--------|-------------|
| `inline_markdown` | Escape a sentence and render the inline markdown it is allowed to carry. |
| `render` | Build the complete HTML page from a workspace and optional product-level prose. |
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
| `inline_markdown` | `inline_markdown(raw: &str) -> String` | HTML-escape `raw`, then render `` `code` ``, `**bold**`, `*italic*` and `[text](url)`. Unmatched markers are left as literal text. Only `http://`, `https://` and root-relative URLs become links. |
| `render` | `render(workspace: &Workspace, product_intent: Option<&str>) -> String` | Build the full document: head, tokens, lead prose, one section per doc, footer with counts. |
| `write` | `write(workspace: &Workspace, out: Option<&str>) -> Result<String>` | Read `INTENT.md` for the lead prose, strip its generated index, render, write to `out` or `intent.html`, and return the repository-relative path. |
| `escape` (private) | `escape(raw: &str) -> String` | Escape `&`, `<`, `>`, `"` and `'`. |
| `find_from` / `find_pair` (private) | `(&[char], usize, char) -> Option<usize>` / `(&[char], usize) -> Option<usize>` | Locate a closing single marker, or a closing `**`. |
| `strip_comments` (private) | `strip_comments(raw: &str) -> String` | Remove every HTML comment span, from `<!--` to `-->`. An unterminated `<!--` drops the remainder of the prose, as a browser would. |
| `paragraphs` (private) | `paragraphs(raw: &str) -> String` | Strip comments, split what is left on blank lines, drop blocks that are now empty, join each block's lines with a space, and render it as one `<p>`. Returns an empty string when nothing survives. |
| `criteria_list` / `feature_section` (private) | `(&[Criterion]) -> String` / `(&Doc) -> String` | Render one list of criteria, and one feature's whole section. |
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
6. The page is self-contained: all CSS is inline and there is no script. The only external
   reference is a Google Fonts stylesheet, and the page is fully legible without it (hi: VIEW-2).
7. Colors are defined on bare `:root` for light, and redefined under both
   `@media (prefers-color-scheme: dark)` guarded by `:root:not([data-theme="light"])` and
   `:root[data-theme="dark"]`, so the page is correct in all three theme states.
8. Case depth is rendered as indentation, capped at four levels so a deep id cannot push text off
   a narrow screen (hi: VIEW-1.b).
9. Retired criteria appear inside a collapsed `<details>`, present but not noise (hi: VIEW-4).
10. Prose passes through `strip_comments` before it is split into paragraphs, so no HTML comment
    span, including the prompt in hi's own new-file template, ever reaches the page. An
    unterminated `<!--` drops everything after it, the way a browser would (hi: VIEW-1.c).
11. A prose block that is empty once comments are stripped emits nothing: no `<div class="intent">`
    for a feature and no `<div class="lead">` for the product. The page never carries an empty
    wrapper element.
12. `view` never rewrites, reorders or reformats a `hi/*.md` file: it reads the workspace and writes
    exactly one file, the output page. That path is the caller's; `out` is joined to the
    repository root and is not otherwise constrained.
13. The page shows no status, because hi stores none. It renders intent, not progress.

## Behavioral Examples

#### Scenario: A product person opens the page

- **Given** a workspace with `INTENT.md` prose and three feature files
- **When** `render(workspace, Some(prose))` runs
- **Then** the product prose appears above the first feature, each feature's `## Intent` appears
  above its criteria, and every id is rendered in a muted monospace span beside its sentence

#### Scenario: Inline markdown in a sentence

- **Given** the criterion `VIEW-3  A criterion can use **bold**, `code`, and links.`
- **When** it is rendered
- **Then** the output carries `<strong>bold</strong>` and `<code>code</code>`

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

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `INTENT.md` is absent or unreadable | The lead section is omitted; the page still renders |
| `INTENT.md` prose is empty once the index, H1 and comments are removed | No `<div class="lead">` is emitted |
| A feature file has no `## Intent` prose | That section renders its criteria with no prose block |
| A feature's prose is only an HTML comment | Same as no prose: no intent block, criteria unaffected |
| A comment in prose is never closed | Everything from `<!--` to the end of that prose is dropped |
| `INTENT.md` has an opening `hi:index` marker with no close | `strip_index` drops the rest of the file from the lead prose. `view` only reads, so unlike `out::write_index` it has nothing to refuse |
| `INTENT.md` holds a fenced example containing a marker, an H1 or `## Features` on its own line | Treated as the real thing and stripped. `strip_index` is not fence-aware, and neither is `out`'s marker search; only `doc`'s parser treats a fence as opaque |
| `INTENT.md` begins with a UTF-8 BOM | It is not stripped here. `str::trim` does not remove U+FEFF, so the first line is not recognised as the H1 and renders as lead prose. `Doc::parse` and workspace discovery strip a BOM; this read does not |
| A feature file has no criteria | The section renders `Nothing written down yet.` |
| The output path cannot be written | `Err` with context `writing <path>`; exit 1 from `main` |
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
