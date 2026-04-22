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
#[test]
#[ignore = "phase-2a-pending: MonotonicSource / TimeSource split lands in G9-A per plan §9.13. Drop #[ignore] once the refresh is cadence-via-monotonic."]
fn wallclock_refresh_ntp_slew_doesnt_skip() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Target API path (G9-A):
    //
    //     let mono = MockMonotonicSource::at_zero();
    //     let wall = MockTimeSource::at_epoch();
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
    //     let handler = iterate_write_handler(400);
    //     let handler_id = engine.register_subgraph(&handler).unwrap();
    //
    //     engine.schedule_revocation_at_iteration(grant_cid, 50).unwrap();
    //
    //     // Advance monotonic 1s per iter; at iter 100, jump wall-clock
    //     // BACKWARD by 3600s (simulating ntpd slew).
    //     mono.advance_per_iter(std::time::Duration::from_secs(1));
    //     mono.on_iter(100, || {
    //         wall.rewind_by(std::time::Duration::from_secs(3600));
    //     });
    //
    //     let outcome = engine
    //         .call(&handler_id, "default", benten_core::Node::empty())
    //         .expect("call wrapper");
    //
    //     // The 300s monotonic-elapsed refresh still fires; revocation
    //     // observed; write at iter 301 denies.
    //     assert_eq!(
    //         outcome.error_code(),
    //         Some("E_CAP_REVOKED_MID_EVAL"),
    //         "backward wall-clock jump MUST NOT skip the monotonic-driven \
    //          refresh; got {:?}",
    //         outcome.error_code()
    //     );

    let _ = engine; // avoid unused-var under #[ignore]
    panic!(
        "red-phase: backward-slew resilience requires monotonic-only \
         cadence (G9-A per §9.13)."
    );
}
