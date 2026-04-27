//! G12-C-cont (Phase 2b R6 A1 closure): assert the migrated
//! `benten_core::Subgraph` canonical-bytes encoding matches the EVAL-SIDE
//! PRODUCTION shape `{handler_id, sorted nodes, sorted edges}`, NOT the
//! Phase-2a stub shape `{handler_id, deterministic}`.
//!
//! # Why the rename
//!
//! The original R3 file name pinned the WRONG encoding direction —
//! `_match_phase_2a_stub_shape` would have asserted the Inv-13-unsafe stub
//! `{handler_id, deterministic}` shape (two handlers with the same
//! `(handler_id, deterministic)` pair could collide even when their node /
//! edge sets differ). The G12-C-cont brief froze the eval-side production
//! shape as authoritative; this file's renamed body asserts the
//! production-shape contract.
//!
//! # What it pins
//!
//! 1. **Round-trip stability** — `decode(encode(sg)) == sg`.
//! 2. **Order independence (Inv-10)** — two builders that produce the same
//!    node/edge set in different construction orders hash to byte-identical
//!    CIDs.
//! 3. **Node-content sensitivity** — adding a node changes the CID.
//! 4. **Edge-content sensitivity** — adding an edge changes the CID.
//! 5. **handler_id sensitivity** — different handler_id => different CID.
//! 6. **deterministic-flag sensitivity** — `deterministic: true` vs
//!    `deterministic: false` => different CIDs.
//! 7. **Inv-13 collision-stability** — two subgraphs with the same
//!    `(handler_id, deterministic)` pair but different nodes have DIFFERENT
//!    CIDs (this is the property the stub shape would have violated).

#![allow(clippy::unwrap_used)]

use benten_core::{
    OperationNode, PrimitiveKind, Subgraph, SubgraphBuilder, canonical_subgraph_bytes,
};

fn read_then_respond(handler_id: &str, deterministic: bool) -> Subgraph {
    let mut b = SubgraphBuilder::new(handler_id);
    let r = b.read("post");
    b.respond(r);
    b.declare_deterministic(deterministic);
    b.build_unvalidated_for_test()
}

#[test]
fn migrated_subgraph_canonical_bytes_round_trip_stable_for_minimal_handler() {
    let sg = read_then_respond("rt-stable", true);
    let bytes_a = canonical_subgraph_bytes(&sg).expect("encode");
    let bytes_b = canonical_subgraph_bytes(&sg).expect("encode again");
    assert_eq!(
        bytes_a, bytes_b,
        "two encodes of the same Subgraph must produce byte-identical bytes"
    );
    let cid_a = sg.cid().expect("cid a");
    let cid_b = sg.cid().expect("cid b");
    assert_eq!(cid_a, cid_b, "cid is deterministic over identical input");
}

#[test]
fn migrated_subgraph_cid_inv10_order_independent_over_node_construction_order() {
    let sg1 = Subgraph::new("inv10-pin")
        .with_node(OperationNode::new("a", PrimitiveKind::Read))
        .with_node(OperationNode::new("b", PrimitiveKind::Respond));
    let sg2 = Subgraph::new("inv10-pin")
        .with_node(OperationNode::new("b", PrimitiveKind::Respond))
        .with_node(OperationNode::new("a", PrimitiveKind::Read));

    assert_eq!(
        canonical_subgraph_bytes(&sg1).expect("enc1"),
        canonical_subgraph_bytes(&sg2).expect("enc2"),
        "different construction orders MUST hash to byte-identical bytes"
    );
}

#[test]
fn migrated_subgraph_cid_changes_when_handler_id_differs() {
    let sg_a = read_then_respond("handler-a", true);
    let sg_b = read_then_respond("handler-b", true);
    assert_ne!(
        sg_a.cid().expect("cid a"),
        sg_b.cid().expect("cid b"),
        "different handler_id MUST produce different CIDs"
    );
}

#[test]
fn migrated_subgraph_cid_changes_when_deterministic_flag_differs() {
    let sg_t = read_then_respond("det-pin", true);
    let sg_f = read_then_respond("det-pin", false);
    assert_ne!(
        sg_t.cid().expect("cid t"),
        sg_f.cid().expect("cid f"),
        "deterministic=true vs deterministic=false MUST produce different CIDs"
    );
}

#[test]
fn migrated_subgraph_cid_changes_when_node_set_differs() {
    let sg_one = Subgraph::new("node-pin").with_node(OperationNode::new("a", PrimitiveKind::Read));
    let sg_two = Subgraph::new("node-pin")
        .with_node(OperationNode::new("a", PrimitiveKind::Read))
        .with_node(OperationNode::new("b", PrimitiveKind::Respond));
    assert_ne!(
        sg_one.cid().expect("cid one"),
        sg_two.cid().expect("cid two"),
        "adding a node MUST change the CID (Inv-13 collision-stability)"
    );
}

#[test]
fn migrated_subgraph_cid_changes_when_edge_set_differs() {
    let base = Subgraph::new("edge-pin")
        .with_node(OperationNode::new("a", PrimitiveKind::Read))
        .with_node(OperationNode::new("b", PrimitiveKind::Respond));
    let with_edge = base.clone().with_edge("a", "b", "next");
    assert_ne!(
        base.cid().expect("cid base"),
        with_edge.cid().expect("cid with-edge"),
        "adding an edge MUST change the CID (Inv-13 collision-stability)"
    );
}

#[test]
fn migrated_subgraph_inv13_collision_stability_two_handlers_with_same_id_and_det_but_different_nodes_differ()
 {
    // The CRITICAL test: this is exactly the property that the Phase-2a
    // stub shape `{handler_id, deterministic}` would have VIOLATED. Under
    // the stub shape both subgraphs would CID-collide because the encoder
    // would only see `(handler_id="x", deterministic=true)`. The
    // production shape includes nodes + edges, so they MUST differ.
    let mut sg_lhs =
        Subgraph::new("inv13-pin").with_node(OperationNode::new("op_left", PrimitiveKind::Read));
    sg_lhs.set_deterministic(true);

    let mut sg_rhs =
        Subgraph::new("inv13-pin").with_node(OperationNode::new("op_right", PrimitiveKind::Write));
    sg_rhs.set_deterministic(true);

    assert_eq!(sg_lhs.handler_id(), sg_rhs.handler_id());
    assert_eq!(
        sg_lhs.is_declared_deterministic(),
        sg_rhs.is_declared_deterministic()
    );

    assert_ne!(
        sg_lhs.cid().expect("cid lhs"),
        sg_rhs.cid().expect("cid rhs"),
        "Inv-13 collision-stability: two handlers with the same (handler_id, \
         deterministic) pair but different node sets MUST produce different CIDs. \
         This is the production-shape contract that the Phase-2a stub shape would \
         have violated."
    );
}

#[test]
fn migrated_subgraph_dagcbor_round_trip_preserves_full_shape() {
    let mut sg = Subgraph::new("dagcbor-rt")
        .with_node(OperationNode::new("a", PrimitiveKind::Read))
        .with_node(OperationNode::new("b", PrimitiveKind::Respond))
        .with_edge("a", "b", "next");
    sg.set_deterministic(true);

    let bytes = sg.to_dagcbor().expect("encode");
    let decoded = Subgraph::from_dagcbor(&bytes).expect("decode");

    assert_eq!(decoded.handler_id(), sg.handler_id());
    assert_eq!(
        decoded.is_declared_deterministic(),
        sg.is_declared_deterministic()
    );
    assert_eq!(decoded.nodes().len(), sg.nodes().len());
    assert_eq!(decoded.edges().len(), sg.edges().len());
    assert_eq!(
        decoded.cid().expect("cid decoded"),
        sg.cid().expect("cid orig"),
        "round-tripped Subgraph must hash to the same CID"
    );
}
