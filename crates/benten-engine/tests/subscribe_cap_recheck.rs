//! R3-B RED-PHASE pins: F6 SUBSCRIBE per-event cap recheck against
//! durable grant store (G14-D wave-5a; plan §3 G14-D + F6 LOAD-BEARING +
//! Compromise #2 D5).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-D):
//!
//! - `tests/subscribe_per_event_cap_recheck_against_durable_grant_store` — plan §3 G14-D (unit)
//! - `tests/subscribe_partial_revoke_cancels_subscription_path` — F6 LOAD-BEARING + Compromise #2 D5 (security)
//! - `tests/subscribe_cross_trust_boundary_filters_at_delivery_not_registration` — plan §3 G14-D (security)
//! - `tests/atrium_grant_revocation_synced_across_peers_terminates_in_flight_subscriptions_within_hlc_bound` — R4-R2-FP ds-r4r2-1 (cross-peer HLC-bounded propagation; closes ds-r4-4(b))
//! - `tests/subscribe_per_zone_scoping_phone_receives_only_subscribed_zone_writes` — R4-R2-FP ds-r4r2-6 (per-zone scoping unit decomposition; closes ds-r4-9)
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
#[ignore = "RED-PHASE: G14-D — plan §3 G14-D — per-event cap recheck against durable grant store (blocked on G14-B durable UCAN backend's `chain-for-audience` accessor wired through `engine.caps()`; G14-B already merged at main `496e144`, un-ignore when accessor stabilizes)"]
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
#[ignore = "RED-PHASE: G14-D — F6 LOAD-BEARING + Compromise #2 D5 — partial revoke cancels path (blocked on G14-B durable UCAN backend's `chain-for-audience` accessor wired through `engine.caps()`; G14-B already merged at main `496e144`, un-ignore when accessor stabilizes)"]
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
    //   // sub_admin observably reports cancelled state via the
    //   // `subscription_active` accessor (per stream-r4r1-6: narrow
    //   // to observable boolean assertion to avoid introducing the
    //   // SubscriptionState enum surface ad-hoc; if the implementer
    //   // chooses to expose a richer reason-typed enum, that is a
    //   // separate G14-D scope item with napi/TS parity):
    //   assert!(!engine.subscription_active(sub_admin),
    //       "sub_admin must report inactive after partial revoke per F6 + #2 D5");
    //   // Subsequent admin writes do not deliver:
    //   assert_eq!(engine.delivered_events_for(sub_admin).len(), 0);
    //
    // OBSERVABLE consequence: precise per-path cancellation; partial
    // revoke isolates correctly; observable consequence asserted via
    // delivery-count + active-flag, not via a new typed enum surface.
    unimplemented!("G14-D wires partial-revoke per-subscription-path cancellation per F6 + #2 D5");
}

#[test]
#[ignore = "RED-PHASE: G14-D — plan §3 G14-D — cross-trust-boundary filters at delivery (blocked on G14-B durable UCAN backend's `chain-for-audience` accessor wired through `engine.caps()`; G14-B already merged at main `496e144`, un-ignore when accessor stabilizes)"]
fn subscribe_cross_trust_boundary_filters_at_delivery_not_registration() {
    // plan §3 G14-D pin. The Phase-2b interim shape filtered cross-
    // trust-boundary subscriptions at REGISTRATION (rejecting at
    // subscribe()). G14-D reverses this: registration is open;
    // delivery enforces the cross-trust-boundary filter via per-event
    // cap recheck.
    //
    // Implementer wires (LOCAL-vs-REMOTE 2-peer scenario per ds-r4r2-1
    // half (a) — drives an explicit cross-peer scenario where peer
    // LOCAL's grant store differs from peer REMOTE's grant store, and
    // asserts LOCAL's cap-set gates LOCAL delivery, NOT REMOTE's
    // cap-set):
    //
    //   // Two engines representing two peers in the same Atrium:
    //   let local_engine = benten_engine::Engine::open(local_store.path()).unwrap();
    //   let remote_engine = benten_engine::Engine::open(remote_store.path()).unwrap();
    //
    //   let local_subscriber_kp = benten_id::keypair::Keypair::generate();
    //   let cross_atrium_zone = "/atrium/other/zone/posts";
    //
    //   // KEY: REMOTE peer's grant store has a grant for local_subscriber;
    //   // LOCAL peer's grant store does NOT. Cross-peer cap-set
    //   // asymmetry — the question is which peer's cap-set gates LOCAL
    //   // delivery.
    //   let remote_grant = ... .audience(local_subscriber_kp.public_key().to_did())
    //                          .capability(cross_atrium_zone, "read") ... ;
    //   remote_engine.caps().install_proof(&remote_grant).unwrap();
    //   // local_engine.caps() — empty for this audience.
    //
    //   // Registration on LOCAL peer succeeds even though LOCAL has no cap
    //   // for this audience (registration is open per G14-D):
    //   let sub_id = local_engine.subscribe(
    //       cross_atrium_zone,
    //       local_subscriber_kp.public_key().to_did(),
    //       |_| {},
    //   );
    //   assert!(sub_id.is_ok(),
    //       "registration must NOT reject; filter is at delivery per G14-D");
    //
    //   // Event from REMOTE peer arrives at LOCAL via Atrium sync:
    //   local_engine.observe_remote_change(cross_atrium_zone, &remote_node).unwrap();
    //
    //   // OBSERVABLE consequence: LOCAL's cap-set is consulted (NOT
    //   // REMOTE's). LOCAL has no grant for local_subscriber, so the
    //   // per-event cap-recheck at LOCAL's delivery boundary denies
    //   // delivery — even though REMOTE's cap-set DOES authorize it.
    //   // This pins the LOCAL-not-REMOTE distinction.
    //   assert_eq!(local_engine.delivered_events_for(sub_id.unwrap()).len(), 0,
    //       "LOCAL cap-set gates LOCAL delivery; REMOTE peer's grant must NOT \
    //        authorize LOCAL delivery (cross-peer trust-boundary preserved)");
    //
    //   // Now install the same grant LOCALLY:
    //   local_engine.caps().install_proof(&remote_grant).unwrap();
    //
    //   // Next event delivers (LOCAL cap-set now has matching grant):
    //   local_engine.observe_remote_change(cross_atrium_zone, &remote_node_2).unwrap();
    //   assert_eq!(local_engine.delivered_events_for(sub_id.unwrap()).len(), 1,
    //       "after local grant install, LOCAL cap-set authorizes delivery");
    //
    // OBSERVABLE consequence: registration is open; events observably
    // do not deliver because the LOCAL cap recheck has no matching
    // grant. The 2-peer LOCAL-vs-REMOTE-cap-set scenario explicitly
    // verifies the LOCAL-not-REMOTE distinction (REMOTE having the
    // grant does NOT authorize LOCAL delivery; only LOCAL's cap-set
    // gates LOCAL delivery). Defends against the "registration-time
    // cap-cache rot" failure shape AND the "REMOTE peer's caps shadow
    // LOCAL caps at delivery" cross-peer failure shape.
    unimplemented!(
        "G14-D wires registration-open + delivery-time cap recheck cross-trust-boundary filter \
         with LOCAL-not-REMOTE cap-set distinction per ds-r4r2-1(a)"
    );
}

// =====================================================================
// R4-R2-FP-C RED-PHASE pins: ds-r4r2-1(b) cross-peer HLC-bounded grant-
// revocation propagation + ds-r4r2-6 per-zone scoping unit decomposition.
//
// Pin sources (per .addl/phase-3/r4-r2-distributed-systems.json):
//
// - ds-r4r2-1(b): atrium_grant_revocation_synced_across_peers_terminates_in_flight_subscriptions_within_hlc_bound
//   (closes ds-r4-4(b); composes G14-D F6 + G16-B Loro/MST sync + G14-pre-D HLC bounded-window math)
// - ds-r4r2-6: subscribe_per_zone_scoping_phone_receives_only_subscribed_zone_writes
//   (closes ds-r4-9; G14-D unit-shape decomposition of the multi-device e2e composite)
//
// Both pins follow pim-2 §3.6b discipline: drive production entry point
// (engine.subscribe / engine.atrium / engine.observe_remote_change) +
// assert observable behavioral consequence + would FAIL if implementation
// arms were silently no-op'd (unimplemented!() body in RED-PHASE).
// =====================================================================

#[test]
#[ignore = "RED-PHASE: G14-D + G16-B — ds-r4r2-1(b) — cross-peer revocation terminates in-flight subscription within HLC-bounded window"]
fn atrium_grant_revocation_synced_across_peers_terminates_in_flight_subscriptions_within_hlc_bound()
{
    // ds-r4r2-1(b) pin (closes ds-r4-4(b) per R4-R2 distributed-systems
    // re-emergent finding). Composes G14-D F6 per-event cap recheck +
    // G16-B Loro/MST cross-peer revocation propagation + G14-pre-D HLC
    // bounded-window math. Three peers A / B / C in the same Atrium:
    //
    //   1. Peer A grants peer B read access to /zone/posts.
    //   2. The grant propagates through Atrium sync to peer C.
    //   3. Peer C opens an in-flight SUBSCRIBE for /zone/posts (a path
    //      whose writes are produced by peer B).
    //   4. Peer A REVOKES the grant for peer B at HLC time t_revoke.
    //   5. The revocation propagates through Atrium sync to peer C.
    //   6. Peer C's in-flight SUBSCRIBE filter MUST observably terminate
    //      within an HLC-bounded window after the revocation event
    //      arrives — i.e. before the next event whose HLC > t_revoke
    //      + bounded_window_ms is delivered.
    //
    // Implementer wires:
    //
    //   use benten_core::Hlc;
    //   let bounded_window_ms = 5_000;  // documented bound per HLC skew tolerance
    //
    //   // Three engines, three Atrium peers. Atrium join via the B-prime
    //   // session-handle DSL per D1 (D-PHASE-3-D1):
    //   let peer_a = benten_engine::Engine::open(a_store.path()).unwrap();
    //   let peer_b = benten_engine::Engine::open(b_store.path()).unwrap();
    //   let peer_c = benten_engine::Engine::open(c_store.path()).unwrap();
    //   let atrium = ATRIUM_TEST_FIXTURE_3_PEER;
    //   peer_a.atrium(atrium.config()).join().unwrap();
    //   peer_b.atrium(atrium.config()).join().unwrap();
    //   peer_c.atrium(atrium.config()).join().unwrap();
    //
    //   // Peer A grants peer B read on /zone/posts:
    //   let peer_b_did = peer_b.local_did();
    //   let grant = ... .issuer(peer_a.local_did())
    //                   .audience(peer_b_did.clone())
    //                   .capability("/zone/posts", "read") ... ;
    //   peer_a.caps().install_proof(&grant).unwrap();
    //   peer_a.atrium_sync_to_convergence().unwrap();  // propagate grant
    //
    //   // Peer C opens in-flight SUBSCRIBE for the path peer B writes
    //   // (peer C is observing peer B's contributions to /zone/posts):
    //   let sub_id = peer_c.subscribe("/zone/posts", peer_b_did.clone(), |_| {}).unwrap();
    //   assert!(peer_c.subscription_active(sub_id),
    //       "subscription must be active before revocation arrives");
    //
    //   // Peer B writes; peer C delivers (cap is still valid):
    //   let n1 = peer_b.write_node_in_zone("/zone/posts", &node_1).unwrap();
    //   peer_b.atrium_sync_to_convergence().unwrap();
    //   assert_eq!(peer_c.delivered_events_for(sub_id).len(), 1);
    //
    //   // Peer A revokes the grant at HLC time t_revoke; revocation
    //   // propagates through Atrium sync to peer C:
    //   let t_revoke: Hlc = peer_a.hlc_now();
    //   peer_a.caps().revoke(&grant.cid()).unwrap();
    //   peer_a.atrium_sync_to_convergence().unwrap();
    //
    //   // Peer C's revocation-arrival observation HLC:
    //   let t_revoke_arrived_at_c: Hlc = peer_c.last_observed_revocation_hlc(&grant.cid())
    //       .expect("peer C must observe the revocation event after sync");
    //   assert!(t_revoke_arrived_at_c >= t_revoke,
    //       "revocation HLC at peer C must be >= original revoke HLC at peer A");
    //
    //   // OBSERVABLE consequence (i): the in-flight SUBSCRIBE at peer C
    //   // observably terminates AFTER the revocation arrives. The
    //   // subscription_active() flag flips to false within the
    //   // bounded window:
    //   let t_termination: Hlc = peer_c.subscription_terminated_at_hlc(sub_id)
    //       .expect("subscription must report typed termination HLC after revoke arrives");
    //   assert!(!peer_c.subscription_active(sub_id),
    //       "subscription must be inactive after cross-peer revocation arrives");
    //   let window_ms = t_termination.physical_ms_since(t_revoke_arrived_at_c);
    //   assert!(window_ms <= bounded_window_ms,
    //       "subscription termination must be within HLC-bounded window of \
    //        {bounded_window_ms} ms after revocation arrives at peer C; got {window_ms} ms");
    //
    //   // OBSERVABLE consequence (ii): peer B writes that arrive at
    //   // peer C with HLC > t_revoke_arrived_at_c + bounded_window_ms
    //   // observably DO NOT deliver via this subscription:
    //   let n2 = peer_b.write_node_in_zone("/zone/posts", &node_2).unwrap();
    //   peer_b.atrium_sync_to_convergence().unwrap();
    //   assert_eq!(peer_c.delivered_events_for(sub_id).len(), 1,
    //       "post-revocation event MUST NOT deliver via terminated subscription");
    //
    //   // OBSERVABLE consequence (iii): a typed termination cause is
    //   // observable (closes the SET-vs-ORDER attack class — peer C
    //   // can distinguish "terminated by cross-peer revocation" from
    //   // "terminated by local revocation" via the typed reason):
    //   let cause = peer_c.subscription_termination_cause(sub_id)
    //       .expect("typed termination cause observable per ds-r4r2-1(b)");
    //   assert!(matches!(cause,
    //       benten_engine::SubscriptionTerminationCause::CrossPeerGrantRevocation { .. }),
    //       "termination cause must be typed CrossPeerGrantRevocation per ds-r4r2-1(b)");
    //
    // OBSERVABLE consequence: a peer's in-flight SUBSCRIBE filter
    // observably terminates within an HLC-bounded window after a
    // cross-peer grant-revocation event propagates via Atrium sync.
    // Defends against the failure shape where revocation propagation
    // races subscription delivery + an unbounded window allows
    // post-revocation events to leak. Composes G14-D F6 (per-event
    // cap recheck) + G16-B (Loro/MST cross-peer revocation sync) +
    // G14-pre-D (HLC bounded-window math from
    // crates/benten-core/src/hlc.rs DEFAULT_SKEW_TOLERANCE_MS).
    unimplemented!(
        "G14-D + G16-B wire cross-peer grant-revocation propagation with HLC-bounded subscription \
         termination per ds-r4r2-1(b)"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — ds-r4r2-6 — per-zone scoping unit decomposition (phone subscribes to one zone, receives only that zone's writes)"]
fn subscribe_per_zone_scoping_phone_receives_only_subscribed_zone_writes() {
    // ds-r4r2-6 pin (closes ds-r4-9 per R4-R2 distributed-systems
    // re-emergent finding). Unit-shape decomposition of the multi-
    // device composite e2e tests/integration/atrium_two_device.rs —
    // failure-localization improvement: when this test fails, the
    // root cause is per-zone subscription scoping (not heterogeneous
    // capability envelopes, not bidirectional sync, not Inv-14
    // device-grain attribution, not the other 4 behaviors bundled in
    // the composite e2e).
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let phone_kp = benten_id::keypair::Keypair::generate();
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope::default();
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, phone_kp.public_key().to_did(), envelope).unwrap();
    //
    //   let engine = benten_engine::Engine::open_for_device(
    //       store.path(), parent_kp.clone(), phone_kp.clone(), attestation).unwrap();
    //
    //   // Phone subscribes ONLY to /zone/notifications:
    //   let phone_did = phone_kp.public_key().to_did();
    //   let sub_id = engine.subscribe("/zone/notifications", phone_did, |_| {}).unwrap();
    //
    //   // Writes hit three different zones — only /zone/notifications
    //   // matches the subscription scope:
    //   engine.write_node_in_zone("/zone/notifications", &notif_1).unwrap();
    //   engine.write_node_in_zone("/zone/posts", &post_1).unwrap();
    //   engine.write_node_in_zone("/zone/admin", &admin_1).unwrap();
    //   engine.write_node_in_zone("/zone/notifications", &notif_2).unwrap();
    //
    //   // OBSERVABLE consequence: phone observably receives ONLY the
    //   // 2 /zone/notifications writes — NOT the /zone/posts write,
    //   // NOT the /zone/admin write. Per-zone scoping holds at unit
    //   // grain, isolating the failure surface from the composite
    //   // e2e's 6 other concerns.
    //   let delivered = engine.delivered_events_for(sub_id);
    //   assert_eq!(delivered.len(), 2,
    //       "phone subscription scoped to /zone/notifications must receive \
    //        exactly the 2 notifications-zone writes, not posts or admin");
    //   for evt in &delivered {
    //       assert!(evt.zone().starts_with("/zone/notifications"),
    //           "every delivered event must be from the subscribed zone; \
    //            got zone {}", evt.zone());
    //   }
    //
    // OBSERVABLE consequence: per-zone subscription scoping holds at
    // unit grain — phone subscribed to /zone/notifications observably
    // does NOT receive /zone/posts or /zone/admin writes. Defends
    // against regression where SUBSCRIBE accidentally delivers
    // cross-zone (a confused-deputy / cross-trust-boundary leak).
    // Decomposes exit-criterion 16 sub-property (a) from the composite
    // tests/integration/atrium_two_device.rs e2e for failure-
    // localization at the G14-D scope.
    unimplemented!(
        "G14-D wires per-zone subscription scoping unit pin per ds-r4r2-6 (decomposes \
         atrium_two_device.rs composite e2e for failure-localization)"
    );
}
