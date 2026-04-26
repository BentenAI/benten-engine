//! Phase 2b R3 (R3-E) — `cargo-vet` baseline non-vacuity self-test.
//!
//! TDD red-phase. Pin source: plan §3.1 Phase-2b CI additions
//! (cargo-vet baseline at `.addl/phase-2b/pre-r1-security-deliverables.md`
//! Section 3 per sec-pre-r1-10 §3.6) + R2 §6 ownership row
//! (`cargo_vet_workflow_non_vacuity_self_test_passes` lives in
//! `crates/benten-engine/tests/ci/cargo_vet_non_vacuity.rs`).
//!
//! Mirrors the Phase-2a R6FP-R3 anti-pattern fix
//! (`supply-chain-seeded-test.yml`): a workflow that "runs cleanly
//! against the current tree but never tested whether the harness
//! actually fires on a regression" is vacuous. The seeded-vuln test
//! proves cargo-deny + cargo-audit both fire on a known-bad
//! dependency. This Rust-side companion proves cargo-vet's policy
//! shape itself constrains SOMETHING — the policy file MUST contain
//! at least one non-trivial rule, and the import set MUST reference
//! at least one upstream vouch source (e.g. mozilla/google audits).
//!
//! Anti-pattern that this self-test prevents: the cargo-vet workflow
//! ships, runs `cargo vet check` cleanly, and goes green — but the
//! `supply-chain/audits.toml` is empty + `imports.toml` references no
//! upstream sources, so the workflow is verifying nothing.
//!
//! **Status:** RED-PHASE (Phase 2b G7 entry pending). The
//! `supply-chain/` directory + `audits.toml` + `imports.toml` do not
//! yet exist; cargo-vet baseline lands with G7 entry per plan §3.1.
//!
//! Owned by R3-E (CI/test ownership row in R2 §10).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `cargo_vet_workflow_non_vacuity_self_test_passes` — R2 §6 (named
/// row) + sec-pre-r1-10 §3.6.
///
/// Asserts the cargo-vet baseline is structurally non-vacuous:
///   1. `supply-chain/audits.toml` exists and is non-empty.
///   2. `supply-chain/config.toml` exists and references at least one
///      upstream vouch source (mozilla, google, bytecode-alliance, or
///      similar) so the workflow's "trust" set is bootstrapped against
///      a known curator.
///   3. The vetting policy is set to a non-trivial level
///      (`safe-to-deploy` or stricter).
///
/// If any of these fail, the cargo-vet workflow runs but constrains
/// nothing (Phase-2a sec-r6r3-01 anti-pattern).
#[test]
#[ignore = "Phase 2b G7 entry pending — supply-chain/ + cargo-vet baseline unimplemented"]
fn cargo_vet_workflow_non_vacuity_self_test_passes() {
    let root = workspace_root();
    let supply_chain_dir = root.join("supply-chain");
    assert!(
        supply_chain_dir.is_dir(),
        "supply-chain/ directory MUST exist after G7 entry lands the \
         cargo-vet baseline (plan §3.1 + sec-pre-r1-10 §3.6); not found at {}",
        supply_chain_dir.display()
    );

    // (1) audits.toml present + non-empty.
    let audits = supply_chain_dir.join("audits.toml");
    let audits_bytes = std::fs::read(&audits).unwrap_or_else(|e| {
        panic!(
            "supply-chain/audits.toml MUST exist after baseline ({}); error: {}",
            audits.display(),
            e
        );
    });
    assert!(
        audits_bytes.len() > 16,
        "supply-chain/audits.toml MUST be non-empty (cargo-vet baseline; \
         empty file = vacuous workflow per Phase-2a sec-r6r3-01)"
    );

    // (2) config.toml references at least one upstream vouch source.
    let config = supply_chain_dir.join("config.toml");
    let config_str = std::fs::read_to_string(&config).unwrap_or_else(|e| {
        panic!(
            "supply-chain/config.toml MUST exist after baseline ({}); error: {}",
            config.display(),
            e
        );
    });
    let known_upstreams =
        ["mozilla", "google", "bytecode-alliance", "rust-lang", "fermyon", "zcash"];
    let has_upstream = known_upstreams.iter().any(|name| config_str.contains(name));
    assert!(
        has_upstream,
        "supply-chain/config.toml MUST reference at least one upstream \
         vouch source ({:?}); cargo-vet without imports = workflow that \
         constrains nothing (sec-pre-r1-10 §3.6 baseline policy)",
        known_upstreams
    );

    // (3) policy enforces at least `safe-to-deploy`.
    assert!(
        config_str.contains("safe-to-deploy") || config_str.contains("safe-to-run"),
        "supply-chain/config.toml MUST set a non-trivial vetting policy \
         (safe-to-deploy or stricter); without an explicit policy the \
         workflow's check is vacuous"
    );
}
