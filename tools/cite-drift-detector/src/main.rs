//! `cite-drift-detector` CLI binary.
//!
//! Usage:
//!
//! ```text
//! cite-drift-detector <root-dir> [--numeric-claims] [--all] [--markdown]
//! ```
//!
//! Flags:
//!
//!   - `--numeric-claims` — run only the numeric-claim drift pass.
//!   - `--all` — run BOTH cite-drift + numeric-claim passes (default).
//!   - `--markdown` — emit findings as a markdown report (the form the CI
//!     workflow posts as a PR comment in non-blocking mode).
//!
//! Exit code: `0` clean, `1` findings emitted, `2` argument error.

#![forbid(unsafe_code)]
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::PathBuf;
use std::process::ExitCode;

use cite_drift_detector::{
    Finding, render_markdown_report, run_cite_drift_check, run_numeric_claim_check,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: cite-drift-detector <root-dir> [--numeric-claims | --all] [--markdown]");
        return ExitCode::from(2);
    }
    let root = PathBuf::from(&args[0]);
    if !root.is_dir() {
        eprintln!("error: <root-dir> `{}` is not a directory", root.display());
        return ExitCode::from(2);
    }
    let mut do_cite = true;
    let mut do_numeric = true;
    let mut markdown = false;
    for a in &args[1..] {
        match a.as_str() {
            "--numeric-claims" => {
                do_cite = false;
                do_numeric = true;
            }
            "--cite-only" => {
                do_cite = true;
                do_numeric = false;
            }
            "--all" => {
                do_cite = true;
                do_numeric = true;
            }
            "--markdown" => {
                markdown = true;
            }
            other => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
        }
    }
    let mut findings: Vec<Finding> = Vec::new();
    if do_cite {
        findings.extend(run_cite_drift_check(&root));
    }
    if do_numeric {
        findings.extend(run_numeric_claim_check(&root));
    }
    findings.sort();
    findings.dedup();

    if markdown {
        print!("{}", render_markdown_report(&findings));
    } else if findings.is_empty() {
        println!("cite-drift: no findings");
    } else {
        for f in &findings {
            println!(
                "[{}] {}:{} — {}",
                f.kind,
                f.path.display(),
                f.line,
                f.message
            );
        }
    }
    if findings.is_empty() {
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}
