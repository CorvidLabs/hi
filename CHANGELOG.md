# Changelog

All notable changes to `hi` (Human Intent). Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The format itself is versioned separately by the `hi:` key in each file's frontmatter. `HI/1` is the
only version so far, and since the Unreleased entry below a file declaring any other version is
refused rather than read as this one.

## Unreleased

### After the 1.0 readiness review of #20

- `hi: "1"`, `hi: '1'` and `hi: 1 # comment` are HI/1. They were refused as unknown versions,
  which locked the whole repository over YAML punctuation that `owner:` already tolerated.
- `hi seed` rewrites `hi/AGENTS.md` atomically. A truncating write that died halfway would have
  left a file that is no longer a template hi shipped, which `hi seed` then refuses forever.
- On Windows, `still_at` now compares the held handle's file identity against the path's, through
  `GetFileInformationByHandle`, instead of relying on the delete-pending argument DECISIONS.md §34
  admitted nothing tested.
- HI-1.md's id grammar names the digits a family may contain, the `a-z` letter levels, and the
  32-bit bound on a number.

### Two files claiming one family is a structural problem, not a tie-break

Two files can both declare `families: [SEND]`. `hi check` exited 0 and said nothing, and
a new capture landed in whichever file sorted first, because `doc_for_family` used
`.position()` over path-sorted docs. Rename a file and later captures moved.

**`duplicate-family` is the seventh `hi check` kind.** It reports the later file, naming
the first, the same shape as `duplicate-id`. Capture of a new top-level id in that family
refuses and writes nothing. A case still follows its parent. A family listed twice in
*one* file is the same declaration written twice, not two homes.

1.0 freezes these seven kinds *and* the policy that they are structural only. An eighth
is a 2.0. The README had promised exactly six; the thing not to freeze in 0.x was the
cardinality, so this seventh could still arrive (DECISIONS.md §39, `CHECK-2.g`,
`CAPTURE-16`).

### `hi seed` migrates a template hi shipped, and refuses a file you edited

Capture still writes `hi/AGENTS.md` only when it is absent. That write-once is what
stops a surprise rewrite of a file somebody touched, and it is unchanged.

A 1.0 that freezes the convention that file describes cannot leave every 0.5.0 adopter
with a hard-wrapped template and no merge sentence. `hi seed` is the verb:

- missing → write the current text, and `hi/CLAUDE.md` beside it
- byte-identical to a template this hi has shipped (BOM and CRLF folded) → rewrite,
  keeping the endings the file had
- already current → say so
- anything else → refuse, exit 1, write nothing

Delete it and run `hi seed` if the current text is what you want. Capture never
overwrites, even when the file is an old template (DECISIONS.md §39, `HABIT-6`).

### The HI/1 contract, and a suite that generates the case nobody wrote

[HI-1.md](HI-1.md) is the compatibility document three 1.0 reviews called blocking.
What is frozen, what is not, the format, the exit codes, the export envelope, the seven
kinds, and the promise. Permanence is a convention over **shared history**: the merged
tree, not an unmerged branch.

`src/promise.rs` and `tests/promise.rs` generate files hi did not write and run random
capture / retire / hand-edit sequences, serially and concurrently. After every step:
every id a verb reported as saved is readable in the section it named; unnamed criteria
keep their shape; no id is assigned twice; `hi check` exiting 0 implies all three.
Zero of the twelve confirmed defects were found this way. That is the item that
changes the finding method (`ID-5`, DECISIONS.md §39).

### The format version is a promise now, not a decoration

`hi: 1` sits in every file's frontmatter and **nothing read it**. `parse_front` stored
the number and no verb ever asked for it, so a file declaring `hi: 2` loaded, `hi check`
exited 0, `hi ls` printed its criteria, and a capture appended to it as HI/1 without
mentioning anything.

That is broken in both directions. `HI/1` could not be frozen by a 1.0, because a
reader that treats every version as 1 has committed to nothing; and an `HI/2` could
never ship, because every binary already installed would open an HI/2 file believing
it understood it.

**A file declaring a version this hi does not read is now refused by name, by every
verb.** The message says which file and which version. A refusal writes nothing at
all — not the criterion, not `hi/`, not `INTENT.md`, not `hi/AGENTS.md`, not even the
lock file — because it happens before the write lock is taken. A file with no `hi:`
line is still HI/1, so files written before the key existed keep working.

It is **not** a structural problem. `hi check` still never fails on unfinished intent. An unknown
version is hi saying it did not read the file, which is the same category as a file it could not
decode (DECISIONS.md §36), not something it found wrong inside one.

### `hi index` takes the write lock

Capture and `hi retire` hold the lock across their whole read-modify-write. `hi index`
did not, and it is the one path that can *install* a generated block: it reads
`INTENT.md`, splices the list in, and writes the rest back. A capture that finished
between that read and that write was undone — the list went back without the new
criterion, and any prose saved in between went with it — while both commands printed
success. It now takes the lock and reloads the workspace under it, like the other two.

`hi view` deliberately takes no lock. It never reads the page it is about to write, so
there is no read-modify-write to protect; its output is derived and gitignored; and
taking the write lock would have a read verb creating `hi/` in a repository it was only
asked to look at. DECISIONS.md §38 argues it rather than assuming it.

### Added

**`hi export` says which shape it is, separately from which format it read.** The
payload carries `"export": 1` beside `"hi": 1`. `hi` is the *file format*'s version and
stays that; `export` is the version of the JSON envelope. One field could never later
mean both, and splitting them is impossible once anything depends on the shape.

**`hi/AGENTS.md` tells an agent to run `hi check` after a merge that touched `hi/`.**
Two branches can each choose the same id, and git merges both cleanly and says nothing
— which happened to this repository (DECISIONS.md §37). One sentence, in the file hi
writes on a first capture. DECISIONS.md §38 records why a second narrowing of that
file's "say the habit and nothing else" rule was admitted, and the rule now written
down for the next one.

### Changed

**`hi check --json` carries its notes as a list, each with a code.** `note` was a single
string with every note joined by a newline and six spaces of terminal indentation, so a
script had to split on whitespace to get them apart. It is now `notes`, an array of
`{ kind, message }`, with the stable codes `no-product-why`, `index-behind`,
`index-markers` and `unexplained-retirement`. The wording is still for people and is
free to change; the codes are the part a script can hold on to. None of them is a
structural problem and none of them moves the exit code.

This changes the shape of `hi check --json`. It is the only breaking change here, and
`hi check`'s text output is unchanged apart from each note printing on its own `note:`
line.

**Documented: how to get the current `hi/AGENTS.md`.** Capture writes that file once and
never again, which is what stops it overwriting something you edited. `hi seed` is the
verb that migrates a template hi has actually shipped; it refuses a file you have
edited. DECISIONS.md §39 is the argument; §38's delete-and-recapture remains true of
capture itself.

**A feature list you deleted stays deleted.** Capture and `hi retire` refresh the block
they find and no longer reinstate a `## Features` section you removed; `hi index` is how
you ask for one. Rewriting the list between hi's markers is what hi promised; adding a
heading to your file is writing prose, and hi cannot tell a block you deleted from one
you never had. An `INTENT.md` hi creates on your first capture now carries its feature
list from birth, so nothing about a fresh repository changes. DECISIONS.md §30 accepted
the old behaviour as a cost; §32 withdraws it.

§30's claim that drift became "structurally impossible" is softened in the same section.
The refresh is best effort by design, so a broken marker pair or a file hi cannot read
leaves the list wrong while your capture succeeds. What is true is narrower: the
ordinary path no longer depends on anybody remembering. (`hi index` typed by hand took
no lock either, until the entry above gave it one.)

### Two writes that landed where nothing reads them

An external review found two more ways to break hi's one promise, that an id is
permanent and never reused. Both were the same mistake: a verb that writes decided
where a section was without asking the code that reads.

**A documented `## Retired` made a real id reusable.** `FILE-9` invites you to show
an example of the format inside your own `## Intent`, and a fence keeps it prose. But
`hi retire` looked for `## Retired` by scanning the raw lines, so it found the one in
your example. The criterion was moved into the intent prose, the command printed
`retired`, and the file parsed back with no criteria and no retirements at all. The
id was then free, and capturing it again succeeded. `hi check` exited 0 throughout.
The file was valid the whole time.

**Capture could "save" the same id over and over into an unfinished fence.** A file
whose last content line sits inside a fence nobody closed got its missing
`## Criteria` section appended inside that fence, along with the criterion. Two
captures of the same id with different sentences both reported success, both lines
were invisible, and `hi check` reported zero criteria.

**Every write now reads itself back.** `insert`, `retire` and the reason-recording
path each parse the file they are about to save and refuse it unless the id is
readable in the section the verb named, every id the file already made readable still
is, and nothing new has been stranded. A refusal writes nothing and leaves the
document exactly as it was, and it names the unclosed fence and the line it opens on
when there is one. Closing the fence makes the same capture succeed.

**A fence is still an example, not structure.** The retired-section lookup now uses
the same fence tracking the parser uses, so the legitimate retirement lands under a
real `## Retired` and your example comes back byte-identical, with the id drawn in it
still free to capture for real.

### Fixed

**A lock test could not fail in a root container.** It makes a directory unwritable and
asserts that taking the lock is refused. Root ignores the mode bits, so the assertion
was about an environment the test was not in. It now probes whether the directory is
really unwritable and skips with a stated reason when it is not, rather than asserting
something it cannot observe.

**DECISIONS.md §26 said more than hi can keep.** It stated the one promise absolutely —
*an id is permanent and never reused* — when the honest version, which README rule 4 has
carried all along, is that hi never reuses an id and never lets one of its own verbs
reuse one. The files are markdown and they are yours; nothing stops a hand edit, and
nothing stops two branches choosing the same id. §30 was rewritten for the same reason
once already.

**An automatic index refresh could erase your whole `INTENT.md`.** `write_index` turned
every read error into an empty string, and then treated an empty string as "no file
here, write the starter". An `INTENT.md` holding your prose and one byte that is not
valid UTF-8 was replaced by the starter prompt and a generated list, and the command
exited 0 saying nothing. Only a file that is genuinely absent is created now; any other
read failure leaves every byte where it is and is reported — as a printed note on a
capture, which still succeeds with your criterion stored, and as the exit code on
`hi index`, where the failure is the whole answer.

This defect had been there since 0.2.0 and was reachable only by typing `hi index`.
0.7.0 made capture and `hi retire` refresh the block on every write, which turned it
into one that fires constantly. DECISIONS.md §32 records that making a call automatic
is a change to every bug inside it.

**An id in a file hi skips was reported as used and then handed out again.** hi does
not read an uppercase-named file in `hi/` as criteria, because those are its own
(`hi/AGENTS.md`, `hi/CLAUDE.md`). `hi check` looked inside them anyway and reported any
criterion it found, but capture did not, so a retired `SEND-1` parked in `hi/Archive.md`
was announced as taken and then reissued with different words. There is now one
reservation lookup covering the files hi loads and the files it skips, and `check` and
`capture` both answer from it. hi's own files still hold no criteria, are still not
counted, and still never appear in the feature list.

### The first capture in a repository is locked like every other one

Thirty-two `hi SEND-N "..."` at once in a repository with no `hi/` yet: all
thirty-two exited 0, nine of the criteria were not in the file, `hi check` reported
nothing wrong, and capturing one of the lost ids afterwards succeeded and wrote a
different sentence under an id that had already been spent. That is ordinary bulk
adoption, and it is the one thing hi promises cannot happen.

The write lock lives inside `hi/`, so before that directory existed there was nowhere
to create it, and the failure to create it was handed back as a lock: every
concurrent first capture ran unlocked. Worse, releasing one of those removed the lock
file of a writer that really did hold it.

`hi` now creates `hi/` before taking the lock, and a lock it could not take is an
error rather than a guard. A guard that did not acquire the lock is no longer
something that can exist, so releasing one can never take somebody else's.

A capture that refuses in a repository that had no `hi/` still writes nothing at all:
the directory the lock created goes with the lock when it is released, and only ever
while it is still empty.

### `hi retire` no longer writes from before it took the lock

`retire` read the files, then took the lock, and never read them again. Two retires
at once both printed `retired`, both exited 0, and one of the two criteria was live
again afterwards with `hi check` reporting nothing wrong. It now reloads inside the
lock, the way capture always has.

### A lock is broken because nobody holds it, not because it is old

Any lock file older than sixty seconds used to be treated as belonging to a dead
process and taken. Age says a holder is old, not that it is gone, and a slow bulk
capture on a slow filesystem is old. That rule could take the lock away from a writer
in the middle of a write.

A writer now refreshes its lock four times a second while it holds it, and a waiter
breaks a lock only after watching that stop for five seconds by its own clock. A `hi`
that was killed still frees its repository without anybody deleting a file by hand,
and a `hi` that is merely slow keeps what it took. Nothing compares this machine's
clock against the file's, so clock skew cannot make a held lock look abandoned.

`hi` now waits up to thirty seconds for another writer rather than five, which is
also what a few hundred queued captures need.

On Windows a removed file stays present until every handle to it closes, so a
waiter arriving while another writer releases is told access is denied rather
than that the lock exists. hi retries such an error for half a second before
reporting it, which is a handoff rather than a locked directory. It still never
hands back a lock it did not take.

Recorded in DECISIONS.md §33, with why the §26 pass did not cover any of this.

### The write lock is the operating system's

An external re-review broke the lock this repository shipped a day earlier, with an interposer
that only delays syscalls. A waiter was frozen at the instant *after* it had confirmed another
lock was abandoned and *before* it removed it; a second waiter then broke the same lock, took its
own, and started writing while heartbeating. The first waiter was released, executed its
already-approved deletion, and removed a live holder's lock. Both processes saved their own
snapshot, both printed the criterion they had stored, both exited 0, one criterion was gone, and
`hi check` reported nothing wrong. The disclosed SIGSTOP case was reproduced too: a holder stopped
part-way through its write was declared abandoned after five seconds, and its criterion was
overwritten by the writer that took over.

No extra check fixes that. Verifying and removing are two operations and the holder can change
between them, which is true of any rule hi invents about when somebody else has finished.

**So hi does not invent one.** It holds `flock(2)` on unix and `LockFileEx` on Windows and never
breaks a lock at all. The kernel releases those when a process exits, however it exits, so a `hi`
that was killed still frees its repository with nothing to delete — and a `hi` that is merely
slow, stopped or waiting on a slow disk keeps what it took for as long as it is alive. Those two
were in tension under every timeout, which is why every timeout was wrong.

**No new dependency.** hi has four, and two `extern` declarations are not worth a fifth.

Two consequences worth knowing:

`.hi.lock` is no longer the lock, so deleting it while a `hi` is running is now the one act that
can let two writers into a repository at once. The timeout message says so instead of telling you
to delete it. A lock file left behind by a killed `hi` is harmless: the next writer takes it
without waiting and without deleting anything.

hi fails rather than writes if the filesystem cannot lock. `flock` is emulated or absent on some
network filesystems, and hi has not been tested on any of them; a local checkout is the supported
answer. Linux, macOS and Windows all run the full suite in CI, the concurrency tests included.

Recorded in DECISIONS.md §34, which also records why the two rules that stop the same defect
returning through the back door — verify the file you were granted is still the file the name
points at, and unlink while holding rather than after — are load-bearing.

### A capture could bring a retired criterion back to life

Two spaces in front of a `## Retired` heading is something people type, and hi reads such a file
exactly as its author meant it. Capturing into one did not. The new criterion was spliced directly
above the indented heading, the parser read that heading as a continuation of the new criterion's
sentence, and everything the heading had separated fell into `## Criteria`: the retired criterion
was active again, still carrying its `retired:` reason, and the new one's sentence had
`## Retired` on the end of it. The command reported success and `hi check` exited 0 before and
after.

Two things were wrong and both are fixed. `read_criterion` now stops a continuation at anything
`parse_body` would read as a heading, through one shared predicate, so the parser cannot disagree
with itself about where a section starts. **The indented heading is still legitimate** — it is
valid markdown and refusing it would have been fixing the file instead of the code.

And the postcondition every write is held to now compares criteria rather than ids. It was a
multiset of ids, and every id in that file was still present afterwards; an id comparison cannot
see a criterion change section, change its sentence, or pick up somebody else's retirement reason.
Every criterion a write did not name now has to come back with the same section, the same sentence
and the same reason, or the write is refused with nothing written and the refusal says which id and
what would have happened to it.

The two fixes are independent on purpose: with the parser fix removed, the postcondition refuses
the capture instead of losing the criterion. Recorded in DECISIONS.md §35.

### A file hi could not read gave away an id that was reserved in it

hi does not read an uppercase-named file in `hi/` as criteria, but it does look inside one for ids,
because an id written there is still taken. That lookup swallowed read errors: a file it could not
open or decode was passed over, and "nothing found" was handed back to both of its callers as "that
id is free". One Latin-1 byte in a retirement reason was enough. A retired `SEND-1` parked in
`hi/Archive.md` was reissued with different words, the reservation was still on disk, and
`hi check` exited 0 before and after.

The read failure is now the answer. `hi check` exits 1 naming the file it could not read, and a
capture refuses before it writes anything at all — not the criterion, not `hi/`, not `INTENT.md`
and not `hi/AGENTS.md`.

**This is not another `hi check` kind.** It still never fails on unfinished intent. A file hi cannot read is hi saying it could not do the check, which is not a finding.

0.7.0 made `check` and `capture` answer from one lookup so they could not disagree. They could
still both be wrong, and here they were. Recorded in DECISIONS.md §36.

### Two branches had both captured FILE-22

Two workstreams open at once each captured `FILE-22`, because on each branch that was the next free
number. The lock branch's want is now `FILE-23`, the write-path branch's keeps `FILE-22`, and
`FILE-24` was captured for the other half of the lock guarantee. Both wants are live and no id
carries two sentences. The same thing happened to a spec requirement — both branches wrote
`REQ-workspace-011`, and git merged them with no conflict into one document with two headings of
that name — which is now `REQ-workspace-012` for the write lock. Recorded in DECISIONS.md §37,
along with why an id is only unique against the tree you captured on.

## [0.7.0] 2026-09-17

### The feature list in INTENT.md stays true by itself

`hi index` regenerated the list and nothing made anyone run it. Three adopter
repositories had already drifted: peck's block said 56 against 57 actual, podo-web's
said 53 against 57. This repository escaped it only because
`scripts/index-is-current.sh` is in its gate, which no adopter has.

Capture and `hi retire` now refresh the block themselves, so the list is true after
every command that changes it. Drift is structurally impossible rather than merely
detectable.

**A capture that stored your criterion is never reported as a failure.** The refresh
runs only after the criterion is on disk, and any reason it could not happen is
printed as a note on stderr while the command still exits 0. That includes hi's
existing refusal to guess where a broken `hi:index` marker pair ends: it still
refuses, it still writes nothing, and your capture still succeeds. This is how
`INTENT.md`'s creation has behaved since it was added.

**Nothing about what gets rewritten changed.** The prose in `INTENT.md` is still
yours; hi still only ever replaces the list between its own markers, still leaves a
marker quoted in prose or inside a fence alone, and still refuses rather than guesses.

`hi check` now says when the list is behind, for the one case no verb can see: a
criterion typed straight into a file, which hi has always accepted. It is a note and
never a problem. `hi check` still fails on exactly six structural things and the exit
code does not move for this.

Two costs, both accepted and both recorded in DECISIONS.md §30. A bulk capture
rewrites `INTENT.md` once per criterion; fledge's adoption was 197 captures. And an
`INTENT.md` you had removed the generated block from gets one back on the next
capture.

### The fledge plugin declares what it now does

`plugin.toml` moves to 0.2.0. The plugin gained two lifecycle hooks and `exec = true` in 0.5.0 and its own version never moved, so a `fledge plugins list` could not tell a plugin that runs a script at `work start` and `push` from one that only adds a command. It missed the 0.6.0 tag, so it lands here.

## [0.6.0] 2026-09-17

### A ticket reads as paragraphs, not as a narrow column

A GitHub issue body is rendered with hard line breaks on, so a single newline in it
is a visible break even though the same bytes in a file are not. `## Intent` prose is
wrapped by hand at whatever margin its author works to, so every ticket `hi issue`
produced arrived broken after each authored line, in a narrow column down the left of
a wide pane.

`hi issue` now unwraps the breaks nobody asked for. A newline inside a paragraph
becomes a space; a blank line stays a paragraph break; and a newline that means
something stays where it is, in a list, a block quote, a heading, a table, a rule, a
fenced block, or after the two trailing spaces that are markdown asking for a break on
purpose.

`hi view` and `hi export` were checked and deliberately not changed. The page already
renders these paragraphs whole, and the export payload is a transport rather than a
rendering, so it keeps the prose exactly as written.

**hi does not touch your file.** This is a render-time fix, and there is no
rewrite-on-write and no new `hi check` problem for wrapping.

### The prose hi writes is one line per paragraph

The durable half of the same fix. What an adopter copies is whatever hi's own files
model for them, so the text hi writes into `hi/AGENTS.md` and `INTENT.md` is now one
line per paragraph, `hi/AGENTS.md` says the convention in one sentence, the README
says it as the fifth rule of the format, and this repository's own `hi/*.md` intent
blocks were reflowed to match, with no word changed.

### First contact, in a repository that has never seen hi

`hi/AGENTS.md` only reaches an agent already looking in `hi/`, so it could never
introduce hi to a repository that has none. hi's fledge plugin now can, because a
fledge plugin installs once per user rather than once per repository.

Two lifecycle hooks: `post_work_start` says, as a feature branch is created, that
nothing is written down here yet; `pre_push` says it once more, differently, before
the branch ships. A repository that already has a `hi/` hears nothing from either.

The nudge never writes anything and never fails. A fledge hook that exits non-zero
aborts the command that ran it, so every path in it exits 0 and
`scripts/nudge-behaves.sh` asserts that before it asserts anything about wording. It
writes to stderr only, so a `fledge work start --json` envelope stays valid.

Needs `FLEDGE_REPO_ROOT` (CorvidLabs/fledge#520) to know which repository it fired
for. On a fledge without it, the nudge says nothing rather than guessing.

The plugin now declares `exec = true`, because fledge skips the hooks of any plugin
that has not, and the honest answer is that it does run a script.

## [0.5.0] 2026-09-16

### An agent finds hi without being told

The first capture in a repository now leaves `hi/AGENTS.md`, and a `hi/CLAUDE.md`
beside it, describing the habit: read what is already written, draft the criteria,
ask the person to confirm them, capture what they agree to, then build. hi writes
them once and never again, and they are yours afterwards. A capture that stored its
criterion is never reported as a failure because these could not be written.

The file says the habit and nothing else. No id grammar, no file format, no list of
the families already here: it is written once and never rewritten, so anything hi
could change underneath it would be wrong later with nothing to notice.

`hi/CLAUDE.md` is a symlink where the platform allows one, and a one-line pointer
file where it does not. A committed symlink arrives as a text file holding the
literal target wherever `core.symlinks` is false, which is the Git-for-Windows
default, and an agent would read that as the whole instruction.

### Files in `hi/` are lowercase

A criteria file is lowercase, because a family names its own file and capture
lowercases it. An uppercase name in `hi/` is hi's own rather than criteria, which is
what keeps `AGENTS.md` and `CLAUDE.md` out of the criterion count and out of the
generated feature list in `INTENT.md`.

Nothing is skipped quietly. `hi check` reads inside the files it does not parse as
criteria and reports any criterion-shaped line as `stray-criterion`, because a
criterion hi cannot see must be reported and never ignored. A fenced example is
still an example.

`AGENTS` and `CLAUDE` cannot be family names. `AGENTS-1` wants `hi/agents.md`, which
is the same file as `hi/AGENTS.md` on a case-insensitive filesystem; capture refuses
both names on every platform, before any write, rather than behaving one way on
macOS and another on Linux.

## [0.4.0] 2026-09-16

### hi could break its own only promise, four ways

An id is permanent and never reused. That is the one thing hi
guarantees, and a 13-agent audit found four ways its own verbs broke it.
Three were silent: `hi check` reported no problem.

**`hi retire` put the criterion in the wrong place.** `retired_end` was a
match with two byte-identical arms, so it always appended at end of file.
If `## Retired` was not the last section, the criterion landed under
whatever followed it, outside every section hi reads. It printed
"retired". `hi ls --retired` lost it, and **the id could then be captured
again**: two `- **SEND-1**` lines, different sentences, one file
(`RETIRE-5`).

**A retire reason could forge a criterion.** Criterion sentences were
collapsed to one line; retire reasons were not. A reason containing a
newline and a criterion-shaped line wrote a second real criterion into
the file. `hi check` reported zero problems, and that id was burned
forever, having never been written by anyone. Every string hi writes into
a file now goes through one normalizer (`RETIRE-6`).

**A criterion inside a fence was invisible.** Fenced blocks are opaque so
you can document the format inside your own `## Intent` (`FILE-9`). Under
`## Criteria` that hid the criterion from all six checks, and capture
would hand the same id out again. A fence inside a criteria section is
now reported as `stray-criterion`, and **capture refuses any id already
written somewhere hi cannot read it** (`FILE-20`, `CAPTURE-14`).

**Eight concurrent captures landed two.** Capture is read-modify-write,
and `write_atomically` made the write atomic while doing nothing about
two processes reading the same original and each writing over the other.
It also used a fixed temp filename, so the two collided. Capture and
retire now hold a lock file across the whole read-modify-write; eight
concurrent captures land eight (`FILE-19`).

This stopped being theoretical: twelve repositories now hold about 1,700
criteria, most captured by agents in bulk.

### Smaller

- `hi` no longer offers `SEND-0` as the next free id at the top of the
  range. Where there is no next id it gives no hint rather than a wrong
  one (`CAPTURE-13`).
- The integration-test fixture path had the same fixed-path bug as
  `write_atomically`, which is why the suite flaked.

Every fix has a regression test, and each is written over a file hi did
not produce, because asserting on hi's own output is the blind spot that
produced three of these.

## [0.3.3] 2026-09-16

### The page is now checked by something that opens it

Every view test asserted on the HTML going in. Two bugs shipped anyway,
and both were found by a person taking a screenshot: `.row { display:
flex }` outranked the user agent's `[hidden]` so filters hid nothing
(0.2.0 to 0.2.3), and the role chip rendered jammed into the sentence
with no separator (0.2.0 to 0.2.3).

`scripts/view-behaves.sh` loads a generated page in headless Chrome and
drives it the way a person would: click a feature and count what is
still on screen, search and check the matches are highlighted, toggle
retired, move the cursor with `j`, focus with `/`, and follow a deep
link into a row a filter would have hidden. Fourteen checks, asserting
on what is visible rather than on what was generated. Removing the
`[hidden]` rule fails seven of them, which is how it was verified.

It is a step in `fledge lanes run verify` and its own CI job. Where no
Chrome or Chromium is installed it says so and skips, so it never blocks
a local gate (`VIEW-20`).

## [0.3.2] 2026-09-16

### Two defects an audit found, and the drift around them

**`hi` no longer panics on an empty file.** A zero-byte `hi/*.md` yields
no lines at all, so the insert spliced past the end of an empty buffer
and aborted with a backtrace at exit 101, instead of the "no
frontmatter" refusal every other unusable file gets. An empty file is
what `touch`, a crashed editor or a partial checkout leaves behind
(`CAPTURE-12`).

**`hi export` and `hi issue` no longer pass hi's own starter prompt off
as your prose.** A day-one repository exported
`"intent": "<!-- What is this for... -->"` for every file, and a
`product` ending in the generated `## Features` heading, and `hi issue`
printed that question back as the author's intent. `hi view` and
`hi check` already refused to; the two verbs that feed an agent and a
tracker did not. `view::strip_comments` and `view::strip_index` are now
public and all three read prose the same way (`EXPORT-5`, `ISSUE-6`).

Both have regression tests.

### The counts stop drifting

`INTENT.md`'s generated index had fallen five criteria behind, and
nothing caught it, so three published surfaces each claimed a different
total. `fledge lanes run verify` now regenerates the index and fails if
that changed anything. The README and the docs site no longer restate a
total by hand: the count per feature is generated into `INTENT.md`, and
a number maintained in three places by hand is a number that ends up
saying three different things.

### Specs that lied about the code

`specs/out/` documented two bugs as current behavior that the code fixed
in 0.2.0, with open tasks asking someone to implement what already
ships. An agent working from those specs, which is the workflow hi
exists to feed, would have regressed them.

`specs/view/view.spec.md` claimed only `src/view.rs`, leaving the five
`include_str!` assets outside the gate, including the CSS that carried
the `[hidden]` bug for four releases. spec-sync goes from 61% to 100%
file coverage.

## [0.3.1] 2026-09-16

### Linux arm64 binaries, and the page is published

No code changed. This release exists to carry two things the 0.3.0 tag
could not.

**Linux arm64.** The release workflow now builds
`aarch64-unknown-linux-gnu` on a native arm64 runner, so there is no
cross-linker to keep working. That was the one gap in the Homebrew
formula, which covered macOS on both architectures and Linux x86_64
only.

**The intent page is published.** `corvidlabs.github.io/hi` is hi's own
`hi view` output: the real 119 criteria in `hi/`, rendered by the real
binary on every push. The demo is the artifact rather than a mock-up of
it, so a page that is wrong is wrong for everybody at the same time
(`VIEW-19`, `VIEW-19.a`). The same deployment carries the Atlas coverage
badges at `/badges/`, which is where corvidlabs.xyz reads them from.

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

### Criteria are directions, which closes the open question in section 21

Section 21 logged a complaint two readers made independently: a reader cannot tell whether a
criterion is met, so on the question they most want answered the document sends them to the code.
It had no answer.

The answer is that it is the wrong question asked of the wrong layer. A criterion says what the
thing should be, not what it currently does, and directions stay correct through a wrong turn. A
criterion that is false right now means the code has not arrived yet, not that the criterion is
wrong. Whether the code has arrived belongs to something that reads code against contracts, which
`hi export` already feeds.

The page eyebrow now reads "What this should be" rather than "What we said we wanted", which was
past tense and read as a report of decisions.

Section 21's three properties survive as requirements on whoever builds the checking layer:
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
reader can learn a standard from. DECISIONS.md section 20.

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
