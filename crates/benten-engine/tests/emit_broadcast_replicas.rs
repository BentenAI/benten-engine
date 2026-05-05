//! R3-B RED-PHASE pin: EMIT broadcast bus fan-out under cross-trust-
//! boundary replicas (G14-D wave-5a; stream-r1-7).
//!
//! Pin source: r2-test-landscape §2.2 G14-D row
//! `emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering`;
//! stream-r1-7.
//!
//! ## Architectural intent
//!
//! When EMIT broadcasts to a fan-out bus across replicas in different
//! trust boundaries (different atriums), each subscriber's delivery
//! is filtered PER-SUBSCRIBER via the cap recheck (G14-D's per-event
//! cap recheck). Without this, a subscriber in a different trust
//! boundary could observe events they have no cap to receive.
//!
//! Per stream-r1-7 the load-bearing assertion is that broadcast
//! events DO arrive at the bus end-to-end across replicas, but
//! per-subscriber filtering observably suppresses delivery for
//! subscribers without the matching cap.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — stream-r1-7 — EMIT broadcast cross-trust-boundary per-subscriber filter"]
fn emit_broadcast_bus_fan_out_under_cross_trust_boundary_replicas_via_per_subscriber_filtering() {
    // stream-r1-7 pin. G14-D implementer wires this with two engine
    // instances simulating cross-trust-boundary replicas:
    //
    //   let trusted_engine = benten_engine::Engine::open(trusted_store.path()).unwrap();
    //   let cross_boundary_engine = benten_engine::Engine::open(cross_store.path()).unwrap();
    //
    //   let trusted_subscriber = ...; // has cap on /zone/posts
    //   let untrusted_subscriber = ...; // does NOT have cap
    //
    //   // Both subscribe to the broadcast topic:
    //   let sub_trusted = trusted_engine.subscribe_broadcast("evt:create_post",
    //       trusted_subscriber, ...).unwrap();
    //   let sub_untrusted = cross_boundary_engine.subscribe_broadcast("evt:create_post",
    //       untrusted_subscriber, ...).unwrap();
    //
    //   // EMIT to broadcast bus (replicates across atriums):
    //   trusted_engine.emit_broadcast("evt:create_post", &payload).unwrap();
    //
    //   // Sync replicates the event to the cross-boundary engine bus:
    //   sync::replicate(&trusted_engine, &cross_boundary_engine).unwrap();
    //
    //   // Trusted subscriber receives:
    //   assert_eq!(trusted_engine.delivered_events_for(sub_trusted).len(), 1);
    //   // Cross-boundary subscriber's per-event cap recheck filtered:
    //   assert_eq!(cross_boundary_engine.delivered_events_for(sub_untrusted).len(), 0);
    //
    // OBSERVABLE consequence: the broadcast event ARRIVES at both
    // engines' buses (sync did its job), but per-subscriber filtering
    // at delivery suppresses delivery for the untrusted subscriber.
    // Closes stream-r1-7 by demonstrating the fan-out + filter
    // composition works end-to-end across trust boundaries.
    unimplemented!(
        "G14-D wires EMIT broadcast cross-trust-boundary per-subscriber filtering at delivery"
    );
}
