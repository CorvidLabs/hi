---
spec: workspace.spec.md
---

## Key Decisions

- **This module has no hi family.** `capture`, `check`, `out` and `view` each answer to one; `workspace` is the infrastructure underneath all of them. Its only intent obligation is indirect: keep the `hi/` directory a folder of ordinary markdown so `FILE-1`, `FILE-1.a` and `FILE-2` stay true. There is no index file, no lockfile, no cache and no database anywhere in hi, and that is why.
- **Discovery is the directory name plus one line of proof.** A directory is the workspace because it is named `hi` *and* holds a `*.md` whose frontmatter carries a `hi:` key. The name alone is not enough: `hi` is ISO 639-1 for Hindi, so `public/locales/hi/` is a real directory in real repositories, and before this check hi would happily adopt one and write a feature file into someone's translations (hi: CAPTURE-6). There is still no config file to find and no marker to plant: the proof is the one machine-facing line the file already had.
- **`.git` is a fallback root, not a stop.** This reversed in the fix pass. `find` now records the *nearest* `.git` ancestor in `git_root` and keeps climbing; it loads that root only if the whole walk finishes without finding a recognized `hi/`. The reason is the locale case: an unrecognized `hi/` has to be walkable *past*, and a nested `.git` between you and the real workspace must not trap the walk either. The fallback still exists so the *first* `hi FAMILY-1 "..."` on a fresh repo succeeds. It lands you at the repo root with an empty workspace, and `capture::create_file` then does `fs::create_dir_all(&workspace.dir)` (hi: CAPTURE-1.a). **If you ever "optimize" the `.git` branch back into an early `return`, the Hindi-locale test breaks and discovery starts preferring a nearer repo root over a farther real workspace.**
- **Recognition gates discovery, not loading.** `holds_hi_files` decides only which directory the walk stops at. Once a root is chosen, `load` reads every `*.md` directly inside `<root>/hi` whether or not it declares `hi:`. That is deliberate: a hand-written file that forgot the version line must still load, still be counted and still be checked. Do not reuse `holds_hi_files` as a filter inside `load`.
- **`--root` is a starting point, not a boundary.** `main` no longer lets clap own the flag alone: `peel_root` takes `--root PATH` / `--root=PATH` out of argv *before* routing, because capture never reaches clap and the flag would otherwise be joined into the criterion's sentence (hi: CAPTURE-8). The subcommand path still gets its own copy from clap (`cli.root`, since `Cli::parse()` reads `args_os` for itself and sees the flag `peel_root` never removed from the process's arguments), so the two paths agree on the value without sharing the code that found it. Either way what reaches `find` is a starting directory it walks *up* from. `hi --root some/sub check` can therefore resolve to an ancestor's `hi/`. This is deliberate ergonomics, but it does mean `--root` is not a sandbox.
- **An empty workspace is valid.** `load` on a root with no `hi/` returns `docs == []` and a `dir` that does not exist. Every consumer has to tolerate that; `capture` is the one that creates the directory.
- **The frontmatter's *style* is none of this module's business.** `doc` now parses `families:` in both inline (`families: [A, B]`) and YAML block form, keeps `families_span` and `families_block` so it can rewrite the list in whatever style the file already used, and also accepts the singular `family:` key. All three land in the same `front.families`, which is the only part of it this module reads. Do not reach for `families_span` or `families_block` here: nothing in discovery or resolution should depend on how a person chose to write the list.
- **Frontmatter is a shortcut, not the authority.** DECISIONS §3 says the `families:` list is what lets capture resolve an id without scanning, but `doc_for_family` scans anyway as a fallback, and `families()` unions declared with used. A hand-written file that declares nothing still works. That is what keeps the one machine-facing line optional in practice (hi: FILE-2).
- **Retired is not gone.** `doc_for_family`, `find_id` and `next_free` all go through `Doc::all()`, which chains `criteria` with `retired`. Only `criteria_count()` is active-only. If you ever "optimize" one of those to iterate `doc.criteria`, you silently allow a retired id to be reissued, which DECISIONS §4 forbids.
- **Indices, not borrows.** `doc_for_family` and `find_id` return `usize` positions into `docs` because `capture` needs `&mut workspace.docs[index]` immediately afterwards; returning `&Doc` would borrow the workspace immutably for the rest of the call. The cost is that an index is invalidated by any push to `docs`.
- **Stray is not claimed.** `Doc::all()` chains `criteria` with `retired`, but *not* `stray`, the new list of criterion-shaped lines found outside every section. That is correct: a stray line is a mistake `check` reports (`Kind::StrayCriterion`, hi: CHECK-2.e), not an id that has been claimed. Do not "helpfully" chain `stray` into `all()` to make `next_free` more conservative; it would make a typo reserve a number forever.
- **The module is strictly read-only.** Nothing here opens a file for writing; `holds_hi_files` reads and nothing more. If a future verb needs to write, it goes in `doc` or in the verb's own module. `doc::write_atomically` is where hi's one durable-write pattern lives (temp sibling, flush, sync, rename); there is no reason for a second copy of it here (hi: FILE-8).

## Files to Read First

- `src/workspace.rs`: the whole module, under 200 lines. `find`, `load` and `holds_hi_files` are the only I/O; the rest are linear scans over `docs`.
- `src/doc.rs`: `Doc::load`, `Doc::parse` (note the BOM strip and the `newline` detection at the top), and especially `Doc::all()` (line ~372) and `Doc::used_families()` (line ~377). Three of this module's five lookups are thin wrappers over `all()`.
- `tests/cli.rs`, the only tests that execute `Workspace::find`. Its `Repo` helper builds a `hi/` and no `.git`, so most of that file is also a smoke test of recognition. `a_hindi_locale_directory_is_not_mistaken_for_a_workspace`, `a_bom_does_not_make_a_valid_file_look_broken` and `outside_a_repository_it_says_so_rather_than_guessing` are the three that target discovery on purpose.
- `src/capture.rs`, the only consumer that mutates a `Workspace`. `capture()` shows the `find_id` → `next_free` → `doc_for_family` sequence, and `create_file()` shows the `dir`-creation path.
- `src/main.rs`: `run_capture` and `run` are the only two call sites of `Workspace::find` in the crate. `peel_root` above them is where `--root` is taken out of argv before capture routing; the clap path reads the same flag independently as `cli.root`, so both call sites end up with the same starting directory by two different routes.
- `src/view.rs`, the newest consumer. `render` walks `docs` and calls `criteria_count`; `write` uses `intent_path`, `root` and `rel`.
- `DECISIONS.md` §3 (layout and frontmatter), §4 (ids are permanent and never reused), §5 (no state, no lifecycle, no evidence), §8.3 (retiring does not free a number).

## Current Status

Fully implemented. The one shape change since the initial build is discovery: `find` gained the
`holds_hi_files` recognition test and demoted `.git` from a stop to a remembered fallback. It
still has **no inline `#[cfg(test)]` module of its own**: the lookups are covered indirectly
through the `capture`, `check`, `out` and `view` test modules, and discovery is now covered from
outside by `tests/cli.rs`, which drives the real binary.

Two gaps remain. `Workspace::find` has no *unit* test: the cli tests cover the walk, the
recognition test and the not-found message, but not the `.git` fallback (no cli fixture creates a
`.git`, so the first-capture-in-a-fresh-repo path ships unverified) and not nearest-wins across
two nested workspaces. The second is narrower than it used to be: `intent_path` is now verified
end to end by the `hi index` tests in `tests/cli.rs` (`out::write_index` reads and rewrites
exactly the path `intent_path` returns, so a test that writes `INTENT.md` at the repo root and
reads back the rewritten body is asserting that path), but nothing names the path directly, and at
the unit level it is only executed, through a whole-repo `export` reaching `read_product_intent`
against the fake `/r` root where the read fails. Both gaps are recorded in `tasks.md`.

## Notes

- **A latent duplicate-doc path worth knowing about.** `capture::create_file` is called when `doc_for_family` returns `None`. If `hi/<family>.md` already exists on disk but declares and uses no matching family, `load` has already parsed it into `docs`, and `create_file` then does `Doc::load` on the same path and pushes it again, leaving two `Doc`s for one file in the same workspace. It is reachable only from an odd hand-edited state and it belongs to `capture`, not here, but it is the reason `docs` cannot be assumed to hold unique paths.
- `docs` is sorted by path at load, but `capture` appends, so "sorted" is a load-time property only. `families()` sorts its own output, so it is unaffected.
- `rel` uses `strip_prefix(&self.root).unwrap_or(path)`. That `unwrap_or` is the fallback for a path outside the root, not a force-unwrap. Keep it.
- `find` resolves symlinks via `fs::canonicalize`, so `root` is always a real path. Output paths are therefore canonical, which matters when comparing `hi check` output across machines.
- The `.md` extension test is `eq_ignore_ascii_case`, so `CHAT.MD` loads. Nothing else in hi depends on that, but a stricter test would be a silent behavior change.
- **The BOM strip is deliberately duplicated.** `Doc::parse` strips a leading `\u{feff}` and so does `holds_hi_files`, because they are two independent readers of the same bytes: one parses the file, the other only sniffs whether the directory is a workspace. Deleting the one in `holds_hi_files` would leave a BOM'd file parseable but undiscoverable, which is the worse of the two failures. The walk would climb past the real workspace entirely (hi: FILE-11). Keep both.
- **`holds_hi_files` is a deliberately separate mini-parser.** It does not construct a `Doc`: it reads the file, takes lines until the closing `---`, and looks for a key of exactly `hi`. It runs at *every* level of the walk, reading each `*.md` in the candidate directory until one qualifies, so it must stay cheap and must never fail; it answers `bool`, and every I/O or encoding error reads as "not a hi file". Replacing it with `Doc::parse` would pull the whole parser into the walk and give it an error path it cannot have.
- **The `///` comment on `find` is stale.** It still reads "stopping at the filesystem root or a `.git` boundary that has no `hi/` beside it", which is the pre-fix rule; the body below it records `.git` and keeps climbing. The code is right and the comment is wrong. Trust the body, and fix the comment the next time the file is touched. This spec is not the place to correct it.
- The recognition test accepts a `hi:` key found anywhere in the frontmatter block, and `key.trim()` means indentation and trailing spaces do not matter. It does not care about the value, so `hi: 1` and `hi: 2` both count; version handling is not discovery's job.
- Scale assumption: every lookup is a linear scan over every criterion in every doc. hi is sized for tens of files and hundreds of criteria, so no index is warranted, and adding one would be the kind of machine-facing state DECISIONS §5 rules out.
