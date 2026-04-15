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
    // Phase 1 red-phase shim. R5 G1-B lands the real constructor.
    todo!(
        "make_edge: R5 must wire Edge::new(source={source:?}, target={target:?}, label={label:?})"
    )
}
