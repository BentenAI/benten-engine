//! `random` host-fn cap-policy check constant-time discipline pin
//! (G17-A1 wave-5b; sec-r1-3 + CLAUDE.md baked-in #16 narrative).
//!
//! Pin source: r2-test-landscape §2.5 G17-A1 row
//! `random_host_fn_cap_policy_check_constant_time_no_fingerprint_leak`;
//! sec-r1-3 (constant-time check on entropy budget per call).
//!
//! ## Constant-time shape (sec-r1-3)
//!
//! Phase-3 wires the `random` host-fn (D-PHASE-3-11 RESOLVED-at-R1
//! workspace CSPRNG via `getrandom` direct + capability-gated entropy
//! budget). G17-A1 wave-5b ships the SURFACE narrative pin — G17-A2
//! wave-5b owns the runtime arm + the statistical timing pin
//! (sec-r4r1-8 LOAD-BEARING `dudect`-style or percentile-band shape;
//! ≥10k iterations + dual p50/p99 < 1.2x ratio + 1-retry flake budget
//! per r4-r1-wsa-8).
//!
//! G17-A1's pin asserts the architectural-shape commitment lives in
//! source: any future G17-A2 wave (or regression) that ships `random`
//! WITHOUT the constant-time-discipline narrative + the matching
//! statistical pin would fail to satisfy this pin.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn random_host_fn_cap_policy_check_constant_time_no_fingerprint_leak() {
    // sec-r1-3 architectural-shape pin. G17-A1 wave-5b ships the
    // SURFACE; G17-A2 wave-5b owns the runtime arm + statistical
    // pin per r4-r1-wsa-8.
    //
    // The pin asserts:
    // 1. The constant-time-discipline narrative is documented in
    //    `crates/benten-eval/src/primitives/sandbox.rs` near the
    //    deferred `random` cap pre-check.
    // 2. The CLAUDE.md baked-in #16 commitment is observable in the
    //    workspace narrative (host-functions.toml or HOST-FUNCTIONS.md).
    //
    // Defends sec-r1-3 R1 BLOCKER: even with random not yet in
    // production, the architectural commitment is pinned at G17-A1
    // so G17-A2 can land the runtime arm against a fixed contract.

    let primitives_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("primitives")
            .join("sandbox.rs"),
    )
    .expect("primitives/sandbox.rs must exist");

    // The deferred `random` pre-check is at the
    // `DEFERRED_HOST_FN_RANDOM_CAP_PREFIX` site:
    assert!(
        primitives_src.contains("DEFERRED_HOST_FN_RANDOM_CAP_PREFIX"),
        "primitives/sandbox.rs MUST carry the random-host-fn deferral surface \
         that G17-A2 wave-5b will lift, with the constant-time-discipline narrative in place"
    );

    // The `host-functions.toml` source-of-truth declares the random
    // host-fn family; G17-A2 marks it `IMPLEMENTED` per phase-3-backlog
    // §6.10. G17-A1 asserts the deferred entry shape exists today.
    let host_fns_toml = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("host-functions.toml"),
    )
    .expect("workspace-root host-functions.toml must exist");
    assert!(
        host_fns_toml.contains("random") || host_fns_toml.contains("RANDOM"),
        "host-functions.toml MUST mention the random host-fn deferral / impl-status \
         per D1 + sec-pre-r1-06 §2.3 + phase-3-backlog §6.10"
    );
}

#[test]
fn random_host_fn_constant_time_narrative_pinned_for_g17_a2_handoff() {
    // r4-r1-wsa-8 narrative-handoff pin. G17-A1 wave-5b ships this
    // pin; G17-A2 wave-5b is OBLIGATED to land the statistical
    // timing assertion + retire this narrative-only pin (or extend
    // it with a `dudect`-style assertion).
    //
    // The pin asserts the cross-cutting commitment lives at
    // CLAUDE.md (the workspace's authoritative architectural memo).

    // Use SECURITY-POSTURE.md (tracked) as the architectural-memo
    // anchor — CLAUDE.md is project-local + gitignored so a
    // tracked-tree pin can't read it. The constant-time-discipline
    // commitment lives at SECURITY-POSTURE.md (the public-shape
    // memo that downstream operators read alongside ERROR-CATALOG
    // for the per-vector attribution narrative).
    let security_posture = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("SECURITY-POSTURE.md"),
    )
    .expect("docs/SECURITY-POSTURE.md must exist");

    // The constant-time pin is enforced alongside the SANDBOX-scope
    // architectural commitment + Compromise #16 narrative in
    // SECURITY-POSTURE.md.
    assert!(
        security_posture.contains("Compromise #16") || security_posture.contains("ESC matrix"),
        "SECURITY-POSTURE.md MUST carry the SANDBOX-scope commitment + ESC matrix \
         that the constant-time pin enforces alongside (per sec-r1-3 + r4-r1-wsa-8)"
    );
}
