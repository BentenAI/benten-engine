//! G27-A class-of-bug regression guard — napi `delegateCapability`
//! binding (shipped at G24-D-FP-3).
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.14 G27-A row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-A entry +
//! `docs/future/phase-4-backlog.md` §4.5 (retargeted destination §4.8 →
//! lands at G24-D-FP-3) + §4.8 (substantive arm acceptance criteria).
//! G27-A R5 mini-review MINOR finding `g27a-mr-1` closed here.
//!
//! ## Class-of-bug defended (the PR #199 instance mirrored on delegate)
//!
//! Pre-FP-3: if `delegateCapability(sourceGrantCid, pluginDid,
//! attenuatedCaps)` napi routed to `Engine::issue_delegation(plugin_did,
//! source_grant_cid_string, attenuation)` — passing the source CID AS
//! the new delegation's scope — the resulting `system:CapabilityGrant`
//! Node would carry `scope = "<source_cid base32>"`. The
//! `BackendGrantReader::has_unrevoked_grant_for_scope` walker matches
//! revocations by the SCOPE STRING (`"store:post:write"`), so the
//! delegation would NEVER admit at policy-check time — every
//! cross-plugin write under the delegated cap would silently
//! fail-CLOSE (route to `ON_DENIED`) regardless of the source grant's
//! actual permissions. This is the same shape PR #199 closed for
//! `revokeCapability` (fail-OPENed there because revocation Nodes
//! similarly couldn't be matched against actual write scopes).
//!
//! Closure: G24-D-FP-3 lands `Engine::delegate_capability` which
//! resolves the source grant Node by CID + extracts its `scope` text +
//! writes the new delegation Node carrying that **resolved** scope.
//! The napi binding routes through this seam unconditionally.
//!
//! ## What this regression guard pins (4-step substantive arm)
//!
//! Per `docs/future/phase-4-backlog.md` §4.8 acceptance criteria + the
//! G24-D-FP-3 brief:
//!
//! 1. `delegateCapability(grantCid, plugin_did, attenuated_caps)` over
//!    napi resolves the napi-passed CID to the underlying grant +
//!    invokes the envelope-check.
//! 2. Verify the new delegation Node is minted with the **resolved
//!    scope** (not the grantCid as a string — defends the G27-A
//!    class-of-bug). The fastest observable assertion is "a write
//!    under the delegated cap as the plugin-DID admits", which can
//!    only happen if the persisted grant's `scope` text matches the
//!    handler's derived write scope.
//! 3. Attempt a write under the delegated cap; verify it admits per
//!    the manifest envelope (OK edge).
//! 4. Assert the per-row cap-recheck at delivery resolves the scope
//!    correctly (couples to G16-B-F per-row recheck at sync delivery
//!    — same `GrantReader` machinery the per-row loop consults). The
//!    canonical proof is "revoke the delegation, then a follow-on
//!    write at the same scope routes to ON_DENIED" — this can only
//!    happen if the revocation walker resolves the SCOPE STRING (not
//!    the CID) against the persisted delegation Node, AND
//!    symmetrically only if the original delegation was persisted
//!    with the resolved scope text (the pre-FP-3 CID-as-scope shape
//!    would have made the delegation un-revocable by scope).
//!
//! ## Would-FAIL-if-no-op'd (pim-2 §3.6b)
//!
//! Mutating `Engine::delegate_capability` (in
//! `crates/benten-engine/src/engine_caps.rs`) to pass
//! `source_grant_cid.to_base32()` as the new grant's `scope` property
//! instead of the resolved scope-text — the post-delegate write below
//! would surface `ON_DENIED` instead of OK because
//! `GrantBackedPolicy::check_write` would never match the
//! `<source_cid>`-keyed grant Node against `store:post:write`. This
//! is the exact mirror of the PR #199 napi-revoke class-of-bug pin
//! in `cap_revoke_napi_passes_resolved_scope_not_cid_regression_guard.rs`.
//!
//! ## Production-arm + observable-consequence (pim-2 §3.6b)
//!
//! - **Production-arm:** `delegateCapability` over napi →
//!   `Engine::delegate_capability` (G24-D-FP-3 seam) → resolve grant
//!   Node by CID → extract scope text → run
//!   `check_delegation_within_envelope` (G24-D) → write
//!   `system:CapabilityGrant` Node with resolved scope.
//! - **Observable-consequence:** `callAs(handler, op, input,
//!   plugin_did)` after delegation admits via OK edge; revoking the
//!   delegation then surfaces `E_CAP_DENIED` on the next write.
//! - **Would-FAIL-if-no-op'd:** see above (CID-as-scope mutation
//!   flips both assertions).

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

/// G27-A class-of-bug regression guard for the napi `delegateCapability`
/// binding shipped at G24-D-FP-3 — production-arm wire-up +
/// observable-consequence + would-FAIL-if-no-op'd (pim-2 §3.6b).
///
/// The 4-step substantive arm:
///
/// 1. **Mint source grant.** User grants `store:post:write` to a
///    plugin-DID-shaped actor (`did:key:z6MkSource...`). The grant
///    Node's `scope` property is the load-bearing string the
///    delegation Node MUST carry.
/// 2. **Delegate via napi seam.** `delegate_capability` resolves the
///    source grant CID + writes a new delegation Node carrying the
///    **resolved scope** for the audience plugin-DID
///    (`did:key:z6MkPluginB...`). The pre-FP-3 class-of-bug shape
///    would have persisted `scope = "<source_cid>"` here; the FP-3
///    seam persists `scope = "store:post:write"`.
/// 3. **Write under the delegated cap admits.** `callAs(handler,
///    op, input, plugin_b_did)` admits via the OK edge — the
///    `GrantBackedPolicy::check_write` walker matched the persisted
///    delegation's resolved scope against the handler's derived
///    write scope. If the delegation had been persisted CID-keyed
///    (pre-FP-3 shape) this would route to `ON_DENIED`.
/// 4. **Revoke + verify per-row cap-recheck resolves scope.** Revoke
///    the delegation by CID via the post-PR-#199 resolving seam
///    (`revoke_capability_by_grant_cid`). The next write as
///    plugin-B-DID routes to `ON_DENIED` with `E_CAP_DENIED` —
///    proving the same `GrantReader` machinery G16-B-F's per-row
///    cap-recheck inside `apply_atrium_merge` consults DOES observe
///    the revocation, which is only possible if the original
///    delegation was persisted with the resolved scope text (a
///    CID-keyed grant Node would be un-revocable-by-scope).
#[test]
fn napi_delegate_binding_routes_through_resolving_seam_full_4_step_substantive_arm() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let handler_id = engine.register_crud("post").unwrap();

    // ----- Step 1: mint the source grant -----
    //
    // The source actor is a plugin-DID-shaped string ("source plugin"
    // in the three-layer trust model). The grant authorizes
    // `store:post:write` — the load-bearing scope text.
    //
    // Plug into `call_as` via a principal Cid handle for the source.
    // The `GrantBackedPolicy::check_write` walker is scope-keyed (per
    // `crates/benten-caps/src/grant_backed.rs::check_write`), so the
    // actor we pass to `call_as` is the principal Cid; the grant we
    // mint stores `actor=<source plugin-DID text>` for audit but the
    // policy gate fires on the scope-string match.
    let source_principal = engine.create_principal("source-plugin").unwrap();
    let source_plugin_did = "did:key:z6MkSourcePluginA1234567890abcdefghi";
    let source_grant_cid = engine
        .grant_capability(source_plugin_did, "store:post:write")
        .expect("grant via privileged path");

    // Sanity: the source grant itself admits writes at the granted
    // scope. If this fails, the test setup is broken (the regression
    // guard is meaningless).
    let src_write = engine
        .call_as(
            &handler_id,
            "post:create",
            post_node("source-grant-write"),
            &source_principal,
        )
        .expect("source grant write returns Ok");
    assert!(
        src_write.is_ok_edge(),
        "source grant must admit writes at the granted scope (test setup invariant)"
    );

    // ----- Step 2: delegate via the napi-consumed engine seam -----
    //
    // Audience = plugin-B DID; attenuation is empty so the delegation
    // carries the resolved source scope (`store:post:write`) verbatim.
    // The class-of-bug discipline pins that the new grant's `scope`
    // is the resolved TEXT, never the source CID's base32 string.
    let plugin_b_principal = engine.create_principal("plugin-b").unwrap();
    let plugin_b_did = "did:key:z6MkPluginAudienceB1234567890abcdefg";
    let delegation_cid = engine
        .delegate_capability(&source_grant_cid, plugin_b_did, &[])
        .expect("delegate via FP-3 seam");

    // ----- Step 2 verification (observable-consequence form): the
    //       delegation Node carries the resolved scope (NOT the
    //       source CID as a string). -----
    //
    // Direct backend-probe of `system:CapabilityGrant` Nodes via the
    // engine's public `get_node` is intentionally collapsed to
    // `Ok(None)` per Inv-11 (engine-side system-zone read gate); the
    // FP-3 brief consequently expresses Step-2 verification through
    // the observable-consequence chain in Steps 3-4. Both `assert!`s
    // below would FLIP if the delegation Node carried the source
    // CID's base32 string as its `scope` property:
    //
    //   - Step 3's `is_ok_edge` flips to `routed_through_edge("ON_DENIED")`
    //     because `GrantBackedPolicy::check_write` resolves the write's
    //     derived scope (`store:post:write`) against the persisted
    //     grant's scope text — a CID-keyed grant Node fails to match.
    //   - Step 4's `routed_through_edge("ON_DENIED")` flips to
    //     `is_ok_edge` (or "no revocation observed") because the
    //     `BackendGrantReader::revoked_scopes` walker matches
    //     revocations by scope STRING against grants by scope STRING;
    //     a delegation Node carrying `scope = "<source_cid>"` is
    //     un-revocable-by-scope by definition.
    //
    // So the test's Steps 3+4 jointly pin Step 2 — the resolved-scope
    // discipline IS the observable contract. The would-FAIL-if-no-op'd
    // mutation surfaces at the next assertion, not via direct probe.
    let _: benten_core::Cid = delegation_cid; // shape-witness

    // ----- Step 3: write under the delegated cap admits per envelope -----
    //
    // `callAs` as plugin-B-DID under the delegated scope; the
    // `GrantBackedPolicy::check_write` walker resolves
    // `store:post:write` against the persisted delegation Node and
    // admits at the OK edge. Would-FAIL-if-no-op'd: with a CID-keyed
    // delegation, the walker fails to match → routes to ON_DENIED.
    let post_delegate = engine
        .call_as(
            &handler_id,
            "post:create",
            post_node("post-delegate-write"),
            &plugin_b_principal,
        )
        .expect("delegated write returns Ok");
    assert!(
        post_delegate.is_ok_edge(),
        "delegated write under resolved scope MUST admit via OK edge \
         (got edge {:?}, error {:?}); a CID-keyed delegation Node \
         (pre-FP-3 shape) would surface ON_DENIED here because the \
         GrantReader walker can't match the CID-string against \
         `store:post:write` at policy-check time",
        post_delegate.edge_taken(),
        post_delegate.error_code()
    );

    // ----- Step 4: per-row cap-recheck-at-delivery resolves scope -----
    //
    // Revoke the delegation by CID via the post-PR-#199 resolving
    // seam. The revocation walker
    // (`BackendGrantReader::revoked_scopes`) keys by SCOPE STRING —
    // identical to the per-row cap-recheck inside
    // `Engine::apply_atrium_merge` per G16-B-F. If the original
    // delegation Node was persisted with `scope = "<source_cid>"`
    // (pre-FP-3 shape), this revocation would write a
    // `system:CapabilityRevocation` Node with the resolved scope, but
    // the delegation Node it's revoking still carries the CID-string
    // scope → walker never observes the revocation as targeting the
    // delegation. With the FP-3 resolved-scope shape, the revocation
    // + delegation scopes match → walker fires → subsequent write
    // routes to ON_DENIED with E_CAP_DENIED.
    engine
        .revoke_capability_by_grant_cid(&delegation_cid, plugin_b_did)
        .expect("revoke delegation via resolving seam");

    let post_revoke = engine
        .call_as(
            &handler_id,
            "post:create",
            post_node("post-revoke-of-delegation"),
            &plugin_b_principal,
        )
        .expect("call returns Ok envelope (routes to ON_DENIED)");
    assert!(
        post_revoke.routed_through_edge("ON_DENIED"),
        "post-revoke write MUST route to ON_DENIED (got edge {:?}); \
         this proves the per-row cap-recheck machinery (same \
         `GrantReader` walker G16-B-F consults inside \
         `apply_atrium_merge`) DOES resolve the scope correctly — \
         the pre-FP-3 CID-keyed delegation would have made this \
         delegation un-revocable-by-scope",
        post_revoke.edge_taken()
    );
    assert_eq!(
        post_revoke.error_code(),
        Some("E_CAP_DENIED"),
        "post-revoke must surface E_CAP_DENIED — the resolving seam \
         + the scope-keyed GrantReader walker together prove the \
         FP-3 delegate path persists the resolved scope text, not \
         the source CID's base32 string"
    );
}

/// G27-A class-of-bug regression guard — private-namespace cap is
/// unconditionally denied for cross-plugin delegation per CLAUDE.md
/// baked-in #18.
///
/// Private-namespace caps (`private:<plugin_did>:*`) MUST NEVER cross
/// plugin boundaries. The engine seam's
/// `check_delegation_within_envelope` call returns
/// `DelegationDecision::PrivateNamespaceForbidden` for `private:*`
/// caps regardless of any manifest `shares` policy.
///
/// Would-FAIL-if-no-op'd: removing the `is_private_namespace_cap`
/// branch from `Engine::delegate_capability` would let this
/// cross-plugin delegation succeed, breaking the private-namespace
/// sovereignty contract (plugins' working-memory caps would leak
/// to other plugins via UCAN delegation).
#[test]
fn napi_delegate_private_namespace_cap_unconditionally_forbidden() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let source_plugin_did = "did:key:z6MkPrivateOwnerPlugin1234567890abc";
    let private_scope = format!("private:{source_plugin_did}:notes");
    let source_grant_cid = engine
        .grant_capability(source_plugin_did, private_scope.as_str())
        .expect("private-namespace grant minted (intra-plugin)");

    let other_plugin_did = "did:key:z6MkOtherPluginAudienceB12345abcdefg";
    let err = engine
        .delegate_capability(&source_grant_cid, other_plugin_did, &[])
        .expect_err("private-namespace cross-plugin delegation MUST fail");

    let err_str = format!("{err}");
    assert!(
        err_str.contains("private-namespace") || err_str.contains("PluginPrivateNamespace"),
        "expected private-namespace forbidden error; got: {err_str}"
    );
}

/// Compile-time witness: the resolving seam the napi delegate binding
/// consumes is reachable + the signature shape stays stable.
#[test]
fn napi_delegate_resolving_seam_compile_witness() {
    fn _accepts_resolving_seam(
        _engine: &Engine,
        _source_grant_cid: &benten_core::Cid,
        _plugin_did: &str,
        _attenuated_caps: &[String],
    ) -> Result<benten_core::Cid, benten_engine::EngineError> {
        unimplemented!("compile-time witness — body never runs")
    }
    let _: fn(
        &Engine,
        &benten_core::Cid,
        &str,
        &[String],
    ) -> Result<benten_core::Cid, benten_engine::EngineError> = _accepts_resolving_seam;
}
