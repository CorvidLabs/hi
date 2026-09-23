# hi (Human Intent) design decisions

Every decision below was made by Leif in a structured interview on 2026-09-15, informed by a
24-agent research pass over the SDD/AC prior art and the CorvidLabs ecosystem. This file is the
record. Where a decision overrode a research recommendation, that is noted.

---

## 1. What hi is

**A shared way for people to write down human intent and acceptance criteria, in the mold of
spec-sync.** A convention plus a small tool, adopted because it is useful, not because it is
mandated. Not a numbered RFC series, not a conformance-vector suite.

hi holds **intent and identity**. Nothing else.

| hi is | hi is not |
|---|---|
| A place to write what a human actually wants | A spec |
| Permanent, human-chosen addresses for each criterion | A lifecycle |
| A generator of tickets and agent payloads | A test runner |
| A structural validator | A CI gate on intent |

**hi never:** stores state · tracks a lifecycle · binds evidence · runs a test · judges your English ·
fails a build because a criterion is unproven.

### The pipeline

```
talk → hi (intent + criteria) → tickets (gh) → specs (agent → spec-sync) → code
```

hi owns the first two boxes. Everything downstream is generated from them.

---

## 2. Layout

```
repo/
  INTENT.md          product-level prose + generated feature index
  hi/
    chat.md
    contacts.md
    billing.md
  specs/             spec-sync, generated from hi via an agent
  src/
```

`INTENT.md` at the root holds the holistic product why, in human words, plus an index of what is
in `hi/` that hi regenerates. The prose above the index is yours; hi never touches it.

---

## 3. The file

```markdown
---
hi: 1
families: [SEND, RECEIPT, OFFLINE, SPEND]
owner: leif
---

# Chat

## Intent

I want to talk to people I trust without anyone in the middle being able to read it, and without it feeling like a security product. It should feel like texting.

## Criteria

- **SEND-1**  As a member, I hit enter and the message shows up right away, marked as sending.
  - **SEND-1.a**  As a member, if I have no connection it queues and tells me, and never silently disappears.
  - **SEND-1.b**  As a member, if the thread was deleted before it sends, it warns me before discarding.
    - **SEND-1.b.1**  As a member, the draft is kept.
- **SEND-2**  As a member, it reaches them and the mark changes to sent.

- **RECEIPT-1**  As a member, I can tell the difference between sent and read without thinking about it.

- **OFFLINE-1**  As a member, I can read old threads with no connection.

- **SPEND-1**  As an operator, I can cap what the service spends in a day.

## Retired

- **SEND-3**  As a member, my messages auto-delete after 24 hours.
  retired: we decided this was a different product
```

### Sections

- **Frontmatter** declares `hi: 1` (format version), `families` (the ID families this file owns),
  `owner`. The `families` list is what lets capture resolve an ID to a file without scanning.
- **`## Intent`** is human prose. What this feature is for and what it should feel like. This is
  the thing a spec can never carry, and the first thing an agent should read. One paragraph is one
  line, however long it runs, and a blank line is the only break (§29).
- **`## Criteria`** holds the numbered lines.
- **`## Retired`** lists criteria we changed our minds about, keeping their IDs spoken for.

### Line grammar

```
<indent by depth>- **<ID>**  <sentence>
```

**One criterion is one markdown list item, on one line.** However long the sentence runs, it is
never wrapped, so criteria stay greppable, diffable, and readable as a list.

**A criterion is one plain sentence and nothing else.** No prefix, no fields, no slots. If a
sentence needs to say who it speaks for, it says so in English, the way any sentence does. §14
required an `As a <role>,` opening for four releases and §24 removed it.

The list item is not decoration. Markdown joins consecutive plain lines into a single paragraph, so
a file of bare `SEND-1  <sentence>` lines is one line per criterion in the source and an unreadable wall of
run-together sentences everywhere it is actually rendered: GitHub, any preview, any docs site.
That falsified `FILE-1`, the first thing the format promises. Cases are indented two spaces per
level, so the tree renders as a nested list too.

The id is bold so it reads as a label rather than as the first two words of the sentence.

The parser still accepts a bare line with no bullet, no emphasis, and indented continuation lines,
so a hand-edited file is never rejected. hi itself always writes the list form.

A sentence may use inline markdown (`` `code` ``, `**bold**`, `*italic*`, and `[links](url)`),
which `hi view` renders and everything else leaves as written.

No checkbox. No status. No `verify:`. No per-line rationale, because rationale lives in `## Intent`.

---

## 4. IDs

**Hand-written per criterion. Human-driven, product-intent, feature-related.**

```
SEND-1        a criterion              (number)
SEND-1.a        a case of it           (letter)
SEND-1.a.1        a step in that case  (number)
SEND-1.a.1.b        a case of that     (letter)
```

- **Families are uppercase**, hand-chosen, declared in frontmatter. One file may hold several
  (`SEND`, `RECEIPT`, `OFFLINE` all live in `hi/chat.md`).
- **Letters are cases. Numbers are steps.** A letter level is another branch or edge case of its
  parent; a number level is an ordered step or detail inside it.
- **Strict alternation by depth.** Number, letter, number, letter. `SEND-1.a.b` is rejected.
- **Permanent and append-first.** New criteria append; nothing is ever renumbered.
- **Never reused.** A retired ID stays reserved in `## Retired` forever.

> Research flagged the risk: hand-written IDs make permanence a discipline rather than a
> guarantee, and two branches appending to the same family can collide. Accepted deliberately.
> Speakability won. `hi check` catches collisions.

---

## 5. No state, no lifecycle, no evidence

Three related decisions, taken together:

- **State is derived, never written.** The text is pure intent and never mutates to reflect
  progress. Nothing in a file can lie about being done.
- **There is no lifecycle.** A criterion exists, or it is retired. No draft/agreed/met.
- **There is no evidence binding.** hi does not connect a criterion to a test, a commit, or a
  sign-off. Whether anything is proven is spec-sync's and your test runner's business.

> This overrides the research's strongest recommendation. Volere's fit criterion and a
> verification binding were the two highest-value borrowings available, and dropping them means
> **hi cannot detect an unverifiable criterion**, the measured failure mode in 90% of real
> AI-generated AC. Accepted deliberately: hi stays a human document, and rigour lives downstream.
> If this proves wrong, the cheapest reversal is an optional advisory `hi lint` that warns and
> never gates.

---

## 6. CLI

Rust, `clap 4` derive.

| Command | Behavior |
|---|---|
| `hi` | Print help. |
| `hi <ID> <sentence>` | Capture. The file is resolved from the ID's family via frontmatter. A new family starts its own file. Writes into `## Criteria`, appending a new criterion and putting a case directly under its parent. |
| `hi check` | Structural validation. Reports everything; exits 1 **only** on a structural error. `--json` emits the same report as JSON, families named. |
| `hi ls` | List criteria, grouped by file, optionally filtered to one family with `--family` and including retired ones with `--retired`. Each sentence is printed exactly as written. |
| `hi retire <ID> [reason]` | Move a criterion and every case under it into `## Retired`, creating the section if the file has none. The reason is optional and is written under the block it explains. The ID stays reserved forever. |
| `hi issue <ID>` | Print a ticket-shaped markdown block. `--create` shells out to `gh` to open a real issue, into `--repo owner/name` if you name one. |
| `hi export [FAMILY \| file]` | JSON payload for an agent. Takes a family, a file, or nothing (the whole repo, including `INTENT.md`). |
| `hi index` | Regenerate the feature index in `INTENT.md`. |
| `hi view [--out FILE]` | Write one self-contained HTML page of the intent, for people who do not read markdown. Search, filter, sort and a link for every id. |
| `hi seed` | Write `hi/AGENTS.md` when it is missing, or replace it when it is still a template hi has shipped. Refuses if the person has edited the file. Capture still only writes that file when it is absent (§39). |

### Capture

Capture resolves on the ID, and the rule is **a new ID just works; an existing ID refuses**.

| You type | hi does |
|---|---|
| A new ID in a known family | Appends to that family's file. |
| A new ID in an unknown family | Creates `hi/<family>.md` (family name, lowercased, underscores written as hyphens, so `SEND_2FA` becomes `hi/send-2fa.md`) with frontmatter, and drops it in. No prompt. |
| An ID that already exists | Refuses, and suggests the next free number. |

```console
$ hi SEND-2 "As a member, it reaches them and the mark changes to sent"
hi/chat.md  +SEND-2

$ hi SEND-2.a "As a member, if they blocked me it just never delivers"
hi/chat.md  +SEND-2.a

$ hi BILLING-1 "As a member, I can see exactly what I paid for"
hi/billing.md  created
hi/billing.md  +BILLING-1

$ hi SEND-2 "As a member, something else"
error: SEND-2 already exists in hi/chat.md:31
hint:  next free is SEND-3
exit 1
```

The very first capture in a repository also writes `INTENT.md`, and says so on its own line:

```console
$ hi CHECKOUT-1 "As a shopper, I can pay without making an account"
INTENT.md  created, for the product-level why
hi/checkout.md  created
hi/checkout.md  +CHECKOUT-1
```

That happens after the criterion is safely on disk, and it is best effort: if `INTENT.md` cannot be
written, the capture that already succeeded is not reported as a failure. §15 records why the file
exists from the first capture rather than waiting to be discovered (`INDEX-3`).

A mistyped family creates a stray file rather than an error. Accepted: it is visible as a new file
holding one criterion in `hi ls`, and as an extra name in the `families` list of
`hi check --json`, and fixing it is a rename. The plain `hi check` summary counts families without
naming them, so it is `hi ls` that shows you the typo.

### Check

**Structural errors** (exit 1), which are exactly the seven variants of `check::Kind`: a duplicate
ID; a case whose parent does not exist; an ID that collides with a retired one; a line shaped like
an ID that is not a valid one; a family a file never declared; a criterion stranded outside
every section, where nothing would read it; and a family two files both claim.

**Never an error**: a criterion with no downstream work, no spec, no test, no anything. Incomplete
intent is the normal state of intent.

```console
$ hi check
hi/chat.md
  15:orphan-case  SEND-1.a has no parent SEND-1
  31:duplicate-id  duplicate id SEND-2, already declared at hi/chat.md:14

18 criteria · 4 families · 3 files
2 problems
exit 1

$ hi check              # unfinished but well-formed
18 criteria · 4 families · 3 files
exit 0
```

Each problem line is `line:code  message`, so the code is greppable and the line number is the
first thing you read.

**One note, which is never a problem.** While `INTENT.md` has no product-level why in it, `hi check`
says so on a `note:` line and still exits 0:

```console
$ hi check
18 criteria · 4 families · 3 files
note: INTENT.md has no product-level why yet
exit 0
```

It is the one thing hi nags about, and it nags in the only way §5 allows: by saying it, not by
failing. The note disappears the moment there is prose in the file (`INDEX-3.a`).

### Retire

`## Retired` was in the format from the start and for a while nothing put anything there, so the
only way to retire a criterion was to hand-edit the markdown. `hi retire` closes that:

```console
$ hi retire SPEND-1 "the operator console is a separate product"
hi/chat.md  SPEND-1 retired

$ hi retire SEND-1
hi/chat.md  SEND-1 retired, with 2 of its cases
```

The criterion and every case under it move together, so nothing is orphaned behind them. The reason
is optional, and it is written on its own line after the whole block rather than after the parent
line, so a parent's cases stay attached to it. Retiring does not free the number: `SEND-4` is still
next after `SEND-3` retires (§8.3), capture refuses a retired ID, `hi issue` refuses to make work
out of one, and `hi check` reports any live criterion that reuses one.

### Generation

`hi issue` prints by default so it works with any tracker and no auth; `--create` opens a real
GitHub issue carrying `hi: SEND-1` as the permanent backlink.

`hi export` is the SDD handoff: it emits the `## Intent` prose plus every criterion and sub-case
as JSON, and an **agent** writes the spec-sync spec from it. hi does not write specs itself and has
no spec-sync coupling.

---

## 7. Stack and distribution

Following established CorvidLabs house style: fledge, spec-sync, attest, augur and rune are all
Rust single binaries.

- **Crate** `human-intent`. `hi` is taken on crates.io (v0.1.18, ~21k downloads) but is
  library-only with no binaries, so the `hi` **command** is free.
- **Binaries** one, `hi`, from **one repo**. `bin/fledge-hi` is a shell shim rather than a second
  compiled binary: it execs this plugin's own `target/release/hi` or `target/debug/hi`, and fails
  with a build hint if neither is there. It deliberately does **not** resolve `hi` through `PATH`,
  because another project ships a binary by that name that is a coding agent with shell access
  (§13). A root `plugin.toml` points fledge at it. Standalone-plus-plugin is the established norm;
  a separate `fledge-plugin-hi` repo is not.
- **Scaffold** `fledge templates init hi --template rust-cli --org CorvidLabs` (nine files,
  including CI and release workflows).
- **Install** `cargo install human-intent`, or a release archive, or
  `fledge plugins install CorvidLabs/hi` for `fledge hi`. The fledge plugin is not bundled with
  fledge and builds from source, so it needs cargo too. Homebrew shipped in 0.3.0:
  `brew install corvidlabs/tap/hi` works on macOS and Linux, on both architectures each since
  0.3.1 added `aarch64-unknown-linux-gnu`.
- Always written as **"hi (Human Intent)"** in anything searchable. Bare `hi` is unsearchable and
  collides with a universal shell greeting.

### Publishing

The full spec-sync treatment: **public repo, a README that teaches the format in 60 seconds, and a
docs site at `corvidlabs.xyz/hi`**, with the binary as the reference implementation. The repo and
the README shipped with v0.1.0, and the docs site shipped alongside 0.3.0: an overview, a
quickstart, the format, ids, a CLI reference and a page on adopting it. `corvidlabs.github.io/hi`
publishes this repository's own `hi view` output, so the demo is the artifact rather than a
mock-up of it. The format is documented prose.
There is no formal grammar and no conformance-vector suite, because the format is five rules and a
file layout.

---

## 8. Assumptions I made where the interview did not reach

These are defaults, not decisions, so flag any you disagree with.

1. Family names match `[A-Z][A-Z0-9_]*`; the number after the hyphen is a plain integer, not
   zero-padded.
2. Exit codes: `0` clean, `1` structural error, `2` bad usage.
3. Retiring a criterion does not free its number (the next `SEND` is still `SEND-4` after
   `SEND-3` retires).
4. Only two things exit non-zero on content: `hi check` on a structural error, and capture on an
   ID that already exists. No verb ever fails because intent is incomplete.
5. `hi export` emits one shape at every scope, so a consumer never branches on which scope was
   asked for. In practice this holds for everything inside `files`, which is identical at every
   scope; the whole-repo export adds one extra top-level key, `product`, carrying `INTENT.md`, and
   a family or file export omits it. Treat `product` as optional and nothing else changes.
6. A criterion's sentence is prose and hi never rewrites, reflows, or normalizes it.

---

## 9. What was deliberately not built

Recorded so it is not silently reinvented.

- **EARS / any sentence grammar.** 81% of real AI-generated "EARS" matched one pseudo-pattern that
  is not EARS, while 90% had no measurable anchor. Syntax conformance is free and worthless.
- **A prose linter.** Requirements-smell detection measures ~59% precision. A two-in-five
  false-alarm rate is how a tool gets disabled in week one. This is the single most-asked-for
  missing feature, so the number now lives in the README's "What it deliberately does not do"
  section as well, where a first-time reader meets the question. It was buried here, and a field
  report told us that was the wrong place for it.
- **Checkboxes.** A tick is a human assertion that nothing backs up.
- **An inbox.** Capture goes straight into the feature file, named now.
- **A CI gate on intent.** hi is installable on a Friday without turning anyone's build red.
- **Invisible ID tags stamped into existing files.** The research's final synthesis proposed this;
  it conflicts with human-authored, human-named files and was dropped.

---

## 10. Decided during the build

Three things settled while dogfooding, after the interview.

**10.1 One criterion is one line.** (Superseded in part by §12: still one line, now a list item.) The first renderer wrapped long sentences under the id. Seeing
it in a real file, it was clearly wrong: a wrapped criterion is harder to grep, produces a
two-line diff for a one-word change, and stops the file reading as a list. hi now never wraps.
Recorded as its own criterion, `FILE-6`.

**10.2 Sentences may carry inline markdown.** `` `code` ``, `**bold**`, `*italic*` and
`[links](url)`. This costs nothing, since the files were already markdown, and it lets a criterion name
a real symbol without it reading as prose. Everything is HTML-escaped before rendering, so a
sentence can never inject markup (`VIEW-3`, `VIEW-3.a`).

**10.3 `hi view` exists, because raw markdown is not a product artifact.** The people who decide
what gets built will not open `hi/chat.md` in an editor, and if that is the only way to see the
intent then the product side goes back to arguing from memory. `hi view` writes one self-contained
HTML page: prose first, criteria as a readable list, ids present but quiet, retired folded away.
No network, no build step, one file you can send to somebody (`VIEW-1` through `VIEW-4`).

This does not reopen §5. The page renders intent; it still shows no status, because there is none.

---

## 11. The hardening pass

A 45-agent adversarial bug hunt ran over the source: six finders on different failure dimensions,
each candidate then handed to a verifier whose instructions were to **refute** it by reproducing it
against the real binary. Sixteen findings survived; thirteen were refuted as cosmetic, deliberate,
or not reachable by a plausible user.

The two that mattered most were both data loss, and neither was visible from reading the code:

- **A failed write destroyed the file.** `fs::write` truncates before writing, so a full disk or a
  hit quota left the person's criteria gone. Reproduced at 15KB truncated to 4KB. Saves are now
  atomic: sibling temp, flush, fsync, rename (`FILE-8`).
- **`hi index` overwrote prose.** Markers were matched by substring, so a sentence that merely
  *mentioned* `<!-- hi:index -->` became the start of the generated block, and everything up to the
  real close marker was replaced. Markers now match whole lines, and an unclosed marker refuses
  rather than guessing (`INDEX-2.a`, `INDEX-2.b`). Whole-line matching was narrower than
  `INDEX-2.a` reads, and the remaining gap was recorded here as open: a marker pair written alone
  on its own lines inside a fenced code block in `INTENT.md` was still taken as the real block, so
  `hi index` wrote the list into your example and left the real block stale. **That is closed.**
  `index_span` and `has_marker_line` now track fences, so a marker inside one is an example and
  nothing else, and the real block below it is the one that gets rewritten.

Three findings were about hi's own promises being false in a corner:

- A fenced code block in `## Intent` was parsed as structure, so documenting the format inside your
  own intent created real criteria and truncated the prose. Any `# ` line in any fenced snippet did
  it. A `# no config` comment in a bash block was enough (`FILE-9`).
- CRLF files were silently rewritten to LF, contradicting the promise never to reformat what you
  wrote (`FILE-10`).
- A UTF-8 BOM hid the frontmatter from both the parser and workspace discovery, so a perfectly
  correct file looked broken (`FILE-11`).

And one was a name collision nobody would guess from the code. **`hi` is the ISO 639-1 code for
Hindi**, so `public/locales/hi/` is an ordinary directory in an ordinary web repository. Discovery
adopted it on name alone and wrote criteria into the locale tree. A `hi/` directory now qualifies
only if it holds a file with `hi:` frontmatter (`CAPTURE-6`).

Every fix carries a regression test, and every one was recorded as intent in `hi/`. That is the
argument for the tool making its own case: the bugs became criteria, and the criteria are what the
specs now cite.

### One boundary, decided rather than discovered

The stray-criterion check does **not** reach inside `## Intent`. An id-shaped line written there is
prose, and nothing reports it.

That is deliberate, and it is a genuine trade. Flagging it would catch someone who wrote their
criteria under the wrong heading. But `## Intent` is defined as human prose, and prose legitimately
starts a sentence with an id. *"SEND-3 was retired because it turned out to be a different
product"* is a sentence someone will write. A checker that refuses that sentence is a checker that
gets an exception added, and then gets ignored.

The mistake it would catch is also self-announcing, because a file whose criteria all sit under
`## Intent` reports `0 criteria` from `hi check` and shows nothing in `hi ls`. Silence is the
failure mode worth fearing, and there is none here.

If this proves wrong in practice, the narrow fix is to report an id-shaped line in `## Intent` only
when the file has no criteria at all, the case where it is unambiguously a mistake.

---

## 12. Correction: a criterion is a markdown list item

§10.1 made every criterion one line and stopped there. That was half the job, and the wrong half
was left undone.

Markdown joins consecutive plain lines into a single paragraph. So a block of

```
SEND-1  I hit enter and it shows up.
SEND-1.a  If I have no connection it queues.
```

is one line per criterion in the file. Everywhere a person actually looks at it (GitHub, an editor
preview, a docs site), it is a paragraph reading *"SEND-1 I hit enter and it shows up. SEND-1.a If
I have no connection it queues."* run together with the next eleven.

That directly falsified `FILE-1`: *"A hi file reads as an ordinary markdown document with no tool
installed."* It did not. It read as a wall of text, and the only reason every check passed is that
hi parses the file itself and never renders it.

Criteria are now markdown list items, indented two spaces per level so a case renders nested under
its parent. The parser still accepts a bare line, so no existing file is rejected.

The lesson worth keeping: hi's own tests all passed because they asserted on what hi parses. Nothing
asserted on what a person sees. `FILE-1.b` now does.

---

## 13. The command name is shared, deliberately

`hi` is a short word, so the command name is contested. This is the decision, made before launch
rather than discovered after it.

**The crate name is settled and costs us nothing.** `hi` on crates.io is taken by an unrelated
library (v0.1.18, about 21k downloads, no binaries in any of its 16 versions). We publish as
`human-intent` with `[[bin]] name = "hi"`, so `cargo install human-intent` puts `hi` on your path.
The registry collision is real and irrelevant.

**The command name is shared with at least five shipped things**, of which one matters:

`PipeNetwork/hi` is a Rust binary named `hi` that is a coding agent: it reads, writes and edits
files, runs shell commands, and talks to Pipe Network's inference service. Created June 2026, last
pushed two days before this was written, 4 stars, no LICENSE file, installed by a shell script
rather than from crates.io. Same language, same binary name, same broad field.

The others are `hi` on Hackage (a cabal scaffolder, last released 2018), `nikopol/hi` (shell host
info), `soveran/hi` (a stdin filter) and `longkeyy/hi` (a Go MQTT client).

**We keep the name.** The products are not confusable in use: theirs is an agent that needs an API
key and writes your code, ours is a markdown convention and a parser that makes no network
connection and runs no model. Neither of us is on the other's distribution channel. Nobody holds
the name in a way that excludes the other, and `hi` is what this project is.

What this costs the user is one thing, and the README says it plainly: if you already have a `hi`,
installing ours shadows it, and you choose which one wins on your path.

**What would change this decision:** PipeNetwork publishing a crate that installs a `hi` binary, or
either project landing in Homebrew core, where there is only one `hi`. Either would turn a shared
name into a contested one, and at that point the honest move is to ship `human-intent` as the
primary command and keep `hi` as the convenience.

---

## 14. Criteria are role-play

> **Reversed by §24.** The field report below is real and the lesson about dogfooding still
> stands. The mechanism it produced did not, and roles were removed in 0.3.0.

*(What follows is the decision as it was made and shipped; read it in the past tense.)*

**Every criterion is written in someone's voice, and the voice comes first.**

```
- **SEND-1**  As a member, I can send a message and see it arrive.
- **SPEND-2**  As an operator, I can cap what the bot spends in a day.
```

### What the field report found

Someone took hi cold to a 33k-line Swift Discord bot, a product with two sides: the people running
the bot and paying for it, and the people in the server using it. They wrote criteria for both, and
then could not read them back.

An operator criterion and a member criterion rendered as the same undifferentiated *I*. Nothing on
the line, in `hi ls`, in the ticket or on the page said which one was speaking. A reader could not
tell whether the person in the sentence was **being paid or doing the paying**, and on that product
that distinction was usually the entire reason two criteria disagreed. The format had nowhere to
put the one fact that made the list readable.

### Why hi's own dogfooding could never find it

hi serves one audience. Its own criteria are written by a person writing intent, for a person
writing intent, and read by a maintainer or an agent doing what that person asked. There is no
second party with money on the other side of the table, so there is never a sentence whose meaning
changes depending on who is saying it. Every criterion in `hi/` could have dropped its role and
nothing would have read worse.

That is the shape of the blind spot, and it is worth stating generally: **dogfooding finds the
failures your product has, and is silent about the ones your users have and you do not.** hi has
one audience and one voice. Most products have several, and several of them are in conflict. No
amount of using hi on hi would have surfaced this. It took one person using it on something else.

### The decision

Criteria are **always** role-play. Not optionally, not for multi-sided products, always. A format
where the role is optional is a format where half the file has one and the other half does not, and
then the absence means nothing.

**There is no new syntax.** The role is ordinary English at a fixed position: `As a <role>,` or
`As an <role>,` and then the sentence. Because the shape is universal, hi can read it back off the
front. `doc::role_of` takes the text between that opening and the first comma; `doc::without_role`
returns the remainder, so the two can be rendered apart. A role is a short noun phrase, at most four
words, followed by a comma.

`hi ls` prints it in brackets ahead of the sentence, `hi export` emits it as its own `role` field
beside the full text, `hi issue` opens the ticket body with *Speaking as operator.*, and `hi view`
carries it onto the page.

### What is deliberately not built

**Nothing enforces it.** There is no seventh `check::Kind`, and there is no warning. A criterion
with no role parses, exports, prints and renders exactly as it is written, and `hi check` stays
silent. Enforcing the opening of a sentence is the sentence grammar §9 refused, and it would be
refused for the same reason: syntax conformance is free to check and worth nothing, and a checker
people argue with is a checker people disable.

**The four-word cap is a heuristic, not a parser**, and it is wrong in both directions.
*"As a matter of fact, the queue is flushed nightly"* is read as a role named *matter of fact*.
*"As an operator responsible for the budget, I can cap spend"* is five words and so is not read as
a role at all: the whole sentence stays as written and `hi ls` shows no bracket. Both are accepted.
The cost of the first is one odd-looking bracket; the cost of the second is that a long role is
simply not surfaced, and shortening it to *operator* fixes it. The alternative is a vocabulary of
legal roles, which is a schema, and a schema is exactly the form that stops people writing things
down.

**There is no `roles:` list in frontmatter**, and no check that a role is one hi has seen before.
If a product later needs a closed set, that is the cheapest addition and it can be made without
touching any file, because the roles are already in the sentences. It is not built now because the
first version of every taxonomy is wrong, and a wrong taxonomy is harder to leave than no taxonomy.

**The format version does not change.** `HI/1` described a sentence, and this is a sentence. A file
written before this decision is still valid; it just cannot tell you who is speaking.

---

## 15. The rest of what the field report changed

The same report produced three more things, all of them holes that only a real product could show.

**`hi retire` exists now.** `## Retired` had been in the format from §3 onward, and for the whole of
0.1.0 no command put anything there. The only way to retire a criterion was to hand-edit the
markdown, in a tool whose entire pitch is that you do not hand-edit your criteria. The report cut
seven criteria by deleting the lines and never found the section at all: those seven sentences are
gone and the IDs they used are written down nowhere, which is precisely the failure §4 says is
worse than having no IDs. `hi retire <ID> [reason]` moves the criterion and its cases in one
command, creates the section when the file has none, and keeps the reason next to what it explains
(`RETIRE-1`, `RETIRE-2`).

**`INTENT.md` exists from the first capture, and `hi check` says so while it is empty.** It used to
appear only when somebody ran `hi index`, which meant it appeared only for people who had read far
enough to know the verb existed. The report never saw the file, so the product-level why for a
33k-line codebase was never written. Capture now creates it as soon as there is anything to have a
why about, and `hi check` carries a `note:` line until there is prose in it. The note never fails
and never affects the exit code, because §5 does not bend for this (`INDEX-3`, `INDEX-3.a`).

**`hi issue` printed the sentence twice and flattened the cases.** The criterion's sentence was the
ticket heading and then the first line of the body, which reads as a bug in the generator rather
than as a ticket. And the case indent was written after the bullet instead of before it, so
markdown rendered nested cases as a flat list of peers, or as a code block once there were four
spaces of it. Both fixed (`ISSUE-3.a`).

The lesson is the one in §14 and it is worth the repetition: everything in these two sections came
from one person using hi on a product that was not hi. Dogfooding and a 45-agent bug hunt (§11) had
already been over this code, and both were looking at the parts hi exercises on itself. A gap in the
format, a verb the file format implied and no command provided, and a file nobody could discover
are not bugs in that sense. They are things you only see from outside.

---

## 16. The role prefix turned out to be the linter

> **Reversed by §24.** The test itself survived; the format requirement that carried it did not.
> "Can I put *As a ___* in front of this?" is now writing advice a person applies while typing,
> not a shape the file has to hold.

Section 9 cut prose linting, and the reasoning still holds: requirements-smell detection measures
about 59% precision, so a checker would be wrong two times in five and people would learn to ignore
it.

Then a field report from someone using hi on a product with three audiences found this, which we
did not design and did not expect:

> 38 of 40 took a role as a pure prefix. Two could not, and both turned out to be defective in a way
> I had not noticed: I could not complete "As a ___" in front of them because I had written a fact
> about the system rather than anyone's want.

Requiring the role is an ambiguity detector, and it does not have the 59% problem, because it is not
a heuristic. Nothing guesses. The author either can finish "As a ___," in front of their sentence or
cannot, and being unable to finish it means what they wrote was not a want. There are no false
positives available to a test the author performs on their own sentence.

So hi has the thing section 9 said could not be built. It arrived as a side effect of asking whose
voice a criterion speaks in, which was a different problem entirely. Section 9 is not reversed: we
still ship no dictionary, no POS tagger and no smell rules. This is the whole of the mechanism, and
it costs three words at the front of a sentence.

`hi check` counted criteria that did not name a role. It never failed on them, because a criterion
without a role was unfinished rather than wrong, and section 5 stood. That count is gone with the
rest of the mechanism.

---

## 17. What hi is upstream of

The README used to say hi is worth reaching for at the start of a feature rather than after, on the
grounds that writing intent for existing code means reverse-engineering the want from the
implementation.

The same reporter tested that directly by writing a family before any of its code existed, and
corrected us:

> The pull does not disappear, it changes source. Issue #109 is written as a solution, and reading
> it I felt exactly the same pull, from a ticket instead of a file. So hi's value is not "before vs
> after code", it is "upstream of whoever already decided the shape". Most tickets are written as
> solutions.

That is a better description of the tool and it is now the one the README gives. Code is one thing
that decides the shape before you get there. A ticket written as a solution is another, and it is
far more common.

Two things were genuinely better before the code existed, and both are about absence rather than
translation. You can write a criterion you have no idea how to implement, which an implementation
never suggests. And four of that reporter's eleven criteria were things their own ticket did not
propose at all: writing from something that exists makes what is missing invisible.

---

## 18. Whether a criterion is built yet

hi does not record it, and will not.

A reader cannot currently tell a shipped criterion from a wish, and on a page titled "What we said
we wanted" that is a real misreading. The argument for recording it is good: unlike a lifecycle, it
flips once rather than moving through states, so it is one bit rather than a workflow.

We are still not doing it, for the reason section 5 gives. Nothing in hi knows when a feature ships,
so the bit would be set by hand and would go stale, and a stale "not built yet" sitting on shipped
behavior is worse than the silence it replaced.

The convention instead, which is what the reporter did before asking:

```markdown
## Intent

Written before any of it was built, which is the point. None of this exists yet.
```

The prose block is the first thing a reader meets, it cannot go stale without somebody reading the
sentence that is now wrong, and it costs one line. The README says so, so the next person does not
have to think of it themselves.

---

## 19. `hi ls` capitalized after the role

> **Obsolete as of §24.** With roles gone there is nothing to strip, and `hi ls` prints every
> sentence byte for byte as the file has it. Kept because the reasoning below is why it was safe.

`hi ls` used to print the sentence with its role stripped and the remainder capitalized, so
`As a person writing intent, hi finds my workspace` listed as `[person writing intent] Hi finds my
workspace`.

That was display only. The file is never rewritten, and section 8.6 still holds: hi does not touch
the words you wrote. It is recorded here because a reader of the list went and checked the file
against 8.6 before concluding it was safe, which is a minute nobody should have to spend.

---

## 20. Retired is how the format teaches

An agent with no access to the code, the repository or any help was handed `hi/` alone and asked
what the product was. It got the product, both roles, the promises and the refusals right, which is
the claim `hi view` exists to make and which nobody had ever tested, because until then the only
readers had also been the author.

Then it did something nobody designed. It read the seven entries in `## Retired`, extracted five
distinct reasons for retiring a criterion, stated the rule those reasons implied, and applied the
rule to the live criteria, catching one as unfalsifiable and citing two retirements as precedent.

It learned the author's editorial standard from the document and turned it on the author.

`## Retired` was specified as an id reservation: a place to keep a number spoken for so it is never
handed out twice. It turns out to be the only part of a hi file that records judgement rather than
intent, which makes it the part a reader can learn a standard from. That is worth knowing before
anyone proposes making it terser, moving it to a separate file, or dropping the reason.

This is also the strongest argument yet for `hi check` nagging about a retirement with no reason.
A reasonless retirement is not just an undocumented decision, it is a lesson the next reader cannot
learn.

---

## 21. Open: a reader cannot tell met from unmet

Recorded as an open question rather than a decision, because the argument against section 5 got
sharper and has not been answered.

Section 5 cut evidence binding on the grounds that hi is about what was wanted, and that a criterion
with no proof is a normal state rather than a defect. Section 18 declined a built-yet bit because
nothing in hi knows when a feature ships, so the bit would go stale.

The field report that tested criteria for rot found none, and then said something we had not
considered:

> Two criteria were born false, which is worse, and nothing caught either. The cost of no mechanism
> is not drift, it is that a reader cannot tell met from unmet and has to read the code, which is
> the thing the document exists to spare them.

The stranger agent hit the same wall independently.

That is a different argument from the one section 5 answers. It is not about staleness and not about
gating. It is that the document's stated purpose is to save a reader from reading the code, and on
the question a reader most wants answered it sends them to the code anyway.

We have no answer we are happy with. An evidence binding reintroduces everything section 5 cut. A
status field reintroduces everything section 18 cut. Doing nothing leaves the complaint standing,
and it has now been made independently by two readers.

It is written down here so the next person to propose evidence binding is arguing with this rather
than with a straw version of section 5.

### What people actually do instead, observed

Asked what they did instead of using hi when they wanted to know whether a criterion held, the same
reporter answered with behavior rather than a proposal. Twice in one stretch of work they ran
`git show origin/main:Sources/...` and read the function with their eyes. Not a test, not a search,
not a document: straight to the implementation, one criterion at a time.

Three properties of that workaround matter more than any feature:

**It is available only to the author.** They could check one criterion because they knew the guard
lived in a particular file. Someone who did not write the code cannot perform this at all, which is
exactly why the stranger agent said plainly that it could not tell met from unmet. The workaround is
unavailable to the audience the document exists for.

**It does not scale and was not attempted at scale.** Two criteria out of fifty were checked, each
because of a specific suspicion. There was no pass over the whole set and there would not have been.
Reading fifty call sites is not something anyone does twice.

**It leaves no residue.** Two criteria turned out to be false. That knowledge lived in one head and
then in a commit message. Nothing in `hi/` is different for the checking having happened, so the
next reader starts from zero, and so does the same person a month later.

So the problem is not that hi cannot prove a criterion is met. It is that verification today is
author-only, small-batch, and evaporates.

Any mechanism anyone proposes should be measured against those three rather than against whether it
can bind a test to an id. In the reporter's words, which are sharper than ours:

> A thing that lets a non-author ask the question at all would beat a thing that proves it
> rigorously for the author, because the author already has the workaround and the reader has
> nothing.

That reframes the design space. The interesting target is not proof, it is giving the reader a way
to ask.

---

## 22. How to measure whether hi gets used

Section 21's companion, and a correction to our own method.

We asked an agent whether it would reach for hi unprompted, and said that if six months passed
without it happening, that would be the answer. The agent corrected the question:

> That assumes a continuity I do not have. I am a session, not a person with a Tuesday. So my zero
> is not the same measurement as a human's zero and should not be counted as one.

That is right, and the mistake is worth keeping because it is easy to repeat. Habit is a property of
something that persists between occasions. An agent that starts fresh each time can leave an
artifact where the next one will find it, and cannot form a habit at all. Asking it to is asking for
a measurement it is not able to produce, and counting its zero as a human zero would have put a
false reading into this file.

What a session can do, and did: `hi/` is committed, the unbuilt work it describes is still unbuilt,
and `hi/see.md` is the specification for an open issue. If a later session reaches for it there
without being told, that is the data point. Whether hi is reached for unprompted is still the
question that decides if any of this matters, and it has to be measured on someone who has a
Tuesday.

---

## 23. Criteria are directions, which answers section 21

Section 21 logged an unanswered complaint: a reader cannot tell whether a criterion is met, so on
the question they most want answered the document sends them to the code. Two readers made it
independently and we had no reply.

The reply is that the question belongs to a different layer, and the owner put it better than we
had:

> Directions of one driving a car. Even if you take a wrong turn, you still have the correct
> direction. So the AC is our directions.

A criterion that is currently false is not a defective criterion. It means the code has not arrived
yet, or has gone the wrong way. The directions are unchanged either way, and rewriting them because
you took a wrong turn is exactly the mistake. A criterion can describe something true today,
something a year out, or something that was true and has since been revised. In every case it says
what the thing should be, which is the only claim hi ever makes.

That resolves section 21 without adding a mechanism. "Is this met" is a question about where the car
currently is. hi holds where it is going. Answering the first one is the job of something that reads
code against contracts, which is what spec-sync and project-specific checks are for, and hi already
feeds them through `hi export`.

So the layering is:

| Layer | Question it answers | Owned by |
|---|---|---|
| hi | Where are we going | A person, in their own words |
| Specs and code checks | Where is the car now | Tools that read the code |
| The code | How we get there | Whoever is driving |

Two things follow.

The value hi is actually claiming is narrower and more defensible than "you will not have to read
the code". It is that a person can read what the thing should be, and an agent can read the same
sentence and get the same answer. Neither of them has to reverse-engineer intent from an
implementation, and neither is told they are looking at a status report.

And section 21's three properties still matter, just not as a job for hi. When someone does build
the checking layer, it should be measured against them: available to a non-author, usable across a
whole set rather than two at a time, and leaving something behind. Those describe a checker worth
having. They were never a description of hi.

The direction of travel is that the checking gets built alongside, not inside. hi stays the
directions.
---

## 24. The role prefix is removed

**A criterion is a plain sentence again.** Section 14 made every criterion open with
`As a <role>,` and section 16 argued the prefix was earning its keep as a linter. Both are
reversed. As of 0.3.0 hi reads no role, stores no role and prints no role, and the 108 criteria in
`hi/` are back to the sentences they were before.

```
- **SEND-1**  I can send a message and see it arrive.
- **SPEND-2**  An operator can cap what the bot spends in a day.
```

### Why

**It changed nothing.** No check kind read it, nothing branched on it, nothing sorted by it that a
reader used, and no output was wrong without it. Four verbs rendered it differently and that was
the entire feature: a variation in how hi printed a string it already had. A format rule that costs
every line four words and buys no behavior is not a rule, it is a habit the file has to carry.

**It cost the sentence.** The prefix sits in front of the only part anyone reads. On hi's own
files 61 of 108 criteria named the same role, so on most lines the first four words carried no
information at all and pushed the content to the right. The sentence is the criterion. Anything
standing in front of it had better be worth more than the words it displaces.

**Reading it back was a guess.** Section 14 admitted this: the four-word cap is wrong in both
directions, reading *a matter of fact* as a role and refusing *an operator responsible for the
budget* as one. That is a heuristic parsing English prose, which is the thing section 9 refused,
arriving through the back door.

**It shipped broken for four releases and nobody could tell.** From 0.2.0 to 0.2.3 `hi view`
rendered the role jammed into the front of the sentence with no separator
(`person writing intentI can write...`). It went out four times, and it was found by taking a
screenshot rather than by anyone reading the page. A field whose corruption is invisible in the
source and survives four releases was not load-bearing.

### What the field report actually found, and the better answer

The finding in section 14 is real: on a two-sided product an operator criterion and a member
criterion read as the same undifferentiated *I*, and that distinction was usually the whole reason
two criteria disagreed. The mistake was answering it with a slot in the format.

English already does this. *An operator can cap what the bot spends in a day* names its subject and
reads as a sentence. *As an operator, I can cap what the bot spends in a day* names the same subject
and reads as a form someone filled in. When a criterion needs to say who it speaks for, it says so
the way writing says things. When it does not, it stops paying for the ceremony.

That also removes the *I* problem at the root rather than labelling it. The first-person voice was
what made the two criteria collide; a prefix in front of *I* leaves the *I* there.

### What survives

**The smell test, as advice.** If you cannot put *As a ___* in front of your sentence, you probably
wrote a fact about the system rather than something somebody wants. That is worth knowing while you
type, and it is in the README as guidance. It is not a rule the format holds, not something
`hi check` counts, and not a shape the file has to carry in order to give you the benefit.

**Section 14's real lesson**, which was never about roles: dogfooding finds the failures your
product has and is silent about the ones your users have and you do not. That stands, and it is why
the field report was worth acting on at all. The correction here is about the mechanism we chose,
not about listening.

### What is deliberately not built

**No `roles:` frontmatter, no `speaker:` field, no replacement.** The problem does not need a place
in the format, so it does not get one. Adding a field is how the last version of this went wrong.

**The format version does not change.** `HI/1` described a sentence, and this is a sentence. Every
file written under the role rule still parses, still checks and still renders. Those criteria are
sentences that happen to begin with *As a*, which is legal English and always was, and hi now treats
them as exactly that: words the author wrote, passed through untouched (§8.6).

**Retired criteria keep their words.** `FILE-16` and `FILE-17` described the role rule, and they are
in `## Retired` with the reason, not edited to pretend they said something else. That is section 20
working as intended: the retirements are where a reader learns the format changed its mind.

**What would change this decision:** a product where the subject genuinely cannot be written into
the sentence, and where a reader of the list needs to group by speaker across hundreds of criteria.
Neither has been seen. If it is, the cheapest version is a filter over words already in the
sentences, not a field in front of them.

---

## 25. The page is a rail and a document, and it wears the brand kit

`hi view` used to open with a 52px wall: an eyebrow reading *What this should be*, the product's
name at display size, and three paragraphs of prose. You scrolled past half a screen of chrome
before reaching a criterion, and there was nothing to navigate with when you got there. The
complaint was exact: the big header is useless, and it should be specific to the project.

### What changed

**The page is two columns.** A 244px rail holds the product's name, the search box, one entry per
feature with that feature's count, and the sort, retired and reset controls. It sticks while the
document scrolls, so navigation is never something you scroll back up to find. Below 900px it
becomes a bar across the top, and only the search box and the feature list stay stuck there,
because a header that eats a third of a phone screen is not navigation.

**The name comes from `INTENT.md`.** The `# ` heading is the one place a person actually named
their product, and `strip_index` was already throwing it away to keep it out of the prose. The page
now reads it for the title and falls back to the directory name. Nothing generic is printed above
the author's own words: the eyebrow is gone, and the lead prose is the first thing on the page.

**Prose yields to results.** The product's why disappears as soon as anything is filtered, and a
feature's own why disappears while a search is running. Both come back when you reset. Intent prose
earns the top of the page you opened; it has not earned the top of the answer you went looking for.

**Search highlights what matched**, by walking text nodes and wrapping hits in `<mark>`, never by
re-parsing rendered HTML, so a criterion's own `` `code` `` or link is never cut in half. **The
keyboard reads the page**: `/` to search, `j` and `k` to move a cursor through visible criteria,
`Enter` to copy a link, `Escape` to reset. **Clicking an id copies its link** and says so, with the
anchor still working when the clipboard is refused.

### The bug this uncovered

Filtering never hid anything. `.row` sets `display: flex`, and an author rule outranks the user
agent's `[hidden] { display: none }` no matter how specific it is, so every filtered-out criterion
stayed on screen while the count underneath claimed it had gone. Search and the feature filter had
both been shipped in that state since 0.2.0. It was found by driving the real page in a browser and
looking at it, which is the same lesson as section 12: the tests all asserted on the HTML that went
in, and nobody asked the browser what came out. The fix is one rule,
`[hidden] { display: none !important; }`, and there is now a test that the stylesheet carries it.

### The brand kit

The colours were already copied from CorvidLabs Brand Kit v1.3, but only a handful of them, and the
rest of the page had invented its own greys next to them. The whole token block is now copied
verbatim out of `design-system/assets/tokens.css`, which is what that file asks for: *import this
file, don't re-derive it*. Inputs sit on `--surface-strong` rather than `--surface`, because the kit
says surfaces lift and wells recess. Prose wraps at `--measure`. The sun/moon toggle, its pre-paint
snippet and `theme.js` are copied from the kit rather than hand-rolled, which `ADOPTION.md` asks for
by name, and the page honours `?theme=`, `data-theme` and `prefers-color-scheme` exactly as every
other CorvidLabs surface does.

**One deliberate divergence.** The kit loads Schibsted Grotesk and Spline Sans Mono from Google
Fonts. This page cannot: `VIEW-2` says it is one file you can attach to an email, and `hi check`'s
own test refuses any reference to another server. So both faces are named first in the stack and
fall back to the system's own. On a CorvidLabs machine the page is in brand type; anywhere else it
is in a sensible sans, and it still opens on a plane. A page that needs the network is not a page
you can send to someone.

**What would change this decision:** the kit shipping the two faces as files we can embed as data
URIs at a size worth paying for. The page is already around 200KB; two subsetted woff2 faces would
roughly double it, which is a trade worth making only if someone asks for it.

---

## 26. The one promise, and the four ways hi broke it

**hi never reuses an id, and never lets one of its own verbs reuse one.** Everything else in hi is
a preference. This is the claim the format rests on, because the whole point of an id is that it
can be quoted somewhere hi will never see: a ticket, a spec, a commit, a conversation. An id that
can be reassigned is worse than no id, because the quotation silently starts pointing at something
else.

*Worded that way deliberately, and it was not always.* This section opened with "an id is permanent
and never reused", flat, as a property of the world. It is not one. The files are markdown and they
are yours: nothing stops you renumbering one in an editor, and nothing stops two branches choosing
the same id and git merging both without a word (§37). README rule 4 has said the honest version
since it was written — *permanence is a convention the tool supports rather than one it enforces;
what it can do is refuse to be the one that breaks it* — and §30 was rewritten for exactly this
reason after claiming more than it could keep. The scope of the promise is hi's own verbs, and that
scope is what the rest of this section is about. Reviewers of a 1.0 candidate read the absolute
wording and were right to: a promise stated wider than it can be kept is the same failure as a
postcondition that compares the wrong thing (§35), one document up.

What the narrower wording costs is nothing at all, because the wider one was never doing any work.
Every fix below is a fix to something hi's own verbs did. What it buys is that the sentence a 1.0
would freeze is one hi can actually keep.

A thirteen-agent audit went looking for what a 1.0 would freeze and found that hi's own verbs broke
that promise four ways. Three were silent: `hi check` reported no problem.

1. **`retired_end` was a `match` with two byte-identical arms**, so retiring always appended at end
   of file. Where `## Retired` was not the last section the criterion landed under whatever
   followed it, outside every section hi reads, while the command printed "retired". The id was
   then free, and capture handed it out again.
2. **Retire reasons were not collapsed to one line**, though criterion sentences always were. A
   reason carrying a newline and a criterion-shaped line wrote a second real criterion, burning an
   id nobody had written.
3. **A fence under `## Criteria` hid a criterion from all six checks.** Fences are opaque so a
   person can document the format inside their own prose (§3, `FILE-9`), and that is right for
   `## Intent` and wrong for a criteria section.
4. **Concurrent captures overwrote each other.** `write_atomically` makes one write atomic and says
   nothing about two processes reading the same original. Eight concurrent captures landed two.

### What that says about the design, not just the code

**Atomic is not the same as safe.** `write_atomically` was written carefully, tested, and specified,
and it protected exactly the failure it named: a torn file. The failure that actually cost data was
one directory up, in the read-modify-write around it. A guarantee is only as wide as its wording.

**Three of the four were invisible to `hi check`.** The checker is deliberately narrow, six
structural problems and no opinion about your English (§9), and that is still right. But "narrow"
has to mean narrow-and-honest, not narrow-and-blind: a criterion hi cannot parse must be reported,
never skipped. That is now `FILE-20`, and it is why capture refuses an id written where hi cannot
read it rather than treating unparseable as absent.

**The blind spot repeated for the third time.** §12 was the wall-of-text bug, found because every
test asserted on parse output. §25 was the `[hidden]` bug, found because every view test asserted on
generated HTML. These four were found because every write-path test asserted on files hi itself had
written. Each regression test added here is written over a file hi did not produce.

**Concurrency stopped being hypothetical without anyone deciding it had.** hi was built for a
person at a terminal writing one sentence at a time. It is now used by agents capturing in bulk:
twelve repositories, about 1,700 criteria, most of it generated. Nothing was announced. The usage
changed and the assumptions did not, which is the ordinary way a tool becomes unsafe.

**What would change this decision:** nothing about the promise, as narrowed above. If the locking
proves too coarse for a real workflow, the lock can narrow from the repository to the file. What
hi's own verbs are allowed to do to an id does not move.

---

## 27. How hi gets reached for

§22 asked whether hi is ever reached for unprompted and said the answer has to be measured on
somebody with a Tuesday. That is still true, and it hid a second thing: most of the reaching is now
done by agents, and an agent cannot form a habit at all. It reaches for whatever is in its context.
hi put nothing there, so a null result was measuring the distribution as much as the demand.

The death is not a `hi/` that goes stale. It is earlier than that. Someone reads the README, agrees
with it, installs the binary and never runs a second command, so there is no `hi/` to rot. That is
what `HABIT-1`, `HABIT-2` and `HABIT-3` are about, and this section is how they get served.

### hi writes its own file, and still never writes outside `hi/`

The obvious mechanism is a marked block in the repository's own `CLAUDE.md` or `AGENTS.md`, owned
by hi the way `INDEX-2` owns its generated block in `INTENT.md`. That was refused. §9 already
declined to stamp into human-authored, human-named files, and "but ours is visible" is how a
refused decision comes back wearing a hat. hi writes `hi/AGENTS.md` and nothing outside `hi/`.

`hi/CLAUDE.md` is a symlink to it, so both conventions are served by one truth rather than two
copies that drift. Where a symlink cannot be created — Windows without Developer Mode, or any
checkout with `core.symlinks` false, which is the Git-for-Windows default — hi writes a one-line
pointer file instead. That is two code paths and a committed result that differs by platform, and
it is still better than the alternative: a committed symlink checks out on those machines as a text
file containing the literal string `AGENTS.md`, which an agent reads as the whole instruction.

### Written at first capture, best effort

Exactly the `INTENT.md` pattern (`INDEX-3`): after the criterion is safely on disk, never before,
and a capture that succeeded is never reported as a failure because this could not be written.
There is no init step and no new verb, because `CAPTURE-1.a` promises hi is useful without one and
the death above *is* a second command that never gets run.

### It says the habit and nothing else

Read the files here, draft the criteria, ask the person to confirm them, capture what they agree
to, then build. For anything about verbs or syntax it points at `hi --help`.

That is a deliberate refusal of two better-sounding files. One carries the id grammar and the
one-line rule, so an agent needs nothing else — and describes a format DECISIONS says is not
frozen, in a file nothing keeps current. The other lists the families already written here, which
is wrong immediately after the next capture. A file written once has to be a file that cannot go
stale, and the only way to get that is to say less.

The confirmation in `HABIT-3` belongs to the agent and never to hi. `hi` itself still asks nothing
in the middle of a capture (`CAPTURE-1.b`); the question happens in the conversation, which is the
one place where a question is already the medium.

### `hi/` was not a free directory

Writing the instruction into `hi/` collided with what `hi/` already means. `Workspace::load` reads
every `*.md` directly inside it as criteria, so `AGENTS.md` and `CLAUDE.md` became documents with no
families, counted by `check` and listed by `index` as two features of the product:

```
- [AGENTS](hi/AGENTS.md): no families yet (0 criteria)
- [CLAUDE](hi/CLAUDE.md): no families yet (0 criteria)
```

That list is `INTENT.md`'s feature list and it is what gets published. The rule that resolves it is
derived rather than invented: **a criteria file is lowercase**, because `capture::start_file`
lowercases every family name, so an uppercase name in `hi/` is never a file hi wrote as criteria and
is therefore hi's own. One rule, and room for a later file without a growing list of names.

The cost is real and was accepted: a file somebody hand-named `Chat.md` stops being read as
criteria. What makes that survivable is the second half. **Skipping quietly would be the `FILE-20`
failure arriving by a new route**, so `Workspace::skipped` keeps every skipped path and `check`
reads inside them, reporting any criterion-shaped line as `stray-criterion`. The kind is reused
rather than added to: a criterion in a skipped file is the same failure that kind already
describes, nothing reads it where it is, and the README promises exactly six structural problems.
A fenced block in such a file stays an example rather than structure, exactly as under `## Intent`
(`FILE-9`), so documenting the format in a `hi/README.md` is not reported as a loss.

**`AGENTS` and `CLAUDE` are not available as family names.** A family names its own file,
lowercased, so `AGENTS-1` wants `hi/agents.md`: the same path as `hi/AGENTS.md` on a
case-insensitive filesystem, and a confusing neighbour on a case-sensitive one. Before the refusal
existed, capturing `AGENTS-1` on macOS produced `hi/agents.md has no frontmatter, so add ...`,
which is an instruction to convert hi's own instruction file into a criteria file. Capture now
refuses both names on every platform, before any write, because a rule that depends on the
filesystem folding case is two behaviours wearing one name.

### What this does not solve

An agent only reads `hi/CLAUDE.md` once it is already looking in `hi/`. Nothing here makes it look.
For a repository that has hi committed this is a small gap, and for a repository that has never
seen hi it is the whole of `HABIT-1` still open: the file cannot introduce hi to an agent working
in a repo where the file does not exist yet. Whatever closes that is not a file hi writes, and it
is not decided here.

**What would change this decision:** a way to reach an agent before `hi/` exists that does not
involve writing into somebody else's file. If one turns up, the first two subsections stay and the
last one gets an answer.

---

## 28. First contact, which is not a file hi writes

§27 closed with what it could not solve: `hi/AGENTS.md` only reaches an agent already looking in
`hi/`, and a repository that has never seen hi has no `hi/` to look in. That is `HABIT-4`, and the
answer had to come from somewhere other than a file in the repository.

It comes from fledge. **A fledge plugin installs once per user, not once per repository**, so hi's
plugin is present in every repository that person works in, including ones hi has never touched.
The plugin takes two lifecycle hooks: `post_work_start`, which fires as a feature branch is created
and before any code exists, and `pre_push`, the last moment anything can be said. In a repository
with no `hi/`, each says one line. In a repository that has one, both say nothing (`HABIT-4.b`).

**The hook can never fail.** `run_lifecycle_hook` propagates a non-zero exit, so a hook that errors
aborts the command that ran it: a bug in the nudge would block `fledge work push`. Every path in
`bin/fledge-hi-nudge` ends at `exit 0`, and `scripts/nudge-behaves.sh` asserts that first, before it
asserts anything about the words (`HABIT-4.a`).

**It writes to stderr and never to stdout.** `fledge work start --json` puts a JSON envelope on
stdout, and a hook printing into it would corrupt the envelope of a command that had nothing to do
with hi.

Two things this cost, both deliberate:

- **The plugin now declares `exec = true`.** fledge skips the hooks of any plugin that has not, so
  without it the nudge is silently never delivered. It is asked for at install time and the honest
  answer is yes: the plugin runs a script.
- **It needed a change to fledge.** A hook runs with its working directory set to the plugin's own
  directory and was given nothing naming the repository that invoked it, so it could not ask whether
  that repository had a `hi/`. `FLEDGE_REPO_ROOT` (CorvidLabs/fledge#520) fixes that for every
  lifecycle hook, not just this one. Where it is absent, on an older fledge, the nudge says nothing
  rather than guessing at the wrong tree.

**What this still does not reach:** somebody who installed hi with Homebrew or cargo and does not
use fledge. The mechanism works because the house norm is fledge-first, which makes it a CorvidLabs
answer rather than a general one. A general answer would have to live in the agent rather than in
any tool, and that is the option §27's interview declined for the instruction file.

---

## 29. A paragraph is one line, and only a blank line is a break

Every `## Intent` block hi had ever written or shipped as an example was hard-wrapped at somebody's
margin, because that is how a person wraps a paragraph in an editor. In a markdown *file* that is
invisible: GitHub joins the lines back into a paragraph, and so does every other renderer. In a
GitHub **issue or comment** it is not, because those are rendered with the hard-line-break extension
on and a single newline becomes a `<br>`. A ticket from `hi issue` therefore arrived as a narrow
column down the left of a wide pane, broken after every line the author had wrapped.
`CorvidLabs/corvid-bot`'s `hi/host.md` is where it was seen.

Two answers, and the second is the one that lasts.

### `hi issue` unwraps at render time

A single newline inside a paragraph is where an editor wrapped; a blank line is the break somebody
asked for. `out::unwrap_soft_breaks` folds the first into a space and keeps the second, and leaves
every newline that means something: a list item, a block quote, a heading, a table row, a thematic
break, the inside of a fenced block, and a line ending in two spaces or a backslash, which is
markdown asking for a break on purpose. Fences are recognised through `doc::fence_marker`, the same
helper the parser uses, because a fence means the same thing everywhere hi reads markdown
(`FILE-9`, `ISSUE-7.b`).

The three other read verbs were checked and left alone. `hi view` already renders these paragraphs
whole, twice over: `view::paragraphs` joins each block's lines with a space, and HTML collapses
whitespace anyway. `hi ls` and `hi index` emit no prose at all. And `hi export` **keeps the prose
verbatim**, deliberately: the payload is a transport rather than a rendering, no JSON consumer turns
a `\n` into a visible break the way an issue body does, a consumer that wants it unwrapped can
unwrap it in a line, and one that wants the source form can never get it back once hi has thrown the
wrap points away. A lossy transform belongs at the edge that needs it.

### The convention is one line per paragraph

Unwrapping at render time papers over the file. The file is what the next adopter copies, and what
they copy is whatever hi's own files, hi's own examples and the text hi writes into their repository
model for them. So `out::agent_instructions` and `out::starter_intent` are one line per paragraph,
this repository's own `hi/*.md` intent blocks were reflowed to match with no word changed, and the
README's example file and rule 5 say it (`FILE-21`, `FILE-21.a`).

`hi/AGENTS.md` says it too, in one sentence, and that is the single narrowing of §27's rule that the
file carries the habit and nothing else. §27 refused to put the id grammar or the file format in
there, because the file is written once and never rewritten and the format is not frozen, so
anything hi could change underneath it would be wrong later with nothing to notice. This sentence is
not the format. It is how markdown reads a newline, which is the same at `hi: 1` and at whatever
comes after it, and the agent that writes the prose is the only reader that file ever has.

A smaller thing the wrap had already cost: `check::product_intent_note` filters a line beginning
`Write it as a person`, and that filter exists only because `starter_intent`'s HTML comment was
wrapped onto a second line which no longer began with `<!--` and therefore read as human prose. The
comment is one line now. The filter stays, inert for anything hi writes from here, because the
`INTENT.md` files older versions already wrote still have the two-line form and are still owed the
nag.

### hi still never reformats a file it did not write

The obvious next steps are capture rewrapping prose as it saves, and a seventh `check::Kind` for a
hard-wrapped paragraph. Both are refused. `FILE-4` promises hi never rewrites, reflows or reformats
prose somebody wrote; `INDEX-2` promises the same for `INTENT.md`; `FILE-7`, `FILE-10` and `FILE-11`
promise their frontmatter style, their line endings and their byte-order mark come back untouched;
and `CHECK-1` promises `hi check` fails on a structurally broken file and on nothing else. A wrapped
paragraph is a preference, and §5 is the whole reason hi does not gate on preferences. The
convention travels as documentation and as the example hi sets, which is the only way a convention
is allowed to travel here.

This was found in somebody else's repository, and hi's own files had exactly the same wrapping. hi
reformatted its own and none of theirs. Ours are ours.

**What would change this decision:** evidence that the render-time fix is not enough, which would
look like the wrapping reaching somewhere a person reads that hi does not render — a spec an agent
wrote out of `hi export`, say. The answer then is still not a rewrite of their file; it is to decide
whether `export` should carry a second, unwrapped field beside the verbatim one.

---

## 30. The generated list has to be true by itself

`INTENT.md` carries a feature list hi generates between two markers, and `hi index` regenerates it.
Nothing made anyone run it. Three adopter repositories had already drifted: `peck`'s block said 56
against 57 actual, `podo-web`'s said 53 against 57. This repository is the one that did not, and
only because `scripts/index-is-current.sh` is wired into its gate, which no adopter has.

That is the same shape as §26's concurrency finding. A guarantee that depends on somebody
remembering is not a guarantee, and the first place it fails is the repository that adopted hi
without adopting hi's own build.

### Capture keeps it current, rather than check reporting it

The verb that changes the live count refreshes the block. Both of them: `capture` and `hi retire`,
since retiring changes the count too. Nobody has to remember, which is the difference between a
promise and a report.

The first version of this section said drift became "structurally impossible". That was too strong
and §32 corrects it. The refresh is best effort by design — the subsection below says so two
paragraphs later — so a file hi cannot read, a marker pair somebody broke, or a full disk all leave
the list wrong while the capture succeeds, which is right and is not impossibility. `hi index` also
still takes no lock of its own: the refresh is safe because it runs inside the lock `capture` and
`hi retire` already hold, and `hi index` typed by hand is an unlocked read-modify-write like any
other. What is true is narrower and still worth having: **the ordinary path no longer depends on
anybody remembering.**

The alternative was a seventh `check::Kind`. It was refused for the reason §9 and `CHECK-1` refuse
every other quality gate: `hi check` fails on a structurally broken file and on nothing else, and
adding a build failure for a stale generated file would make hi one more thing that turns a build
red on a Friday.

### Best effort, never fatal

The refresh runs only after the criterion is on disk, and any reason it could not happen is printed
as a line rather than raised. A capture that stored a criterion must never be reported as a failure,
because the report sends somebody looking for a sentence that is in fact there, and the four seconds
that sentence cost is the whole thing hi is protecting (`CAPTURE-1`, `INDEX-4.a`).

This is exactly how `INTENT.md`'s creation has behaved since it was added (`start_product_intent`,
`INDEX-3`, `CAPTURE-1.a`). The rule is now written down rather than repeated: everything capture
does after `Doc::save` is best effort and cannot fail the capture.

`INDEX-2.b`'s refusal to guess past a broken marker pair is included in that. It still refuses, it
still writes nothing, and the caller still exits 0. The refusal is printed on stderr, so stdout
stays the record of what landed.

### And `check` nags when the block still disagrees

One way to make the list wrong survives: type a criterion straight into a file. `FILE-14` explicitly
allows that, and no verb sees it happen. `hi check` compares the block to what it would generate and
says so, as a `note:` that never touches the exit code, following `INDEX-3.a`.

**This is the first time `check` nags about something hi itself maintains**, and that is worth
naming rather than letting it read as a natural extension. `INDEX-3.a` nags about a product-level
why, and `RETIRE-3` about a reason for a retirement: both are words only a person can write, and hi
is asking for something it could never supply. This one is about a list hi generates. The only thing
that makes it admissible is that capture and retire now keep that list current, so the note has
exactly one cause left and that cause is a human edit. If a future change makes the note fire for
something hi could have fixed itself, the answer is to fix it, not to keep nagging.

### What this costs

A bulk capture rewrites `INTENT.md` once per criterion. fledge's adoption was 197 captures, so 197
atomic replaces of a file a few hundred bytes long, serialized by a lock those captures already
contend for. That is real and it is accepted; the alternative is a list that is wrong 196 times out
of 197 and right by luck at the end.

A person who deleted the generated block gets one back on the next capture, through the append
branch `write_index` has always had. That branch was previously only ever reached by somebody typing
`hi index`, and it is now reached without being asked for. **§32 withdraws this one**: it was
accepted as a cost, and on a second look it is hi arguing with the person.

`scripts/index-is-current.sh` stays in the gate. It is no longer the thing that keeps this
repository's list true — capture is — and it is now a backstop for hand-edits here, which is what
`INDEX-4.b` covers for everybody else.

One more thing was found on the way in, and it is the reason the first version of this shipped a
count that was one too low: `Doc::insert` splices the rendered line into `Doc::lines` and shifts the
indexes around it, but does not add the criterion to `doc.criteria`. The in-memory `Doc` therefore
described the file as it was a moment earlier, and anything that counted from it counted short.
Capture reloads the file it just saved before anything counts. `Doc::insert` was deliberately left
alone: `retire` and `set_retired_reason` both end by reparsing, and making `insert` do the same
would have made its index bookkeeping — which the next insert and `rewrite_families` both depend on
— untestable, and there is a test that catches exactly that.

**What would change this decision:** a repository where the rewrite-per-capture is genuinely too
expensive, which would look like a bulk import measured in thousands rather than hundreds. The
answer then is not to stop refreshing; it is for a bulk path to refresh once at the end, which needs
a bulk path to exist first.

---

## 31. The write path and the parse path have to agree about where the sections are

Two more ways to break the one promise, found in an external review of 0.7.0. Both are the same
mistake, and neither is anybody's fault twice removed: a verb that *writes* decided where a section
was without asking the code that *reads*.

1. **`Doc::retired_heading` scanned the raw lines** for `## Retired`. A fence is opaque to the
   parser, on purpose, so that a person can document the format inside their own prose (§3,
   `FILE-9`). Put an ordinary, properly closed ```` ```markdown ```` example under `## Intent`
   showing the three headings, above a real `## Criteria` holding `SEND-1`, and `hi retire SEND-1`
   found the `## Retired` *in the example*. It moved the criterion into the intent prose after the
   fence, printed `SEND-1 retired`, and produced a file that parses back with zero criteria and zero
   retirements. `hi SEND-1 "something else"` then succeeded. `hi check` exited 0 at every step.
2. **`Doc::insert` appended a missing `## Criteria` section** after the file's last content line
   without asking whether that line was inside a fence. Leave a fence open — the normal state of a
   paragraph somebody is in the middle of writing — and the heading hi appends, and the criterion
   under it, are both part of the example. Capture printed `hi/chat.md  +SEND-1` twice for the same
   id with different sentences. Both lines were invisible, `hi check` reported zero criteria, and the
   file was valid the whole time.

The first one is the more alarming, because the file it destroys is *correct*. `FILE-9` invites the
example. The person did nothing wrong.

### The fix is a postcondition, not two patches

The obvious repair is to teach `retired_heading` about fences and to make `insert` check its
insertion point. Both are done, through the same state machine `parse_body` uses, because the
legitimate retirement still has to work rather than merely fail safely. But neither is the fix.

The fix is that **no write site is trusted to know where it landed.** `insert`, `retire` and
`set_retired_reason` each parse the buffer they are about to save and hold it to three things:

- every id the verb named is readable in the section the verb named;
- every id the document already made readable still is, compared as a multiset;
- nothing new has been stranded.

Anything else is refused, with the in-memory document restored, so nothing reaches disk
(`CAPTURE-5`). The refusal names an unclosed fence and the line it opens on when there is one.

This is the shape §26 was reaching for and did not take. Every one of the four bugs there, and both
of these, would have been caught by it, because all six have the same signature: hi wrote a
criterion somewhere and then could not read it back. A checker that asks "did the bytes land where I
put them" cannot catch that. A checker that asks "can a reader find this" catches all of it, and
catches the next one too, whatever the cause turns out to be.

`insert` still does not reparse *itself*, and §30's reasoning is untouched: its line-index
bookkeeping is the contract with `rewrite_families` and the next insert, and re-deriving it would
make that contract untestable. The verified parse is thrown away, and `capture` still reloads the
file it saved before anything counts. The read-back is a check on the bytes, not a replacement for
the bookkeeping.

### Refusing an unfinished file is a decision, and it is the smaller harm

Accepting an unfinished document is right, and stays right: `Doc::parse` is infallible, `hi check`
fails on structure and nothing else, and a half-written fence is somebody mid-thought. What is not
right is reporting a successful capture into it. So the refusal is narrow: the write is refused, the
prose is left exactly as it was, and the message says which line to close. Closing the fence makes
the identical capture succeed, which is the test that the refusal is not a dead end.

### The blind spot, for the fourth time

§12 was found because every test asserted on parse output. §25 because every view test asserted on
generated HTML. §26 because every write-path test asserted on files hi itself had written. These two
were found by somebody reading the code who was not us, and the fixture that reproduces them is a
file hi did not write: hand-typed, with a documented example in it, exactly what `FILE-9` invites and
exactly what none of our own `hi/*.md` contains. The regression tests are written over that file,
and one of them exists only to fail if the fix ever over-corrects into treating a fence as structure.

**What would change this decision:** nothing about the postcondition. If the reparse-per-write ever
costs too much, it narrows — check only the ids the verb touched rather than the whole document —
before it is removed. If a future write path genuinely cannot express its postcondition this way,
that is the signal it is doing something the format does not support, not the signal to skip it.

---

## 32. Making a write automatic widens whatever was already wrong with it

§30 moved the index refresh from something a person typed to something every `capture` and every
`hi retire` does. Nothing about `write_index` changed. Two defects that had sat in it since 0.2.0
went from reachable-if-you-type-a-command to running on every write in every repository, and one of
them destroys a file.

**That is the general lesson, and it is the third time this repository has met it.** §26 found that
concurrency stopped being hypothetical without anyone deciding it had. §30 found that a guarantee
depending on somebody remembering is not a guarantee. This one is the same shape from the other
side: *a code path's blast radius is a property of who calls it, not of the code*, so making a call
automatic is a change to every bug inside it. The review that ships an automatic call has to be a
review of the thing being called, not only of the calling.

### The one that destroyed the file

```rust
let existing = fs::read_to_string(&path).unwrap_or_default();
```

Every read error became an empty string, and the branch below it treats an empty string as "no file
here, write the starter". So an `INTENT.md` holding somebody's prose and one byte that is not valid
UTF-8 — a Latin-1 `é` pasted in, a truncated write, anything — was replaced by the starter prompt
and a generated list. Reproduced against the built binary: a three-line file became the scaffold and
`hi index` printed `INTENT.md  index updated` and exited 0.

`INDEX-2` promises hi only ever rewrites the list it generated and the prose around it stays the
person's. Replacing the whole file is the largest possible way to break that, and it is also the
quietest: nothing is reported, because from inside the function nothing went wrong.

**Only absence may create.** The read now matches: `NotFound` is an empty file, and every other
error returns `reading <path>` having written nothing. Through a capture that is a printed line and
exit 0 with the criterion stored (`INDEX-4.a`); through `hi index` it is the exit code, because
there the failure is the whole answer. That is `INDEX-2.c`.

**`unwrap_or_default` on a read is the shape to distrust.** It reads as a default and behaves as an
assertion that the file is empty. Where the next thing the code does is decide whether to create
something, the two are not the same and the difference is the file.

### The one that handed an id out twice

`Workspace::find_stray` walked `docs`. `check` separately walked `skipped`, the uppercase-named
files in `hi/` that are hi's own (§27). Both were looking for the same thing — a criterion-shaped
line nothing reads — and neither knew about the other's half. So a retired `SEND-1` parked in
`hi/Archive.md` was *reported by `check` as taken* and *handed out again by capture* with different
words. `CAPTURE-14` says an id written somewhere hi cannot read it is still taken and is never
handed out twice, and this was hi announcing the id was used and then reusing it.

There is now one lookup, `Workspace::strays`, covering both sets. `check` reports from it and
`capture` refuses from it, so the two cannot disagree again — not because they agree, but because
there is only one answer. Reading inside hi's own files reserves ids and nothing else: `docs`,
the criteria count, the families and the generated feature list are untouched, so `hi/AGENTS.md`
still never becomes a file that holds criteria.

**Two checks over overlapping inputs is the shape to distrust.** The audit in §26 found four ways
one promise broke; this is a fifth, and its cause is not a missing check but two checks that were
each correct about their own half.

### Withdrawing §30's accepted cost

§30 accepted that a person who deleted the generated block gets one back on the next capture,
through `write_index`'s append branch. On a second look that is wrong, and the reasoning is §9's:
hi does not stamp into human-authored prose. Rewriting the list between two markers hi wrote is what
`INDEX-2` permits. Adding a `## Features` heading is writing prose, and doing it on every capture
means a person who removes the block cannot keep it removed — they delete, hi restores, forever,
with no verb to say no with.

So the two acts are now separate. `refresh_index` passes `Absent::LeaveAlone`: it rewrites a block
that is there and installs none. `hi index` passes `Absent::Install`, because there it was asked
for. The effect of the automatic path on disk is now exactly a span replacement between two
markers, which is `INDEX-2` stated as a mechanism rather than as good behaviour — and that is what
makes it safe to run on every write.

Two consequences, both deliberate:

**The starter file carries its own list.** A first capture used to write the prose prompt and let
the refresh append the `## Features` section a moment later. With the refresh installing nothing,
`capture::start_product_intent` writes `out::starter_intent_file`, which is the prompt and the block
together. The file hi creates is complete from birth, and a refresh over it changes nothing.

**A file with no block gets silence.** Not a restored section, and not a note either. hi cannot tell
a block somebody deleted from one that was never written, so it does not guess; and `check` nagging
about it would be the thing §30's last paragraph warns against, a nag about something hi could have
fixed itself. Here hi *could* fix it and has decided not to, so nagging would be arguing in a
different tone. `hi index` is in `hi --help`, in the README's verb table, and in the note `check`
already prints when a block that exists is behind. That is enough places.

`INDEX-4.c` is the criterion for this, and it is the only genuinely new *want* in this pass. The
other two fixes restore `INDEX-2` and `CAPTURE-14`, which were already written down and already
true on paper.

**What would change this decision:** evidence that adopters are ending up with an `INTENT.md` that
has no feature list and no idea one is available. The answer then is a better first-run message or a
line in the file hi already writes, not hi editing a file somebody edited on purpose.

## 33. The fifth way, which was live through the audit that found the other four

> **The heartbeat this section introduces was wrong and is gone; §34 replaces it.** What survives
> here is the bootstrap half — `hi/` is created before the lock and a `Guard` is only ever a lock
> that was really taken — and the argument that age is not evidence. The mechanism that replaced
> age was reproduced as a defect within a day of shipping.

§26 named four ways hi broke its one promise, fixed them, and said the promise does not move. A
fifth was open the whole time, in the fix for the fourth.

Thirty-two concurrent `hi SEND-N "..."` in a fresh repository with no `hi/` yet: all thirty-two
exit 0, nine of them are not in the file, `hi check` exits 0 and reports nothing wrong, and
`hi SEND-1 "a completely different intent"` then succeeds and writes a second sentence under an id
that had already been spent. That is not a torture test. It is ordinary bulk adoption, which is how
this organisation uses hi: one repository holds 197 criteria and the estate about 1,600, nearly all
of it captured by agents that do not wait for each other.

### Why the lock did not cover the first capture

The lock file lives in the directory it protects, `hi/.hi.lock`. Before `hi/` exists there is
nowhere to create it, so `create_new` failed with `NotFound` — and `acquire` had this:

```rust
Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
Err(_) => return Ok(Guard { path }),
```

The comment above that last arm said a `hi/` we cannot write to is a problem the caller will hit
anyway, and it will say something more useful than "could not lock". That reasoning is sound for a
directory somebody made read-only. It is wrong for the case it actually met most often, which is a
directory that is not there yet, and it fails **open**: every writer is handed a guard, every writer
believes it is alone, and the failure it causes is the silent one hi exists to prevent.

The second half is worse. `Guard::drop` removed the lock file unconditionally, so a guard that had
never acquired anything deleted the lock of a writer that had. One bootstrap capture could unlock
the repository for everybody.

Three things generalize out of that:

**A lock that fails open is not a lock.** It is a lock-shaped thing that works when nothing is
happening. Fail closed, and say which file could not be taken.

**Do not hand back a token for work you did not do.** `Guard` is now constructed only by a private
function that takes the `File` that was exclusively created. "A guard that did not acquire" stopped
being a bug and started being a value that cannot exist, which is a stronger fix than remembering to
check a flag in `drop`.

**Bootstrap is a state, not a preamble.** The fix for §26's fourth failure was tested against a
repository that already had a `hi/`, because every fixture in the suite makes one — `capture::tests`
and `cli::Repo::new` both. The one state every adopting repository passes through was the one state
nothing tested. `cli::Repo::bare` now makes a repository holding nothing but a `.git`, which is
where the 32-way test starts.

### Retire wrote from a snapshot

Capture reloads the workspace under the lock, in as many words, with a comment saying why. `retire`
took the lock after `run` had already loaded the workspace and never read it again. Two concurrent
retires both print `retired`, both exit 0, and the file ends with one criterion retired and the
other live again, with `hi check` reporting nothing wrong. It is the same defect as §26's fourth,
one verb over, and it survived because the reload was written where the bug had been found rather
than everywhere the shape occurs.

The rule is now on both write paths and in `specs/main/`: **read inside the lock, or you are writing
from before it.** Holding a lock around a write you decided on before you held it protects nothing.

### Age is not death

The old rule broke any lock file older than sixty seconds, on the grounds that its holder must have
died. Age establishes that a process is old. A bulk capture is slow, a network filesystem is slow,
and a stopped process is not a dead one; the rule as written could take a lock away from a writer in
the middle of a read-modify-write, which is the exact failure the lock exists to prevent.

Two honest options: never break a lock, and tell the person which file to delete; or make the holder
say it is alive. Never breaking it is simpler and safer, and it wedges a repository on a `kill -9`
until a human reads stderr — which for an estate captured by agents is a real cost, because an agent
will retry rather than read.

So the holder now proves it: a thread refreshes the lock file every 250ms while the guard is held,
which moves its mtime. A waiter remembers the mtime it first saw and the moment it saw it, and
breaks the lock only after that mtime has stood still for five seconds of **the waiter's own**
elapsed time, re-reading it once more immediately before removing it so that a lock another waiter
has just taken is never the one removed. Nothing compares this machine's clock against the file's,
so clock skew on a shared filesystem cannot make a held lock look ancient.

That inverts what is being inferred. Before, "old" was treated as evidence of death. Now, absence of
a signal only the living emit is. `PATIENCE` went from five seconds to thirty, because a waiter has
to outlast the abandonment window or a lock whose holder was killed could never be recovered, and
because two hundred captures queueing through one lock is a few seconds on its own.

It costs a thread and a channel per write, for a hold that is normally measured in milliseconds and
usually ends before the first heartbeat fires. That is cheap, and the alternative was leaving in a
rule that can take a lock away from somebody who is using it.

### Windows made the fail-closed rule say more than it meant

Failing closed says: if you could not take the lock, do not pretend you did. The first version read
that as "any error is a failure", and Windows disagrees. A removed file there stays present until
every handle to it closes, so a waiter whose create lands in the window between one writer
releasing and the file actually going is told **access denied** rather than **already exists**.
Thirty-two queued captures are thirty-one handoffs, and one of them failed a capture on CI.

A handoff is not a locked-out directory, and the difference between them is not the error code but
whether it is still happening a moment later. An error that is neither "somebody has it" nor "the
directory went away" is now retried for half a second before it is reported. Nothing about failing
closed moved: no guard is handed back, ever. What moved is how long hi waits before deciding an
error was the truth.

Worth saying plainly, because it is the second time on this page: the bug was found by running the
thing in the state it actually runs in. Thirty-two concurrent processes on a filesystem that is not
the author's found it in one CI run, and no amount of reading the arm would have.

### What this says about the audit

Thirteen agents looked for what a 1.0 would freeze, found four ways the promise broke, and closed
them. The fix for the fourth one shipped with the fifth inside it, and every test written to prove
the fourth was fixed passed against it, because they all started from a repository that had already
been bootstrapped. §26 said each regression test added there was written over a file hi did not
produce. The same discipline applies one level up: **a write-path test has to start from a
repository hi has not touched yet**, or the most common state in the estate is the one state nothing
covers.

**What would change this decision:** nothing about the promise. If the heartbeat proves to be more
machinery than a small tool should carry, the fallback is to stop breaking locks at all and print
the file to delete; that is strictly safer and strictly less convenient. The one thing that may not
come back is age as evidence.

*It did not survive that long.* The heartbeat was reproduced as a defect immediately, and §34 took
a third option neither this paragraph nor the review that prompted it considered: hand the lock to
the operating system, where nothing has to be inferred and a killed process is not a special case.

## 34. The lock belongs to the kernel, because every other owner is a guess

§33 replaced "a lock older than sixty seconds is dead" with "a lock nobody has refreshed for five
seconds is dead". Both are the same shape: hi looking at a file and deciding, on its own, that
somebody else has finished. An external re-review took the second one apart in an afternoon.

### What was reproduced

Two `hi` processes, and a syscall interposer that only ever *delays* execution — it changes no
behaviour, it just holds a process at a chosen call until it is let go. Waiter A was paused at the
instant after it had confirmed the lock's mtime had stood still and immediately before the
`remove_file` that acts on that confirmation. Waiter B then broke the same lock, took one of its
own, and started writing; it was heartbeating throughout. A was released and executed its
already-approved deletion, removing **B's live lock**. Both saved their own snapshot of the file,
both printed the criterion they had stored, both exited 0, and one criterion was not in the file
afterwards. `hi check` reported nothing wrong, and capturing the missing id again succeeded with
different words.

The re-read immediately before the removal is exactly the mitigation §33 describes, and it is not
one: it narrows the window, and the window is still there, because *verifying* and *removing* are
two operations and the world moves between them. There is no third check that fixes that. Anything
hi can observe about a file it has to observe before it acts on it.

The disclosed SIGSTOP case is the same defect without the interposer: a holder stopped part-way
through its write has no heartbeat, so a second `hi` declared it abandoned after five seconds,
took the lock, wrote, and exited 0; the stopped process then continued and wrote its own pre-image
over the top, losing the second one's criterion.

### The decision

**hi does not decide when somebody else's lock has expired. The operating system does.** `flock(2)`
on unix, `LockFileEx` on Windows. Both belong to the open file rather than to the pathname, which
gives the one property no file-watching scheme can have: the kernel releases the lock when the
process exits, however it exits — cleanly, by panic, by SIGKILL, by the machine losing power on the
next boot's empty `/proc`. So:

- a `hi` that was killed frees its repository by itself, with nothing to delete (`FILE-23`), and
- a `hi` that is merely slow, stopped, swapped out or waiting on a slow filesystem keeps what it
  took for as long as it is alive (`FILE-24`),

and those two stop being in tension, which is what made every timeout-based design wrong. hi now
never breaks a lock at all. There is no age, no heartbeat, no takeover, and no code path that
removes a lock file hi does not itself hold the OS lock on.

### The dependency question, answered explicitly

hi has four dependencies — clap, serde, serde_json, anyhow — and no `libc`. An OS lock needs one of
them or raw FFI, and the reviewer offered "never break a lock, print the path to delete" as an
acceptable alternative. That alternative was rejected, because `FILE-23` is a criterion this
repository captured and a person would have to act on every crash for the rest of the tool's life.

**No dependency was added.** `flock` is one `extern "C"` declaration and two integer constants that
have been stable on Linux, macOS and the BSDs for thirty years; `LockFileEx` is one
`extern "system"` declaration, two flags and a zeroed `OVERLAPPED`. That is about twenty lines in
`src/lock.rs`, in one module, behind three functions with the same signatures on every platform.
A `libc` or `fs2` dependency would buy the same twenty lines and a supply chain. The trade is the
`unsafe` blocks, which are two calls that take a descriptor or a handle the caller owns.

### The part that is not obvious, and was nearly the same defect again

**Holding the kernel's lock on a file is not holding the pathname.** The holder unlinks
`.hi.lock` when it releases. A waiter that opened that file *before* the unlink is still queued on
it, and the kernel will happily grant it the lock afterwards — on an inode with no name — while a
third writer creates a brand new `.hi.lock` and is granted the lock on *that*. Two processes, two
valid locks, one repository. This is a well-known hazard of unlinking lock files and it would have
reproduced the original defect with a different mechanism.

Two rules close it, and both are load-bearing:

1. **Verify after acquiring.** Once the kernel grants the lock, `still_at` compares the open file's
   `dev`/`ino` against the pathname's. If they differ, hi is holding an orphan: it drops it and
   starts over. `lock::tests::a_lock_granted_on_a_file_that_was_replaced_is_not_the_repository_s_lock`
   forces exactly that sequence rather than hoping to hit it.
2. **Unlink while holding, never after.** `Guard::drop` removes the file and *then* closes it. In
   the other order, a new holder could be granted the lock between the close and the removal, and
   hi would delete a live lock — the original defect, rebuilt.

On Windows neither is available and neither is needed. A delete there marks the file and leaves it
in place until the last handle closes, and while it is marked every `CreateFile` on that name is
refused, so no replacement can exist during the window `still_at` covers. `still_at` is therefore
`Ok(true)` on Windows.

**What is actually verified there, and what is not.** CI runs the whole suite on `windows-latest`,
so the `LockFileEx` path builds, links and passes: one process cannot take a lock another holds,
the bootstrap into a repository with no `hi/` locks like every later capture, a holder killed with
`TerminateProcess` frees the repository with nothing cleaned up, and thirty-two concurrent captures
into a fresh repository all land. What is *not* asserted anywhere is the delete-pending argument
itself — that no replacement lock file can be created while a handoff is in flight. Nothing in the
suite forces that interleaving on Windows the way
`a_lock_granted_on_a_file_that_was_replaced_is_not_the_repository_s_lock` forces it on unix. *Since the
1.0 readiness pass, Windows makes the same comparison unix does: `still_at` compares the volume
serial and file index of the held handle against the path's, through `GetFileInformationByHandle`,
a fourth `extern` and no new dependency. The delete-pending argument may well be right; the promise
no longer rests on it.*

### What is still true, and what is not claimed

- Exclusion rests on nothing but the OS lock and the file identity check. It does **not** rest on
  timing, on mtimes, on pids — the pid in the file is for a person and nothing reads it back — or
  on anybody deleting anything.
- It does rest on the filesystem implementing `flock` properly. On NFS, SMB and similar, `flock`
  may be emulated, local-only or refused. hi fails closed on a refusal, with a message naming the
  path, and **hi has not been tested on any network filesystem**. A local checkout is the supported
  answer.
- Deleting `.hi.lock` by hand while a `hi` is running now breaks the exclusion that hi provides,
  where before it was the documented remedy. So the timeout message no longer says to delete it; it
  says the lock belongs to a running process and that hi takes it back by itself when that process
  exits.
- On a platform that is neither unix nor Windows the fallback is exclusive creation with no
  breaking, which is safe and is not self-healing. `FILE-23` is unmet there. hi ships for Linux,
  macOS and Windows, and this arm exists so the crate still builds elsewhere.

**What would change this decision:** a filesystem hi's users actually work on where `flock` cannot
be relied on. The answer then is a lock whose location is configurable, not a return to guessing
when somebody else is done. Age is not coming back, and neither is the heartbeat.

## 35. Every id is still there is not every criterion is still there

§31 made the write path and the parse path agree about where the sections are. §26 and §33 made a
write read itself back. Both were right, and a re-review walked straight through them with nine
lines of markdown:

```markdown
## Criteria
  ## Retired

- **SEND-1**  Original retired intent.
  retired: Dropped.
```

That file is legitimate. Two spaces in front of a heading is a thing people type, `parse_body`
trims before it looks for `## `, and hi reads the file exactly as its author meant it: nothing
active, SEND-1 retired. `hi check` exits 0.

Then `hi SEND-2 "A new want."` succeeded, and afterwards there were **two active criteria and none
retired**. SEND-2's sentence was `A new want. ## Retired` and SEND-1 was live again, still carrying
`retired: Dropped.` as a note. `hi check` exited 0 after as well. A retired id had come back, which
is the one thing `RETIRE-2` says cannot happen, and nothing anywhere reported a problem.

### Two causes, and both had to be fixed

**The parser disagreed with itself.** `parse_body` reads `  ## Retired` as a heading.
`read_criterion` reads any indented, non-blank line that is not itself a criterion as a
*continuation* of the criterion above it. Neither is wrong on its own; they are wrong together, the
moment a write puts a criterion immediately above such a line — which `insertion_point` does,
because the section's append point is the last content line before that heading. So the new
criterion ate the heading, and everything the heading had separated fell into `## Criteria`.

`read_criterion` now stops at anything `parse_body` would read as a heading, through a shared
`is_heading_line` so the two cannot drift apart again. **The indented heading is not rejected.** It
is valid markdown and valid hi, and refusing it would be fixing the file instead of the code.

**The postcondition was about ids.** `read_back` compared `readable()` before and after: a sorted
multiset of `raw_id`. Every id in that file *was* still readable afterwards. A comparison of ids
cannot see a criterion changing section, changing its sentence, or acquiring somebody else's
retirement reason, and all three happened here.

It now compares `shapes()`: (id, section, sentence, reason), sorted, for every criterion the verb
did **not** name. For the ones it did name, `retire` and `set_retired_reason` may change the
section and the reason and may not change the words. The refusal says which id and what would have
happened to it — `move SEND-1 into ## Criteria` — because "would leave SEND-1 unreadable" was
never going to be printed here: SEND-1 was perfectly readable, in the wrong place.

**The general lesson, which is the third time this repository has met a version of it.** §26 found
that a promise nobody checked was not a promise. §31 found that two pieces of code answering the
same question separately will eventually answer it differently. This one is narrower and sharper:
*a postcondition is only as strong as the thing it compares.* `readable()` was the right check for
the bug that prompted it — a criterion vanishing into a fence — and it was never a check on
anything else, while reading as though it were a check on the file. The two fixes are independent
on purpose: with the parser fix reverted, the postcondition refuses the capture rather than
losing SEND-1, and the test that fails is the one saying the legitimate file must still work.

### One thing this does not do

The comparison is strict equality of the unaffected criteria, but its baseline is a reparse of the
document's own text rather than its parsed fields, because `insert` deliberately leaves
`criteria` one splice behind `lines` (§30) and comparing against the fields would refuse a second
insert into the same `Doc`. So the guarantee is about the bytes hi is replacing, which is the right
thing to guarantee, and it is not a guarantee about anything that happened to the file between
`Workspace::load` and the write. That window is the lock's job (§34), not this one's.

**What would change this decision:** nothing, but the next postcondition added here should be
written by asking what a reader of the file would notice, not by asking what the writer changed.

## 36. Unreadable is not absent, and sharing a lookup shares its mistakes

§32 replaced two disagreeing reservation scans with one, `Workspace::strays`, so that an id `check`
reports as used and an id `capture` refuses are by construction the same set. That is still right.
It also had this in it:

```rust
let Ok(raw) = fs::read_to_string(path) else {
    continue;
};
```

A file hi could not read was passed over, and the lookup returned "no stray here" — which its two
callers read as *this id is free*. The re-review put one Latin-1 byte in a retirement reason in
`hi/Archive.md`, where a retired `SEND-1` was parked. `hi check` exited 0. `hi SEND-1 "Different
intent."` succeeded. `hi check` exited 0 again. The reservation was still sitting on disk the whole
time, in a file nothing had managed to open.

**Making the two callers share a lookup makes them share its mistakes.** §32's argument was that
they could no longer disagree, and they cannot; it did not follow that the shared answer was right,
and here it made a quiet wrong answer authoritative in a place — capture — where it had not been
before. That is worth saying plainly, because "one source of truth" is usually offered as though it
were the whole of correctness.

`strays` now returns `Result<Vec<Stray>>` and a file it cannot read is the failure, named, with a
hint. `find_stray` propagates it, so `capture` refuses before it writes anything at all — no
criterion, no `hi/`, no `INTENT.md`, no `hi/AGENTS.md`. `check::run` returns `Result<Report>` and
exits 1 through the same path any other I/O failure takes.

**This is not a seventh check kind and it is not a rule about what a criterion may say.** `hi check`
still fails on exactly six structural things, and still never fails on unfinished intent (§5,
`CHECK-1`). An unreadable file is hi saying it could not do the check, which is a different
sentence from hi saying the check found something. The six stay six.

The shape to distrust is `let Ok(x) = read(...) else { continue }` wherever the answer feeds a
decision about whether something exists. §32 named `unwrap_or_default` on a read as that shape; this
is the same shape spelled differently, in the function that same pass introduced. Both convert "I
could not look" into "there is nothing there", and the second of those is a claim.

**What would change this decision:** a repository where an unreadable file in `hi/` is normal and
the refusal is in the way. The answer then is to say which file and let the person move it, which
is what the hint already says, not to go back to guessing on their behalf.

## 37. Two workstreams reached for the same hand-chosen id, and git said nothing

hi's whole premise is that a person chooses the id and the id is permanent. Two branches were open
at once against this repository. One captured `FILE-22` for *when hi tells me it wrote something
down, I can find it again*; the other captured `FILE-22` for *if hi is killed while holding the
write lock, the next capture recovers by itself*. Both were valid captures: each ran against a
workspace where `FILE-22` was the next free number, because the other branch's file was not in it.

The same thing happened in the specs, where nothing even resembles an allocator. Both branches
wrote a requirement numbered `REQ-workspace-011`, and git merged the two files with **no conflict**
at all, because the headings landed in different places with different text around them. The result
was one document with two `### REQ-workspace-011` sections saying unrelated things — which
`specsync check` is not looking for and a reader would meet as a contradiction rather than as an
error.

### How they were resolved

`FILE-22` **keeps the want that reached `main` first**, from the write-path pass. The lock branch's
want was renumbered to `FILE-23` before it merged, and `FILE-24` was captured later in the same
family for the other half of the lock guarantee. No id carries two sentences; nothing was retired,
because nothing was withdrawn — both wants are live, under one id each.

`REQ-workspace-011` keeps the reservation-lookup requirement, and the write-lock one became
`REQ-workspace-012`, with its references in `specs/workspace/testing.md` and
`specs/capture/requirements.md` moved with it.

### Why this is written down rather than quietly fixed

Because the failure mode is invisible and the tool is about ids. `hi check` would have caught the
duplicate id in `hi/format.md` the moment both branches were on one tree — `duplicate-id` is one of
the six — and that is exactly what the branch that renamed its criterion was reacting to. Nothing
catches it *before* the merge, and nothing at all catches the spec one. So:

- **An id is only unique against the tree you captured on.** Two agents working in parallel on
  branches are two workspaces. hi does not coordinate across them and is not going to start: an id
  allocator with shared state is a state file, and §5 says hi has none.
- **The merge is where ids are reconciled**, and it is a human step. `hi check` on the merged tree
  is the thing that proves it; run it before trusting a merge that touched `hi/`.
- **A silent merge is worse than a conflict.** Both files here merged cleanly and both were wrong.
  Text that carries an identifier — a criterion, a `### REQ-` heading — deserves a look after any
  merge, whatever git said.

**What would change this decision:** nothing about how ids are chosen. If parallel capture becomes
common enough to hurt, the answer is a check that runs over a *merge result* — the one place the
duplicate is visible — not a reservation protocol between branches.

## 38. What two 1.0 reviews agreed on, and the one they left to be argued

Two independent readings of 0.7.0 asked what a 1.0 would be committing to, and arrived at the same
answer: not yet. Both landed on a version number that means nothing, a payload whose one version
field could never mean two things, and one write that never took the lock the other two take. This
section is what was done about each, and the reasoning for the two that were judgement calls rather
than defects.

### `hi: 1` was a number nothing read

`parse_front` stored the `hi:` value in `Front::version` and nothing outside `doc.rs` ever read it;
the only other references were test assertions. So a file saying `hi: 2` loaded, `hi check` exited
0, `hi ls` printed its criteria, and a capture appended to it as HI/1 and said nothing.

That is worse than a missing feature, and it is worse in both directions at once. Going forward,
`HI/1` cannot be frozen by a 1.0, because a reader that treats every version as 1 has not committed
to anything. Going backward, an HI/2 can never ship, because every binary already installed would
open an HI/2 file, believe it understood it, and write into it. The version field is only worth
having if the *old* binary refuses; a new one understanding a new format is the easy half.

So: absent is HI/1, an empty value is HI/1, `1` is HI/1, and anything else is refused. An absent key
is HI/1 because every file written before the key existed has none, and refusing those would be the
version check breaking the format it exists to protect. An empty value declares no more than an
absent key does, and refusing a file for being untidy is not what this is for.

**It is not a seventh check kind, and the question is not close.** `hi check` fails on six
structural things and the README says exactly six (§5, `CHECK-1`). The six are things hi found wrong
*inside a file it read*. An unknown version is hi saying it did not read the file, which is the same
sentence as "I could not decode this file" and already has a home: §36 put that one in the
operational channel for the same reason. Adding a seventh kind would also make the refusal a finding
of `check` alone, when what is wanted is that no verb touches the file.

**The refusal lives in `Workspace::load`**, which every verb goes through before it does anything
else. One place rather than eight, and because `load` runs before `lock::acquire`, a refused capture
has not written the criterion, has not made `hi/`, has not started `INTENT.md` or `hi/AGENTS.md`,
and has not left a lock file (`FILE-25.a`, `CAPTURE-5`).

One file refuses the whole repository. A neighbour at this version does not rescue it, for the
reason §36 gives: hi cannot answer a question about a repository it has only partly read, and an id
it could not look for is not an id that is free.

### `"hi": 1` in the export payload was two things wearing one name

`hi export` emitted `"hi": 1`, and that 1 is the *file format*'s version. It can therefore never
later mean "this JSON is shape 2". The day the payload grows a field or moves one, `hi` cannot be
the thing that says so without also claiming the files on disk changed: a consumer pinned to a shape
would be told the format moved, and a consumer reading HI/1 files would be told it had not.

`export: 1` is now beside it. Both are 1 and both are free to move apart. This costs one field today
and is impossible once anything depends on the shape, which is the whole argument — there is no
later moment at which it gets cheaper.

### `hi index` was a read-modify-write outside the lock

Capture and retire hold `lock::acquire` across their whole read-modify-write. `hi index` did not,
and it is the one path that may *install* a block: it reads `INTENT.md`, splices the generated list
into it, and writes the rest back. Unlocked, a capture that finishes between that read and that
write is undone — the list goes back without the new criterion, and any prose saved in between goes
with it — while both commands print success. That is the §33 shape exactly, in the one verb nobody
had looked at, in a tool whose last three defect rounds were all about racing writes.

It takes the lock now, and reloads the workspace under it for the reason capture and retire reload.

**`hi view` deliberately does not, and this is the part worth arguing rather than assuming.** It
looks similar: it writes a file, and the file is derived from the criteria. It is not the same
thing. `view::write` never reads the page it is about to write; it renders the whole page from the
workspace and writes it. There is no window between a read and a write in which somebody else's
work can be put back, because there is no read. Three further reasons, none of them decisive alone:

- `lock::acquire` creates `hi/` to live in. A read verb that takes the write lock starts writing
  into a repository it was only asked to look at, and in a repository with no `hi/` yet it would
  create and then remove one.
- The page is derived and gitignored. The worst a race can do is publish a page one criterion out of
  date, which the next run fixes, and no id depends on it. That is a different order of consequence
  from losing a criterion.
- `hi view` in CI would queue behind a bulk capture for nothing.

The honest summary is that the lock is for read-modify-write and `hi view` is not one. If `hi view`
ever grows a reason to read its own output — an incremental render, say — it takes the lock that
day. One thing this section is *not* claiming: `view::write` uses `fs::write` rather than
`doc::write_atomically`, so a page can still be left half written by a crash. That is a torn derived
artifact and a different problem from this one; it is noted here so the next reader does not mistake
silence for a decision.

### The notes have codes, and no layout

`check` built its notes as strings and joined them with `"\n      "` — six spaces of terminal
indentation, inside the data. That is why `--json` carried one `note` string and a consumer had to
split on whitespace to get the notes back apart.

Each note now has a `NoteKind` with a stable code and a message with no indentation in it. Four
codes, chosen for the remedy rather than for the sentence: `no-product-why`, `index-behind`,
`index-markers`, `unexplained-retirement`. A missing `INTENT.md` and an `INTENT.md` with no prose
share a code because the answer to both is *write the why*; a stale list and an unpaired marker pair
do not, because one is fixed by running `hi index` and the other by hand.

None of them moves the exit code, `Report::ok` does not read them, and none of them is a `Kind`. The
six stay six.

While the codes were being made stable, the second definition of them went: `Kind` used to derive
`Serialize` with `rename_all = "kebab-case"` *and* have a `code()` method, which agreed with it. §31
is about two pieces of code answering the same question and eventually answering it differently, and
a code a consumer is invited to match on is the wrong place to leave that open. Both enums serialize
through `code()` now.

### A test that could not fail, and what it was really wrong about

`lock::tests::a_lock_that_cannot_be_taken_is_refused_rather_than_pretended` makes a directory
unwritable and asserts that `acquire` refuses. Root ignores the mode bits, and so does anything with
`CAP_DAC_OVERRIDE`.

The review that raised this said the test *passes vacuously* as root. It does not: as root the
directory is writable, `acquire` succeeds, and the assertion fails. So the defect was a test that
fails for a reason that has nothing to do with the code, in exactly the environment a container CI
job runs in — which is the noisier failure and the one that gets a test deleted.

Either way the fix is the same, and it is not a uid check. The test probes the precondition it
actually needs: it writes a file into the directory it has just made unwritable, and if that
succeeds, the directory is not unwritable — whoever this process is and whatever granted it — so the
test skips with a message saying so. A uid comparison would be a proxy for that, and would be wrong
under a capability that grants the same power without uid 0.

### The `hi/AGENTS.md` migration already exists, and is not a mechanism

hi writes `hi/AGENTS.md` and `hi/CLAUDE.md` when they are absent, and never again (§27). The seed
text just changed, so every repository that has one has an older one.

**The migration is: delete the file and capture. The next capture writes the current text.** That is
the whole of it, it has always worked, and it was undocumented. It is documented now, in the README
and here.

A `--rewrite-agents` verb was considered and refused. Write-once is not an accident to be worked
around; it is the property that makes the file unable to go stale, because there is exactly one
moment when hi's words are in it and no moment at which hi overwrites something a person edited.
The file is theirs after it is written — that is what `hi/` being the repository's own directory
means. A verb that rewrites it has to decide what to do with a file somebody has changed, and every
answer to that is worse than the one-line remedy above. If enough adopters ever want the newer text
badly enough to ask for a verb, that is a 1.x conversation and it starts from evidence, not from a
1.0 obligation.

### The one the reviews left open: does the seed file say more?

A reviewer proposed that `hi/AGENTS.md` should tell an agent to run `hi check` after a merge that
touched `hi/`, because cross-branch id collision is the one failure mode the promise has left (§37)
and nothing in the file mentions it.

The tension is real and it is §27's: that file says the habit and **nothing else**, precisely so it
cannot go stale, and §29 already bent it once by adding a sentence about wrapping. "Just one more
sentence" is how a file that was supposed to say less ends up saying everything.

**It was added.** The reasoning, stated so the next person can hold it against the same standard:

The test is not *is this one more sentence*. It is *can this sentence ever become false*, because
the file is written once and nothing will ever come back to correct it. §27 refused two specific
things — the id grammar and the list of families already in the repository — and both fail that test
loudly: the grammar describes a format DECISIONS says is not frozen, and the family list is wrong
immediately after the next capture. §29's wrapping sentence passes it, because it is how markdown
reads a newline, which is the same at `hi: 1` and at whatever comes after.

This one passes it too, on both halves. It is a *habit* — read, draft, confirm, capture, check after
a merge — which is the category §27 said this file carries, rather than a fact about the format. And
the only thing it names is `hi check` finding a duplicate id, which is one of the six structural
problems the README promises, which `CHECK-2.a` captures, and which is about as close to frozen as
anything in hi is.

There is a real cost and it should be said plainly rather than argued away: the file is one sentence
longer, the third narrowing would be easier to justify than this one was, and there is no mechanism
stopping a fourth. So the rule is written down here rather than left to judgement. **A sentence
earns a place in `hi/AGENTS.md` only if it is a habit rather than a fact about the format, and only
if the one thing it names is something hi has committed not to change.** Anything that fails either
half goes in the README, where it can be corrected.

The other half of the cost is the one §27 already carries: existing repositories keep the old text
until somebody deletes the file. That is the migration above, and it is why the migration needed
writing down before this sentence was worth adding at all.

**What would change this decision:** an adopter's `hi/AGENTS.md` turning out to be read and then
ignored, which would mean length is the problem and the answer is to cut rather than to add. Or a
sentence that fails the rule above getting in anyway, which would mean the rule is not load-bearing
and the file needs a hard cap instead.

## 39. The seventh kind, the migration verb, and what a 1.0 actually freezes

Three 1.0 readings of 0.7.0 left the same three things open: two files can both declare a family
and capture will pick one by path order; `hi/AGENTS.md` is write-once, so every adopter of 0.5.0
keeps a hard-wrapped file that does not mention the merge; and there is no document a consumer can
hold a 1.0 to. This section is the argument for what was done about each, and for the property test
that is the only change to how the next defect gets found.

### A family is a function from name to file, and first-wins is not a function

Two files can both list `families: [SEND]`. `hi check` exited 0. A new `SEND-3` landed in whichever
file sorted first, because `doc_for_family` uses `.position()` over path-sorted docs. Rename a file
and later captures move. That is not a hypothetical: it is what the binary did, and it is the same
shape as handing an id out twice, one directory up — the *home* of a family is not stable, so the
id that is about to be written is not either.

Three options, and the first two are the ones that look like decisions.

**Define it: the first declaration owns the family.** Path order is already how load works, so
freezing it is free in the code and a sentence in the README. It is also freezing a bug. The owner
of a family would be a function of the names of the *other* files in `hi/`, which a person does not
control by writing the file they are looking at. A rename, a squash, a `git mv` to match a title
change, would move later captures without touching a word of the family. A 1.0 that froze that
would be a 1.0 that froze "whatever `Path::cmp` does to the names you happened to pick."

**Leave it outside the contract.** Honest about 0.7.0, and it makes 1.0 a freeze of a tool that
will silently put `SEND-4` in a different file from `SEND-3` because somebody renamed `chat.md` to
`messaging.md`. The whole point of a family declaration is that capture can resolve an id to a
file without scanning. Two declarations means it cannot. Calling that out-of-scope is calling the
resolver out-of-scope.

**Refuse it.** A family two files both claim is a structural problem: the file is well-formed on
its own and the *workspace* is not. That is the same category as `duplicate-id`, which is also a
fact about two files rather than about one. It is not operational-at-load. Load refusing the
workspace would block `hi ls` of a salvageable tree, and the salvage is one frontmatter line. It
is not a note: a note would leave capture writing into the first-wins result, which is the defect.

So: `duplicate-family` is the seventh `Kind`. `hi check` reports it on the later file, naming the
first, the same shape as `duplicate-id`. Capture of a new top-level id in that family refuses,
writes nothing, and names both files (`CAPTURE-5`, `CAPTURE-16`). A case still follows its parent
(`CAPTURE-4.a`), because the parent has a unique home even when the family does not. Reads still
work. A family listed twice in *one* file is the same declaration written twice, not two homes.

The README promised exactly six kinds. The argument against adding a seventh, made in this file
more than once (§29, §30, §36, §38), was never "six is the number." It was: `hi check` fails on a
structurally broken file and on nothing else, and a quality gate, a wrapped paragraph, an unknown
version, or an unreadable file is not that. Duplicate-family *is* that. What 1.0 freezes is the
policy (structural only) *and* these seven members. An eighth is a 2.0, because `Kind` is a closed
set a consumer matches on. The cardinality was the thing not to freeze in 0.x so that this seventh
could still arrive; freezing 1.0 without it would have frozen the first-wins bug instead.

### Write-once is still the property; `hi seed` is the verb that is allowed to touch the file

§38 refused a `--rewrite-agents` flag because write-once is what makes the file unable to go stale
in the dangerous direction: hi's words go in once, and after that the file is the person's. That
half is still right, and capture still only writes `hi/AGENTS.md` when it is absent. The path that
runs on every thought never overwrites.

The other half was not right, and a third 1.0 reading said so. 1.0 freezes the convention that
file describes. An adopter who captured on 0.5.0 has a hard-wrapped template with no merge
sentence, and "delete the file and capture a dummy criterion" is a migration that (a) requires a
thought they do not have, which is the death `HABIT-1` is about, and (b) destroys a file they may
have edited, which is the thing write-once was protecting. Both of those are true at once. A
migration that is "delete and recapture" is a migration that no longer works the moment 1.0
commits to the convention, because the file an agent actually reads is the old one, forever, in
every repository that adopted before the freeze.

So there is a verb, `hi seed`, and it is narrow on purpose.

| The file is | `hi seed` does |
|---|---|
| Missing | Writes the current text, and `hi/CLAUDE.md` beside it. |
| Byte-identical to a template hi has shipped, after folding a BOM and CRLF | Rewrites it to the current text, keeping the endings the file had. |
| Already current | Says so. |
| Anything else | Refuses, exit 1, writes nothing. |

Recognition is byte identity after folding storage. A BOM and CRLF are how an editor saved the
file, not how a person edited it. A single added space is an edit. Fuzzy matching would be hi
deciding the person's words were close enough to its own, which is the rewrite §38 refused, in a
softer voice.

The known templates are files in `src/seed/`, the bytes 0.5.0 and 0.6.0 actually wrote. Current
is `out::agent_instructions()`. Adding a template to the known set is how a later 1.x ships a
new sentence; the rule — identity against a list hi shipped — does not move.

An adopter with a hard-wrapped 0.5.0 file runs `hi seed`. That is written in the README, in
`hi seed --help`, in HI-1.md, and here, which is every place they will look before they look in
DECISIONS.

### The contract is a file, and permanence is over shared history

Three reviews called a compatibility document blocking and none of them wrote it. [HI-1.md](HI-1.md)
is that file. It ships with the crate (`docs/` does not). It names what is frozen, what is not, the
normative format, the exit codes, the export envelope, the seven kinds, and the promise.

The README sentence, reconciled from two wordings that were each half of it:

> Permanence of an id is a convention over shared history: the merged tree, not an unmerged branch.
> hi's own verbs refuse to be the one that breaks it; `hi check` on the merged tree is the thing
> that proves it.

"Shared history" is §37, stated as a scope rather than as an anecdote. "hi's own verbs" is §26,
stated as the width of the promise. Together they are a sentence a 1.0 can keep. The wider wording
— "an id is permanent" as a property of the world — is one it cannot, because the files are
markdown and two branches are two workspaces.

### The suite now generates the case nobody wrote

Zero of the twelve confirmed defects were found by hi's own tests generating a case. Each was found
by a person writing a fixture that looked like the bug, which is why the thirteenth would have the
same shape. `src/promise.rs` starts from files hi did not write — bare lines, a fenced example, CRLF,
a stray, two families in one file — and runs random capture, retire, and hand-edit sequences.
`tests/promise.rs` does the concurrent half through the real binary, including against a file hi
did not write. After every step: every id a verb reported as saved is readable by `Workspace::load`
in the section the verb named; the shape of every criterion the step did not name is unchanged; no
id is assigned twice; and `hi check` exiting 0 implies all three (`ID-5`, `ID-5.a`).

That is the item that changes the finding method. It does not change the promise.

**What would change this decision:** evidence that `duplicate-family` is firing on a workspace
people meant to split across files, which would mean the kind is right and the refusal is too
sharp — then capture of a new top-level id could ask, and asking is `CAPTURE-1.b`, so the answer
is still refuse, with a better hint. Evidence that `hi seed` rewrote a file somebody had edited
because a template drifted into their words by chance, which would mean byte identity is not
enough and the answer is to also require that the file is still only the paragraphs hi writes,
not to fuzzy-match. Evidence that the property test is not finding things because it cannot
reach them, which is a gap in the generator, not a reason to go back to only-authored fixtures.

## 40. One criterion is a scope, because a context window is a budget

`hi export` took a family, a file, or the whole repository. On a product with a few hundred
criteria the whole repository is tens of thousands of tokens, and a family can be most of it. An
agent building one criterion needs that criterion, what it is a case of, and why the feature
exists. Everything else in its context is cost: it is paid for on every turn and it crowds out the
code the agent is actually reading. hi's own `hi/` exports at roughly 10,000 tokens whole and about
300 for `EXPORT-7`.

So an id is now a scope (`EXPORT-7`). It is the smallest change that answers the problem, and it
is shaped by three things already decided.

**It is the same envelope.** `EXPORT-3` says a smaller export is the same payload with less in it,
and HI-1.md freezes the envelope at `"export": 1`. An id scope adds no field and moves none. HI-1.md
gains the id in the list of values `scope` can hold, and one normative line saying what an id
scope carries.

**An id wins, and that is a precedence rule.** A file and an id cannot collide: an id's family
starts with an uppercase letter and a file hi reads as criteria starts with a lowercase one. A
family and an id should not collide either, because a family has no hyphen, but nothing validates
the names a file *declares* in frontmatter. A cold read of this change declared `families: [SEND,
SEND-1]`, `hi check` passed, and `hi export SEND-1` came back empty because the family arm won.
So when a scope parses as an id it is read as an id, whatever frontmatter declares. Nothing that
selected a file or a real family before selects anything different now.

**The criteria above come with it** (`EXPORT-7.a`). Export carries `parent` on every entry, and a
payload whose `parent` names something that is not in it is a reference to nowhere. A case read
alone is also a sentence with its subject missing: "if I have no connection it queues" means
nothing until you know what "it" is. The cost is a handful of lines, and the alternative is an
agent asking for the parent or, worse, guessing it.

That holds in a workspace `hi check` passes. A case whose parent was deleted by hand is an
`orphan-case`, and export still names the parent it does not have rather than inventing one or
dropping the case. Export reports what the files hold; `hi check` is where the break is reported.

### What was deliberately not built

- **A token budget.** `--budget 20k` would have to decide what to drop, which is hi judging which
  of a person's criteria matter. The id scope lets the *caller* choose the slice, which is the only
  party that knows what the work is.
- **A reverse lookup from an id to the specs and code that cite it.** §23 put "where is the car
  now" in a different layer from hi. A lookup that walks the repository for `hi: SEND-1` is that
  layer's job, and spec-sync already reads those citations.
- **An outline mode for large sets.** The feature list in `INTENT.md` and `hi ls --family` already
  answer "what is here" without the sentences.
- **Several ids in one call.** `scope` is one string, and making it a list is a shape change.
  Two calls are two payloads, which a consumer can already hold.

### The agent file says it, on §38's test

§27 said `hi/AGENTS.md` carries the habit and nothing else, and points at `hi --help` for verbs.
§38 admitted one sentence naming a verb on a test: it is a habit rather than a fact about an
unfrozen format, and the one thing it rests on is frozen. This sentence passes the same test.
"Read only the criterion you are building" is a habit, and the one thing it rests on is that an
id is a scope `hi export` accepts, which HI-1.md now states beside the envelope.

There is also a reason specific to this sentence. "Read the files here" is the right first step
and becomes the wrong one as `hi/` grows, because the instruction stays the same size while what
it costs keeps growing. That is the one way a file written once can go stale without a word in it
changing, and the sentence is what keeps it from happening.

The 0.8 template joins `src/seed/` as a known file, so `hi seed` upgrades an untouched copy and
leaves an edited one alone, exactly as §39 describes.

### Where this lands

0.8.0 is the 1.0 release candidate, and nothing but a defect in the promise was planned before
the tag. This is not a defect. It is additive: no new field, no new kind, no change to the
format or the exit codes. It sits under `Unreleased` in the changelog, and whether it ships before
or after the 1.0 tag is a release decision, not one this section makes.

**What would change this decision:** evidence that agents given an id scope keep asking for the
rest of the family, which would mean the slice is too narrow and siblings belong in it as one-line
context. Evidence that `scope` needs to name several ids in practice, which would be an
`"export": 2`, not a comma inside a string.
