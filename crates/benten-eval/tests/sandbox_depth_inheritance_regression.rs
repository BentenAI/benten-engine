//! Phase 2b G7-A — D20 sandbox_depth inheritance regression test.
//!
//! **wsa-g7a-mr-7 fix-pass:** D20-RESOLVED makes load-bearing the
//! property that `AttributionFrame.sandbox_depth` INHERITS across CALL
//! boundaries (not reset). G7-A's scaffold has `SandboxConfig.max_nest_depth`
//! defaulted to 4 but the AttributionFrame schema slot for the depth
//! counter is added by G7-B PR #32 (which extends `exec_state.rs`'s
//! AttributionFrame with `sandbox_depth: u8`).
//!
//! This test STAYS `#[ignore]` until the cross-PR coordination resolves
//! (either G7-B merges first + this PR rebases, or this PR merges first
//! + G7-B rebases). Pointer is named-specific to G7-B PR #32.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "Phase 3 — sandbox_depth runtime threading regression body deferred per docs/future/phase-3-backlog.md §7.3.A.1 (SECURITY-POSTURE.md discloses Inv-4 runtime threading is structural-only in 2b)"]
fn sandbox_depth_inherits_across_call_boundary_not_reset() {
    // wsa-g7a-mr-7 + D20 — assert that 4 nested SANDBOX calls through
    // CALL boundaries (rather than 4 direct nested SANDBOX calls)
    // saturates at the configured max_nest_depth (default 4) and fires
    // E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED. Body lands when:
    //   - G7-B's AttributionFrame.sandbox_depth: u8 lands.
    //   - G7-C's engine integration wires the inheritance + the
    //     check-runtime-entry call site that consumes
    //     `invariants::sandbox_depth::check_runtime_entry`.
    //
    // The structural pin lives at SandboxConfig::max_nest_depth in
    // G7-A. The CALL-boundary inheritance behavior is the load-bearing
    // surface this test pins.
    todo!(
        "G7-B PR #32 + G7-C PR #33 — wire SANDBOX → CALL → SANDBOX chain + assert depth-saturation"
    );
}
