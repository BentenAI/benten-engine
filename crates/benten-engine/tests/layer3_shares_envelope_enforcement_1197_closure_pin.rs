//! Wave-E HELD #1197 / #1146 / #596 / #1018 closure-pin —
//! Layer-3 plugin-manifest `shares`-envelope enforcement at the
//! `Engine::delegate_capability` production path.
//!
//! **What this pins (would-FAIL-if-reverted):**
//!
//! Pre-Wave-E, `EngineCapsHandle::delegate_capability` consulted a
//! hard-coded `AllPermit` `SharesPolicyView` shim — `fn permits(..) ->
//! true` for every cap / every target. CLAUDE.md baked-in #18 Layer 3
//! ("plugins delegate UCANs to each other freely *if and only if* the
//! request fits the source plugin's manifest `shares` policy") was
//! therefore **paper-only at HEAD** (META #669; Safe-3 #596): a hostile
//! plugin could delegate ANY cap to ANY target and the engine admitted
//! it.
//!
//! Wave-E deletes the `AllPermit` shim and consults the installed
//! [`benten_engine::shares_policy_resolver::SharesPolicyResolver`] port
//! keyed on the source grant's `actor`. This pin installs a real
//! resolver and asserts the FOUR security-load-bearing behaviors:
//!
//! 1. **Deny path fires.** A plugin-principal whose `shares` policy
//!    denies the step → `delegate_capability` REJECTS with the typed
//!    [`benten_errors::ErrorCode::PluginDelegationOutsideManifestEnvelope`].
//!    Reverting to `AllPermit` makes this assertion fail (the
//!    delegation would be admitted, returning `Ok(cid)`).
//! 2. **Fail-CLOSED on no-manifest.** A plugin-principal with NO
//!    resolvable manifest → REJECTS (never fail-OPEN). Reverting to
//!    `AllPermit` admits it.
//! 3. **No over-fire on user-root.** A user-root principal classified
//!    `NotPluginPrincipal` is ADMITTED (Layer 1 anchors user-issued
//!    caps; the manifest-`shares` gate must not block legitimate
//!    user-rooted delegations).
//! 4. **Private-namespace clause still fires FIRST.** A `private:*`
//!    source scope rejects with the private-namespace code BEFORE the
//!    resolver is consulted — the resolver wire-up must not regress the
//!    always-on private-namespace defense.

use std::sync::Arc;

use benten_engine::shares_policy_resolver::{DelegationResolution, SharesPolicyResolver};
use benten_engine::{Engine, EngineError};
use benten_errors::ErrorCode;

const USER_ROOT_DID: &str = "did:key:z6MkUserRootHarnessAAAAAAAAAAAAAAAAAAAAAAAAAA";
const HOSTILE_PLUGIN_DID: &str = "did:key:z6MkHostilePluginHarnessBBBBBBBBBBBBBBBBBBBBBB";
const UNKNOWN_PLUGIN_DID: &str = "did:key:z6MkUnknownPluginHarnessCCCCCCCCCCCCCCCCCCCCC";
const TARGET_PLUGIN_DID: &str = "did:key:z6MkTargetPluginHarnessDDDDDDDDDDDDDDDDDDDDDDD";

/// Real test resolver modelling a concrete manifest-`shares` policy
/// store. Mirrors what the production `benten-platform-foundation`
/// adapter does (resolve principal → manifest → `shares.permits`),
/// without dragging the foundation crate into this engine-lib test.
struct TestSharesResolver;

impl SharesPolicyResolver for TestSharesResolver {
    fn resolve_delegation(
        &self,
        source_principal_did: &str,
        _cap_pattern: &str,
        _target_plugin_did: &str,
    ) -> DelegationResolution {
        match source_principal_did {
            // The user-root anchors caps at Layer 1 — not a plugin
            // principal; the manifest-`shares` gate does not apply.
            USER_ROOT_DID => DelegationResolution::NotPluginPrincipal,
            // Hostile plugin: it HAS an installed manifest, but its
            // `shares` policy is the conservative `none` default — it
            // is not permitted to delegate this cap.
            HOSTILE_PLUGIN_DID => DelegationResolution::Denied,
            // Any other plugin-looking DID has no resolvable manifest
            // — fail-CLOSED (cannot verify the envelope ⇒ deny).
            _ => DelegationResolution::NoManifest,
        }
    }
}

fn engine_with_resolver() -> (Engine, tempfile::TempDir) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let mut engine = Engine::builder()
        .capability_policy_grant_backed()
        .open(tempdir.path().join("layer3-pin.redb"))
        .expect("engine opens with grant-backed policy");
    engine.set_shares_policy_resolver(Arc::new(TestSharesResolver));
    (engine, tempdir)
}

#[test]
fn deny_path_fires_reverting_to_allpermit_would_fail_this() {
    let (engine, _td) = engine_with_resolver();

    // Mint a user-rooted grant whose `actor` is the HOSTILE plugin-DID
    // and whose scope is a NON-private cap. Pre-Wave-E `AllPermit`
    // would admit the cross-plugin delegation; the real resolver
    // returns `Denied`.
    let source = engine
        .caps()
        .grant_capability(HOSTILE_PLUGIN_DID, "store:notes:write")
        .expect("mint source grant");

    let err = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[])
        .expect_err(
            "LOAD-BEARING: hostile plugin whose manifest `shares` policy \
             denies the step MUST be rejected; an `Ok(cid)` here means the \
             AllPermit shim is back and Layer 3 is paper-only again",
        );

    match err {
        EngineError::Other { code, ref message } => {
            assert_eq!(
                code,
                ErrorCode::PluginDelegationOutsideManifestEnvelope,
                "must surface typed PluginDelegationOutsideManifestEnvelope; got {code:?}"
            );
            assert!(
                message.contains("Layer-3") && message.contains(HOSTILE_PLUGIN_DID),
                "diagnostic must name Layer-3 + the offending source principal; got: {message}"
            );
        }
        other => {
            panic!("expected typed EngineError::Other deny; got {other:?} (AllPermit regression?)")
        }
    }
}

#[test]
fn no_manifest_is_fail_closed() {
    let (engine, _td) = engine_with_resolver();

    // The source `actor` is a plugin-looking DID with NO installed
    // manifest. Fail-CLOSED: an un-verifiable cross-plugin delegation
    // MUST reject, never admit.
    let source = engine
        .caps()
        .grant_capability(UNKNOWN_PLUGIN_DID, "store:notes:read")
        .expect("mint source grant");

    let err = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[])
        .expect_err(
            "fail-CLOSED: plugin-principal with no resolvable manifest MUST be \
             rejected — never fail-OPEN",
        );

    match err {
        EngineError::Other { code, .. } => assert_eq!(
            code,
            ErrorCode::PluginDelegationOutsideManifestEnvelope,
            "no-manifest path must reject with the typed envelope code; got {code:?}"
        ),
        other => panic!("expected fail-CLOSED reject; got {other:?}"),
    }
}

#[test]
fn user_root_principal_is_not_over_blocked() {
    let (engine, _td) = engine_with_resolver();

    // A user-rooted grant (actor = the user-root DID) is classified
    // `NotPluginPrincipal` — Layer 1 anchors it; the manifest-`shares`
    // gate must NOT block it. Regression-guard: the resolver wire-up
    // must not break legitimate user-rooted delegation.
    let source = engine
        .caps()
        .grant_capability(USER_ROOT_DID, "store:notes:read")
        .expect("mint source grant");

    let delegation = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[]);

    assert!(
        delegation.is_ok(),
        "user-root principal (NotPluginPrincipal) MUST be admitted — the \
         Layer-3 gate over-fired on a legitimate user-rooted delegation: {delegation:?}"
    );
}

#[test]
fn private_namespace_clause_fires_before_resolver() {
    let (engine, _td) = engine_with_resolver();

    // Source scope is a `private:<plugin>:*` cap. The always-on
    // private-namespace clause MUST reject with the private-namespace
    // code BEFORE the resolver is consulted — even though the source
    // actor is the user-root (which the resolver would classify
    // `NotPluginPrincipal` ⇒ admit). Proves the resolver wire-up did
    // not regress the structural private-namespace defense ordering.
    let private_scope = format!("private:{HOSTILE_PLUGIN_DID}:scratch");
    let source = engine
        .caps()
        .grant_capability(USER_ROOT_DID, private_scope.as_str())
        .expect("mint source grant");

    let err = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[])
        .expect_err("private-namespace cap MUST never cross plugin boundaries");

    match err {
        EngineError::Other { code, ref message } => {
            assert_eq!(
                code,
                ErrorCode::PluginPrivateNamespaceDelegationForbidden,
                "private-NS clause must fire (not the resolver path); got {code:?}"
            );
            assert!(
                message.contains("private-namespace"),
                "message must identify the private-namespace clause; got: {message}"
            );
        }
        other => panic!("expected private-namespace reject; got {other:?}"),
    }
}
