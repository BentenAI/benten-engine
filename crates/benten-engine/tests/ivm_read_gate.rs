//! R3-C RED-PHASE pins for IVM per-row read-gate at materialization
//! (G15-A wave-5a; closes Compromise #11 in coordination with G14-D).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-A rows
//!   `ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate`
//!   + `ivm_view_per_row_read_gate_against_actor_cap_set`.
//! - plan §3 G15-A row + plan §1 deliverable 6 (Compromise #11
//!   per-row read-gate closure).
//! - `ivm-major-2` (gate fires AT MATERIALIZATION TIME, separately
//!   from G14-D delivery-time gate).
//! - LOAD-BEARING #11 closure pin per plan §1 line "Compromise #11
//!   ... closed end-to-end".
//! - composes G15-A label-hint extraction + G14-D per-subscriber
//!   filtering at the SUBSCRIBE delivery side.
//!
//! ## Compromise #11 closure narrative
//!
//! Phase-2b shipped IVM views with COARSE-GRAINED read-gating: a view
//! either was visible to an actor in full or not at all (per-zone
//! gating only). Phase-3 closes Compromise #11 by adding a per-row
//! READ-cap check at view materialization time. The G14-D F6 SUBSCRIBE
//! filtering at delivery time is a DIFFERENT layer: that gates which
//! events flow to a subscriber. The G15-A materialization-time gate
//! prevents an actor from materializing a view whose backing rows
//! they cannot READ.
//!
//! ## RED-PHASE discipline
//!
//! Every test is `#[ignore]`'d with rationale
//! `"RED-PHASE: G15-A wave-5a closes Compromise #11"`. Tests stay
//! ignored until G15-A wave-5a un-ignores at mini-review.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — ivm-major-2 — materialization-time gate distinct from G14-D delivery"]
fn ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate() {
    // ivm-major-2 pin. The per-row READ gate fires at MATERIALIZATION
    // TIME — not at delivery time (which is G14-D's job). Concrete
    // shape:
    //
    //   let actor_caps = ActorCapSet::new()
    //       .grant_read("/zone/posts/public/*")
    //       .build();
    //   let view = engine.user_view("posts_view")
    //       .with_actor_cap_set(&actor_caps)
    //       .materialize();
    //   // Only rows the actor can READ are returned; the gate fires
    //   // at materialization, not at SUBSCRIBE delivery.
    //   for row in view.rows() {
    //       assert!(row.zone_path().starts_with("/zone/posts/public/"));
    //   }
    //   // A row in /zone/posts/private/ exists in the underlying
    //   // graph, but does NOT appear in this actor's materialized
    //   // view because the materialization-time gate filtered it.
    //   assert!(!view.rows().iter().any(|r| r.zone_path().starts_with("/zone/posts/private/")));
    //
    // OBSERVABLE consequence: actors with restricted READ caps see
    // ONLY their permitted rows in user-view materializations; this
    // is independent of G14-D delivery filtering (which gates
    // ChangeEvent stream subscriptions, not snapshot reads).
    unimplemented!(
        "G15-A wires per-row READ gate at materialization separate from G14-D delivery gate"
    );
}

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — LOAD-BEARING Compromise #11 closure"]
fn ivm_view_per_row_read_gate_against_actor_cap_set() {
    // LOAD-BEARING per plan §1 deliverable 6: Compromise #11 closes
    // end-to-end. G15-A implementer wires this against the production
    // materialization arm + a fixture actor cap set:
    //
    //   let posts: Vec<Node> = (0..100)
    //       .map(|i| make_post_node(i, if i % 2 == 0 { "public" } else { "private" }))
    //       .collect();
    //   for n in &posts { engine.write_node(n); }
    //
    //   let read_only_public = ActorCapSet::new()
    //       .grant_read("/zone/posts/public/*")
    //       .build();
    //   let view = engine.user_view("posts_view")
    //       .with_actor_cap_set(&read_only_public)
    //       .materialize();
    //   assert_eq!(view.rows().len(), 50);
    //   for row in view.rows() {
    //       assert_eq!(row.label(), "post");
    //       assert!(row.zone_path().contains("/public/"));
    //   }
    //
    // OBSERVABLE consequence: under a 100-node fixture split 50/50
    // public vs private, an actor with READ caps only on public sees
    // EXACTLY 50 rows in their view materialization. The
    // load-bearing #11-closure pin asserts the per-row gate fires
    // for every row, not at view-aggregate level (Phase-2b coarse
    // gate would have returned 0 or 100 — never 50).
    unimplemented!(
        "G15-A wires the LOAD-BEARING #11-closure per-row gate against the production materialization arm"
    );
}
