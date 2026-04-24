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

#[allow(clippy::print_stdout, clippy::print_stderr)]
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(subcmd) = args.next() else {
        eprintln!("usage: benten-dev <subcommand> [args]");
        eprintln!();
        eprintln!("subcommands:");
        eprintln!("  inspect-state <path>   pretty-print a suspended ExecutionStateEnvelope");
        return ExitCode::from(2);
    };

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
