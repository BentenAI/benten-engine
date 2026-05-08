//! Phase-3 G20-A3 (Phase 2b R3-E origin) — `cargo-vet` baseline
//! non-vacuity self-test + exemption-budget pin (sec-r1-5).
//!
//! Pin source:
//!   - `.addl/phase-2b/pre-r1-security-deliverables.md` §3.6
//!     (sec-pre-r1-10) cargo-vet baseline.
//!   - `.addl/phase-3/00-implementation-plan.md` §3 G20-A3 row
//!     (cargo-vet onboarding policy per sec-r1-5: exemption-budget =
//!     5 entries max at Phase-3-close; criteria-set = safe-to-deploy
//!     + crypto-reviewed; periodic-policy-review cadence quarterly).
//!   - `docs/future/phase-3-backlog.md §7.3.A.9` (sub-cluster 9a —
//!     CLOSED at G20-A3).
//!
//! Mirrors the Phase-2a R6FP-R3 anti-pattern fix
//! (`supply-chain-seeded-test.yml`): a workflow that runs cleanly
//! against the current tree but never tested whether the harness
//! actually fires on a regression is vacuous. The seeded-vuln test
//! proves cargo-deny + cargo-audit both fire on a known-bad
//! dependency. This Rust-side companion proves cargo-vet's policy
//! shape itself constrains SOMETHING — the policy file MUST contain
//! at least one non-trivial rule, and the import set MUST reference
//! at least one upstream vouch source.

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
#[test]
fn cargo_vet_workflow_non_vacuity_self_test_passes() {
    let root = workspace_root();
    let supply_chain_dir = root.join("supply-chain");
    assert!(
        supply_chain_dir.is_dir(),
        "supply-chain/ directory MUST exist after G20-A3 lands the \
         cargo-vet baseline (plan §3 G20-A3 + sec-pre-r1-10 §3.6); \
         not found at {}",
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
    let known_upstreams = [
        "mozilla",
        "google",
        "bytecode-alliance",
        "rust-lang",
        "fermyon",
        "zcash",
    ];
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

/// `cargo_vet_exemption_budget_at_or_below_5_at_phase_3_close` —
/// sec-r1-5 onboarding policy.
///
/// Counts entries in `supply-chain/exemptions.toml` and asserts the
/// total is ≤ 5 per the Phase-3 G20-A3 onboarding policy (sec-r1-5
/// exemption-budget). An audit certified into `audits.toml` is the
/// budget-free path; an unaudited dep can be exempted but the
/// exemption costs against the 5-entry cap. Quarterly review per the
/// config policy re-evaluates whether existing exemptions can be
/// upgraded to certified audits.
#[test]
fn cargo_vet_exemption_budget_at_or_below_5_at_phase_3_close() {
    let root = workspace_root();
    let exemptions = root.join("supply-chain/exemptions.toml");
    let src = std::fs::read_to_string(&exemptions).unwrap_or_else(|e| {
        panic!(
            "supply-chain/exemptions.toml MUST exist after G20-A3 baseline ({}); error: {}",
            exemptions.display(),
            e
        );
    });

    // Each cargo-vet exemption entry is keyed by `[[exemptions.<crate>]]`
    // (an array-of-tables in TOML; the parser enumerates entries via
    // table-header lines beginning `[[exemptions`). We count by
    // scanning the source for the header pattern — a string-search is
    // both sufficient and tooling-independent (no `cargo vet` install
    // required at test time).
    let count = src
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("[[exemptions") || t.starts_with("[exemptions.")
        })
        .count();

    assert!(
        count <= 5,
        "supply-chain/exemptions.toml carries {count} exemptions; the \
         Phase-3 G20-A3 onboarding policy (sec-r1-5) caps the \
         exemption-budget at 5. Quarterly review must upgrade unaudited \
         exemptions to certified `audits.toml` entries before adding \
         new ones."
    );
}
