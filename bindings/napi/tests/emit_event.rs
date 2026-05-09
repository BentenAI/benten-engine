//! Phase-3 G19-B ACTIVATED pins (wave-7 parallel) — Engine.emitEvent
//! EmitBroadcast wire-through (§7.8 + r1-napi-8).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-B +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-B must-pass column):
//!
//! - `tests/engine_emit_event_publishes_to_subscribed_on_emit_callback_end_to_end` —
//!   r1-napi-8 (renamed to enforce the pim-2 §3.6b end-to-end pin:
//!   production entry point + observable consequence + would FAIL if
//!   silently no-op'd).
//! - `tests/engine_emit_event_no_longer_returns_e_primitive_not_implemented` — §7.8
//!
//! ## What G19-B establishes (§7.8)
//!
//! G19-A (50 LOC) folded into G19-B per scope-real-05. Pre-G19-B
//! `engine.emitEvent` returned `E_PRIMITIVE_NOT_IMPLEMENTED`; G19-B
//! drops the deferred sentinel + threads `engine.emitEvent` directly
//! through the `EmitBroadcast` bus so any
//! `engine.subscribe_emit_events(cb)` subscriber receives the payload.
//!
//! ## Pim-2 §3.6b end-to-end discipline
//!
//! Drives the production-grade entry point (real `Engine::emit_event`
//! call via `benten_napi::testing::emit_event_round_trip` — which itself
//! threads through the same code path the napi cdylib `#[napi] fn
//! emit_event` consumes) AND asserts an observable consequence (the
//! subscribed callback fires with the verbatim payload). Would FAIL if
//! the wire-through were silently no-op'd back to
//! `E_PRIMITIVE_NOT_IMPLEMENTED`.

#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

#[test]
fn engine_emit_event_publishes_to_subscribed_on_emit_callback_end_to_end() {
    // r1-napi-8 LOAD-BEARING pin per §3.6b. Drives the production-grade
    // emit_event entry point + asserts the subscribed callback fires.
    let received: Arc<Mutex<Vec<benten_engine::EmitEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let payload = serde_json::json!({"hello": "world"});

    let _engine = benten_napi::testing::emit_event_round_trip(
        "test-channel",
        payload.clone(),
        move |event: &benten_engine::EmitEvent| {
            received_clone.lock().unwrap().push(event.clone());
        },
    )
    .expect("emit_event_round_trip must succeed post-G19-B");

    // OBSERVABLE consequence: subscribed callback received the payload
    // verbatim. Sentinel-presence (the bus instance exists) does NOT
    // suffice — the callback firing is the load-bearing assertion.
    let collected = received.lock().unwrap();
    assert_eq!(
        collected.len(),
        1,
        "subscribed onEmit callback must fire once post-G19-B; got {} events",
        collected.len()
    );
    assert_eq!(collected[0].channel, "test-channel");
    // The Value is a benten_core::Value::Map; round-trip via Display
    // (the EmitEvent payload is a Value, not a JSON Value, so we
    // just verify the channel + presence here; the structured-field
    // surfacing is covered by the `benten_error_context` test).
    let payload_repr = format!("{:?}", collected[0].payload);
    assert!(
        payload_repr.contains("hello"),
        "payload must round-trip; repr was {payload_repr}"
    );
}

#[test]
fn engine_emit_event_no_longer_returns_e_primitive_not_implemented() {
    // §7.8 pin (negative — verifies the deferred sentinel is GONE).
    // Per pim-2 §3.6b: drives the production entry point + asserts
    // the deferred-state placeholder is fully retired.
    //
    // Calling emit_event_round_trip with no subscribed listeners
    // should still succeed (publishing an event with no subscribers
    // is not an error); the no-op closure here just ensures the
    // helper completes without an EngineError surface.
    let result = benten_napi::testing::emit_event_round_trip(
        "no-subscribers-channel",
        serde_json::json!({}),
        |_event: &benten_engine::EmitEvent| { /* drop event */ },
    );
    assert!(
        result.is_ok(),
        "engine.emitEvent must NOT return E_PRIMITIVE_NOT_IMPLEMENTED post-G19-B; got {:?}",
        result.err()
    );

    // OBSERVABLE consequence: even when no subscribers exist, the
    // emit path completes cleanly through EmitBroadcast — there is
    // no deferred-sentinel guard left in the code path.
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — napi EmitBroadcast bus per-replica filter under cross-trust-boundary. G14-D wave-5a + G19-B wave-7 + G16-D wave-6b ALL shipped (PR #115 + #127 + #163); test body pins specific napi EmitBroadcast cross-trust-boundary filter contract; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
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
