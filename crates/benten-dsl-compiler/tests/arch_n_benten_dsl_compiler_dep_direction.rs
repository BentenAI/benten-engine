//! arch-N pin: `benten-dsl-compiler` dep direction.
//!
//! Per `r1-architect-reviewer.json` D-point G12-B-scope + plan §3.2 G12-B +
//! `00-implementation-plan.md` line 582: `benten-dsl-compiler` sits as a
//! sibling of `benten-engine`, depends on `benten-core` for `Subgraph` /
//! `SubgraphSpec` types, and **MUST NOT depend on `benten-eval` or
//! `benten-graph`** — preserves arch-1.
//!
//! Lifted from red-phase 2026-04-28 — file scans `Cargo.toml` for forbidden
//! dep entries and asserts the allowed set.
//!
//! Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn cargo_toml_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

/// Extract the line set inside the `[dependencies]`, `[dev-dependencies]`,
/// and `[build-dependencies]` tables only — comments + descriptions in
/// `[package]` are excluded.
fn dep_lines(toml: &str) -> Vec<&str> {
    let mut in_dep_table = false;
    let mut out = Vec::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_dep_table = matches!(
                trimmed,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if !in_dep_table {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        out.push(line);
    }
    out
}

#[test]
fn benten_dsl_compiler_does_not_depend_on_benten_eval() {
    let toml = cargo_toml_text();
    for line in dep_lines(&toml) {
        assert!(
            !line.contains("benten-eval"),
            "benten-dsl-compiler must NOT depend on benten-eval (preserves arch-1); offending: {line}"
        );
    }
}

#[test]
fn benten_dsl_compiler_does_not_depend_on_benten_graph() {
    let toml = cargo_toml_text();
    for line in dep_lines(&toml) {
        assert!(
            !line.contains("benten-graph"),
            "benten-dsl-compiler must NOT depend on benten-graph; offending: {line}"
        );
    }
}

#[test]
fn benten_dsl_compiler_does_not_depend_on_benten_engine() {
    let toml = cargo_toml_text();
    for line in dep_lines(&toml) {
        assert!(
            !line.contains("benten-engine"),
            "benten-dsl-compiler must NOT depend on benten-engine — sibling, not child; offending: {line}"
        );
    }
}

#[test]
fn benten_dsl_compiler_depends_on_benten_core_for_subgraph_types() {
    let toml = cargo_toml_text();
    let has_core = dep_lines(&toml).iter().any(|l| l.contains("benten-core"));
    assert!(
        has_core,
        "benten-dsl-compiler MUST depend on benten-core for Subgraph types"
    );
}
