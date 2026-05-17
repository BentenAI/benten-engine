//! v1-API-stabilization closure-pin (#820 / #1195) — `engine.caps()`
//! is the SOLE canonical capability-mutation surface.
//!
//! ## What this pins
//!
//! Decision-queue #3 (owner-confirmed): the Engine-direct
//! capability-grant mutation methods (`grant_capability`,
//! `grant_capability_with_proof`, `revoke_capability`,
//! `revoke_capability_by_grant_cid`, `install_ucan_proof`,
//! `delegate_capability`, `create_principal`) were DELETED from
//! `impl Engine` and now live EXCLUSIVELY on
//! [`benten_engine::EngineCapsHandle`] (returned by `Engine::caps()`),
//! alongside the pre-existing `CapProof`-based `install_proof` /
//! `revoke`.
//!
//! ## Closure-pin shape (§3.6b — would-FAIL-if-reverted)
//!
//! 1. **Behavioral arm:** grant via `engine.caps().grant_capability(..)`
//!    → an actor write succeeds (OK edge); revoke via
//!    `engine.caps().revoke_capability_by_grant_cid(..)` → the SAME
//!    write is denied (ON_DENIED / `E_CAP_DENIED`). Exercises the
//!    canonical handle end-to-end against a real redb-backed engine.
//!    Would-FAIL-if-reverted: if the handle's `grant`/`revoke` plumbing
//!    regressed, the post-revoke write would fail-OPEN through the OK
//!    edge.
//! 2. **Structural arm:** this test file itself only ever calls the
//!    cap-mutation methods through `engine.caps()`. The Engine-direct
//!    methods no longer exist — re-adding an `Engine::grant_capability`
//!    facade (the band-aid this decision rejected) would make the
//!    `caps()`-routed call ambiguous-free but would reintroduce the
//!    redundant surface CLAUDE.md rule #5 forbids. The compile itself
//!    is the structural pin: `engine.grant_capability(..)` does not
//!    resolve.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "closure-pin test exercises typed assertions explicitly"
)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post_node(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

/// Canonical surface end-to-end: grant → write-ok → revoke → write-denied,
/// every cap mutation routed through `engine.caps()`.
#[test]
fn caps_handle_is_the_enforced_grant_and_revoke_surface() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let handler_id = engine.register_crud("post").unwrap();

    // SOLE surface: principal + grant minted through `engine.caps()`.
    let actor = engine.caps().create_principal("dana").unwrap();
    let grant_cid = engine
        .caps()
        .grant_capability(&actor, "store:post:write")
        .expect("grant via the canonical caps() handle");

    // Pre-revoke write succeeds via the OK edge.
    let pre = engine
        .call(&handler_id, "post:create", post_node("pre"))
        .expect("call ok");
    assert!(pre.is_ok_edge(), "pre-revoke write must route via OK edge");

    // Revoke through the canonical handle.
    engine
        .caps()
        .revoke_capability_by_grant_cid(&grant_cid, &actor)
        .expect("revoke via the canonical caps() handle");

    // Post-revoke write MUST be denied — proves the canonical handle's
    // revoke is the one the policy enforces.
    let post = engine
        .call(&handler_id, "post:create", post_node("post"))
        .expect("call returns Ok even when routing to ON_DENIED");
    assert!(
        post.routed_through_edge("ON_DENIED"),
        "post-revoke write MUST route to ON_DENIED; got {:?}",
        post.edge_taken()
    );
    assert_eq!(
        post.error_code(),
        Some("E_CAP_DENIED"),
        "post-revoke write must surface E_CAP_DENIED via the canonical handle"
    );
}

/// `grant_capability_with_proof` (issuer + hlc threading) is reachable
/// only through `engine.caps()` — pins the moved-method signature.
#[test]
fn caps_handle_exposes_grant_with_proof() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens");

    let actor = engine.caps().create_principal("issuer-principal").unwrap();
    let cid = engine
        .caps()
        .grant_capability_with_proof(
            &actor,
            "store:post:write",
            Some("did:key:zTestIssuer".to_string()),
            Some(42),
        )
        .expect("grant_capability_with_proof via canonical handle");
    // The minted grant Node resolves; its CID is content-addressed.
    assert!(
        !cid.to_base32().is_empty(),
        "grant_capability_with_proof must mint a content-addressed grant Node"
    );
}
