//! Phase 2b Wave-8h audit-gap fix #2 — standalone EMIT primitives are
//! observable via the engine's dedicated EMIT broadcast.
//!
//! Pin source:
//! `.addl/phase-2b/r4b-followup-primitive-executor-docs-vs-code-audit.json`
//! "EMIT" PARTIAL verdict.
//!
//! ## Pre-fix behaviour (the bug)
//!
//! `crates/benten-engine/src/primitive_host.rs::emit_event` was a
//! documented no-op: a handler that used a standalone EMIT primitive
//! (no backing WRITE) silently dropped the payload. The
//! `ChangeBroadcast` channel did not see the EMIT (it's wired only to
//! storage WRITEs), and there was no other observer surface for
//! emit-only events.
//!
//! ## Post-fix behaviour (this test)
//!
//! Wave-8h adds [`crate::emit_broadcast::EmitBroadcast`] (a channel
//! structurally separate from `ChangeBroadcast` — see the module's
//! own doc for why). The engine's `emit_event` wrapper now publishes
//! an [`benten_engine::EmitEvent`] with the channel name + payload;
//! consumers attach via [`Engine::subscribe_emit_events`]. A standalone
//! EMIT now produces an observable event.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use benten_core::Value;
use benten_engine::{EmitEvent, Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

/// Build a 2-node SubgraphSpec (EMIT -> RESPOND) carrying the EMIT
/// node's `channel` + `payload` properties on the primitive's
/// properties bag. This is the "standalone EMIT" shape the audit
/// flagged as silently dropping payloads.
fn emit_only_spec(handler_id: &str, channel: &str, payload: Value) -> SubgraphSpec {
    let mut emit_props: BTreeMap<String, Value> = BTreeMap::new();
    emit_props.insert("channel".into(), Value::Text(channel.to_string()));
    emit_props.insert("payload".into(), payload);

    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "e0".into(),
            kind: PrimitiveKind::Emit,
            properties: emit_props,
        })
        .respond()
        .build()
}

/// Wave-8h audit-gap fix #2 — standalone EMIT publishes through the
/// engine's EMIT broadcast. A subscriber attached via
/// `subscribe_emit_events` MUST receive the (channel, payload) pair.
#[test]
fn emit_event_observable_via_emit_broadcast() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Capture observed events. `Mutex<Vec<EmitEvent>>` is the simplest
    // shape; the broadcast invokes the callback synchronously on the
    // dispatch thread so a plain Mutex collection is sufficient.
    let observed: Arc<Mutex<Vec<EmitEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let observed_for_cb = Arc::clone(&observed);
    engine.subscribe_emit_events(move |ev| {
        observed_for_cb.lock().unwrap().push(ev.clone());
    });
    assert_eq!(
        engine.emit_subscriber_count(),
        1,
        "after subscribe_emit_events, the subscriber count must be 1 — \
         confirms the broadcast is wired"
    );

    // Register a standalone-EMIT handler. The payload includes both a
    // string + a numeric so the assertion can verify the Value
    // payload survives the broadcast intact.
    let payload = Value::Map({
        let mut m = BTreeMap::new();
        m.insert("kind".into(), Value::Text("user.signed_up".into()));
        m.insert("user_id".into(), Value::Int(42));
        m
    });
    let spec = emit_only_spec("wave8h.emit_observable", "users.signups", payload.clone());
    let handler_id = engine.register_subgraph(spec).unwrap();

    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect("standalone EMIT dispatch must complete cleanly through the OK edge");
    assert!(
        outcome.is_ok_edge(),
        "EMIT is fire-and-forget; the dispatch MUST route through OK; \
         got edge {:?}",
        outcome.edge_taken(),
    );

    // The load-bearing assertion: the broadcast subscriber observed
    // the EMIT. Pre-wave-8h the no-op host wrapper would have left
    // `observed` empty regardless of the dispatch outcome.
    let captured = observed.lock().unwrap();
    assert_eq!(
        captured.len(),
        1,
        "subscribe_emit_events callback MUST have fired exactly once for \
         the standalone EMIT primitive; got {} events. Pre-wave-8h this \
         vector would be empty because emit_event was a no-op.",
        captured.len()
    );
    assert_eq!(
        captured[0].channel, "users.signups",
        "EmitEvent.channel must equal the EMIT node's `channel` property"
    );
    assert_eq!(
        captured[0].payload, payload,
        "EmitEvent.payload must equal the EMIT node's `payload` property \
         (Value-equality, including nested Map structure)"
    );
}

/// Companion regression test — multiple subscribers all receive the
/// same EMIT event (proves fan-out + protects against accidental
/// "first subscriber wins" regressions).
#[test]
fn emit_event_fans_out_to_every_subscriber() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let count_a = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_b = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_a_cb = Arc::clone(&count_a);
    let count_b_cb = Arc::clone(&count_b);
    engine.subscribe_emit_events(move |_| {
        count_a_cb.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    engine.subscribe_emit_events(move |_| {
        count_b_cb.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    assert_eq!(engine.emit_subscriber_count(), 2);

    let spec = emit_only_spec(
        "wave8h.emit_fans_out",
        "test.channel",
        Value::Text("payload".into()),
    );
    let handler_id = engine.register_subgraph(spec).unwrap();
    let _ = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["x".into()], Default::default()),
        )
        .unwrap();

    assert_eq!(
        count_a.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "subscriber A must receive exactly one event"
    );
    assert_eq!(
        count_b.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "subscriber B must receive exactly one event (fan-out is structural)"
    );
}
