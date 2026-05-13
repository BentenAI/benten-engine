//! Phase-4-Foundation R5 G24-B-FP-1 — T7 LOAD-BEARING pin: admin UI v0
//! private namespace isolated from other plugins.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-5 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T7
//! ("Per-plugin private-namespace integrity") + CLAUDE.md baked-in #18
//! private-namespaces section.
//! Closure-destination: `docs/future/phase-4-backlog.md §4.14`
//! (G24-A mini-review g24a-mr-2 BLOCKER).
//!
//! ## What this pin establishes
//!
//! Per CLAUDE.md baked-in #18 (private namespaces): admin UI v0's
//! writes to its `private:admin-ui-v0:*` scope go to a DID-scoped
//! namespace; namespace cap is held by admin-UI-DID with `shares:
//! none`. Engine refuses to issue cross-plugin caps for private
//! namespaces — gives plugins a sovereign space without breaking
//! cross-plugin sharing semantics.
//!
//! End-to-end LOAD-BEARING per threat-model §T7 test-pin plan: an
//! "other plugin" attempts to gain access to admin-UI-DID's
//! `private:admin-ui-v0:*` scope through the engine's
//! [`benten_engine::Engine::delegate_capability`] seam — the
//! production-runtime entrypoint napi callers consume. The structural
//! defense at `benten-caps::plugin_delegation::check_delegation_within_envelope`
//! returns `DelegationDecision::PrivateNamespaceForbidden`, and the
//! engine threads this verdict into a typed
//! `E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN` `EngineError`.
//!
//! ## Coupling to sibling structural pins
//!
//! - `benten-platform-foundation/tests/g24d_substantive_pipeline.rs::private_namespace_cap_unconditionally_denied_cross_plugin`
//!   pins the **structural** check at the cap-policy layer (calls
//!   `check_delegation_within_envelope` directly).
//! - `benten-platform-foundation/tests/plugin_private_namespace_cap_no_cross_plugin_delegation.rs`
//!   pins the **per-finding** check (declared shares: any does NOT
//!   override `private:*`).
//! - THIS pin closes the **end-to-end** arm: the engine's full
//!   `Engine::delegate_capability` production-runtime entrypoint —
//!   the surface the napi binding + future plugin-install pipeline
//!   actually consumes.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires the structural check at
//! `crates/benten-caps/src/plugin_delegation.rs` but forgets to thread
//! the `PrivateNamespaceForbidden` variant into the
//! [`benten_engine::Engine::delegate_capability`] result mapping. A
//! malicious plugin then issues itself a cap-delegation for
//! `private:admin-ui-v0:auto_save`; the engine seam silently admits
//! it; cross-plugin namespace escape succeeds at runtime. This
//! end-to-end pin catches the gap that the unit-level pin alone
//! misses.

#![allow(clippy::unwrap_used)]

mod common;

use benten_engine::EngineError;
use benten_errors::ErrorCode;
use benten_platform_foundation::ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX;

use common::admin_ui_v0_harness::AdminUiV0TestHarness;

#[test]
fn admin_ui_v0_private_namespace_isolated_from_other_plugins_end_to_end() {
    // Substantive arm per pim-2 §3.6b — PRODUCTION-ARM (real
    // benten_engine::Engine with grant-backed policy + 2 plugin-DID
    // principals + Engine::delegate_capability call), OBSERVABLE-
    // CONSEQUENCE (typed E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN
    // surfaces with a diagnostic naming the private-namespace clause),
    // WOULD-FAIL-IF-NO-OP'd (removing the
    // `DelegationDecision::PrivateNamespaceForbidden` arm in
    // engine_caps.rs::delegate_capability would let the cross-plugin
    // delegation succeed, breaking the private-namespace sovereignty
    // contract).
    let harness = AdminUiV0TestHarness::new();

    // ------------------------------------------------------------------
    // (1) User installs admin UI v0 — mints a private-namespace grant
    //     under admin UI v0's prefix. The grant Node's scope text is
    //     `private:admin-ui-v0:auto_save` — exactly the shape the
    //     plugin-arch-r1-18 manifest section + CLAUDE.md baked-in #18
    //     promise plugins for their working memory.
    //
    //     Note: the admin-UI plugin DID string is what gets persisted
    //     into the grant Node's `actor` property; the harness's
    //     `admin_ui_plugin_principal_cid` is a separate
    //     `system:Principal` CID that other code paths consult. The
    //     `delegate_capability` seam resolves the source grant by CID
    //     + extracts its scope text — so what matters here is that the
    //     scope text starts with `private:`.
    // ------------------------------------------------------------------
    let private_scope = format!("{ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX}:auto_save");
    assert!(
        private_scope.starts_with("private:"),
        "T7 invariant: admin-UI private-NS prefix MUST start with `private:` \
         (CLAUDE.md baked-in #18 private-namespace clause keys off this prefix)"
    );
    let source_grant_cid = harness
        .mint_user_rooted_grant(harness.admin_ui_plugin_did_str(), &private_scope)
        .expect("admin-UI private-namespace grant minted via privileged path");

    // ------------------------------------------------------------------
    // (2) Hostile / "other" plugin attempts to receive a delegation of
    //     the admin-UI private-namespace cap via the production
    //     Engine::delegate_capability seam. Per
    //     CLAUDE.md baked-in #18 + benten-caps::plugin_delegation, the
    //     engine MUST refuse with typed
    //     E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN — the
    //     structural defense fires regardless of the source plugin's
    //     declared `shares` policy (even `shares: any` does NOT
    //     override the private-NS rule).
    // ------------------------------------------------------------------
    let err = harness
        .attempt_cross_plugin_delegation(&source_grant_cid, harness.hostile_plugin_did_str())
        .expect_err(
            "T7 LOAD-BEARING: cross-plugin delegation for private-NS cap \
             MUST be refused at the Engine::delegate_capability seam — \
             `shares: none` is the structural defense (CLAUDE.md #18)",
        );

    // ------------------------------------------------------------------
    // (3) Typed-code observable consequence — the rejection MUST surface
    //     E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN through the
    //     EngineError::Other variant the engine seam emits at
    //     `crates/benten-engine/src/engine_caps.rs:578-587`. A generic
    //     EngineError::Other with some other code, or a `NotFound`, or
    //     an opaque Backend error would fail this assertion — proving
    //     the typed-code path is the load-bearing surface, not just a
    //     boolean refusal.
    // ------------------------------------------------------------------
    match err {
        EngineError::Other {
            ref code,
            ref message,
        } => {
            assert_eq!(
                *code,
                ErrorCode::PluginPrivateNamespaceDelegationForbidden,
                "T7: must surface typed \
                 E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN; got {code:?}",
            );
            // Defense-in-depth diagnostic: the message MUST name the
            // private-namespace clause for forensic visibility — a
            // generic "delegation denied" string would hide the
            // structural nature of the rejection.
            assert!(
                message.contains("private-namespace") || message.contains("private:"),
                "T7: error message MUST identify the private-namespace \
                 clause for forensic clarity; got: {message}"
            );
            assert!(
                message.contains(harness.hostile_plugin_did_str())
                    || message.contains(&private_scope),
                "T7: error message MUST name the source scope OR target \
                 audience for operator triage; got: {message}"
            );
        }
        other => panic!(
            "T7 LOAD-BEARING: hostile cross-plugin private-NS delegation \
             MUST surface typed EngineError::Other with code \
             PluginPrivateNamespaceDelegationForbidden; got: {other:?}"
        ),
    }
}

#[test]
fn admin_ui_v0_private_namespace_refusal_independent_of_target_plugin_identity() {
    // Defense-in-depth boundary: the private-namespace refusal is
    // STRUCTURAL — it fires regardless of WHICH plugin DID attempts
    // the audience role. A regression that accidentally narrowed the
    // refusal to a hardcoded plugin-DID list would let new attackers
    // bypass the defense by minting fresh DIDs; this pin asserts that
    // doesn't happen.
    let harness = AdminUiV0TestHarness::new();

    let private_scope = format!("{ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX}:scratch");
    let source_grant_cid = harness
        .mint_user_rooted_grant(harness.admin_ui_plugin_did_str(), &private_scope)
        .unwrap();

    // Try delegation against THREE distinct target DIDs — none may
    // succeed; all must surface the same typed code.
    let target_dids = [
        "did:key:z6MkOtherPluginVariantOneAaaaa12345",
        "did:key:z6MkOtherPluginVariantTwoBbbbb67890",
        "did:key:z6MkOtherPluginVariantThreeCccccdef",
    ];
    for target in &target_dids {
        let err = harness
            .attempt_cross_plugin_delegation(&source_grant_cid, target)
            .expect_err("private-NS refusal fires for every target DID");
        match err {
            EngineError::Other { code, .. } => {
                assert_eq!(
                    code,
                    ErrorCode::PluginPrivateNamespaceDelegationForbidden,
                    "T7 structural: every target DID must hit the same \
                     typed-code refusal; target {target} got {code:?}"
                );
            }
            other => panic!(
                "T7 structural: expected EngineError::Other for target {target}; got {other:?}"
            ),
        }
    }
}

#[test]
fn admin_ui_v0_non_private_scope_delegation_does_not_trip_private_namespace_arm() {
    // Boundary regression-guard: the private-namespace structural
    // refusal MUST NOT over-fire on non-private scopes. A scope of
    // shape `store:notes:read` is NOT `private:*`; the
    // `is_private_namespace_cap` detector returns false; the
    // delegation seam consults the SharesPolicyView (which defaults
    // to `AllPermit` at G24-D's pre-manifest seam) and admits.
    //
    // This is the regression-guard half of the T7 pin: if the
    // private-namespace detector over-matched (e.g. recognising
    // `store:*` as private), the engine would refuse legitimate
    // cross-plugin delegations and break the layered consent flow.
    let harness = AdminUiV0TestHarness::new();

    let non_private_scope = "store:Note:read";
    let source_grant_cid = harness
        .mint_user_rooted_grant(harness.admin_ui_plugin_did_str(), non_private_scope)
        .unwrap();

    // Cross-plugin delegation for a non-private scope succeeds OR
    // fails on a different code (envelope, etc.), but MUST NOT
    // surface PrivateNamespaceDelegationForbidden — that would be
    // an over-fire of the structural defense.
    let result = harness
        .attempt_cross_plugin_delegation(&source_grant_cid, harness.hostile_plugin_did_str());
    match result {
        Ok(_delegation_cid) => {
            // Permitted by the AllPermit pre-manifest seam — fine;
            // the test's contract is "no false PrivateNS refusal".
        }
        Err(EngineError::Other { code, .. }) => {
            assert_ne!(
                code,
                ErrorCode::PluginPrivateNamespaceDelegationForbidden,
                "T7 regression-guard: non-private scope `{non_private_scope}` \
                 MUST NOT trip the private-namespace refusal arm — \
                 detector over-fire breaks legitimate cross-plugin delegation",
            );
        }
        Err(other) => {
            // Any other engine-side error path is acceptable; the
            // contract is specifically about the private-NS code.
            let s = format!("{other}");
            assert!(
                !s.contains("private-namespace"),
                "T7 regression-guard: non-private scope error path MUST \
                 NOT mention private-namespace clause; got: {s}"
            );
        }
    }
}
