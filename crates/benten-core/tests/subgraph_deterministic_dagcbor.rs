//! R3 unit tests for C5: `Subgraph` DAG-CBOR schema carries the `deterministic`
//! field, and the field participates in the content hash (CID).
//!
//! TDD red-phase (un-skipped from `.pending-g5a` per qa-r4-02 R3-followup):
//! these tests reference the migrated `benten_core::Subgraph` /
//! `SubgraphBuilder` post-G12-C. Until G12-C lands, the file is gated behind
//! `phase_2b_landed`; once landed, the gate flips on and the tests transition
//! from compile-skip to runtime red-phase.
//!
//! Owner: R5 G12-C (R2 landscape §2.1 C5; qa-r4-02 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_core::{Subgraph, SubgraphBuilder};

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
#[ignore = "R5 G12-C red-phase: Subgraph migration to benten-core not yet landed"]
fn subgraph_dagcbor_roundtrip_preserves_deterministic_field() {
    let mut b = SubgraphBuilder::new("det-roundtrip");
    let r = b.read("r");
    b.respond(r);
    b.declare_deterministic(true);
    let sg = b.build_validated().expect("valid subgraph");

    assert!(
        sg.is_declared_deterministic(),
        "pre-encode: deterministic=true must be reflected in accessor"
    );

    // DAG-CBOR round-trip. Requires Subgraph to expose to_dagcbor / from_dagcbor
    // (or canonical_bytes + load_verified) that preserve the flag.
    let bytes = sg.to_dagcbor().expect("encode");
    let decoded: Subgraph = Subgraph::from_dagcbor(&bytes).expect("decode");

    assert!(
        decoded.is_declared_deterministic(),
        "deterministic=true must round-trip through DAG-CBOR"
    );
    assert_eq!(decoded.handler_id(), sg.handler_id());
    assert_eq!(decoded.nodes().len(), sg.nodes().len());
}

#[test]
#[ignore = "R5 G12-C red-phase: Subgraph migration to benten-core not yet landed"]
fn subgraph_cid_differs_when_deterministic_flag_differs() {
    let mut b_true = SubgraphBuilder::new("det-cid-diff");
    let r1 = b_true.read("r");
    b_true.respond(r1);
    b_true.declare_deterministic(true);
    let sg_true = b_true.build_validated().expect("valid true");

    let mut b_false = SubgraphBuilder::new("det-cid-diff");
    let r2 = b_false.read("r");
    b_false.respond(r2);
    b_false.declare_deterministic(false);
    let sg_false = b_false.build_validated().expect("valid false");

    let cid_true = sg_true.cid().expect("cid true");
    let cid_false = sg_false.cid().expect("cid false");

    assert_ne!(
        cid_true, cid_false,
        "deterministic=true vs deterministic=false must produce different CIDs (C5 contract)"
    );
}
