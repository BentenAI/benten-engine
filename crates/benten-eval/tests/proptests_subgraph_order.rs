//! Proptest: subgraph CID is independent of builder insertion order.
//!
//! Exercises the `SubgraphBuilder` → `Subgraph::cid()` path which
//! `benten-core` cannot test directly (no reverse dependency on
//! `benten-eval`). The contract being locked here:
//!
//!   Two subgraphs built by inserting the same multiset of primitives and
//!   edges in different orders must produce byte-identical canonical
//!   encodings and therefore the same CID.
//!
//! Invariant 10 (content-addressed hash) requires `canonical_subgraph_bytes`
//! to sort nodes+edges before DAG-CBOR serialization; this proptest fuzzes
//! that sort with random insertion orders.
//!
//! See also: `benten-core/tests/proptests_subgraph_order.rs` for the
//! Node-level sibling proptest. G6 mini-review finding `g6-cag-2`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{OperationNode, PrimitiveKind, Subgraph};
use proptest::prelude::*;

/// A single (node_id, kind) tuple.
#[derive(Debug, Clone)]
struct NodeSpec {
    id: String,
    kind: PrimitiveKind,
}

fn any_kind() -> impl Strategy<Value = PrimitiveKind> {
    // Pick from the Phase-1 executable set so the subgraph stays close to
    // real handler shapes. The CID property holds across all 12 kinds, but
    // restricting the surface keeps the test deterministic and fast.
    prop_oneof![
        Just(PrimitiveKind::Read),
        Just(PrimitiveKind::Write),
        Just(PrimitiveKind::Transform),
        Just(PrimitiveKind::Branch),
        Just(PrimitiveKind::Respond),
        Just(PrimitiveKind::Emit),
    ]
}

fn any_node() -> impl Strategy<Value = NodeSpec> {
    (
        proptest::string::string_regex("[a-z]{1,4}[0-9]{1,2}").unwrap(),
        any_kind(),
    )
        .prop_map(|(id, kind)| NodeSpec { id, kind })
}

/// Build a Subgraph by pushing nodes and edges in the supplied order.
/// Uses the order-independent `Subgraph::with_node` / `with_edge` helpers
/// so inputs are reflected identically into the internal vectors.
fn build(nodes: &[NodeSpec], edges: &[(usize, usize)]) -> Subgraph {
    let mut sg = Subgraph::new("proptest");
    for n in nodes {
        sg = sg.with_node(OperationNode::new(n.id.clone(), n.kind));
    }
    for (i, j) in edges {
        let from = nodes[*i].id.clone();
        let to = nodes[*j].id.clone();
        sg = sg.with_edge(from, to, "next");
    }
    sg
}

proptest! {
    /// The subgraph CID depends only on its structural content (node
    /// multiset + edge multiset), not on the order of builder insertions.
    #[test]
    fn prop_subgraph_cid_order_independent(
        nodes in proptest::collection::vec(any_node(), 2..8),
        edge_idx in proptest::collection::vec((0usize..8, 0usize..8), 0..6),
    ) {
        // Deduplicate node ids so we don't violate the "unique node id"
        // structural assumption — pick the first occurrence of each id.
        let mut seen = std::collections::BTreeSet::new();
        let nodes: Vec<NodeSpec> = nodes
            .into_iter()
            .filter(|n| seen.insert(n.id.clone()))
            .collect();
        prop_assume!(nodes.len() >= 2);

        // Clamp edge indices to the unique-node count.
        let edges: Vec<(usize, usize)> = edge_idx
            .into_iter()
            .map(|(a, b)| (a % nodes.len(), b % nodes.len()))
            .collect();

        let sg_a = build(&nodes, &edges);

        let mut nodes_rev: Vec<NodeSpec> = nodes.clone();
        nodes_rev.reverse();
        // Edges also reversed so the second build is a genuine permutation.
        // Remap edge indices to the reversed node list so each edge points
        // to the same (id, id) pair as in the first build.
        let n = nodes.len();
        let edges_rev: Vec<(usize, usize)> = edges
            .iter()
            .rev()
            .map(|(a, b)| (n - 1 - *a, n - 1 - *b))
            .collect();
        let sg_b = build(&nodes_rev, &edges_rev);

        let cid_a = sg_a.cid().expect("CID for sg_a");
        let cid_b = sg_b.cid().expect("CID for sg_b");
        prop_assert_eq!(
            cid_a.to_string(),
            cid_b.to_string(),
            "subgraph CID must be invariant under insertion-order permutations"
        );
    }
}
