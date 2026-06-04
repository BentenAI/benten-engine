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
#[ignore = "DISAGREE-WITH-EXPLANATION (HARD RULE clause-c) — cross-trust-boundary 2-peer Atrium fan-out scenario cannot drive through `benten_napi::testing::emit_event_round_trip` (single-engine harness; no peer-replica sync surface). Engine-side coverage is GREEN at `crates/benten-engine/tests/emit_broadcast_replicas.rs::emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering` (R3-B; G14-D-tagged) — that pin drives the cross-trust-boundary per-subscriber filter directly against the engine's `EmitBroadcast` bus. The napi shim at `bindings/napi/src/lib.rs::Engine::emit_event` is zero-policy delegation to `Engine::emit_event` (G19-B wave-7). Cross-trust-boundary semantics require Atrium-replica sync (G16-D wave-6b) which is engine-layer; the napi boundary contributes zero policy logic. Asymmetry-shape defense (peer-A's cap-recheck would authoritatively pass-or-fail for peer-B's subscribers) is structurally engine-layer; napi-side observation cannot meaningfully add coverage. Original pin's reference to a 2-peer harness via `emit_event_round_trip` is not implementable without adding multi-engine plumbing to the rlib testing surface (production-code change out of Class A scope). Companion engine-side GREEN pin remains load-bearing per pim-2 §3.6b."]
fn napi_emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering()
 {
    // RE-DISPOSITION RATIONALE (pre-v1 Class A un-ignore, 2026-05-10):
    //
    // Original RED-PHASE body asked for a 2-peer scenario:
    //   peer-A emits an event;
    //   peer-A has subscriber S_A (cap-pass);
    //   peer-B has subscriber S_B (cap-fail at peer-B's filter).
    // Assert: S_A receives, S_B does NOT.
    //
    // The napi rlib testing surface
    // (`benten_napi::testing::emit_event_round_trip`) opens a SINGLE
    // in-memory engine. Driving the 2-peer scenario from the napi-side
    // integration test would require adding multi-engine + sync-replica
    // plumbing to the production-code testing module — out of Class A
    // scope (no production-code modifications).
    //
    // The cross-trust-boundary semantics are STRUCTURALLY engine-layer:
    // peer-A's authoritative filter vs peer-B's per-replica filter
    // composition runs on the engine's `EmitBroadcast` bus. The napi
    // adapter at `bindings/napi/src/lib.rs::Engine::emit_event` is a
    // zero-policy delegation — there is no napi-side widening surface
    // a regression could exploit that the engine-side pin doesn't
    // already protect.
    //
    // GREEN engine-side coverage:
    //   crates/benten-engine/tests/emit_broadcast_replicas.rs
    //   ::emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering
    //
    // The 2-peer test name is preserved here for retrospective
    // traceability; the test stays `#[ignore]`-with-DISAGREE since the
    // body is structurally not implementable at this layer.
    panic!(
        "see #[ignore] rationale — DISAGREE-WITH-EXPLANATION; engine-side GREEN pin at \
         crates/benten-engine/tests/emit_broadcast_replicas.rs is the load-bearing site"
    );
}
