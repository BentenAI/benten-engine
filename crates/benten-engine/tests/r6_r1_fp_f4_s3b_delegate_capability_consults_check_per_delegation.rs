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
use benten_engine::{Engine, EngineError};
use benten_errors::ErrorCode;

const SOURCE_PLUGIN_DID: &str = "did:key:z6MkSourcePluginS3bSiblingHarnessAAAAAAAAAAAAAA";
const TARGET_PLUGIN_DID: &str = "did:key:z6MkTargetPluginS3bSiblingHarnessBBBBBBBBBBBBBB";

/// Custom policy that records every check_per_delegation call (with
/// observed-argument capture) + can deny on flag.
struct CountingPolicy {
    per_delegation_calls: Arc<AtomicUsize>,
    deny_per_delegation: bool,
    observed_source: std::sync::Mutex<Vec<String>>,
    observed_target: std::sync::Mutex<Vec<String>>,
    observed_scope: std::sync::Mutex<Vec<String>>,
}

impl CountingPolicy {
    fn new(deny: bool) -> Self {
        Self {
            per_delegation_calls: Arc::new(AtomicUsize::new(0)),
            deny_per_delegation: deny,
            observed_source: std::sync::Mutex::new(Vec::new()),
            observed_target: std::sync::Mutex::new(Vec::new()),
            observed_scope: std::sync::Mutex::new(Vec::new()),
        }
    }
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
        source: &str,
        target: &str,
        scope: &str,
    ) -> Result<(), CapError> {
        self.per_delegation_calls.fetch_add(1, Ordering::SeqCst);
        self.observed_source
            .lock()
            .unwrap()
            .push(source.to_string());
        self.observed_target
            .lock()
            .unwrap()
            .push(target.to_string());
        self.observed_scope.lock().unwrap().push(scope.to_string());
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

/// Construct a real Engine + install the supplied counting policy +
/// mint a user-rooted grant whose `actor` is the source plugin-DID.
/// Returns (engine, tempdir, source_grant_cid).
fn engine_with_policy_and_seeded_grant(
    policy: Arc<CountingPolicy>,
) -> (Engine, tempfile::TempDir, benten_core::Cid) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let path = tempdir.path().join("s3b-sibling.redb");
    let engine = benten_engine::EngineBuilder::new()
        .capability_policy(Box::new(CountingPolicyHandle(policy.clone())))
        .open(&path)
        .expect("engine opens with custom policy");

    // Mint the source grant: actor = SOURCE_PLUGIN_DID, non-private
    // scope so Step 2a (private-namespace clause) doesn't short-circuit.
    let source = engine
        .caps()
        .grant_capability(SOURCE_PLUGIN_DID, "store:s3b-sibling:write")
        .expect("mint source grant under custom policy");
    (engine, tempdir, source)
}

/// Wrapper that forwards the trait to the shared Arc<CountingPolicy>
/// (the engine takes a `Box<dyn CapabilityPolicy>` so the policy itself
/// must live behind a Box; we hand-thread the Arc for observation).
struct CountingPolicyHandle(Arc<CountingPolicy>);

impl benten_caps::__sealed_for_workspace_tests::Sealed for CountingPolicyHandle {}

impl CapabilityPolicy for CountingPolicyHandle {
    fn check_write(&self, ctx: &CapWriteContext) -> Result<(), CapError> {
        self.0.check_write(ctx)
    }
    fn check_read(&self, ctx: &ReadContext) -> Result<(), CapError> {
        self.0.check_read(ctx)
    }
    fn check_per_delegation(
        &self,
        source: &str,
        target: &str,
        scope: &str,
    ) -> Result<(), CapError> {
        self.0.check_per_delegation(source, target, scope)
    }
}

/// **§S3b arm 1 — production-arm: delegate_capability surfaces typed
/// PluginPerDelegationDenied on hook denial.**
/// Builds a real Engine with a denying policy, mints a source grant,
/// invokes `engine.caps().delegate_capability(...)`. The hook denial
/// must surface as the typed `PluginPerDelegationDenied` code at the
/// production boundary (engine_caps.rs:582).
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** delete the
/// `policy.check_per_delegation(...)` wiring at engine_caps.rs:572-590
/// → delegation admits → `expect_err` fails. Equivalent: re-introduce
/// the trait-direct `assert_eq!(code.as_str(), ...)` shape → the
/// production wiring is no longer exercised.
#[test]
fn delegate_capability_surfaces_plugin_per_delegation_denied_code() {
    let policy = Arc::new(CountingPolicy::new(/*deny=*/ true));
    let (engine, _td, source) = engine_with_policy_and_seeded_grant(policy.clone());

    let err = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[])
        .expect_err(
            "LOAD-BEARING: denying CapabilityPolicy::check_per_delegation MUST surface \
             typed PluginPerDelegationDenied at delegate_capability boundary; an \
             `Ok(cid)` here means the hook wiring at engine_caps.rs:572-590 was \
             reverted (Layer-3 §8-E hook #2 regression)",
        );

    match err {
        EngineError::Other { code, ref message } => {
            assert_eq!(
                code,
                ErrorCode::PluginPerDelegationDenied,
                "must surface typed PluginPerDelegationDenied; got {code:?}"
            );
            assert!(
                message.contains("check_per_delegation"),
                "diagnostic must reference the hook surface; got: {message}"
            );
        }
        other => panic!("expected EngineError::Other PluginPerDelegationDenied; got {other:?}"),
    }
    // Verify the hook actually FIRED at least once.
    assert!(
        policy.per_delegation_calls.load(Ordering::SeqCst) >= 1,
        "denying policy MUST have been consulted via delegate_capability; \
         per_delegation_calls={}",
        policy.per_delegation_calls.load(Ordering::SeqCst)
    );
}

/// **§S3b arm 2 — production-arm: counting policy observes hook
/// invocation through delegate_capability end-to-end.**
/// Admitting policy + real engine + real delegate_capability call →
/// per_delegation_calls increments + delegation succeeds.
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** delete the
/// `policy.check_per_delegation(...)` call at engine_caps.rs:572-590
/// → counter stays at 0 → assertion fires.
#[test]
fn check_per_delegation_observes_invocation_via_delegate_capability() {
    let policy = Arc::new(CountingPolicy::new(/*deny=*/ false));
    let (engine, _td, source) = engine_with_policy_and_seeded_grant(policy.clone());

    let _delegation_cid = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[])
        .expect("admitting CapabilityPolicy → delegation admits end-to-end");

    let observed = policy.per_delegation_calls.load(Ordering::SeqCst);
    assert!(
        observed >= 1,
        "LOAD-BEARING: production delegate_capability path MUST consult \
         check_per_delegation; observed={observed}"
    );

    // Verify the hook received the exact source-principal-DID, plugin-DID,
    // and scope strings the engine threaded through.
    let sources = policy.observed_source.lock().unwrap();
    let targets = policy.observed_target.lock().unwrap();
    let scopes = policy.observed_scope.lock().unwrap();
    assert!(
        sources.iter().any(|s| s == SOURCE_PLUGIN_DID),
        "source-principal-DID forwarding gap: observed_sources={sources:?}"
    );
    assert!(
        targets.iter().any(|t| t == TARGET_PLUGIN_DID),
        "target-plugin-DID forwarding gap: observed_targets={targets:?}"
    );
    assert!(
        scopes.iter().any(|s| s == "store:s3b-sibling:write"),
        "cap-scope forwarding gap: observed_scopes={scopes:?}"
    );
}

/// **§S3b arm 3 — production-arm: deny + admit divergence at
/// delegate_capability boundary.** Two engine instances differ ONLY in
/// the policy's `deny_per_delegation` flag; the admitting one returns
/// Ok(cid), the denying one returns Err. This pins the production path
/// observably-FAILS-CLOSED on hook denial vs admits-on-Ok.
#[test]
fn admit_vs_deny_diverges_at_delegate_capability_boundary() {
    // Admit path
    let admit_policy = Arc::new(CountingPolicy::new(/*deny=*/ false));
    let (engine_admit, _td1, source_admit) =
        engine_with_policy_and_seeded_grant(admit_policy.clone());
    let admit_result =
        engine_admit
            .caps()
            .delegate_capability(&source_admit, TARGET_PLUGIN_DID, &[]);
    assert!(
        admit_result.is_ok(),
        "admitting CapabilityPolicy → delegate_capability MUST admit; got {admit_result:?}"
    );

    // Deny path
    let deny_policy = Arc::new(CountingPolicy::new(/*deny=*/ true));
    let (engine_deny, _td2, source_deny) = engine_with_policy_and_seeded_grant(deny_policy.clone());
    let deny_result = engine_deny
        .caps()
        .delegate_capability(&source_deny, TARGET_PLUGIN_DID, &[]);
    assert!(
        deny_result.is_err(),
        "denying CapabilityPolicy → delegate_capability MUST deny; got {deny_result:?}"
    );

    // Both paths consulted the hook (the divergence is in the hook
    // return value, not whether the hook was called).
    assert!(admit_policy.per_delegation_calls.load(Ordering::SeqCst) >= 1);
    assert!(deny_policy.per_delegation_calls.load(Ordering::SeqCst) >= 1);
}

/// **§S3b arm 4 — production-arm: default impl admits delegation
/// end-to-end (via NoAuthBackend at the engine boundary).** Mirrors
/// existing precedent: a `NoAuthBackend`-policy engine successfully
/// delegates because the trait default for `check_per_delegation`
/// returns `Ok(())`.
#[test]
fn default_check_per_delegation_admits_via_engine_with_noauth_backend() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let path = tempdir.path().join("s3b-default-noauth.redb");
    let engine = benten_engine::EngineBuilder::new()
        .capability_policy(Box::new(benten_caps::NoAuthBackend::new()))
        .open(&path)
        .expect("engine opens with NoAuthBackend");
    let source = engine
        .caps()
        .grant_capability(SOURCE_PLUGIN_DID, "store:default-noauth:write")
        .expect("mint source grant under NoAuth");

    let delegated = engine
        .caps()
        .delegate_capability(&source, TARGET_PLUGIN_DID, &[])
        .expect(
            "default check_per_delegation (NoAuthBackend → trait default Ok(())) \
             MUST admit the delegation end-to-end",
        );
    assert!(!delegated.to_base32().is_empty());
}

/// **§S3b arm 5 — production-arm: hook fires AFTER Step-2b shares-
/// policy resolver (ordering pin).** A private-namespace scope short-
/// circuits at Step-2a with `PluginPrivateNamespaceDelegationForbidden`
/// BEFORE the policy hook is consulted, so a DENYING policy installed
/// for a private-namespace delegation still surfaces the
/// PRIVATE-NAMESPACE error (not PerDelegationDenied). Reverting the
/// wiring ordering would surface PerDelegationDenied here instead.
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** reorder so the policy hook
/// fires before Step-2a → typed code shifts from
/// `PluginPrivateNamespaceDelegationForbidden` to
/// `PluginPerDelegationDenied`.
#[test]
fn check_per_delegation_fires_after_private_namespace_clause() {
    let policy = Arc::new(CountingPolicy::new(/*deny=*/ true));
    let (engine, _td, _source_unused) = engine_with_policy_and_seeded_grant(policy.clone());

    // Mint a private-namespace grant (separate from the seeded one).
    let private_source = engine
        .caps()
        .grant_capability(SOURCE_PLUGIN_DID, "private:did:key:zPrivate:notes")
        .expect("mint private-namespace source grant");

    let err = engine
        .caps()
        .delegate_capability(&private_source, TARGET_PLUGIN_DID, &[])
        .expect_err("private-namespace delegation MUST reject");

    match err {
        EngineError::Other { code, .. } => {
            assert_eq!(
                code,
                ErrorCode::PluginPrivateNamespaceDelegationForbidden,
                "ORDERING PIN: private-namespace clause fires at Step-2a BEFORE \
                 the §S3b policy hook at engine_caps.rs:572-590; a typed code of \
                 PluginPerDelegationDenied here would indicate the ordering was \
                 reverted (policy hook moved BEFORE Step-2a). Got: {code:?}"
            );
        }
        other => panic!("expected EngineError::Other; got {other:?}"),
    }
}
