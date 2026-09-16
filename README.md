<div align="center">

# hi

**Human Intent; acceptance criteria in human words, with ids that never move.**

Rust · single binary · no network, no model, no API key

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

- **SEND-1**  As a member, I hit enter and the message shows up right away, marked as sending.
  - **SEND-1.a**  As a member, if I have no connection it queues and tells me, and never silently disappears.
  - **SEND-1.b**  As a member, if the thread was deleted before it sends, it warns me first.
- **SEND-2**  As a member, it reaches them and the mark changes to sent.

- **RECEIPT-1**  As a member, I can tell sent from read without thinking about it.

- **SPEND-1**  As an operator, I can cap what the service spends in a day.

## Retired

- **SEND-3**  As a member, my messages auto-delete after 24 hours.
  retired: we decided this was a different product
```

That is the whole format. Five rules:

1. **A criterion is one markdown list item: a bold id, two spaces, and a sentence.** However long the
   sentence runs it stays on one line, so criteria stay greppable and diffable. And because it is
   a list item, it renders as its own line everywhere, instead of markdown joining it into a
   paragraph with its neighbours.
2. **Every criterion is role-play, and the role comes first.** You write `As a member,` or
   `As an operator,` and then the sentence. A role is a short noun phrase, at most four words,
   followed by a comma. There is no new syntax and nothing to configure: because the shape is
   universal, hi reads the role off the front of the sentence, and `hi ls`, `hi export`, `hi issue`
   and `hi view` all carry it through.
3. **You write the id yourself**, because you are the one who has to say it out loud. `SEND-1` is a
   name, not a position.
4. **Letters are cases, numbers are steps**, alternating strictly: `SEND-1.a.1.b`. Reading an id
   tells you what kind of thing it is.
5. **Ids are permanent and append-first.** hi never renumbers anything, refuses an id that is
   already taken, and keeps a retired id reserved. It cannot stop you renumbering a file by hand,
   so permanence is a convention the tool supports rather than one it enforces.

Rule 2 is the one that looks optional and is not. Drop the roles from the file above and `SEND-1`
and `SPEND-1` become the same undifferentiated *I*, so a reader cannot tell that one of them is
being paid and the other is doing the paying. That distinction is usually the whole reason the two
criteria disagree, and there is nowhere else in the file to put it.

Nothing enforces it. A sentence that does not open with a role still parses, and `hi check` will
not say a word about it, because judging your English is not hi's job. You simply lose the answer
to *who wants this*. [DECISIONS.md](DECISIONS.md) §14 records why this became a rule.

The `## Intent` block is the part a spec can never carry, and it is the first thing an agent
should read.

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
$ hi CHECKOUT-1 "As a shopper, something else"
error: CHECKOUT-1 already exists in hi/checkout.md:14
hint:  next free is CHECKOUT-2
```

Reading it back puts the role in front, where you cannot miss which side of the product is talking.
This is `hi/chat.md` from the top of this page:

```console
$ hi ls
hi/chat.md
  SEND-1  [member] I hit enter and the message shows up right away, marked as sending.
    SEND-1.a  [member] if I have no connection it queues and tells me, and never silently disappears.
    SEND-1.b  [member] if the thread was deleted before it sends, it warns me first.
  SEND-2  [member] it reaches them and the mark changes to sent.
  RECEIPT-1  [member] I can tell sent from read without thinking about it.
  SPEND-1  [operator] I can cap what the service spends in a day.
```

`hi export` carries the role as its own JSON field beside the sentence, `hi issue` opens the ticket
body with *Speaking as operator.*, and `hi view` carries it onto the page. A sentence with no role
is passed through exactly as written, everywhere.

| Command | What it does |
|---|---|
| `hi <ID> <sentence>` | Capture. The family picks the file; a new family starts one. |
| `hi check` | Structural problems only. Exits 1 on a broken file, never on unfinished intent. |
| `hi ls [--family F] [--retired]` | Read what you have agreed to. |
| `hi retire <ID> [reason]` | Change your mind. Moves a criterion and its cases into `## Retired`. |
| `hi issue <ID> [--create]` | Print a ticket, or open a real GitHub issue with `gh`. |
| `hi export [FAMILY \| file]` | JSON for an agent, intent prose included. |
| `hi index` | Rewrite the feature list inside `INTENT.md`, and nothing else in it. |
| `hi view [--out FILE]` | One self-contained HTML page of the intent, for people who do not read markdown. |

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

**Name the person, not the permission.** If your codebase says `admin`, the role is still probably
`operator`. `admin` is a permission bit; an operator is someone with a job to do. The person is
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

**The role prefix turned out to do the job anyway.** Someone using hi on a real product found that
two of their forty criteria would not take a role, and that both were defective in a way they had
not noticed: they could not write "As a ___" in front of them because they had written a fact about
the system rather than anyone's want.

That test has no false positives, because nothing is guessing. You either can finish the sentence or
you cannot, and being unable to finish it means what you wrote was not a want. It is the check that
the 59% number says is impossible, and it costs three words at the front of a line instead of a
dictionary and a CI job.

Those are not omissions, they are the design. Every one of them is a thing you would have to
maintain, and a tool you maintain is a tool you stop writing in. The reasoning behind each is in
[DECISIONS.md](DECISIONS.md). [docs/ac-formats.html](docs/ac-formats.html) surveys the
acceptance-criteria formats the design came from: EARS, Volere, Gherkin, OpenFastTrace, Kiro,
spec-kit. There is no docs site yet, so GitHub shows that file as source; save it and open it in a
browser. Read that page as history rather than documentation. It was written before any code
existed and argues for a stricter product than the one that shipped, with evidence bindings and a
lifecycle that were later cut; DECISIONS.md records why.

`hi check` fails on exactly six things, all structural: a duplicate id, a case with no parent, an
id that collides with a retired one, a line shaped like an id that is not a valid one, a family a
file never declared, and a criterion stranded outside every section where nothing would read it.

## Dogfooding

`hi` describes itself. Its own intent lives in [`hi/`](hi/), currently 102 criteria across 9
families in 6 files, every one of them in a role, and the feature list in `INTENT.md` at the root is
generated by `hi index`. The `## Intent` blocks in those files are the honest version of why this
exists.

Dogfooding has a blind spot, and it is worth naming here. hi has one audience, so most of its own
criteria speak as *a person writing intent* and the rest as *a reader*, *a maintainer* or *an
agent*. A product with an operator on one side and a member on the other has voices that actually
contradict each other, and that is a gap hi could never have found in its own files. It took
someone using it on their product to find it. See [DECISIONS.md](DECISIONS.md) §14.

## Status

**v0.1.0.** On [crates.io](https://crates.io/crates/human-intent), with binaries for Linux, macOS
and Windows. The format is deliberately not frozen: this is 0.x, and `HI/1` may still change before
a 1.0 that commits to it.

The role-play voice, `hi retire` and the `INTENT.md` note landed after v0.1.0 and are not in the
published crate yet, so build from `main` for those. [CHANGELOG.md](CHANGELOG.md) lists them under
Unreleased.

## License

MIT
