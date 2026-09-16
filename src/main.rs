//! hi (Human Intent) keeps acceptance criteria in human words, with permanent ids.
//!
//! hi holds intent and identity. It stores no state, tracks no lifecycle, binds
//! no evidence, and never fails a build because a criterion is unproven. See
//! DECISIONS.md for why each of those is deliberate.

mod capture;
mod check;
mod doc;
mod id;
mod out;
mod view;
mod workspace;

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
        /// A family, a file stem, or nothing for the whole repository
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
    let mut workspace = Workspace::find(&start)?;
    let done = capture::capture(&mut workspace, raw_id, &sentence)?;

    if let Some(intent) = &done.started_intent {
        println!("{intent}  created, for the product-level why");
    }
    if done.created_file {
        println!("{}  created", done.file);
    }
    println!("{}  +{}", done.file, done.id);
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
            let report = check::run(&workspace);
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
            let mut workspace = workspace;
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
            let moved = doc.retire(&parsed, reason.as_deref())?;
            doc.save()?;
            let file = workspace.rel(&workspace.docs[index].path);
            match moved {
                1 => println!("{file}  {parsed} retired"),
                n => println!("{file}  {parsed} retired, with {} of its cases", n - 1),
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Index => {
            let path = out::write_index(&workspace)?;
            println!("{path}  index updated");
            Ok(ExitCode::SUCCESS)
        }
        Command::View { out } => {
            let path = view::write(&workspace, out.as_deref())?;
            println!("{path}  written");
            Ok(ExitCode::SUCCESS)
        }
    }
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
    if let Some(note) = &report.note {
        println!("note: {note}");
    }
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}
