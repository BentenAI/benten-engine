//! R4-FP-R3-B RED-PHASE pins: Compromise #11 end-to-end composition
//! (G14-D wave-5a + G15-A wave-5a; cap-r4-3 MAJOR closure of
//! cap-minor-4 fix-now-action; load-bearing exit-criterion-6).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-3):
//!
//! - `tests/ivm_view_subscribe_per_row_gate_AND_per_subscriber_filtering_compose_end_to_end`
//! - `tests/ivm_view_subscribe_compose_materialization_deny_wins_over_delivery_allow`
//! - `tests/ivm_view_subscribe_compose_delivery_deny_wins_over_materialization_allow`
//!
//! ## Architectural intent (cap-r4-3 MAJOR closure)
//!
//! Exit-criterion-6 names "per-row READ gate on IVM-materialized
//! views composes G15-A label-hint extraction + G14-D per-subscriber
//! filtering" as a load-bearing closure narrative for Compromise #11.
//!
//! The R3 corpus pins the two layers SEPARATELY:
//! - `ivm_view_per_row_read_gate_against_actor_cap_set` (G15-A
//!   materialization-only)
//! - `ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate`
//!   (G15-A materialization-only)
//! - `subscribe_per_event_cap_recheck_against_durable_grant_store`
//!   (G14-D delivery-only)
//! - `compromise_11_per_row_read_gate_composes_via_helper` (helper
//!   shape only, not full production runtime path)
//!
//! NO existing pin asserts that a SUBSCRIBE on an IVM view fires
//! BOTH gates and that deny-from-either-layer wins. This file closes
//! the gap end-to-end.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G15-A wave-5a
//! AND G14-D wave-5a BOTH land — this composition test un-ignores when
//! the LATER of the two waves lands (per pim-4 §3.10 wave-pairing
//! protocol). Per §3.6b pim-2 these tests must drive the production
//! `engine.subscribe_view(...)` end-to-end + assert the BOTH-gates
//! observable behavior.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G15-A + G14-D — cap-r4-3 — Compromise #11 end-to-end composition"]
fn ivm_view_subscribe_per_row_gate_and_per_subscriber_filtering_compose_end_to_end() {
    // cap-r4-3 pin (load-bearing for exit-criterion-6 Compromise #11
    // closure). The full production runtime path: SUBSCRIBE on an IVM
    // view fires BOTH the materialization-time per-row READ gate
    // (G15-A) AND the delivery-time per-subscriber cap recheck (G14-D),
    // and a deny from EITHER layer suppresses delivery.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let alice_kp = benten_id::keypair::Keypair::generate();
    //   let bob_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Alice has read on /zone/posts (delivery gate would allow):
    //   let alice_grant = ... .audience(alice_kp.public_key().to_did())
    //                          .capability("/zone/posts", "read") ... ;
    //   engine.caps().install_proof(&alice_grant).unwrap();
    //
    //   // Register a user view materialized over /zone/posts:
    //   let view = engine.register_user_view("posts_summary", &subgraph_with_label_hint).unwrap();
    //
    //   // Alice subscribes on the view:
    //   let sub_id = engine.subscribe_view(&view, alice_kp.public_key().to_did(),
    //       |evt| { /* delivery callback */ }).unwrap();
    //
    //   // 1. Write a node ALICE has read on:
    //   engine.write_node(&node_in_zone_posts).unwrap();
    //   // BOTH gates pass: materialization gate sees Alice has read on
    //   // the row's source-zone; delivery gate sees Alice's grant.
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 1,
    //       "BOTH-gates-pass case must deliver");
    //
    //   // 2. Write a node Alice does NOT have read on (e.g., admin zone):
    //   engine.write_node(&node_in_zone_admin).unwrap();
    //   // Materialization gate denies the row from entering Alice's view;
    //   // delivery gate sees no row to deliver.
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 1,
    //       "deny-from-either-layer wins; admin write must not deliver");
    //
    // OBSERVABLE consequence: the full production runtime path
    // composes BOTH gates; deny from either layer wins. Defends
    // against the "helper-shape composability tested but full runtime
    // path silently bypasses one gate" failure shape per pim-2 §3.6b.
    unimplemented!(
        "G15-A + G14-D wire end-to-end composition of materialization + delivery gates per cap-r4-3"
    );
}

#[test]
#[ignore = "RED-PHASE: G15-A + G14-D — cap-r4-3 — materialization deny wins over delivery allow"]
fn ivm_view_subscribe_compose_materialization_deny_wins_over_delivery_allow() {
    // cap-r4-3 pin (negative control 1). When the materialization
    // gate denies a row but the delivery gate would allow it, the
    // composition MUST honor the materialization deny (no delivery).
    //
    // Concrete shape:
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let alice_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Alice has read on /zone/posts (delivery would allow):
    //   let alice_grant = ... .audience(alice_kp.public_key().to_did())
    //                          .capability("/zone/posts", "read") ... ;
    //   engine.caps().install_proof(&alice_grant).unwrap();
    //
    //   // Register a view materialized from /zone/posts (Alice has cap)
    //   // but with label-hint extraction filtering out specific rows
    //   // Alice cannot read at materialization time (e.g., admin-tagged
    //   // posts that require additional cap):
    //   let view = engine.register_user_view("posts_filtered", &subgraph_with_admin_tag_hint).unwrap();
    //
    //   let sub_id = engine.subscribe_view(&view, alice_kp.public_key().to_did(),
    //       |_| {}).unwrap();
    //
    //   // Write a node tagged admin (delivery gate sees Alice has /zone/posts read,
    //   // but materialization gate sees the admin tag requires admin cap Alice lacks):
    //   engine.write_node(&node_in_zone_posts_with_admin_tag).unwrap();
    //
    //   // Materialization deny wins:
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 0,
    //       "materialization deny must win even when delivery gate would allow per cap-r4-3");
    //
    // OBSERVABLE consequence: the materialization gate's deny is
    // load-bearing — it prevents the row from entering the view's
    // change-stream so the delivery gate never sees it. Defends against
    // information leakage at the materialization seam.
    unimplemented!("G15-A + G14-D wire materialization-deny-wins composition per cap-r4-3");
}

#[test]
#[ignore = "RED-PHASE: G15-A + G14-D — cap-r4-3 — delivery deny wins over materialization allow"]
fn ivm_view_subscribe_compose_delivery_deny_wins_over_materialization_allow() {
    // cap-r4-3 pin (negative control 2). When the materialization
    // gate would allow a row but the delivery gate denies it (e.g.,
    // subscriber's grant was revoked between materialization and
    // delivery), the composition MUST honor the delivery deny.
    //
    // Concrete shape:
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let alice_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Alice has read on /zone/posts:
    //   let alice_grant = ... .audience(alice_kp.public_key().to_did())
    //                          .capability("/zone/posts", "read") ... ;
    //   engine.caps().install_proof(&alice_grant).unwrap();
    //
    //   let view = engine.register_user_view("posts_summary", &subgraph_with_label_hint).unwrap();
    //   let sub_id = engine.subscribe_view(&view, alice_kp.public_key().to_did(),
    //       |_| {}).unwrap();
    //
    //   // Revoke Alice's grant (delivery gate now denies):
    //   engine.caps().revoke(&alice_grant.cid()).unwrap();
    //
    //   // Write a node — materialization gate would allow (it doesn't see the
    //   // revocation, since materialization sees structural label-hints, not
    //   // live grant store), but delivery gate observably denies:
    //   engine.write_node(&node_in_zone_posts).unwrap();
    //
    //   // Delivery deny wins:
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 0,
    //       "delivery deny must win even when materialization gate would allow per cap-r4-3");
    //
    // OBSERVABLE consequence: live revocation observably suppresses
    // delivery even on rows the materialization gate had already
    // admitted to the view. Defends against the "admitted row
    // continues delivering forever" failure shape post-revocation.
    unimplemented!("G15-A + G14-D wire delivery-deny-wins composition per cap-r4-3");
}
