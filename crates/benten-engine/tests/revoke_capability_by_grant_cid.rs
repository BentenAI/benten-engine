//! Phase-3.5 §13.11 closure — regression guard for the
//! `revoke_capability_by_grant_cid` seam.
//!
//! ## Production gap
//!
//! Pre-3.5 the napi `revokeCapability(grantCid, actor)` surface invoked
//! `Engine::revoke_capability(actor, grant_cid)` — passing the grant
//! CID AS the scope string. The engine wrote a
//! `system:CapabilityRevocation` Node with `scope = "<cid>"`, but
//! `BackendGrantReader::revoked_scopes` matches revocations by the
//! scope STRING (e.g. `"store:post:write"`). The revocation Node
//! never matched any real write scope; every post-revoke `callAs`
//! silently fail-OPENed. Real correctness gap, not a presentation gap.
//!
//! ## Acceptance criterion (per pim-2 §3.6b end-to-end pin)
//!
//! The would-FAIL-if-no-op'd shape: revert the napi (or engine) fix and
//! the post-revoke `has_unrevoked_grant_for_scope` call flips from
//! `Ok(false)` to `Ok(true)` — observable consequence at the
//! `GrantReader` contract layer that `GrantBackedPolicy::check_write`
//! consumes. The pre-3.5 path stored the revocation under `scope =
//! "<cid>"`, which the reader's `revoked_scopes()` walker collected
//! but never matched against the actual query.
//!
//! ## Defense-in-depth
//!
//! The Phase-3 G16-B-F `apply_atrium_merge` per-row cap-recheck masks
//! this gap during cross-peer sync but does NOT fix it for same-peer
//! revoke-then-write sequences — those go through
//! `GrantBackedPolicy::check_write` directly, which is the surface
//! this test exercises.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "regression test exercises typed assertions explicitly"
)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post_node(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

/// End-to-end pin (pim-2 §3.6b): grant → revoke-by-grant-cid → call
/// arc through the grant-backed policy. Would-FAIL-if-no-op'd: revert
/// `revoke_capability_by_grant_cid` to its pre-3.5 napi semantics
/// (writing `scope = "<cid>"`) and the post-revoke call routes through
/// the success edge rather than `ON_DENIED`.
#[test]
fn revoke_capability_by_grant_cid_denies_post_revoke_write() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let handler_id = engine.register_crud("post").unwrap();
    let actor = engine.caps().create_principal("carol").unwrap();
    let grant_cid = engine
        .caps()
        .grant_capability(&actor, "store:post:write")
        .expect("grant via privileged path");

    // Pre-revoke: write succeeds via OK edge.
    let pre = engine
        .call(&handler_id, "post:create", post_node("pre-revoke"))
        .expect("call ok");
    assert!(pre.is_ok_edge(), "pre-revoke must route via OK edge");

    // Revoke by grant CID — the seam the napi surface consumes.
    engine
        .caps()
        .revoke_capability_by_grant_cid(&grant_cid, &actor)
        .expect("revoke by grant CID resolves scope + writes revocation");

    // Post-revoke: write MUST route to ON_DENIED with E_CAP_DENIED.
    let post = engine
        .call(&handler_id, "post:create", post_node("post-revoke"))
        .expect("call returns Ok even when routing to ON_DENIED");
    assert!(
        post.routed_through_edge("ON_DENIED"),
        "expected ON_DENIED route; got {:?}",
        post.edge_taken()
    );
    assert_eq!(
        post.error_code(),
        Some("E_CAP_DENIED"),
        "post-revoke must surface E_CAP_DENIED — pre-3.5 napi semantics fail-OPENed here"
    );
}

/// Negative pin: an unknown grant CID surfaces a typed `E_NOT_FOUND`
/// rather than silently writing a revocation under that CID. Catches
/// future regressions where the seam reverts to "stuff the CID in as
/// scope" without erroring on lookup miss.
#[test]
fn revoke_capability_by_grant_cid_unknown_cid_errors_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let actor = engine.caps().create_principal("mallory").unwrap();
    // A CID that is well-formed but does not resolve to any stored Node.
    // We mint one by hashing an arbitrary throwaway Node and then NOT
    // writing it to the backend.
    let throwaway = Node::new(vec!["nonexistent".into()], BTreeMap::new());
    let synthetic_cid = throwaway.cid().expect("cid mint");

    let err = engine
        .caps()
        .revoke_capability_by_grant_cid(&synthetic_cid, &actor)
        .expect_err("unknown grant CID MUST surface a typed not-found error");
    let code = err.code();
    assert_eq!(
        code.as_str(),
        "E_NOT_FOUND",
        "unknown grant CID MUST surface E_NOT_FOUND (got {:?})",
        code
    );
}

/// Negative pin: a CID that resolves to a Node whose primary label is
/// NOT `system:CapabilityGrant` also surfaces `E_NOT_FOUND` — the seam
/// refuses to write a revocation that has no grant scope to mirror.
#[test]
fn revoke_capability_by_grant_cid_wrong_label_errors_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let actor = engine.caps().create_principal("eve").unwrap();
    // The principal Node IS in the backend (we just created it) but its
    // label is `system:Principal`, not `system:CapabilityGrant`.
    let err = engine
        .caps()
        .revoke_capability_by_grant_cid(&actor, &actor)
        .expect_err("non-grant CID MUST surface a typed not-found error");
    assert_eq!(
        err.code().as_str(),
        "E_NOT_FOUND",
        "wrong-label CID MUST surface E_NOT_FOUND (got {:?})",
        err.code()
    );
}
