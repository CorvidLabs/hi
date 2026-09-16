# Changelog

All notable changes to `hi` (Human Intent). Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The format itself is versioned separately by the `hi:` key in each file's frontmatter. `HI/1` is the
only version so far.

## [Unreleased]

Nothing is published. There is no tag, the crate is not on crates.io, and there is no Homebrew
formula. Tagging `v*` is what triggers a release build, so the repository is deliberately untagged.

### The format (HI/1)

- A criterion is one markdown list item: `- **ID**  sentence`, indented two spaces per depth level.
  One criterion is always exactly one line, however long the sentence runs.
- Ids are hand-written and permanent: `FAMILY-n`, with levels alternating number, letter, number.
  A letter is another *case* of its parent; a number is a *step* or detail inside it. Nothing is
  ever renumbered, and a retired id stays reserved forever.
- A file carries `hi: 1` frontmatter, a `## Intent` prose block, `## Criteria`, and optionally
  `## Retired`. Frontmatter is read in inline or YAML block form, and written back in whichever
  style the file already uses.
- Criterion sentences may use inline markdown: `` `code` ``, `**bold**`, `*italic*`, and links.
- The parser accepts a bare line, any bullet, any emphasis and any indentation, so a hand-edited or
  pre-existing file is never rejected.

### The tool

- `hi <ID> <sentence>` captures, and is the default action. A new id just works; an existing id
  refuses and names the next free one. A new family starts its own file without asking.
- `hi check` does structural validation only. Exits 1 on a duplicate id, an orphan case, a
  retired-id collision, an unparseable id, an undeclared family, or a criterion stranded outside
  every section. Never fails because a criterion is unproven.
- `hi ls` reads what you have agreed to.
- `hi issue <ID> [--create]` prints a ticket, or opens a GitHub issue via `gh`.
- `hi export [FAMILY | file]` emits JSON for an agent, intent prose included.
- `hi index` rewrites only the generated block inside `INTENT.md`.
- `hi view [--out FILE]` renders one self-contained HTML page for people who do not read markdown.
- Ships as the `human-intent` crate with a `hi` binary, plus a `fledge-hi` shim and `plugin.toml`
  so the whole CLI is reachable as `fledge hi`.

### Deliberately not built

No stored state, no lifecycle, no evidence binding, no sentence grammar, no prose linter, and no CI
gate on unproven criteria. Each was considered and cut; the reasoning is in
[DECISIONS.md](DECISIONS.md) §5, §9 and §11.

### Fixed before first release

Found by dogfooding and by an adversarial bug hunt, all with regression tests:

- A failed write truncated the file. Saves are atomic now.
- `hi index` overwrote prose that merely mentioned its own marker.
- A fenced code block in `## Intent` was parsed as structure, creating phantom criteria and
  truncating the prose.
- Bare criterion lines were joined by markdown into one paragraph, so a file read as a wall of text
  everywhere it was rendered.
- A `hi/` directory was adopted by name alone. `hi` is the ISO code for Hindi, so a locale
  directory was being treated as a workspace.
- A zero-padded id parsed to a different spelling than it was written.
- CRLF files were converted to LF; a BOM hid the frontmatter; a non-UTF-8 argument panicked.
- Repo-relative paths used the host separator, producing `hi\chat.md` on Windows.
