---
spec: view.spec.md
---

## User Stories

- As someone who decides what we build, I want to see what we agreed to without opening a markdown file full of identifiers, so that I can actually read it (hi: VIEW-1)
- As a reader, I want the why before the list, so that the criteria mean something when I reach them (hi: VIEW-1.a)
- As a reader, I want a case to sit visually under what it is a case of, so that the structure is obvious without decoding the id (hi: VIEW-1.b)
- As someone writing intent, I want the notes I leave myself in a comment to stay out of the page, so that I can think in the file without it showing up in front of the whole company (hi: VIEW-1.c)
- As someone sharing it, I want one file I can send to anybody, so that there is no build step or hosting question (hi: VIEW-2)
- As someone writing a criterion, I want to name a real symbol in `code` or emphasize a word, so that the sentence can be precise (hi: VIEW-3)
- As a maintainer, I want a sentence to be incapable of injecting markup, so that rendering someone's prose is never a security question (hi: VIEW-3.a)
- As a reader, I want the things we changed our minds about to be present but out of the way, so that the history is there without being noise (hi: VIEW-4)

## Acceptance Criteria

### REQ-view-001

The view module SHALL render the whole workspace as one HTML page carrying the product prose, each feature's intent, and every active criterion (hi: VIEW-1, VIEW-1.a).

Acceptance Criteria

- The product-level prose from `INTENT.md` renders above the first feature section.
- Each feature renders its title, its `## Intent` prose as paragraphs, and its criteria in order.
- Each criterion renders its id in a muted monospace span beside its sentence.
- A feature with no criteria renders `Nothing written down yet.` rather than an empty list.
- A feature whose prose is empty, or is empty once comments are stripped, renders no prose block at all.

### REQ-view-002

The view module SHALL indent a criterion by its id depth, capped at four levels (hi: VIEW-1.b).

Acceptance Criteria

- A depth-2 id renders with class `d2`, depth 3 with `d3`, depth 4 and deeper with `d4`.
- Indentation is reduced at phone width (16/32/48px), where id and sentence stack instead.

### REQ-view-003

The view module SHALL produce one self-contained file with no build step and no required network access (hi: VIEW-2).

Acceptance Criteria

- All CSS is inline in the document; there is no external stylesheet other than Google Fonts.
- There is no script element of any kind.
- The page is legible with the font stylesheet blocked, because every family declares a real fallback.
- Colors are defined for light on bare `:root` and redefined for both dark theme states.

### REQ-view-004

The view module SHALL render inline markdown in a criterion sentence: code, bold, italic, and links (hi: VIEW-3).

Acceptance Criteria

- `` `code` `` renders as `<code>code</code>`.
- `**bold**` renders as `<strong>bold</strong>` and `*italic*` as `<em>italic</em>`.
- `[text](https://example.com)` renders as an anchor to that URL.
- `**` is matched before `*`, so bold is never mis-parsed as italic.
- An unmatched marker renders as literal text; `2 * 3 is 6` is unchanged.

### REQ-view-005

The view module SHALL escape every input before interpreting any marker, so that no sentence can inject markup (hi: VIEW-3.a).

Acceptance Criteria

- `<script>alert(1)</script>` in a sentence renders as escaped text and produces no element.
- A double quote in prose renders as `&quot;` and cannot terminate an HTML attribute.
- A link URL that is not `http://`, `https://` or root-relative produces no anchor; `javascript:` renders as text.
- Titles, family names and file names are escaped on the same path as sentences.

### REQ-view-006

The view module SHALL present retired criteria in a collapsed section rather than omitting or inlining them (hi: VIEW-4).

Acceptance Criteria

- Retired criteria render inside `<details>` summarised as `<n> retired`.
- The section is absent entirely when a file has no retired criteria.
- Retired entries are visually de-emphasized relative to active ones.

### REQ-view-007

The view module SHALL read the workspace and never modify it.

Acceptance Criteria

- This module opens no `hi/*.md` file for writing; it reads them through `Workspace`.
- The only file written is the output page. That is `intent.html` at the repository root by default, or whatever `out` names, joined to that root and not otherwise constrained.
- A missing or unreadable `INTENT.md` omits the lead section rather than failing.
- The generated `hi:index` block, the H1 and the `## Features` heading are stripped from the lead prose so the page does not repeat its own navigation.
- The `hi:index` markers are recognised only as a whole trimmed line, matching how `out::index_span` and `out::has_marker_line` find them, so a marker mentioned inside a sentence does not start stripping.
- Neither `strip_index` nor `out`'s marker search is fence-aware, so a marker, an H1 or a `## Features` line standing alone inside a fenced example in `INTENT.md` is treated as the real thing. Fences are opaque only to `doc`'s parser.
- `INTENT.md` is read with a plain `fs::read_to_string` and no byte-order mark is stripped, unlike `Doc::parse` and workspace discovery.

### REQ-view-008

The view module SHALL remove HTML comment spans from prose before rendering it, and SHALL emit no block for prose that is empty afterwards (hi: VIEW-1.c).

Acceptance Criteria

- An HTML comment span anywhere in `## Intent` or `INTENT.md` prose is absent from the page, including the prompt comment in the new-file template hi writes itself.
- A comment that opens and is never closed drops the remainder of that prose rather than rendering as literal text.
- A feature whose prose is only a comment renders no `<div class="intent">`, empty or otherwise, and its criteria still render.
- A product prose that is only a comment renders no `<div class="lead">`.
- Comment stripping happens on prose only; a criterion sentence is escaped, so `<!--` inside one renders as visible text.

## Constraints

- hi stores no state, so the page shows no status, no progress bar, and no completion count. It renders intent.
- Rendering must stay dependency-free: no HTML templating crate, no markdown crate. The inline subset is small enough to own, and owning it is what makes the escaping guarantee auditable.
- The page must follow the CorvidLabs brand tokens rather than inventing a palette.
- The page must work at phone width with no horizontal scrolling.

## Out of Scope

- A full markdown implementation. Tables, headings, block quotes and images inside a criterion sentence are not supported and are not planned.
- Interactivity of any kind (filtering, search, editing, or collapsing anything but the retired section).
- Publishing or hosting the page. `hi view` writes a file; where it goes is the caller's business.
