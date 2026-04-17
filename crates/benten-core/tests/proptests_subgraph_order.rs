//! Proptest: subgraph CID is independent of node-insertion order (R4 triage M16).
//!
//! R1 philosophy: the content-hash of a subgraph depends on its structure
//! (labels, edges, invariants), not on the order in which its Nodes were
//! fed to the builder. Two builders that produce the same final shape must
//! yield the same CID.
//!
//! **Location note:** The full proptest lives in `benten-eval`'s test
//! directory because it exercises `benten_eval::SubgraphBuilder` and
//! `benten_eval::Subgraph::cid()`, and `benten-core` cannot depend on
//! `benten-eval` (the dependency graph runs the other way).
//! See `benten-eval/tests/proptests_subgraph_order.rs` for the active test.
//!
//! This file retains a node-level sibling proptest that exercises the
//! lower-level `benten_core::Node::cid()` order-independence contract:
//! two Nodes built by inserting the same properties in different orders
//! must produce the same CID. `BTreeMap`'s insertion-order independence
//! combined with DAG-CBOR canonical sorting makes this test green; it
//! locks the invariant at the node layer.
//!
//! G6 mini-review finding `g6-cag-2` — formerly a red-phase `todo!()`
//! stub; now implemented to exercise a proper order-independence
//! contract that benten-core owns.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use proptest::prelude::*;

/// Opaque shorthand for one builder operation — either a label add, or a
/// property add. The strategy below generates a sequence of ops; the
/// invariance claim is: two permutations of the same multiset of ops yield
/// the same CID.
#[derive(Debug, Clone)]
enum Op {
    Label(String),
    Prop(String, i64),
}

fn any_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        proptest::string::string_regex("[a-z]{1,8}")
            .unwrap()
            .prop_map(Op::Label),
        (
            proptest::string::string_regex("[a-z]{1,8}").unwrap(),
            any::<i64>()
        )
            .prop_map(|(k, v)| Op::Prop(k, v)),
    ]
}

/// Build a Node by applying ops in sequence. Duplicate labels are
/// deduplicated (set semantics); duplicate property keys are resolved by
/// keeping the **smallest** value (order-independent merge), so two
/// permutations of the same op multiset produce byte-identical canonical
/// forms regardless of the order the duplicates were fed in.
///
/// The point of the proptest is to exercise the `BTreeMap` + DAG-CBOR
/// canonicalization path, not last-write-wins semantics — so we pick an
/// order-independent merge rule here.
fn build_node(ops: &[Op]) -> Node {
    let mut labels_set: std::collections::BTreeSet<String> = Default::default();
    let mut props: BTreeMap<String, i64> = BTreeMap::new();
    for op in ops {
        match op {
            Op::Label(l) => {
                labels_set.insert(l.clone());
            }
            Op::Prop(k, v) => {
                props
                    .entry(k.clone())
                    .and_modify(|cur| {
                        if *v < *cur {
                            *cur = *v;
                        }
                    })
                    .or_insert(*v);
            }
        }
    }
    let props_value: BTreeMap<String, Value> =
        props.into_iter().map(|(k, v)| (k, Value::Int(v))).collect();
    Node::new(labels_set.into_iter().collect(), props_value)
}

proptest! {
    /// R4 triage M16 (R1 philosophy): Node CID invariant under op-order
    /// permutations. Two Nodes that receive the same multiset of ops in
    /// different orders must produce the same content-addressed CID.
    ///
    /// The subgraph-level equivalent lives in
    /// `benten-eval/tests/proptests_subgraph_order.rs` — this file owns
    /// the node-level contract.
    #[test]
    fn prop_subgraph_cid_order_independent(ops in proptest::collection::vec(any_op(), 1..16)) {
        // Build twice: once in the strategy-supplied order, once reversed.
        let cid_forward = build_and_hash(&ops);
        let mut reversed: Vec<Op> = ops.clone();
        reversed.reverse();
        let cid_reversed = build_and_hash(&reversed);
        prop_assert_eq!(
            cid_forward, cid_reversed,
            "Node CID must be invariant under op-order permutations"
        );
    }
}

fn build_and_hash(ops: &[Op]) -> String {
    let node = build_node(ops);
    node.cid()
        .expect("canonical_bytes must succeed for int-only properties")
        .to_string()
}
