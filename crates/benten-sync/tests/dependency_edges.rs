//! G16-A LANDED pin for `benten-sync` dependency-edge architectural
//! constraint per arch-r1-11 + D-PHASE-3-14.
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-A row
//!   `benten_sync_no_dependency_on_benten_engine_or_eval`.
//! - `arch-r1-11` (architectural constraint: dependency direction
//!   is engine → sync, never the reverse).
//! - `D-PHASE-3-14` (per-NEW-crate dep-edge audit).
//! - plan §3 G16-A row.
//!
//! ## Architectural constraint
//!
//! `benten-sync` is the new 10th workspace crate landing at G16-A.
//! To keep the dependency graph layered, `benten-sync` MUST NOT
//! depend on:
//!
//! - `benten-engine` (orchestrator — `benten-sync` is consumed BY
//!   the engine via `engine.atrium.*` surface, not the reverse).
//! - `benten-eval` (evaluator — `benten-sync` is consumed BY the
//!   evaluator's primitive arms, not the reverse).
//!
//! ## Implementation note
//!
//! The pin walks Cargo.toml's dep-tables PROGRAMMATICALLY (toml
//! parse) rather than raw-grepping the manifest text. Otherwise the
//! prose comment in `[lib]` that NAMES `benten-engine`/`benten-eval`
//! (to document the forbidden-list) would false-positive.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

#[test]
fn benten_sync_no_dependency_on_benten_engine_or_eval() {
    let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let raw = std::fs::read_to_string(&manifest_path).expect("read Cargo.toml");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse Cargo.toml");

    // Walk every dep-table (top-level + target.*-conditional) +
    // collect dep-key names.
    let mut deps: BTreeSet<String> = BTreeSet::new();
    collect_deps(&parsed, &mut deps);

    const FORBIDDEN: &[&str] = &["benten-engine", "benten-eval"];
    for forbidden in FORBIDDEN {
        assert!(
            !deps.contains(*forbidden),
            "benten-sync MUST NOT depend on {forbidden} per arch-r1-11 + D-PHASE-3-14 \
             (dependency direction is engine → sync, never reverse). \
             Found `{forbidden}` in dep tables of {manifest_path:?}.",
        );
    }
}

/// Recursively collect dep-key names from a TOML value, walking the
/// canonical `[dependencies]` / `[dev-dependencies]` /
/// `[build-dependencies]` / `[target.*.dependencies]` etc. tables.
fn collect_deps(value: &toml::Value, deps: &mut BTreeSet<String>) {
    let toml::Value::Table(table) = value else {
        return;
    };
    for (key, sub) in table {
        match key.as_str() {
            "dependencies" | "dev-dependencies" | "build-dependencies" => {
                if let toml::Value::Table(t) = sub {
                    for k in t.keys() {
                        deps.insert(k.clone());
                    }
                }
            }
            "target" => {
                // `target.<cfg>.dependencies` → recurse.
                if let toml::Value::Table(targets) = sub {
                    for (_cfg, cfg_table) in targets {
                        collect_deps(cfg_table, deps);
                    }
                }
            }
            // Don't recurse into [package] / [lib] / [features] / etc.
            _ => {}
        }
    }
}
