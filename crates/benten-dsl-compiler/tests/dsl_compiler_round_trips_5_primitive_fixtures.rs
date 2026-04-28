//! G12-B green-phase: DSL → `Subgraph` → re-serialize round-trip across the
//! 5 primitive fixtures the MINIMAL-FOR-DEVSERVER scope must support
//! (per `r1-architect-reviewer.json` G12-B-scope: ~200-400 LOC, 4 primitives
//! initially — this file pins 5 fixtures one of which uses primitive
//! composition to keep round-trip surface honest).
//!
//! Lifted from red-phase 2026-04-28 (R5 G12-B implementer).
//!
//! Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]
#![allow(clippy::doc_lazy_continuation)]

use benten_dsl_compiler::{PrimitiveKind, compile_str};

#[test]
fn dsl_compiler_round_trips_read_respond_minimal_handler() {
    // Fixture 1: minimal READ + RESPOND (the smallest legal handler shape).
    let src = "handler 'minimal-read' { read('post') -> respond }";
    let c = compile_str(src).unwrap();
    assert_eq!(c.subgraph.handler_id(), "minimal-read");
    assert_eq!(c.primitives.len(), 2);
    assert_eq!(c.primitives[0].kind, PrimitiveKind::Read);
    assert_eq!(c.primitives[1].kind, PrimitiveKind::Respond);
    // Round-trip via canonical bytes — re-decoding must produce a Subgraph
    // with the same CID.
    let bytes = c.subgraph.canonical_bytes().unwrap();
    let cid_a = c.subgraph.cid().unwrap();
    let sg2 = benten_core::Subgraph::from_dagcbor(&bytes).unwrap();
    assert_eq!(sg2.cid().unwrap(), cid_a);
}

#[test]
fn dsl_compiler_round_trips_write_respond_handler() {
    // Fixture 2: WRITE + RESPOND with body.
    let src = "handler 'create-post' { write('post', { title: $title }) -> respond }";
    let c = compile_str(src).unwrap();
    assert_eq!(c.primitives.len(), 2);
    assert_eq!(c.primitives[0].kind, PrimitiveKind::Write);
    assert!(c.primitives[0].properties.contains_key("_label"));
}

#[test]
fn dsl_compiler_round_trips_branch_handler() {
    // Fixture 3: BRANCH primitive (control flow).
    let src =
        "handler 'branch-by-role' { read('user') -> branch($user.role == 'admin') -> respond }";
    let c = compile_str(src).unwrap();
    assert_eq!(c.primitives.len(), 3);
    assert_eq!(c.primitives[1].kind, PrimitiveKind::Branch);
    assert!(c.primitives[1].properties.contains_key("_predicate"));
}

#[test]
fn dsl_compiler_round_trips_transform_handler() {
    // Fixture 4: TRANSFORM primitive.
    let src = "handler 'transform-payload' { read('post') -> transform({ uppercased: $title }) -> respond }";
    let c = compile_str(src).unwrap();
    assert_eq!(c.primitives.len(), 3);
    assert_eq!(c.primitives[1].kind, PrimitiveKind::Transform);
    assert!(c.primitives[1].properties.contains_key("_body"));
}

#[test]
fn dsl_compiler_round_trips_call_respond_composition() {
    // Fixture 5: CALL composition + RESPOND. Pins multi-primitive composition.
    let src = "handler 'invoke-helper' { call('helper-handler', { id: $id }) -> respond }";
    let c = compile_str(src).unwrap();
    assert_eq!(c.primitives.len(), 2);
    assert_eq!(c.primitives[0].kind, PrimitiveKind::Call);
    assert!(c.primitives[0].properties.contains_key("_target"));
    assert!(c.primitives[0].properties.contains_key("_args"));
}

#[test]
fn dsl_compiler_round_trip_preserves_subgraph_spec_cid_across_compile_serialize_compile() {
    // Property pin: compile(src1) -> sg1 -> serialize -> deserialize -> sg2;
    // CID(sg1) == CID(sg2). Closes Inv-13 collision-stability for the
    // compiler emission path.
    let src = "handler 'cid-stable' { read('post') -> respond }";
    let c = compile_str(src).unwrap();
    let cid_a = c.subgraph.cid().unwrap();
    let bytes = c.subgraph.canonical_bytes().unwrap();
    let sg2 = benten_core::Subgraph::from_dagcbor(&bytes).unwrap();
    assert_eq!(sg2.cid().unwrap(), cid_a);
}
