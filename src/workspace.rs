//! Finding and loading the `hi/` directory.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::doc::Doc;
use crate::id::Id;

/// Every hi file in one repository, plus where they live.
pub struct Workspace {
    /// The directory containing `hi/`.
    pub root: PathBuf,
    /// The `hi/` directory itself.
    pub dir: PathBuf,
    pub docs: Vec<Doc>,
    /// Files in `hi/` that are hi's own rather than criteria, kept so `check`
    /// can look inside them instead of pretending they are not there.
    pub skipped: Vec<PathBuf>,
}

/// Why nothing reads a criterion-shaped line where it sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrayPlace {
    /// In a criteria file, but outside `## Criteria` and `## Retired`.
    OutsideSection,
    /// In a file hi does not load as criteria at all, because its name is not
    /// lowercase and is therefore hi's own (DECISIONS.md §27).
    UnreadFile,
}

/// One criterion-shaped line hi cannot read as structure, located.
///
/// The id on it is taken whatever hi can do with the line, so this is both what
/// `check` reports and what `capture` refuses to hand out (hi: CAPTURE-14).
#[derive(Debug, Clone)]
pub struct Stray {
    /// Workspace-relative path of the file holding the line.
    pub file: String,
    /// One-based line number, the way an editor counts.
    pub line: usize,
    /// The id-shaped token exactly as the file has it.
    pub token: String,
    /// Why nothing reads it.
    pub place: StrayPlace,
}

impl Workspace {
    /// Walk up from `start` looking for a `hi/` directory, stopping at the
    /// filesystem root or a `.git` boundary that has no `hi/` beside it.
    pub fn find(start: &Path) -> Result<Workspace> {
        let start =
            fs::canonicalize(start).with_context(|| format!("resolving {}", start.display()))?;

        // `hi` is also the ISO code for Hindi, so `public/locales/hi/` is a real
        // directory in real repositories. A directory is only this tool's
        // workspace if something inside it is actually a hi file.
        let mut cursor: Option<&Path> = Some(&start);
        while let Some(dir) = cursor {
            if holds_hi_files(&dir.join("hi")) {
                return Workspace::load(dir);
            }
            // A repository is a boundary. Without this, adding hi to a project
            // that happens to sit inside another one adopts the outer
            // project's criteria as if they were yours (hi: CHECK-1).
            if dir.join(".git").exists() {
                return Workspace::load(dir);
            }
            cursor = dir.parent();
        }

        bail!(
            "this is not a repository, and no hi/ directory was found above it. \
             hi anchors to a repository, so run it inside one"
        )
    }

    /// Load every lowercase `*.md` directly inside `<root>/hi`.
    ///
    /// A criteria file is lowercase, because `capture::start_file` lowercases
    /// every family name. An uppercase name is therefore never one hi wrote,
    /// and is hi's own: `AGENTS.md` and the `CLAUDE.md` beside it carry the
    /// instruction an agent reads, not criteria (DECISIONS.md §27).
    ///
    /// Skipped paths are kept rather than dropped. `check` scans them for
    /// criterion-shaped lines, because a criterion hi cannot see must be
    /// reported and never silently ignored (hi: FILE-20).
    pub fn load(root: &Path) -> Result<Workspace> {
        let dir = root.join("hi");
        let mut docs = Vec::new();
        let mut skipped = Vec::new();

        if dir.is_dir() {
            let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
                .with_context(|| format!("reading {}", dir.display()))?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file()
                        && path
                            .extension()
                            .map(|e| e.eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                })
                .collect();
            paths.sort();

            for path in paths {
                if is_hi_own_file(&path) {
                    skipped.push(path);
                    continue;
                }
                let doc = Doc::load(&path)?;
                if let Some(declared) = doc.front.unreadable_version() {
                    bail!("{}", unreadable_version(&rel_to(root, &path), declared));
                }
                docs.push(doc);
            }
        }

        Ok(Workspace {
            root: root.to_path_buf(),
            dir,
            docs,
            skipped,
        })
    }

    /// Path to the product-level intent file.
    pub fn intent_path(&self) -> PathBuf {
        self.root.join("INTENT.md")
    }

    /// Total criteria, active only.
    pub fn criteria_count(&self) -> usize {
        self.docs.iter().map(|d| d.criteria.len()).sum()
    }

    /// Find which doc holds a family, by frontmatter declaration or by use.
    ///
    /// Returns the first path-sorted file that declares it, then the first
    /// that uses it. That is a lookup, not a decision: when two files declare
    /// the same family, `hi check` reports `duplicate-family` and capture
    /// refuses a new top-level id rather than writing into this result
    /// (hi: CHECK-2.g, CAPTURE-16).
    pub fn doc_for_family(&self, family: &str) -> Option<usize> {
        self.docs
            .iter()
            .position(|doc| doc.front.families.iter().any(|f| f == family))
            .or_else(|| {
                self.docs.iter().position(|doc| {
                    doc.all()
                        .any(|c| c.id.as_ref().is_some_and(|id| id.family == family))
                })
            })
    }

    /// Every doc whose frontmatter lists this family, in load order.
    ///
    /// Load order is path-sorted, so this is stable for a given tree and
    /// unstable across a rename — which is why a length other than one is a
    /// problem, not a tie-break (hi: CHECK-2.g).
    pub fn family_declarers(&self, family: &str) -> Vec<usize> {
        self.docs
            .iter()
            .enumerate()
            .filter(|(_, doc)| doc.front.families.iter().any(|f| f == family))
            .map(|(index, _)| index)
            .collect()
    }

    /// Look up one criterion by id, active or retired.
    pub fn find_id(&self, id: &Id) -> Option<(usize, &crate::doc::Criterion)> {
        for (index, doc) in self.docs.iter().enumerate() {
            if let Some(found) = doc.all().find(|c| c.id.as_ref() == Some(id)) {
                return Some((index, found));
            }
        }
        None
    }

    /// Every criterion-shaped line hi does not read as structure, wherever it
    /// sits: in a criteria file outside every section, or in a file hi does not
    /// read as criteria at all.
    ///
    /// One list rather than two scans, because `check` and `capture` have to
    /// agree about which ids are taken. They did not: `find_stray` walked
    /// `docs` and `check` separately walked `skipped`, so a retired `SEND-1`
    /// in `hi/Archive.md` was reported by `check` and handed out again by
    /// capture with different words. An id reported as used and then reissued
    /// is worse than one nobody noticed (hi: CAPTURE-14, FILE-20,
    /// DECISIONS.md §27, §32).
    ///
    /// A skipped file hi cannot read is an answer hi does not have, and it is
    /// returned as the failure it is. Swallowing the read error made this
    /// lookup say "free" about an id it could not see, and because `check` and
    /// `capture` now share the lookup, both agreed on the same wrong answer:
    /// `check` exited 0 and `capture` reissued a reserved id (hi: CAPTURE-15,
    /// DECISIONS.md §36).
    pub fn strays(&self) -> Result<Vec<Stray>> {
        let mut found = Vec::new();

        // A file hi skips is still a file somebody may have written a
        // criterion into. Skipping quietly is the FILE-20 failure with a new
        // cause, so look inside rather than assume (DECISIONS.md §27).
        for path in &self.skipped {
            // Built by hand rather than with `with_context`, so the cause
            // stays on the error line and the hint is the last thing read,
            // which is the shape every other refusal in hi has.
            let raw = fs::read_to_string(path).map_err(|err| {
                anyhow!(
                    "reading {}: {err}\n\
                     hint:  hi has to read it before it can say whether an id is already taken, \
                     and hi files are UTF-8 text. Fix that one or move it out of hi/, and \
                     nothing else has to change",
                    self.rel(path)
                )
            })?;
            let file = self.rel(path);
            for (line, token) in crate::doc::criterion_tokens(&raw) {
                found.push(Stray {
                    file: file.clone(),
                    line: line + 1,
                    token,
                    place: StrayPlace::UnreadFile,
                });
            }
        }

        for doc in &self.docs {
            let file = self.rel(&doc.path);
            for (line, token) in &doc.stray {
                found.push(Stray {
                    file: file.clone(),
                    line: line + 1,
                    // Recorded as written, emphasis and all: `**SEND-2**`. It
                    // is quoted back at the person, so it is the line's own
                    // text rather than hi's reading of it.
                    token: token.clone(),
                    place: StrayPlace::OutsideSection,
                });
            }
        }

        Ok(found)
    }

    /// A criterion-shaped line hi could not read, but which spoke for this id.
    ///
    /// An id written where nothing parses it, outside every section, inside a
    /// fence, or in a file hi does not read as criteria, has still been used.
    /// Handing it out again produces two lines with the same id and different
    /// sentences, which is the one thing hi promises cannot happen, so capture
    /// refuses and points at the line (hi: FILE-20, CAPTURE-14).
    pub fn find_stray(&self, id: &Id) -> Result<Option<Stray>> {
        let wanted = id.to_string().to_ascii_uppercase();
        Ok(self
            .strays()?
            .into_iter()
            .find(|stray| crate::doc::strip_emphasis(&stray.token).to_ascii_uppercase() == wanted))
    }

    /// The next free top-level number in a family.
    pub fn next_free(&self, family: &str) -> u32 {
        let highest = self
            .docs
            .iter()
            .flat_map(|doc| doc.all())
            .filter_map(|c| c.id.as_ref())
            .filter(|id| id.family == family)
            .filter_map(|id| id.root_number())
            .max();
        // At the ceiling `n + 1` wrapped in release and printed
        // "next free is SEND-0", which is a wrong id offered as a hint rather
        // than a crash. Say there is no next one instead (hi: CAPTURE-13).
        highest.map(|n| n.saturating_add(1)).unwrap_or(1)
    }

    /// Every family in the workspace, sorted, from frontmatter and from use.
    pub fn families(&self) -> Vec<String> {
        let mut families: Vec<String> = Vec::new();
        for doc in &self.docs {
            for family in doc
                .front
                .families
                .iter()
                .cloned()
                .chain(doc.used_families())
            {
                if !families.contains(&family) {
                    families.push(family);
                }
            }
        }
        families.sort();
        families
    }

    /// Display path relative to the root, always with forward slashes.
    ///
    /// These strings are not just for the terminal: they go into `hi export`
    /// JSON that an agent reads, into `hi issue` bodies, and into markdown
    /// links. A backslash is wrong in all three, so the separator is POSIX on
    /// every platform rather than the host's.
    pub fn rel(&self, path: &Path) -> String {
        rel_to(&self.root, path)
    }
}

/// `Workspace::rel`, before there is a `Workspace` to ask.
///
/// `load` refuses a file whose format version hi cannot read, and the refusal
/// names the file, so it needs this one line of the workspace before the
/// workspace exists (hi: FILE-25, FILE-12).
fn rel_to(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// What to say to somebody whose file was written for a different hi.
///
/// An operational failure and never a structural problem. The
/// kinds are things hi found wrong *in* a file it read; this is hi saying it did
/// not read the file at all, which is the same sentence as "unreadable" and
/// already has a home (hi: CHECK-1, FILE-25, DECISIONS.md §36, §38).
///
/// It is raised in `load`, which every verb goes through before it does
/// anything else, so the refusal is in front of every read and every write at
/// once — and in front of `lock::acquire`, so a refused capture has not even
/// made `hi/` (hi: FILE-25.a, CAPTURE-5).
fn unreadable_version(file: &str, declared: &str) -> String {
    format!(
        "{file} says `hi: {declared}`, and this hi reads HI/{v} only.\n\
         hint:  a newer hi may understand it, and this one will not guess. Nothing in this \
         repository is read or written while that file is here, because reading HI/{declared} as \
         HI/{v} is how a format version stops meaning anything",
        v = crate::doc::FORMAT_VERSION
    )
}

/// True when a file in `hi/` belongs to hi rather than to the person.
///
/// The test is the first character of the file name. Capture lowercases every
/// family name when it starts a file, so a criteria file hi wrote is always
/// lowercase and an uppercase name is always something else (DECISIONS.md §27).
fn is_hi_own_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.chars().next())
        .is_some_and(|first| first.is_ascii_uppercase())
}

/// True when a directory holds at least one file this tool would recognize:
/// a markdown file whose frontmatter carries a `hi:` key.
fn holds_hi_files(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file()
            || !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("md"))
        {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        // Strip a BOM here too, or an editor-written marker hides the file
        // from discovery just as it would from the parser.
        let raw = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
        let mut lines = raw.lines();
        if lines.next().map(str::trim_end) != Some("---") {
            continue;
        }
        for line in lines {
            if line.trim_end() == "---" {
                break;
            }
            if line.split_once(':').map(|(key, _)| key.trim()) == Some("hi") {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_uppercase_file_is_hi_s_own_and_is_never_read_as_criteria() {
        let root = std::env::temp_dir().join(format!("hi-ws-upper-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
        fs::write(root.join("hi/AGENTS.md"), "# Human intent\n").unwrap();
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n",
        )
        .unwrap();

        let workspace = Workspace::load(&root).unwrap();

        assert_eq!(workspace.docs.len(), 1, "AGENTS.md is not a criteria file");
        assert_eq!(workspace.skipped.len(), 1, "but it is not forgotten either");
        assert!(workspace.skipped[0].ends_with("AGENTS.md"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn an_id_in_a_file_hi_skips_is_still_taken() {
        // `find_stray` walked `docs` and `check` separately walked `skipped`,
        // so a retired SEND-1 parked in an uppercase file was reported by one
        // and handed out again by the other (hi: CAPTURE-14).
        let root = std::env::temp_dir().join(format!("hi-ws-reserve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
        fs::write(
            root.join("hi/Archive.md"),
            "# Archive\n\n## Retired\n\n- **SEND-1**  the old way of sending.\n",
        )
        .unwrap();
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-2**  One.\n",
        )
        .unwrap();

        let workspace = Workspace::load(&root).unwrap();

        let found = workspace
            .find_stray(&Id::parse("SEND-1").unwrap())
            .unwrap()
            .expect("an id written where hi cannot read it is still taken");
        assert_eq!(found.file, "hi/Archive.md");
        assert_eq!(found.line, 5);
        assert_eq!(found.place, StrayPlace::UnreadFile);
        // And an id nobody wrote anywhere is still free.
        assert!(
            workspace
                .find_stray(&Id::parse("SEND-3").unwrap())
                .unwrap()
                .is_none()
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_file_hi_cannot_read_is_a_failure_and_not_an_empty_answer() {
        // The read error used to be swallowed, so the lookup said "free" about
        // an id it had not been able to look for. Because `check` and `capture`
        // share the lookup, both agreed on the same wrong answer: `check`
        // exited 0 and `capture` reissued a reserved id (hi: CAPTURE-15,
        // DECISIONS.md §36).
        let root = std::env::temp_dir().join(format!("hi-ws-unreadable-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
        // One Latin-1 byte in the retirement reason is enough.
        fs::write(
            root.join("hi/Archive.md"),
            b"# Archive\n\n## Retired\n\n- **SEND-1**  The old way.\n  retired: Old caf\xe9.\n"
                .as_slice(),
        )
        .unwrap();
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-2**  One.\n",
        )
        .unwrap();

        let workspace = Workspace::load(&root).unwrap();

        let err = workspace
            .strays()
            .expect_err("unreadable is not absent")
            .to_string();
        assert!(err.contains("hi/Archive.md"), "it names the file: {err}");
        assert!(
            workspace.find_stray(&Id::parse("SEND-1").unwrap()).is_err(),
            "and the lookup capture refuses by cannot answer either"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_file_written_for_another_format_version_is_refused_by_every_verb_at_once() {
        // The refusal lives in `load` rather than in each verb, because every
        // verb goes through `load` before it does anything else. That is what
        // makes "hi does not touch this repository" true of the read verbs and
        // the write verbs at the same time, without seven places to keep in
        // step (hi: FILE-25, FILE-25.a).
        let root = std::env::temp_dir().join(format!("hi-ws-version-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 2\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n",
        )
        .unwrap();

        let err = match Workspace::load(&root) {
            Err(err) => err.to_string(),
            Ok(_) => panic!("HI/2 is not HI/1"),
        };
        assert!(err.contains("hi/chat.md"), "it names the file: {err}");
        assert!(err.contains("hi: 2"), "and the version: {err}");

        // A neighbour at this version does not rescue it: one unreadable file
        // is a repository hi cannot answer questions about.
        fs::write(
            root.join("hi/other.md"),
            "---\nhi: 1\nfamilies: [RECEIPT]\n---\n\n## Criteria\n\n- **RECEIPT-1**  Two.\n",
        )
        .unwrap();
        assert!(Workspace::load(&root).is_err(), "one file is enough");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hi_s_own_files_hold_no_criteria() {
        // The reservation lookup reads inside hi's own files, so the habit hi
        // writes there must not read as structure. It is prose, a numbered
        // list and a fenced example (DECISIONS.md §27).
        let root = std::env::temp_dir().join(format!("hi-ws-own-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
        fs::write(root.join("hi/AGENTS.md"), crate::out::agent_instructions()).unwrap();
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n",
        )
        .unwrap();

        let workspace = Workspace::load(&root).unwrap();

        assert!(
            workspace.strays().unwrap().is_empty(),
            "hi's own instruction file is not a file that holds criteria: {:?}",
            workspace.strays().unwrap()
        );
        assert_eq!(workspace.criteria_count(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn relative_paths_always_use_forward_slashes() {
        let workspace = Workspace {
            root: PathBuf::from("/r"),
            dir: PathBuf::from("/r/hi"),
            docs: Vec::new(),
            skipped: Vec::new(),
        };
        // These strings reach exported JSON, issue bodies and markdown links,
        // so they must not depend on the host's separator.
        assert_eq!(
            workspace.rel(&PathBuf::from("/r").join("hi").join("chat.md")),
            "hi/chat.md"
        );
        assert_eq!(
            workspace.rel(&PathBuf::from("/r").join("INTENT.md")),
            "INTENT.md"
        );
    }
}
