//! Concurrent property tests of the promise, through the real binary.
//!
//! Serial sequences live in `src/promise.rs` so they can ask `Workspace::load`
//! about section and shape. These cover the other half: two processes, a file
//! hi did not write, and the same assertions after the dust settles
//! (hi: ID-5, ID-5.a, FILE-19).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_hi");

struct Repo {
    root: PathBuf,
}

impl Repo {
    fn bare(name: &str) -> Repo {
        let root =
            std::env::temp_dir().join(format!("hi-promise-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".git")).unwrap();
        Repo { root }
    }

    fn write(&self, rel: &str, body: &str) {
        if let Some(parent) = self.root.join(rel).parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(self.root.join(rel), body).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.root.join(rel)).unwrap()
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("running hi")
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// A file a person typed: bare lines, no list markers, two thoughts already in
/// it. Capture would have written `- **WANT-1**`.
fn typed_file() -> &'static str {
    "---\nhi: 1\nfamilies: [WANT]\n---\n\n# Want\n\n## Intent\n\nTyped by hand, not by hi.\n\n## Criteria\n\nWANT-1  The first thought they typed.\nWANT-2  The second.\n"
}

#[test]
fn concurrent_captures_of_their_own_ids_all_land_in_a_file_hi_did_not_write() {
    let repo = Repo::bare("conc-own");
    repo.write("hi/want.md", typed_file());
    assert!(
        !repo.read("hi/want.md").contains("**WANT-1**"),
        "the starting file is not one hi wrote"
    );

    let handles: Vec<_> = (3..=10)
        .map(|n| {
            let root = repo.root.clone();
            std::thread::spawn(move || {
                let out = Command::new(BIN)
                    .args([
                        "--root",
                        root.to_str().unwrap(),
                        &format!("WANT-{n}"),
                        "a concurrent want",
                    ])
                    .output()
                    .unwrap();
                (n, out)
            })
        })
        .collect();

    let mut saved = Vec::new();
    for handle in handles {
        let (n, out) = handle.join().unwrap();
        if out.status.success() {
            assert!(
                stdout(&out).contains(&format!("+WANT-{n}")),
                "reported saved: {}",
                stdout(&out)
            );
            saved.push(n);
        } else {
            panic!("WANT-{n} refused: {}", stderr(&out));
        }
    }

    let body = repo.read("hi/want.md");
    for n in &saved {
        assert!(
            body.contains(&format!("WANT-{n}")),
            "WANT-{n} was reported saved and is not readable:\n{body}"
        );
    }
    assert!(body.contains("The first thought they typed."));
    assert!(body.contains("The second."));

    let check = repo.run(&["check"]);
    if check.status.success() {
        for n in &saved {
            assert!(body.contains(&format!("WANT-{n}")));
        }
        let ids: Vec<_> = saved.iter().map(|n| format!("WANT-{n}")).collect();
        let unique: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "an id was assigned twice");
    }
}

#[test]
fn concurrent_captures_of_the_same_id_assign_it_once() {
    let repo = Repo::bare("conc-same");
    repo.write("hi/want.md", typed_file());

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let root = repo.root.clone();
            std::thread::spawn(move || {
                Command::new(BIN)
                    .args([
                        "--root",
                        root.to_str().unwrap(),
                        "WANT-3",
                        &format!("sentence from waiter {i}"),
                    ])
                    .output()
                    .unwrap()
            })
        })
        .collect();

    let mut wins = 0;
    let mut sentences = Vec::new();
    for handle in handles {
        let out = handle.join().unwrap();
        if out.status.success() {
            wins += 1;
            sentences.push(stdout(&out));
        }
    }
    assert_eq!(
        wins, 1,
        "the same id was assigned {wins} times: {sentences:?}"
    );

    let body = repo.read("hi/want.md");
    let count = body.matches("WANT-3").count();
    assert_eq!(count, 1, "WANT-3 appears {count} times:\n{body}");
    assert!(body.contains("The first thought they typed."));
}

#[test]
fn concurrent_retire_and_capture_keep_every_unnamed_criterion() {
    let repo = Repo::bare("conc-mix");
    repo.write("hi/want.md", typed_file());
    let out = repo.run(&["WANT-3", "a third thought"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let capture = {
        let root = repo.root.clone();
        std::thread::spawn(move || {
            Command::new(BIN)
                .args([
                    "--root",
                    root.to_str().unwrap(),
                    "WANT-4",
                    "a fourth thought",
                ])
                .output()
                .unwrap()
        })
    };
    let retire = {
        let root = repo.root.clone();
        std::thread::spawn(move || {
            Command::new(BIN)
                .args([
                    "--root",
                    root.to_str().unwrap(),
                    "retire",
                    "WANT-2",
                    "generated change of mind",
                ])
                .output()
                .unwrap()
        })
    };

    let captured = capture.join().unwrap();
    let retired = retire.join().unwrap();
    assert!(captured.status.success(), "{}", stderr(&captured));
    assert!(retired.status.success(), "{}", stderr(&retired));

    let body = repo.read("hi/want.md");
    assert!(
        body.contains("The first thought they typed."),
        "WANT-1 was not named and moved:\n{body}"
    );
    assert!(
        body.contains("WANT-4"),
        "reported saved and missing:\n{body}"
    );
    let retired_at = body.find("## Retired").expect("a retired section");
    let want2 = body.find("WANT-2").expect("WANT-2 vanished");
    assert!(want2 > retired_at, "WANT-2 is live again:\n{body}");
}
