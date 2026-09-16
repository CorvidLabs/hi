//! Capture: `hi SEND-2 "it reaches them and the mark changes to sent"`.
//!
//! The rule is that a new id just works and an existing id refuses. A new
//! family creates its own file rather than asking, because stopping to answer a
//! question is the thing that loses the thought.

use std::fs;

use anyhow::{Result, bail};

use crate::doc::{Doc, Section, new_file_text};
use crate::id::Id;
use crate::workspace::Workspace;

/// What capture did, so the caller can print it.
#[derive(Debug)]
pub struct Captured {
    pub id: Id,
    pub file: String,
    pub created_file: bool,
    /// Set when this capture also started the product-level INTENT.md.
    pub started_intent: Option<String>,
}

/// Add one criterion. Returns an error rather than writing when the id is taken
/// or when a case has no parent to hang off.
pub fn capture(workspace: &mut Workspace, raw_id: &str, sentence: &str) -> Result<Captured> {
    let id = match Id::parse(raw_id) {
        Ok(id) => id,
        Err(err) => bail!("'{raw_id}' is not a valid id: {err}"),
    };

    let sentence = sentence.trim();
    if sentence.is_empty() {
        bail!("a criterion needs a sentence. Say what you actually want");
    }

    // An id that already exists is the one case that refuses.
    if let Some((index, existing)) = workspace.find_id(&id) {
        let file = workspace.rel(&workspace.docs[index].path);
        let next = Id {
            family: id.family.clone(),
            levels: vec![crate::id::Level::Number(workspace.next_free(&id.family))],
        };
        bail!(
            "{id} already exists in {file}:{}\nhint:  next free is {next}",
            existing.line_no()
        );
    }

    // A case has to hang off something, and that something has to be live.
    // Placing a case under a retired parent would nest it below whatever
    // criterion happens to sit last, so the file would show it as a case of
    // something it is not (hi: CAPTURE-4).
    if let Some(parent) = id.parent() {
        match workspace.find_id(&parent) {
            None => bail!("{id} needs a parent {parent}, which does not exist yet"),
            Some((_, found)) if found.section == Section::Retired => bail!(
                "{parent} is retired, so {id} has nothing live to hang off.\n\
                 hint:  bring {parent} back, or write this as its own criterion"
            ),
            Some(_) => {}
        }
    }

    // A case belongs beside its parent. Resolving only by declared family
    // would put it in a different file and strand it as an orphan there.
    let parent_file = id
        .parent()
        .and_then(|parent| workspace.find_id(&parent).map(|(index, _)| index));

    let (index, created_file) = match parent_file.or_else(|| workspace.doc_for_family(&id.family)) {
        Some(index) => (index, false),
        None => start_file(workspace, &id.family)?,
    };

    // Only now does anything touch the disk. Building the new file in memory
    // first means a failed write leaves no half-made file and loses no thought
    // (hi: CAPTURE-5).
    if !workspace.dir.exists() {
        fs::create_dir_all(&workspace.dir)?;
    }
    let doc = &mut workspace.docs[index];
    doc.insert(&id, sentence)?;
    doc.save()?;

    // Only after the criterion is safely on disk. A product with criteria but
    // no stated why is the common failure, so the file exists from the start
    // rather than waiting to be discovered.
    let started_intent = start_product_intent(workspace);

    let file = workspace.rel(&workspace.docs[index].path);
    Ok(Captured {
        id,
        file,
        created_file,
        started_intent,
    })
}

/// Prepare a file for a family hi has not seen before, in memory only.
///
/// Returns its index and whether it is genuinely new. Nothing is written here:
/// the caller saves once the criterion is known to be storable, so a refusal
/// never leaves a half-made file behind (hi: CAPTURE-5).
fn start_file(workspace: &mut Workspace, family: &str) -> Result<(usize, bool)> {
    let stem = family.to_ascii_lowercase().replace('_', "-");
    let path = workspace.dir.join(format!("{stem}.md"));

    if path.exists() {
        // The file is there but does not declare the family; adopt it. It was
        // not created by us, so say so rather than claiming we made it.
        workspace.docs.push(Doc::load(&path)?);
        return Ok((workspace.docs.len() - 1, false));
    }

    let doc = Doc::parse(path, &new_file_text(&title_for(family), family));
    workspace.docs.push(doc);
    Ok((workspace.docs.len() - 1, true))
}

/// Create INTENT.md if the repository has none, returning its path.
///
/// Best effort: a criterion that is already stored must not be reported as a
/// failure because this could not be written.
fn start_product_intent(workspace: &Workspace) -> Option<String> {
    let path = workspace.intent_path();
    if path.exists() {
        return None;
    }
    let body = crate::out::starter_intent(workspace);
    fs::write(&path, body).ok()?;
    Some(workspace.rel(&path))
}

/// `SEND` becomes `Send`, `TWO_FACTOR` becomes `Two factor`.
fn title_for(family: &str) -> String {
    let lower = family.to_ascii_lowercase().replace('_', " ");
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => lower,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hi-capture-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("hi")).unwrap();
        dir
    }

    fn seeded(name: &str) -> Workspace {
        let root = temp_dir(name);
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Criteria\n\nSEND-1  I hit enter.\n",
        )
        .unwrap();
        Workspace::load(&root).unwrap()
    }

    #[test]
    fn appends_to_an_existing_family() {
        let mut workspace = seeded("append");
        let done = capture(&mut workspace, "SEND-2", "It reaches them.").unwrap();
        assert_eq!(done.id.to_string(), "SEND-2");
        assert!(!done.created_file);
        let text = fs::read_to_string(workspace.root.join("hi/chat.md")).unwrap();
        assert!(text.contains("**SEND-2**  It reaches them."));
    }

    #[test]
    fn creates_a_file_for_a_brand_new_family() {
        let mut workspace = seeded("newfamily");
        let done = capture(&mut workspace, "BILLING-1", "I can see what I paid.").unwrap();
        assert!(done.created_file);
        assert_eq!(done.file, "hi/billing.md");
        let text = fs::read_to_string(workspace.root.join("hi/billing.md")).unwrap();
        assert!(text.contains("families: [BILLING]"));
        assert!(text.contains("# Billing"));
        assert!(text.contains("**BILLING-1**  I can see what I paid."));
    }

    #[test]
    fn refuses_an_id_that_already_exists() {
        let mut workspace = seeded("dupe");
        let err = capture(&mut workspace, "SEND-1", "Something else.").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("already exists"), "{message}");
        assert!(message.contains("next free is SEND-2"), "{message}");
    }

    #[test]
    fn a_refusal_leaves_no_half_made_file() {
        // The destination file is built in memory, so a failed write cannot
        // leave an empty scaffold behind and lose the thought (hi: CAPTURE-5).
        let root = temp_dir("orphanfile");
        fs::write(
            root.join("hi/w.md"),
            "---\nhi: 1\nfamilies: [W]\n---\n\n## Criteria\n\n- **W-1**  One.\n",
        )
        .unwrap();
        // A directory where the file wants to go makes the save fail.
        fs::create_dir_all(root.join("hi/billing.md")).unwrap();
        let mut workspace = Workspace::load(&root).unwrap();

        assert!(capture(&mut workspace, "BILLING-1", "I can see what I paid.").is_err());
        assert!(
            fs::read_dir(root.join("hi"))
                .unwrap()
                .flatten()
                .all(|e| e.file_name() != "billing.md" || e.path().is_dir()),
            "no scaffold file may be left behind"
        );
    }

    #[test]
    fn a_retired_criterion_cannot_take_new_cases() {
        // Otherwise the case nests under whatever sits last, and the file shows
        // it as a case of something it is not (hi: CAPTURE-4).
        let root = temp_dir("retiredparent");
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n- **SEND-2**  Two.\n\n## Retired\n\n- **SEND-3**  Gone.\n",
        )
        .unwrap();
        let before = fs::read_to_string(root.join("hi/chat.md")).unwrap();
        let mut workspace = Workspace::load(&root).unwrap();

        let err = capture(&mut workspace, "SEND-3.a", "A case of the retired one.").unwrap_err();
        assert!(err.to_string().contains("is retired"), "{err}");
        assert_eq!(fs::read_to_string(root.join("hi/chat.md")).unwrap(), before);
    }

    #[test]
    fn adopting_an_existing_file_does_not_claim_to_have_created_it() {
        let root = temp_dir("adopt");
        fs::write(
            root.join("hi/w.md"),
            "---\nhi: 1\nfamilies: [W]\n---\n\n## Criteria\n\n- **W-1**  One.\n",
        )
        .unwrap();
        fs::write(
            root.join("hi/billing.md"),
            "---\nhi: 1\nfamilies: []\n---\n\n# Billing\n\n## Intent\n\nHand-written prose.\n\n## Criteria\n\n",
        )
        .unwrap();
        let mut workspace = Workspace::load(&root).unwrap();

        let done = capture(&mut workspace, "BILLING-1", "I can see what I paid.").unwrap();
        assert!(!done.created_file, "the file was already there");
        let body = fs::read_to_string(root.join("hi/billing.md")).unwrap();
        assert!(body.contains("Hand-written prose."), "prose must survive");
        assert!(body.contains("**BILLING-1**"));
    }

    #[test]
    fn refuses_a_case_with_no_parent() {
        let mut workspace = seeded("orphan");
        let err = capture(&mut workspace, "SEND-4.a", "An orphan.").unwrap_err();
        assert!(err.to_string().contains("needs a parent SEND-4"));
    }

    #[test]
    fn refuses_a_malformed_id_without_writing() {
        let mut workspace = seeded("malformed");
        assert!(capture(&mut workspace, "SEND-1.a.b", "Two letters.").is_err());
        assert!(capture(&mut workspace, "send-1", "Lowercase family.").is_err());
        let text = fs::read_to_string(workspace.root.join("hi/chat.md")).unwrap();
        assert!(!text.contains("Two letters"));
        assert!(!text.contains("Lowercase family"));
    }

    #[test]
    fn refuses_an_empty_sentence() {
        let mut workspace = seeded("empty");
        assert!(capture(&mut workspace, "SEND-2", "   ").is_err());
    }

    #[test]
    fn a_case_lands_in_the_file_holding_its_parent() {
        // The family is declared in one file but actually used in another.
        let root = temp_dir("parentfile");
        fs::write(
            root.join("hi/decl.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n",
        )
        .unwrap();
        fs::write(
            root.join("hi/real.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  Lives here.\n",
        )
        .unwrap();
        let mut workspace = Workspace::load(&root).unwrap();

        let done = capture(&mut workspace, "SEND-1.a", "A case.").unwrap();
        assert_eq!(done.file, "hi/real.md", "a case follows its parent");
        assert!(
            fs::read_to_string(root.join("hi/real.md"))
                .unwrap()
                .contains("SEND-1.a")
        );
        assert!(
            !fs::read_to_string(root.join("hi/decl.md"))
                .unwrap()
                .contains("SEND-1.a")
        );
    }

    #[test]
    fn accepts_a_case_under_an_existing_parent() {
        let mut workspace = seeded("case");
        capture(
            &mut workspace,
            "SEND-1.a",
            "If I have no connection it queues.",
        )
        .unwrap();
        let text = fs::read_to_string(workspace.root.join("hi/chat.md")).unwrap();
        let parent = text.find("SEND-1 ").unwrap();
        let case = text.find("SEND-1.a").unwrap();
        assert!(parent < case);
    }
}
