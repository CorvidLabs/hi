---
spec: id.spec.md
---

## User Stories

- As someone writing criteria, I want to choose the id myself so that I can say it out loud in a
  standup a year from now and still mean the same line (hi: ID-2, ID-1)
- As someone reading a hi file, I want the shape of an id to tell me whether a level is a case or a
  step, without a legend (hi: ID-3)
- As someone mistyping an id, I want hi to say which level is wrong and what it expected there,
  rather than quietly accepting a shape that will not survive (hi: ID-4)
- As `hi check`, I want a line that is *shaped* like a criterion but is not a valid id to still be
  recognized as an intended criterion, so I can report it at its file and line instead of reading it
  as prose (hi: CHECK-2.d, CHECK-3)
- As `hi capture`, I want to derive a file, a parent, and the next free number from an id alone, so
  a brand new id just works and an existing one refuses with a usable hint (hi: CAPTURE-2,
  CAPTURE-3, CAPTURE-4)
- As `hi issue` and `hi export`, I want a criterion's cases and its tree position to fall out of the
  id, so a ticket and an agent payload are the whole picture without stored structure (hi: ISSUE-3,
  EXPORT-3)
- As a maintainer, I want the id module to stay pure and dependency-free so nothing about identity
  can drift with I/O, configuration, or state

## Acceptance Criteria

- `Id::parse` accepts exactly `FAMILY-level(.level)*` where the family matches `[A-Z][A-Z0-9_]*`,
  odd-depth levels are ASCII digits, and even-depth levels are ASCII lowercase letters
- Every rejection is a named `IdError` variant whose `Display` sentence identifies the offending
  family, level text, or depth
- A parsed id renders back through `Display` to the string it came from, for every id `parse`
  accepts; a zero-padded number is refused outright rather than normalized
- `parent`, `is_descendant_of`, `depth`, and `root_number` are derived from `levels` alone and never
  consult a file, a workspace, or any stored state
- `looks_like_id` accepts every string `Id::parse` accepts, plus id-shaped strings that `parse`
  rejects (a wrongly cased family such as `send-2`, a padded or malformed level such as `SEND-007`),
  and rejects ordinary prose words such as `Given`, `I`, `well-formed`, and `spec-sync`, because the
  first level must start with a digit
- The module compiles against `std::fmt` only, with no sibling module, no external crate, and no I/O

### REQ-id-001

`Id::parse` SHALL accept a string of the form `FAMILY-level(.level)*`, splitting the family from the
level path at the first hyphen, and SHALL return a structured `Id` carrying the family and the
ordered levels.

Acceptance Criteria

- `SEND-1` parses to `family == "SEND"` and `levels == [Number(1)]` (hi: ID-2).
- `SEND-1.a.1.b` parses to `[Number(1), Letter("a"), Number(1), Letter("b")]`.
- The split is at the **first** hyphen, so `SE-ND-1` is read as family `SE` with a level `ND-1` and
  fails as `BadLevel`, not as `BadFamily`.
- A token with no hyphen at all (`SEND1`) fails as `MissingHyphen`.
- Nothing after the hyphen (`SEND-`) fails as `NoLevels`.

### REQ-id-002

A family SHALL match `[A-Z][A-Z0-9_]*`, and any other family text SHALL be rejected as
`IdError::BadFamily` carrying the offending family.

Acceptance Criteria

- `SEND_2FA-1` parses: digits and underscores are legal after the leading uppercase letter.
- `send-1` and `1SEND-1` are rejected as `BadFamily`.
- An empty family (the token begins with `-`) is rejected as `BadFamily`.
- The error sentence names the family and states the charset.

### REQ-id-003

Levels SHALL alternate strictly by 1-based depth (odd depths a number, even depths a letter), and a
level of the wrong kind for its depth SHALL be rejected as `IdError::Alternation` carrying that depth
and what was expected (hi: ID-3, ID-4).

Acceptance Criteria

- A number level is a step or detail inside its parent (hi: ID-3.b); a letter level is another case
  of its parent (hi: ID-3.a).
- `SEND-a` fails with `depth: 1, expected: "a number"`, because the first level is always a number.
- `SEND-1.a.b` fails with `depth: 3, expected: "a number"`: a case of a case must be expressed as a
  step containing cases.
- The error sentence restates the rule: "levels alternate number, letter, number, letter".

### REQ-id-004

A level SHALL be either all ASCII digits parsed as `u32` or all ASCII lowercase letters, and anything
else SHALL be rejected as `IdError::BadLevel`, `IdError::EmptyLevel`, or, for a zero-padded digit
level, `IdError::PaddedLevel` (REQ-id-011).

Acceptance Criteria

- A mixed or uppercase level (`SEND-1a`, `SEND-1.A`) fails as `BadLevel` carrying the level text.
- A doubled or trailing dot (`SEND-1..a`, `SEND-1.`) fails as `EmptyLevel`.
- A digit level wider than `u32::MAX` fails as `BadLevel` rather than panicking or truncating.
- A letter level may be more than one character long, because `Level::Letter` holds a `String`.

### REQ-id-005

`Display` SHALL render an id back to its source text, so an id printed by hi is an id hi can parse
(hi: ID-1).

Acceptance Criteria

- `Id::parse(s)?.to_string() == s` for every accepted `s`, with no exceptions.
- `Level` renders as the bare number or the bare letters, and `Id` joins them with `.` after
  `FAMILY-`.
- The one input that would have broken the round trip, a zero-padded number, is refused by
  `parse` rather than normalized, so `Display` never renames an id (REQ-id-011, hi: ID-1.c).
- Rendering is the only formatting this module does; it never rewrites, reflows, or reorders
  anything (hi: ID-1.a, FILE-4).

### REQ-id-006

`Id::parent` SHALL return the id one level up with the family preserved, and SHALL return `None` for
a top-level criterion.

Acceptance Criteria

- `SEND-1.a.1` has parent `SEND-1.a`, whose parent is `SEND-1`.
- `SEND-1` has no parent, which is how `capture` knows a top-level criterion needs no parent to hang
  off (hi: CAPTURE-4) and how `check` knows not to report it as an orphan (hi: CHECK-2.b).
- `parent` never mutates the receiver; it clones the levels and pops one.
- Repeated `parent` calls terminate at depth 1.

### REQ-id-007

`Id::is_descendant_of` SHALL be true only when the two ids share a family, the receiver is strictly
deeper, and the receiver's leading levels equal the other id's levels.

Acceptance Criteria

- `SEND-1.a` and `SEND-1.a.1` are descendants of `SEND-1`; that relation is what puts a criterion's
  cases in its ticket body (hi: ISSUE-3) and under its parent on insertion (hi: CAPTURE-4).
- `SEND-2` and `RECEIPT-1.a` are not descendants of `SEND-1`.
- An id is not its own descendant.
- `SEND-11` is not a descendant of `SEND-1`, because the comparison is over `Level` values, not over
  the rendered text.

### REQ-id-008

`Id::depth` SHALL report the number of levels, and `Id::root_number` SHALL report the top-level
number when there is one.

Acceptance Criteria

- A top-level criterion is depth 1, and `depth() == parent().depth() + 1` for any id with a parent.
- `depth` drives list indentation, ticket-case indentation, and the `depth` field of the export
  payload, so the payload shape is the same at every scope (hi: EXPORT-3).
- `root_number` returns `Some(n)` when the first level is a `Number`, which is how `next_free`
  suggests the next id in a family after a refusal (hi: CAPTURE-3).
- `root_number` returns `None` when the first level is not a number or the id has no levels. Both
  states are reachable only for an `Id` built without `parse`.

### REQ-id-009

`looks_like_id` SHALL recognize id-shaped tokens without committing to their validity, so a
malformed id is reported rather than silently read as prose (hi: CHECK-2.d). A token is id-shaped
when its family is non-empty, starts with an ASCII letter of either case, and is otherwise ASCII
alphanumeric or `_`, and when the text after the first hyphen starts with an ASCII digit.

Acceptance Criteria

- `SEND-1` and `SEND-1.a` are id-shaped.
- `send-2`, `Send-3`, `SEND-007`, `SEND-1a`, and `SEND-1.a.b` are id-shaped although `Id::parse`
  rejects them, so `doc` records the `IdError` and `check` reports it at its file and line
  (hi: CHECK-3). The family test is case-insensitive precisely so a lowercase id is refused with a
  reason instead of vanishing into prose.
- `I`, `Given`, `well-formed`, `spec-sync`, and `co-authored` are not id-shaped, so ordinary prose in
  a `## Criteria` section is never captured as a criterion. The first-level digit rule is the whole
  of that distinction.
- `SEND-a`, `SEND-a.b`, and `1ST-4` are **not** id-shaped either, so an alternation break at depth 1
  and a family that starts with a digit are read as prose and never reported by `check`. That is the
  accepted cost of the previous bullet, recorded in the spec as invariant 13; those errors surface
  only when the id arrives as a command-line argument.
- The predicate does not call `valid_family`; it writes its own family test, which differs from
  `parse`'s only in accepting either case.
- `doc` applies the same predicate outside every section, so a criterion-shaped line stranded above
  `## Criteria` is recorded as a stray and reported rather than silently vanishing (hi: CHECK-2.e),
  while a line inside a fenced code block never reaches the predicate at all (hi: FILE-9).
- `main` uses the same predicate to route an id-shaped first argument to capture before clap parses,
  so writing a thought stays one command with no setup (hi: CAPTURE-1). It tests the argument tail
  left after `--root` is peeled off, so a flag is never mistaken for an id (hi: CAPTURE-8), and only
  an argument that is valid UTF-8, so a non-UTF-8 argument is a plain error and not a panic
  (hi: CAPTURE-1.c).

### REQ-id-010

The module SHALL remain pure: no I/O, no stored state, no configuration, and no dependency beyond
`std::fmt`.

Acceptance Criteria

- The only `use` in the file is `std::fmt`.
- No function reads or writes a file, an environment variable, or a global.
- Nothing in the module renumbers, reserves, retires, or allocates an id; permanence is a property
  of the text a human wrote, which this module only reads (hi: ID-1, ID-1.a, ID-1.b).
- Parsing the same string twice always produces the same result.

### REQ-id-011

A numeric level of more than one digit that begins with `0` SHALL be rejected as
`IdError::PaddedLevel` carrying the offending level text, so that a padded spelling and its unpadded
spelling can never be two names for one line (hi: ID-1.c).

Acceptance Criteria

- `SEND-007` and `SEND-1.a.01` are rejected as `PaddedLevel`; neither is normalized to `SEND-7` or
  `SEND-1.a.1`.
- The check runs before the `u32` parse, so the padded text is still available to name in the error.
- The error sentence names the unpadded spelling to write instead:
  "level '007' has a leading zero; write it as '7', so the id always means the same thing".
- `SEND-0` parses as `Number(0)`: a single zero is a number, not padding. `SEND-7` and `SEND-10`
  parse unchanged.
- `looks_like_id("SEND-007")` is still `true`, so a padded id already written in a file is reported
  by `check` at its file and line rather than read as prose (hi: CHECK-2.d).
- Capture refuses a padded id and writes nothing, which is the same refusal path as any other
  malformed id (hi: CAPTURE-5).

## Constraints

- The grammar is frozen by ID-1: an id already written down must keep parsing to the same value
  forever. Widening the grammar is possible; narrowing it retroactively invalidates files people
  already have. `PaddedLevel` is a deliberate narrowing, taken because the old behavior did not
  satisfy ID-1 in the first place: `SEND-007` parsed to `SEND-7`, so the id a person wrote down was
  not the id hi meant. A file that already contains a padded id now reports as a `check` problem at
  its file and line instead of silently resolving to a different criterion.
- `Id.family` and `Id.levels` are public fields, and `capture` constructs an `Id` directly to build
  its "next free" hint. Making them private is a breaking change to a sibling module.
- `Level::Number` is a `u32`, so the addressable range of a top-level criterion is bounded by
  `u32::MAX`; an overflow is a `BadLevel`, never a panic.
- `Id` derives `PartialEq`, `Eq`, and `Hash` but not `Ord`, so callers cannot sort ids directly.
  `Level` does derive `Ord`.
- `IdError::Alternation::expected` is a `&'static str` written into the user-facing sentence;
  changing its wording changes `hi check` output that tests assert on.
- `looks_like_id`'s first-level-digit rule is what tells an id from an ordinary hyphenated word, and
  it is load-bearing for every hi file that contains the words `spec-sync` or `well-formed`.
  Loosening it turns prose into criteria; keeping it means `SEND-a` and `1ST-4` cannot be reported
  from a file. Neither side of that trade is free, and the rule as written chooses silence on
  malformed ids over noise on ordinary prose.
- Everything is ASCII by construction: `is_ascii_uppercase`, `is_ascii_digit` and
  `is_ascii_lowercase` in `parse`, plus `is_ascii_alphabetic` and `is_ascii_alphanumeric` in
  `looks_like_id`. No Unicode case folding or normalization is performed, so a family written in
  non-ASCII letters is not id-shaped and is read as prose.
- Depth is unbounded by the grammar; alternation is the only structural limit.

## Out of Scope

- Uniqueness, collisions, retired-id reuse, and orphan detection across a workspace: that is
  `check` (hi: CHECK-2.a, CHECK-2.b, CHECK-2.c).
- Resolving an id to a file or a family's frontmatter, and finding the next free number. That is
  `workspace`.
- Reading criterion lines out of markdown, continuation lines, and `retired:` notes, which is `doc`.
- Deciding where a new criterion is written and refusing an id that already exists. That is
  `capture`.
- Rendering ids into lists, tickets, JSON payloads, or the `INTENT.md` index is `out`'s job.
- Judging the sentence attached to an id. hi never lints prose, and this module never sees it.
- Any notion of status, lifecycle, evidence, or whether a criterion is met. hi holds intent and
  identity only.
