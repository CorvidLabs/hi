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

/// How a line behaves when the line under it is only a wrap.
enum Block {
    /// A list item or a block quote: a wrapped line under it belongs to it.
    Continued,
    /// A heading, a rule, a table row or raw HTML: its newline is structural,
    /// and nothing may be folded onto it.
    Closed,
}

/// Join the lines somebody only wrapped, and leave every newline that means
/// something exactly where it is.
///
/// GitHub renders an issue body with hard line breaks on, so a newline is a
/// `<br>` there even though the same file on a repository page is not. Prose
/// wrapped at a person's own margin therefore arrives as a narrow column down
/// the left of a wide pane, broken after every authored line. A single newline
/// inside a paragraph is where an editor wrapped; a blank line is the break
/// somebody asked for (hi: ISSUE-7, ISSUE-7.a).
///
/// This is the rendering, never the file. hi does not reflow prose a person
/// wrote, here or anywhere else (hi: FILE-4).
fn unwrap_soft_breaks(prose: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    // A fence makes its contents an example, so every newline inside one is
    // the author's. The same reading `doc` takes under `## Intent`, through
    // the same helper (hi: FILE-9, ISSUE-7.b).
    let mut fence: Option<(char, usize)> = None;
    // True while the last line emitted is one a wrapped line may continue.
    let mut open = false;

    for line in prose.lines() {
        let trimmed = line.trim();

        if let Some(marker) = crate::doc::fence_marker(trimmed) {
            match fence {
                Some((opened, width)) if opened == marker.0 && marker.1 >= width => fence = None,
                Some(_) => {}
                None => fence = Some(marker),
            }
            out.push(line.to_string());
            open = false;
            continue;
        }
        if fence.is_some() {
            out.push(line.to_string());
            continue;
        }

        if trimmed.is_empty() {
            out.push(String::new());
            open = false;
            continue;
        }

        match (open, block_start(trimmed)) {
            // A wrap, not a break: the line keeps going.
            (true, None) => {
                if let Some(last) = out.last_mut() {
                    last.truncate(last.trim_end().len());
                    last.push(' ');
                    last.push_str(line.trim_start());
                }
            }
            (_, block) => {
                out.push(line.to_string());
                open = !matches!(block, Some(Block::Closed));
            }
        }

        // Two trailing spaces, or a trailing backslash, is a break somebody
        // typed rather than a margin they hit. Nothing folds onto it.
        if line.ends_with("  ") || line.ends_with('\\') {
            open = false;
        }
    }

    out.join("\n")
}

/// What kind of block a line begins, or `None` when it is ordinary prose.
///
/// Indentation is not read here. A line only reaches this function while a
/// paragraph or a list item is already open, where four spaces is a lazy
/// continuation rather than a code block; a line that opens its own paragraph
/// is emitted as it was written, indentation and all.
fn block_start(trimmed: &str) -> Option<Block> {
    // A thematic break is tested first, because `- - -` and `* * *` are one
    // and would otherwise read as a list item.
    if is_thematic_break(trimmed)
        || is_heading(trimmed)
        || trimmed.starts_with('|')
        || trimmed.starts_with('<')
    {
        return Some(Block::Closed);
    }
    if is_list_item(trimmed) || trimmed.starts_with('>') {
        return Some(Block::Continued);
    }
    None
}

/// One to six `#` followed by a space or nothing: an ATX heading.
fn is_heading(trimmed: &str) -> bool {
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&hashes)
        && trimmed[hashes..]
            .chars()
            .next()
            .is_none_or(|c| c == ' ' || c == '\t')
}

/// Three or more `-`, `*` or `_`, with nothing else but spaces.
fn is_thematic_break(trimmed: &str) -> bool {
    ['-', '*', '_'].into_iter().any(|marker| {
        trimmed.chars().filter(|c| *c == marker).count() >= 3
            && trimmed
                .chars()
                .all(|c| c == marker || c == ' ' || c == '\t')
    })
}

/// A bullet or an ordered marker, followed by a space or nothing.
fn is_list_item(trimmed: &str) -> bool {
    if matches!(trimmed, "-" | "*" | "+")
        || trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
    {
        return true;
    }
    let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || digits > 9 {
        return false;
    }
    let rest = &trimmed[digits..];
    rest.starts_with(". ") || rest.starts_with(") ") || rest == "." || rest == ")"
}

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

    // The starter prompt hi writes into a new file is not intent, and a ticket
    // that quotes it reads as though nobody has written the why yet because
    // somebody pasted the question in (hi: ISSUE-6).
    let intent = crate::view::strip_comments(&doc.intent);
    if !intent.trim().is_empty() {
        body.push_str(&format!(
            "\n---\n\nIntent for {}:\n\n{}\n",
            doc.name(),
            // An issue body is rendered with hard line breaks on, so the
            // author's own wrapping would arrive as a break after every line
            // (hi: ISSUE-7).
            unwrap_soft_breaks(intent.trim())
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
            // The page and the ticket both refuse to pass hi's starter prompt
            // off as prose; the payload an agent reads must agree with them
            // (hi: EXPORT-5).
            intent: crate::view::strip_comments(&doc.intent).trim().to_string(),
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
    // The same reading the page takes: drop the generated index, the title and
    // the generated `## Features` heading, then drop hi's own starter prompt.
    // An agent asking what this product is for should get what a person wrote
    // or nothing, never the question hi left behind (hi: EXPORT-5).
    let text = crate::view::strip_comments(&crate::view::strip_index(raw))
        .trim()
        .to_string();
    (!text.is_empty()).then_some(text)
}

// --- index -------------------------------------------------------------

/// The opening of a product-level INTENT.md.
///
/// This is the holistic why, above any one feature. It exists from the first
/// capture rather than waiting for someone to discover `hi index`, because a
/// file nobody knows about is the silence worth fearing.
///
/// One line per paragraph, like everything hi writes. The first file somebody
/// sees is the one they write the rest of their prose to match (hi: FILE-21.a).
pub fn starter_intent(workspace: &Workspace) -> String {
    let title = workspace
        .root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Product".to_string());
    format!(
        "# {title}\n\n\
         <!-- What is this product for, holistically, and who is it for? Write it as a person. \
         This is the part a spec can never carry. -->\n\n"
    )
}

/// What hi writes into `hi/AGENTS.md` on the first capture.
///
/// The habit, and one sentence about how prose is written here. It carries no
/// id grammar, no file format and no list of the families already here: this
/// file is written once and never rewritten, so anything in it that hi could
/// change underneath it would be wrong later with nothing to notice
/// (DECISIONS.md §27).
///
/// The wrapping sentence is the one thing §27's "say less" rule was narrowed
/// for, and it is narrowed by exactly one sentence. It is not the format and
/// cannot go stale with it: it is how markdown reads a newline, which is the
/// same whatever hi's format version is, and the agent that writes the prose
/// is the only one who will ever see this file (DECISIONS.md §29, hi: FILE-21).
///
/// The text is itself one line per paragraph, because the file somebody reads
/// first is the one they write the rest of their prose to match (FILE-21.a).
pub fn agent_instructions() -> String {
    "# Human intent\n\n\
     This repository writes down what people want before building it. Every sentence in this \
     directory is something somebody wants, and each one has an id that never moves and is never \
     reused.\n\n\
     Before you build a feature:\n\n\
     1. Read the files here, so you know what has already been said.\n\
     2. Draft the criteria for what you are about to build, as plain sentences about what somebody \
     wants rather than what the code will do.\n\
     3. Ask the person to confirm them. Nothing lands that they did not agree to.\n\
     4. Capture what they agreed to, then build it.\n\n\
     That happens before every feature, not only the first one.\n\n\
     Write the prose in these files as one line per paragraph, with a blank line between \
     paragraphs. A newline inside a paragraph is a visible break wherever the file is rendered, \
     and it was only ever where your editor wrapped.\n\n\
     Run `hi --help` for the commands.\n"
        .to_string()
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
            skipped: Vec::new(),
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
    fn hi_own_starter_prompt_is_never_passed_off_as_prose() {
        // A day-one repository has hi's question in every `## Intent` block and
        // a generated `## Features` heading in INTENT.md. The page already
        // refuses to print either; the export and the ticket have to agree,
        // because those are what an agent and a tracker read (hi: EXPORT-5,
        // ISSUE-6).
        let placeholder = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Send\n\n\
             ## Intent\n\n<!-- What is this for, and what should it feel like? -->\n\n\
             ## Criteria\n\n- **SEND-1**  I hit enter and it shows up.\n";
        let doc = Doc::parse(PathBuf::from("/r/hi/send.md"), placeholder);

        let (_, body) = issue_markdown(&doc, doc.criteria.first().unwrap());
        assert!(
            !body.contains("What is this for"),
            "a ticket must not quote hi's own starter prompt: {body}"
        );
        assert!(
            !body.contains("Intent for send"),
            "with no prose written there is no intent section to print: {body}"
        );
    }

    /// The prose here is wrapped the way a person wraps prose: at their own
    /// margin, mid-sentence, with a blank line only where they meant a break.
    const WRAPPED: &str = "---\nhi: 1\nfamilies: [HOST]\n---\n\n# Host\n\n## Intent\n\n\
         Most people who would want this do not want a VPS, and should not have to\n\
         learn one to give their community a role that matches what they hold.\n\n\
         It has to work with nothing set up first.\n\n\
         ## Criteria\n\n- **HOST-1**  I can run this without a server.\n";

    #[test]
    fn a_ticket_unwraps_prose_the_author_only_wrapped() {
        // A GitHub issue body is rendered with hard line breaks on, so every
        // place somebody wrapped their own prose becomes a `<br>` and the
        // ticket reads as a narrow column down a wide pane (hi: ISSUE-7).
        let doc = Doc::parse(PathBuf::from("/r/hi/host.md"), WRAPPED);
        let (_, body) = issue_markdown(&doc, doc.criteria.first().unwrap());

        assert!(
            body.contains(
                "Most people who would want this do not want a VPS, and should not have to learn \
                 one to give their community a role that matches what they hold."
            ),
            "a paragraph the author wrapped has to arrive as one line: {body}"
        );
        assert!(
            !body.contains("should not have to\n"),
            "no authored wrap may survive as a newline: {body}"
        );
        assert!(
            body.contains("what they hold.\n\nIt has to work with nothing set up first."),
            "a blank line is a break somebody asked for and stays one (hi: ISSUE-7.a): {body}"
        );
    }

    #[test]
    fn only_a_wrapped_line_is_joined() {
        // A newline is structural inside a list, a fence, a table, a heading
        // and a rule. Folding one of those is a worse bug than the one being
        // fixed, because it changes what the markdown means (hi: ISSUE-7.b).
        for (name, raw, expected) in [
            (
                "a paragraph",
                "one line\nand its wrap",
                "one line and its wrap",
            ),
            (
                "a blank line",
                "one thought\nwrapped\n\nanother",
                "one thought wrapped\n\nanother",
            ),
            (
                "a list",
                "the shape:\n- one thing\n- another, itself\n  wrapped",
                "the shape:\n- one thing\n- another, itself wrapped",
            ),
            (
                "an ordered list",
                "1. first\n2. second\n   wrapped",
                "1. first\n2. second wrapped",
            ),
            (
                "a fence",
                "look:\n\n```\nHOST-1  an example\nHOST-2  another\n```",
                "look:\n\n```\nHOST-1  an example\nHOST-2  another\n```",
            ),
            ("a tilde fence", "~~~\na\nb\n~~~", "~~~\na\nb\n~~~"),
            (
                "a heading",
                "### Why\nthe reason, which\nwrapped",
                "### Why\nthe reason, which wrapped",
            ),
            ("a table", "| a | b |\n| - | - |", "| a | b |\n| - | - |"),
            ("a quote", "> quoted\n> more", "> quoted\n> more"),
            ("a rule", "above\n\n---\n\nbelow", "above\n\n---\n\nbelow"),
            (
                "a break somebody typed",
                "one thought.  \na deliberately separate one",
                "one thought.  \na deliberately separate one",
            ),
        ] {
            assert_eq!(unwrap_soft_breaks(raw), expected, "{name}");
        }
    }

    #[test]
    fn the_files_hi_writes_are_one_line_per_paragraph() {
        // hi models the convention it asks for. A hard-wrapped starter file is
        // the thing every adopter copies (hi: FILE-21.a).
        for (name, text) in [
            ("hi/AGENTS.md", agent_instructions()),
            ("INTENT.md", starter_intent(&workspace(&[]))),
        ] {
            let written = text.trim();
            assert_eq!(
                unwrap_soft_breaks(written),
                written,
                "{name} carries a soft wrap, so it models the defect it should model the fix for"
            );
        }
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
