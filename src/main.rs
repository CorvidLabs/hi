//! hi (Human Intent) keeps acceptance criteria in human words, with permanent ids.
//!
//! hi holds intent and identity. It stores no state, tracks no lifecycle, binds
//! no evidence, and never fails a build because a criterion is unproven. See
//! DECISIONS.md for why each of those is deliberate.

mod capture;
mod check;
mod doc;
mod id;
mod lock;
mod out;
mod view;
mod workspace;

#[cfg(test)]
mod promise;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

use workspace::Workspace;

#[derive(Parser)]
#[command(
    name = "hi",
    version,
    about = "Human Intent: acceptance criteria in human words, with permanent ids",
    long_about = "hi (Human Intent)\n\n\
        Write what you actually want, one line at a time, with an id that never \
        moves. Tickets and specs are generated from it.\n\n\
        Capture is the default action:\n    \
        hi SEND-2 \"it reaches them and the mark changes to sent\"",
    arg_required_else_help = true
)]
struct Cli {
    /// Directory to work from. Defaults to the current directory.
    #[arg(long, global = true, value_name = "PATH")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check every hi file for structural problems
    Check {
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
    },
    /// List criteria
    Ls {
        /// Only this family
        #[arg(long, value_name = "FAMILY")]
        family: Option<String>,
        /// Include retired criteria
        #[arg(long)]
        retired: bool,
    },
    /// Print a criterion as a ticket, or open one with --create
    Issue {
        /// The criterion id, for example SEND-1
        id: String,
        /// Open a real GitHub issue with `gh` instead of printing
        #[arg(long)]
        create: bool,
        /// Target repository, as owner/name
        #[arg(long, value_name = "OWNER/NAME")]
        repo: Option<String>,
    },
    /// Emit intent and criteria as JSON for an agent
    Export {
        /// A family, a file stem, an id for just that criterion, or nothing for the whole repository
        scope: Option<String>,
    },
    /// Move a criterion into Retired, keeping its id reserved forever
    Retire {
        /// The criterion id, for example SEND-3
        id: String,
        /// Why you changed your mind. Optional, and worth typing.
        reason: Option<String>,
    },
    /// Regenerate the feature index inside INTENT.md
    Index,
    /// Write a readable page of the intent, for people who do not read markdown
    View {
        /// Where to write it, relative to the repository root
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
    },
    /// Rewrite hi/AGENTS.md when it is still a template hi has shipped
    Seed,
}

fn main() -> ExitCode {
    // Capture is the default action, so an id-shaped first argument routes
    // there before clap sees it. Flags never look like ids.
    // `env::args()` panics on a non-UTF-8 argument, so read the OS strings and
    // report the problem instead of aborting with a backtrace.
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let (root, tail) = peel_root(&args[1..]);

    let is_capture = tail
        .first()
        .and_then(|arg| arg.to_str())
        .is_some_and(id::looks_like_id);

    if is_capture {
        let words: Option<Vec<String>> = tail
            .iter()
            .map(|arg| arg.to_str().map(str::to_owned))
            .collect();
        let Some(words) = words else {
            return fail(anyhow::anyhow!(
                "that sentence is not valid UTF-8. hi files are text, so it cannot be stored as written"
            ));
        };
        return match run_capture(root.as_deref(), &words[0], &words[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => fail(err),
        };
    }

    // `parse()` reads args_os internally, so it reports a bad argument rather
    // than panicking the way `env::args()` would.
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(err) => fail(err),
    }
}

/// Take an optional `--root PATH` (or `--root=PATH`) out of the arguments.
///
/// `--root` is global, and capture never reaches clap, so without this the flag
/// and its value would be joined into the criterion's sentence.
fn peel_root(args: &[std::ffi::OsString]) -> (Option<PathBuf>, Vec<std::ffi::OsString>) {
    let mut rest: Vec<std::ffi::OsString> = Vec::with_capacity(args.len());
    let mut root: Option<PathBuf> = None;
    let mut index = 0;

    // Only a LEADING `--root` is a flag. Once a non-flag argument appears the
    // rest is the person's sentence, and a word in it that happens to read like
    // a flag stays a word (hi: CAPTURE-8).
    while index < args.len() {
        match args[index].to_str() {
            Some("--root") if index + 1 < args.len() => {
                root = Some(PathBuf::from(&args[index + 1]));
                index += 2;
                continue;
            }
            Some(flag) if flag.starts_with("--root=") => {
                root = Some(PathBuf::from(&flag["--root=".len()..]));
                index += 1;
                continue;
            }
            _ => break,
        }
    }
    rest.extend_from_slice(&args[index..]);

    (root, rest)
}

fn fail(err: anyhow::Error) -> ExitCode {
    eprintln!("error: {err:#}");
    ExitCode::from(1)
}

fn run_capture(root: Option<&std::path::Path>, raw_id: &str, rest: &[String]) -> Result<()> {
    let sentence = rest.join(" ");
    let start = match root {
        Some(root) => root.to_path_buf(),
        None => std::env::current_dir()?,
    };
    // Held across the whole read-modify-write, not just the write: two
    // captures that both load the same original will otherwise each write
    // their own version over the other (hi: FILE-19).
    let mut workspace = Workspace::find(&start)?;
    let _writing = lock::acquire(&workspace.root.join("hi"))?;
    // Reload under the lock, in case another writer finished between the find
    // above and the lock being granted.
    workspace = Workspace::find(&start)?;
    let done = capture::capture(&mut workspace, raw_id, &sentence)?;

    if let Some(intent) = &done.started_intent {
        println!("{intent}  created, for the product-level why");
    }
    for file in &done.started_agent {
        println!("{file}  created, so an agent finds this without being told");
    }
    if done.created_file {
        println!("{}  created", done.file);
    }
    println!("{}  +{}", done.file, done.id);
    // The criterion is stored and this command succeeded. Said on stderr, so
    // stdout stays the record of what landed and nothing reads this as the
    // capture having failed (hi: INDEX-4.a).
    if let Some(why) = &done.index_error {
        eprintln!("note: the feature list in INTENT.md was not refreshed: {why}");
    }
    Ok(())
}

fn run(cli: Cli) -> Result<ExitCode> {
    let start = match &cli.root {
        Some(root) => root.clone(),
        None => std::env::current_dir()?,
    };
    let workspace = Workspace::find(&start)?;

    match cli.command {
        Command::Check { json } => {
            let report = check::run(&workspace)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_report(&report);
            }
            Ok(if report.ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Command::Ls { family, retired } => {
            out::ls(&workspace, family.as_deref(), retired);
            Ok(ExitCode::SUCCESS)
        }
        Command::Issue { id, create, repo } => {
            out::issue(&workspace, &id, create, repo.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Export { scope } => {
            println!("{}", out::export(&workspace, scope.as_deref())?);
            Ok(ExitCode::SUCCESS)
        }
        Command::Retire { id, reason } => {
            let _writing = lock::acquire(&workspace.root.join("hi"))?;
            // Read the files again, under the lock. The copy above was loaded
            // before the lock was granted, so another writer may have finished
            // in between; retiring from that snapshot writes the file back the
            // way it was and brings whatever they retired back to life, while
            // both commands print "retired" and exit 0. Capture reloads for the
            // same reason (hi: RETIRE-7, FILE-19, DECISIONS.md §33).
            let mut workspace = Workspace::find(&start)?;
            let parsed =
                id::Id::parse(&id).map_err(|e| anyhow::anyhow!("'{id}' is not a valid id: {e}"))?;
            let Some((index, found)) = workspace.find_id(&parsed) else {
                bail!("{parsed} does not exist");
            };
            // Already retired: this is someone coming back to say why, which
            // is the normal shape of changing your mind.
            if found.section == doc::Section::Retired {
                let Some(reason) = reason.as_deref() else {
                    bail!(
                        "{parsed} is already retired.\n\
                         hint:  say why with `hi retire {parsed} \"...\"`"
                    )
                };
                let doc = &mut workspace.docs[index];
                doc.set_retired_reason(&parsed, reason)?;
                doc.save()?;
                let file = workspace.rel(&workspace.docs[index].path);
                println!("{file}  {parsed} now says why");
                return Ok(ExitCode::SUCCESS);
            }
            let doc = &mut workspace.docs[index];
            let taken = doc.retire(&parsed, reason.as_deref())?;
            doc.save()?;
            let file = workspace.rel(&workspace.docs[index].path);
            println!("{file}  {parsed} retired");
            if !taken.is_empty() {
                // Named, not counted: a case may belong to a different concern
                // than its parent, and you should see what went with it.
                println!("        its cases went too: {}", taken.join(", "));
            }
            // Retiring changes the live count too, so the list at the front of
            // the product is no longer true either. Inside the lock above, and
            // best effort for the same reason capture's is: what was retired is
            // already on disk (hi: INDEX-4, INDEX-4.a).
            if let Some(why) = out::refresh_index(&workspace) {
                eprintln!("note: the feature list in INTENT.md was not refreshed: {why}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Index => {
            let path = run_index(&start, &workspace)?;
            println!("{path}  index updated");
            Ok(ExitCode::SUCCESS)
        }
        // No lock, decided rather than overlooked (hi: INDEX-5).
        //
        // `hi index` takes one because it is a read-modify-write: it reads
        // INTENT.md, replaces the block inside it, and writes the rest back.
        // `hi view` is not. It never reads the page it is about to write; it
        // regenerates the whole file from the criteria, so there is no window
        // in which it could put back a stale version of somebody else's work.
        // `write_atomically` already rules out a torn file.
        //
        // Three further reasons not to take one anyway. The lock is a *write*
        // lock on `hi/` and `lock::acquire` creates that directory to live in,
        // so a read verb would start writing into the repository. A page is
        // derived and gitignored, so the worst a race can do is publish a page
        // one criterion out of date, which the next run fixes and which no id
        // depends on. And `hi view` in CI would queue behind a bulk capture
        // for no gain.
        Command::View { out } => {
            let path = view::write(&workspace, out.as_deref())?;
            println!("{path}  written");
            Ok(ExitCode::SUCCESS)
        }
        Command::Seed => {
            let _writing = lock::acquire(&workspace.root.join("hi"))?;
            let workspace = Workspace::find(&start)?;
            match capture::seed_agent_files(&workspace)? {
                capture::Seeded::Created(files) => {
                    for file in files {
                        println!("{file}  created, so an agent finds this without being told");
                    }
                }
                capture::Seeded::Updated(file) => {
                    println!("{file}  updated to the current instruction");
                }
                capture::Seeded::Current => {
                    println!("hi/AGENTS.md  already current");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Regenerate the feature list in `INTENT.md`, holding the write lock.
///
/// It is a read-modify-write — read the file, splice the generated block into
/// it, write the rest back — and it was the one left outside the lock that
/// capture and retire take. Unlocked, a capture that finishes between the read
/// and the write is undone: `hi index` puts back the INTENT.md it loaded,
/// without that capture's criterion in the list and without whatever prose the
/// person saved in between, and both commands print success (hi: INDEX-5,
/// FILE-19, DECISIONS.md §34).
///
/// The workspace is loaded again under the lock, for the reason `capture` and
/// `retire` reload: the copy the caller has was read before the lock was
/// granted, so the list built from it can already be out of date by the time
/// hi is allowed to write it (hi: RETIRE-7).
fn run_index(start: &std::path::Path, workspace: &Workspace) -> Result<String> {
    let _writing = lock::acquire(&workspace.root.join("hi"))?;
    let workspace = Workspace::find(start)?;
    out::write_index(&workspace, out::Absent::Install)
}

fn print_report(report: &check::Report) {
    let mut current = String::new();
    for problem in &report.problems {
        if problem.file != current {
            println!("{}", problem.file);
            current = problem.file.clone();
        }
        println!(
            "  {}:{}  {}",
            problem.line,
            problem.kind.code(),
            problem.message
        );
    }

    if !report.problems.is_empty() {
        println!();
    }

    let files = plural(report.files, "file", "files");
    let criteria = plural(report.criteria, "criterion", "criteria");
    let families = plural(report.families.len(), "family", "families");
    print!("{criteria} · {families} · {files}");
    if report.retired > 0 {
        print!(" · {} retired", report.retired);
    }
    println!();

    if !report.problems.is_empty() {
        println!("{}", plural(report.problems.len(), "problem", "problems"));
    }
    // One line each, rather than one string with the indentation of the second
    // line and the third baked into it. The code stays out of the terminal:
    // a problem prints its kind because a person navigates by it, and a note
    // is a sentence that already says what to do. The code is for the reader
    // that is not a person, and `--json` is where that reader looks
    // (hi: CHECK-6).
    for note in &report.notes {
        println!("note: {}", note.message);
    }
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    /// `hi index` was the one read-modify-write outside the lock, and it is the
    /// only path that can install a block, so it writes the whole of somebody's
    /// `INTENT.md` back from a snapshot it took before it was allowed to.
    ///
    /// The assertion is not "it eventually agrees"; two writers racing agree
    /// often enough that a test on the result would pass while the bug was
    /// live. It is that while another writer holds the lock, `hi index` has not
    /// written anything at all. Without the lock it finishes in under a
    /// millisecond, so the wait below is three hundred times the time it needs
    /// (hi: INDEX-5, FILE-19).
    #[test]
    fn index_will_not_write_while_another_writer_holds_the_lock() {
        let root = std::env::temp_dir().join(format!("hi-main-index-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("hi")).unwrap();
        fs::write(
            root.join("hi/chat.md"),
            "---\nhi: 1\nfamilies: [SEND]\n---\n\n## Criteria\n\n- **SEND-1**  One.\n",
        )
        .unwrap();
        let intent = root.join("INTENT.md");
        let stale = "# P\n\nMine.\n\n## Features\n\n<!-- hi:index -->\n- nothing captured yet\n<!-- /hi:index -->\n";
        fs::write(&intent, stale).unwrap();

        let workspace = Workspace::find(&root).unwrap();
        let held = lock::acquire(&workspace.root.join("hi")).unwrap();

        let waiting = std::thread::spawn({
            let root = root.clone();
            move || {
                let workspace = Workspace::find(&root)?;
                run_index(&root, &workspace)
            }
        });

        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            fs::read_to_string(&intent).unwrap(),
            stale,
            "hi index rewrote INTENT.md beside a writer that was holding the lock"
        );

        drop(held);
        waiting.join().unwrap().expect("the index write");
        assert_ne!(
            fs::read_to_string(&intent).unwrap(),
            stale,
            "and once the lock is free it does the work it waited for"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
