//! Phase-3 G17-B `bench-wat-rebake` binary entry — `cargo bench-wat-rebake`
//! alias (defined in `.cargo/config.toml`) invokes this main.
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "bench-wat-rebake is a dev-only CLI tool; eprintln! status output is the public surface (mirrors cite-drift-detector + benten-dev tooling pattern)"
)]
//!
//! Modes:
//! - default: regenerate `.wasm` siblings from every `.wat` under
//!   `crates/benten-eval/tests/fixtures/sandbox/` (recursive over the
//!   top + `escape/` first-level subdir).
//! - `--check`: report drift WITHOUT writing; non-zero exit if any
//!   `.wasm` would change. Mirrors the existing
//!   `scripts/build_wasm.sh --check` shape but uses the Rust `wat`
//!   crate directly (no `wabt` binary dep, no shell-portability issues
//!   per phase-2-backlog wsa-12).

use std::path::PathBuf;
use std::process::ExitCode;

use bench_wat_rebake::{fixture_root_from_workspace, rebake_all};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dry_run = args.iter().any(|a| a == "--check");
    let workspace_root = match resolve_workspace_root() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bench-wat-rebake: {e}");
            return ExitCode::from(2);
        }
    };
    let fixture_root = fixture_root_from_workspace(&workspace_root);
    eprintln!(
        "bench-wat-rebake: walking {} (dry_run={dry_run})",
        fixture_root.display()
    );
    match rebake_all(&fixture_root, dry_run) {
        Ok(outcomes) => {
            let drifted: Vec<_> = outcomes.iter().filter(|o| o.changed).collect();
            for o in &outcomes {
                if o.changed {
                    if dry_run {
                        eprintln!("DRIFT: {}", o.wasm_path.display());
                    } else {
                        eprintln!("rebaked: {}", o.wasm_path.display());
                    }
                } else {
                    eprintln!("ok: {}", o.wasm_path.display());
                }
            }
            if dry_run && !drifted.is_empty() {
                eprintln!(
                    "FAIL: {} of {} fixture(s) drift — rerun without --check to regenerate",
                    drifted.len(),
                    outcomes.len()
                );
                ExitCode::from(2)
            } else {
                eprintln!(
                    "OK: {} fixture(s){}",
                    outcomes.len(),
                    if dry_run {
                        " match committed bytes"
                    } else {
                        " regenerated"
                    }
                );
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("bench-wat-rebake: {e}");
            ExitCode::from(1)
        }
    }
}

/// Walk parents from the binary's manifest dir until we find a path
/// containing a top-level `Cargo.toml` with `[workspace]` (mirrors the
/// pattern used by other Phase-3 dev binaries; ~5-10 LOC).
fn resolve_workspace_root() -> Result<PathBuf, String> {
    // CARGO_MANIFEST_DIR is set by cargo at compile time to the dir
    // containing this crate's Cargo.toml. From there: parent (tools/)
    // + parent (workspace root).
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // tools/bench-wat-rebake -> tools/
    p.pop(); // tools/ -> workspace root
    if !p.join("Cargo.toml").exists() {
        return Err(format!(
            "expected workspace root Cargo.toml at {} (tools/bench-wat-rebake's manifest-dir-up-2 \
             walk); set CWD to workspace root or invoke via the `cargo bench-wat-rebake` alias",
            p.display()
        ));
    }
    Ok(p)
}
