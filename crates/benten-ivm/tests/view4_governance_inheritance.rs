//! Edge-case tests for View 4: governance inheritance with depth-cap edge.
//!
//! Per §2.3 I6: governance inheritance maintains effective-rules transitive
//! closure. Depth cap = 5 hops (ENGINE-SPEC §8). Inputs at exactly 5 hops
//! succeed; inputs at 6 hops must either truncate cleanly or route to an
//! error — the View must not silently recurse forever.
//!
//! This is the "API honestly said no at the configured limit" edge; happy-path
//! governance-inheritance at depths 1-4 is rust-test-writer-unit territory.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_ivm::View;
use benten_ivm::views::governance_inheritance::{GovernanceInheritanceView, MAX_GOVERNANCE_DEPTH};

extern crate alloc;
use alloc::collections::BTreeMap;

fn governed_node(name: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("name".into(), Value::text(name));
    Node::new(vec!["Community".into()], props)
}

#[test]
fn governance_depth_exactly_max_resolved_cleanly() {
    // Build a chain of exactly MAX_GOVERNANCE_DEPTH communities,
    // each GovernedBy the previous. The final community's
    // effective-rules resolution must succeed at depth == MAX.
    assert_eq!(
        MAX_GOVERNANCE_DEPTH, 5,
        "ENGINE-SPEC §8 pins the depth cap at 5 for Phase 1"
    );

    let mut view = GovernanceInheritanceView::new();

    let chain: Vec<_> = (0..MAX_GOVERNANCE_DEPTH)
        .map(|i| governed_node(&format!("c{i}")).cid().unwrap())
        .collect();

    // Link c_{i} -GovernedBy-> c_{i+1} for i in 0..MAX-1.
    for pair in chain.windows(2) {
        view.add_edge(&pair[0], &pair[1]);
    }

    let leaf = &chain[0];
    let effective = view.effective_rules(leaf);
    assert_eq!(
        effective.depth(),
        MAX_GOVERNANCE_DEPTH - 1,
        "exactly-at-cap chain must resolve without truncation"
    );
}

#[test]
fn governance_depth_exceeding_max_truncates_at_cap() {
    // Chain of MAX + 2 communities. View must truncate at MAX, not recurse
    // beyond it. No panic, no stack overflow, no infinite loop.
    let mut view = GovernanceInheritanceView::new();

    let chain: Vec<_> = (0..MAX_GOVERNANCE_DEPTH + 2)
        .map(|i| governed_node(&format!("c{i}")).cid().unwrap())
        .collect();

    for pair in chain.windows(2) {
        view.add_edge(&pair[0], &pair[1]);
    }

    let leaf = &chain[0];
    let effective = view.effective_rules(leaf);
    assert!(
        effective.depth() <= MAX_GOVERNANCE_DEPTH,
        "depth resolution must cap at MAX, got depth={}",
        effective.depth()
    );
    assert!(
        effective.was_truncated(),
        "depth-exceeded case must set the truncation flag so callers can detect it"
    );
}

#[test]
fn governance_cycle_does_not_infinite_loop() {
    // Degenerate input: a cycle in the governance graph (c0 -> c1 -> c0).
    // Cycles are rejected at registration for operation subgraphs (Invariant 1),
    // but governance is data, not code — cycles can happen via sync merge.
    // The View must bail cleanly, not infinite-loop.
    let mut view = GovernanceInheritanceView::new();
    let c0 = governed_node("c0").cid().unwrap();
    let c1 = governed_node("c1").cid().unwrap();
    view.add_edge(&c0, &c1);
    view.add_edge(&c1, &c0);

    let effective = view.effective_rules(&c0);
    assert!(
        effective.was_truncated(),
        "cycle must trigger the same truncation flag as depth-exceeded — the detection path is shared"
    );
    assert!(
        effective.depth() <= MAX_GOVERNANCE_DEPTH,
        "cycle must not escape the depth cap"
    );
}

#[test]
fn governance_empty_graph_returns_empty_rules() {
    // Boundary: node with no GovernedBy edge. Effective rules = empty set,
    // depth = 0. Not an error — the absence of governance is data.
    let view = GovernanceInheritanceView::new();
    let orphan = governed_node("orphan").cid().unwrap();

    let effective = view.effective_rules(&orphan);
    assert_eq!(effective.depth(), 0);
    assert!(effective.rules().is_empty());
    assert!(!effective.was_truncated());
}
