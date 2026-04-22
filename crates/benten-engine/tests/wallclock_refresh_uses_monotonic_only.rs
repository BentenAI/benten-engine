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
#[test]
#[ignore = "phase-2a-pending: MonotonicSource trait + evaluator ITERATE cadence wiring land in G9-A per plan §9.13. Drop #[ignore] once the dual-source split is live."]
fn wallclock_refresh_uses_monotonic_only() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Target API path (G9-A):
    //
    //     use benten_eval::time_source::{MockMonotonicSource, MockTimeSource};
    //
    //     let mono = MockMonotonicSource::at_zero();
    //     let wall = MockTimeSource::frozen_at_epoch();  // frozen wall clock
    //
    //     let engine = Engine::builder()
    //         .path(dir.path().join("benten.redb"))
    //         .monotonic_source(mono.clone())
    //         .time_source(wall.clone())
    //         .capability_policy_grant_backed()
    //         .build()
    //         .unwrap();
    //
    //     let alice = engine.create_principal("alice").unwrap();
    //     let grant_cid = engine
    //         .grant_capability(&alice, "store:post:write").unwrap();
    //
    //     // 400-iter handler; revoke at iter 50 so refresh-point-5 fires
    //     // on the 300s monotonic elapsed boundary even though wall is
    //     // frozen.
    //     let handler = iterate_write_handler(400);
    //     let handler_id = engine.register_subgraph(&handler).unwrap();
    //
    //     engine.schedule_revocation_at_iteration(grant_cid, 50).unwrap();
    //
    //     // Advance monotonic past 300s during the walk. The mock drives
    //     // `elapsed()` forward on each iteration's work block.
    //     mono.advance_per_iter(std::time::Duration::from_secs(1));
    //     // Wall clock stays at epoch forever.
    //
    //     let outcome = engine
    //         .call(&handler_id, "default", benten_core::Node::empty())
    //         .expect("call wrapper");
    //
    //     // Writes 1..=300 succeed (batch-boundary + first 300s of
    //     // monotonic elapsed permit writes); write 301 fires the
    //     // monotonic-driven refresh, sees the revoked grant, denies.
    //     assert_eq!(
    //         outcome.error_code(),
    //         Some("E_CAP_REVOKED_MID_EVAL"),
    //         "monotonic refresh must fire at 300s elapsed even though \
    //          wall-clock is frozen"
    //     );

    let _engine = engine; // avoid unused-var noise under #[ignore]
    panic!(
        "red-phase: MonotonicSource + evaluator cadence wiring not yet \
         present. G9-A to land per plan §9.13 refresh-point-5."
    );
}
