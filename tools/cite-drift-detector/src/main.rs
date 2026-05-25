//! `cite-drift-detector` CLI binary.
//!
//! Usage:
//!
//! ```text
//! cite-drift-detector <root-dir> \
//!   [--numeric-claims | --cite-only | --read-view-with | --glob-cites |
//!    --check-prs | --all] \
//!   [--markdown | --json]
//! ```
//!
//! Flags:
//!
//!   - `--cite-only` — run only the file/symbol/line cite pass.
//!   - `--numeric-claims` — run only the numeric-claim drift pass.
//!   - `--read-view-with` — run only the read_view_with literal pass.
//!   - `--glob-cites` — run only the glob-cite phantom-match pass
//!     (R6-R2-FP-C extension; closes L11 + L16 phantom-glob class).
//!   - `--check-prs` — run the PR-cite verification pass (off by default
//!     since it requires `gh` + network; R6-R2-FP-C extension; closes
//!     L18 phantom PR-cite class).
//!   - `--all` — run cite + numeric + read-view-with + glob passes
//!     (PR-cite stays off unless `--check-prs` is also passed).
//!   - `--markdown` — emit findings as a markdown report (the form the CI
//!     workflow posts as a PR comment in non-blocking mode).
//!   - `--json` — emit findings as canonical-schema JSON per §3.6i
//!     (R6-R2-FP-C extension; supports the orchestrator's JSON-disposition
//!     pipeline). Mutually exclusive with `--markdown`.
//!
//! Exit code: `0` clean, `1` findings emitted, `2` argument error.

#![forbid(unsafe_code)]
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::PathBuf;
use std::process::ExitCode;

use cite_drift_detector::{
    Finding, render_json_report, render_markdown_report, run_cite_drift_check, run_glob_cite_check,
    run_numeric_claim_check, run_pr_cite_check, run_read_view_with_lint,
};

#[allow(clippy::too_many_lines)]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "usage: cite-drift-detector <root-dir> \
             [--cite-only | --numeric-claims | --read-view-with | --glob-cites | \
              --check-prs | --all] [--markdown | --json]"
        );
        return ExitCode::from(2);
    }
    let root = PathBuf::from(&args[0]);
    if !root.is_dir() {
        eprintln!("error: <root-dir> `{}` is not a directory", root.display());
        return ExitCode::from(2);
    }
    let mut do_cite = true;
    let mut do_numeric = true;
    let mut do_read_view_with = true;
    let mut do_glob = true;
    let mut do_pr = false;
    let mut markdown = false;
    let mut json = false;
    for a in &args[1..] {
        match a.as_str() {
            "--numeric-claims" => {
                do_cite = false;
                do_numeric = true;
                do_read_view_with = false;
                do_glob = false;
            }
            "--cite-only" => {
                do_cite = true;
                do_numeric = false;
                do_read_view_with = false;
                do_glob = false;
            }
            "--read-view-with" => {
                do_cite = false;
                do_numeric = false;
                do_read_view_with = true;
                do_glob = false;
            }
            "--glob-cites" => {
                do_cite = false;
                do_numeric = false;
                do_read_view_with = false;
                do_glob = true;
            }
            "--check-prs" => {
                // Additive — runs alongside whatever else is selected.
                do_pr = true;
            }
            "--all" => {
                do_cite = true;
                do_numeric = true;
                do_read_view_with = true;
                do_glob = true;
                // --all does NOT enable --check-prs; that flag stays
                // opt-in per the design (network + gh dep).
            }
            "--markdown" => {
                markdown = true;
            }
            "--json" => {
                json = true;
            }
            other => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
        }
    }
    if markdown && json {
        eprintln!("error: --markdown and --json are mutually exclusive");
        return ExitCode::from(2);
    }
    let mut findings: Vec<Finding> = Vec::new();
    if do_cite {
        findings.extend(run_cite_drift_check(&root));
    }
    if do_numeric {
        findings.extend(run_numeric_claim_check(&root));
    }
    if do_read_view_with {
        findings.extend(run_read_view_with_lint(&root));
    }
    if do_glob {
        findings.extend(run_glob_cite_check(&root));
    }
    if do_pr {
        findings.extend(run_pr_cite_check(&root));
    }
    findings.sort();
    findings.dedup();

    if json {
        print!("{}", render_json_report(&findings));
    } else if markdown {
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
