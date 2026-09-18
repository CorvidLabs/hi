---
module: id
version: 1
status: active
files:
  - src/id.rs

db_tables: []
depends_on: []
---

# Id

## Purpose

Owns the criterion-id grammar: the one thing in hi that is strict. An id is a hand-written family,
a hyphen, and a dotted path of levels that alternate number, letter, number, letter
(`SEND-1`, `SEND-1.a`, `SEND-1.a.1`, `SEND-1.a.1.b`). The module turns an id string into a value
that can be compared, hashed, rendered back, asked for its parent, and asked whether it sits
beneath another id. It also names precisely why a string is not an id.

It exists because an id that moves is worse than no id at all (hi: ID-1). Everything else in hi
reads identity through this module: capture resolves a file from `Id::parent` first and `Id::family`
second (hi: CAPTURE-4.a), check finds orphan cases through `Id::parent`, doc separates criterion
lines from prose through `looks_like_id` (reached through its own `is_criterion_line`, which strips
a bullet and any emphasis first), export publishes `depth` and `parent` as the payload's tree shape,
and view indents a criterion by `depth`.

This module is pure. It performs no I/O, holds no state, allocates no files, and never renumbers,
reorders, or rewrites anything. It only answers questions about a string (hi: ID-1.a).

## Public API

| Export | Description |
|--------|-------------|
| `Level` | One level of an id's dotted path: `Number(u32)` for a step or detail, `Letter(String)` for a case or branch. Derives `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`; hand-written `Display` renders the bare number or letters with no separator. |
| `Id` | A parsed criterion id: `pub family: String` plus `pub levels: Vec<Level>`. Derives `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` (not `Ord`); hand-written `Display` renders `FAMILY-l1.l2.l3`. Both fields are public, so a caller may build an `Id` directly without going through `parse`. |
| `IdError` | Why a string is not a valid id: `MissingHyphen`, `BadFamily(String)`, `NoLevels`, `EmptyLevel`, `BadLevel(String)`, `PaddedLevel(String)`, `Alternation { depth, expected }`. Derives `Debug`, `Clone`, `PartialEq`, `Eq`; hand-written `Display` writes the human sentence hi prints, and `PaddedLevel`'s sentence names the unpadded spelling to use instead. It does not implement `std::error::Error`; callers wrap it in an `anyhow` message instead. |
| `parse` | `Id::parse(raw: &str) -> Result<Id, IdError>`. Splits at the first `-`, validates the family charset, then reads the dot-separated levels enforcing strict alternation. The only constructor that guarantees the grammar. |
| `parent` | `Id::parent(&self) -> Option<Id>`. The id one level up, keeping the family; `None` at depth 1 or below. |
| `is_descendant_of` | `Id::is_descendant_of(&self, other: &Id) -> bool`. True when `self` sits strictly beneath `other`: same family, deeper, and level-for-level equal over `other`'s prefix. Never true for an id against itself. |
| `depth` | `Id::depth(&self) -> usize`. How many levels the id has. A top-level criterion is depth 1. |
| `root_number` | `Id::root_number(&self) -> Option<u32>`. The top-level number, used to find the next free id in a family; `None` when the first level is not a `Number`. |
| `looks_like_id` | `looks_like_id(token: &str) -> bool`. True when a token is shaped like an id: a non-empty family that starts with an ASCII letter of **either case** and continues in ASCII alphanumerics or `_`, a hyphen, and a remainder whose first character is an ASCII digit. It never commits to the id being valid. It is looser than `parse` on family case and on level text, so `send-2` and `SEND-007` are id-shaped; it is tighter than "a hyphen and anything" on the first level, so `spec-sync` and `well-formed` are ordinary words. |

### Structs & Enums

| Type | Description |
|------|-------------|
| `Level` | Enum. `Number(u32)` is a step or detail and occupies odd depths (the first level included); `Letter(String)` is a case or branch and occupies even depths (hi: ID-3.a, ID-3.b). `Letter` holds a `String`, not a `char`, so a multi-letter level such as `aa` is representable. |
| `Id` | Struct. `family` is the uppercase family token; `levels` is the dotted path in order. Equality and hashing are structural over both fields. The families must match and the levels must match element for element, so `SEND-1` differs from `RECEIPT-1` and from `SEND-1.a`. Since `parse` refuses a padded level, no two distinct strings parse to equal `Id`s any more: `SEND-01` is `PaddedLevel`, not a second spelling of `SEND-1`. |
| `IdError` | Enum. Seven variants, each carrying enough to print a sentence that names the offending text or the offending depth. Stored by `doc::Criterion::id_error` so `hi check` can report a malformed line at its file and line (hi: CHECK-2.d). |

### Traits

| Trait | Description |
|-------|-------------|
| None. | The module defines no traits. It implements `std::fmt::Display` for `Level`, `Id`, and `IdError`, and derives the standard comparison and hashing traits listed above. |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse` | `fn parse(raw: &str) -> Result<Id, IdError>` | Associated function on `Id`. Validates family charset and strict level alternation in one pass and returns the first failure. |
| `parent` | `fn parent(&self) -> Option<Id>` | Clones the levels, drops the last, keeps the family. `None` when `levels.len() <= 1`. |
| `is_descendant_of` | `fn is_descendant_of(&self, other: &Id) -> bool` | Family equality, strictly greater depth, and a level-wise prefix match. |
| `depth` | `fn depth(&self) -> usize` | Returns `levels.len()`. |
| `root_number` | `fn root_number(&self) -> Option<u32>` | Matches the first level; `Some(n)` for `Level::Number(n)`, `None` otherwise. |
| `looks_like_id` | `fn looks_like_id(token: &str) -> bool` | Free function. Splits at the first `-`, requires a non-empty family that starts with `is_ascii_alphabetic` and is otherwise `is_ascii_alphanumeric` or `_`, and requires the remainder to start with `is_ascii_digit`. It writes both tests inline and does **not** call `valid_family`. |

## Invariants

1. An id is split at the **first** hyphen: everything before it is the family, everything after it
   is the dotted level path. A second hyphen therefore lands inside a level and fails as
   `BadLevel`, not as `BadFamily`. `SE-ND-1` reads as family `SE` with a level `ND-1`.
2. A family matches `[A-Z][A-Z0-9_]*`: the first character is ASCII uppercase, and the rest are
   ASCII uppercase, ASCII digits, or `_`. `SEND_2FA` is a family; `send` and `1SEND` are not.
3. Levels alternate strictly by 1-based depth: odd depths are numbers, even depths are letters
   (hi: ID-3). `SEND-1.a.b` and `SEND-a` are rejected, and the error names the depth and what was
   expected there (hi: ID-4).
4. A number level is all ASCII digits parsed through `u32`; a letter level is all ASCII lowercase
   letters, of any length. A level that mixes the two, or that contains anything else, is
   `BadLevel`. A numeric level longer than one digit that starts with `0` is refused as
   `PaddedLevel` *before* the `u32` parse, so `SEND-007` never quietly becomes `SEND-7` and the two
   spellings can never name one line (hi: ID-1.c). A bare `0` is not padding and parses.
5. `Display` is the inverse of `parse` for every accepted id: `Id::parse(s)?.to_string() == s` for
   every `s` that parses. Rejecting `PaddedLevel` is what closes the one gap that used to exist
   here; an integer wider than `u32::MAX` is rejected as `BadLevel`.
6. `parse` is the only constructor that enforces the grammar. `Id::family` and `Id::levels` are
   public, so a caller may build an `Id` whose levels do not alternate; nothing in the type stops
   it. `capture` is the one place that builds an `Id` without `parse` (`Id { family, levels:
   vec![Level::Number(next_free)] }` for the "next free is SEND-3" hint), and that value happens to
   be a valid depth-1 id, so no unchecked id is in circulation today. The alternation rule is still
   a parse invariant, not a type invariant: code must not assume an `Id` in hand came from `parse`.
7. `parent` preserves the family and removes exactly one level. Following `parent` repeatedly from
   any parsed id terminates at the depth-1 criterion, which returns `None`.
8. `is_descendant_of` is strict and structural: never reflexive, never true across families, and
   compares `Level` values rather than text, so `SEND-11` is not a descendant of `SEND-1`.
9. `depth` equals `levels.len()`, and for any id with a parent, `id.depth() == parent.depth() + 1`.
10. `looks_like_id` is weaker than `parse` in two ways and stricter in one. Every id that parses
    looks like an id. On top of those it accepts a wrongly cased family and a level `parse` refuses,
    so `send-2`, `Send-3`, `SEND-007`, `SEND-1a` and `SEND-1.` all look like ids and all fail to
    parse. That gap is what lets `doc` treat a malformed line as an intended criterion and `check`
    report it instead of silently reading it as prose (hi: CHECK-2.d). The case half of the gap is
    deliberate: a lowercase id is plainly meant as an id, so it is recognized here and refused by
    `parse` with a reason rather than lost.
11. `looks_like_id` requires the first level to begin with an ASCII digit. That single rule is what
    separates an id from an ordinary hyphenated word, so `spec-sync`, `well-formed` and
    `co-authored` are prose and are never read as criteria.
12. The two tests `looks_like_id` applies are written inline; it does **not** go through
    `valid_family`. `valid_family` belongs to `parse` alone and demands ASCII uppercase, while
    `looks_like_id` accepts either case. The predicates agree on which characters may appear in a
    family and disagree, on purpose, only about case. A charset change must be made in both places.
13. The shape tests in invariants 10 through 12 have a cost, recorded here rather than discovered
    later. A token whose family starts with a digit, or whose first level is a letter, fails the
    family test or the digit test, so it is not id-shaped, never reaches `parse`, and
    never becomes a criterion. `SEND-a`, `SEND-a.b` and `1ST-4` written under `## Criteria` are read
    as prose: `hi check` does not count them and does not report them, and `hi ls` does not list
    them. `IdError::Alternation { depth: 1 }` is therefore unreachable from a file, and so is the
    `IdError::BadFamily` raised by a family that starts with a digit or carries a character outside
    `[A-Za-z0-9_]`. Both are reached only when the id arrives as a command-line argument to a
    subcommand, as in `hi issue SEND-a` or `hi issue 1ST-4`; as a bare first argument, `hi SEND-a
    "..."` and `hi 1ST-4 "..."` are not routed to capture at all and clap answers "unrecognized
    subcommand" with exit 2. The `BadFamily` raised by a wrongly cased family is a separate matter
    and *is* reachable from a file, because `send-2` is id-shaped (invariant 10). So ID-4's promise
    holds for an alternation break at depth 3 or deeper (`SEND-1.a.b` is reported) and stops one
    step short at depth 1.
14. The same predicate is what `doc` uses outside every section to record a stray criterion, so a
    criterion-shaped line stranded above `## Criteria` is reported rather than lost
    (hi: CHECK-2.e). `looks_like_id` judges one token and nothing else. Whether that token is even
    structure is decided in `doc` before this module is consulted: `doc::is_criterion_line` strips a
    markdown bullet and any surrounding emphasis and hands over the first whitespace-separated
    token, and a line inside a fenced code block is prose whatever it looks like (hi: FILE-9).
15. The module is pure: no I/O, no globals, no interior mutability, and no dependency beyond
    `std::fmt`. Nothing here ever renumbers, reorders, or rewrites an id (hi: ID-1, ID-1.a).

## Behavioral Examples

#### Scenario: A top-level criterion id

- **Given** the string `SEND-1`
- **When** `Id::parse` runs
- **Then** `family` is `SEND`, `levels` is `[Number(1)]`, `to_string()` is `SEND-1`, `depth()` is 1,
  and `parent()` is `None`

#### Scenario: A full alternating path

- **Given** the string `SEND-1.a.1.b`
- **When** `Id::parse` runs
- **Then** `levels` is `[Number(1), Letter("a"), Number(1), Letter("b")]` and the id renders back as
  `SEND-1.a.1.b`

#### Scenario: A case of a case is refused

- **Given** the string `SEND-1.a.b`, a letter directly beneath a letter
- **When** `Id::parse` runs
- **Then** it returns `IdError::Alternation { depth: 3, expected: "a number" }`, because a case of a
  case has to be expressed as a step containing cases (hi: ID-4)

#### Scenario: The first level must be a number

- **Given** the string `SEND-a`
- **When** `Id::parse` runs
- **Then** it returns `IdError::Alternation { depth: 1, expected: "a number" }`

#### Scenario: A zero-padded number is refused rather than renamed

- **Given** the string `SEND-007`, or `SEND-1.a.01` at a deeper level
- **When** `Id::parse` runs
- **Then** it returns `IdError::PaddedLevel("007")` (or `PaddedLevel("01")`), whose sentence says the
  level has a leading zero and names the unpadded spelling to write instead, because a padded level
  would parse to a different spelling than it was written, and every reference to it would point at
  a line that does not exist (hi: ID-1.c)
- **And** `SEND-0`, `SEND-7`, and `SEND-10` all parse: a bare zero is not padding

#### Scenario: Families may carry digits and underscores

- **Given** the string `SEND_2FA-1`
- **When** `Id::parse` runs
- **Then** it succeeds, because `_` and `0-9` are legal after the leading uppercase letter

#### Scenario: Walking up the tree

- **Given** the parsed id `SEND-1.a.1`
- **When** `parent()` is called, and then `parent()` again on the result
- **Then** the results are `SEND-1.a` and `SEND-1`, and a third call returns `None`

#### Scenario: Descendants are structural, not textual

- **Given** the parsed root `SEND-1`
- **When** `SEND-1.a`, `SEND-1.a.1`, `SEND-2`, `RECEIPT-1.a`, `SEND-1` itself, and `SEND-11` are each
  asked `is_descendant_of(&root)`
- **Then** the first two are true and the rest are false: a criterion is not its own descendant,
  and `SEND-11` does not read as a descendant of `SEND-1`

#### Scenario: Telling a criterion line from prose

- **Given** the first token of a line in a hi file, after `doc` has stripped any bullet and emphasis
- **When** `looks_like_id` runs
- **Then** `SEND-1` and `SEND-1.a` are id-shaped, and so are the malformed `send-2` and `Send-3`,
  because a wrongly cased id is plainly meant as an id and should be refused with a reason rather
  than lost (hi: CHECK-2.d)
- **And** `I`, `Given`, `well-formed`, `spec-sync`, `co-authored`, and `SEND-a` are not id-shaped, so
  a sentence that starts a paragraph and an ordinary hyphenated word are never mistaken for criteria

#### Scenario: A wrongly cased id is reported, not swallowed

- **Given** a `## Criteria` section holding `- **send-2**  Lowercase, meant as a criterion.`
- **When** `hi check` runs
- **Then** `looks_like_id` says the token is id-shaped, `Id::parse` returns
  `IdError::BadFamily("send")`, and `check` prints
  `unparseable-id  'send-2' is not a valid id: family 'send' must start with A-Z and contain only
  A-Z, 0-9, _` at that file and line, exiting 1

#### Scenario: A hyphenated word stays a word

- **Given** a line of prose reading `spec-sync and well-formed are ordinary words.`
- **When** `doc` parses the file
- **Then** nothing on the line is id-shaped, because the first level does not start with a digit, so
  the line stays prose and `hi check` exits 0

#### Scenario: The first level being a letter puts the token out of reach

- **Given** `- **SEND-a.b**  a case of a case` written under `## Criteria`
- **When** `hi check` runs
- **Then** the line is **not** reported: `looks_like_id` is false because the first level is a
  letter, so the line is read as prose and the criterion count does not include it
- **And** the same id given as a subcommand argument, `hi issue SEND-a.b`, does reach `Id::parse` and
  prints `error: 'SEND-a.b' is not a valid id: level 1 must be a number, because levels alternate
  number, letter, number, letter` at exit 1. The break is reported at depth 1, not depth 3, because
  the letter `a` is already wrong where it stands (invariant 13)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No `-` in the token (`SEND1`) | `Err(IdError::MissingHyphen)`, which prints "no '-' between family and number" |
| Family is empty, lowercase, or starts with a non-letter (`send-1`, `1SEND-1`, `-1`) | `Err(IdError::BadFamily(family))`, whose sentence names the family: "family 'send' must start with A-Z and contain only A-Z, 0-9, _" |
| Nothing after the hyphen (`SEND-`) | `Err(IdError::NoLevels)`, printing "nothing after the '-'" |
| A doubled or trailing dot (`SEND-1.`, `SEND-1..a`) | `Err(IdError::EmptyLevel)`. The sentence is "empty level (a doubled or trailing '.')" |
| A level that is neither all digits nor all lowercase letters (`SE-ND-1`, `SEND-1.A`, `SEND-1a`) | `Err(IdError::BadLevel(level))`, whose sentence quotes the level: "level 'ND-1' must be a number or lowercase letters" |
| A number level too large for `u32` | `Err(IdError::BadLevel(level))`; the `u32` parse failure is mapped to the same variant |
| A numeric level with a leading zero (`SEND-007`, `SEND-1.a.01`) | `Err(IdError::PaddedLevel(level))`, whose sentence is "level '007' has a leading zero; write it as '7', so the id always means the same thing". The check runs before the `u32` parse, so the padding is never normalized away. A bare `0` is not padding and parses |
| A level of the right shape at the wrong depth (`SEND-a`, `SEND-1.a.b`) | `Err(IdError::Alternation { depth, expected })`. The sentence is "level 3 must be a number, because levels alternate number, letter, number, letter", with `expected` reading either "a number" or "a letter" |
| `parent()` on a depth-1 id | `None`. Not an error; it is how `capture` and `check` learn that a criterion is top-level and needs no parent |
| `root_number()` on an id whose first level is a letter, or that has no levels | `None`. Only reachable for a hand-built `Id`, since `parse` guarantees a leading number |
| A token that is id-shaped but not parseable (`send-2`, `SEND-007`, `SEND-1a`, `SEND-1.`, `SEND-1.a.b`) | `looks_like_id` returns `true` while `Id::parse` returns `Err`. The caller decides: `doc` stores the `IdError` on the criterion and `check` reports it as a structural error |
| A token that is not id-shaped but would not parse either (`SEND-a`, `SEND-a.b`, `1ST-4`) | `looks_like_id` returns `false`, so `doc` never calls `parse` and the line is prose. `check` reports nothing and `ls` lists nothing. The error is only reachable when the id is an argument to a subcommand, such as `hi issue SEND-a` (invariant 13) |

## Dependencies

**Consumes**

| Crate/Module | What is used |
|-------------|-------------|
| `std::fmt` | `Display` and `Formatter` for the three `Display` impls |

No sibling module and no external crate. This is the bottom of hi's dependency graph.

**Consumed By**

| Module | What is used |
|--------|-------------|
| `capture` (`src/capture.rs`) | `Id::parse` for the captured id; `Id::parent` twice, first to refuse a case with no parent, then to resolve the file that actually holds that parent before falling back to `Id.family` (hi: CAPTURE-4.a); `Id { family, levels }` with `Level::Number` to render the "next free is SEND-3" hint |
| `check` (`src/check.rs`) | `Id::parent` for orphan-case detection; `Id.family` for the undeclared-family check; `Id` rendering for problem messages |
| `doc` (`src/doc.rs`) | `looks_like_id`, always through `doc`'s own `is_criterion_line`, which strips a markdown bullet (`- `, `* `, `+ `) and any surrounding `*`/`_` emphasis and tests the first whitespace-separated token. There are three call sites: inside a section to tell a criterion line from prose, outside every section to record a stray criterion, and in the continuation scan, where an indented line that is itself a criterion ends the one above it. Also `Id::parse` with the `IdError` stored on `Criterion::id_error`; `Id::parent`, `Id` equality, and `Id::is_descendant_of` in `insertion_point` to place a new criterion after its parent and that parent's last descendant |
| `out` (`src/out.rs`) | `Id::parse` in `issue`; `Id::depth` for list and ticket indentation; `Id::is_descendant_of` to gather a criterion's cases; `Id::depth` and `Id::parent` for the export payload's `depth` and `parent` fields |
| `workspace` (`src/workspace.rs`) | `Id` equality (`PartialEq`) in `find_id`; `Id.family` and `Id::root_number` in `next_free` |
| `view` (`src/view.rs`) | `Id::depth` in `criteria_list` to build the `d{depth}` indent class for a criterion in the rendered page. The page styles `d2` through `d4`; `d1` and anything deeper fall back to no extra padding |
| `main` (`src/main.rs`) | `looks_like_id` to route an id-shaped first argument to capture before clap parses the command line. It is applied to the tail left after `peel_root` removes `--root` (which `peel_root` only consumes *before* the id, so everything after the id is the sentence), so a flag is never the token tested (hi: CAPTURE-8), and only to an argument that converts through `OsStr::to_str`, so a non-UTF-8 argument is simply not id-shaped and gets a plain error rather than a panic (hi: CAPTURE-1.c). A first argument that is not id-shaped falls through to clap, so `hi SEND-a "..."` is an "unrecognized subcommand" usage error with exit 2 rather than the alternation error with exit 1 (invariant 13) |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-09-16 | Leif | Initial specification. |
| 2026-09-16 | Leif | Verification pass against `src/id.rs`: added `view` as a consumer (`Id::depth` in `criteria_list`), added `Id.family` to the `workspace` row, corrected the hand-built-`Id` invariant (the `Id` `capture` builds is a valid depth-1 id), and stopped describing `Display` as derived. |
| 2026-09-16 | Leif | Reconciled with the bug-fix pass. Added the `IdError::PaddedLevel` variant (seven variants now), so a zero-padded number is refused instead of normalized (hi: ID-1.c): rewrote the level-shape and `Display`-inverse invariants, added the refusal scenario and error row, and added REQ-id-011. Recorded that `looks_like_id` also gates `doc`'s stray-criterion detection (hi: CHECK-2.e) and that fence opacity is decided in `doc` before this module is consulted (hi: FILE-9). Updated the `capture` consumer row for parent-first file resolution (hi: CAPTURE-4.a) and the `main` row for `peel_root` and `args_os` (hi: CAPTURE-8, CAPTURE-1.c). |
| 2026-09-16 | Leif | Adversarial re-verification against `src/id.rs`. Two residues of the padding change survived the previous row: the `Id` struct row still claimed `SEND-1` and `SEND-01` compare equal "both parse to `Number(1)`" (`SEND-01` is now `PaddedLevel` and does not parse at all), and the Purpose paragraph still said `capture` resolves a file from `Id::family` alone. Both corrected; everything else in the spec traced clean to the source. |
| 2026-09-16 | Leif | Reconciled with the rewritten `looks_like_id`. The predicate is now case-insensitive on the family and requires the first level to start with a digit, and it no longer calls `valid_family`. Rewrote its Public API and Functions rows, split invariant 10 into invariants 10 through 14 (the gap, the first-level-digit rule, the `valid_family` split, the blind spot it creates, and `doc`'s use of it), replaced the `SEND-a.b` prose example with `send-2` / `spec-sync` scenarios, and added two scenarios and an error row for tokens that are now out of `parse`'s reach. Corrected the `PaddedLevel` and `Alternation` sentences to the bytes the binary prints (`; write it as` and `, because levels alternate`, not `. Write it as` and `. Levels alternate`). Updated the `doc` consumer row for `is_criterion_line`/`strip_bullet`/`strip_emphasis` and its third call site, the `main` row for `peel_root` and the clap fall-through, and the `view` row for the `d{depth}` class. |
| 2026-09-18 | Claude | Added REQ-id-012 for the property test of the promise: generated files hi did not write, random capture / retire / hand-edit sequences, serial and concurrent, asserting no id is assigned twice and every unnamed criterion keeps its shape (hi: ID-5, ID-5.a, DECISIONS.md §39). |
