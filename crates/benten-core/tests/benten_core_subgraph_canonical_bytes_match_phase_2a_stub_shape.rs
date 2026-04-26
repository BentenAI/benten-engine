//! G12-C red-phase: assert the migrated `benten_core::Subgraph` produces
//! canonical bytes (and CID) matching the Phase-2a C5 stub for equivalent
//! input — the migration is a strict superset, NOT a re-encoding.
//!
//! Per plan §3.2 G12-C: "Phase-2a C5 stub already pins canonical-bytes shape
//! (BLAKE3-over-DAG-CBOR with `deterministic` field); migrated type is a
//! strict superset."
//!
//! Property pin (Inv-13 collision-stability): if the migration silently
//! re-orders fields or adds a non-cfg-gated field, this test catches the
//! CID drift before downstream `subgraph_cache.rs` keys break.
//!
//! TDD red-phase. Owner: R5 G12-C (qa-r4-02 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_core::{Subgraph, SubgraphBuilder};

/// Pin the canonical bytes of the minimal "read → respond" handler post-migration
/// against the Phase-2a stub canonical bytes.
#[test]
#[ignore = "R5 G12-C red-phase: assert canonical_bytes() matches Phase-2a stub byte-for-byte"]
fn migrated_subgraph_canonical_bytes_match_phase_2a_stub_for_minimal_handler() {
    let mut b = SubgraphBuilder::new("phase-2a-shape-pin");
    let r = b.read("post");
    b.respond(r);
    b.declare_deterministic(true);
    let _sg = b.build_validated().expect("valid subgraph");

    // Phase-2a stub canonical bytes for this exact handler shape — checked-in
    // hex literal (G12-C R5 implementer regenerates via the C5 stub at first
    // landing, then the literal is frozen to detect drift).
    let _expected_phase_2a_stub_bytes_hex: &str = "PHASE_2A_STUB_BYTES_HEX_TBD";

    todo!(
        "R5 G12-C: \
         (1) regenerate Phase-2a stub bytes against the C5 stub branch; \
         (2) freeze the hex literal; \
         (3) assert sg.canonical_bytes() == hex::decode(EXPECTED_HEX)"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: cid stability check across migration not yet wired"]
fn migrated_subgraph_cid_matches_phase_2a_stub_cid_for_minimal_handler() {
    let mut b = SubgraphBuilder::new("phase-2a-cid-pin");
    let r = b.read("post");
    b.respond(r);
    b.declare_deterministic(true);
    let _sg = b.build_validated().expect("valid subgraph");

    let _expected_phase_2a_stub_cid_str: &str = "bafy...PHASE_2A_STUB_CID_TBD";

    todo!(
        "R5 G12-C: \
         (1) regenerate stub CID; (2) freeze the literal; \
         (3) assert sg.cid().expect(\"cid\").to_string() == EXPECTED"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: deterministic-field encoding parity not yet asserted"]
fn migrated_subgraph_deterministic_field_encoded_at_same_dagcbor_position_as_stub() {
    // Pin: the DAG-CBOR encoder MUST place `deterministic` at the same map-key
    // ordering position as the Phase-2a stub. Since DAG-CBOR canonical-bytes
    // sorts map keys lexicographically, this comes for free IF the field name
    // matches; this test pins the field-name == "deterministic" contract.
    let mut b = SubgraphBuilder::new("field-position-pin");
    let r = b.read("post");
    b.respond(r);
    b.declare_deterministic(false);
    let _sg = b.build_validated().expect("valid subgraph");

    todo!(
        "R5 G12-C: decode bytes via raw DAG-CBOR; assert top-level map carries \
         a key named exactly \"deterministic\" (NOT renamed during migration)"
    )
}

/// G12-C non-regression: Subgraph value semantics carry through the migration.
#[test]
#[ignore = "R5 G12-C red-phase: equality semantics across migration not yet asserted"]
fn migrated_subgraph_equality_under_handler_id_normalisation_matches_stub() {
    let mut b1 = SubgraphBuilder::new("eq-pin");
    let r1 = b1.read("post");
    b1.respond(r1);
    let _sg1 = b1.build_validated().expect("valid");

    let mut b2 = SubgraphBuilder::new("eq-pin");
    let r2 = b2.read("post");
    b2.respond(r2);
    let _sg2 = b2.build_validated().expect("valid");

    todo!("R5 G12-C: assert sg1.cid() == sg2.cid() for byte-identical input")
}
