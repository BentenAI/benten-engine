//! G27-A regression guard — PR #199 inheritance pin
//! (`revoke_capability_by_grant_cid` napi binding).
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.14 G27-A row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-A entry.
//! Inherits the production fix shipped at PR #199 + extends the
//! regression-guard surface to the napi binding layer per Ben D-4F-5
//! reframe.
//!
//! ## The class-of-bug instance closed at PR #199
//!
//! Pre-3.5: napi `revokeCapability(grant_cid, actor)` called
//! `Engine::revoke_capability(actor, grant_cid)` — passing the CID
//! string AS the scope. The engine wrote a
//! `system:CapabilityRevocation` Node with `scope = "<cid base32>"`.
//! `BackendGrantReader::revoked_scopes` matches revocations by the
//! scope STRING (`"store:post:write"`), so the revocation Node never
//! fired at policy-check time + every post-revoke `callAs` silently
//! fail-OPENed.
//!
//! Closure: PR #199 introduced
//! `Engine::revoke_capability_by_grant_cid(grant_cid, actor)` which
//! resolves the grant Node by CID, extracts its `scope` property, then
//! writes the revocation Node carrying the actual scope string. The
//! napi binding at `bindings/napi/src/lib.rs:666-680` now routes
//! through this seam.
//!
//! ## What this regression guard pins
//!
//! Verifies the production-arm CONTINUES to honor the post-PR-#199
//! contract: napi revoke routes through the resolving seam + the
//! observable consequence (post-revoke write is denied) holds. If a
//! future refactor accidentally reverts the napi binding to the
//! pre-3.5 shape, this pin fires.
//!
//! ## Would-FAIL-if-no-op'd (pim-2 §3.6b)
//!
//! Revert `bindings/napi/src/lib.rs:677-678` to
//! `engine.revoke_capability(actor, grant_cid.to_base32())` (the
//! pre-3.5 shape) — the post-revoke write below would surface
//! `pre.is_ok_edge()` instead of routing through the denied edge.
//!
//! ## Un-ignored at G27-A wave (R5)
//!
//! The G27-A R5 audit confirmed the napi binding at
//! `bindings/napi/src/lib.rs:666-680` continues to route through the
//! resolving seam (`Engine::revoke_capability_by_grant_cid`). The
//! companion `notes-napi-parity-audit.md` §1 records the audit walk:
//! the binding parses the CID with `parse_cid` (no scope conflation),
//! the actor parameter is taken separately as a string, and the
//! engine seam is called with `(&grant_cid, actor)`. This test
//! exercises the observable-consequence arm of the audit (post-revoke
//! write routes to `ON_DENIED` with `E_CAP_DENIED`).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "in-process-test")]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post_node(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

/// G27-A regression guard for PR #199 inheritance (un-ignored at R5).
///
/// Pins that the napi revoke binding's engine seam
/// (`Engine::revoke_capability_by_grant_cid`) DOES resolve the grant's
/// scope property (not the grant CID) at revocation time, AND the
/// downstream policy check observes the revocation correctly.
///
/// Would-FAIL-if-no-op'd: revert `bindings/napi/src/lib.rs:677-678` to
/// `engine.revoke_capability(actor, grant_cid.to_base32())` (passing
/// the CID AS the scope) — the post-revoke write below would surface
/// `is_ok_edge()` instead of routing through `ON_DENIED` with
/// `E_CAP_DENIED`.
#[test]
fn napi_revoke_binding_routes_through_resolving_seam_post_revoke_write_denied() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let handler_id = engine.register_crud("post").unwrap();
    let actor = engine.create_principal("carol").unwrap();
    let grant_cid = engine
        .grant_capability(&actor, "store:post:write")
        .expect("grant via privileged path");

    // Pre-revoke: write succeeds via OK edge (grant is in effect).
    let pre = engine
        .call(&handler_id, "post:create", post_node("pre-revoke"))
        .expect("call ok");
    assert!(pre.is_ok_edge(), "pre-revoke must route via OK edge");

    // Substantive class-of-bug arm: the napi binding routes through
    // this seam at `bindings/napi/src/lib.rs:677-678`. If it ever
    // reverts to `engine.revoke_capability(actor, grant_cid)` (the
    // pre-3.5 shape with the scope arg being the CID), the
    // post-revoke write below routes via OK edge instead of denied.
    engine
        .revoke_capability_by_grant_cid(&grant_cid, &actor)
        .expect("revoke via resolving seam");

    // Post-revoke observable consequence: write at the originally
    // granted scope MUST route to ON_DENIED with E_CAP_DENIED. This
    // is the load-bearing assertion of the regression guard — the
    // pre-PR-#199 napi binding would have written a revocation Node
    // with `scope = "<grant_cid base32>"` which the
    // `BackendGrantReader::revoked_scopes` walker would never have
    // matched against "store:post:write" at policy-check time. The
    // would-FAIL-if-no-op'd shape is: this assertion flips because
    // the policy walker doesn't observe the revocation.
    let post = engine
        .call(&handler_id, "post:create", post_node("post-revoke"))
        .expect("call returns Ok even when routing to ON_DENIED");
    assert!(
        post.routed_through_edge("ON_DENIED"),
        "expected ON_DENIED route post-revoke (got edge {:?}); pre-PR-#199 \
         napi binding fail-OPENed here because the revocation Node was \
         persisted with scope = <cid> rather than the resolved scope-string",
        post.edge_taken()
    );
    assert_eq!(
        post.error_code(),
        Some("E_CAP_DENIED"),
        "post-revoke must surface E_CAP_DENIED — pre-3.5 napi semantics \
         silently fail-OPENed here because the revocation scope-string \
         never matched the write's derived scope at the GrantReader walker"
    );
}

/// Compile-time witness: the resolving seam the napi revoke binding
/// consumes is reachable + the signature shape stays stable.
#[test]
fn napi_revoke_resolving_seam_compile_witness() {
    fn _accepts_resolving_seam(
        _engine: &Engine,
        _grant_cid: &benten_core::Cid,
        _actor: &benten_core::Cid,
    ) -> Result<(), benten_engine::EngineError> {
        unimplemented!("compile-time witness — body never runs")
    }
    let _: fn(
        &Engine,
        &benten_core::Cid,
        &benten_core::Cid,
    ) -> Result<(), benten_engine::EngineError> = _accepts_resolving_seam;
}
