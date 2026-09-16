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
        let root = std::env::temp_dir().join(format!("hi-cli-{name}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
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
fn view_writes_a_self_contained_page() {
    let repo = Repo::with_chat("view");
    let out = repo.run(&["view"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let page = repo.read("intent.html");
    assert!(page.contains("I hit enter and it shows up."));
    assert!(page.contains("prefers-color-scheme: dark"));
    assert!(!page.contains("<script"), "the page must carry no script");
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
        "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  As a member, I can send a message.\n  - **SEND-1.a**  As a member, if I am offline it queues.\n- **SEND-2**  As an operator, I can see the queue depth.\n",
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
fn a_criterion_carries_the_role_it_speaks_for() {
    let repo = Repo::new("roles");
    repo.write(
        "hi/bot.md",
        "---\nhi: 1\nfamilies: [SPEND, PLAY]\n---\n\n## Criteria\n\n- **SPEND-2**  As an operator, I can cap what the bot spends in a day.\n- **PLAY-2**  As a member, I can see what I won without opening my wallet.\n",
    );

    let ls = repo.run(&["ls"]);
    assert!(stdout(&ls).contains("[operator]"), "{}", stdout(&ls));
    assert!(stdout(&ls).contains("[member]"), "{}", stdout(&ls));

    let export = repo.run(&["export"]);
    let value: serde_json::Value = serde_json::from_str(&stdout(&export)).unwrap();
    let criteria = value["files"][0]["criteria"].as_array().unwrap();
    assert_eq!(criteria[0]["role"], "operator");
    assert_eq!(criteria[1]["role"], "member");

    // And the page shows who is speaking rather than an undifferentiated "I".
    assert!(repo.run(&["view"]).status.success());
    let page = repo.read("intent.html");
    assert!(
        page.contains("class=\"role\""),
        "the page must label the voice"
    );
    // A span with no rule behind it renders as bare text jammed into the
    // sentence, which is how it shipped in 0.2.0 through 0.2.3. Assert the
    // style exists, not just the markup.
    assert!(
        page.contains(".role {"),
        "the role label must be styled, not merely emitted"
    );
    assert!(page.contains("operator") && page.contains("member"));
}

#[test]
fn a_ticket_does_not_print_the_sentence_twice() {
    let repo = Repo::new("ticket");
    repo.write(
        "hi/bot.md",
        "---\nhi: 1\nfamilies: [SPEND]\n---\n\n## Criteria\n\n- **SPEND-1**  As an operator, I can cap the daily spend.\n",
    );
    let out = repo.run(&["issue", "SPEND-1"]);
    let text = stdout(&out);
    assert_eq!(
        text.matches("I can cap the daily spend").count(),
        1,
        "the heading already carries it:\n{text}"
    );
    assert!(text.contains("Speaking as operator"), "{text}");
    assert!(text.contains("hi: SPEND-1"));
}

#[test]
fn what_hi_writes_passes_hi_own_check() {
    // hi retire wrote the reason after the cases while hi check looked for it
    // under the parent, so hi produced files that failed its own check.
    let repo = Repo::new("selfcheck");
    repo.write(
        "hi/gift.md",
        "---\nhi: 1\nfamilies: [GIFT]\n---\n\n## Criteria\n\n- **GIFT-1**  As an operator, I can cancel a pending gift.\n  - **GIFT-1.a**  As a member, I am told when one is cancelled.\n  - **GIFT-1.b**  As an operator, the refund is automatic.\n",
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
