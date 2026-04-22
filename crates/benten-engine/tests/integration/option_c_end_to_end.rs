//! Phase 2a R3 integration — BUNDLED exit gate 4: `crud:post:get` dispatched
//! through `Engine::call` honours symmetric-None end-to-end (Option C
//! evaluator-path) and the same check_read_capability gate threads through
//! all 4 content-returning PrimitiveHost methods.
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §1 exit criterion 4
//! (Option C evaluator-path threaded) + §3 G4-A Option C flanking methods
//! (sec-r1-5 / atk-5) + plan §2.5 P3 (GrantBackedPolicy wired into READ
//! primitive execute path).
//!
//! `crud_post_get_symmetric_none` is the headline sub-test — per plan §1
//! gate 4 and the file-level R2-landscape row 305. Flanking-method tests
//! trace to plan §3 G4-A file-ownership note about threading
//! `check_read_capability` through `read_node` + 3 content-returning
//! methods (`get_by_label`, `get_by_property`, `read_view`). Owned by
//! `qa-expert` per R2 landscape §8.5. TDD red-phase.

#![cfg(feature = "phase_2a_pending_apis")]
// R4 fix-pass: blocked on G4-A landing `register_crud_with_grants` +
// `call_as` dispatch + `testing_insert_user_post` + `Outcome::is_ok_edge` +
// `Outcome::error_code` at the expected shape. See the
// wait_resume_determinism.rs header for the rationale.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

/// Open an engine under `GrantBackedPolicy` with no read-capability grants
/// seeded. A caller with no read grant hits the denied-read path on every
/// content-returning method.
fn engine_with_read_denial() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();
    (dir, engine)
}

/// Write a post under a privileged internal path so the backend has content
/// to deny reads against. Returns the CID of the written Node.
fn seed_denied_post(engine: &Engine) -> benten_core::Cid {
    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("denied_read_fixture".into()));
    engine
        .testing_insert_user_post(Node::new(vec!["post".into()], p))
        .expect("privileged fixture write")
}

/// Headline gate-4 assertion (plan §1 #4): dispatching
/// `engine.call('crud:post:get', { cid })` through the evaluator path
/// returns `Ok(None)` both for a missing CID AND for a cap-denied read.
/// The two paths are symmetric — the caller cannot distinguish them at the
/// public API surface, which is the Option C contract.
#[test]
fn crud_post_get_symmetric_none() {
    let (_dir, engine) = engine_with_read_denial();
    let handler_id = engine
        .register_crud_with_grants("post")
        .expect("grant-backed crud registers");

    // Grant WRITE only — reads remain denied under the grant-backed policy.
    let alice = engine.create_principal("alice").unwrap();
    engine.grant_capability(&alice, "store:post:write").unwrap();

    // Path 1: missing CID — symmetric None.
    let missing_cid = benten_core::Cid::from_blake3_digest(blake3::hash(b"not-in-backend").into());
    let mut missing_input = BTreeMap::new();
    missing_input.insert("cid".into(), Value::Text(missing_cid.to_base32()));
    let missing_outcome = engine
        .call(
            &handler_id,
            "post:get",
            Node::new(vec!["input".into()], missing_input),
        )
        .expect("call succeeds (symmetric None, not an error)");
    assert!(
        missing_outcome.is_ok_edge(),
        "missing CID must return on the OK edge, not the error edge; got {missing_outcome:?}"
    );
    assert!(
        missing_outcome.as_list().map_or(true, |v| v.is_empty()),
        "missing CID must return no items; got {missing_outcome:?}"
    );

    // Path 2: present CID but read denied — symmetric None (the key claim).
    let existing_cid = seed_denied_post(&engine);
    let mut denied_input = BTreeMap::new();
    denied_input.insert("cid".into(), Value::Text(existing_cid.to_base32()));
    let denied_outcome = engine
        .call_as(
            &handler_id,
            "post:get",
            Node::new(vec!["input".into()], denied_input),
            &alice,
        )
        .expect("call succeeds (symmetric None, not an error)");
    assert!(
        denied_outcome.is_ok_edge(),
        "cap-denied read must return on OK (symmetric None), not ON_DENIED (Option C); \
         got {denied_outcome:?}"
    );
    assert!(
        denied_outcome.as_list().map_or(true, |v| v.is_empty()),
        "cap-denied read must return no items; got {denied_outcome:?}"
    );

    // Contract: the two outcomes must be indistinguishable by a public-API
    // caller. (Internal diagnostic paths via `diagnose_read` stay — those
    // are gated on a `debug:read` capability, documented in
    // `docs/SECURITY-POSTURE.md` Option C named compromise #2.)
    assert_eq!(
        missing_outcome.edge_taken(),
        denied_outcome.edge_taken(),
        "symmetric-None contract: missing-CID and cap-denied outcomes must \
         route through the same public-API edge"
    );
    assert_eq!(
        missing_outcome.error_code(),
        denied_outcome.error_code(),
        "neither path surfaces an error code to the public API (Option C)"
    );
}

// ---------------------------------------------------------------------------
// Option C flanking-methods: the same gate threads through 3 other
// content-returning PrimitiveHost methods (sec-r1-5 / atk-5).
// ---------------------------------------------------------------------------

/// `read_node` (the direct READ path) honours check_read_capability.
/// Covers the base case the plan names explicitly (atk-5 base).
///
/// Red-phase: body references R3-consolidated Phase-2a APIs. The engine
/// surfaces are `todo!()`-bodied in the benten-engine source; this test
/// panics at the first `todo!()` carrying the owning group pointer. R5
/// G4-A makes it green. See `.addl/phase-2a/r4-triage.md` tq-1.
#[test]
fn option_c_read_node_respects_check_read() {
    let (_dir, engine) = engine_with_read_denial();
    let alice = engine.create_principal("alice").unwrap();
    engine.grant_capability(&alice, "store:post:write").unwrap();
    let cid = seed_denied_post(&engine);

    // Under the grant-backed policy, a caller without the read grant
    // receives symmetric None (Option C). Exercised via the evaluator's
    // read-capability gate (read_node_with_policy), NOT the raw backend.
    let got = engine
        .read_node_with_policy(&cid)
        .expect("read path is infallible under Option C — denial collapses to None");
    assert!(
        got.is_none(),
        "Option C: cap-denied read_node must return None (symmetric with missing); got {got:?}"
    );
}

/// `get_by_label` (flanking method 1) honours check_read_capability.
/// sec-r1-5 / atk-5: a content-returning list method must also honour
/// the same Option-C denial collapse.
#[test]
fn option_c_get_by_label_respects_check_read() {
    let (_dir, engine) = engine_with_read_denial();
    let alice = engine.create_principal("alice").unwrap();
    engine.grant_capability(&alice, "store:post:write").unwrap();
    let _cid = seed_denied_post(&engine);

    // With a grant-backed policy and no READ grant, the by-label probe
    // must return an empty list — not an error, not a populated list.
    let handler_id = engine
        .register_crud_with_grants("post")
        .expect("crud-with-grants registers");

    let outcome = engine
        .call_as(
            &handler_id,
            "post:list",
            Node::new(vec!["input".into()], BTreeMap::new()),
            &alice,
        )
        .expect("call_as succeeds (Option C: denial collapses to Ok/empty)");
    assert!(outcome.is_ok_edge(), "must route OK; got {outcome:?}");
    assert!(
        outcome.as_list().map_or(true, |v| v.is_empty()),
        "Option C: by-label listing under cap denial must surface empty; got {outcome:?}"
    );
}

/// `get_by_property` (flanking method 2) honours check_read_capability.
#[test]
fn option_c_get_by_property_respects_check_read() {
    let (_dir, engine) = engine_with_read_denial();
    let alice = engine.create_principal("alice").unwrap();
    engine.grant_capability(&alice, "store:post:write").unwrap();
    let _cid = seed_denied_post(&engine);

    // A by-property handler dispatched via the evaluator must also honour
    // the gate and collapse to empty under denial.
    let handler_id = engine
        .register_crud_with_grants("post")
        .expect("crud-with-grants registers");

    let mut input = BTreeMap::new();
    input.insert("title".into(), Value::Text("denied_read_fixture".into()));
    let outcome = engine
        .call_as(
            &handler_id,
            "post:find_by_title",
            Node::new(vec!["input".into()], input),
            &alice,
        )
        .expect("call_as surfaces None/empty under denial, not an error");
    assert!(outcome.is_ok_edge(), "must route OK; got {outcome:?}");
    assert!(
        outcome.as_list().map_or(true, |v| v.is_empty()),
        "Option C: by-property under cap denial must surface empty; got {outcome:?}"
    );
}

/// `read_view` (flanking method 3) honours check_read_capability at the
/// COARSE-GRAINED level per named Compromise #N+2 in SECURITY-POSTURE.md.
/// Per-row gating is Phase 3; in Phase 2a, lacking the view's read scope
/// causes `read_view` to surface empty/None under the Option C contract.
#[test]
fn option_c_read_view_respects_check_read_coarse_grained() {
    let (_dir, engine) = engine_with_read_denial();
    let alice = engine.create_principal("alice").unwrap();
    engine.grant_capability(&alice, "store:post:write").unwrap();
    let _cid = seed_denied_post(&engine);

    // The handler used here dispatches through `read_view` internally and
    // is gated by the same check_read_capability hook. Without the view's
    // read scope the caller sees an empty view (coarse-grained).
    let handler_id = engine
        .register_crud_with_grants("post")
        .expect("crud-with-grants registers");

    let outcome = engine
        .call_as(
            &handler_id,
            "post:list_via_view",
            Node::new(vec!["input".into()], BTreeMap::new()),
            &alice,
        )
        .expect("call_as surfaces empty under coarse-grained view denial");
    assert!(outcome.is_ok_edge());
    assert!(
        outcome.as_list().map_or(true, |v| v.is_empty()),
        "Option C coarse-grained: read_view under cap denial must surface empty; got {outcome:?}"
    );
}
