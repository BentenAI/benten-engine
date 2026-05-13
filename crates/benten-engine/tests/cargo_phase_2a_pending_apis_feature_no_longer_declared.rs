//! Phase 4-Foundation R3 (Family A — G22-A `phase_2a_pending_apis` feature
//! lapsing). RED-PHASE grep-assert: at R5 G22-A merge time, the
//! `phase_2a_pending_apis` cargo feature MUST no longer be declared in
//! `crates/benten-engine/Cargo.toml` and MUST no longer be cited as a
//! `#[cfg(feature = "phase_2a_pending_apis")]` guard anywhere in the
//! `crates/benten-engine/tests/` tree.
//!
//! # Charter
//!
//! Per `.addl/phase-4-foundation/r2-test-landscape.md` §2.1 G22-A row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 wave-1 G22-A
//! files-owned: the feature gate has outlived its purpose — R5 G2-B /
//! G3-B / G9-A landed the wider APIs that the gate was masking; the gate
//! stayed only because downstream R6 work hadn't completed the WAIT-
//! related test bodies. Phase-4-Foundation G22-A closes the gate +
//! removes BOTH the Cargo declaration AND the 8 `#![cfg]` guards in the
//! test bodies (un-cfg-gate per R2 §2.1 G22-A rows).
//!
//! # What this pin asserts (would-FAIL-if-no-op'd per §3.6b)
//!
//! 1. `crates/benten-engine/Cargo.toml` MUST NOT contain the line
//!    `phase_2a_pending_apis = []` in its `[features]` table.
//! 2. NO file under `crates/benten-engine/tests/` may contain the
//!    string `feature = "phase_2a_pending_apis"` (the cfg-guard form).
//!
//! Removing the declaration WITHOUT removing the guards causes cargo to
//! reject the unknown-feature reference; removing the guards WITHOUT
//! removing the declaration leaves a dead feature in the public Cargo
//! manifest. Either half-landing of G22-A trips one of the two
//! assertions in this test.
//!
//! # RED-PHASE
//!
//! At write-time (R3 Family A; base SHA `f3930e1`) the Cargo declaration
//! IS present (Cargo.toml carries `phase_2a_pending_apis = []`) and 8
//! test files carry the cfg-guard. Therefore this test's body fails
//! against current HEAD — it is intentionally `#[ignore]`-marked with a
//! RED-PHASE tag so CI stays green; R5 G22-A un-ignores when the gate is
//! removed.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer. Closes at R5 G22-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

/// Workspace-relative path to the `benten-engine` crate root (where
/// `Cargo.toml` and `tests/` live).
fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G22-A (phase_2a_pending_apis feature lapsing). Un-ignore when Cargo.toml declaration removed + 8 test cfg-guards stripped."]
fn cargo_toml_does_not_declare_phase_2a_pending_apis_feature() {
    let cargo_toml = crate_root().join("Cargo.toml");
    let body = std::fs::read_to_string(&cargo_toml)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cargo_toml.display()));
    // Substantive match: pin the canonical empty-feature declaration
    // form so a future re-introduction in the [features] table trips
    // the assertion. Match the declaration line, not free-text
    // mentions in doc-comments.
    let needle = "phase_2a_pending_apis = []";
    assert!(
        !body.contains(needle),
        "expected `phase_2a_pending_apis` feature declaration to be \
         removed from {} but found `{needle}` (G22-A `phase_2a_pending_apis` \
         feature lapsing not yet landed; un-ignore + un-block this test \
         at R5 G22-A close)",
        cargo_toml.display(),
    );
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G22-A (phase_2a_pending_apis feature lapsing). Un-ignore when 8 test files have their `#![cfg(feature = \"phase_2a_pending_apis\")]` guards removed."]
fn no_test_file_carries_phase_2a_pending_apis_cfg_guard() {
    let tests_dir = crate_root().join("tests");
    let mut offenders: Vec<PathBuf> = Vec::new();
    collect_offenders(&tests_dir, &mut offenders);
    assert!(
        offenders.is_empty(),
        "expected NO `feature = \"phase_2a_pending_apis\"` cfg-guards in \
         crates/benten-engine/tests/ but found {} file(s): {:?} (G22-A \
         lapsing not yet landed; un-ignore + un-block this test at R5 \
         G22-A close)",
        offenders.len(),
        offenders,
    );
}

/// Recursive walk: collect any `.rs` file under `dir` whose contents
/// mention the `feature = "phase_2a_pending_apis"` cfg-guard form.
/// Skips this test file itself so the grep-assertion doesn't
/// self-match.
fn collect_offenders(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let self_filename = "cargo_phase_2a_pending_apis_feature_no_longer_declared.rs";
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_offenders(&path, out);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|s| s.to_str()) == Some(self_filename) {
            continue;
        }
        let Ok(body) = std::fs::read_to_string(&path) else {
            continue;
        };
        if body.contains("feature = \"phase_2a_pending_apis\"") {
            out.push(path);
        }
    }
}
