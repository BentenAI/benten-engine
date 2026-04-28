#![allow(unknown_lints, clippy::duration_suboptimal_units)] // MSRV 1.91 — Rust 1.95 lint suggests from_mins/from_hours, both stabilized in 1.95
//! R3 unit tests for G9-A (P1, P2): wall-clock refresh ceiling + iterate-batch
//! boundary delegation.
//!
//! P1 (FROZEN): `CapabilityPolicy::wallclock_refresh_ceiling()` returns
//!   `Duration::from_secs(300)` by default; a cap-grant can override.
//! P2: `CapabilityPolicy::iterate_batch_boundary` delegation is end-to-end
//!   (the evaluator consults the policy's override, not the Phase-1 constant).
//! ucca-5: HLC consulted alongside `MonotonicSource`.
//!
//! TDD red-phase: `wallclock_refresh_ceiling` does not yet exist on the trait,
//! and the end-to-end delegation from engine to policy does not fire. Tests
//! will fail to compile / fail at runtime until G9-A lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.4 G9-A P1/P2 + ucca-5).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::CapError;
use benten_caps::{CapabilityPolicy, NoAuthBackend, WriteContext};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn caps_wallclock_bound_refreshes_at_300s_default() {
    let policy = NoAuthBackend;
    assert_eq!(
        policy.wallclock_refresh_ceiling(),
        Duration::from_secs(300),
        "NoAuthBackend default wallclock_refresh_ceiling must be 300s"
    );
}

/// Cap-grant-scoped override must replace the default (P1 configurability).
#[test]
fn caps_wallclock_refresh_ceiling_cap_grant_configurable() {
    struct TightPolicy;
    impl CapabilityPolicy for TightPolicy {
        fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
            Ok(())
        }
        fn wallclock_refresh_ceiling(&self) -> Duration {
            Duration::from_secs(30)
        }
    }

    let p = TightPolicy;
    assert_eq!(
        p.wallclock_refresh_ceiling(),
        Duration::from_secs(30),
        "Override must replace the 300s default"
    );
}

/// P2 delegation: a counting-mock policy reports how many times
/// `iterate_batch_boundary` is consulted. When the engine's `PrimitiveHost`
/// delegates to the policy, the counter advances exactly once per batch.
#[test]
fn caps_iterate_batch_delegation_end_to_end() {
    struct CountingPolicy {
        calls: Arc<AtomicUsize>,
        boundary: usize,
    }
    impl CapabilityPolicy for CountingPolicy {
        fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
            Ok(())
        }
        fn iterate_batch_boundary(&self) -> usize {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.boundary
        }
    }

    let calls = Arc::new(AtomicUsize::new(0));
    let policy = CountingPolicy {
        calls: calls.clone(),
        boundary: 7,
    };

    // The shared helper under test MUST consult the policy override. This
    // helper lives in `benten_caps::evaluator_delegation` and is consumed by
    // the engine's PrimitiveHost so the per-batch counter advances exactly
    // once per consulted batch.
    let observed = benten_caps::evaluator_delegation::iterate_batch_boundary_for(&policy);
    assert_eq!(
        observed, 7,
        "evaluator_delegation helper must return the policy's override"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "delegation helper must consult policy exactly once per batch"
    );
}

/// ucca-5: refresh event carries HLC stamp for federation correlation.
/// HLC skew MUST NOT influence cadence (MonotonicSource is authoritative).
#[test]
fn wallclock_hlc_rides_alongside_monotonic() {
    // The `emit_refresh_event` helper surfaces an `HlcStampedRefreshEvent`.
    // The HLC stamp is recorded but does not drive cadence.
    let event = benten_caps::emit_refresh_event_for_test();
    assert!(
        event.hlc_stamp.is_some(),
        "every refresh event must carry an HLC stamp for federation correlation"
    );
    assert!(
        event.monotonic_authoritative,
        "MonotonicSource must be the authoritative cadence driver"
    );
}
