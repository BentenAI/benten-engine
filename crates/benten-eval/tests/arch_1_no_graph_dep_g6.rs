//! R3-A red-phase: arch-1 dep-break re-verification post-G6 (G6-A).
//!
//! Pin source: arch-pre-r1-2 + arch-1 carry. After G6 lands SUBSCRIBE +
//! STREAM in `benten-eval`, the crate MUST NOT have regained a `benten-graph`
//! dep — change-stream observation routes through `PrimitiveHost` or the
//! new `benten-core::ChangeStream` port (D23).
//!
//! Mirrors the existing `arch_1_no_graph_dep.rs` enforcement; this is the
//! Phase-2a non-regression carry pinned to G6 specifically per R2 §1.9.
//!
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

fn cargo_toml_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

#[test]
fn arch_1_benten_eval_no_graph_dep_post_g6() {
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
        "arch-1 violated post-G6: benten-eval/Cargo.toml regained a benten-graph dep. \
         SUBSCRIBE change-stream observation MUST route through PrimitiveHost or the \
         benten-core::ChangeStream port (D23). Offending lines: {offenders:?}"
    );
}
