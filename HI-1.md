# HI/1

The compatibility contract a 1.0 of hi freezes. This file is the thing a
consumer, an adopter, and a later hi are invited to hold the binary to.
[DECISIONS.md](DECISIONS.md) is the reasoning; this is the freeze.

HI/1 is the only format version this binary reads or writes. A file that
declares any other `hi:` value is refused by every verb, by name, before
anything is written. A file that declares none is HI/1, so files written
before the key existed keep working.

**Permanence of an id is a convention over shared history: the merged tree,
not an unmerged branch.** Two workstreams can each choose the same
hand-chosen id against the tree they captured on, and git will merge both
without a conflict marker. hi's own verbs refuse to be the one that breaks
the convention; `hi check` on the merged tree is the thing that proves it
([DECISIONS.md](DECISIONS.md) §26, §37).

## Frozen

A 1.0 of this binary, once tagged, commits to all of the following. Changing
any of them is a new format version, a new export envelope version, or a 2.0
of the tool — named in each row.

| What | Frozen as | Moves only with |
|---|---|---|
| The file format | The grammar, sections, and frontmatter below | `hi: 2` |
| The seven `hi check` kinds | The codes in [Check kinds](#check-kinds) | 2.0 of hi |
| The check policy | Structural problems only; unfinished intent is never an error | 2.0 of hi |
| Exit-code semantics | [Exit codes](#exit-codes) | 2.0 of hi |
| The export envelope | [Export envelope](#export-envelope) at `"export": 1` | `"export": 2` |
| The one promise | hi's own verbs never reuse an id, and a criterion they reported as saved is readable in the section they named | 2.0 of hi |
| Capture's write-once of `hi/AGENTS.md` | Capture writes the file when it is absent and never again | 2.0 of hi |
| `hi seed`'s recognition | Byte identity after folding a BOM and CRLF, against the templates this binary ships | a new template in a 1.x is added to the known set; the rule does not move |

What a 1.0 is **not** freezing:

- The wording of a note, a hint, or a help string. The *codes* of notes are frozen (`no-product-why`, `index-behind`, `index-markers`, `unexplained-retirement`); the sentences beside them are not.
- The wording of `hi/AGENTS.md`. `hi seed` exists so that wording can move without a format version.
- The HTML chrome of `hi view`. The page is derived and gitignored.
- The crate version, the binary's `--version`, and the presence of additional verbs that do not change the format, the kinds, or the envelope.
- Anything [DECISIONS.md](DECISIONS.md) §5, §9 and §24 already refused: state, lifecycle, evidence bindings, a prose linter, a role prefix, checkboxes, status fields, or failing a build because a criterion is unproven.

An eighth check kind is a 2.0, not because seven is a magic number, but because `Kind` is a closed set a consumer is invited to match on, and because 1.0 freezes both the members and the policy that those members are structural. The cardinality was never the contract; the policy is. Seven is the membership that policy had when it froze.

## The file

A criteria file is markdown a person could have typed. Nothing below the
frontmatter exists to serve the machine.

```markdown
---
hi: 1
families: [SEND, RECEIPT]
owner: leif
---

# Chat

## Intent

One paragraph is one line. A blank line is the only break.

## Criteria

- **SEND-1**  A criterion is one list item: a bold id, two spaces, a sentence.
  - **SEND-1.a**  A case is indented two spaces per level under its parent.
- **SEND-2**  A sibling is another list item, not a continuation.

## Retired

- **SEND-3**  A retired id stays reserved.
  retired: the reason, on the line under it
```

Normative:

1. **Frontmatter.** A YAML block opening the file. `hi:` is the format version; absent, empty, or `1` is HI/1, anything else is unreadable. `families:` is the list of id families this file claims; inline (`[SEND, RECEIPT]`) or a YAML block list, both legal. `owner:` is prose, unread.
2. **`## Intent`.** Human prose. A fenced block in it is prose, so an example of the format is not criteria.
3. **`## Criteria`.** Live criteria. One criterion is one markdown list item, one line, however long the sentence runs. The parser also accepts a bare `ID  sentence` line with no bullet and no bold, so a hand-typed file is still read.
4. **`## Retired`.** Criteria the author changed their mind about. The id stays spoken for.
5. **Ids.** `FAMILY-1`, then letters for cases and numbers for steps, alternating strictly: `SEND-1.a.1.b`. A family starts with `A-Z` and continues with `A-Z`, `0-9` and `_`; a letter level is one or more of `a-z`; a number level is a decimal integer that fits in 32 bits. A leading zero on a multi-digit level is not an id (`SEND-007` is refused, not normalised to `SEND-7`).
6. **A criteria file is lowercase `*.md` directly inside `hi/`.** An uppercase name in `hi/` is hi's own (`AGENTS.md`, `CLAUDE.md`) and is not read as criteria.

hi itself always writes the list form, LF or the file's existing endings, and the frontmatter style the file already had. It never rewrites, reflows, or reformats prose it did not write.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The command did what it was asked. `hi check` with only notes is 0. A capture that stored its criterion is 0 even if a best-effort side write (`INTENT.md`, `hi/AGENTS.md`) could not happen. |
| 1 | The command could not. A structural problem (`hi check`), an operational failure (unreadable file, unknown `hi:` version, lock not taken), or a refusal (id taken, case with no parent, `hi seed` of an edited file). Stderr names the reason. Nothing is written on a refusal. |
| 2 | Usage. clap could not parse the arguments. Not a hi finding. |

`hi check --json` writes the `Report` to stdout and uses the same 0/1 rule: `Report::ok()` is true exactly when `problems` is empty, and notes are not problems.

## Check kinds

Seven codes, and no others. Each is a structural problem inside a file hi read. An unreadable file, an unknown format version, and a lock hi could not take are operational failures (exit 1, no `Kind`), not members of this list.

| Code | When |
|---|---|
| `duplicate-id` | The same id is written more than once, live or retired, in any file. |
| `orphan-case` | A case's parent is not in the same file. |
| `retired-collision` | A live criterion reuses an id that is in `## Retired`. |
| `unparseable-id` | A line is shaped like an id and is not a valid one. |
| `undeclared-family` | A criterion uses a family the file's frontmatter does not list. |
| `stray-criterion` | A criterion-shaped line sits outside `## Criteria` and `## Retired`, or in a file hi does not read as criteria. |
| `duplicate-family` | Two files both list the same family in frontmatter. A family listed twice in *one* file is the same declaration written twice, not two homes. |

`hi check` reports every problem in every file it could read, names the file and the 1-based line, and exits 1 if there is any. Incomplete intent is the normal state of intent and is never one of these.

Notes (`no-product-why`, `index-behind`, `index-markers`, `unexplained-retirement`) are not kinds, never move the exit code, and travel in `--json` as a list of `{ "kind", "message" }`.

## Export envelope

`hi export` writes pretty-printed JSON. `"hi"` is the format version of the files the payload was built from. `"export"` is the version of this envelope. They are both `1` and they are free to move apart; one field could never later mean both.

```json
{
  "hi": 1,
  "export": 1,
  "scope": "repo",
  "product": "the prose from INTENT.md, when the scope is the whole repository",
  "files": [
    {
      "file": "hi/chat.md",
      "title": "Chat",
      "intent": "the ## Intent prose, verbatim",
      "families": ["SEND", "RECEIPT"],
      "criteria": [
        {
          "id": "SEND-1",
          "text": "the sentence as written",
          "depth": 1,
          "parent": null
        }
      ],
      "retired": [
        {
          "id": "SEND-3",
          "text": "the sentence as written",
          "depth": 1,
          "parent": null,
          "retired": "the reason, when there is one"
        }
      ]
    }
  ]
}
```

Normative for `"export": 1`:

- Every payload has `hi`, `export`, `scope`, and `files`. `product` is present only on a whole-repository export, and omitted rather than null when absent.
- `scope` is `"repo"`, a family name, or a file stem, matching the argument that was given.
- `file` uses forward slashes on every platform.
- `title` is omitted when the file has no `# ` heading.
- `criteria` are the live ones; `retired` are the ones under `## Retired`. `retired` on a criterion object is the reason string and is omitted when there is none.
- `depth` is 1 for a top-level id. `parent` is the parent id as a string, or `null` at depth 1.
- Criterion `text` is the sentence as written, not reflowed.

A scoped export is this payload with less in it, not a different shape. Additive fields bump `"export"`; removing or renaming a field is a breaking envelope change.

## The promise

hi never reuses an id, and never lets one of its own verbs reuse one. After every capture or retire:

- every id the verb reported as saved is readable by `Workspace::load` in the section the verb named;
- the shape — id, section, sentence, retirement reason — of every criterion the verb did not name is unchanged;
- no id is assigned to a second sentence.

`hi check` exiting 0 implies all three, over the tree it was run against. That tree is shared history: the files as merged, not as they stood on an unmerged branch. An id is only unique against the tree it was captured on ([DECISIONS.md](DECISIONS.md) §37).

The files are markdown and they are yours. Nothing stops a person renumbering one in an editor. Permanence is a convention the tool supports rather than one it enforces; what it can do is refuse to be the one that breaks it.

## `hi/AGENTS.md`

hi writes this file on the first capture, when it is absent, and never again on that path. That is the property that stops a surprise rewrite of a file somebody edited.

`hi seed` is the verb that migrates it:

| The file is | `hi seed` does |
|---|---|
| Missing | Writes the current text, and `hi/CLAUDE.md` beside it. |
| Byte-identical to a template this hi has shipped, after folding a BOM and CRLF | Rewrites it to the current text, keeping the endings the file had. |
| Already the current text | Says so, writes nothing. |
| Anything else | Refuses, exit 1, writes nothing. Delete it and run `hi seed` if the current text is what you want. |

Capture still only writes the file when it is absent. The path that runs on every thought never overwrites.

A sentence earns a place in the file only if it is a habit rather than a fact about the format, and only if the one thing it names is something hi has committed not to change ([DECISIONS.md](DECISIONS.md) §38).
