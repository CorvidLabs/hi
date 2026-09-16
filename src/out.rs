//! The verbs that read: `ls`, `issue`, `export`, `index`.

use std::fs;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::doc::{Criterion, Doc, Section};
use crate::id::Id;
use crate::workspace::Workspace;

const INDEX_OPEN: &str = "<!-- hi:index -->";
const INDEX_CLOSE: &str = "<!-- /hi:index -->";

// --- ls ----------------------------------------------------------------

/// Print every criterion, grouped by file, with cases indented by depth.
pub fn ls(workspace: &Workspace, family: Option<&str>, include_retired: bool) {
    let mut shown = 0;

    for doc in &workspace.docs {
        let matching: Vec<&Criterion> = doc
            .criteria
            .iter()
            .filter(|c| family.is_none_or(|f| c.id.as_ref().is_some_and(|id| id.family == f)))
            .collect();

        let retired: Vec<&Criterion> = if include_retired {
            doc.retired
                .iter()
                .filter(|c| family.is_none_or(|f| c.id.as_ref().is_some_and(|id| id.family == f)))
                .collect()
        } else {
            Vec::new()
        };

        if matching.is_empty() && retired.is_empty() {
            continue;
        }

        println!("{}", workspace.rel(&doc.path));
        for criterion in &matching {
            print_criterion(criterion);
            shown += 1;
        }
        for criterion in &retired {
            let indent = "  ".repeat(criterion.id.as_ref().map(|i| i.depth()).unwrap_or(1));
            println!(
                "{indent}{}  {}  (retired)",
                criterion.raw_id, criterion.text
            );
            shown += 1;
        }
        println!();
    }

    if shown == 0 {
        println!("no criteria yet. Capture one with `hi SEND-1 \"...\"`");
    }
}

fn print_criterion(criterion: &Criterion) {
    let depth = criterion.id.as_ref().map(|id| id.depth()).unwrap_or(1);
    let indent = "  ".repeat(depth);
    println!("{indent}{}  {}", criterion.raw_id, criterion.text);
}

// --- issue -------------------------------------------------------------

/// Render a criterion and its cases as a ticket body.
pub fn issue_markdown(doc: &Doc, criterion: &Criterion) -> (String, String) {
    let title = criterion.text.trim_end_matches('.').to_string();

    let mut body = String::new();
    // The heading already carries the sentence; repeating it reads as a bug.
    body.push_str(&format!("hi: {}\n", criterion.raw_id));

    if let Some(id) = &criterion.id {
        let cases: Vec<&Criterion> = doc
            .criteria
            .iter()
            .filter(|c| {
                c.id.as_ref()
                    .is_some_and(|other| other.is_descendant_of(id))
            })
            .collect();

        if !cases.is_empty() {
            body.push_str("\nCases:\n");
            for case in cases {
                let depth = case.id.as_ref().map(|i| i.depth()).unwrap_or(1);
                let indent = "  ".repeat(depth.saturating_sub(id.depth()).saturating_sub(1));
                // The indent goes before the bullet. After it, markdown reads
                // the spaces as content and renders a flat list, or a code
                // block once there are four of them (hi: ISSUE-3.a).
                body.push_str(&format!("{indent}- **{}**  {}\n", case.raw_id, case.text));
            }
        }
    }

    if !doc.intent.trim().is_empty() {
        body.push_str(&format!(
            "\n---\n\nIntent for {}:\n\n{}\n",
            doc.name(),
            doc.intent
        ));
    }

    (title, body)
}

/// Print a ticket, or hand it to `gh` when asked.
pub fn issue(workspace: &Workspace, raw_id: &str, create: bool, repo: Option<&str>) -> Result<()> {
    let id = Id::parse(raw_id).map_err(|e| anyhow::anyhow!("'{raw_id}' is not a valid id: {e}"))?;
    let Some((index, criterion)) = workspace.find_id(&id) else {
        bail!("{id} does not exist");
    };
    if criterion.section == Section::Retired {
        bail!("{id} is retired, so it should not become work");
    }

    let (title, body) = issue_markdown(&workspace.docs[index], criterion);

    if !create {
        println!("## {title}\n");
        println!("{body}");
        return Ok(());
    }

    let mut command = Command::new("gh");
    command.args(["issue", "create", "--title", &title, "--body", &body]);
    if let Some(repo) = repo {
        command.args(["--repo", repo]);
    }

    let status = command
        .status()
        .context("running `gh` (is the GitHub CLI installed and authenticated?)")?;
    if !status.success() {
        bail!("gh issue create failed");
    }
    Ok(())
}

// --- export ------------------------------------------------------------

#[derive(Serialize)]
struct ExportCriterion {
    id: String,
    text: String,
    depth: usize,
    parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retired: Option<String>,
}

#[derive(Serialize)]
struct ExportFile {
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    intent: String,
    families: Vec<String>,
    criteria: Vec<ExportCriterion>,
    retired: Vec<ExportCriterion>,
}

#[derive(Serialize)]
struct Export {
    hi: u32,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    product: Option<String>,
    files: Vec<ExportFile>,
}

fn export_criterion(criterion: &Criterion) -> ExportCriterion {
    ExportCriterion {
        id: criterion.raw_id.clone(),
        text: criterion.text.clone(),
        depth: criterion.id.as_ref().map(|id| id.depth()).unwrap_or(1),
        parent: criterion
            .id
            .as_ref()
            .and_then(|id| id.parent())
            .map(|p| p.to_string()),
        retired: criterion.note.clone(),
    }
}

/// True when `scope` names this doc: its stem, its file name, or the
/// repository-relative path that every other verb prints.
fn matches_file(doc: &Doc, scope: &str) -> bool {
    let stem = doc.name();
    let file = format!("{stem}.md");
    scope == stem
        || scope == file
        || scope
            .trim_start_matches("./")
            .ends_with(&format!("hi/{file}"))
}

/// Build the agent payload. `scope` is a family, a file, or nothing.
pub fn export(workspace: &Workspace, scope: Option<&str>) -> Result<String> {
    let families = workspace.families();
    let is_family = scope.is_some_and(|s| families.iter().any(|f| f == s));

    let mut files = Vec::new();
    for doc in &workspace.docs {
        let is_this_file = scope.is_some_and(|s| matches_file(doc, s));
        if let Some(scope) = scope
            && !is_this_file
            && !(is_family
                && doc
                    .all()
                    .any(|c| c.id.as_ref().is_some_and(|id| id.family == scope)))
        {
            continue;
        }

        let keep = |criterion: &&Criterion| -> bool {
            match scope {
                Some(scope) if is_family && !is_this_file => {
                    criterion.id.as_ref().is_some_and(|id| id.family == scope)
                }
                _ => true,
            }
        };

        files.push(ExportFile {
            file: workspace.rel(&doc.path),
            title: doc.title.clone(),
            intent: doc.intent.clone(),
            families: doc.front.families.clone(),
            criteria: doc
                .criteria
                .iter()
                .filter(keep)
                .map(export_criterion)
                .collect(),
            retired: doc
                .retired
                .iter()
                .filter(keep)
                .map(export_criterion)
                .collect(),
        });
    }

    if let Some(scope) = scope
        && files.is_empty()
    {
        bail!(
            "nothing matches '{scope}'. Give a family like SEND, a file like chat, or \
             nothing at all for the whole repository"
        );
    }

    // The product-level why only belongs in a whole-repo export.
    let product = if scope.is_none() {
        read_product_intent(workspace)
    } else {
        None
    };

    let export = Export {
        hi: 1,
        scope: scope.unwrap_or("repo").to_string(),
        product,
        files,
    };
    Ok(serde_json::to_string_pretty(&export)?)
}

/// The prose from INTENT.md, with the generated index stripped out.
fn read_product_intent(workspace: &Workspace) -> Option<String> {
    let raw = fs::read_to_string(workspace.intent_path()).ok()?;
    let mut out = Vec::new();
    let mut skipping = false;
    for line in raw.lines() {
        if line.trim() == INDEX_OPEN {
            skipping = true;
            continue;
        }
        if line.trim() == INDEX_CLOSE {
            skipping = false;
            continue;
        }
        if !skipping {
            out.push(line);
        }
    }
    let text = out.join("\n").trim().to_string();
    (!text.is_empty()).then_some(text)
}

// --- index -------------------------------------------------------------

/// The opening of a product-level INTENT.md.
///
/// This is the holistic why, above any one feature. It exists from the first
/// capture rather than waiting for someone to discover `hi index`, because a
/// file nobody knows about is the silence worth fearing.
pub fn starter_intent(workspace: &Workspace) -> String {
    let title = workspace
        .root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Product".to_string());
    format!(
        "# {title}\n\n\
         <!-- What is this product for, holistically, and who is it for?\n\
              Write it as a person. This is the part a spec can never carry. -->\n\n"
    )
}

/// Byte range of the generated block, matched on whole lines only.
///
/// A substring search would match a marker quoted in prose, and rewriting from
/// there destroys everything between it and the real close. INTENT.md's prose
/// belongs to the person who wrote it (hi: INDEX-2).
fn index_span(existing: &str) -> Option<std::ops::Range<usize>> {
    let mut open: Option<std::ops::Range<usize>> = None;
    let mut offset = 0usize;
    // A fence makes its contents an example, not structure. Without this, a
    // person who documents the format inside their own INTENT.md gets the
    // generated list written into their example (hi: INDEX-2.a).
    let mut fence: Option<(char, usize)> = None;

    for line in existing.split_inclusive('\n') {
        let trimmed = line.trim();
        let span = offset..offset + line.len();

        if let Some(marker) = trimmed.chars().next().filter(|c| *c == '`' || *c == '~') {
            let run = trimmed.chars().take_while(|c| *c == marker).count();
            if run >= 3 {
                match fence {
                    None => fence = Some((marker, run)),
                    Some((opened, width)) if opened == marker && run >= width => fence = None,
                    Some(_) => {}
                }
                offset += line.len();
                continue;
            }
        }
        if fence.is_some() {
            offset += line.len();
            continue;
        }

        if trimmed == INDEX_OPEN {
            if open.is_none() {
                open = Some(span);
            }
        } else if trimmed == INDEX_CLOSE
            && let Some(start) = &open
        {
            // Stop at the end of the marker text, leaving the line's own
            // newline in place so the blank line after it survives.
            return Some(start.start..span.start + line.trim_end().len());
        }
        offset += line.len();
    }
    None
}

/// True when a marker appears alone on a line, rather than quoted in prose.
fn has_marker_line(existing: &str, marker: &str) -> bool {
    let mut fence: Option<(char, usize)> = None;
    for line in existing.lines() {
        let trimmed = line.trim();
        if let Some(ch) = trimmed.chars().next().filter(|c| *c == '`' || *c == '~') {
            let run = trimmed.chars().take_while(|c| *c == ch).count();
            if run >= 3 {
                match fence {
                    None => fence = Some((ch, run)),
                    Some((opened, width)) if opened == ch && run >= width => fence = None,
                    Some(_) => {}
                }
                continue;
            }
        }
        if fence.is_none() && trimmed == marker {
            return true;
        }
    }
    false
}

/// The generated feature list that sits inside INTENT.md.
pub fn index_block(workspace: &Workspace) -> String {
    let mut out = String::new();
    for doc in &workspace.docs {
        let families = if doc.front.families.is_empty() {
            doc.used_families()
        } else {
            doc.front.families.clone()
        };
        let count = doc.criteria.len();
        let unit = if count == 1 { "criterion" } else { "criteria" };
        out.push_str(&format!(
            "- [{}]({}): {} ({count} {unit})\n",
            doc.name(),
            format_args!("hi/{}.md", doc.name()),
            if families.is_empty() {
                "no families yet".to_string()
            } else {
                families.join(", ")
            },
        ));
    }
    if out.is_empty() {
        out.push_str("- nothing captured yet\n");
    }
    out
}

/// Rewrite the index in INTENT.md between its markers, creating the file or the
/// section when they are not there yet. Prose outside the markers is untouched.
pub fn write_index(workspace: &Workspace) -> Result<String> {
    let path = workspace.intent_path();
    let block = index_block(workspace);
    let generated = format!("{INDEX_OPEN}\n{block}{INDEX_CLOSE}");

    let existing = fs::read_to_string(&path).unwrap_or_default();

    let updated = match index_span(&existing) {
        Some(span) => {
            let mut next = String::with_capacity(existing.len());
            next.push_str(&existing[..span.start]);
            next.push_str(&generated);
            next.push_str(&existing[span.end..]);
            next
        }
        None if has_marker_line(&existing, INDEX_OPEN) => {
            bail!(
                "{} has an opening {INDEX_OPEN} with no matching {INDEX_CLOSE}. \
                 Fix the markers rather than have hi guess where the block ends",
                workspace.rel(&path)
            )
        }
        None if existing.trim().is_empty() => {
            format!(
                "{}\n## Features\n\n{generated}\n",
                starter_intent(workspace)
            )
        }
        None => format!("{}\n\n## Features\n\n{generated}\n", existing.trim_end()),
    };

    // Atomic, for the same reason Doc::save is: a failed write must not destroy
    // the prose INDEX-2 promises stays yours (hi: FILE-8).
    crate::doc::write_atomically(&path, &updated)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(workspace.rel(&path))
}

#[cfg(test)]
mod tests {
    use super::*;
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
        }
    }

    const CHAT: &str = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nIt should feel like texting.\n\n## Criteria\n\nSEND-1  I hit enter and it shows up.\nSEND-1.a  If I have no connection it queues.\nSEND-2  It reaches them.\n";

    #[test]
    fn issue_body_carries_the_id_and_its_cases() {
        let workspace = workspace(&[("chat.md", CHAT)]);
        let id = Id::parse("SEND-1").unwrap();
        let (index, criterion) = workspace.find_id(&id).unwrap();
        let (title, body) = issue_markdown(&workspace.docs[index], criterion);
        assert_eq!(title, "I hit enter and it shows up");
        assert!(body.contains("hi: SEND-1"));
        assert!(body.contains("SEND-1.a"));
        // A sibling is not a case.
        assert!(!body.contains("SEND-2"));
        assert!(body.contains("It should feel like texting."));
    }

    #[test]
    fn export_of_the_whole_repo_includes_every_file() {
        let workspace = workspace(&[("chat.md", CHAT)]);
        let json = export(&workspace, None).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["scope"], "repo");
        assert_eq!(value["files"][0]["criteria"].as_array().unwrap().len(), 3);
        assert_eq!(value["files"][0]["criteria"][1]["parent"], "SEND-1");
        assert_eq!(value["files"][0]["intent"], "It should feel like texting.");
    }

    #[test]
    fn export_scoped_to_a_family_keeps_only_that_family() {
        let other = "---\nhi: 1\nfamilies: [BILLING]\n---\n\n## Criteria\n\nBILLING-1  I can see what I paid.\n";
        let workspace = workspace(&[("chat.md", CHAT), ("billing.md", other)]);
        let json = export(&workspace, Some("SEND")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["files"].as_array().unwrap().len(), 1);
        assert_eq!(value["files"][0]["file"], "hi/chat.md");
    }

    #[test]
    fn export_scoped_to_a_file_keeps_that_file() {
        let workspace = workspace(&[("chat.md", CHAT)]);
        let json = export(&workspace, Some("chat")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["scope"], "chat");
        assert_eq!(value["files"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn export_rejects_an_unknown_scope() {
        let workspace = workspace(&[("chat.md", CHAT)]);
        assert!(export(&workspace, Some("NOPE")).is_err());
    }

    #[test]
    fn a_marker_quoted_in_prose_is_not_the_generated_block() {
        let prose = "# Product\n\nNote: hi rewrites everything between <!-- hi:index --> and the close.\n\n## Features\n\n<!-- hi:index -->\nstale\n<!-- /hi:index -->\n";
        let span = index_span(prose).expect("the real block");
        assert!(
            prose[..span.start].contains("rewrites everything between"),
            "the sentence must survive"
        );
        assert!(prose[span.clone()].contains("stale"));
        // The close marker's own newline stays outside the span, so whatever
        // follows the block keeps its blank line.
        assert!(prose[span.end..].starts_with('\n'));
    }

    #[test]
    fn a_marker_inside_a_fence_is_an_example_not_the_block() {
        let prose = "# Product\n\nDocumenting the format:\n\n```markdown\n<!-- hi:index -->\n- [chat](hi/chat.md)\n<!-- /hi:index -->\n```\n\n## Features\n\n<!-- hi:index -->\nstale\n<!-- /hi:index -->\n";
        let span = index_span(prose).expect("the real block, not the example");
        assert!(
            prose[span.clone()].contains("stale"),
            "the span must be the real block: {:?}",
            &prose[span.clone()]
        );
        assert!(
            prose[..span.start].contains("```markdown"),
            "the fenced example must be left before the span"
        );
    }

    #[test]
    fn an_unclosed_marker_inside_a_fence_does_not_trigger_a_refusal() {
        assert!(!has_marker_line(
            "# P\n\n```\n<!-- hi:index -->\n```\n",
            INDEX_OPEN
        ));
    }

    #[test]
    fn an_unclosed_marker_has_no_span() {
        assert!(index_span("# P\n\n<!-- hi:index -->\n- a\n").is_none());
        assert!(has_marker_line("# P\n\n<!-- hi:index -->\n", INDEX_OPEN));
    }

    #[test]
    fn index_lists_each_file_with_its_families_and_count() {
        let workspace = workspace(&[("chat.md", CHAT)]);
        let block = index_block(&workspace);
        assert!(block.contains("[chat](hi/chat.md)"));
        assert!(block.contains("SEND"));
        assert!(block.contains("(3 criteria)"));
    }
}
