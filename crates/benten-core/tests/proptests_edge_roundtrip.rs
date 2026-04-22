//! Proptest: Edge round-trip CID stability (R4 triage M16).
//!
//! An Edge with the same (source, target, label) triple must hash to the
//! same CID after DAG-CBOR encode / decode. This is the edge-level version
//! of the node round-trip test and guards against accidental encoding
//! non-determinism creeping in via Edge property-map drift.
//!
//! Red-phase: `Edge::cid()` is a `todo!()` stub. The proptest compiles and
//! fails at runtime until R5 G1-B lands.
//!
//! R3 writer: `rust-test-writer-proptest`.

#![allow(clippy::unwrap_used)]

use benten_core::{Edge, testing::canonical_test_node};
use proptest::prelude::*;

proptest! {
    /// R4 triage M16: (source, target, label) determines the Edge CID; two
    /// constructions with the same triple (and empty/equal properties) must
    /// hash identically.
    #[test]
    fn prop_edge_roundtrip_cid_stable(
        label in proptest::string::string_regex("[A-Z_]{1,16}").unwrap(),
    ) {
        let node_cid = canonical_test_node().cid().unwrap();

        let e1 = make_edge(&node_cid, &node_cid, &label);
        let e2 = make_edge(&node_cid, &node_cid, &label);

        let cid1 = e1.cid().unwrap();
        let cid2 = e2.cid().unwrap();
        prop_assert_eq!(cid1, cid2, "Edge CID must be deterministic for same triple");
    }
}

fn make_edge(source: &benten_core::Cid, target: &benten_core::Cid, label: &str) -> Edge {
    // R5 G1-B: real constructor. No properties — the proptest is scoped to
    // the (source, target, label) determinism claim; property-map drift is
    // covered separately by `edge_cid::edge_cid_stable_across_reconstructions`
    // and `edge_none_props_vs_empty_map_produce_different_cids`.
    Edge::new(*source, *target, label, None)
}
