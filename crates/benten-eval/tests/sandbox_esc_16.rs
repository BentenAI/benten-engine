//! ESC-16 (fingerprint-collapse defense) closure pins
//! (G17-A1 wave-5b; r1-wsa-4 MAJOR + phase-3-backlog §6.1).
//!
//! Pin sources: r2-test-landscape §2.5 G17-A1 row
//! `sandbox_esc_16_fingerprint_collapse_fires_via_committed_wat_fixture`;
//! r1-wsa-4 MAJOR + phase-3-backlog §6.1.
//!
//! ## ESC-16 closure shape
//!
//! Phase-2b deferred ESC-16 (fingerprint-collapse — guest reads a
//! wallclock-influenced internal state to fingerprint host
//! nondeterminism) pending the engine-side memory-read helper
//! architecture. r1-wsa-4 pinned the architecture at
//! `crates/benten-eval/src/sandbox/fingerprint.rs` (NEW per G17-A1).
//!
//! G17-A1 wave-5b ships:
//!
//! 1. The [`benten_eval::sandbox::WallclockTaintedAddress`] side-table
//!    marker + [`benten_eval::sandbox::record_wallclock_write`] /
//!    [`benten_eval::sandbox::read_collapse_state`] helpers in
//!    `fingerprint.rs`.
//! 2. The [`benten_eval::sandbox::FINGERPRINT_COLLAPSE_THRESHOLD`]
//!    detection threshold (3 reads in one call).
//! 3. The [`benten_eval::sandbox::run_esc16_check`] defense entry
//!    point that fires the typed `SandboxError::EscapeAttempt` with
//!    [`benten_eval::sandbox::EscVector::Esc16FingerprintCollapse`].
//! 4. The
//!    [`benten_eval::testing::testing_simulate_fingerprint_collapse_pattern`]
//!    helper that drives the threshold-met state.
//!
//! The full fixture-driven runtime test (driving an `.wat` fixture
//! end-to-end) un-ignores at G20-A1 wave-8a per phase-3-backlog
//! §7.3.A.7. G17-A1 wave-5b ships the SURFACE + simulation pin.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::sandbox::SandboxError;
use benten_eval::sandbox::{
    EscDefenseState, EscVector, FINGERPRINT_COLLAPSE_THRESHOLD, WallclockTaintedAddress,
    read_collapse_state, record_wallclock_write, run_esc16_check,
};
use benten_eval::testing::testing_simulate_fingerprint_collapse_pattern;

#[test]
fn sandbox_esc_16_fingerprint_collapse_fires_via_committed_wat_fixture() {
    // r1-wsa-4 pin — the simulation helper sets the read counter to
    // the threshold; the defense fires.
    let mut state = EscDefenseState::new();
    testing_simulate_fingerprint_collapse_pattern(&mut state);

    let err = run_esc16_check(&state).expect_err("ESC-16 attack must surface as Err");

    assert!(
        matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc16FingerprintCollapse,
                ..
            }
        ),
        "ESC-16 attack MUST surface as EscapeAttempt(Esc16FingerprintCollapse); got: {err:?}"
    );

    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::SandboxEscapeAttempt,
        "ESC-16 routes to E_SANDBOX_ESCAPE_ATTEMPT"
    );

    // The engine-side memory-read helper lives at the r1-wsa-4-pinned
    // location. Source-cite assertion (the source-of-truth shape):
    let helper_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("sandbox")
            .join("fingerprint.rs"),
    )
    .expect("fingerprint.rs must exist at the r1-wsa-4-pinned location");
    assert!(
        helper_src.contains("read_collapse_state") || helper_src.contains("fingerprint"),
        "ESC-16 engine-side memory-read helper MUST live at fingerprint.rs per r1-wsa-4"
    );
}

#[test]
fn esc_16_below_threshold_silent() {
    // A read counter below the threshold does NOT fire — guards
    // against an over-eager regression that would falsely accuse
    // legitimate guests of fingerprint-collapse.
    let mut state = EscDefenseState::new();
    state.fingerprint_correlated_reads = FINGERPRINT_COLLAPSE_THRESHOLD.saturating_sub(1);
    assert!(
        run_esc16_check(&state).is_ok(),
        "ESC-16 must be silent below the {} read threshold",
        FINGERPRINT_COLLAPSE_THRESHOLD
    );
}

#[test]
fn esc_16_read_collapse_state_increments_only_on_tainted_addresses() {
    // The engine-side memory-read helper increments only when the
    // address is in the tainted-address side-table.
    let mut state = EscDefenseState::new();
    let tainted_addr = record_wallclock_write(0x1000);
    let tainted_table = vec![tainted_addr];

    // Untainted read — counter does not increment.
    read_collapse_state(&mut state, WallclockTaintedAddress(0x2000), &tainted_table);
    assert_eq!(state.fingerprint_correlated_reads, 0);

    // Tainted read — counter increments.
    read_collapse_state(&mut state, tainted_addr, &tainted_table);
    assert_eq!(state.fingerprint_correlated_reads, 1);
}
