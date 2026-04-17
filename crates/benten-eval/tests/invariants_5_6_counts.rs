//! Edge-case tests for ENGINE-SPEC §4 Invariants 5 and 6 (node-count / edge-count).
//!
//! Covers error codes:
//! - `E_INV_TOO_MANY_NODES` — subgraph has more Nodes than the configured max
//!   (default 4096).
//! - `E_INV_TOO_MANY_EDGES` — subgraph has more Edges than the configured max
//!   (default 8192).
//!
//! These are the "the API honestly said no: this subgraph is too big" edges.
//! Adversarial oversized-subgraph tests (attempted DoS) are security's lane.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes of diagnostic context per R1 triage; test helpers mirror the crate-internal API. Public engine surface boxes it."
)]

use benten_eval::{ErrorCode, Subgraph, SubgraphBuilder};

/// Build a fan-out subgraph of exactly `n` nodes (1 root + n-1 leaves).
/// Edge count = n - 1. Fan-out = n - 1 (one branch point).
///
/// To hit the node-count cap without blowing the fan-out cap, we chain
/// the leaves with a small fan-out at each level.
fn subgraph_with_exactly_n_nodes(n: usize) -> Result<Subgraph, benten_eval::RegistrationError> {
    let mut sb = SubgraphBuilder::new(&format!("n_nodes_{n}"));

    // Build a balanced binary tree so fan-out stays at 2. Total nodes = n.
    let root = sb.read("root");
    let mut frontier = vec![root];
    let mut total = 1;

    while total < n {
        let mut next_frontier = Vec::new();
        for node in frontier {
            if total >= n {
                break;
            }
            let child1 = sb.transform(node, "$input");
            total += 1;
            if total >= n {
                break;
            }
            let child2 = sb.transform(node, "$input");
            total += 1;
            next_frontier.push(child1);
            next_frontier.push(child2);
        }
        frontier = next_frontier;
    }
    sb.build_validated()
}

#[test]
fn accepts_nodes_at_limit() {
    let cap = benten_eval::limits::DEFAULT_MAX_NODES;
    let _sg = subgraph_with_exactly_n_nodes(cap).expect("exactly-cap node count must be accepted");
}

#[test]
fn rejects_too_many_nodes() {
    let cap = benten_eval::limits::DEFAULT_MAX_NODES;
    let err =
        subgraph_with_exactly_n_nodes(cap + 1).expect_err("node count cap + 1 must be rejected");
    assert_eq!(err.code(), ErrorCode::InvTooManyNodes);

    assert_eq!(err.nodes_actual().unwrap(), cap + 1);
    assert_eq!(err.nodes_max().unwrap(), cap);
}

#[test]
fn rejects_too_many_edges() {
    // Construct a subgraph where edge count exceeds the edge cap but
    // node count is well within the node cap. This forces the checker
    // to report E_INV_TOO_MANY_EDGES specifically, not substitute node-cap.
    let edge_cap = benten_eval::limits::DEFAULT_MAX_EDGES;
    let mut sb = SubgraphBuilder::new("edge_bomb");

    let root = sb.read("root");
    // Create nodes_needed leaves all pointing back to root via EMIT edges.
    // Each EMIT is one edge, so to produce edge_cap+1 edges we need
    // edge_cap+1 EMIT nodes. But fan-out from root would then violate
    // Invariant 3 first. So we tier: root -> hub_1 -> 200 leaves;
    // -> hub_2 -> 200 leaves ...
    //
    // Simpler: use the builder's internal cross-edge API. Actual topology
    // doesn't matter for this test — what matters is edge-count-cap is
    // the firing invariant.
    sb.force_add_cross_edges_for_testing(edge_cap + 1);

    let err = sb
        .build_validated()
        .expect_err("edge count cap + 1 must be rejected");
    assert_eq!(err.code(), ErrorCode::InvTooManyEdges);

    assert_eq!(err.edges_actual().unwrap(), edge_cap + 1);
    assert_eq!(err.edges_max().unwrap(), edge_cap);
}

// R4 triage (m9): explicit `_one_over_with_actual_field` tests for the
// diagnostic accessors. Duplicates assertions from `rejects_too_many_nodes`
// / `rejects_too_many_edges` but under the named grep target; per R2
// landscape the per-invariant diagnostic-accessor surface wants a dedicated
// test name for each code so future critics can match test-name → error-code
// → accessor triples mechanically.

#[test]
fn invariant_5_nodes_rejects_one_over_with_actual_field() {
    let cap = benten_eval::limits::DEFAULT_MAX_NODES;
    let err = subgraph_with_exactly_n_nodes(cap + 1).expect_err("rejection");
    assert_eq!(err.code(), ErrorCode::InvTooManyNodes);
    assert_eq!(err.nodes_actual().unwrap(), cap + 1);
    assert_eq!(err.nodes_max().unwrap(), cap);
}

#[test]
fn invariant_6_edges_rejects_one_over_with_actual_field() {
    let edge_cap = benten_eval::limits::DEFAULT_MAX_EDGES;
    let mut sb = SubgraphBuilder::new("edge_bomb_m9");
    let _root = sb.read("root");
    sb.force_add_cross_edges_for_testing(edge_cap + 1);
    let err = sb.build_validated().expect_err("rejection");
    assert_eq!(err.code(), ErrorCode::InvTooManyEdges);
    assert_eq!(err.edges_actual().unwrap(), edge_cap + 1);
    assert_eq!(err.edges_max().unwrap(), edge_cap);
}

#[test]
fn single_node_subgraph_passes_node_and_edge_caps() {
    // Boundary: the minimum subgraph (1 node, 0 edges) passes both caps.
    // Defends against an off-by-one in the checker that would reject
    // "too small" subgraphs.
    let mut sb = SubgraphBuilder::new("minimal");
    let r = sb.read("r");
    sb.respond(r);
    let _ = sb.build_validated().expect("minimal subgraph must pass");
}
