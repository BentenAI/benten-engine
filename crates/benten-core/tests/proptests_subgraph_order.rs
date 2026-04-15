//! Proptest: subgraph CID is independent of node-insertion order (R4 triage M16).
//!
//! R1 philosophy: the content-hash of a subgraph depends on its structure
//! (labels, edges, invariants), not on the order in which its Nodes were
//! fed to the builder. Two builders that produce the same final shape must
//! yield the same CID.
//!
//! Phase 1 red-phase: this proptest asserts against `todo!()` bodies via
//! `SubgraphBuilder` and friends that R5 will land. It compiles clean,
//! fails at runtime, and locks in the contract so R5 can't regress it.
//!
//! R3 writer: `rust-test-writer-proptest`.

#![allow(clippy::unwrap_used)]

use proptest::prelude::*;

/// Opaque shorthand for one builder operation — either a label add, a prop
/// add, or an edge add. The strategy below generates a sequence of ops; the
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

proptest! {
    /// R4 triage M16 (R1 philosophy): subgraph CID invariant under op-order
    /// permutations. Two builders that apply the same multiset of ops in
    /// different orders must produce the same content-addressed CID.
    ///
    /// Red-phase: `todo!()` bodies panic before the assertion runs; that's
    /// the TDD contract. R5 lands the builder and this test goes green.
    #[test]
    fn prop_subgraph_cid_order_independent(ops in proptest::collection::vec(any_op(), 1..16)) {
        // Build twice: once in the strategy-supplied order, once reversed.
        let cid_forward = build_and_hash(&ops);
        let mut reversed: Vec<Op> = ops.clone();
        reversed.reverse();
        let cid_reversed = build_and_hash(&reversed);
        prop_assert_eq!(
            cid_forward, cid_reversed,
            "subgraph CID must be invariant under op-order permutations"
        );
    }
}

/// Phase 1 red-phase shim. R5 replaces the body with a real
/// `SubgraphBuilder` construction path.
fn build_and_hash(_ops: &[Op]) -> String {
    todo!("build_and_hash: R5 must wire SubgraphBuilder + canonical_bytes + cid path")
}
