//! Wave-E HELD #1018 / #1206 closure-pin — the napi `delegateCapability`
//! surface inherits the now-real Layer-3 manifest-`shares` envelope
//! enforcement (no bypass at the napi boundary).
//!
//! ## Class-of-bug defended
//!
//! Pre-Wave-E, `EngineCapsHandle::delegate_capability` — the seam the
//! napi `delegateCapability` binding routes through unconditionally —
//! consulted a hard-coded `AllPermit` `SharesPolicyView` shim. CLAUDE.md
//! baked-in #18 Layer 3 ("plugins delegate UCANs to each other freely
//! *iff* the request fits the source plugin's manifest `shares`
//! policy") was therefore **paper-only at the napi boundary** (#1018):
//! every cross-plugin delegation issued through the Node.js binding was
//! admitted regardless of the source plugin's signed, user-consented
//! manifest `shares` policy — the napi face of the META #593 / META
//! #669 auth-bypass cluster.
//!
//! Wave-E deleted the `AllPermit` shim (engine half, commit
//! `0dbfa438`). This pin asserts the napi-consumed engine seam now
//! enforces Layer-3 fail-CLOSED: the napi binding has NO separate
//! bypass — it routes through `inner.caps().delegate_capability(..)`
//! unconditionally, so the engine-side enforcement IS the napi-side
//! enforcement.
//!
//! ## Why an engine-seam test in the napi crate
//!
//! The `#[napi] JsEngine` object cannot be instantiated outside a
//! Node.js runtime, so — exactly like the sibling
//! `cap_delegate_napi_resolved_scope_regression_guard.rs` — the
//! production arm exercised is `benten_engine::Engine::caps()
//! .delegate_capability(..)`, the seam the napi binding wraps verbatim
//! (`bindings/napi/src/lib.rs` `delegate_capability` →
//! `self.inner.caps().delegate_capability(&source, plugin_did, ..)`).
//! The napi binding adds ONLY the #1010 plugin-DID format gate
//! (resolved-on-main) on top of this seam; it adds no `shares` bypass.
//!
//! ## Would-FAIL-if-reverted (pim-2 §3.6b)
//!
//! Restoring the `AllPermit` shim in `engine_caps.rs` makes
//! `hostile_plugin_delegation_denied_through_napi_consumed_seam` fail
//! (the delegation would return `Ok(cid)` instead of the typed deny).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "in-process-test")]

use benten_engine::shares_policy_resolver::{DelegationResolution, SharesPolicyResolver};
use benten_engine::{Engine, EngineError};
use benten_errors::ErrorCode;
use std::sync::Arc;

const HOSTILE_PLUGIN_DID: &str = "did:key:z6MkHostileNapiPluginAAAAAAAAAAAAAAAAAAAAAAA";
const TARGET_PLUGIN_DID: &str = "did:key:z6MkTargetNapiPluginBBBBBBBBBBBBBBBBBBBBBBBBB";

/// Production-shaped resolver: a plugin whose manifest `shares` policy
/// denies the step. Mirrors what the composition-root adapter installs.
struct DenyHostileResolver;

impl SharesPolicyResolver for DenyHostileResolver {
    fn resolve_delegation(
        &self,
        source_principal_did: &str,
        _cap_pattern: &str,
        _target_plugin_did: &str,
    ) -> DelegationResolution {
        if source_principal_did == HOSTILE_PLUGIN_DID {
            DelegationResolution::Denied
        } else {
            DelegationResolution::NoManifest
        }
    }
}

#[test]
fn hostile_plugin_delegation_denied_through_napi_consumed_seam() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();
    // The composition root (napi binding / platform integration)
    // installs the real adapter; this test installs the production-
    // shaped resolver to exercise the seam the napi binding routes
    // through.
    engine.set_shares_policy_resolver(Arc::new(DenyHostileResolver));

    // User mints a grant whose `actor` is the hostile plugin-DID
    // (non-private scope). Pre-Wave-E the napi delegate path's
    // `AllPermit` shim would admit the cross-plugin delegation.
    let source = engine
        .caps()
        .grant_capability(HOSTILE_PLUGIN_DID, "store:post:write")
        .unwrap();

    // This is the EXACT seam call the napi `delegateCapability` binding
    // makes (`self.inner.caps().delegate_capability(..)`).
    let err = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[])
        .expect_err(
            "LOAD-BEARING #1018: a cross-plugin delegation whose source \
             plugin's manifest `shares` policy denies it MUST be rejected \
             at the napi-consumed seam; `Ok(cid)` means the AllPermit \
             bypass is back and Layer 3 is paper-only at the napi boundary",
        );

    match err {
        EngineError::Other { code, .. } => assert_eq!(
            code,
            ErrorCode::PluginDelegationOutsideManifestEnvelope,
            "napi-consumed seam must surface the typed Layer-3 deny; got {code:?}"
        ),
        other => panic!("expected typed Layer-3 deny; got {other:?} (AllPermit regression?)"),
    }
}
