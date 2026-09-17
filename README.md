<div align="center">

# hi

**Human Intent; acceptance criteria in human words, with ids that never move.**

Rust · single binary · no network, no model, no API key

**[See a live page](https://corvidlabs.github.io/hi)** · [Docs](https://corvidlabs.xyz/hi) · [Install](#install)

</div>

---

Specs drift into coding intent: modules, contracts, API shapes. That is the right thing for a spec
to do, and [spec-sync](https://github.com/CorvidLabs/spec-sync) already checks it. But by the time
something is a spec, what a person actually wanted has usually been translated away.

`hi` is where the untranslated version lives.

A criterion says what the thing **should be**, not what it currently does. It can describe something
true today, something a year out, or something that has since been revised. They are directions, and
directions stay correct through a wrong turn: a criterion that is false right now means the code has
not arrived yet, not that the criterion is wrong. Whether the code has arrived is a question for
something that reads the code, which is what `hi export` feeds.

## The format in 60 seconds

`hi/chat.md`:

```markdown
---
hi: 1
families: [SEND, RECEIPT, SPEND]
owner: leif
---

# Chat

## Intent

I want to talk to people I trust without anyone in the middle being able to
read it, and without it feeling like a security product. It should feel like
texting.

## Criteria

- **SEND-1**  I hit enter and the message shows up right away, marked as sending.
  - **SEND-1.a**  If I have no connection it queues and tells me, and never silently disappears.
  - **SEND-1.b**  If the thread was deleted before it sends, it warns me first.
- **SEND-2**  It reaches them and the mark changes to sent.

- **RECEIPT-1**  I can tell sent from read without thinking about it.

- **SPEND-1**  An operator can cap what the service spends in a day.

## Retired

- **SEND-3**  My messages auto-delete after 24 hours.
  retired: we decided this was a different product
```

That is the whole format. Four rules:

1. **A criterion is one markdown list item: a bold id, two spaces, and a sentence.** However long the
   sentence runs it stays on one line, so criteria stay greppable and diffable. And because it is
   a list item, it renders as its own line everywhere, instead of markdown joining it into a
   paragraph with its neighbours.
2. **You write the id yourself**, because you are the one who has to say it out loud. `SEND-1` is a
   name, not a position.
3. **Letters are cases, numbers are steps**, alternating strictly: `SEND-1.a.1.b`. Reading an id
   tells you what kind of thing it is.
4. **Ids are permanent and append-first.** hi never renumbers anything, refuses an id that is
   already taken, and keeps a retired id reserved, including one written somewhere hi cannot parse.
   Captures running at the same time all land rather than overwriting each other. It cannot stop
   you renumbering a file by hand, so permanence is a convention the tool supports rather than one
   it enforces; what it can do is refuse to be the one that breaks it.

There is no fifth rule about who the sentence speaks for. Notice that `SEND-1` says *I* and
`SPEND-1` says *an operator*, and that the difference is in the sentence, where anyone can read it.
On a product with a paying side and a using side, say which one you mean the way you would say it
out loud. On a product with one audience, do not: the ceremony costs four words and buys nothing.
hi will never read a subject off the front of your line, and `hi check` has no opinion on your
English. [DECISIONS.md](DECISIONS.md) §24 records why an earlier version of this was a rule and why
it is not one now.

The `## Intent` block is the part a spec can never carry, and it is the first thing an agent
should read.

Files in `hi/` are lowercase, because a family names its own file and hi lowercases it. An
uppercase name in there is hi's own rather than criteria: your first capture leaves a
`hi/AGENTS.md`, and a `hi/CLAUDE.md` beside it, describing the habit so an agent working in your
repository finds it without being told. hi writes them once and never again, and they are yours
afterwards. If a criterion ever ends up in one, `hi check` says so rather than letting it go quiet.

## Install

```bash
cargo install human-intent     # the command it installs is `hi`
```

Or take a binary from the [latest release](https://github.com/CorvidLabs/hi/releases) for Linux,
macOS (Intel or Apple Silicon) or Windows.

If you use [fledge](https://github.com/CorvidLabs/fledge), the same CLI is available as a plugin.
It is not bundled with fledge, so install it once, and then every `hi` command works as `fledge hi`:

```bash
fledge plugins install CorvidLabs/hi    # builds from source, so it needs cargo
fledge hi check
```

The crate is `human-intent` because the crate name `hi` is taken on crates.io by something
unrelated. A crate's name and its binary's name are independent, so `cargo install human-intent`
puts `hi` on your path. The command name is shared: several projects install a binary called `hi`, including
[PipeNetwork/hi](https://github.com/PipeNetwork/hi), which is a coding agent rather than anything
like this. If you already have one, installing ours shadows it, and you pick which wins on your
`PATH`. [DECISIONS.md](DECISIONS.md) §13 explains why we kept the name.

## Use

`hi` anchors to a repository: run it anywhere inside one and your criteria land at the top. It
needs no init and no config, and it creates `hi/` the first time you capture something.

```console
$ hi CHECKOUT-1 "As a shopper, I can pay without making an account"
INTENT.md  created, for the product-level why
hi/checkout.md  created
hi/checkout.md  +CHECKOUT-1

$ hi CHECKOUT-1.a "As a shopper, if my card is declined it tells me which field to fix"
hi/checkout.md  +CHECKOUT-1.a
```

Two files, not one. `hi/checkout.md` holds the feature. `INTENT.md` at the root holds the
product-level why, above any one feature, and it exists from the first capture rather than waiting
for you to discover it, because a product with criteria and no stated why is the common failure.
The prose in it is yours; hi only ever regenerates the feature list between its own markers. Until
you have written that why, `hi check` mentions it, as a note and never as a failure:

```console
$ hi check
2 criteria · 1 family · 1 file
note: INTENT.md has no product-level why yet
```

A new id just works. An id that is already taken refuses, and never overwrites it:

```console
$ hi CHECKOUT-1 "something else"
error: CHECKOUT-1 already exists in hi/checkout.md:14
hint:  next free is CHECKOUT-2
```

Reading it back gives you the file as a tree, one sentence per line, exactly as you wrote them.
This is `hi/chat.md` from the top of this page:

```console
$ hi ls
hi/chat.md
  SEND-1  I hit enter and the message shows up right away, marked as sending.
    SEND-1.a  If I have no connection it queues and tells me, and never silently disappears.
    SEND-1.b  If the thread was deleted before it sends, it warns me first.
  SEND-2  It reaches them and the mark changes to sent.
  RECEIPT-1  I can tell sent from read without thinking about it.
  SPEND-1  An operator can cap what the service spends in a day.
```

`hi export` hands an agent the same sentences as JSON with the `## Intent` prose attached,
`hi issue` shapes one into a ticket, and `hi view` puts the whole set on a page you can search and
filter. Your words are passed through untouched in all of them.

| Command | What it does |
|---|---|
| `hi <ID> <sentence>` | Capture. The family picks the file; a new family starts one. |
| `hi check` | Structural problems only. Exits 1 on a broken file, never on unfinished intent. |
| `hi ls [--family F] [--retired]` | Read what you have agreed to. |
| `hi retire <ID> [reason]` | Change your mind. Moves a criterion and its cases into `## Retired`. |
| `hi issue <ID> [--create]` | Print a ticket, or open a real GitHub issue with `gh`. |
| `hi export [FAMILY \| file]` | JSON for an agent, intent prose included. |
| `hi index` | Rewrite the feature list inside `INTENT.md`, and nothing else in it. |
| `hi view [--out FILE]` | One self-contained HTML page: a sticky feature rail, search with match highlighting, sort, keyboard navigation, and a copyable link for every id. Named after your `INTENT.md` heading, in CorvidLabs brand colors, light and dark. Works offline, and with scripting off it is still readable. |

Every command takes `--root <PATH>` to work on a repository other than the one you are standing in.
When you are capturing, put it before the id: everything after the id is your sentence, word for
word, so `--root` written after the id is just a word you typed.

### Changing your mind

`## Retired` is where a criterion goes when you decide against it, and `hi retire` is what puts it
there, so you never hand-edit the markdown to do it:

```console
$ hi retire SPEND-1 "the operator console is a separate product"
hi/chat.md  SPEND-1 retired

$ hi retire SEND-1
hi/chat.md  SEND-1 retired, with 2 of its cases
```

The reason is optional, and worth typing: it is written on its own line under what it explains, and
it is the only record of why the sentence stopped being true. Cases go with their parent, so nothing
is left orphaned behind it. The section is created if the file has none. The id is reserved forever
after this: capture refuses it, `hi issue` refuses to make work out of it, and `hi check` reports
any live criterion that tries to reuse it.

### The readable page

`hi view` writes `intent.html` into the repository root. It is a generated file, rewritten whole
every run, so put it in your `.gitignore` rather than committing a fresh copy of the page each time
a sentence changes. `--out FILE` puts it somewhere else, relative to the root, in a directory that
already exists.

## Where it sits

```
talk → hi (intent + criteria) → tickets (gh) → specs (agent → spec-sync) → code
```

Intent is written once, by a human. Everything downstream is generated from it:

```console
$ hi issue SEND-1 --create            # a ticket, with hi: SEND-1 as the permanent backlink
$ hi export SEND | claude -p "write the spec-sync module spec for this"
```

**Reach for hi upstream of whoever already decided the shape.** Writing intent for code that
already exists means reverse-engineering the want from the implementation, and you will feel the
pull the whole way. But a ticket written as a solution does exactly the same thing: someone who
tried this before any of the code existed reported the identical pull, sourced from the ticket
instead of the file. Most tickets are written as solutions.

Two things get easier when nothing has decided the shape yet. You can write a criterion you have no
idea how to implement, which an implementation never suggests. And absence becomes visible: writing
from something that already exists hides what is missing from it.

**Say in the prose if none of it is built yet.** hi records what was wanted, not what exists, so a
reader cannot tell a shipped criterion from a wish. One line at the top of the `## Intent` block
fixes that, and unlike a status field it cannot go stale without somebody reading the sentence that
is now wrong:

> Written before any of it was built, which is the point. None of this exists yet.

**Say who only when who matters.** *An operator can cap what the service spends in a day* and
*I can tell sent from read* are both fine. Naming a person on every line when the product has one
audience is filler, and naming nobody on a product with two sides loses the distinction that is
usually the whole reason two criteria disagree. Use the subject the sentence actually needs.

**Name the person, not the permission.** If your codebase says `admin`, the sentence probably wants
`an operator`. `admin` is a permission bit; an operator is someone with a job to do, and the job is
what the criterion is about.

## What it deliberately does not do

`hi` stores **no state**, tracks **no lifecycle**, binds **no evidence**, and **never fails a build
because a criterion is unproven**. It has no grammar rules on your sentence and no prose linter.

**The prose linter is the first thing everyone asks for, and the number is why there is not one.**
Requirements-smell detection, the published state of the art at flagging a vague or untestable
sentence, measures about **59% precision**. A linter built on it would be wrong two times in five.
Nobody argues with a tool that is right; they argue with the two, and after a week of arguing they
stop reading the output, and a month later somebody deletes it from CI. A checker you have learned
to ignore is worse than no checker, because it still looks like coverage. So the sentence stays
yours, and `hi check` never has an opinion about it.

**There is a test you can run on yourself, and it is free.** Try putting *As a ___,* in front of
your sentence. Someone using hi on a real product found that two of their forty criteria would not
take it, and that both were defective in a way they had not noticed: they had written a fact about
the system rather than anything anybody wants.

That test has no false positives, because nothing is guessing. You either can finish the sentence or
you cannot, and being unable to finish it means what you wrote was not a want. It is the check the
59% number says is impossible, and it costs nothing, because it happens in your head while you
type. hi does not ask you to leave the words in the file. An earlier version did, for four
releases; [DECISIONS.md](DECISIONS.md) §24 is why it no longer does.

Those are not omissions, they are the design. Every one of them is a thing you would have to
maintain, and a tool you maintain is a tool you stop writing in. The reasoning behind each is in
[DECISIONS.md](DECISIONS.md), and the docs are at
[corvidlabs.xyz/hi](https://corvidlabs.xyz/hi). [docs/ac-formats.html](docs/ac-formats.html)
surveys the acceptance-criteria formats the design came from: EARS, Volere, Gherkin,
OpenFastTrace, Kiro, spec-kit. GitHub shows that file as source, so save it and open it in a
browser. Read that page as history rather than documentation. It was written before any code
existed and argues for a stricter product than the one that shipped, with evidence bindings and a
lifecycle that were later cut; DECISIONS.md records why.

This is the one promise the format rests on, because the whole point of an id is that it can be
quoted somewhere hi will never see. Four ways hi's own verbs could quietly reuse one were found and
closed in 0.4.0, three of which `hi check` had reported as fine; [DECISIONS.md](DECISIONS.md) §26
records them and what they say about the design.

`hi check` fails on exactly six things, all structural: a duplicate id, a case with no parent, an
id that collides with a retired one, a line shaped like an id that is not a valid one, a family a
file never declared, and a criterion stranded outside every section where nothing would read it.

## Dogfooding

`hi` describes itself. Its own intent lives in [`hi/`](hi/), across 9 families in 6 files, and the
feature list in [`INTENT.md`](INTENT.md) at the root is generated by `hi index` with the count per
feature. `hi check` prints the total; this README deliberately does not restate it, because the
one number here that was maintained by hand is the one that went stale. The `## Intent` blocks in
those files are the honest version of why this exists, and
[corvidlabs.github.io/hi](https://corvidlabs.github.io/hi) is what `hi view` makes of them.

Dogfooding has a blind spot, and it is worth naming here. hi has one audience and one voice. A
product with an operator on one side and a member on the other has voices that actually contradict
each other, and that is a gap hi could never have found in its own files. It took someone using it
on their own product to find it, and the first fix we shipped for it was the wrong one. See
[DECISIONS.md](DECISIONS.md) §14 for what they found and §24 for the correction.

## Status

**v0.4.0** on [crates.io](https://crates.io/crates/human-intent), with binaries for Linux (x86_64
and arm64), macOS (Intel and Apple silicon) and Windows on the
[release page](https://github.com/CorvidLabs/hi/releases). The format is
deliberately not frozen: this is 0.x, and `HI/1` may still change before a 1.0 that commits to it,
as §24 just demonstrated by removing a rule that four releases had required.

```bash
brew install corvidlabs/tap/hi
```

**See it before installing it.** [corvidlabs.github.io/hi](https://corvidlabs.github.io/hi) is this
repository's own `hi view` output: the real criteria in [`hi/`](hi/), rendered by the real binary on
every push. Docs are at [corvidlabs.xyz/hi](https://corvidlabs.xyz/hi).
[CHANGELOG.md](CHANGELOG.md) has the history.

## License

MIT
