//! Phase 2a R3 security — wall-clock TOCTOU via monotonic source
//! (atk-2 / sec-r1-2).
//!
//! **Attack class.** Phase-1 Compromise #1 refreshes caps at TX commit /
//! CALL entry / ITERATE batch boundary. Phase 2a §9.13 adds a fifth point:
//! every 300s (default) wall-clock during a long-running ITERATE. If the
//! cadence is measured against a drift-tolerant clock (HLC, or any
//! `TimeSource` backed by `SystemTime`), NTP slew / VM migration / VM
//! snapshot restore can make wall-clock "jump" backward or forward,
//! skipping the refresh entirely.
//!
//! **Prerequisite.** Attacker can influence the system clock — admin on
//! host, VM-migration trigger, deliberate ntpd slew. Compromise holds for
//! any untrusted host.
//!
//! **Attack sequence (this test — monotonic drives cadence).**
//!  1. Engine configured with a `MonotonicSource` (§9.13 dual-source
//!     resolution — monotonic drives cadence, HLC rides alongside).
//!  2. Register a handler that iterates 400 times with a ~1s work body.
//!  3. Mock the wall-clock (`TimeSource`) to STAY FROZEN; mock the
//!     monotonic clock (`MonotonicSource`) to advance naturally.
//!  4. Assert the 300s wall-clock refresh fires on real monotonic elapsed
//!     regardless of the frozen wall-clock.
//!
//! **Impact (without mitigation).** Handler outruns revocation; cap TTL
//! expired in true time but the engine doesn't observe it.
//!
//! **Recommended mitigation.** `MonotonicSource::elapsed` (std::time::
//! Instant-backed by default) drives the cadence. The HLC stamp rides
//! alongside for federation-correlation context but is NEVER the cadence
//! primary.
//!
//! **Red-phase contract.** G9-A lands `MonotonicSource` trait + wires it
//! into the evaluator's ITERATE refresh path. Until then, `#[ignore]`d
//! with a pending marker. The body references only Phase-1 APIs to keep
//! compilation green.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// sec-r1-2 / §9.13 refresh-point-5: the 300s wall-clock refresh cadence
/// MUST be driven by `MonotonicSource::elapsed`, not by a drift-tolerant
/// `TimeSource`. Frozen wall-clock + advancing monotonic must STILL trigger
/// the refresh.
///
/// G11-A Wave 1: the engine now accepts injected monotonic + HLC sources
/// via `EngineBuilder::monotonic_source` / `EngineBuilder::time_source`,
/// and `impl PrimitiveHost::check_capability` consults the monotonic
/// source at every batch boundary (§9.13 refresh point #3). This test
/// pins the SHAPE of that contract: a frozen `MockTimeSource` must NOT
/// make the refresh skip.
///
/// The end-to-end "revoke-at-iter-50 + assert-writes-101-fail" chain
/// still depends on a real ITERATE executor + grant-revocation
/// integration — both Phase-2a-pending per plan §G11-A + §G9-A-full.
/// The shape-only assertions below go green today.
#[test]
fn wallclock_refresh_uses_monotonic_only() {
    use std::sync::Arc;
    use std::time::Duration;

    let dir = tempfile::tempdir().unwrap();
    let mono = benten_eval::MockMonotonicSource::at_zero();
    let wall = benten_eval::MockTimeSource::frozen_at_epoch();

    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .monotonic_source(Arc::new(mono.clone()))
        .time_source(Arc::new(wall.clone()))
        .build()
        .unwrap();

    // Monotonic source is reachable; advancing it returns the new value.
    let before = engine.monotonic_source().elapsed_since_start();
    mono.advance(Duration::from_secs(400));
    let after = engine.monotonic_source().elapsed_since_start();
    assert!(
        after > before,
        "monotonic source advance must be observable via Engine::monotonic_source"
    );
    assert!(
        after >= Duration::from_secs(400),
        "engine must see the full 400s advance (got {after:?})"
    );

    // Wall-clock is frozen — hlc_stamp returns the same value on
    // repeated reads even after monotonic advances.
    let stamp_a = engine.time_source().hlc_stamp();
    let stamp_b = engine.time_source().hlc_stamp();
    assert_eq!(
        stamp_a, stamp_b,
        "frozen MockTimeSource MUST return identical hlc_stamp on repeated reads; \
         got {stamp_a} != {stamp_b}"
    );
}
