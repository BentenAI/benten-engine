//! Phase 2b R3 (R3-E) — D12 + Q4 (G12-E owns WAIT TTL) integration.
//!
//! TDD red-phase. Pin source: plan §5 D12-RESOLVED (optional
//! `ttl_hours: NonZeroU32` with 24h default + 720h max + hybrid GC) +
//! r1-streaming-systems D12 implementation hint (TTL field lives in
//! `SubgraphSpec.primitives` per-WAIT property; GC lives in
//! `crates/benten-eval/src/primitives/wait.rs`; new error codes
//! `E_WAIT_TTL_EXPIRED` + `E_WAIT_TTL_INVALID`) + orchestrator Q4
//! resolution (G12-E owns the WAIT TTL surface alongside the
//! generalized SuspensionStore).
//!
//! This test asserts the cross-cutting integration:
//!   suspend with `ttl_hours: 1` →
//!   advance simulated wall-clock past the deadline →
//!   resume →
//!   surface `E_WAIT_TTL_EXPIRED` (NOT a permissive `Complete(value)` fallback).
//!
//! The simulated clock advance uses the §9 backdoor helper
//! `testing_advance_wait_clock` (R3-consolidation addendum) to avoid
//! wall-clock-real test latency.
//!
//! **Status:** Test scaffold SHIPPED at tag `phase-2b-close`
//! (`3d0f018`); body deferred to Phase 3 per
//! `docs/future/phase-3-backlog.md §7.3.A.6`. `ttl_hours` field on
//! WaitSpec + `E_WAIT_TTL_EXPIRED` variant landed structurally; the
//! GC sweep + `testing_advance_wait_clock` simulated-clock helper
//! land Phase-3 alongside the broader durable-WAIT cluster.
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use std::time::Duration;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// `wait_ttl_expires_via_suspension_store` — R2 §1.10 + §8.1 D12 +
/// orchestrator Q4 (G12-E owns).
///
/// End-to-end: register a WAIT with `ttl_hours: 1`; suspend; advance the
/// engine's wait-clock by 2 hours; attempt resume; assert the typed
/// `E_WAIT_TTL_EXPIRED` error fires AND the SuspensionStore entry has
/// been GC'd.
#[test]
#[ignore = "Phase 3 — WAIT ttl_hours + GC + E_WAIT_TTL_EXPIRED end-to-end body deferred per docs/future/phase-3-backlog.md §7.3.A.6"]
fn wait_ttl_expires_via_suspension_store() {
    let (_dir, mut engine) = fresh_engine();

    // Register a handler whose body waits with ttl_hours: 1.
    let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    engine
        .register_subgraph("test.wait_with_ttl", spec)
        .unwrap();

    // Drive the engine to the suspension point + capture the envelope.
    let envelope =
        benten_engine::testing::testing_call_to_suspend(&mut engine, "test.wait_with_ttl")
            .expect("handler must reach the WAIT suspension point cleanly");

    // Confirm the SuspensionStore now holds the wait metadata.
    assert!(
        benten_engine::testing::testing_suspension_store_has_wait(&engine, &envelope),
        "after suspend, the SuspensionStore MUST hold the wait metadata \
         (G12-E generalized store; D12 ttl_hours stored alongside)"
    );

    // Advance the wait clock 2 hours past the 1h deadline.
    benten_engine::testing::testing_advance_wait_clock(&mut engine, Duration::from_secs(2 * 3600));

    // Resume MUST surface E_WAIT_TTL_EXPIRED — NOT a permissive fallback.
    let err = engine
        .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
        .expect_err(
            "resume after TTL expiry MUST fail with E_WAIT_TTL_EXPIRED \
             (D12 fail-closed; permissive Complete(value) fallback rejected)",
        );
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_WAIT_TTL_EXPIRED"),
        "expected E_WAIT_TTL_EXPIRED error code in rendered body, got: {}",
        rendered
    );

    // The hybrid GC (event-driven on resume) MUST have removed the
    // entry by the time we land here.
    assert!(
        !benten_engine::testing::testing_suspension_store_has_wait(&engine, &envelope),
        "after resume-on-expiry, GC MUST have removed the entry from the \
         SuspensionStore (D12 hybrid GC: event-driven sweep on resume + \
         interval backstop + drop-final)"
    );
}
