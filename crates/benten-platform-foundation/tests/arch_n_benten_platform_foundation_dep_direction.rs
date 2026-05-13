//! arch-N pin: `benten-platform-foundation` dep direction regression-guard.
//!
//! ## Pin source
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.18 (cross-cutting
//!   12-primitive irreducibility + dep-direction defense).
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 wave charter
//!   (closes r4-arch-2 — new-crate dep-direction enforcement orphaned
//!   from R3 family enumeration).
//! - CLAUDE.md baked-in arch-1 + arch-r1-15 (engine-internal crates
//!   `benten-eval` + `benten-graph` MUST NOT be reverse-depended upon by
//!   sibling crates).
//! - `crates/benten-platform-foundation/Cargo.toml` package description:
//!   "MUST NOT depend on benten-eval or benten-graph (preserves arch-1
//!   / arch-r1-15)."
//!
//! ## What this pin asserts
//!
//! `benten-platform-foundation` is the 11th workspace crate landing in
//! Phase-4-Foundation per D-4F-2 ratification. It depends on
//! `benten-core` (Subgraph types) + `benten-errors` (ErrorCode) +
//! `benten-id` (Did for plugin-DID + peer-DID per CLAUDE.md #18). It
//! MUST NOT take a runtime dependency on `benten-eval` or `benten-graph`
//! (engine internals; that would invert the dependency graph and
//! couple plugin/schema/materializer surfaces to evaluator + storage
//! internals).
//!
//! ## State at HEAD
//!
//! At HEAD the crate's `[dependencies]` table is already clean. This
//! test is PASSING at HEAD as a permanent regression-guard against
//! future Phase-4-Foundation waves accidentally re-introducing the
//! forbidden direction.
//!
//! ## §3.6f SHAPE-not-SUBSTANCE
//!
//! SHAPE: parse `Cargo.toml`. SUBSTANCE: assert specific dep lines are
//! absent + assert benten-core IS present (regression-guard against the
//! file being emptied silently).

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn cargo_toml_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

/// Extract the line set inside the `[dependencies]` + `[build-dependencies]`
/// tables only. `[dev-dependencies]` is permitted to depend on
/// `benten-engine` (for round-trip testing) per the crate's Cargo.toml
/// `[dev-dependencies]` block; the production dep direction is what
/// arch-1 / arch-r1-15 constrain.
fn production_dep_lines(toml: &str) -> Vec<&str> {
    let mut in_dep_table = false;
    let mut out = Vec::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_dep_table = matches!(trimmed, "[dependencies]" | "[build-dependencies]");
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
fn benten_platform_foundation_does_not_depend_on_benten_eval() {
    let toml = cargo_toml_text();
    for line in production_dep_lines(&toml) {
        assert!(
            !line.contains("benten-eval"),
            "benten-platform-foundation MUST NOT depend on benten-eval (preserves arch-1 / \
             arch-r1-15 + CLAUDE.md #18/#19 — plugins are subgraphs not direct evaluator \
             consumers); offending: {line}"
        );
    }
}

#[test]
fn benten_platform_foundation_does_not_depend_on_benten_graph() {
    let toml = cargo_toml_text();
    for line in production_dep_lines(&toml) {
        assert!(
            !line.contains("benten-graph"),
            "benten-platform-foundation MUST NOT depend on benten-graph (preserves arch-1 — \
             storage internals are below the platform-foundation surface); offending: {line}"
        );
    }
}

#[test]
fn benten_platform_foundation_does_not_depend_on_benten_engine_in_production() {
    // `benten-engine` is permitted as a dev-dependency for round-trip
    // testing but NOT as a production runtime dependency — the engine
    // facade depends on platform-foundation, not the other way around
    // (per `00-implementation-plan.md` G23-0 + arch-r1-1 dep direction).
    let toml = cargo_toml_text();
    for line in production_dep_lines(&toml) {
        assert!(
            !line.contains("benten-engine"),
            "benten-platform-foundation MUST NOT depend on benten-engine in production \
             (dev-dependency for round-trip tests is permitted); offending: {line}"
        );
    }
}

#[test]
fn benten_platform_foundation_depends_on_benten_core_for_subgraph_types() {
    let toml = cargo_toml_text();
    let has_core = production_dep_lines(&toml)
        .iter()
        .any(|l| l.contains("benten-core"));
    assert!(
        has_core,
        "benten-platform-foundation MUST depend on benten-core for Subgraph / SubgraphSpec \
         types (per D-4F-2 + arch-r1-1)"
    );
}

#[test]
fn benten_platform_foundation_depends_on_benten_id_for_did_types() {
    let toml = cargo_toml_text();
    let has_id = production_dep_lines(&toml)
        .iter()
        .any(|l| l.contains("benten-id"));
    assert!(
        has_id,
        "benten-platform-foundation MUST depend on benten-id for Did types (per CLAUDE.md \
         #18 plugin-DID + peer-DID model)"
    );
}

#[test]
fn benten_platform_foundation_src_does_not_import_benten_eval_or_graph() {
    // SUBSTANCE check (pim-18 §3.6f): even if Cargo.toml is clean, an
    // accidental `extern crate benten_eval;` style use in src/ would
    // surface a different way. Walk src/ for forbidden `use benten_eval::`
    // / `use benten_graph::` lines.
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    if !src.is_dir() {
        return; // RED-PHASE — crate skeleton may not yet have src/.
    }
    let mut offenders: Vec<String> = Vec::new();
    walk_rs_files(&src, &mut |path, body| {
        for (lineno, line) in body.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("use benten_eval::")
                || t.starts_with("use benten_graph::")
                || t.starts_with("extern crate benten_eval")
                || t.starts_with("extern crate benten_graph")
            {
                offenders.push(format!("{}:{} {}", path.display(), lineno + 1, line.trim()));
            }
        }
    });
    assert!(
        offenders.is_empty(),
        "benten-platform-foundation src/ MUST NOT `use benten_eval::*` / `use benten_graph::*` \
         (preserves arch-1 / arch-r1-15); offenders: {:#?}",
        offenders,
    );
}

fn walk_rs_files(dir: &std::path::Path, visit: &mut dyn FnMut(&std::path::Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_rs_files(&p, visit);
        } else if p.extension().is_some_and(|e| e == "rs")
            && let Ok(body) = std::fs::read_to_string(&p)
        {
            visit(&p, &body);
        }
    }
}
