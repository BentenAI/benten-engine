//! Edge-case tests for ENGINE-SPEC §4 Invariant 1 (DAG-ness).
//!
//! Covers error code:
//! - `E_INV_CYCLE` — operation subgraph contains a back-edge; registration rejects.
//!
//! Partners with `invariant_1_cycle` happy-path tests (owned by
//! rust-test-writer-unit, in `tests/invariants_9_10_12.rs` for the positive pair).
//!
//! R3 contract: `benten-eval` is a stub today. R5 (G6-C) lands the invariant
//! checker. These tests compile-fail until then.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError intentionally carries ~360 bytes of diagnostic context per R1 triage; test helpers mirror the crate-internal API. Public engine surface boxes it (see benten-engine::EngineError::Invariant)."
)]

use benten_eval::{ErrorCode, RegistrationError, Subgraph, SubgraphBuilder};

/// Build a 3-node chain A -> B -> C. Valid DAG.
fn chain_dag() -> Subgraph {
    let mut sb = SubgraphBuilder::new("valid_chain");
    let a = sb.read("a");
    let b = sb.transform(a, "$input");
    let _c = sb.respond(b);
    sb.build_validated().unwrap()
}

/// Build a 3-node cycle A -> B -> C -> A (C points back to A).
fn simple_cycle() -> Result<Subgraph, RegistrationError> {
    let mut sb = SubgraphBuilder::new("simple_cycle");
    let a = sb.read("a");
    let b = sb.transform(a, "$input");
    let c = sb.transform(b, "$input");
    sb.add_edge(c, a); // creates back-edge -> cycle
    sb.build_validated()
}

/// Build a self-loop: A -> A.
fn self_loop() -> Result<Subgraph, RegistrationError> {
    let mut sb = SubgraphBuilder::new("self_loop");
    let a = sb.read("a");
    sb.add_edge(a, a);
    sb.build_validated()
}

#[test]
fn rejects_cycle() {
    let err = simple_cycle().expect_err("cycle must be rejected at registration");
    assert_eq!(err.code(), ErrorCode::InvCycle);
    // Error context must include the cycle path so the dev can fix it.
    assert!(
        !err.cycle_path()
            .expect("E_INV_CYCLE must name the cycle path")
            .is_empty(),
        "E_INV_CYCLE must include the cycle path in its context"
    );
}

#[test]
fn accepts_valid_dag() {
    // Positive boundary pair — a chain with no back-edge must pass.
    let _sg = chain_dag();
}

#[test]
fn rejects_self_loop_as_cycle() {
    // Self-loop is the minimal cycle. Must be caught with the same code.
    let err = self_loop().expect_err("self-loop must be rejected as a cycle");
    assert_eq!(err.code(), ErrorCode::InvCycle);
}

#[test]
fn rejects_deeply_buried_cycle() {
    // Chain of 50 nodes with a back-edge from node 49 to node 5. The
    // detector must traverse the full subgraph; a shortcut "check first
    // few nodes only" bug would miss this.
    let mut sb = SubgraphBuilder::new("deep_cycle");
    let mut nodes = Vec::new();
    let root = sb.read("root");
    nodes.push(root);
    for _ in 0..50 {
        let prev = *nodes.last().unwrap();
        nodes.push(sb.transform(prev, "$input"));
    }
    // Back-edge from node 49 (index 50) to node 5.
    sb.add_edge(nodes[50], nodes[5]);

    let err = sb
        .build_validated()
        .expect_err("deep cycle must be rejected");
    assert_eq!(err.code(), ErrorCode::InvCycle);
}

#[test]
fn rejects_cycle_spanning_branch_arms() {
    // A BRANCH whose ON_TRUE arm eventually loops back to the BRANCH is
    // still a cycle (just harder to spot visually). Detector must find it.
    let mut sb = SubgraphBuilder::new("branch_cycle");
    let a = sb.read("a");
    let branch = sb.branch(a, "$input");
    let true_arm = sb.transform(branch, "$input");
    let more = sb.transform(true_arm, "$input");
    sb.add_edge(more, branch); // cycles back

    let err = sb
        .build_validated()
        .expect_err("cycle via branch arm must fail");
    assert_eq!(err.code(), ErrorCode::InvCycle);
}
