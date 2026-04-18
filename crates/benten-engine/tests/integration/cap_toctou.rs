//! Phase 1 R3 integration — Capability revoked mid-evaluation.
//!
//! Start a long-running ITERATE handler (300 iterations) with capability
//! re-check at each 100-iter batch boundary. Revoke the capability after
//! batch 1 completes (iter ~150). Assert: iter 200+ fails with
//! E_CAP_REVOKED_MID_EVAL; iter 149 does NOT fail (bounds the TOCTOU window).
//!
//! This is the integration-level partner to
//! `crates/benten-caps/tests/toctou_iteration.rs` (unit-level). It proves the
//! hook is called across batch boundaries by the evaluator and the error
//! surfaces correctly via ON_DENIED.
//!
//! **Status:** FAILING until G3 + G4 + G6-B ITERATE land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;

#[test]
#[ignore = "TODO(phase-2-grant-backed-policy): capability_policy_grant_backed() builder hook is a Phase-1 no-op (returns self); call_with_revocation_at depends on Phase-2 grant-backed policy."]
fn capability_revoked_mid_eval_surfaces_at_batch_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Long-running iterate subgraph that writes a Node per iteration.
    let sg = benten_engine::SubgraphSpec::builder()
        .handler_id("long:iter")
        .iterate(300, |body| {
            body.write(|w| {
                w.label("post")
                    .requires("store:post:write")
                    .property("n", Value::Int(0))
            })
        })
        .respond()
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();

    let actor = engine.create_principal("alice").unwrap();
    engine.grant_capability(&actor, "store:post:write").unwrap();

    // Run with a revocation hook: after iter 150 completes, caller revokes.
    let outcome = engine
        .call_with_revocation_at(
            &handler_id,
            "long:iter",
            Node::empty(),
            &actor,
            "store:post:write",
            150,
        )
        .unwrap();

    assert!(outcome.routed_through_edge("ON_DENIED"));
    assert_eq!(outcome.error_code(), Some("E_CAP_REVOKED_MID_EVAL"));

    // TOCTOU window: iter 149 completed before revoke was observed.
    let completed = outcome
        .completed_iterations()
        .expect("iterate exposes this");
    assert!(
        completed >= 149 && completed < 250,
        "batch-boundary window expected 149..250; got {completed}"
    );
}
