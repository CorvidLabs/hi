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
families: [SEND, RECEIPT, OFFLINE]
owner: leif
---

# Chat

## Intent

I want to talk to people I trust without anyone in the middle being able to read it,
and without it feeling like a security product. It should feel like texting.

## Criteria

- **SEND-1**  I hit enter and the message shows up right away, marked as sending.
  - **SEND-1.a**  If I have no connection it queues and tells me, and never silently disappears.
  - **SEND-1.b**  If the thread was deleted before it sends, it warns me before discarding.
    - **SEND-1.b.1**  The draft is kept.
- **SEND-2**  It reaches them and the mark changes to sent.

- **RECEIPT-1**  I can tell the difference between sent and read without thinking about it.

- **OFFLINE-1**  I can read old threads with no connection.

## Retired

- **SEND-3**  Messages auto-delete after 24 hours.
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
| `hi ls` | List criteria, grouped by file, optionally filtered to one family with `--family` and including retired ones with `--retired`. |
| `hi issue <ID>` | Print a ticket-shaped markdown block. `--create` shells out to `gh` to open a real issue, into `--repo owner/name` if you name one. |
| `hi export [FAMILY \| file]` | JSON payload for an agent. Takes a family, a file, or nothing (the whole repo, including `INTENT.md`). |
| `hi index` | Regenerate the feature index in `INTENT.md`. |
| `hi view [--out FILE]` | Write one self-contained HTML page of the intent, for people who do not read markdown. |

### Capture

Capture resolves on the ID, and the rule is **a new ID just works; an existing ID refuses**.

| You type | hi does |
|---|---|
| A new ID in a known family | Appends to that family's file. |
| A new ID in an unknown family | Creates `hi/<family>.md` (family name, lowercased, underscores written as hyphens, so `SEND_2FA` becomes `hi/send-2fa.md`) with frontmatter, and drops it in. No prompt. |
| An ID that already exists | Refuses, and suggests the next free number. |

```console
$ hi SEND-2 "it reaches them and the mark changes to sent"
hi/chat.md  +SEND-2

$ hi SEND-2.a "if they blocked me it just never delivers"
hi/chat.md  +SEND-2.a

$ hi BILLING-1 "I can see exactly what I paid for"
hi/billing.md  created
hi/billing.md  +BILLING-1

$ hi SEND-2 "something else"
error: SEND-2 already exists in hi/chat.md:31
hint:  next free is SEND-3
exit 1
```

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
  compiled binary: it execs the `hi` on your `PATH`, or the repo's own `target/release/hi`. A root
  `plugin.toml` points fledge at it. Standalone-plus-plugin is the established norm; a separate
  `fledge-plugin-hi` repo is not.
- **Scaffold** `fledge templates init hi --template rust-cli --org CorvidLabs` (nine files,
  including CI and release workflows).
- **Install** `cargo install human-intent`, or a release archive, or
  `fledge plugins install CorvidLabs/hi` for `fledge hi`. The fledge plugin is not bundled with
  fledge and builds from source, so it needs cargo too. Homebrew was planned and is not built:
  there is no `hi` formula in `CorvidLabs/homebrew-tap` yet, so `brew install corvidlabs/tap/hi`
  does not work.
- Always written as **"hi (Human Intent)"** in anything searchable. Bare `hi` is unsearchable and
  collides with a universal shell greeting.

### Publishing

The full spec-sync treatment: **public repo, a README that teaches the format in 60 seconds, and a
docs site at `corvidlabs.xyz/hi`**, with the binary as the reference implementation. The repo and
the README shipped with v0.1.0; the docs site has not been built yet and `corvidlabs.xyz/hi` still
returns 404, so the README is the only documentation there is. The format is documented prose.
There is no formal grammar and no conformance-vector suite, because the format is four rules and a
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
  false-alarm rate is how a tool gets disabled in week one.
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
  rather than guessing (`INDEX-2.a`, `INDEX-2.b`). Whole-line matching is narrower than `INDEX-2.a`
  reads, and the gap is known and still open: a marker pair written alone on its own lines inside a
  fenced code block in `INTENT.md` is still taken as the real block, so `hi index` writes the list
  into your example and leaves the real block stale. `specs/out/` records it.

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
