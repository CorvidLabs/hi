//! Capture: `hi SEND-2 "it reaches them and the mark changes to sent"`.
//!
//! The rule is that a new id just works and an existing id refuses. A new
//! family creates its own file rather than asking, because stopping to answer a
//! question is the thing that loses the thought.

use std::fs;

use anyhow::{Result, bail};

use crate::doc::{Doc, Section, new_file_text};
use crate::id::Id;
use crate::workspace::{StrayPlace, Workspace};

/// Family names that would want a file hi already keeps in `hi/`.
///
/// Short on purpose. It is exactly the files `start_agent_files` writes, and it
/// grows only if that does (DECISIONS.md §27).
const RESERVED_FAMILIES: [&str; 2] = ["AGENTS", "CLAUDE"];

/// What capture did, so the caller can print it.
#[derive(Debug)]
pub struct Captured {
    pub id: Id,
    pub file: String,
    pub created_file: bool,
    /// Set when this capture also started the product-level INTENT.md.
    pub started_intent: Option<String>,
    /// Files started so an agent finds the habit without being told: usually
    /// `hi/AGENTS.md` and the `hi/CLAUDE.md` beside it.
    pub started_agent: Vec<String>,
    /// Why the generated index in `INTENT.md` could not be refreshed, when it
    /// could not. A line to print, never a failure: the criterion is already
    /// stored (hi: INDEX-4.a).
    pub index_error: Option<String>,
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

    // A family names its own file, lowercased, so a family called AGENTS wants
    // the file hi keeps its instructions in. That is the same path on a
    // case-insensitive filesystem and a confusing neighbour on a case-sensitive
    // one, so refuse on both rather than behave differently by platform
    // (DECISIONS.md §27).
    if let Some(reserved) = RESERVED_FAMILIES
        .iter()
        .find(|name| id.family.eq_ignore_ascii_case(name))
    {
        bail!(
            "{reserved} cannot be a family: hi keeps its own instructions in \
             hi/{reserved}.md, which is the file that family would want.\n\
             hint:  name it something else, and nothing else has to change"
        );
    }

    // An id that already exists is the one case that refuses.
    if let Some((index, existing)) = workspace.find_id(&id) {
        let file = workspace.rel(&workspace.docs[index].path);
        // At the very top of the range there is no next one, and the old
        // arithmetic wrapped and offered SEND-0. A refusal with no hint is
        // better than a refusal with a wrong hint (hi: CAPTURE-13).
        let free = workspace.next_free(&id.family);
        let hint = if free == u32::MAX
            && workspace
                .find_id(&Id {
                    family: id.family.clone(),
                    levels: vec![crate::id::Level::Number(free)],
                })
                .is_some()
        {
            String::new()
        } else {
            let next = Id {
                family: id.family.clone(),
                levels: vec![crate::id::Level::Number(free)],
            };
            format!("\nhint:  next free is {next}")
        };
        bail!("{id} already exists in {file}:{}{hint}", existing.line_no());
    }

    // An id hi cannot read is still an id somebody wrote. Reusing it would put
    // two lines with the same id and different sentences in one file, which no
    // amount of later checking undoes (hi: CAPTURE-14). The lookup covers the
    // files hi loads and the ones it skips alike: a retired id parked in
    // `hi/Archive.md` was reported by `check` and reissued here for four
    // releases (DECISIONS.md §32).
    if let Some(stray) = workspace.find_stray(&id)? {
        let (file, line) = (&stray.file, stray.line);
        let hint = match stray.place {
            StrayPlace::OutsideSection => {
                "move that line under ## Criteria or ## Retired, or delete it, then capture again"
            }
            // Renaming the file would work too, and is the wrong advice: it
            // makes hi's own AGENTS.md into a criteria file.
            StrayPlace::UnreadFile => {
                "move that line into a lowercase file under ## Criteria or ## Retired, \
                 or delete it, then capture again"
            }
        };
        bail!("{id} is already written at {file}:{line}, where hi cannot read it.\nhint:  {hint}");
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

    // Two files declaring the same family means a new top-level id has no
    // unique home. First-wins-by-path-order used to pick one, and renaming a
    // file moved later captures. A case still follows its parent
    // (hi: CAPTURE-4.a, CAPTURE-16, CHECK-2.g).
    if parent_file.is_none() {
        let owners = workspace.family_declarers(&id.family);
        if owners.len() > 1 {
            let files = owners
                .iter()
                .map(|&index| workspace.rel(&workspace.docs[index].path))
                .collect::<Vec<_>>()
                .join(" and ");
            bail!(
                "{} is declared in {files}, so a new criterion has nowhere that is uniquely its home.\n\
                 hint:  leave it in one file's families list, then capture again",
                id.family
            );
        }
    }

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

    // `Doc::insert` splices the line in and shifts the indexes around it; it
    // does not add the criterion to `doc.criteria`, so the Doc in memory still
    // describes the file as it was a moment ago. Anything that counts from here
    // would be one short, and the generated index below counts. Read back what
    // was just written rather than teach a second place how to bookkeep.
    if let Ok(saved) = Doc::load(&workspace.docs[index].path) {
        workspace.docs[index] = saved;
    }

    // Only after the criterion is safely on disk. A product with criteria but
    // no stated why is the common failure, so the file exists from the start
    // rather than waiting to be discovered.
    let started_intent = start_product_intent(workspace);
    let started_agent = start_agent_files(workspace);

    // The live count just changed, so the list at the front of the product is
    // no longer true. Nothing made anyone run `hi index`, and three adopter
    // repositories drifted; refreshing here makes that structurally impossible
    // rather than merely detectable (hi: INDEX-4, DECISIONS.md §30).
    //
    // Best effort, for the same reason INTENT.md's creation is: the criterion
    // is already on disk above, so no failure here may turn a capture that
    // stored it into a reported failure (hi: INDEX-4.a).
    let index_error = crate::out::refresh_index(workspace);

    let file = workspace.rel(&workspace.docs[index].path);
    Ok(Captured {
        id,
        file,
        created_file,
        started_intent,
        started_agent,
        index_error,
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
    // With the feature list already in it. The refresh below rewrites a block
    // it finds and installs none, so a file that is born without one never
    // gets one (hi: INDEX-1.a, INDEX-4.c, DECISIONS.md §32).
    let body = crate::out::starter_intent_file(workspace);
    fs::write(&path, body).ok()?;
    Some(workspace.rel(&path))
}

/// Write the agent-facing instruction into `hi/`, returning what was started.
///
/// Best effort, and for the same reason `INTENT.md` is: the criterion is
/// already stored, so nothing here may turn a successful capture into a
/// reported failure. hi writes inside `hi/` and nowhere else; a block in the
/// repository's own CLAUDE.md was refused (DECISIONS.md §27, §9).
fn start_agent_files(workspace: &Workspace) -> Vec<String> {
    let mut started = Vec::new();

    let agents = workspace.dir.join("AGENTS.md");
    if agents.symlink_metadata().is_err()
        && fs::write(&agents, crate::out::agent_instructions()).is_ok()
    {
        started.push(workspace.rel(&agents));
    }

    // Written only beside a real AGENTS.md, so the pointer never dangles.
    let claude = workspace.dir.join("CLAUDE.md");
    if claude.symlink_metadata().is_err() && agents.is_file() && link_to_agents(&claude).is_ok() {
        started.push(workspace.rel(&claude));
    }

    started
}

/// What `hi seed` did to `hi/AGENTS.md`.
#[derive(Debug)]
pub enum Seeded {
    /// The file was missing; hi wrote the current text (and the CLAUDE.md
    /// pointer beside it).
    Created(Vec<String>),
    /// The file was still a template hi had shipped; hi replaced it.
    Updated(String),
    /// The file already is the current text.
    Current,
}

/// Rewrite `hi/AGENTS.md` when it is still a template hi shipped, create it
/// when it is missing, and refuse when a person has edited it.
///
/// Write-once is the property that protects a file somebody touched. It is
/// not a reason to leave hi's own unmodified bytes frozen at 0.5.0 when 1.0
/// freezes the convention those bytes describe (hi: HABIT-6, DECISIONS.md §39).
/// Capture still only writes the file when it is absent, so the path that
/// runs on every thought never overwrites; this is the verb that migrates.
pub fn seed_agent_files(workspace: &Workspace) -> Result<Seeded> {
    if !workspace.dir.exists() {
        fs::create_dir_all(&workspace.dir)?;
    }
    let agents = workspace.dir.join("AGENTS.md");
    match fs::read_to_string(&agents) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::write(&agents, crate::out::agent_instructions())?;
            let mut started = vec![workspace.rel(&agents)];
            let claude = workspace.dir.join("CLAUDE.md");
            if claude.symlink_metadata().is_err() {
                link_to_agents(&claude)?;
                started.push(workspace.rel(&claude));
            }
            Ok(Seeded::Created(started))
        }
        Err(err) => bail!("reading {}: {err}", workspace.rel(&agents)),
        Ok(raw) => match crate::out::classify_agent_file(&raw) {
            crate::out::AgentTemplate::Current => Ok(Seeded::Current),
            crate::out::AgentTemplate::Prior => {
                crate::out::write_with_endings(&agents, &crate::out::agent_instructions(), &raw)?;
                Ok(Seeded::Updated(workspace.rel(&agents)))
            }
            crate::out::AgentTemplate::Other => bail!(
                "{} is not a template hi has shipped; it is yours.\n\
                 hint:  delete it and run `hi seed` if you want the current text, \
                 and nothing else has to change",
                workspace.rel(&agents)
            ),
        },
    }
}

/// Point `hi/CLAUDE.md` at `AGENTS.md`, by symlink where the platform allows.
///
/// One truth beats two copies that drift. Where a symlink cannot be made, a
/// one-line pointer says the same thing: Windows needs Developer Mode or admin
/// to create one, and a committed symlink checks out as a text file holding the
/// literal target wherever `core.symlinks` is false, which an agent would read
/// as the whole instruction (DECISIONS.md §27).
fn link_to_agents(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink("AGENTS.md", path);
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_file("AGENTS.md", path);
    #[cfg(not(any(unix, windows)))]
    let linked: std::io::Result<()> = Err(std::io::Error::other("symlinks unavailable"));

    match linked {
        Ok(()) => Ok(()),
        Err(_) => fs::write(path, "See @AGENTS.md\n"),
    }
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
    fn a_first_capture_leaves_the_habit_where_an_agent_will_read_it() {
        let root = temp_dir("agent-files");
        let mut workspace = Workspace::load(&root).unwrap();
        let done = capture(&mut workspace, "SEND-1", "I get a link in my inbox.").unwrap();

        assert_eq!(done.started_agent, vec!["hi/AGENTS.md", "hi/CLAUDE.md"]);
        let text = fs::read_to_string(root.join("hi/AGENTS.md")).unwrap();
        assert!(text.contains("Read the files here"));
        // The habit and nothing hi could change underneath it (DECISIONS.md §27).
        assert!(
            !text.contains("FAMILY-1"),
            "no id grammar in a file never rewritten"
        );
        assert!(root.join("hi/CLAUDE.md").symlink_metadata().is_ok());
    }

    #[test]
    fn a_later_capture_does_not_rewrite_the_habit() {
        let root = temp_dir("agent-files-once");
        let mut workspace = Workspace::load(&root).unwrap();
        capture(&mut workspace, "SEND-1", "I get a link in my inbox.").unwrap();
        // Someone edited it. hi wrote it once and has no business touching it.
        fs::write(root.join("hi/AGENTS.md"), "mine now\n").unwrap();

        let mut workspace = Workspace::load(&root).unwrap();
        let done = capture(&mut workspace, "SEND-2", "Clicking it signs me in.").unwrap();

        assert!(done.started_agent.is_empty());
        assert_eq!(
            fs::read_to_string(root.join("hi/AGENTS.md")).unwrap(),
            "mine now\n"
        );
    }

    #[test]
    fn a_reserved_family_refuses_before_anything_is_written() {
        let root = temp_dir("reserved-family");
        let mut workspace = Workspace::load(&root).unwrap();
        capture(&mut workspace, "SEND-1", "I get a link in my inbox.").unwrap();
        let before = fs::read_to_string(root.join("hi/AGENTS.md")).unwrap();

        let mut workspace = Workspace::load(&root).unwrap();
        let err = capture(&mut workspace, "AGENTS-1", "a criterion").unwrap_err();
        assert!(err.to_string().contains("cannot be a family"));

        // Refused on every platform, not only where the filesystem folds case,
        // and the instruction file is untouched (hi: CAPTURE-5).
        assert!(
            !root.join("hi/agents.md").is_file()
                || before == fs::read_to_string(root.join("hi/agents.md")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(root.join("hi/AGENTS.md")).unwrap(),
            before
        );
    }

    #[test]
    fn a_family_two_files_both_claim_is_refused_and_writes_nothing() {
        let root = temp_dir("dup-family");
        fs::write(
            root.join("hi/a.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n# A\n\n## Criteria\n\nSEND-1  One.\n",
        )
        .unwrap();
        fs::write(
            root.join("hi/b.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n# B\n\n## Criteria\n\nSEND-2  Two.\n",
        )
        .unwrap();
        let before_a = fs::read_to_string(root.join("hi/a.md")).unwrap();
        let before_b = fs::read_to_string(root.join("hi/b.md")).unwrap();
        let mut workspace = Workspace::load(&root).unwrap();
        let err = capture(&mut workspace, "SEND-3", "three").unwrap_err();
        let said = err.to_string();
        assert!(
            said.contains("hi/a.md") && said.contains("hi/b.md"),
            "{said}"
        );
        assert_eq!(fs::read_to_string(root.join("hi/a.md")).unwrap(), before_a);
        assert_eq!(fs::read_to_string(root.join("hi/b.md")).unwrap(), before_b);
    }

    #[test]
    fn a_case_still_lands_with_its_parent_when_the_family_is_split() {
        let root = temp_dir("dup-family-case");
        fs::write(
            root.join("hi/a.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n# A\n\n## Criteria\n\nSEND-1  One.\n",
        )
        .unwrap();
        fs::write(
            root.join("hi/b.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n# B\n\n## Criteria\n\nSEND-2  Two.\n",
        )
        .unwrap();
        let mut workspace = Workspace::load(&root).unwrap();
        let done = capture(&mut workspace, "SEND-1.a", "a case of one").unwrap();
        assert_eq!(done.file, "hi/a.md");
        assert!(
            fs::read_to_string(root.join("hi/a.md"))
                .unwrap()
                .contains("SEND-1.a")
        );
        assert!(
            !fs::read_to_string(root.join("hi/b.md"))
                .unwrap()
                .contains("SEND-1.a")
        );
    }

    #[test]
    fn seed_rewrites_a_template_hi_has_shipped() {
        let root = temp_dir("seed-prior");
        fs::write(
            root.join("hi/AGENTS.md"),
            include_str!("seed/agents_0_5.md"),
        )
        .unwrap();
        let workspace = Workspace::load(&root).unwrap();
        match seed_agent_files(&workspace).unwrap() {
            Seeded::Updated(file) => assert_eq!(file, "hi/AGENTS.md"),
            other => panic!("expected Updated, got {other:?}"),
        }
        // The endings the template on disk had are kept, so compare folded:
        // a checkout with autocrlf gives include_str! CRLF bytes.
        assert_eq!(
            fs::read_to_string(root.join("hi/AGENTS.md"))
                .unwrap()
                .replace("\r\n", "\n"),
            crate::out::agent_instructions()
        );
    }

    #[test]
    fn seed_refuses_a_file_somebody_edited() {
        let root = temp_dir("seed-edited");
        fs::write(root.join("hi/AGENTS.md"), "mine now\n").unwrap();
        let workspace = Workspace::load(&root).unwrap();
        let err = seed_agent_files(&workspace).unwrap_err().to_string();
        assert!(err.contains("it is yours"), "{err}");
        assert_eq!(
            fs::read_to_string(root.join("hi/AGENTS.md")).unwrap(),
            "mine now\n"
        );
    }

    #[test]
    fn seed_is_quiet_when_the_file_is_already_current() {
        let root = temp_dir("seed-current");
        fs::write(root.join("hi/AGENTS.md"), crate::out::agent_instructions()).unwrap();
        let workspace = Workspace::load(&root).unwrap();
        assert!(matches!(
            seed_agent_files(&workspace).unwrap(),
            Seeded::Current
        ));
    }

    #[test]
    fn seed_writes_the_file_when_it_is_missing() {
        let root = temp_dir("seed-missing");
        let workspace = Workspace::load(&root).unwrap();
        match seed_agent_files(&workspace).unwrap() {
            Seeded::Created(files) => {
                assert!(files.contains(&"hi/AGENTS.md".to_string()), "{files:?}");
                assert!(files.contains(&"hi/CLAUDE.md".to_string()), "{files:?}");
            }
            other => panic!("expected Created, got {other:?}"),
        }
        assert_eq!(
            fs::read_to_string(root.join("hi/AGENTS.md")).unwrap(),
            crate::out::agent_instructions()
        );
    }

    #[test]
    fn seed_recognises_a_prior_template_through_a_bom_and_crlf() {
        let root = temp_dir("seed-folded");
        let prior = include_str!("seed/agents_0_6.md")
            .replace("\r\n", "\n")
            .replace('\n', "\r\n");
        fs::write(root.join("hi/AGENTS.md"), format!("\u{feff}{prior}")).unwrap();
        let workspace = Workspace::load(&root).unwrap();
        assert!(matches!(
            seed_agent_files(&workspace).unwrap(),
            Seeded::Updated(_)
        ));
        let wrote = fs::read_to_string(root.join("hi/AGENTS.md")).unwrap();
        assert!(wrote.contains("\r\n"), "kept the endings the file had");
        assert!(!wrote.contains('\u{feff}'));
        assert_eq!(
            wrote.replace("\r\n", "\n"),
            crate::out::agent_instructions()
        );
    }

    #[test]
    fn capture_does_not_rewrite_a_known_old_template() {
        // Write-once on the capture path is what keeps a surprise rewrite off
        // the command that runs on every thought. `hi seed` is the verb
        // (hi: HABIT-6, DECISIONS.md §39).
        let root = temp_dir("capture-leaves-old");
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  One.\n",
        )
        .unwrap();
        fs::write(
            root.join("hi/AGENTS.md"),
            include_str!("seed/agents_0_5.md"),
        )
        .unwrap();
        fs::write(root.join("hi/CLAUDE.md"), "See @AGENTS.md\n").unwrap();
        let before = fs::read_to_string(root.join("hi/AGENTS.md")).unwrap();
        let mut workspace = Workspace::load(&root).unwrap();
        let done = capture(&mut workspace, "SEND-2", "two").unwrap();
        assert!(done.started_agent.is_empty());
        assert_eq!(
            fs::read_to_string(root.join("hi/AGENTS.md")).unwrap(),
            before
        );
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
    fn a_capture_leaves_the_generated_list_true() {
        // An INTENT.md typed by a person, with a count that is already wrong.
        let mut workspace = seeded("indexrefresh");
        fs::write(
            workspace.root.join("INTENT.md"),
            "# P\n\nMine.\n\n## Features\n\n<!-- hi:index -->\n- [chat](hi/chat.md): SEND (7 criteria)\n<!-- /hi:index -->\n\nAlso mine.\n",
        )
        .unwrap();

        let done = capture(&mut workspace, "SEND-2", "It reaches them.").unwrap();
        assert_eq!(done.index_error, None);

        let body = fs::read_to_string(workspace.root.join("INTENT.md")).unwrap();
        assert!(body.contains("(2 criteria)"), "{body}");
        assert!(
            body.contains("Mine.") && body.contains("Also mine."),
            "{body}"
        );
    }

    #[test]
    fn an_index_that_cannot_be_refreshed_is_not_a_failed_capture() {
        // The criterion is on disk before the index is touched, so nothing the
        // index does may turn this into a refusal (hi: INDEX-4.a).
        let mut workspace = seeded("indexrefusal");
        let intent = "# P\n\n<!-- hi:index -->\n- a\n";
        fs::write(workspace.root.join("INTENT.md"), intent).unwrap();

        let done = capture(&mut workspace, "SEND-2", "It reaches them.").unwrap();
        assert!(done.index_error.is_some(), "and hi has a line to print");
        assert!(
            fs::read_to_string(workspace.root.join("hi/chat.md"))
                .unwrap()
                .contains("**SEND-2**  It reaches them."),
            "the thought is what matters, and it landed"
        );
        assert_eq!(
            fs::read_to_string(workspace.root.join("INTENT.md")).unwrap(),
            intent,
            "the broken file is left exactly as it was"
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
