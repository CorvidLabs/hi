# Changelog

All notable changes to `hi` (Human Intent). Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The format itself is versioned separately by the `hi:` key in each file's frontmatter. `HI/1` is the
only version so far.

## [0.3.0] 2026-09-16

### The page is a rail and a document, and it wears the brand kit

`hi view` opened with a display-size title and three paragraphs before the
first criterion, and gave you nothing to navigate with. It is now two columns:
a sticky rail with the product's name, search, every feature with its count,
and the sort, retired and reset controls, beside the document itself.

- **The name comes from your `INTENT.md` heading**, which is the one place you
  actually named your product. It falls back to the directory name. The generic
  eyebrow above it is gone; your own prose is the first thing on the page.
- **Search highlights what matched**, wrapping hits in `<mark>` by walking text
  nodes, so a criterion's own `code` or link is never cut in half.
- **The keyboard reads the page**: `/` to search, `j` and `k` to move, `Enter`
  to copy a link, `Escape` to reset.
- **Clicking an id copies its link** and says so; the anchor still works if the
  clipboard is refused.
- **Prose yields to results.** The product why hides while anything is
  filtered; a feature's why hides while a search is running.
- **A sun/moon theme toggle**, copied from the design system rather than
  hand-rolled, remembering your choice and honouring `?theme=`.

**Fixed: filtering never hid anything.** `.row` sets `display: flex`, which
outranks the user agent's `[hidden] { display: none }`, so every filtered-out
criterion stayed on screen while the count said it had gone. Search and the
feature filter had both been in that state since 0.2.0.

The whole token block is now copied verbatim from CorvidLabs Brand Kit v1.3
rather than partly copied and partly invented. The kit's webfonts are the one
thing not taken: the page has to open with no network, so both brand faces are
named first in the stack and fall back to the system's own. DECISIONS.md
section 25.

### Roles are removed. A criterion is a plain sentence again

From 0.2.0 to 0.2.5 every criterion had to open with `As a <role>,`. That is
gone. hi reads no role, stores no role and prints no role, and the 108 criteria
in `hi/` are back to the sentences they were.

The prefix changed nothing. No check read it, nothing branched on it, and no
output was wrong without it: four verbs rendered it differently and that was the
whole feature. It cost every line four words in front of the only part anyone
reads, and on hi's own files 61 of 108 named the same role, so most lines paid
for zero information. Reading it back was a four-word heuristic over English
prose, wrong in both directions, which is the thing DECISIONS.md section 9
refused. And it shipped visibly broken on the HTML page for four releases
without anyone noticing, which is its own verdict on how load-bearing it was.

The finding that produced it stands: on a two-sided product an operator
criterion and a member criterion read as the same undifferentiated *I*. The
answer is to write the subject into the sentence, where English already puts it.
*An operator can cap what the bot spends in a day* says it and reads as writing.

Files written under the role rule still parse, check and render unchanged.
Those criteria are sentences that happen to start with *As a*, and hi now treats
them as exactly that: your words, passed through. `HI/1` is unchanged, because
it always described a sentence.

The *As a ___* test survives as advice, in the README and in `CLAUDE.md`: if you
cannot put it in front of your sentence, you wrote a fact rather than a want.
It is worth running in your head and not worth keeping in the file.

`hi/FILE-16` and `FILE-17` described the role rule and are retired with reasons
rather than edited. DECISIONS.md section 24 is the full reversal; sections 14,
16 and 19 carry a pointer to it. Two sections had been numbered 14 and two 15;
they are renumbered, so sections 16 through 23 have each moved up by two.

### The page can be searched, sorted, filtered and linked

`hi view` produced a static document. It coloured and nested, and that was all:
no way to search it, no way to narrow it, and no way to link to a criterion,
which is absurd for a format whose whole point is permanent, quotable ids.

It now ships:

- **Search** across ids and wording, with `/` to focus it and Escape to clear.
- **Filter** by feature, by clicking a chip.
- **Sort** by id or by family, or stay grouped by feature.
- **Deep links.** Every criterion is an anchor. Click an id to get
  `intent.html#SEND-1`, and that link lands on the criterion even when a filter
  would have hidden it, so a shared link never silently shows nothing.
- **Retired criteria** on a toggle, with their reason attached.

Still one self-contained file with no network access at all. The script is
inline, so the page works from an email attachment, and with scripting off
every criterion is still there while the controls stay hidden rather than
offering a search box that cannot search.

## [0.2.5] 2026-09-16

### The roles were jargon

hi's own criteria used "person writing intent" for 56 of them, which renders on
the page as a four-word chip reading PERSON WRITING INTENT. That is not a
person. "Intent" is this project's word for the thing, not a name anybody would
call themselves, and the README's own rule is to name the person rather than
describe them.

Now four roles: developer, reader, maintainer, agent. This changes only hi's
own criteria, not the format, which has always been free-form on purpose.

The lesson generalises. If a role does not fit comfortably in a chip, it is a
description rather than a name.

## [0.2.4] 2026-09-16

### The role label was never styled

The role chip shipped in 0.2.0 without its CSS rule, so every criterion on the
page rendered as `person writing intentI can write a thought down`: the label
as bare text, jammed into the sentence with no separator. Three releases of the
headline feature looked broken on the one surface built for people who do not
read markdown.

The edit that added the rule reported success and silently did nothing, because
the CSS lives inside a Rust `format!` where braces are doubled and the search
text did not match. The test asserted the span was emitted, which it was, and
not that it was styled. It now asserts both.

Found by looking at the rendered page, which no test does.

## [0.2.3] 2026-09-16

### Criteria are directions, which closes the open question in section 19

Section 19 logged a complaint two readers made independently: a reader cannot tell whether a
criterion is met, so on the question they most want answered the document sends them to the code.
It had no answer.

The answer is that it is the wrong question asked of the wrong layer. A criterion says what the
thing should be, not what it currently does, and directions stay correct through a wrong turn. A
criterion that is false right now means the code has not arrived yet, not that the criterion is
wrong. Whether the code has arrived belongs to something that reads code against contracts, which
`hi export` already feeds.

The page eyebrow now reads "What this should be" rather than "What we said we wanted", which was
past tense and read as a report of decisions.

Section 19's three properties survive as requirements on whoever builds the checking layer:
available to a non-author, usable across a whole set, and leaving something behind. They were never
a description of hi.

## [0.2.2] 2026-09-16

### hi wrote files that failed hi's own check

Reproducible in four commands. `hi retire` on a criterion with cases wrote the reason after the
cases, `hi check` looked for it directly under the parent, so the note landed on the last case and
the parent was reported as having no reason. Introduced while making the file look tidier.

The reason now sits directly under the criterion it explains, which is also what renders correctly
as a markdown list item with nested cases following it. `check` asks only the root of a retirement
for a reason, since a case went along with its parent and was never a separate decision.

### Retiring says what it took

A case can belong to a different concern than its parent, and taking it along silently is a decision
made on your behalf. `hi retire` names what went rather than counting it.

### Recorded, not changed

An agent with no code, no repository and no help read `hi/` and got the product, both roles, the
promises and the refusals right. It then derived the editorial standard from `## Retired`, read
seven retirements, extracted five distinct reasons, and used the rule to catch a live criterion as
unfalsifiable. `## Retired` was specified as an id reservation; it is the part of the format a
reader can learn a standard from. DECISIONS.md section 18.

## [0.2.1] 2026-09-16

### Retiring works the way people actually retire things

`hi retire GIFT-4` with no reason succeeded silently, and coming back later with a reason failed
with "not an active criterion", forcing a hand edit. The reporter's summary: it did not make them
write something perfunctory, it made them write nothing.

A reason can now be added after the fact, and `hi check` says how many retired criteria never say
why.

### The role prefix is the linter section 9 said could not exist

A discovery rather than a change. Two of forty criteria would not take a role, and both turned out
to be defective: the author could not write "As a ___" in front of them because they had written a
fact about the system rather than anyone's want. That test has no false positives because nothing
guesses. It is in the README beside the 59% figure.

### hi is upstream of whoever decided the shape

The README said to reach for hi before the code rather than after. Tested by writing a family before
its code existed, that was the wrong axis: a ticket written as a solution exerts the same pull, and
most tickets are written as solutions.

### Also

- Whether a criterion is built yet stays out of the tool; the README documents the prose convention.
- "1 criterion does not say who they speak for" disagreed in number at n=1.

## [0.2.0] 2026-09-16

Everything here comes from one field report: someone used v0.1.0 cold on a 33k-line Swift Discord
bot and wrote up where it let them down. The three biggest findings were all things hi could never
have found by dogfooding itself, because hi serves one audience and that project serves several.
[DECISIONS.md](DECISIONS.md) §14 and §15 record the decisions and the blind spot behind them.

### Criteria say whose voice they speak in

A criterion is now written as role-play: `- **SPEND-2**  As an operator, I can cap what the service
spends in a day.` On a product with several audiences, an operator criterion and a member criterion
used to render as the same undifferentiated "I", and a reader could not tell whether the person was
being paid or doing the paying.

There is no new syntax and no new field. The role is ordinary English at a fixed position, so hi
reads it back off the front of the sentence: `As a <role>,` or `As an <role>,`, where a role is a
short noun phrase of at most four words followed by a comma. `hi ls` prints it in brackets ahead of
the sentence, `hi export` emits it as its own `role` field beside the full text, `hi issue` opens
the ticket body with *Speaking as operator.*, and `hi view` carries it onto the page.

**Nothing enforces it.** There is no new `hi check` kind and no warning: a sentence with no role
parses, exports and renders exactly as written. Enforcing the opening of a sentence is the sentence
grammar DECISIONS.md §9 refused, and it is still refused.

`HI/1` is unchanged. A file written before this is still valid; it just cannot tell you who is
speaking.

hi's own 96 criteria were rewritten in this voice, across six roles.

### `hi retire <ID> [reason]`

`## Retired` existed in the format from the first release and no command put anything there, so the
only way to retire a criterion was to hand-edit markdown, in a tool whose pitch is that you do not
hand-edit. The reporter cut seven criteria by deleting lines and never found the section.

Retiring now takes one command. Cases go with their parent so nothing is orphaned, the reason is
optional and kept next to what it explains, and the id stays reserved forever.

### The product-level why is no longer undiscoverable

`INTENT.md` is created on the first capture rather than waiting for someone to run `hi index`, and
`hi check` keeps saying so while it still has no why written in it. It is in the README walkthrough
now too, which never named the file at all.

### Also

- `hi issue` no longer prints the sentence twice, and its ticket body nests cases correctly. The
  indent was on the wrong side of the bullet, so GitHub rendered them as a flat list, and at four
  spaces as a code block.
- A refused capture no longer leaves a half-made file behind. The new family's file was written
  before hi knew the criterion could be stored.
- A case can no longer be hung off a retired parent, where it used to nest under whatever criterion
  happened to sit last.
- `fledge hi` runs this plugin's own binary instead of whatever `hi` is first on `PATH`. Another
  project ships a coding agent by that name.
- `hi index` no longer rewrites a marker pair that sits inside a fenced code block. DECISIONS.md §11
  recorded that as a known open gap; it is closed, and the section now says so.
- Adopting a file hi did not create no longer reports it as `created`.

### Documentation

- The README teaches the role as rule 2 of five, and its example file is written in the new voice.
- **The 59% number is in the README.** Requirements-smell detection measures about 59% precision, so
  a prose linter would be wrong two times in five and people would learn to ignore it. That is why
  hi has none. It was buried in DECISIONS.md §9; it now sits in "What it deliberately does not do",
  where a first-time reader meets the question.
- The README says that `hi view` writes `intent.html` into the repository root and that it belongs
  in your `.gitignore`. Nothing told anyone that before.
- The first-run walkthrough names `INTENT.md`, which it never did.
- DECISIONS.md §7 no longer claims the fledge shim resolves `hi` through `PATH`. The shim stopped
  doing that, and §7 was describing the version that still did.
- DECISIONS.md §14 and §15 are new: the role-play decision, what the field report found, why hi's
  own dogfooding could never have found it, and the smaller holes the same report opened.

### Not changed, deliberately

The reporter withdrew their prose-linter complaint after reading the design record, and said the
format caught them writing implementation four times while they were typing. That is the intended
mechanism working without a checker.

Their last finding has no fix in the tool: hi is worth reaching for at the start of a feature, not
after. Writing intent for code that already exists means reverse-engineering the want from the
implementation. The README now says so rather than pretending otherwise.

## [0.1.0] 2026-09-16

First release. On crates.io as `human-intent`, installing a binary named `hi`, with archives for
x86_64 and aarch64 macOS, x86_64 Linux and x86_64 Windows. No Homebrew formula yet.

The format is not frozen. This is 0.x on purpose.

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
- Ships as the `human-intent` crate with a `hi` binary, plus a `bin/fledge-hi` shell shim and a
  root `plugin.toml`. The plugin is not bundled with fledge: run
  `fledge plugins install CorvidLabs/hi` once, which builds from source, and then the whole CLI is
  reachable as `fledge hi`.

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
