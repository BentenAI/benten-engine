//! R3 unit tests for G9-A (P1, P2): wall-clock refresh ceiling + iterate-batch
//! boundary delegation.
//!
//! P1 (FROZEN): `CapabilityPolicy::wallclock_refresh_ceiling()` returns
//!   `Duration::from_mins(5)` by default; a cap-grant can override.
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
        Duration::from_mins(5),
        "NoAuthBackend default wallclock_refresh_ceiling must be 5 minutes"
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

// =====================================================================
// R4-FP-R3-B RED-PHASE pins: Phase-3 evaluator-delegation runtime-arm
// closure (G14-B wave-4b; cap-r4-8 MINOR closure of cap-minor-8
// fix-now-action; closes Phase-2a residual TODOs at policy.rs:281-326).
//
// Pin sources (per R4 R1 capability-system-reviewer lens, finding
// r4-r1-cap-8):
//
// - `policy_iterate_batch_boundary_evaluator_delegation_observable_in_runtime_arm`
// - `policy_wallclock_refresh_ceiling_evaluator_delegation_observable_in_runtime_arm`
//
// ## Architectural intent
//
// The Phase-2a-era SHAPE-PINS above (`caps_iterate_batch_delegation_end_to_end`
// and `caps_wallclock_refresh_ceiling_cap_grant_configurable`) test the helper
// shape; they do NOT exercise end-to-end runtime delegation through the
// engine's evaluator. The TODOs at `policy.rs:281-326` document the
// remaining wire-up work, paired with G14-B's durable UCAN backend.
// These pins close that residual end-to-end.
// =====================================================================

#[test]
#[ignore = "phase-3-backlog §2.3 (ii) — cap-r4-8 — iterate-batch evaluator delegation observable in runtime arm. Destination: v1-assessment-window co-routed with §10.1 Compromise #1 TOCTOU window bound (the two items share the same iterate-batch-boundary cap-recheck mechanism + must ship together; coupling is structural, not speculative). Un-ignore once Engine::run_iterate_subgraph_with_metrics + IterateMetrics::refresh_count + the §10.1 bound-revisit decision land per §2.3 (ii) item 1."]
fn policy_iterate_batch_boundary_evaluator_delegation_observable_in_runtime_arm() {
    // cap-r4-8 pin (cap-minor-8 closure). Custom policy with override
    // = 5; ITERATE-heavy subgraph; observe refresh fires every 5 iters
    // in production runtime arm.
    //
    // Concrete shape:
    //   struct OverridePolicy { boundary: usize }
    //   impl benten_caps::CapabilityPolicy for OverridePolicy {
    //       fn check_write(&self, _ctx: &benten_caps::WriteContext) -> Result<(), benten_caps::CapError> { Ok(()) }
    //       fn iterate_batch_boundary(&self) -> usize { self.boundary }
    //   }
    //
    //   let policy = OverridePolicy { boundary: 5 };
    //   let engine = benten_engine::Engine::builder()
    //       .with_policy(policy)
    //       .open(store_dir.path()).unwrap();
    //
    //   // Drive an ITERATE-heavy subgraph (e.g., 100 iterations):
    //   let metrics = engine.run_iterate_subgraph_with_metrics(&iterate_100x_subgraph).unwrap();
    //
    //   // Refresh fires every 5 iters (override observed end-to-end):
    //   assert_eq!(metrics.refresh_count, 100 / 5,
    //       "evaluator must consult policy override per cap-r4-8");
    //
    //   // Source-cite that the TODO at policy.rs:281-326 is closed:
    //   let src = std::fs::read_to_string("crates/benten-caps/src/policy.rs").unwrap();
    //   assert!(!src.contains("TODO(phase-3 — iterate-batch-boundary policy delegation)"),
    //       "policy.rs iterate-batch TODO must be closed at G14-B per cap-r4-8");
    //
    // OBSERVABLE consequence: the evaluator threads through to the
    // policy's iterate_batch_boundary at every batch; not just the
    // helper shape but the production runtime path. Closes the
    // Phase-2a residual end-to-end.
    unimplemented!(
        "phase-3-backlog §2.3 (ii) item 1 — Engine::run_iterate_subgraph_with_metrics + IterateMetrics::refresh_count + policy.rs:281-326 TODO closure (v1-assessment-window per CLAUDE.md item #15)"
    );
}

#[test]
#[ignore = "phase-3-backlog §2.3 (ii) — cap-r4-8 — wallclock refresh ceiling evaluator delegation observable in runtime arm. Destination: v1-assessment-window co-routed with §10.1 Compromise #1 TOCTOU window bound (shared iterate-batch-boundary cap-recheck mechanism; structural coupling). Un-ignore once Engine::run_call_with_metrics_for_duration + CallMetrics::wallclock_refresh_count + the §10.1 bound-revisit decision land per §2.3 (ii) item 2."]
fn policy_wallclock_refresh_ceiling_evaluator_delegation_observable_in_runtime_arm() {
    // cap-r4-8 pin (cap-minor-8 closure). Custom policy with override
    // = 30s; long-running CALL; observe refresh fires at 30s wall-clock
    // in production runtime arm.
    //
    // Concrete shape:
    //   struct ShortRefreshPolicy;
    //   impl benten_caps::CapabilityPolicy for ShortRefreshPolicy {
    //       fn check_write(&self, _ctx: &benten_caps::WriteContext) -> Result<(), benten_caps::CapError> { Ok(()) }
    //       fn wallclock_refresh_ceiling(&self) -> Duration { Duration::from_secs(30) }
    //   }
    //
    //   let engine = benten_engine::Engine::builder()
    //       .with_policy(ShortRefreshPolicy)
    //       .open(store_dir.path()).unwrap();
    //
    //   // Drive a long-running CALL (>30s wall-clock):
    //   let metrics = engine.run_call_with_metrics_for_duration(
    //       &long_running_subgraph, Duration::from_secs(90)).unwrap();
    //
    //   // Refresh fires every 30s (override observed end-to-end):
    //   assert_eq!(metrics.wallclock_refresh_count, 90 / 30,
    //       "evaluator must consult policy override per cap-r4-8");
    //
    //   // Source-cite that the TODO at policy.rs:281-326 is closed:
    //   let src = std::fs::read_to_string("crates/benten-caps/src/policy.rs").unwrap();
    //   assert!(!src.contains("TODO(phase-3 — wallclock-refresh-ceiling evaluator wire-up)"),
    //       "policy.rs wallclock TODO must be closed at G14-B per cap-r4-8");
    //
    // OBSERVABLE consequence: the evaluator threads through to the
    // policy's wallclock_refresh_ceiling at runtime; production path
    // not just helper shape. Closes the Phase-2a residual end-to-end.
    unimplemented!(
        "phase-3-backlog §2.3 (ii) item 2 — Engine::run_call_with_metrics_for_duration + CallMetrics::wallclock_refresh_count + policy.rs:281-326 TODO closure (v1-assessment-window per CLAUDE.md item #15)"
    );
}
