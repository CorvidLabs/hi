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

I want to talk to people I trust without anyone in the middle being able to read it,
and without it feeling like a security product. It should feel like texting.

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
  the thing a spec can never carry, and the first thing an agent should read.
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

**Structural errors** (exit 1), which are exactly the six variants of `check::Kind`: a duplicate
ID; a case whose parent does not exist; an ID that collides with a retired one; a line shaped like
an ID that is not a valid one; a family a file never declared; and a criterion stranded outside
every section, where nothing would read it.

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

**An id is permanent and never reused.** Everything else in hi is a preference. This is the claim
the format rests on, because the whole point of an id is that it can be quoted somewhere hi will
never see: a ticket, a spec, a commit, a conversation. An id that can be reassigned is worse than
no id, because the quotation silently starts pointing at something else.

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

**What would change this decision:** nothing about the promise. If the locking proves too coarse
for a real workflow, the lock can narrow from the repository to the file. The promise does not move.

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
