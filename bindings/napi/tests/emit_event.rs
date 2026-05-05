//! R3-E RED-PHASE pins for G19-B Engine.emitEvent EmitBroadcast wire-through
//! (wave-7 parallel; §7.8 + r1-napi-8).
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
#[ignore = "RED-PHASE: G16-D wave-6b un-ignores; G19-B wave-7 + G14-D wave-5a wire prerequisites — napi EmitBroadcast bus per-replica filter under cross-trust-boundary (stream-r1-7 + stream-r4r1-3)"]
fn napi_emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering()
 {
    // stream-r1-7 + stream-r4r1-3 cross-wave coordination pin.
    //
    // ## Cross-wave ownership (stream-r4r1-3 RECOMMEND)
    //
    // Three implementer waves contribute to making this test green;
    // the test is un-ignored by the LAST of the three:
    //
    //   - G14-D wave-5a: per-subscriber filtering at delivery
    //     (cap_recheck closure consulting durable grant store).
    //   - G19-B wave-7: EmitBroadcast standalone surface
    //     (engine.emitEvent → EmitBroadcast bus publish path).
    //   - G16-D wave-6b: Atrium-replica sync replication (the bus
    //     fans out across replicas for cross-trust-boundary delivery).
    //
    // Per pim-4 §3.10 wave-pairing protocol, **G16-D wave-6b** is the
    // un-ignore wave (the last of the three required-implementer waves
    // — sync-replication is the load-bearing seam that ties the other
    // two waves together at runtime).
    //
    // ## Test-name disambiguation (stream-r4r1-3)
    //
    // This is the napi-side end-to-end pin; the engine-side sibling
    // pin lives at
    // `crates/benten-engine/tests/emit_broadcast_replicas.rs::emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering`
    // (R3-B; G14-D-tagged). Renamed from the duplicate engine-side
    // name to disambiguate per pim-7 §3.5 dim #5 (duplicate test names
    // across crates are a future drift hazard); the napi-side prefix
    // `napi_` preserves the cross-binding ownership signal.
    //
    // G14-D + G19-B + G16-D implementer wires this (un-ignored at G16-D
    // wave-6b — see cross-wave ownership note above):
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
    unimplemented!(
        "G16-D wave-6b un-ignores; G14-D + G19-B prerequisites land per-subscriber filter + EmitBroadcast publish path"
    );
}
