//! R3-E RED-PHASE pins for G19-B Engine.emitEvent EmitBroadcast wire-through
//! (wave 7 parallel; §7.8 + r1-napi-8).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-B +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-B must-pass column):
//!
//! - `tests/engine_emit_event_publishes_to_subscribed_on_emit_callback_end_to_end` —
//!   r1-napi-8 (renamed from `engine_emit_event_napi_surface_wires_through_emit_broadcast_bus`
//!   to enforce the pim-2 §3.6b end-to-end pin: production entry point +
//!   observable consequence + would FAIL if silently no-op'd).
//! - `tests/engine_emit_event_no_longer_returns_e_primitive_not_implemented` — §7.8
//!
//! ## What G19-B establishes (§7.8)
//!
//! G19-A (50 LOC) folded into G19-B per scope-real-05. The current state:
//! `engine.emitEvent` is wired to return `E_PRIMITIVE_NOT_IMPLEMENTED`
//! (deferred sentinel). G19-B drops "deferred" + threads
//! `engine.emitEvent` directly through `EmitBroadcast` bus so that any
//! `engine.onEmit(channel, cb)` subscriber receives the payload.
//!
//! ## RED-PHASE discipline
//!
//! Per pim-2 §3.6b end-to-end pin requirement: this test drives the
//! production-grade entry point (real `engine.emitEvent` call) AND
//! asserts an observable consequence (the subscribed callback fires
//! with the payload). If the wire-through were silently no-op'd back
//! to `E_PRIMITIVE_NOT_IMPLEMENTED`, the test would FAIL.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-B wave-7 wires engine.emitEvent through EmitBroadcast bus"]
fn engine_emit_event_publishes_to_subscribed_on_emit_callback_end_to_end() {
    // r1-napi-8 LOAD-BEARING pin per §3.6b. G19-B implementer wires this:
    //
    //   let engine = benten_napi::testing::open_in_memory_engine().unwrap();
    //   let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    //   let received_clone = received.clone();
    //
    //   // Subscribe via the production-grade onEmit entry point:
    //   engine.on_emit("test-channel", move |payload: &serde_json::Value| {
    //       received_clone.lock().unwrap().push(payload.to_string());
    //   }).unwrap();
    //
    //   // Drive the production-grade emit_event entry point:
    //   let payload = serde_json::json!({"hello": "world"});
    //   engine.emit_event("test-channel", payload.clone()).unwrap();
    //
    //   // OBSERVABLE consequence: subscribed callback received the payload
    //   // verbatim. Sentinel-presence (the bus instance exists) does NOT
    //   // suffice; the callback firing is the load-bearing assertion.
    //   let collected = received.lock().unwrap();
    //   assert_eq!(collected.len(), 1, "subscribed onEmit callback must fire once");
    //   assert!(collected[0].contains("\"hello\""),
    //       "subscribed onEmit callback received the wrong payload");
    //
    // The pim-2 contract: would FAIL if engine.emitEvent silently no-op'd
    // (the prior E_PRIMITIVE_NOT_IMPLEMENTED state OR a regression that
    // left the bus publish unwired).
    unimplemented!("G19-B wires engine.emitEvent → EmitBroadcast → onEmit callback round-trip");
}

#[test]
#[ignore = "RED-PHASE: G19-B wave-7 retires E_PRIMITIVE_NOT_IMPLEMENTED for emitEvent"]
fn engine_emit_event_no_longer_returns_e_primitive_not_implemented() {
    // §7.8 pin (negative — verifies the deferred sentinel is GONE).
    // G19-B implementer wires this:
    //
    //   let engine = benten_napi::testing::open_in_memory_engine().unwrap();
    //   let result = engine.emit_event("test-channel", serde_json::json!({}));
    //   assert!(result.is_ok(),
    //       "engine.emitEvent should not return E_PRIMITIVE_NOT_IMPLEMENTED \
    //        after G19-B; got {:?}", result.err());
    //
    //   // Defensive: even the non-Ok path must NOT carry E_PRIMITIVE_NOT_IMPLEMENTED:
    //   if let Err(e) = engine.emit_event("nonexistent-channel-no-subscribers", serde_json::json!({})) {
    //       assert_ne!(e.code(), "E_PRIMITIVE_NOT_IMPLEMENTED",
    //           "post-G19-B emit_event must not return the deferred sentinel");
    //   }
    //
    // OBSERVABLE consequence: the deferred-state placeholder is fully
    // retired. Composes with the positive end-to-end pin above.
    unimplemented!("G19-B retires E_PRIMITIVE_NOT_IMPLEMENTED for emit_event");
}

#[test]
#[ignore = "RED-PHASE: G16-D wave-6 — EmitBroadcast bus per-replica filter under cross-trust-boundary (stream-r1-7 + stream-r4r1-3)"]
fn napi_emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering()
 {
    // stream-r4r1-3 disambiguation: renamed from
    // `emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering`
    // (which clashed with the engine-side pin at
    // `crates/benten-engine/tests/emit_broadcast_replicas.rs`) to the
    // napi-prefixed form. Per stream-r4r1-3 wave-pairing: this pin
    // un-ignores at G16-D wave-6 (the LAST of the three required
    // implementer waves: G14-D wave-5a per-subscriber filtering +
    // G19-B wave-7 EmitBroadcast standalone surface + G16-D wave-6
    // sync replication).
    //
    // stream-r1-7 cross-pin: G19-B's EmitBroadcast bus must NOT cache
    // cap-pass decisions across replica boundaries; each Atrium-replica's
    // per-subscriber cap-recheck fires independently at delivery (the
    // SUBSCRIBE-side discipline carries through to EMIT — see G16-D
    // sibling test `tests/emit_event_fan_out_across_atrium_each_replica_filters_independently_at_delivery`).
    //
    // G19-B + G16-D implementer wires this:
    //   // Two-peer Atrium fan-out scenario:
    //   //   peer-A emits an event;
    //   //   peer-A has subscriber S_A (cap-pass);
    //   //   peer-B has subscriber S_B (cap-fail at the replica's
    //   //   per-subscriber filter, NOT at peer-A's authoritative filter).
    //   // Assert: S_A receives the payload; S_B does NOT, because the
    //   // per-replica filter fires independently and S_B's filter denies.
    //   //
    //   // Defends against the asymmetry shape (peer-A's cap-recheck
    //   // would authoritatively pass-or-fail for peer-B's subscribers,
    //   // a cross-trust-boundary leak).
    unimplemented!("G19-B + G16-D wires EmitBroadcast cross-trust-boundary per-replica filtering");
}
