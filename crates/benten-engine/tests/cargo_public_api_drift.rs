//! Phase 2b R4-FP B-4 — `cargo-public-api` drift detector workflow stub.
//!
//! TDD red-phase. Pin source: plan §3.1 Phase-2b CI additions
//! (`cargo-public-api` baselines on first PR; the manifest produced
//! becomes the OSS-launch public-surface contract — wire on PR
//! alongside G6 first push so STREAM/SUBSCRIBE surface gets baselined
//! immediately) + R2 §6 (cargo-public-api.yml workflow).
//!
//! Workflow-level test: asserts the `.github/workflows/cargo-public-api.yml`
//! file exists and references the `cargo-public-api` cargo extension.
//! The actual drift detection is performed by the workflow at PR-time
//! (it diffs the public surface against a committed baseline at
//! `docs/public-api/<crate>.txt` or similar); this Rust-side test
//! verifies workflow wiring presence + non-vacuity, mirroring the
//! Phase-2a `cargo_vet_policy_self_test.rs` pattern.
//!
//! Owned by R3-E (CI workflow tests row); test landed by R4-FP B-4.
//!
//! **B-3 ownership note:** B-3 owns workflow YAML authoring; this test
//! REFERENCES the workflow file but does NOT create it. Until B-3
//! lands the workflow, this test fails at the file-presence assert,
//! exactly the red-phase shape we want.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `cargo_public_api_workflow_present_and_non_vacuous` — plan §3.1 +
/// R2 §6.
#[test]
#[ignore = "Phase 2b G6 entry pending — cargo-public-api.yml workflow + baseline unimplemented (B-3 owns YAML)"]
fn cargo_public_api_workflow_present_and_non_vacuous() {
    let root = workspace_root();
    let workflow = root.join(".github/workflows/cargo-public-api.yml");

    let yaml = std::fs::read_to_string(&workflow).unwrap_or_else(|e| {
        panic!(
            ".github/workflows/cargo-public-api.yml MUST exist after G6 \
             entry lands per plan §3.1 ({}); error: {}. B-3 owns workflow \
             authoring.",
            workflow.display(),
            e
        );
    });

    // Non-vacuity: the workflow must reference cargo-public-api the tool
    // (otherwise it's a `name: cargo-public-api` shell with no actual
    // surface check — same anti-pattern Phase-2a sec-r6r3-01 flagged).
    assert!(
        yaml.contains("cargo-public-api") || yaml.contains("cargo public-api"),
        ".github/workflows/cargo-public-api.yml MUST invoke the \
         `cargo-public-api` cargo extension (otherwise the workflow is \
         vacuous — Phase-2a sec-r6r3-01 anti-pattern). Workflow body:\n{}",
        yaml
    );

    // Should run on PR (per plan §3.1: "wire on PR alongside G6 first
    // push so STREAM/SUBSCRIBE surface gets baselined immediately").
    assert!(
        yaml.contains("pull_request"),
        ".github/workflows/cargo-public-api.yml MUST fire on \
         `pull_request` (plan §3.1 — PR-time drift detection); only \
         on-push or on-schedule misses the per-PR contract."
    );
}

/// Pins that a public-surface baseline lives somewhere checked-in so
/// the workflow's diff has a stable base. G6 close authors the
/// baseline; until then this assert fails to make the dependency
/// explicit.
#[test]
#[ignore = "Phase 2b G6 close pending — public-api baseline file unimplemented"]
fn cargo_public_api_baseline_committed() {
    let root = workspace_root();
    // Allow either docs/public-api/ subdir OR per-crate
    // public-api.txt baseline; G6 close picks layout.
    let candidates = [
        root.join("docs/public-api"),
        root.join("crates/benten-engine/public-api.txt"),
        root.join("crates/benten-eval/public-api.txt"),
        root.join("crates/benten-core/public-api.txt"),
        root.join("crates/benten-graph/public-api.txt"),
    ];

    let any_present = candidates.iter().any(|p| p.exists());
    assert!(
        any_present,
        "cargo-public-api baseline MUST be committed somewhere (one of: \
         {:?}) so the workflow's diff has a stable base. G6 close \
         authors the baseline per plan §3.1.",
        candidates
    );
}
