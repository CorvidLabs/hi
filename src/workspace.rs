//! Finding and loading the `hi/` directory.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::doc::Doc;
use crate::id::Id;

/// Every hi file in one repository, plus where they live.
pub struct Workspace {
    /// The directory containing `hi/`.
    pub root: PathBuf,
    /// The `hi/` directory itself.
    pub dir: PathBuf,
    pub docs: Vec<Doc>,
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

    /// Load every `*.md` directly inside `<root>/hi`.
    pub fn load(root: &Path) -> Result<Workspace> {
        let dir = root.join("hi");
        let mut docs = Vec::new();

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
                docs.push(Doc::load(&path)?);
            }
        }

        Ok(Workspace {
            root: root.to_path_buf(),
            dir,
            docs,
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

    /// Look up one criterion by id, active or retired.
    pub fn find_id(&self, id: &Id) -> Option<(usize, &crate::doc::Criterion)> {
        for (index, doc) in self.docs.iter().enumerate() {
            if let Some(found) = doc.all().find(|c| c.id.as_ref() == Some(id)) {
                return Some((index, found));
            }
        }
        None
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
        highest.map(|n| n + 1).unwrap_or(1)
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
        let relative = path.strip_prefix(&self.root).unwrap_or(path);
        relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    }
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
    fn relative_paths_always_use_forward_slashes() {
        let workspace = Workspace {
            root: PathBuf::from("/r"),
            dir: PathBuf::from("/r/hi"),
            docs: Vec::new(),
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
