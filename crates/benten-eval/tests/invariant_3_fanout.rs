//! Edge-case tests for ENGINE-SPEC §4 Invariant 3 (max fan-out per node).
//!
//! Covers error code:
//! - `E_INV_FANOUT_EXCEEDED` — a single Node has more outgoing edges than
//!   the configured max fan-out.
//!
//! Typical cases this catches: a BRANCH with dozens of cases (should be a
//! match-table). An ITERATE that forks too many parallel child subgraphs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{ErrorCode, Subgraph, SubgraphBuilder};

/// Build a BRANCH with `n` outgoing cases.
fn branch_with_cases(n: usize) -> Result<Subgraph, benten_eval::RegistrationError> {
    let mut sb = SubgraphBuilder::new(&format!("branch_{n}"));
    let root = sb.read("root");
    let branch = sb.branch(root, "$input");

    // Each additional edge from `branch` increments the fan-out.
    for i in 0..n {
        let arm = sb.transform(branch, &format!("'case{i}'"));
        sb.respond(arm);
    }
    sb.build_validated()
}

#[test]
fn accepts_fanout_at_limit() {
    let cap = benten_eval::limits::DEFAULT_MAX_FANOUT;
    let _sg = branch_with_cases(cap).expect("at-limit fan-out must be accepted");
}

#[test]
fn rejects_fanout() {
    let cap = benten_eval::limits::DEFAULT_MAX_FANOUT;
    let err = branch_with_cases(cap + 1).expect_err("fan-out > cap must be rejected");
    assert_eq!(err.code(), ErrorCode::InvFanoutExceeded);

    assert_eq!(err.fanout_actual().unwrap(), cap + 1);
    assert_eq!(err.fanout_max().unwrap(), cap);
    assert!(
        err.fanout_node_id().is_some(),
        "E_INV_FANOUT_EXCEEDED must name the offending node"
    );
}

#[test]
fn rejects_zero_fan_in_single_node_bomb() {
    // Adversarial-adjacent (but "honest no" shaped): a WRITE with
    // 300 outgoing EMIT edges is legal semantically but violates the
    // per-node fan-out cap. The API honestly says "too many fan-outs
    // from a single node — restructure."
    let cap = benten_eval::limits::DEFAULT_MAX_FANOUT;
    let mut sb = SubgraphBuilder::new("emit_bomb");
    let root = sb.read("root");
    for _ in 0..(cap + 5) {
        sb.emit(root, "some:event");
    }
    let err = sb
        .build_validated()
        .expect_err("EMIT fan-out bomb must be rejected");
    assert_eq!(err.code(), ErrorCode::InvFanoutExceeded);
}

#[test]
fn fanout_per_node_not_subgraph_total() {
    // Sharp boundary: the fan-out cap is per-node, not subgraph-total.
    // Two BRANCHes each with cap-size cases should both pass. If R5
    // accidentally measures subgraph-wide outgoing-edge count, this
    // test trips.
    let cap = benten_eval::limits::DEFAULT_MAX_FANOUT;
    let mut sb = SubgraphBuilder::new("two_branches_each_at_cap");

    let root = sb.read("root");
    let b1 = sb.branch(root, "$input");
    let b2 = sb.branch(root, "$input");
    for _ in 0..cap {
        sb.transform(b1, "$input");
    }
    for _ in 0..cap {
        sb.transform(b2, "$input");
    }
    let _sg = sb
        .build_validated()
        .expect("two branches each at cap must both pass");
}

#[test]
fn fanout_applies_to_iterate_parallel_forks() {
    // ITERATE with `parallel: N` where N > cap would fork N child
    // evaluations from a single source Node. Rejected.
    let cap = benten_eval::limits::DEFAULT_MAX_FANOUT;
    let mut sb = SubgraphBuilder::new("iterate_parallel_over_cap");
    let root = sb.read("list_source");
    let _iter = sb.iterate_parallel(root, "body_handler", cap + 1);
    let err = sb
        .build_validated()
        .expect_err("iterate parallel fan-out over cap must be rejected");
    assert_eq!(err.code(), ErrorCode::InvFanoutExceeded);
}
