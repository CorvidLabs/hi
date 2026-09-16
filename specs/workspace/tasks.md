---
spec: workspace.spec.md
---

## Tasks

- [ ] Add a `#[cfg(test)]` module to `src/workspace.rs` covering `Workspace::find`: the ancestor walk from a nested directory, the nearest-recognized-`hi/`-wins case across two nested workspaces, and the not-found error message (REQ-workspace-001). The cli tests cover the walk end to end but nothing exercises `find` in isolation.
- [ ] Cover the `.git` fallback, which is the one branch of `find` no test reaches: a repo root with `.git` and no `hi/`, plus the case that now matters most, which is a nearer `.git` with a farther real `hi/` above it, where the farther workspace must win (REQ-workspace-001, hi: CAPTURE-1.a).
- [ ] Cover `holds_hi_files` directly: a directory with no `.md` at all, one whose only `.md` has no frontmatter, one whose file is not valid UTF-8, one where `hi` is a file rather than a directory, and a `hi:` key sitting below other frontmatter keys (REQ-workspace-009, hi: CAPTURE-6).
- [ ] Cover `Workspace::load` selection rules directly: sorted order across several files, a `.txt` skipped, a `hi/sub/` not walked, a `.MD` loaded, and a missing `hi/` yielding an empty workspace (REQ-workspace-002).
- [ ] Add a *unit* test for the `doc_for_family` resolve-by-use fallback, on a file whose frontmatter declares nothing but whose criteria use the family. The fallback is covered end to end today by `tests/cli.rs::block_style_frontmatter_is_understood_and_preserved`, but nothing calls `doc_for_family` on an undeclared family directly (REQ-workspace-004, hi: FILE-2).
- [ ] Assert `families()` over a file that declares its list in YAML block form. `front.families` is proven to be filled from a block list, but only through `check`'s per-doc undeclared-family test and through capture. No fixture that asserts a family list writes one in block form (REQ-workspace-007).
- [ ] Cover the retired paths that have no test today: `find_id` returning a retired criterion, and `next_free` returning one past a retired number (REQ-workspace-005, REQ-workspace-006).
- [ ] Cover `families()` unioning declared and used families across two docs, deduplicating and sorting (REQ-workspace-007).
- [ ] Add a direct assertion for `intent_path`. Its value is already proven end to end (`out::write_index` reads and rewrites exactly the path it returns, and three `hi index` tests in `tests/cli.rs` assert the rewritten `INTENT.md` at the repo root), but no test names the path itself, and the unit-level `export(workspace, None)` path only executes it against a `/r` root where the read fails (REQ-workspace-008).

## Gaps

- `Workspace::find` has no unit test. `tests/cli.rs` now covers the walk, the recognition test and the `no hi/ directory found` error through the real binary, but nothing calls `find` directly, and the `.git` fallback that makes a first-run repo work still ships unverified. The cli `Repo` helper never creates a `.git`.
- `holds_hi_files` is only covered through its two happy-path outcomes (a Hindi locale directory rejected, a BOM'd file accepted). Its failure branches (unlistable directory, unreadable file, non-UTF-8 file) are unexercised.
- `intent_path` has no direct assertion. It is verified indirectly and solidly by the `hi index` tests in `tests/cli.rs`, which write `INTENT.md` at the repo root and then read back exactly what `out::write_index` rewrote at `workspace.intent_path()`; at the unit level it is only executed, through `export(workspace, None)` → `read_product_intent` against a fake `/r` root where the read returns `None`.
- Nothing in hi reports two files declaring the same family in their frontmatter. `doc_for_family` silently takes the earlier one, and `check`'s six problem kinds all key on ids or on where a line sits, never on the frontmatter `families:` lists of two different files, so the second file's criteria resolve to the first file. Whether that should be a `check` finding belongs to the `check` spec, not here; it is recorded because this module's first-wins behavior is what makes it invisible.
- `load` is exercised on a two-file `hi/` only by `a_case_lands_in_the_file_holding_its_parent`, and that test passes whichever order the two come back in, so the sort is executed but never asserted. The extension filter and the sub-directory exclusion are unexercised entirely.
- The read-only guarantee (REQ-workspace-003) is asserted only indirectly, through capture tests that check a file was not written on an error path.
- `capture::create_file` can push a second `Doc` for a path that `load` already parsed, when the file exists but claims no matching family. It belongs to the `capture` spec, but it is why `docs` cannot be assumed to hold unique paths here.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
