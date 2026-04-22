//! R3 unit test for phil-7 / §9.14 arch-1 dep-break: `benten-eval/Cargo.toml`
//! does NOT depend on `benten-graph`.
//!
//! Runs as part of `.github/workflows/arch-1-dep-break.yml`.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.1 + §9.14).

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

fn cargo_toml_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

#[test]
fn arch_1_no_graph_dep() {
    let path = cargo_toml_path();
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    let offenders: Vec<&str> = src
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            t.starts_with("benten-graph ")
                || t.starts_with("benten-graph=")
                || t.starts_with("[dependencies.benten-graph]")
                || t.starts_with("[dev-dependencies.benten-graph]")
                || t.starts_with("[build-dependencies.benten-graph]")
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "arch-1 dep break regressed: benten-eval must NOT depend on benten-graph \
         (phil-7, §9.14). Offending lines in {}: {:?}",
        path.display(),
        offenders
    );
}
