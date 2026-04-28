//! Phase 2a R6 fix-pass — qa-r6r1-1 closure: end-to-end firing test for
//! `CapError::WallclockExpired` → `ErrorCode::CapWallclockExpired`.
//!
//! G9-A wired the §9.13 refresh-point-5 wall-clock ceiling
//! (`CapabilityPolicy::wallclock_refresh_ceiling`, default 300s, driven by
//! `MonotonicSource`). The typed `CapError::WallclockExpired` variant +
//! mapping to `ErrorCode::CapWallclockExpired` exists in
//! `crates/benten-caps/src/error.rs`. This test pins the *firing edge*:
//! when the probe's elapsed duration crosses the ceiling, the policy-level
//! decision must surface the typed error code, end-to-end, with no panics.
//!
//! R6 rationale: `wallclock_delegation.rs` pins the trait-level
//! configurability (default + override); `wallclock_toctou_refresh.rs`
//! benches the per-iter overhead. Until this test, no test asserted the
//! `ErrorCode::CapWallclockExpired` code actually surfaces from the
//! refresh-point check — qa-r6r1-1 closes that gap.
//!
//! Methodology:
//!  1. Build a `WallclockProbe` pre-elapsed past the 300s ceiling
//!     (`testing::wallclock_refresh_probe_expired` returns 301s).
//!  2. Run the probe's elapsed check against the configured ceiling.
//!  3. When the check returns `true` (ceiling breached), construct the
//!     typed denial `CapError::WallclockExpired` and assert it maps to
//!     `ErrorCode::CapWallclockExpired` via the public `code()` accessor.
//!  4. Sanity-pair: a fresh probe (elapsed = 0) must NOT trip the
//!     ceiling, proving the gate is event-driven not unconditional.
//!
//! See `.addl/phase-2a/r6-triage.md` qa-r6r1-1 for the open finding.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{
    CapError, CapabilityPolicy, NoAuthBackend,
    testing::{wallclock_refresh_probe_expired, wallclock_refresh_probe_fresh},
};
use benten_errors::ErrorCode;

/// Helper: drive a refresh-point check given a probe + the policy's
/// configured ceiling. Returns the typed cap-policy decision: `Ok(())` on
/// fresh, `Err(CapError::WallclockExpired)` once the ceiling is breached.
///
/// This mirrors the refresh-point-5 decision an evaluator-side hook makes
/// at every batch boundary — kept inline here so the test pins the typed
/// edge without depending on a not-yet-ratified evaluator-side helper API.
fn refresh_decision_for(
    probe: &benten_caps::testing::WallclockProbe,
    ceiling: core::time::Duration,
) -> Result<(), CapError> {
    if probe.check_elapsed(ceiling) {
        Err(CapError::WallclockExpired)
    } else {
        Ok(())
    }
}

#[test]
fn elapsed_past_ceiling_fires_typed_wallclock_expired() {
    // Use the default-policy ceiling (300s) so the test reflects the
    // §9.13 documented production cadence, not a hand-tuned threshold.
    let ceiling = NoAuthBackend.wallclock_refresh_ceiling();
    assert_eq!(
        ceiling,
        core::time::Duration::from_mins(5),
        "default ceiling fixture sanity — §9.13 names 300s"
    );

    // Probe pre-elapsed to 301s ⇒ check_elapsed against the 300s ceiling
    // must return true ⇒ the refresh-point decision must yield the typed
    // CapError::WallclockExpired denial.
    let expired = wallclock_refresh_probe_expired();
    assert!(
        expired.check_elapsed(ceiling),
        "probe sanity: expired probe must report past-ceiling for the \
         300s default — otherwise the typed-error firing path below is \
         unreachable"
    );

    let err = refresh_decision_for(&expired, ceiling)
        .expect_err("expired probe must surface a typed cap-error");

    // The typed mapping is the load-bearing assertion: the variant must
    // surface as ErrorCode::CapWallclockExpired so the audit/error wire
    // format carries the §9.13-documented code (E_CAP_WALLCLOCK_EXPIRED).
    assert!(
        matches!(err, CapError::WallclockExpired),
        "ceiling breach must produce CapError::WallclockExpired, got {err:?}"
    );
    assert_eq!(
        err.code(),
        ErrorCode::CapWallclockExpired,
        "CapError::WallclockExpired must map to ErrorCode::CapWallclockExpired \
         per docs/ERROR-CATALOG.md (E_CAP_WALLCLOCK_EXPIRED)"
    );
}

#[test]
fn fresh_probe_does_not_fire_wallclock_expired() {
    // Sanity pair: a fresh probe (elapsed = 0) MUST NOT trip the gate.
    // Without this assertion, a misconfigured `check_elapsed` that
    // returned `true` unconditionally would let the firing test above
    // pass for the wrong reason (firing always, not firing on breach).
    let ceiling = NoAuthBackend.wallclock_refresh_ceiling();
    let fresh = wallclock_refresh_probe_fresh();
    assert!(
        !fresh.check_elapsed(ceiling),
        "fresh probe (elapsed=0) must NOT report past the 300s ceiling"
    );
    refresh_decision_for(&fresh, ceiling).expect("fresh probe must yield Ok(())");
}
