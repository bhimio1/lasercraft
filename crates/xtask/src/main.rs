//! Project tasks for LaserCraft. `cargo xtask parity` checks the parity
//! checklist (`docs/spec/parity.md`).

mod parity;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask parity [--markdown <path>] [--check <path>]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("parity") => run_parity(&args[1..]),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root: two levels above this crate.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn run_parity(args: &[String]) -> ExitCode {
    let (mut markdown, mut check) = (None, None);
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match (arg.as_str(), it.next()) {
            ("--markdown", Some(p)) => markdown = Some(PathBuf::from(p)),
            ("--check", Some(p)) => check = Some(PathBuf::from(p)),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
        }
    }
    let root = root();
    let checklist = match parity::Checklist::load(&root.join("docs/parity/checklist.json")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut problems = checklist.check(&root, &parity::Facts::gather(&root));
    problems.extend(checklist.coverage(&root));
    for p in &problems {
        eprintln!("problem: {p}");
    }
    println!("{}", checklist.summary());
    let report = checklist.markdown();
    if let Some(path) = markdown
        && let Err(e) = std::fs::write(&path, &report)
    {
        eprintln!("error: cannot write {}: {e}", path.display());
        return ExitCode::FAILURE;
    }
    if let Some(path) = check {
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        if current != report {
            eprintln!(
                "problem: {} is out of date; run `cargo xtask parity --markdown {}`",
                path.display(),
                path.display()
            );
            problems.push(String::new());
        }
    }
    if problems.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
