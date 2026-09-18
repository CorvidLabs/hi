//! Property tests of the one promise: hi's own verbs never reuse an id, a
//! criterion they said they saved is one they can find again, and a step
//! leaves every criterion it did not name exactly as it was.
//!
//! Twelve confirmed defects were found by people writing a case. None were
//! found by hi generating one. These tests generate files hi did not write
//! and then run random capture / retire / hand-edit sequences over them
//! (hi: ID-5, ID-5.a, FILE-22, DECISIONS.md §26, §35).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::capture;
use crate::check;
use crate::doc::Section;
use crate::id::Id;
use crate::workspace::Workspace;

/// Splitmix64. Enough to walk a space of files and steps, no new dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.next() as usize % items.len()]
    }

    fn below(&mut self, n: u32) -> u32 {
        (self.next() as u32) % n.max(1)
    }
}

/// One criterion as a reader of the file would find it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Shape {
    id: String,
    section: Section,
    text: String,
    note: Option<String>,
}

fn shapes_of(workspace: &Workspace) -> Vec<Shape> {
    let mut shapes: Vec<Shape> = workspace
        .docs
        .iter()
        .flat_map(|doc| doc.all())
        .filter_map(|c| {
            let id = c.id.as_ref()?.to_string();
            Some(Shape {
                id,
                section: c.section,
                text: c.text.clone(),
                note: c.note.clone(),
            })
        })
        .collect();
    shapes.sort();
    shapes
}

fn ids_in(workspace: &Workspace) -> BTreeMap<String, Vec<(Section, String)>> {
    let mut map: BTreeMap<String, Vec<(Section, String)>> = BTreeMap::new();
    for doc in &workspace.docs {
        for c in doc.all() {
            let Some(id) = c.id.as_ref() else { continue };
            map.entry(id.to_string())
                .or_default()
                .push((c.section, c.text.clone()));
        }
    }
    map
}

fn readable_in(workspace: &Workspace, id: &str, section: Section) -> bool {
    workspace.docs.iter().any(|doc| {
        doc.all().any(|c| {
            c.id.as_ref().is_some_and(|parsed| parsed.to_string() == id) && c.section == section
        })
    })
}

fn unnamed_shapes(before: &[Shape], named: &BTreeSet<String>) -> Vec<Shape> {
    before
        .iter()
        .filter(|s| !named.contains(&s.id))
        .cloned()
        .collect()
}

fn temp_root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("hi-promise-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("hi")).unwrap();
    dir
}

/// Files a person typed. None of these is what capture would have written.
fn generated_body(kind: u32) -> String {
    match kind % 6 {
        0 => "---\nhi: 1\nfamilies: [WANT]\n---\n\n# Want\n\n## Intent\n\nWhy this exists, typed by hand.\n\n## Criteria\n\nWANT-1  A thought somebody typed.\nWANT-2  Another thought, still bare.\n".into(),
        1 => "---\nhi: 1\nfamilies: [WANT]\n---\n\n# Want\n\n## Criteria\n\n- **WANT-1**  A list item they typed.\n  - **WANT-1.a**  A case hanging off it.\n- **WANT-2**  A sibling.\n".into(),
        2 => "---\nhi: 1\nfamilies: [WANT]\n---\n\n# Want\n\n## Intent\n\nShowing the format inside the prose:\n\n```markdown\n## Criteria\n\n- **WANT-9**  an example, not a criterion.\n```\n\n## Criteria\n\n- **WANT-1**  The real one.\n\n## Retired\n\n- **WANT-3**  We changed our minds.\n  retired: not this product\n".into(),
        3 => "---\nhi: 1\nfamilies: [WANT, HOLD]\n---\n\n# Mixed\n\n## Criteria\n\n- **WANT-1**  One family.\n- **HOLD-1**  Another family in the same file.\n".into(),
        4 => "---\r\nhi: 1\r\nfamilies: [WANT]\r\n---\r\n\r\n# Want\r\n\r\n## Criteria\r\n\r\n- **WANT-1**  Windows endings, typed in notepad.\r\n".into(),
        _ => "---\nhi: 1\nfamilies: [WANT]\n---\n\n# Want\n\n## Intent\n\nHalfway through a thought.\n\n## Criteria\n\n- **WANT-1**  Live.\n\nWANT-8  A line that landed outside after a heading moved.\n\n# Appendix\n\nNotes, not criteria.\n".into(),
    }
}

fn live_ids(workspace: &Workspace) -> Vec<Id> {
    workspace
        .docs
        .iter()
        .flat_map(|doc| doc.criteria.iter())
        .filter_map(|c| c.id.clone())
        .collect()
}

fn all_parsed_ids(workspace: &Workspace) -> Vec<Id> {
    workspace
        .docs
        .iter()
        .flat_map(|doc| doc.all())
        .filter_map(|c| c.id.clone())
        .collect()
}

fn next_top_level(workspace: &Workspace, family: &str) -> String {
    format!("{family}-{}", workspace.next_free(family))
}

fn assert_promise(
    root: &Path,
    reported: &[(String, Section)],
    before: &[Shape],
    named: &BTreeSet<String>,
    assigned: &BTreeMap<String, String>,
) {
    let workspace = Workspace::load(root).expect("workspace still loads");
    let after = shapes_of(&workspace);

    for (id, section) in reported {
        assert!(
            readable_in(&workspace, id, *section),
            "{id} was reported saved in {section:?} and Workspace::load cannot find it there"
        );
    }

    let before_unnamed = unnamed_shapes(before, named);
    let after_unnamed = unnamed_shapes(&after, named);
    assert_eq!(
        after_unnamed, before_unnamed,
        "a step moved a criterion it did not name"
    );

    let seen = ids_in(&workspace);
    for (id, appearances) in &seen {
        if appearances.len() > 1 {
            // Two lines, same id. hi's verbs must not have been the ones that
            // wrote a second sentence under it.
            if let Some(first) = assigned.get(id) {
                for (_, text) in appearances {
                    assert_eq!(
                        text, first,
                        "{id} was assigned a second sentence by a verb: first {first:?}, now {text:?}"
                    );
                }
            }
        }
    }

    if check::run(&workspace).expect("check is operational").ok() {
        for (id, appearances) in &seen {
            assert_eq!(
                appearances.len(),
                1,
                "hi check exited 0 with {id} written twice: {appearances:?}"
            );
        }
        for (id, section) in reported {
            assert!(
                readable_in(&workspace, id, *section),
                "hi check exited 0 but {id} is not readable in {section:?}"
            );
        }
        assert_eq!(
            after_unnamed, before_unnamed,
            "hi check exited 0 but an unnamed criterion moved"
        );
    }
}

fn step(rng: &mut Rng, root: &Path, assigned: &mut BTreeMap<String, String>) {
    let workspace = Workspace::load(root).unwrap();
    let before = shapes_of(&workspace);
    let live = live_ids(&workspace);
    let all = all_parsed_ids(&workspace);
    let kind = rng.below(6);

    match kind {
        0 | 1 => {
            // A new top-level id, or a case of something live.
            let (raw, sentence) = if kind == 1 && !live.is_empty() {
                let parent = rng.pick(&live);
                let letter = ((b'a') + (rng.below(8) as u8)) as char;
                let raw = format!("{parent}.{letter}");
                (raw, format!("a generated case of {parent}"))
            } else {
                let family = if rng.below(4) == 0 { "HOLD" } else { "WANT" };
                (
                    next_top_level(&workspace, family),
                    format!("generated want {}", rng.next()),
                )
            };
            let mut workspace = workspace;
            match capture::capture(&mut workspace, &raw, &sentence) {
                Ok(done) => {
                    let id = done.id.to_string();
                    if let Some(first) = assigned.get(&id) {
                        panic!("{id} assigned twice: first {first:?}, then {sentence:?}");
                    }
                    assigned.insert(id.clone(), sentence);
                    let named = BTreeSet::from([id.clone()]);
                    assert_promise(root, &[(id, Section::Criteria)], &before, &named, assigned);
                }
                Err(_) => {
                    // A refusal writes nothing, so every shape is unnamed.
                    assert_promise(root, &[], &before, &BTreeSet::new(), assigned);
                }
            }
        }
        2 => {
            // Capture an id that is already spoken for, which must refuse.
            let raw = if all.is_empty() {
                "WANT-1".to_string()
            } else {
                rng.pick(&all).to_string()
            };
            let mut workspace = workspace;
            let err = capture::capture(&mut workspace, &raw, "a second sentence for a taken id");
            assert!(err.is_err(), "a taken id must refuse, got a write of {raw}");
            assert_promise(root, &[], &before, &BTreeSet::new(), assigned);
        }
        3 if !live.is_empty() => {
            let target = rng.pick(&live);
            let mut workspace = Workspace::load(root).unwrap();
            let Some((index, _)) = workspace.find_id(target) else {
                return;
            };
            let taken = {
                let doc = &mut workspace.docs[index];
                match doc.retire(target, Some("generated change of mind")) {
                    Ok(taken) => {
                        doc.save().unwrap();
                        taken
                    }
                    Err(_) => {
                        assert_promise(root, &[], &before, &BTreeSet::new(), assigned);
                        return;
                    }
                }
            };
            let mut named = BTreeSet::from([target.to_string()]);
            for id in &taken {
                named.insert(id.clone());
            }
            let reported: Vec<(String, Section)> = named
                .iter()
                .map(|id| (id.clone(), Section::Retired))
                .collect();
            assert_promise(root, &reported, &before, &named, assigned);
        }
        4 if !all.is_empty() => {
            // Hand-edit a sentence. The person named that id; everyone else
            // stays put. This is a file hi did not write the new words of.
            let target = rng.pick(&all);
            let workspace = Workspace::load(root).unwrap();
            let Some((index, found)) = workspace.find_id(target) else {
                return;
            };
            let old_text = found.text.clone();
            let path = workspace.docs[index].path.clone();
            let body = fs::read_to_string(&path).unwrap();
            let new_text = format!("{old_text} (edited {})", rng.next());
            // Replace the line that carries this id, not the first occurrence of
            // its sentence: two hand-typed lines can share a prefix, and a
            // substring replace would edit the wrong criterion.
            let updated = if let Some(line) = body
                .lines()
                .find(|l| l.contains(&target.to_string()) && l.contains(&old_text))
            {
                body.replacen(line, &line.replacen(&old_text, &new_text, 1), 1)
            } else {
                body.replacen(&old_text, &new_text, 1)
            };
            fs::write(&path, updated).unwrap();
            let named = BTreeSet::from([target.to_string()]);
            assert_promise(root, &[], &before, &named, assigned);
        }
        _ => {
            // Hand-add a criterion hi did not write, with an id nothing has.
            let family = "WANT";
            let raw = next_top_level(&workspace, family);
            let path = workspace
                .docs
                .first()
                .map(|d| d.path.clone())
                .unwrap_or_else(|| root.join("hi/want.md"));
            let mut body = if path.exists() {
                fs::read_to_string(&path).unwrap()
            } else {
                "---\nhi: 1\nfamilies: [WANT]\n---\n\n## Criteria\n\n".into()
            };
            if !body.contains("## Criteria") {
                body.push_str("\n## Criteria\n");
            }
            body.push_str(&format!("- **{raw}**  a hand-typed line for {raw}.\n"));
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(&path, body).unwrap();
            let named = BTreeSet::from([raw]);
            assert_promise(root, &[], &before, &named, assigned);
        }
    }
}

#[test]
fn random_sequences_over_files_hi_did_not_write_keep_the_promise() {
    for seed in [1u64, 7, 13, 29, 41, 99, 256, 1024] {
        let root = temp_root(&format!("serial-{seed}"));
        fs::write(root.join("hi/want.md"), generated_body((seed % 6) as u32)).unwrap();
        let mut rng = Rng(seed);
        let mut assigned = BTreeMap::new();
        // Anything already in the generated file was not assigned by a verb.
        for _ in 0..24 {
            step(&mut rng, &root, &mut assigned);
        }
        let workspace = Workspace::load(&root).unwrap();
        let seen = ids_in(&workspace);
        for (id, appearances) in &seen {
            if let Some(first) = assigned.get(id) {
                for (_, text) in appearances {
                    if appearances.len() == 1 {
                        let _ = (first, text);
                    }
                }
            }
        }
        // Every id a verb reported as saved is still readable somewhere.
        for id in assigned.keys() {
            assert!(
                seen.contains_key(id),
                "seed {seed}: {id} was saved and is gone"
            );
        }
    }
}

#[test]
fn a_file_hi_did_not_write_is_where_the_sequence_starts() {
    // Pin the fixture: the starting file is not the list-item form capture
    // writes, so a pass that only ever round-trips hi's own output cannot
    // satisfy this test (hi: ID-5, DECISIONS.md §26).
    let root = temp_root("not-ours");
    let body = generated_body(0);
    assert!(!body.contains("**WANT-1**"), "{body}");
    fs::write(root.join("hi/want.md"), &body).unwrap();
    let mut assigned = BTreeMap::new();
    let mut rng = Rng(3);
    for _ in 0..8 {
        step(&mut rng, &root, &mut assigned);
    }
    let workspace = Workspace::load(&root).unwrap();
    assert!(
        workspace.find_id(&Id::parse("WANT-1").unwrap()).is_some(),
        "the typed WANT-1 is still there"
    );
}
