//! Edge-case tests for ENGINE-SPEC §4 Invariant 2 (max depth).
//!
//! Covers error code:
//! - `E_INV_DEPTH_EXCEEDED` — operation subgraph nesting exceeds the
//!   configured depth cap (default per capability grant).
//!
//! R3 contract: `benten-eval`'s invariant checker is stub. R5 (G6-C) lands it.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes of diagnostic context per R1 triage; test helpers mirror the crate-internal API. Public engine surface boxes it."
)]

use benten_eval::{ErrorCode, RegistrationError, Subgraph, SubgraphBuilder};

fn linear_chain_of_calls(n: usize) -> Result<Subgraph, RegistrationError> {
    // Chain of `n` nested CALL primitives. Each CALL counts as +1 depth.
    let mut sb = SubgraphBuilder::new(&format!("chain_{n}"));
    let root = sb.read("root");
    let mut prev = root;
    for _ in 0..n {
        prev = sb.call(prev, "inner_handler");
    }
    sb.respond(prev);
    sb.build_validated()
}

#[test]
fn accepts_depth_at_limit() {
    // At exactly the limit, the subgraph is legal.
    let cap = benten_eval::limits::DEFAULT_MAX_DEPTH;
    let _sg = linear_chain_of_calls(cap).expect("at-limit depth must be accepted");
}

#[test]
fn rejects_depth_exceeded() {
    // One past the limit must fail.
    let cap = benten_eval::limits::DEFAULT_MAX_DEPTH;
    let err = linear_chain_of_calls(cap + 1).expect_err("depth > cap must be rejected");
    assert_eq!(err.code(), ErrorCode::InvDepthExceeded);

    // Context: actual, max, longest_path
    assert_eq!(err.depth_actual().unwrap(), cap + 1);
    assert_eq!(err.depth_max().unwrap(), cap);
    assert!(
        !err.longest_path()
            .expect("longest_path context must be set")
            .is_empty(),
        "E_INV_DEPTH_EXCEEDED must include the offending path"
    );
}

#[test]
fn rejects_depth_far_exceeded() {
    // A massively over-limit subgraph must still be rejected cleanly (not
    // stack-overflow the checker itself). Test a far-over case.
    let cap = benten_eval::limits::DEFAULT_MAX_DEPTH;
    let err = linear_chain_of_calls(cap * 3).expect_err("far-over depth must be rejected");
    assert_eq!(err.code(), ErrorCode::InvDepthExceeded);
}

#[test]
fn depth_one_always_accepted() {
    // Boundary: a single-node subgraph has depth 1 (or 0, depending on
    // the counting scheme). Either way it must be accepted — it cannot
    // possibly exceed any positive cap.
    let mut sb = SubgraphBuilder::new("single");
    let r = sb.read("r");
    sb.respond(r);
    let _sg = sb.build_validated().expect("depth-1 subgraph must pass");
}

#[test]
fn depth_cap_configurable_via_capability_grant() {
    // The depth cap is configurable per capability grant. A lower cap
    // must reject subgraphs that the default would accept.
    let lower_cap = 3;
    let mut sb = SubgraphBuilder::new("would_pass_default");
    let root = sb.read("root");
    let mut prev = root;
    for _ in 0..5 {
        prev = sb.call(prev, "inner");
    }
    sb.respond(prev);

    // With default cap: accepts. With lower_cap: rejects.
    let err = sb
        .build_validated_with_max_depth(lower_cap)
        .expect_err("depth 5 must exceed configured cap of 3");
    assert_eq!(err.code(), ErrorCode::InvDepthExceeded);
    assert_eq!(err.depth_max().unwrap(), lower_cap);
}
