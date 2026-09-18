//! Structural validation.
//!
//! `hi check` never judges a criterion's content. It reports only things that
//! make a file structurally wrong: a duplicate id, a case with no parent, an id
//! that collides with a retired one, or a line that cannot be parsed as an id.
//! Incomplete intent is the normal state of intent and is never an error.

use std::collections::HashMap;

use serde::Serialize;

use crate::doc::Section;
use crate::id::Id;
use crate::workspace::{Stray, StrayPlace, Workspace};

/// What kind of structural problem this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    DuplicateId,
    OrphanCase,
    RetiredCollision,
    UnparseableId,
    UndeclaredFamily,
    StrayCriterion,
}

impl Kind {
    pub fn code(self) -> &'static str {
        match self {
            Kind::DuplicateId => "duplicate-id",
            Kind::OrphanCase => "orphan-case",
            Kind::RetiredCollision => "retired-collision",
            Kind::UnparseableId => "unparseable-id",
            Kind::UndeclaredFamily => "undeclared-family",
            Kind::StrayCriterion => "stray-criterion",
        }
    }
}

/// One structural problem, located.
#[derive(Debug, Clone, Serialize)]
pub struct Problem {
    pub kind: Kind,
    pub file: String,
    pub line: usize,
    pub id: String,
    pub message: String,
}

/// Everything `hi check` found.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// A note about the product-level intent, when there is something to say.
    /// Never a problem: hi does not fail on unfinished intent (hi: CHECK-1).
    pub note: Option<String>,
    pub files: usize,
    pub criteria: usize,
    pub retired: usize,
    pub families: Vec<String>,
    pub problems: Vec<Problem>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.problems.is_empty()
    }
}

/// What to say about the product-level intent, if anything.
fn product_intent_note(workspace: &Workspace) -> Option<String> {
    if workspace.docs.is_empty() {
        return None;
    }
    let path = workspace.intent_path();
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Some("no INTENT.md yet. `hi index` starts one for the product-level why".into());
    };
    // Strip the generated index and the starter comment; if nothing human is
    // left, the why has not been written.
    let prose: String = raw
        .lines()
        .filter(|l| !l.trim_start().starts_with("<!--") && !l.trim_start().starts_with("- ["))
        .filter(|l| !l.trim().starts_with('#'))
        .filter(|l| !l.contains("hi:index"))
        .filter(|l| !l.trim_start().starts_with("Write it as a person"))
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();
    prose
        .is_empty()
        .then(|| format!("{} has no product-level why yet", workspace.rel(&path)))
}

/// Run every structural check across the workspace.
pub fn run(workspace: &Workspace) -> Report {
    let mut problems: Vec<Problem> = Vec::new();

    // Where each id was first seen, so duplicates can name the original.
    let mut seen: HashMap<String, (String, usize)> = HashMap::new();
    // Every retired id, to catch an active criterion reusing one.
    let mut retired: HashMap<String, String> = HashMap::new();

    for doc in &workspace.docs {
        let file = workspace.rel(&doc.path);
        for criterion in doc.all().filter(|c| c.section == Section::Retired) {
            if let Some(id) = &criterion.id {
                retired.insert(id.to_string(), file.clone());
            }
        }
    }

    // A criterion nothing reads, wherever it sits: outside every section in a
    // criteria file, or inside a file hi skips because its name is hi's own.
    // Both come from `Workspace::strays`, which is the same lookup `capture`
    // refuses an id by, so what is reported here and what is refused there can
    // never disagree again (hi: CAPTURE-14, FILE-20, DECISIONS.md §32).
    for stray in workspace.strays() {
        let Stray {
            file, line, token, ..
        } = &stray;
        problems.push(Problem {
            kind: Kind::StrayCriterion,
            file: file.clone(),
            line: *line,
            id: token.clone(),
            message: match stray.place {
                StrayPlace::UnreadFile => format!(
                    "{token} sits in {file}, which hi does not read as criteria \
                     because its name is not lowercase. Move it into a lowercase \
                     file under ## Criteria, because nothing reads it where it is"
                ),
                StrayPlace::OutsideSection => format!(
                    "{token} sits outside any section. Move it under ## Criteria or ## Retired, \
                     because nothing reads it where it is"
                ),
            },
        });
    }

    for doc in &workspace.docs {
        let file = workspace.rel(&doc.path);

        // Every id the file declares, for the orphan check.
        let present: Vec<String> = doc
            .all()
            .filter_map(|c| c.id.as_ref())
            .map(|id| id.to_string())
            .collect();

        for criterion in doc.all() {
            let Some(id) = &criterion.id else {
                let reason = criterion
                    .id_error
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "not a valid id".to_string());
                problems.push(Problem {
                    kind: Kind::UnparseableId,
                    file: file.clone(),
                    line: criterion.line_no(),
                    id: criterion.raw_id.clone(),
                    message: format!("'{}' is not a valid id: {reason}", criterion.raw_id),
                });
                continue;
            };

            let key = id.to_string();

            if let Some((first_file, first_line)) = seen.get(&key) {
                problems.push(Problem {
                    kind: Kind::DuplicateId,
                    file: file.clone(),
                    line: criterion.line_no(),
                    id: key.clone(),
                    message: format!(
                        "duplicate id {key}, already declared at {first_file}:{first_line}"
                    ),
                });
            } else {
                seen.insert(key.clone(), (file.clone(), criterion.line_no()));
            }

            // An active criterion must not reuse a retired id.
            if criterion.section == Section::Criteria
                && let Some(where_retired) = retired.get(&key)
            {
                problems.push(Problem {
                    kind: Kind::RetiredCollision,
                    file: file.clone(),
                    line: criterion.line_no(),
                    id: key.clone(),
                    message: format!(
                        "{key} is retired in {where_retired}; retired ids stay reserved forever"
                    ),
                });
            }

            // A case must hang off a parent that exists in the same file.
            if let Some(parent) = id.parent()
                && !present.contains(&parent.to_string())
            {
                problems.push(Problem {
                    kind: Kind::OrphanCase,
                    file: file.clone(),
                    line: criterion.line_no(),
                    id: key.clone(),
                    message: format!("{key} has no parent {parent}"),
                });
            }

            // A used family should be declared, so capture can resolve it.
            if criterion.section == Section::Criteria
                && !doc.front.families.iter().any(|f| f == &id.family)
            {
                problems.push(Problem {
                    kind: Kind::UndeclaredFamily,
                    file: file.clone(),
                    line: criterion.line_no(),
                    id: key.clone(),
                    message: format!(
                        "family {} is not in this file's frontmatter families list",
                        id.family
                    ),
                });
            }
        }
    }

    problems.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    let mut notes: Vec<String> = Vec::new();
    if let Some(note) = product_intent_note(workspace) {
        notes.push(note);
    }
    // The first thing check nags about that hi itself maintains. Capture and
    // retire keep the generated list current, so the one way left to make it
    // wrong is to type a criterion straight into a file, which FILE-14 allows
    // and no verb can see. A note, never a problem, and never the exit code
    // (hi: INDEX-4.b, CHECK-1, DECISIONS.md §30).
    if let Some(note) = crate::out::index_note(workspace) {
        notes.push(note);
    }
    // Retiring in a hurry is fine; never saying why is a decision nobody can
    // reconstruct later. A note rather than a problem, and never a failure.
    // Only the root of a retirement needs a reason. A case went with its
    // parent and was never a separate decision, so asking it to explain itself
    // is asking for the same sentence twice.
    let unexplained = workspace
        .docs
        .iter()
        .flat_map(|doc| {
            let retired_ids: Vec<&Id> = doc.retired.iter().filter_map(|c| c.id.as_ref()).collect();
            doc.retired.iter().filter(move |c| {
                if c.note.is_some() {
                    return false;
                }
                match c.id.as_ref().and_then(|id| id.parent()) {
                    // Retired alongside its parent: not its own decision.
                    Some(parent) => !retired_ids.iter().any(|other| **other == parent),
                    None => true,
                }
            })
        })
        .count();
    if unexplained > 0 {
        let subject = if unexplained == 1 {
            "1 retired criterion does".to_string()
        } else {
            format!("{unexplained} retired criteria do")
        };
        notes.push(format!(
            "{subject} not say why. Add it with `hi retire <ID> \"...\"`"
        ));
    }

    Report {
        note: (!notes.is_empty()).then(|| notes.join("\n      ")),
        files: workspace.docs.len(),
        criteria: workspace.criteria_count(),
        retired: workspace.docs.iter().map(|d| d.retired.len()).sum(),
        families: workspace.families(),
        problems,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::Doc;
    use std::path::PathBuf;

    fn workspace(files: &[(&str, &str)]) -> Workspace {
        let docs = files
            .iter()
            .map(|(name, raw)| Doc::parse(PathBuf::from(format!("/r/hi/{name}")), raw))
            .collect();
        Workspace {
            root: PathBuf::from("/r"),
            dir: PathBuf::from("/r/hi"),
            docs,
            skipped: Vec::new(),
        }
    }

    fn head(families: &str) -> String {
        format!("---\nhi: 1\nfamilies: [{families}]\n---\n\n## Criteria\n\n")
    }

    #[test]
    fn a_clean_workspace_has_no_problems() {
        let raw = format!("{}SEND-1  One.\nSEND-1.a  A case.\n", head("SEND"));
        let report = run(&workspace(&[("chat.md", &raw)]));
        assert!(report.ok(), "{:?}", report.problems);
        assert_eq!(report.criteria, 2);
        assert_eq!(report.families, vec!["SEND"]);
    }

    #[test]
    fn catches_duplicate_ids_across_files() {
        let a = format!("{}SEND-1  One.\n", head("SEND"));
        let b = format!("{}SEND-1  Again.\n", head("SEND"));
        let report = run(&workspace(&[("a.md", &a), ("b.md", &b)]));
        assert_eq!(report.problems.len(), 1);
        assert_eq!(report.problems[0].kind, Kind::DuplicateId);
    }

    #[test]
    fn catches_a_case_with_no_parent() {
        let raw = format!("{}SEND-1.a  An orphan.\n", head("SEND"));
        let report = run(&workspace(&[("chat.md", &raw)]));
        assert_eq!(report.problems.len(), 1);
        assert_eq!(report.problems[0].kind, Kind::OrphanCase);
        assert!(report.problems[0].message.contains("SEND-1"));
    }

    #[test]
    fn catches_reuse_of_a_retired_id() {
        let raw = format!(
            "{}SEND-3  Back again.\n\n## Retired\n\nSEND-3  Gone.\n        retired: no\n",
            head("SEND")
        );
        let report = run(&workspace(&[("chat.md", &raw)]));
        let kinds: Vec<Kind> = report.problems.iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&Kind::RetiredCollision));
        assert!(kinds.contains(&Kind::DuplicateId));
    }

    #[test]
    fn catches_a_malformed_id() {
        let raw = format!("{}SEND-1.a.b  Two letters.\n", head("SEND"));
        let report = run(&workspace(&[("chat.md", &raw)]));
        assert_eq!(report.problems.len(), 1);
        assert_eq!(report.problems[0].kind, Kind::UnparseableId);
    }

    #[test]
    fn catches_an_undeclared_family() {
        let raw = format!("{}OFFLINE-1  Undeclared.\n", head("SEND"));
        let report = run(&workspace(&[("chat.md", &raw)]));
        assert!(
            report
                .problems
                .iter()
                .any(|p| p.kind == Kind::UndeclaredFamily)
        );
    }

    #[test]
    fn catches_a_criterion_stranded_outside_every_section() {
        // A `# ` heading closes the section, so anything below it is invisible.
        let raw = format!(
            "{}SEND-1  Seen.\n\n# Appendix\n\nSEND-9  Invisible.\n",
            head("SEND")
        );
        let report = run(&workspace(&[("chat.md", &raw)]));
        assert_eq!(report.problems.len(), 1);
        assert_eq!(report.problems[0].kind, Kind::StrayCriterion);
        assert_eq!(report.problems[0].id, "SEND-9");
    }

    #[test]
    fn prose_outside_a_section_is_not_a_stray_criterion() {
        let raw = format!(
            "{}SEND-1  One.\n\n# Notes\n\nJust prose about the feature.\n",
            head("SEND")
        );
        assert!(run(&workspace(&[("chat.md", &raw)])).ok());
    }

    #[test]
    fn unfinished_intent_is_never_a_problem() {
        // No specs, no tests, no evidence, nothing downstream. Still fine.
        let raw = format!("{}SEND-1  It should feel fast.\n", head("SEND"));
        assert!(run(&workspace(&[("chat.md", &raw)])).ok());
    }

    #[test]
    fn a_retired_criterion_may_keep_its_case_parent() {
        let raw = format!(
            "{}SEND-1  Live.\n\n## Retired\n\nSEND-1.a  Retired case.\n",
            head("SEND")
        );
        assert!(run(&workspace(&[("chat.md", &raw)])).ok());
    }
}
