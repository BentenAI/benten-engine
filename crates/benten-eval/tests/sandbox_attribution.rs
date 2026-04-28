//! Phase 2b R3-B — SANDBOX AttributionFrame threading unit test (G7-A).
//!
//! Pin source: sec-pre-r1-03 — closes audit-trail-laundering vector.
//! AttributionFrame must thread through the host-fn dispatch boundary so
//! that audit logs of host-fn invocations carry the dispatching
//! (actor, handler, capability_grant) tuple.
//!
//! Note: D20's `sandbox_depth: u8` extension to AttributionFrame is the
//! Inv-4 / R3-B-fixtures concern (separate file).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — sec-pre-r1-03 attribution threading"]
fn sandbox_attribution_frame_threads_through_host_fn() {
    // sec-pre-r1-03 — register a SANDBOX module that calls `log` once;
    // the engine's host-fn audit trail records the
    // (actor_cid, handler_cid, capability_grant_cid) of the dispatching
    // primitive call.
    //
    // White-box: capture host-fn invocation log; assert the recorded
    // AttributionFrame matches the SANDBOX-primitive's parent frame.
    //
    // Anti-laundering: a host-fn invocation NEVER appears with a
    // null/default AttributionFrame — every call carries the dispatcher's
    // identity.
    todo!("R5 G7-A — capture host-fn audit + assert frame equality");
}
