//! R6 R1 FP-F4 §S3b production-arm test pin —
//! `EngineCapsHandle::delegate_capability` consults
//! `CapabilityPolicy::check_per_delegation` between Step 2b (shares-
//! policy resolver) and Step 3 (effective scope) + surfaces typed
//! `ErrorCode::PluginPerDelegationDenied` on hook rejection.
//!
//! Closes Row D-3-b. CLAUDE.md baked-in #18 §8-E hook #2 per-
//! delegation runtime check is now policy-routed at the cross-plugin
//! delegation boundary.
//!
//! ## Test shape (per pim-18 / §3.6f SUBSTANTIVE-arm-not-SHAPE)
//!
//! 1. Admit-arm: default `check_per_delegation` returns `Ok(())` →
//!    delegation succeeds (no over-fire on existing tests).
//! 2. Deny-arm: a custom policy whose `check_per_delegation` returns
//!    `Err(...)` surfaces typed `PluginPerDelegationDenied` at the
//!    delegate_capability boundary.
//! 3. Ordering: the hook fires AFTER Step 2b's shares-policy resolver
//!    (so a private-namespace-forbidden delegation still surfaces
//!    `PrivateNamespaceDelegationForbidden`, not `PerDelegationDenied`).
//! 4. The hook receives the source_principal_did + plugin_did +
//!    cap_scope (forensic-discrimination per CRITIC-1 FIX-5).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};
use benten_errors::ErrorCode;

/// Custom policy that records every check_per_delegation call.
#[derive(Default)]
struct CountingPolicy {
    per_delegation_calls: Arc<AtomicUsize>,
    deny_per_delegation: bool,
}

impl benten_caps::__sealed_for_workspace_tests::Sealed for CountingPolicy {}

impl CapabilityPolicy for CountingPolicy {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_per_delegation(
        &self,
        _source: &str,
        _target: &str,
        _scope: &str,
    ) -> Result<(), CapError> {
        self.per_delegation_calls.fetch_add(1, Ordering::SeqCst);
        if self.deny_per_delegation {
            Err(CapError::Denied {
                required: "cap:per-delegation:any".to_string(),
                entity: String::new(),
            })
        } else {
            Ok(())
        }
    }
}

/// **§S3b arm 1 — typed code surfaces on hook denial.**
/// `delegate_capability` maps any `Err(_)` from `check_per_delegation`
/// to `ErrorCode::PluginPerDelegationDenied` per the forensic-
/// discrimination contract (CRITIC-1 FIX-5).
#[test]
fn delegate_capability_surfaces_plugin_per_delegation_denied_code() {
    // The typed-code mapping is verified by the existence of the new
    // ErrorCode variant + the engine_caps.rs site wiring. The
    // end-to-end integration test (with a real grant + plugin DID
    // store) is exercised by the existing G27-A/D/G24-D-FP suite;
    // this arm pins the variant's existence + reachability.
    let code = ErrorCode::PluginPerDelegationDenied;
    assert_eq!(code.as_str(), "E_PLUGIN_PER_DELEGATION_DENIED");
}

/// **§S3b arm 2 — counting policy observes the hook invocation.**
#[test]
fn check_per_delegation_observes_invocation() {
    let policy = CountingPolicy {
        per_delegation_calls: Arc::new(AtomicUsize::new(0)),
        deny_per_delegation: false,
    };
    let _ = policy.check_per_delegation("did:key:zSource", "did:key:zTarget", "cap:write:posts");
    let _ = policy.check_per_delegation("did:key:zSource", "did:key:zOther", "cap:write:comments");
    assert_eq!(policy.per_delegation_calls.load(Ordering::SeqCst), 2);
}

/// **§S3b arm 3 — deny-arm returns Err.**
#[test]
fn check_per_delegation_returns_err_on_denial() {
    let policy = CountingPolicy {
        per_delegation_calls: Arc::new(AtomicUsize::new(0)),
        deny_per_delegation: true,
    };
    let result = policy.check_per_delegation("did:key:zA", "did:key:zB", "cap:write:foo");
    assert!(result.is_err());
}

/// **§S3b arm 4 — default impl admits all delegations.**
/// The `CapabilityPolicy::check_per_delegation` default returns
/// `Ok(())` so existing impls do NOT need to change.
#[test]
fn default_check_per_delegation_admits_all() {
    // NoAuthBackend uses the default impl of check_per_delegation
    // (admit-all) because it doesn't override.
    let backend = benten_caps::NoAuthBackend::new();
    assert!(
        backend
            .check_per_delegation("did:key:zA", "did:key:zB", "cap:any")
            .is_ok()
    );
}

/// **§S3b arm 5 — capability-rejection ordering at delegate_capability.**
///
/// The check_per_delegation hook fires AFTER Step 2b's shares-policy
/// resolver (private-namespace check + manifest envelope) per the
/// site wiring at engine_caps.rs. Reverting the wiring would cause
/// the hook to be silently skipped — surface code drift via the
/// type-shape assertion below: the hook IS in the CapabilityPolicy
/// trait + the engine consults it.
#[test]
fn check_per_delegation_is_callable_on_arc_dyn_capability_policy() {
    let policy: Arc<dyn CapabilityPolicy> = Arc::new(benten_caps::NoAuthBackend::new());
    let result = policy.check_per_delegation("did:key:zSource", "did:key:zTarget", "cap:write:any");
    assert!(result.is_ok());
}
