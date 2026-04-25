//! `benten-dev` CLI binary.
//!
//! Subcommands:
//! - `inspect-state <path>` — read DAG-CBOR `ExecutionStateEnvelope` bytes
//!   from the given file and pretty-print the suspended state.
//!
//! The watcher / hot-reload loop is a Phase-2b integration concern — the
//! CLI today exposes only the diagnostic surface so the dev-server crate
//! is reachable from the command line for the inspect-state DX item the
//! plan calls out as a Phase-2a deliverable.

use std::process::ExitCode;

/// Hand-rolled usage banner. Kept inline (no `clap` / `argh` dep) — the CLI
/// surface is a single subcommand plus the standard `--help` / `--version`
/// trio, which doesn't justify pulling in an argparse crate (and the
/// transitive supply-chain audit footprint that comes with one).
const USAGE: &str = "\
usage: benten-dev <subcommand> [args]

subcommands:
  inspect-state <path>   pretty-print a suspended ExecutionStateEnvelope

options:
  -h, --help             print this help text to stdout and exit 0
  -V, --version          print version to stdout and exit 0
";

#[allow(clippy::print_stdout, clippy::print_stderr)]
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(subcmd) = args.next() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };

    // Top-level flags take precedence over subcommand dispatch. `--help` /
    // `-h` print the same usage banner the bare-invocation path emits, but
    // to STDOUT (so `benten-dev --help | grep ...` works) and exit 0 (the
    // operator asked for help, that's not a usage error). `--version` /
    // `-V` print the binary name + Cargo-supplied package version.
    match subcmd.as_str() {
        "--help" | "-h" => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        "--version" | "-V" => {
            println!("benten-dev {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        _ => {}
    }

    match subcmd.as_str() {
        "inspect-state" => {
            let Some(path) = args.next() else {
                eprintln!("usage: benten-dev inspect-state <path>");
                return ExitCode::from(2);
            };
            match std::fs::read(&path) {
                Ok(bytes) => match benten_dev::pretty_print_envelope_bytes(&bytes) {
                    Ok(rendered) => {
                        println!("{rendered}");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("benten-dev inspect-state: {e:?}");
                        ExitCode::from(1)
                    }
                },
                Err(e) => {
                    eprintln!("benten-dev inspect-state: read {path}: {e}");
                    ExitCode::from(1)
                }
            }
        }
        other => {
            eprintln!("benten-dev: unknown subcommand {other:?}");
            ExitCode::from(2)
        }
    }
}
