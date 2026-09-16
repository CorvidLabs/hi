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

## The format in 60 seconds

`hi/chat.md`:

```markdown
---
hi: 1
families: [SEND, RECEIPT]
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

## Retired

- **SEND-3**  Messages auto-delete after 24 hours.
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
   already taken, and keeps a retired id reserved. It cannot stop you renumbering a file by hand,
   so permanence is a convention the tool supports rather than one it enforces.

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
$ hi CHECKOUT-1 "I can pay without making an account"
hi/checkout.md  created
hi/checkout.md  +CHECKOUT-1

$ hi CHECKOUT-1.a "if my card is declined it tells me which field to fix"
hi/checkout.md  +CHECKOUT-1.a
```

A new id just works. An id that is already taken refuses, and never overwrites it:

```console
$ hi CHECKOUT-1 "something else"
error: CHECKOUT-1 already exists in hi/checkout.md:14
hint:  next free is CHECKOUT-2
```

| Command | What it does |
|---|---|
| `hi <ID> <sentence>` | Capture. The family picks the file; a new family starts one. |
| `hi check` | Structural problems only. Exits 1 on a broken file, never on unfinished intent. |
| `hi ls [--family F] [--retired]` | Read what you have agreed to. |
| `hi issue <ID> [--create]` | Print a ticket, or open a real GitHub issue with `gh`. |
| `hi export [FAMILY \| file]` | JSON for an agent, intent prose included. |
| `hi index` | Rewrite the feature list inside `INTENT.md`, and nothing else in it. |
| `hi view [--out FILE]` | One self-contained HTML page of the intent, for people who do not read markdown. |

Every command takes `--root <PATH>` to work on a repository other than the one you are standing in.
When you are capturing, put it before the id: everything after the id is your sentence, word for
word, so `--root` written after the id is just a word you typed.

## Where it sits

```
talk → hi (intent + criteria) → tickets (gh) → specs (agent → spec-sync) → code
```

Intent is written once, by a human. Everything downstream is generated from it:

```console
$ hi issue SEND-1 --create            # a ticket, with hi: SEND-1 as the permanent backlink
$ hi export SEND | claude -p "write the spec-sync module spec for this"
```

## What it deliberately does not do

`hi` stores **no state**, tracks **no lifecycle**, binds **no evidence**, and **never fails a build
because a criterion is unproven**. It has no grammar rules on your sentence and no prose linter.

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

`hi` describes itself. Its own intent lives in [`hi/`](hi/), currently 89 criteria across 8
families in 5 files, and `INTENT.md` at the root is generated by `hi index`. The `## Intent` blocks
in those files are the honest version of why this exists.

## Status

**v0.1.0.** On [crates.io](https://crates.io/crates/human-intent), with binaries for Linux, macOS
and Windows. The format is deliberately not frozen: this is 0.x, and `HI/1` may still change before
a 1.0 that commits to it.

## License

MIT
