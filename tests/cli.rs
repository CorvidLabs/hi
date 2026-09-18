//! Integration tests that drive the real binary.
//!
//! The unit tests cover the modules; these cover the things only a process has:
//! argv routing, exit codes, and which stream output lands on.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_hi");

/// A throwaway repository with a `hi/` directory.
struct Repo {
    root: PathBuf,
}

impl Repo {
    fn new(name: &str) -> Repo {
        // Same mistake as `doc::write_atomically` had: a fixed path shared by
        // every process. Two `cargo test` runs at once wiped each other's
        // fixtures and the failures read as flakes.
        let root = std::env::temp_dir().join(format!("hi-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
        Repo { root }
    }

    /// A repository that has never run hi: a `.git` for `Workspace::find` to
    /// stop at, and no `hi/` at all. This is the state every first capture in
    /// an adopting repository starts in.
    fn bare(name: &str) -> Repo {
        let root = std::env::temp_dir().join(format!("hi-cli-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".git")).unwrap();
        Repo { root }
    }

    fn with_chat(name: &str) -> Repo {
        let repo = Repo::new(name);
        repo.write(
            "hi/chat.md",
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nIt should feel like texting.\n\n## Criteria\n\nSEND-1  I hit enter and it shows up.\n",
        );
        repo
    }

    fn write(&self, rel: &str, body: &str) {
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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn bare_invocation_prints_help_and_writes_nothing() {
    let repo = Repo::new("bare");
    let out = repo.run(&[]);
    let text = format!("{}{}", stdout(&out), stderr(&out));
    assert!(text.contains("Usage"), "expected help, got: {text}");
    // Nothing was created, so hi/ is still empty.
    assert_eq!(fs::read_dir(repo.root.join("hi")).unwrap().count(), 0);
}

#[test]
fn an_id_shaped_argument_captures() {
    let repo = Repo::with_chat("capture");
    let out = repo.run(&["SEND-2", "it reaches them and the mark changes to sent"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("+SEND-2"));
    assert!(
        repo.read("hi/chat.md")
            .contains("**SEND-2**  it reaches them and the mark changes to sent")
    );
}

#[test]
fn a_long_sentence_stays_on_one_line() {
    let repo = Repo::with_chat("oneline");
    let long = "I hit enter and the message shows up right away in the thread marked as sending so that I never have to wonder whether it actually went anywhere at all";
    repo.run(&["SEND-2", long]);
    let body = repo.read("hi/chat.md");
    let line = body
        .lines()
        .find(|l| l.contains("SEND-2"))
        .expect("the captured line");
    assert_eq!(line, format!("- **SEND-2**  {long}"));
}

#[test]
fn a_new_family_starts_its_own_file() {
    let repo = Repo::with_chat("newfamily");
    let out = repo.run(&["BILLING-1", "I can see exactly what I paid for"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("created"));
    let body = repo.read("hi/billing.md");
    assert!(body.contains("families: [BILLING]"));
    assert!(body.contains("**BILLING-1**  I can see exactly what I paid for"));
}

#[test]
fn an_existing_id_refuses_with_exit_1_and_writes_nothing() {
    let repo = Repo::with_chat("refuse");
    let before = repo.read("hi/chat.md");
    let out = repo.run(&["SEND-1", "something else entirely"]);

    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("already exists"), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("next free is SEND-2"),
        "{}",
        stderr(&out)
    );
    assert!(
        stdout(&out).is_empty(),
        "a refusal must print nothing to stdout"
    );
    assert_eq!(
        repo.read("hi/chat.md"),
        before,
        "a refusal must write nothing"
    );
}

#[test]
fn a_malformed_id_refuses_without_writing() {
    let repo = Repo::with_chat("malformed");
    let before = repo.read("hi/chat.md");
    let out = repo.run(&["SEND-1.a.b", "two letters in a row"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(repo.read("hi/chat.md"), before);
}

#[test]
fn check_is_clean_on_unfinished_intent() {
    // A criterion with nothing implementing it is never an error.
    let repo = Repo::with_chat("clean");
    let out = repo.run(&["check"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains("1 criterion"));
}

#[test]
fn a_criterion_in_a_file_hi_skips_is_reported_rather_than_vanishing() {
    // Written by hand, not by hi. Three bugs shipped because every test
    // asserted over files hi itself produced (DECISIONS.md §26).
    let repo = Repo::with_chat("skipped-criterion");
    repo.write(
        "hi/NOTES.md",
        "# Notes\n\n- **SEND-9**  a criterion somebody put in the wrong file.\n",
    );

    let out = repo.run(&["check"]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    assert!(text.contains("stray-criterion"), "{text}");
    assert!(text.contains("SEND-9"), "{text}");
    assert!(text.contains("hi/NOTES.md"), "{text}");
    assert!(text.contains("not lowercase"), "{text}");
}

#[test]
fn hi_s_own_files_are_not_counted_as_features() {
    let repo = Repo::with_chat("own-files");
    repo.run(&["SEND-2", "It reaches them."]);

    // AGENTS.md and the CLAUDE.md beside it sit in hi/ and are hi's, so they
    // are neither criteria files nor lines in the product's feature list.
    let out = repo.run(&["check"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(stdout(&out).contains("1 file"), "{}", stdout(&out));

    repo.run(&["index"]);
    let intent = repo.read("INTENT.md");
    assert!(!intent.contains("AGENTS"), "{intent}");
    assert!(!intent.contains("CLAUDE"), "{intent}");
}

/// The fledge lifecycle hook, driven as a process the way fledge drives it.
///
/// `scripts/nudge-behaves.sh` covers the same ground, but only the local gate
/// runs it. These run wherever `cargo test` does, which includes Windows.
#[cfg(unix)]
mod nudge {
    use std::path::PathBuf;
    use std::process::{Command, Output};

    fn nudge() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bin/fledge-hi-nudge")
    }

    fn run(moment: &str, root: Option<&std::path::Path>) -> Output {
        run_from(moment, root, &decoy())
    }

    /// A repository with no `hi/`, used as the hook's working directory.
    ///
    /// Running from the crate root instead would hide the bug this guards:
    /// hi's own repo has a `hi/`, so a hook that ignored `FLEDGE_REPO_ROOT` and
    /// guessed from its cwd would fall silent there and look correct. Mutation
    /// testing found exactly that, passing for the wrong reason.
    fn decoy() -> PathBuf {
        let d = std::env::temp_dir().join(format!("hi-nudge-decoy-{}", std::process::id()));
        std::fs::create_dir_all(d.join(".git")).unwrap();
        let _ = std::fs::remove_dir_all(d.join("hi"));
        d
    }

    fn run_from(moment: &str, root: Option<&std::path::Path>, cwd: &std::path::Path) -> Output {
        let mut cmd = Command::new(nudge());
        cmd.arg(moment);
        match root {
            // fledge sets the hook's cwd to the plugin directory, so the
            // repository is only ever knowable from the environment.
            Some(p) => cmd.env("FLEDGE_REPO_ROOT", p),
            None => cmd.env_remove("FLEDGE_REPO_ROOT"),
        };
        cmd.current_dir(cwd).output().expect("running the nudge")
    }

    fn repo(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("hi-nudge-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".git")).unwrap();
        root
    }

    /// The one that can break somebody's day: `run_lifecycle_hook` propagates a
    /// non-zero exit, so a hook that fails aborts the command that ran it.
    #[test]
    fn no_path_through_the_hook_can_abort_a_push() {
        let present = repo("exit-present");
        let with_hi = repo("exit-with-hi");
        std::fs::create_dir_all(with_hi.join("hi")).unwrap();
        let missing = PathBuf::from("/nope/does/not/exist");

        for moment in ["start", "push", "", "unknown-moment"] {
            for root in [
                Some(present.as_path()),
                Some(with_hi.as_path()),
                Some(missing.as_path()),
                None,
            ] {
                let out = run(moment, root);
                assert_eq!(
                    out.status.code(),
                    Some(0),
                    "moment {moment:?} root {root:?} must exit 0, or it aborts the command"
                );
            }
        }
        let _ = std::fs::remove_dir_all(&present);
        let _ = std::fs::remove_dir_all(&with_hi);
    }

    #[test]
    fn a_repository_with_nothing_written_down_is_told_so_at_both_moments() {
        let root = repo("speaks");
        let start = String::from_utf8(run("start", Some(&root)).stderr).unwrap();
        let push = String::from_utf8(run("push", Some(&root)).stderr).unwrap();

        assert!(start.contains("No hi/ here"), "{start}");
        assert!(push.contains("before it ships"), "{push}");
        assert_ne!(start, push, "the two moments say different things");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn nothing_reaches_stdout_so_a_json_envelope_stays_valid() {
        // `fledge work start --json` puts its envelope on stdout. A hook that
        // printed there would corrupt a command that had nothing to do with hi.
        let root = repo("stdout");
        for moment in ["start", "push"] {
            let out = run(moment, Some(&root));
            assert!(out.stdout.is_empty(), "{moment} wrote to stdout");
            assert!(!out.stderr.is_empty(), "{moment} should speak on stderr");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn it_goes_quiet_once_something_is_written_down() {
        let root = repo("quiet");
        std::fs::create_dir_all(root.join("hi")).unwrap();
        for moment in ["start", "push"] {
            let out = run(moment, Some(&root));
            assert!(
                out.stderr.is_empty(),
                "{moment} spoke despite a hi/ being present"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn it_never_guesses_at_a_repository_it_was_not_told_about() {
        // An older fledge sets no FLEDGE_REPO_ROOT. The hook's own cwd is the
        // plugin directory, which is a repository with a hi/ in it: guessing
        // from cwd would read the wrong tree and say the wrong thing.
        // The working directory here is a repository with no `hi/`, so a hook
        // that guessed from cwd would speak and this would catch it.
        assert!(
            run("start", None).stderr.is_empty(),
            "spoke with no root given"
        );
        assert!(
            run("push", None).stderr.is_empty(),
            "spoke at push with no root given"
        );
        assert!(
            run("start", Some(&PathBuf::from("/nope/does/not/exist")))
                .stderr
                .is_empty(),
            "spoke about a root that does not exist"
        );
        let not_a_repo = std::env::temp_dir().join(format!("hi-nudge-bare-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&not_a_repo);
        std::fs::create_dir_all(&not_a_repo).unwrap();
        assert!(
            run("start", Some(&not_a_repo)).stderr.is_empty(),
            "spoke outside a repository"
        );
        let _ = std::fs::remove_dir_all(&not_a_repo);
    }
}

#[test]
fn check_fails_on_a_structural_problem() {
    let repo = Repo::new("orphan");
    repo.write(
        "hi/chat.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1.a  An orphan case.\n",
    );
    let out = repo.run(&["check"]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    assert!(text.contains("orphan-case"), "{text}");
    assert!(text.contains("hi/chat.md"), "{text}");
    // The line number must be there so a person can jump to it.
    assert!(text.contains(":8") || text.contains("8:"), "{text}");
}

#[test]
fn check_fails_on_a_duplicate_id() {
    let repo = Repo::new("dupe");
    repo.write(
        "hi/chat.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  One.\nSEND-1  Again.\n",
    );
    let out = repo.run(&["check"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).contains("duplicate-id"));
}

#[test]
fn export_stdout_is_parseable_json() {
    let repo = Repo::with_chat("export");
    let out = repo.run(&["export"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let value: serde_json::Value =
        serde_json::from_str(&stdout(&out)).expect("export stdout must be valid JSON");
    assert_eq!(value["hi"], 1);
    assert_eq!(value["scope"], "repo");
    assert_eq!(value["files"][0]["criteria"][0]["id"], "SEND-1");
}

#[test]
fn export_rejects_a_scope_that_matches_nothing() {
    let repo = Repo::with_chat("badscope");
    let out = repo.run(&["export", "NOPE"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).is_empty());
}

#[test]
fn issue_prints_a_ticket_carrying_the_id() {
    let repo = Repo::with_chat("issue");
    let out = repo.run(&["issue", "SEND-1"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("hi: SEND-1"), "{text}");
    assert!(text.contains("It should feel like texting."), "{text}");
}

#[test]
fn a_ticket_unwraps_prose_and_leaves_the_file_alone() {
    // A GitHub issue body is rendered with hard line breaks on, so the margin
    // somebody wrapped their own prose at arrives as a break after every line
    // and a wide pane shows a narrow column. The ticket is unwrapped; the file
    // it was read from is not (hi: ISSUE-7, FILE-4).
    let repo = Repo::new("issue-wrapped");
    let source = "---\nhi: 1\nfamilies: [HOST]\n---\n\n# Host\n\n## Intent\n\n\
         Most people who would want this do not want a VPS, and should not have to\n\
         learn one to give their community a role that matches what they hold.\n\n\
         It has to work with nothing set up first.\n\n\
         ## Criteria\n\n- **HOST-1**  I can run this without a server.\n";
    repo.write("hi/host.md", source);

    let out = repo.run(&["issue", "HOST-1"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(
            "Most people who would want this do not want a VPS, and should not have to learn one \
             to give their community a role that matches what they hold."
        ),
        "the paragraph has to arrive whole: {text}"
    );
    assert!(
        text.contains("what they hold.\n\nIt has to work with nothing set up first."),
        "a blank line is a break somebody asked for: {text}"
    );
    assert_eq!(
        repo.read("hi/host.md"),
        source,
        "rendering a ticket must not touch the file it read"
    );
}

#[test]
fn an_id_is_never_handed_out_twice() {
    // The one promise hi makes. Six of its own write paths have broken it; each
    // block below is one of them, and the concurrent one has a test of its own
    // (hi: FILE-13, FILE-20, FILE-22, RETIRE-5, RETIRE-6).
    let repo = Repo::new("permanence");

    // 1. Retiring into a file whose `## Retired` is not the last section used
    //    to append at EOF, stranding the criterion and freeing its id.
    repo.write(
        "hi/send.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  first\n\n## Retired\n\n- **SEND-9**  old\n  retired: dropped\n\n## Notes\n\nProse after the retired block.\n",
    );
    assert!(
        repo.run(&["retire", "SEND-1", "changed my mind"])
            .status
            .success()
    );
    let after = repo.read("hi/send.md");
    let retired_at = after.find("## Retired").expect("a retired section");
    let notes_at = after.find("## Notes").expect("the notes section");
    let moved_at = after.find("- **SEND-1**").expect("the retired criterion");
    assert!(
        moved_at > retired_at && moved_at < notes_at,
        "SEND-1 must land inside ## Retired, not after ## Notes:\n{after}"
    );
    let out = repo.run(&["SEND-1", "a completely different thing"]);
    assert!(!out.status.success(), "a retired id must never be reusable");

    // 2. A reason with newlines used to write a real criterion line, burning
    //    an id nobody wrote.
    let forge = Repo::new("permanence-forge");
    forge.write(
        "hi/send.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  first\n",
    );
    assert!(
        forge
            .run(&["retire", "SEND-1", "a\n- **SEND-5**  forged\nb"])
            .status
            .success()
    );
    let body = forge.read("hi/send.md");
    // The words survive inside the reason, collapsed onto one line, which is
    // fine. What must not survive is a LINE that reads as a criterion.
    assert!(
        !body
            .lines()
            .any(|l| l.trim_start().starts_with("- **SEND-5**")),
        "a reason must never become a criterion line:\n{body}"
    );
    assert!(
        forge.run(&["SEND-5", "the real one"]).status.success(),
        "SEND-5 was never written, so it must still be free"
    );

    // 3. A criterion inside a fence under `## Criteria` was invisible to every
    //    check, and capture would hand the same id out again.
    let fenced = Repo::new("permanence-fence");
    fenced.write(
        "hi/send.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  first\n\n```markdown\n- **SEND-2**  hidden\n```\n",
    );
    let out = fenced.run(&["SEND-2", "a different thing"]);
    assert!(
        !out.status.success(),
        "an id hi cannot read is still taken: {}",
        stdout(&out)
    );

    // 4. A properly closed example of the format in somebody's own prose, which
    //    `FILE-9` explicitly invites. `retired_heading` scanned raw lines, so
    //    the `## Retired` inside the example was the section `hi retire` moved
    //    into: the criterion landed in the intent prose, the command printed
    //    success, and the id was handed straight back out (hi: FILE-22).
    let documented = Repo::new("permanence-documented");
    documented.write(
        "hi/send.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nA file has three sections, and a criterion looks like this:\n\n```markdown\n## Intent\n\n## Criteria\n\n- **SEND-4**  an example of the shape.\n\n## Retired\n```\n\nThat is all there is to it.\n\n## Criteria\n\n- **SEND-1**  I hit enter and it is on its way.\n",
    );
    let out = documented.run(&["retire", "SEND-1", "changed my mind"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let listed = stdout(&documented.run(&["ls", "--retired"]));
    assert!(
        listed.contains("SEND-1"),
        "a retirement hi cannot read back is a lost criterion:\n{listed}"
    );
    let out = documented.run(&["SEND-1", "a completely different thing"]);
    assert!(
        !out.status.success(),
        "a documented example must not free a real id: {}",
        stdout(&out)
    );
    assert!(
        documented
            .read("hi/send.md")
            .contains("```markdown\n## Intent\n\n## Criteria\n\n- **SEND-4**  an example of the shape.\n\n## Retired\n```"),
        "and the example is still the person's prose, untouched"
    );
    // The other half of the same rule: an example is an example, so the id it
    // draws was never spoken for and is still free (hi: FILE-9).
    assert!(
        documented.run(&["SEND-4", "a real one"]).status.success(),
        "a drawn id must not be burned by being drawn"
    );

    // 5. A fence the person never closed swallowed the `## Criteria` section
    //    `insert` appended below it, so capture reported the same id saved
    //    twice, both lines invisible, `hi check` clean (hi: FILE-22.a).
    let unfinished = Repo::new("permanence-unfinished");
    let half_written = "---\nhi: 1\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nI was in the middle of writing this out:\n\n```markdown\n## Criteria\n";
    unfinished.write("hi/send.md", half_written);
    for sentence in ["I hit enter.", "Something completely different."] {
        let out = unfinished.run(&["SEND-1", sentence]);
        assert!(
            !out.status.success(),
            "a capture nothing can read back is not a capture: {}",
            stdout(&out)
        );
    }
    assert_eq!(
        unfinished.read("hi/send.md"),
        half_written,
        "and the unfinished prose is left exactly as it was (hi: CAPTURE-5)"
    );
}

#[test]
fn concurrent_captures_all_land() {
    // Capture is read-modify-write. Without a lock two processes both read the
    // same original and the last write wins: eight concurrent captures used to
    // land two (hi: FILE-19).
    let repo = Repo::with_chat("concurrent");
    let handles: Vec<_> = (2..=9)
        .map(|n| {
            let root = repo.root.clone();
            std::thread::spawn(move || {
                std::process::Command::new(BIN)
                    .args([
                        "--root",
                        root.to_str().unwrap(),
                        &format!("SEND-{n}"),
                        "a sentence",
                    ])
                    .output()
                    .unwrap()
            })
        })
        .collect();
    for handle in handles {
        let _ = handle.join();
    }
    let body = repo.read("hi/chat.md");
    for n in 2..=9 {
        assert!(
            body.contains(&format!("**SEND-{n}**")),
            "SEND-{n} was lost to a concurrent write:\n{body}"
        );
    }
}

#[test]
fn concurrent_captures_into_a_repository_with_no_hi_directory_all_land() {
    // The first capture in a repository has no `hi/` to put a lock file in, and
    // the lock used to hand back a guard it had not taken when opening one
    // failed. Every bootstrap capture then ran unlocked: thirty-two of these
    // reported success into a fresh repository and nine of them were gone,
    // `hi check` said nothing, and the ids they had spent were handed out again
    // (hi: FILE-19, CAPTURE-14, DECISIONS.md §33).
    //
    // Thirty-two rather than eight because this is the shape adoption actually
    // has, and because eight did not reproduce it reliably.
    let repo = Repo::bare("bootstrap-race");
    let handles: Vec<_> = (1..=32)
        .map(|n| {
            let root = repo.root.clone();
            std::thread::spawn(move || {
                let out = std::process::Command::new(BIN)
                    .args([
                        "--root",
                        root.to_str().unwrap(),
                        &format!("SEND-{n}"),
                        "a sentence",
                    ])
                    .output()
                    .unwrap();
                (n, out)
            })
        })
        .collect();

    let mut reported = Vec::new();
    let mut refused = Vec::new();
    for handle in handles {
        let (n, out) = handle.join().unwrap();
        if out.status.success() {
            reported.push(n);
        } else {
            refused.push(format!("SEND-{n}: {}", stderr(&out).trim()));
        }
    }

    let body = repo.read("hi/send.md");
    for n in &reported {
        assert!(
            body.contains(&format!("**SEND-{n}**")),
            "SEND-{n} reported success and is not in the file:\n{body}"
        );
    }
    // Nothing may quietly refuse either: an id nobody else asked for is free.
    // The reason is printed rather than counted, because the one time this fired
    // it was a lock handoff on Windows and a count says nothing about that.
    assert!(
        refused.is_empty(),
        "every capture asked for an id of its own, and these did not land:\n{}",
        refused.join("\n")
    );
}

#[test]
fn concurrent_retires_never_bring_a_criterion_back() {
    // `retire` loaded the workspace before taking the lock and never read it
    // again, so the second retire wrote the file back from a snapshot taken
    // before the first one landed. Both printed "retired", both exited 0, and
    // one of the two criteria was live again with `hi check` reporting nothing
    // (hi: RETIRE-7, DECISIONS.md §33).
    let repo = Repo::bare("retire-race");
    for (id, sentence) in [
        ("SEND-1", "the first want"),
        ("SEND-2", "the second want"),
        ("SEND-3", "the third want"),
    ] {
        let out = repo.run(&[id, sentence]);
        assert!(out.status.success(), "{}", stderr(&out));
    }

    let handles: Vec<_> = ["SEND-1", "SEND-2"]
        .iter()
        .map(|id| {
            let root = repo.root.clone();
            let id = id.to_string();
            std::thread::spawn(move || {
                std::process::Command::new(BIN)
                    .args([
                        "--root",
                        root.to_str().unwrap(),
                        "retire",
                        &id,
                        "changed my mind",
                    ])
                    .output()
                    .unwrap()
            })
        })
        .collect();
    for handle in handles {
        let out = handle.join().unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
    }

    // Both said they retired something, so both have to be retired.
    let body = repo.read("hi/send.md");
    let retired_at = body.find("## Retired").expect("a retired section");
    for id in ["SEND-1", "SEND-2"] {
        let at = body
            .find(&format!("**{id}**"))
            .unwrap_or_else(|| panic!("{id} vanished:\n{body}"));
        assert!(at > retired_at, "{id} is live again:\n{body}");
    }
    assert!(
        body.contains("**SEND-3**"),
        "the one nobody retired stayed:\n{body}"
    );
}

#[test]
fn view_writes_a_self_contained_page() {
    let repo = Repo::with_chat("view");
    let out = repo.run(&["view"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let page = repo.read("intent.html");
    assert!(page.contains("I hit enter and it shows up."));
    assert!(page.contains("prefers-color-scheme: dark"));
    // An inline script is fine; a request to someone else's server is not.
    assert!(!page.contains("src="), "the page must fetch nothing");
    assert!(!page.contains("https://"), "the page must fetch nothing");
    // And the controls it ships are the point of the page.
    assert!(page.contains("id=\"q\""), "search");
    assert!(page.contains("<option value=\"id\">By id</option>"), "sort");
    assert!(
        page.contains("class=\"navitem\" data-value=\"chat\""),
        "filter"
    );
    assert!(page.contains("href=\"#SEND-1\""), "deep link");
    // The rail is the navigation, and it names the product rather than the tool.
    assert!(
        page.contains("<nav class=\"nav\" id=\"nav\""),
        "feature list"
    );
    assert!(page.contains("class=\"navcount\""), "per-feature counts");
}

#[test]
fn index_rewrites_only_the_generated_block() {
    let repo = Repo::with_chat("index");
    repo.write(
        "INTENT.md",
        "# Product\n\nMy own prose, which is mine.\n\n## Features\n\n<!-- hi:index -->\nstale\n<!-- /hi:index -->\n",
    );
    let out = repo.run(&["index"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let body = repo.read("INTENT.md");
    assert!(
        body.contains("My own prose, which is mine."),
        "prose must survive"
    );
    assert!(!body.contains("stale"));
    assert!(body.contains("hi/chat.md"));
}

/// An INTENT.md no hi ever wrote: hand-typed prose either side of a block whose
/// count is already wrong. Every write-path test that asserted on a file hi
/// produced itself missed four id bugs, so these assert on somebody else's file
/// (DECISIONS.md §26).
const HAND_WRITTEN_INTENT: &str = "# Product\n\nWhat we are for, in my own words.\n\n## Features\n\n<!-- hi:index -->\n- [chat](hi/chat.md): SEND (1 criterion)\n<!-- /hi:index -->\n\n## Bets\n\nIntent before code.\n";

#[test]
fn a_capture_refreshes_the_feature_list() {
    let repo = Repo::with_chat("captureindex");
    repo.write("INTENT.md", HAND_WRITTEN_INTENT);

    let out = repo.run(&["SEND-2", "it reaches them and the mark changes to sent"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let body = repo.read("INTENT.md");
    assert!(
        body.contains("(2 criteria)"),
        "the list has to be true after the command that changed it:\n{body}"
    );
    // And only the list. The prose either side of the markers is the person's.
    assert!(
        body.starts_with("# Product\n\nWhat we are for, in my own words.\n\n## Features\n\n"),
        "prose above the block is mine:\n{body}"
    );
    assert!(
        body.ends_with("<!-- /hi:index -->\n\n## Bets\n\nIntent before code.\n"),
        "prose below the block is mine:\n{body}"
    );
}

#[test]
fn a_retire_refreshes_the_feature_list() {
    // Retiring changes the live count too, so it owes the list the same.
    let repo = Repo::with_chat("retireindex");
    repo.run(&["SEND-2", "it reaches them"]);
    repo.run(&["SEND-3", "I can see when they read it"]);
    // A count that is neither the one before the retirement nor the one after,
    // so the assertion below cannot pass by the block simply never moving.
    repo.write(
        "INTENT.md",
        &HAND_WRITTEN_INTENT.replace("(1 criterion)", "(9 criteria)"),
    );

    let out = repo.run(&["retire", "SEND-3", "we dropped read receipts"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let body = repo.read("INTENT.md");
    assert!(
        body.contains("(2 criteria)"),
        "a retired id is not a feature any more:\n{body}"
    );
    assert!(
        body.contains("Intent before code."),
        "prose is mine:\n{body}"
    );
}

#[test]
fn a_capture_survives_an_index_it_cannot_refresh() {
    // hi refuses to guess where a broken block ends (INDEX-2.b). That refusal
    // must never reach a capture whose criterion is already stored.
    let repo = Repo::with_chat("indexrefusal");
    repo.write(
        "INTENT.md",
        "# Product\n\nMine.\n\n<!-- hi:index -->\n- a\n",
    );
    let before = repo.read("INTENT.md");

    let out = repo.run(&["SEND-2", "it reaches them"]);
    assert!(
        out.status.success(),
        "a stored criterion is not a failure: {}",
        stderr(&out)
    );
    assert!(
        repo.read("hi/chat.md")
            .contains("**SEND-2**  it reaches them"),
        "the criterion still lands"
    );
    assert_eq!(repo.read("INTENT.md"), before, "and nothing was guessed at");
    assert!(
        stderr(&out).contains("not refreshed"),
        "but hi says so rather than going quiet:\n{}",
        stderr(&out)
    );
}

#[test]
fn a_root_file_hi_cannot_read_survives_a_capture_byte_for_byte() {
    // Written by hand, not by hi, and not valid UTF-8. `write_index` turned
    // every read error into an empty string and then wrote the starter
    // scaffold over the file: years of somebody's prose replaced by a starter
    // prompt and a generated list, exit 0, nothing said. 0.7.0 made every
    // capture run that path (hi: INDEX-2, INDEX-2.c, DECISIONS.md §32).
    let repo = Repo::with_chat("undecodable-intent");
    let mut bytes =
        b"# Product\n\nWhy we exist, in my own words, written over three years.\n\nCaf".to_vec();
    bytes.push(0xe9); // Latin-1 e-acute: one byte, and not UTF-8.
    bytes.extend_from_slice(b" is how we spell it.\n");
    fs::write(repo.root.join("INTENT.md"), &bytes).unwrap();

    let out = repo.run(&["SEND-2", "it reaches them and the mark changes to sent"]);

    assert!(
        out.status.success(),
        "the criterion is what mattered: {}",
        stderr(&out)
    );
    assert!(
        repo.read("hi/chat.md")
            .contains("**SEND-2**  it reaches them and the mark changes to sent"),
        "and it is on disk"
    );
    assert_eq!(
        fs::read(repo.root.join("INTENT.md")).unwrap(),
        bytes,
        "a file hi could not read is never a file hi replaces"
    );
    assert!(
        stderr(&out).contains("not refreshed"),
        "but hi says so rather than going quiet:\n{}",
        stderr(&out)
    );
}

#[test]
fn hi_index_refuses_a_root_file_it_cannot_read_rather_than_replacing_it() {
    // The same read, reached by the verb that is allowed to install a section.
    // There the empty-string fallback was fatal: `existing.trim().is_empty()`
    // was true, so the starter scaffold was written straight over the file and
    // the command reported success (hi: INDEX-2, INDEX-2.c).
    let repo = Repo::with_chat("undecodable-index");
    let mut bytes = b"# Product\n\nEverything I have ever written about why: ".to_vec();
    bytes.push(0xe9);
    bytes.extend_from_slice(b"\n");
    fs::write(repo.root.join("INTENT.md"), &bytes).unwrap();

    let out = repo.run(&["index"]);

    assert!(
        !out.status.success(),
        "a file hi cannot read is not a file hi rewrites: {}",
        stdout(&out)
    );
    assert_eq!(
        fs::read(repo.root.join("INTENT.md")).unwrap(),
        bytes,
        "and every byte of it is still there"
    );
    assert!(stderr(&out).contains("INTENT.md"), "{}", stderr(&out));
}

#[test]
fn a_retired_id_in_a_file_hi_skips_is_never_handed_out_again() {
    // Written by hand, not by hi (DECISIONS.md §26). `check` reported this id
    // and `capture` reissued it, because they were two separate scans over two
    // different sets of files (hi: CAPTURE-14, DECISIONS.md §32).
    let repo = Repo::with_chat("skipped-reservation");
    repo.write(
        "hi/Archive.md",
        "# Archive\n\nWhat we dropped.\n\n## Retired\n\n- **SEND-4**  the old outbox.\n",
    );

    let out = repo.run(&["SEND-4", "a completely different thing"]);
    assert!(
        !out.status.success(),
        "an id written where hi cannot read it is still taken:\n{}",
        stdout(&out)
    );
    assert!(stderr(&out).contains("hi/Archive.md"), "{}", stderr(&out));
    assert!(stderr(&out).contains("lowercase"), "{}", stderr(&out));
    // And nothing was written on the way to refusing (hi: CAPTURE-5).
    assert!(!repo.read("hi/chat.md").contains("SEND-4"));

    // check still reports it, from the same lookup that refused it.
    let check = repo.run(&["check"]);
    assert_eq!(check.status.code(), Some(1));
    assert!(stdout(&check).contains("SEND-4"), "{}", stdout(&check));

    // An id nobody wrote anywhere is still free, so this is a reservation and
    // not a wall.
    assert!(
        repo.run(&["SEND-5", "the real one"]).status.success(),
        "SEND-5 was never written"
    );
}

#[test]
fn a_feature_list_i_deleted_stays_deleted() {
    // Refreshing the list hi generated is what INDEX-2 permits. Putting a
    // `## Features` heading back into somebody's file on every capture is
    // writing prose they removed, with no way to say no (hi: INDEX-4.c).
    let repo = Repo::with_chat("deleted-list");
    let mine = "# Product\n\nI keep this file by hand, thanks.\n";
    repo.write("INTENT.md", mine);

    let out = repo.run(&["SEND-2", "it reaches them"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(repo.read("INTENT.md"), mine, "my file, unchanged");
    assert!(
        !stderr(&out).contains("not refreshed"),
        "and nothing to apologize for:\n{}",
        stderr(&out)
    );

    // Asking for one is how you get one back.
    assert!(repo.run(&["index"]).status.success());
    let body = repo.read("INTENT.md");
    assert!(body.starts_with(mine), "prose is still mine:\n{body}");
    assert!(body.contains("## Features"), "{body}");
    assert!(body.contains("(2 criteria)"), "{body}");
}

#[test]
fn check_says_the_feature_list_is_behind_without_failing() {
    // FILE-14 lets somebody type a criterion straight into the file, and no
    // verb can see that happen. `hi check` is what notices afterwards.
    let repo = Repo::with_chat("handedit");
    repo.write(
        "hi/chat.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  I hit enter and it shows up.\nSEND-2  It reaches them.\n",
    );
    repo.write("INTENT.md", HAND_WRITTEN_INTENT);

    let out = repo.run(&["check"]);
    assert!(
        out.status.success(),
        "a stale list is never a build failure (CHECK-1): {}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("feature list is behind"),
        "{}",
        stdout(&out)
    );
    assert!(
        !stdout(&out).contains("problem"),
        "a note is not a seventh problem:\n{}",
        stdout(&out)
    );

    // And running the verb it names clears it.
    assert!(repo.run(&["index"]).status.success());
    assert!(
        !stdout(&repo.run(&["check"])).contains("feature list is behind"),
        "the note goes away once the list is true"
    );
}

#[test]
fn an_unknown_subcommand_is_a_usage_error() {
    let repo = Repo::with_chat("usage");
    let out = repo.run(&["wat"]);
    assert_eq!(out.status.code(), Some(2), "clap reports usage errors as 2");
}

#[test]
fn outside_a_repository_it_says_so_rather_than_guessing() {
    let root = std::env::temp_dir().join("hi-cli-norepo/deep/nested");
    let _ = fs::remove_dir_all(std::env::temp_dir().join("hi-cli-norepo"));
    fs::create_dir_all(&root).unwrap();
    let out = Command::new(BIN)
        .args(["check"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("not a repository"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_hindi_locale_directory_is_not_mistaken_for_a_workspace() {
    // `hi` is the ISO code for Hindi, so `public/locales/hi/` is a real thing.
    let repo = Repo::with_chat("locale");
    fs::create_dir_all(repo.root.join("public/locales/hi")).unwrap();
    repo.write("public/locales/hi/common.md", "# Strings\n\nNamaste.\n");

    let out = Command::new(BIN)
        .args(["SEND-2", "it reaches them"])
        .current_dir(repo.root.join("public/locales"))
        .output()
        .unwrap();

    assert!(out.status.success(), "{}", stderr(&out));
    // It walked up to the real workspace instead of adopting the locale dir.
    assert!(
        repo.read("hi/chat.md").contains("SEND-2"),
        "capture must land in the real workspace"
    );
    assert!(
        !repo.root.join("public/locales/hi/send.md").exists(),
        "nothing may be created inside a locale directory"
    );
}

#[test]
fn a_zero_padded_id_is_refused_rather_than_silently_renamed() {
    let repo = Repo::with_chat("padded");
    let before = repo.read("hi/chat.md");
    let out = repo.run(&["SEND-007", "padded seven"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("leading zero"), "{}", stderr(&out));
    assert_eq!(repo.read("hi/chat.md"), before);
}

#[test]
fn block_style_frontmatter_is_understood_and_preserved() {
    let repo = Repo::new("block");
    // A file whose families are a YAML block list, with a second family that is
    // used but not yet declared, so capture has to rewrite the list.
    repo.write(
        "hi/chat.md",
        "---\nhi: 1\nfamilies:\n  - SEND\n---\n\n## Criteria\n\nSEND-1  One.\nRECEIPT-1  Two.\n",
    );

    // The block list is read, so SEND is not falsely reported as undeclared.
    let check = repo.run(&["check"]);
    let text = stdout(&check);
    assert!(
        !text.contains("family SEND is not"),
        "block form must be understood:\n{text}"
    );

    // Capturing into the undeclared family adds it, keeping the file's style.
    let out = repo.run(&["RECEIPT-2", "three"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let body = repo.read("hi/chat.md");
    assert!(
        body.contains("  - SEND"),
        "block style must survive:\n{body}"
    );
    assert!(
        body.contains("  - RECEIPT"),
        "the new family joins it:\n{body}"
    );
    assert!(
        !body.contains("families: ["),
        "must not be collapsed to inline:\n{body}"
    );
    assert!(body.contains("**RECEIPT-2**  three"));
    assert!(repo.run(&["check"]).status.success());
}

#[test]
fn index_refuses_rather_than_guessing_when_a_marker_is_unclosed() {
    let repo = Repo::with_chat("unclosed");
    repo.write(
        "INTENT.md",
        "# Product\n\nMine.\n\n<!-- hi:index -->\n- a\n",
    );
    let before = repo.read("INTENT.md");
    let out = repo.run(&["index"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        repo.read("INTENT.md"),
        before,
        "a refusal must write nothing"
    );
}

#[test]
fn index_leaves_a_marker_quoted_in_prose_alone() {
    let repo = Repo::with_chat("quoted");
    repo.write(
        "INTENT.md",
        "# Product\n\nhi rewrites what is between <!-- hi:index --> and the close.\n\n## Features\n\n<!-- hi:index -->\nstale\n<!-- /hi:index -->\n\n## Bets\n\nIntent first.\n",
    );
    assert!(repo.run(&["index"]).status.success());
    let body = repo.read("INTENT.md");
    assert!(
        body.contains("hi rewrites what is between"),
        "the person's sentence must survive:\n{body}"
    );
    assert!(!body.contains("stale"));
    assert!(
        body.contains("<!-- /hi:index -->\n\n"),
        "the blank line after the block must survive:\n{body}"
    );
}

#[cfg(unix)]
#[test]
fn a_non_utf8_argument_is_reported_not_panicked() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let repo = Repo::with_chat("nonutf8");
    let bad = OsStr::from_bytes(b"caf\xe9 works offline");
    let out = Command::new(BIN)
        .arg("SEND-2")
        .arg(bad)
        .current_dir(&repo.root)
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(1), "a clean error, not exit 101");
    assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));
    assert!(stderr(&out).contains("not valid UTF-8"), "{}", stderr(&out));
}

#[test]
fn root_is_honored_by_capture_and_not_swallowed_into_the_sentence() {
    let repo = Repo::with_chat("rootflag");
    let elsewhere = std::env::temp_dir();

    for args in [
        vec![
            "--root",
            repo.root.to_str().unwrap(),
            "SEND-2",
            "it reaches them",
        ],
        vec![
            &format!("--root={}", repo.root.display())[..],
            "SEND-3",
            "and again",
        ],
    ] {
        let out = Command::new(BIN)
            .args(&args)
            .current_dir(&elsewhere)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
    }

    let body = repo.read("hi/chat.md");
    assert!(body.contains("**SEND-2**  it reaches them"), "{body}");
    assert!(body.contains("**SEND-3**  and again"), "{body}");
    assert!(
        !body.contains("--root"),
        "the flag must not land in a sentence:\n{body}"
    );
}

#[test]
fn a_bom_does_not_make_a_valid_file_look_broken() {
    let repo = Repo::new("bom");
    repo.write(
        "hi/chat.md",
        "\u{feff}---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\nSEND-1  One.\n",
    );
    let out = repo.run(&["check"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains("1 criterion"));
}

#[test]
fn a_hi_file_renders_as_a_list_not_a_wall_of_text() {
    // Markdown joins consecutive plain lines into one paragraph. Every
    // criterion must therefore be its own list item, or the file reads as a
    // block of run-together sentences wherever it is actually looked at.
    let repo = Repo::new("markdown");
    // A real repository root, so discovery has somewhere to start.
    fs::create_dir_all(repo.root.join(".git")).unwrap();
    assert!(
        repo.run(&["SEND-1", "I hit enter and it shows up"])
            .status
            .success()
    );
    assert!(
        repo.run(&["SEND-1.a", "If I have no connection it queues"])
            .status
            .success()
    );
    assert!(repo.run(&["SEND-2", "It reaches them"]).status.success());

    let body = repo.read("hi/send.md");
    let criteria: Vec<&str> = body
        .lines()
        .filter(|line| line.contains("SEND-"))
        .filter(|line| !line.contains("families"))
        .collect();

    assert_eq!(criteria.len(), 3, "one line each:\n{body}");
    assert_eq!(criteria[0], "- **SEND-1**  I hit enter and it shows up");
    assert_eq!(
        criteria[1], "  - **SEND-1.a**  If I have no connection it queues",
        "a case is a nested list item"
    );
    assert_eq!(criteria[2], "- **SEND-2**  It reaches them");

    // Round-trips: what was written parses back to the same three criteria.
    let out = repo.run(&["export"]);
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(value["files"][0]["criteria"].as_array().unwrap().len(), 3);
}

#[test]
fn a_flag_looking_word_inside_a_sentence_stays_a_word() {
    // `--root` is a flag only before the id. After it, everything is prose.
    let repo = Repo::with_chat("flagword");
    let out = repo.run(&[
        "SEND-2",
        "the",
        "--root",
        "docs",
        "option",
        "should",
        "be",
        "documented",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        repo.read("hi/chat.md")
            .contains("**SEND-2**  the --root docs option should be documented"),
        "the sentence must survive intact:\n{}",
        repo.read("hi/chat.md")
    );
}

#[test]
fn a_repository_is_a_boundary_for_discovery() {
    // Adding hi to a project nested inside another must not adopt the outer
    // project's criteria as if they were yours.
    let outer = Repo::with_chat("boundary");
    let inner = outer.root.join("child");
    fs::create_dir_all(inner.join(".git")).unwrap();

    let out = Command::new(BIN)
        .args(["check"])
        .current_dir(&inner)
        .output()
        .unwrap();

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("0 criteria"),
        "the inner repository has none of its own:\n{}",
        stdout(&out)
    );
}

#[test]
fn a_wrongly_cased_id_is_reported_rather_than_read_as_prose() {
    let repo = Repo::new("cased");
    repo.write(
        "hi/chat.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  Real.\n- **send-2**  Lowercase, meant as a criterion.\n",
    );
    let out = repo.run(&["check"]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stdout(&out).contains("unparseable-id"), "{}", stdout(&out));
    assert!(stdout(&out).contains("send-2"), "{}", stdout(&out));
}

#[test]
fn an_ordinary_hyphenated_word_is_not_mistaken_for_an_id() {
    let repo = Repo::new("hyphen");
    repo.write(
        "hi/chat.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  Real.\n\nspec-sync and well-formed are ordinary words.\n",
    );
    let out = repo.run(&["check"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
}

#[test]
fn export_accepts_the_path_it_prints() {
    let repo = Repo::with_chat("exportpath");
    for scope in ["chat", "chat.md", "hi/chat.md"] {
        let out = repo.run(&["export", scope]);
        assert!(
            out.status.success(),
            "export {scope} failed: {}",
            stderr(&out)
        );
    }
}

#[test]
fn retire_moves_a_criterion_and_its_cases_out_of_the_way() {
    let repo = Repo::new("retire");
    repo.write(
        "hi/chat.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  I can send a message.\n  - **SEND-1.a**  if I am offline it queues.\n- **SEND-2**  I can see the queue depth.\n",
    );
    let out = repo.run(&["retire", "SEND-1", "different product"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("retired"), "{}", stdout(&out));

    let body = repo.read("hi/chat.md");
    assert!(body.contains("## Retired"));
    assert!(body.contains("retired: different product"));
    // The case went with its parent, so nothing is orphaned.
    assert!(
        repo.run(&["check"]).status.success(),
        "check must still pass"
    );

    // And the id stays spoken for.
    let again = repo.run(&["SEND-1", "something else"]);
    assert_eq!(again.status.code(), Some(1), "a retired id is not free");
}

#[test]
fn retire_works_without_a_reason() {
    let repo = Repo::with_chat("retirebare");
    assert!(repo.run(&["retire", "SEND-1"]).status.success());
    let body = repo.read("hi/chat.md");
    assert!(body.contains("## Retired"));
    assert!(!body.contains("retired:"), "no reason given, so no note");
}

#[test]
fn the_first_capture_starts_the_product_intent_file() {
    let repo = Repo::new("intentfile");
    fs::create_dir_all(repo.root.join(".git")).unwrap();
    let out = repo.run(&[
        "SPEND-1",
        "As an operator, I can cap what the bot spends in a day",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        repo.root.join("INTENT.md").exists(),
        "the product-level why must exist from the first capture"
    );
    assert!(
        stdout(&out).contains("INTENT.md"),
        "and hi must say so: {}",
        stdout(&out)
    );
    // With its feature list already in it. The refresh that follows rewrites a
    // block and installs none, so the file has to be born with one
    // (hi: INDEX-1.a, INDEX-4.c).
    let intent = repo.read("INTENT.md");
    assert!(intent.contains("## Features"), "{intent}");
    assert!(
        intent.contains("[spend](hi/spend.md): SPEND (1 criterion)"),
        "and the list is true from the first capture:\n{intent}"
    );

    // check mentions it while it is still unwritten, and never fails.
    let check = repo.run(&["check"]);
    assert!(check.status.success());
    assert!(
        stdout(&check).contains("no product-level why"),
        "{}",
        stdout(&check)
    );
}

#[test]
fn a_ticket_does_not_print_the_sentence_twice() {
    let repo = Repo::new("ticket");
    repo.write(
        "hi/bot.md",
        "---\nhi: 1\nfamilies: [SPEND]\n---\n\n## Criteria\n\n- **SPEND-1**  I can cap the daily spend.\n",
    );
    let out = repo.run(&["issue", "SPEND-1"]);
    let text = stdout(&out);
    assert_eq!(
        text.matches("I can cap the daily spend").count(),
        1,
        "the heading already carries it:\n{text}"
    );
    assert!(text.contains("hi: SPEND-1"));
}

#[test]
fn what_hi_writes_passes_hi_own_check() {
    // hi retire wrote the reason after the cases while hi check looked for it
    // under the parent, so hi produced files that failed its own check.
    let repo = Repo::new("selfcheck");
    repo.write(
        "hi/gift.md",
        "---\nhi: 1\nfamilies: [GIFT]\n---\n\n## Criteria\n\n- **GIFT-1**  I can cancel a pending gift.\n  - **GIFT-1.a**  I am told when one is cancelled.\n  - **GIFT-1.b**  the refund is automatic.\n",
    );

    let out = repo.run(&["retire", "GIFT-1", "we never shipped gifting"]);
    assert!(out.status.success(), "{}", stderr(&out));
    // What went is named, not counted: a case can belong to another concern.
    assert!(
        stdout(&out).contains("GIFT-1.a, GIFT-1.b"),
        "{}",
        stdout(&out)
    );

    let check = repo.run(&["check"]);
    assert!(check.status.success(), "{}", stdout(&check));
    assert!(
        !stdout(&check).contains("does not say why"),
        "hi must not write a file that fails its own check:\n{}",
        stdout(&check)
    );

    // The reason belongs to the criterion it explains, not to the last case.
    let export = repo.run(&["export"]);
    let value: serde_json::Value = serde_json::from_str(&stdout(&export)).unwrap();
    let retired = value["files"][0]["retired"].as_array().unwrap();
    let parent = retired.iter().find(|c| c["id"] == "GIFT-1").unwrap();
    let case = retired.iter().find(|c| c["id"] == "GIFT-1.a").unwrap();
    assert_eq!(parent["retired"], "we never shipped gifting");
    assert!(
        case.get("retired").is_none(),
        "the case carries no reason of its own"
    );
}

#[test]
fn a_capture_never_brings_a_retired_criterion_back_to_life() {
    // An indented `## Retired` is a heading to the parser and was a
    // continuation line to the criterion capture spliced above it. So the
    // sentence swallowed the heading, every retired criterion below it came
    // back under `## Criteria` with its reason still attached, and `hi check`
    // exited 0 before and after (hi: FILE-22.c, DECISIONS.md §35).
    let repo = Repo::new("resurrect");
    repo.write(
        "hi/send.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n  ## Retired\n\n- **SEND-1**  Original retired intent.\n  retired: Dropped.\n",
    );

    let before = stdout(&repo.run(&["check"]));
    assert!(before.contains("1 retired"), "{before}");

    let capture = repo.run(&["SEND-2", "A new want."]);
    assert!(capture.status.success(), "{}", stderr(&capture));

    let after = stdout(&repo.run(&["check"]));
    assert!(
        after.contains("1 criterion") && after.contains("1 retired"),
        "SEND-1 stays retired and SEND-2 is the only active one: {after}"
    );

    let listed = stdout(&repo.run(&["ls"]));
    assert!(
        listed.contains("A new want.") && !listed.contains("A new want. ## Retired"),
        "the heading is not part of the sentence: {listed}"
    );

    // And the id it retired is still spent.
    let reuse = repo.run(&["SEND-1", "Different intent."]);
    assert!(!reuse.status.success(), "{}", stdout(&reuse));
}

#[test]
fn a_file_hi_cannot_read_never_frees_the_id_reserved_in_it() {
    // A retired id parked in a file hi skips is reserved (CAPTURE-14). If hi
    // cannot read that file it cannot see the reservation, and it used to
    // treat "could not read" as "nothing there": the capture succeeded, the
    // reservation stayed on disk, and `hi check` exited 0 before and after. An
    // answer hi does not have is not the answer "free" (hi: CAPTURE-15,
    // DECISIONS.md §36).
    let repo = Repo::new("unreadable");
    repo.write(
        "hi/send.md",
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-2**  Existing.\n",
    );
    // One Latin-1 byte in the retirement reason, which is how a pasted "café"
    // arrives out of an older file.
    fs::write(
        repo.root.join("hi/Archive.md"),
        b"---\nhi: 1\nfamilies: [SEND]\n---\n\n## Retired\n\n- **SEND-1**  Original.\n  retired: Old caf\xe9.\n".as_slice(),
    )
    .unwrap();

    let before: Vec<String> = listing(&repo.root);

    let check = repo.run(&["check"]);
    assert!(!check.status.success(), "{}", stdout(&check));
    assert!(
        stderr(&check).contains("hi/Archive.md"),
        "it names the file it could not read: {}",
        stderr(&check)
    );

    let capture = repo.run(&["SEND-1", "Different intent."]);
    assert!(
        !capture.status.success(),
        "an id hi cannot rule out is not free: {}",
        stdout(&capture)
    );

    assert_eq!(
        listing(&repo.root),
        before,
        "and the refusal wrote nothing at all, INTENT.md and hi/AGENTS.md included"
    );
    assert_eq!(
        repo.read("hi/send.md"),
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-2**  Existing.\n"
    );
}

/// Every path under `root`, with its bytes, so a refusal can be held to
/// changing nothing at all rather than to changing no criteria.
fn listing(root: &std::path::Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let name = path.strip_prefix(root).unwrap().display().to_string();
                found.push(format!("{name}\u{0}{:?}", fs::read(&path).unwrap()));
            }
        }
    }
    found.sort();
    found
}

/// A repository whose one criteria file declares a format version this hi has
/// never heard of.
fn future_format(name: &str) -> Repo {
    let repo = Repo::new(name);
    repo.write(
        "hi/chat.md",
        "---\nhi: 2\nfamilies: [SEND]\n---\n\n# Chat\n\n## Intent\n\nWhy.\n\n## Criteria\n\n- **SEND-1**  I hit enter and it shows up.\n",
    );
    repo
}

#[test]
fn a_file_from_a_later_format_is_refused_by_every_verb() {
    // `hi:` was written by capture, stored by the parser, and read by nothing.
    // A file saying `hi: 2` was parsed, checked and appended to as HI/1: the
    // number could never be frozen, and an HI/2 could never ship, because
    // every binary already released would edit an HI/2 file believing it
    // understood it (hi: FILE-25).
    let repo = future_format("version-refused");

    for args in [
        vec!["check"],
        vec!["check", "--json"],
        vec!["ls"],
        vec!["ls", "--retired"],
        vec!["export"],
        vec!["issue", "SEND-1"],
        vec!["index"],
        vec!["view"],
        vec!["retire", "SEND-1", "changed my mind"],
        vec!["SEND-2", "a new want"],
    ] {
        let out = repo.run(&args);
        let said = stderr(&out);
        assert_eq!(
            out.status.code(),
            Some(1),
            "`hi {}` should refuse: {said}",
            args.join(" ")
        );
        assert!(
            said.contains("hi/chat.md") && said.contains("hi: 2"),
            "`hi {}` must name the file and the version: {said}",
            args.join(" ")
        );
        assert!(
            stdout(&out).is_empty(),
            "`hi {}` printed a result it did not have",
            args.join(" ")
        );
    }
}

#[test]
fn a_refusal_over_the_format_version_writes_nothing_at_all() {
    // The refusal is in `Workspace::load`, which runs before `lock::acquire`,
    // so a refused capture has not even made the lock file. The file it could
    // not read is untouched, and nothing hi normally starts on a first capture
    // exists (hi: FILE-25.a, CAPTURE-5).
    let repo = future_format("version-writes-nothing");
    let before = repo.read("hi/chat.md");

    let out = repo.run(&["SEND-2", "a new want"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));

    assert_eq!(
        repo.read("hi/chat.md"),
        before,
        "the file it refused to read"
    );
    for path in ["INTENT.md", "hi/AGENTS.md", "hi/CLAUDE.md", "hi/.hi.lock"] {
        assert!(
            !repo.root.join(path).exists(),
            "{path} was created by a command that refused"
        );
    }
    assert_eq!(
        fs::read_dir(repo.root.join("hi")).unwrap().count(),
        1,
        "only the file that was already there"
    );
}

#[test]
fn a_file_that_declares_nothing_is_still_this_format() {
    // Files written before `hi:` existed have no key at all, and they are
    // HI/1. Refusing them would be the version check breaking the format it
    // exists to protect (hi: FILE-25).
    // Discovery itself looks for the `hi:` key, so this file is found through
    // the repository boundary rather than through the key it does not have.
    let repo = Repo::bare("version-absent");
    fs::create_dir_all(repo.root.join("hi")).unwrap();
    repo.write(
        "hi/chat.md",
        "---\nfamilies: [SEND]\n---\n\n# Chat\n\n## Criteria\n\n- **SEND-1**  One.\n",
    );
    let out = repo.run(&["ls"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("SEND-1"), "{}", stdout(&out));
}

#[test]
fn check_json_carries_its_notes_as_a_list_with_codes() {
    // The notes were one string joined with a newline and six spaces of
    // terminal indentation, so JSON could not carry them apart and a reader
    // had to split on whitespace to get them back (hi: CHECK-6).
    let repo = Repo::with_chat("note-codes");
    // No prose of its own, and a generated list that counts the wrong number:
    // two notes at once, which is the case the joined string existed for.
    repo.write(
        "INTENT.md",
        "# P\n\n## Features\n\n<!-- hi:index -->\n- [chat](hi/chat.md): SEND (9 criteria)\n<!-- /hi:index -->\n",
    );
    let out = repo.run(&["check", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let json = stdout(&out);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    let notes = value["notes"].as_array().expect("notes is a list");
    let codes: Vec<&str> = notes
        .iter()
        .map(|note| note["kind"].as_str().expect("a code"))
        .collect();
    assert!(
        codes.contains(&"no-product-why") && codes.contains(&"index-behind"),
        "two separate notes, each under its own code: {json}"
    );
    for note in notes {
        let message = note["message"].as_str().expect("a message");
        assert!(
            !message.contains('\n'),
            "a note carries no layout of its own: {message:?}"
        );
    }
    assert!(
        value["note"].is_null(),
        "the joined string is gone, not kept beside the list: {json}"
    );
}

#[test]
fn export_says_which_shape_it_is_apart_from_which_format_it_read() {
    // `hi` is the version of the files. It cannot also be the version of this
    // payload's shape, or the day the shape changes every consumer is told the
    // file format moved (hi: EXPORT-6).
    let repo = Repo::with_chat("envelope");
    let out = repo.run(&["export"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid json");
    assert_eq!(value["hi"], 1, "the format the files are in");
    assert_eq!(value["export"], 1, "the shape of this payload");
}

#[test]
fn the_agent_file_says_to_check_the_ids_after_a_merge() {
    // An id is only unique against the tree it was captured on. Two branches
    // choosing the same id merge cleanly and git says nothing, which is the
    // one failure mode the promise has left and the one this file never
    // mentioned (hi: HABIT-5, DECISIONS.md §37, §38).
    let repo = Repo::bare("agent-merge");
    let out = repo.run(&["SEND-1", "I hit enter and it shows up"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let text = repo.read("hi/AGENTS.md");
    assert!(text.contains("hi check"), "{text}");
    assert!(text.contains("merge"), "{text}");
    assert!(text.contains("same id"), "{text}");
}
