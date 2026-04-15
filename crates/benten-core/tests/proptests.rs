//! Property-based tests for benten-core (R2 landscape §3 harnesses).
//!
//! The R3 cadence is `proptest!` with reduced case counts (≥ 64) so the test
//! suite stays fast at TDD time. Phase 1 G1-A will raise the count to the
//! landscape targets (100k for `prop_node_roundtrip_cid_stable`) in CI-gated
//! proptest runs via the `PROPTEST_CASES` env var.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Strategy for an arbitrary text property key (short, ASCII, non-empty).
fn arb_key() -> impl Strategy<Value = String> {
    "[a-z]{1,8}".prop_map(|s| s.to_string())
}

/// Strategy for a leaf Value — no nested collections.
fn arb_leaf_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        "[a-zA-Z0-9 ]{0,16}".prop_map(Value::text),
    ]
}

/// Strategy for a Node with 0-4 leaf-valued properties.
fn arb_node() -> impl Strategy<Value = Node> {
    let labels = prop::collection::vec("[A-Z][a-z]{0,8}".prop_map(|s| s.to_string()), 0..3);
    let props = prop::collection::btree_map(arb_key(), arb_leaf_value(), 0..4);
    (labels, props).prop_map(|(labels, properties)| Node::new(labels, properties))
}

proptest! {
    /// `prop_node_roundtrip_cid_stable`: encoding → decoding → re-hashing a
    /// random Node produces a byte-identical CID (R2 landscape §3 row 1).
    #[test]
    fn prop_node_roundtrip_cid_stable(node in arb_node()) {
        let cid_before = node.cid().unwrap();
        let bytes = node.canonical_bytes().unwrap();
        let decoded: Node = serde_ipld_dagcbor::from_slice(&bytes).unwrap();
        let cid_after = decoded.cid().unwrap();
        prop_assert_eq!(cid_before, cid_after);
    }

    /// `prop_node_anchor_id_excluded_from_cid`: setting any anchor_id must
    /// never change the Node CID (R2 landscape §3 row 13).
    #[test]
    fn prop_node_anchor_id_excluded_from_cid(node in arb_node(), anchor in any::<u64>()) {
        let baseline = node.cid().unwrap();
        let mut with_anchor = node.clone();
        with_anchor.anchor_id = Some(anchor);
        prop_assert_eq!(baseline, with_anchor.cid().unwrap());
    }

    /// `prop_value_float_bits_stable`: non-NaN, finite floats round-trip
    /// through DAG-CBOR with a stable CID (R2 landscape §3 row 3). Fuzzes
    /// 64-bit bit patterns; filters NaN and ±Inf per the C3 rejection rules.
    #[test]
    fn prop_value_float_bits_stable(bits in any::<u64>()) {
        let f = f64::from_bits(bits);
        prop_assume!(f.is_finite());
        prop_assume!(!f.is_nan());
        let mut p = BTreeMap::new();
        p.insert("f".to_string(), Value::Float(f));
        let n = Node::new(vec!["T".to_string()], p);
        let cid_before = n.cid().unwrap();
        let bytes = n.canonical_bytes().unwrap();
        let decoded: Node = serde_ipld_dagcbor::from_slice(&bytes).unwrap();
        prop_assert_eq!(cid_before, decoded.cid().unwrap());
    }

    /// `prop_version_chain_linearizable` — walk_versions produces a total
    /// order compatible with the NEXT_VERSION DAG (R2 landscape §3 row 5).
    #[test]
    fn prop_version_chain_linearizable(len in 1usize..8) {
        use benten_core::{append_version, walk_versions, Anchor};
        let anchor = Anchor::new();
        let mut expected = Vec::new();
        for i in 0..len {
            let mut p = BTreeMap::new();
            p.insert("seq".to_string(), Value::Int(i as i64));
            let node = Node::new(vec!["Post".to_string()], p);
            let cid = append_version(&anchor, &node).unwrap();
            expected.push(cid);
        }
        let chain = walk_versions(&anchor).unwrap();
        prop_assert_eq!(chain, expected);
    }
}
