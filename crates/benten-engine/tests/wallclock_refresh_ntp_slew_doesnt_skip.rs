#![allow(unknown_lints, clippy::duration_suboptimal_units)]
// MSRV 1.91 — Rust 1.95 lint
//! Phase 2a R3 security — wall-clock NTP slew cannot skip cap refresh
//! (atk-2 / sec-r1-2).
//!
//! R4 qa-r4-10 cross-reference: R2 §4.6 lists this under
//! `crates/benten-engine/tests/integration/wallclock_toctou.rs`. Phase-2a
//! keeps the per-scenario file split; the R2 collective filename is the
//! landscape anchor.
//!
//! **Attack class.** Adversary arranges an NTP slew that jumps the wall-
//! clock BACKWARD by an hour (or VM-restore snapshot), making the cap
//! refresh cadence's naive arithmetic (`now - last_refreshed`) go negative
//! — implementations that treat negative elapsed as "not yet due" skip the
//! refresh entirely for the duration of the slew.
//!
//! **Prerequisite.** Same as sec-r1-2 sibling test: host clock attacker.
//! More adversarial than the frozen-wall case — this one deliberately
//! moves the wall-clock BACKWARD.
//!
//! **Attack sequence.**
//!  1. Engine uses `MonotonicSource` for cadence (§9.13).
//!  2. Register a long-running ITERATE handler.
//!  3. Mock wall-clock jumps from T to T-3600s after iter 50.
//!  4. Monotonic continues advancing.
//!  5. At monotonic elapsed = 300s, refresh MUST fire irrespective of the
//!     wall-clock's apparent backward travel.
//!
//! **Impact.** Unbounded cap-TTL bypass under clock manipulation.
//!
//! **Recommended mitigation.** Cadence measured against monotonic only.
//! Wall-clock backward jump doesn't affect the scheduler. This test pins
//! that semantic so a future regression that ties cadence to wall-clock
//! diff is caught.
//!
//! **Red-phase contract.** Same as the monotonic-only sibling: G9-A wires
//! the dual-source; `#[ignore]`d until it lands.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// sec-r1-2 companion to `wallclock_refresh_uses_monotonic_only`: NTP slew
/// that jumps the wall-clock BACKWARD must not skip the refresh.
///
/// G11-A Wave 1 pins the shape contract (monotonic + wall-clock sources
/// are independently controllable; a backward wall-clock rewind does NOT
/// reduce monotonic elapsed). The end-to-end "backward-slew at iter 100
/// still denies at the 300s monotonic boundary" assertion requires a
/// real ITERATE + grant-revocation chain (Phase-2a-pending per plan
/// §G9-A-full).
#[test]
fn wallclock_refresh_ntp_slew_doesnt_skip() {
    use std::sync::Arc;
    use std::time::Duration;

    let dir = tempfile::tempdir().unwrap();
    let mono = benten_eval::MockMonotonicSource::at_zero();
    let wall = benten_eval::MockTimeSource::at(Duration::from_secs(7_200));

    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .monotonic_source(Arc::new(mono.clone()))
        .time_source(Arc::new(wall.clone()))
        .build()
        .unwrap();

    // Advance monotonic forward 400s.
    mono.advance(Duration::from_secs(400));
    let mono_after_fwd = engine.monotonic_source().elapsed_since_start();
    assert!(mono_after_fwd >= Duration::from_secs(400));

    // Simulate an NTP slew: jump the wall-clock BACKWARD 1 hour.
    let wall_before_slew = engine.time_source().hlc_stamp();
    wall.rewind_by(Duration::from_secs(3_600));
    let wall_after_slew = engine.time_source().hlc_stamp();
    assert!(
        wall_after_slew < wall_before_slew,
        "backward wall-clock rewind MUST be observable as hlc_stamp decrease; \
         got {wall_before_slew} -> {wall_after_slew}"
    );

    // Monotonic has NOT moved backward — still at 400s elapsed. This is
    // the semantic guarantee refresh-point-3 depends on.
    let mono_after_slew = engine.monotonic_source().elapsed_since_start();
    assert!(
        mono_after_slew >= mono_after_fwd,
        "monotonic source MUST be strictly non-decreasing regardless of \
         wall-clock slew; got {mono_after_fwd:?} -> {mono_after_slew:?}"
    );
}
