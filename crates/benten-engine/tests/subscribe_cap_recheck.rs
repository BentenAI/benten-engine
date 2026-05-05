//! R3-B RED-PHASE pins: F6 SUBSCRIBE per-event cap recheck against
//! durable grant store (G14-D wave-5a; plan §3 G14-D + F6 LOAD-BEARING +
//! Compromise #2 D5).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-D):
//!
//! - `tests/subscribe_per_event_cap_recheck_against_durable_grant_store` — plan §3 G14-D (unit)
//! - `tests/subscribe_partial_revoke_cancels_subscription_path` — F6 LOAD-BEARING + Compromise #2 D5 (security)
//! - `tests/subscribe_cross_trust_boundary_filters_at_delivery_not_registration` — plan §3 G14-D (security)
//!
//! ## Architectural intent
//!
//! Phase-2b shipped SUBSCRIBE production-runtime; Phase-3 G14-D adds
//! per-event capability recheck against the durable grant store
//! (G14-B). When a subscription path receives a change event, the
//! delivery layer rechecks the subscriber's capability set BEFORE
//! delivering — a partial revocation between subscribe-time and
//! event-time observably cancels the path.
//!
//! Per Compromise #2 D5, cross-trust-boundary filtering happens at
//! DELIVERY, not at REGISTRATION (registration is open; delivery is
//! the auth gate). This reverses the Phase-2b interim shape per
//! plan §3 G14-D.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 these tests must drive the
//! production SUBSCRIBE entry point + assert observable consequences
//! (revoke-then-event observably no-deliver).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — plan §3 G14-D — per-event cap recheck against durable grant store"]
fn subscribe_per_event_cap_recheck_against_durable_grant_store() {
    // plan §3 G14-D pin. G14-D implementer wires this:
    //
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let subscriber_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Subscribe-time: subscriber has read cap on /zone/posts.
    //   let grant = ... .audience(subscriber_kp.public_key().to_did())
    //                   .capability("/zone/posts", "read") ... ;
    //   engine.caps().install_proof(&grant).unwrap();
    //
    //   let sub_id = engine.subscribe("/zone/posts", subscriber_kp.public_key().to_did(), |evt| {
    //       /* delivery callback */
    //   }).unwrap();
    //
    //   // Event 1: cap is still valid → delivers.
    //   engine.write_node(&node_in_zone_posts).unwrap();
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 1);
    //
    //   // Revoke at the durable grant store:
    //   engine.caps().revoke(&grant.cid()).unwrap();
    //
    //   // Event 2: per-event recheck fires; delivery skipped.
    //   engine.write_node(&another_node_in_zone_posts).unwrap();
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 1,
    //       "post-revoke event MUST NOT deliver per F6 per-event cap recheck");
    //
    // OBSERVABLE consequence: writing a node within the subscribed
    // zone after revocation does NOT trigger a delivery, even though
    // the subscription is still REGISTERED.
    unimplemented!("G14-D wires per-event cap recheck at delivery against benten-caps UCANBackend");
}

#[test]
#[ignore = "RED-PHASE: G14-D — F6 LOAD-BEARING + Compromise #2 D5 — partial revoke cancels path"]
fn subscribe_partial_revoke_cancels_subscription_path() {
    // F6 LOAD-BEARING + Compromise #2 D5 pin. When a subscriber's
    // grant is PARTIALLY revoked (e.g., revoke ONLY the read on
    // /zone/admin while leaving /zone/posts intact), only the AFFECTED
    // subscription paths cancel.
    //
    // Implementer wires:
    //
    //   let subscriber_kp = ...;
    //   let grant_posts = ... .capability("/zone/posts", "read") ... ;
    //   let grant_admin = ... .capability("/zone/admin", "read") ... ;
    //   engine.caps().install_proof(&grant_posts).unwrap();
    //   engine.caps().install_proof(&grant_admin).unwrap();
    //
    //   let sub_posts = engine.subscribe("/zone/posts", ..., ...).unwrap();
    //   let sub_admin = engine.subscribe("/zone/admin", ..., ...).unwrap();
    //
    //   // Revoke ONLY admin grant:
    //   engine.caps().revoke(&grant_admin.cid()).unwrap();
    //
    //   // Posts subscription continues delivering:
    //   engine.write_node(&node_in_posts).unwrap();
    //   assert_eq!(engine.delivered_events_for(sub_posts).len(), 1);
    //   // Admin subscription drops:
    //   engine.write_node(&node_in_admin).unwrap();
    //   assert_eq!(engine.delivered_events_for(sub_admin).len(), 0);
    //
    //   // sub_admin observably reports cancelled state:
    //   assert_eq!(engine.subscription_state(sub_admin),
    //       benten_engine::SubscriptionState::CancelledByCapabilityRevocation);
    //
    // OBSERVABLE consequence: precise per-path cancellation; partial
    // revoke isolates correctly; subscription state surface reports
    // the cancellation reason.
    unimplemented!("G14-D wires partial-revoke per-subscription-path cancellation per F6 + #2 D5");
}

#[test]
#[ignore = "RED-PHASE: G14-D — plan §3 G14-D — cross-trust-boundary filters at delivery"]
fn subscribe_cross_trust_boundary_filters_at_delivery_not_registration() {
    // plan §3 G14-D pin. The Phase-2b interim shape filtered cross-
    // trust-boundary subscriptions at REGISTRATION (rejecting at
    // subscribe()). G14-D reverses this: registration is open;
    // delivery enforces the cross-trust-boundary filter via per-event
    // cap recheck.
    //
    // Implementer wires:
    //
    //   let local_subscriber = ...;
    //   let cross_atrium_zone = "/atrium/other/zone/posts";
    //
    //   // Registration succeeds even though caller has no cap:
    //   let sub_id = engine.subscribe(cross_atrium_zone, local_subscriber, ...);
    //   assert!(sub_id.is_ok(),
    //       "registration must NOT reject; filter is at delivery per G14-D");
    //
    //   // Event from cross-atrium replica arrives:
    //   engine.observe_remote_change(cross_atrium_zone, &remote_node).unwrap();
    //
    //   // Delivery DID NOT fire because per-event cap recheck saw no grant:
    //   assert_eq!(engine.delivered_events_for(sub_id.unwrap()).len(), 0);
    //
    // OBSERVABLE consequence: registration is open; events observably
    // do not deliver because the cap recheck has no matching grant.
    // Defends against the "registration-time cap-cache rot" failure
    // shape: caps can be granted AFTER subscribe-time and the delivery
    // path picks them up live.
    unimplemented!(
        "G14-D wires registration-open + delivery-time cap recheck cross-trust-boundary filter"
    );
}
