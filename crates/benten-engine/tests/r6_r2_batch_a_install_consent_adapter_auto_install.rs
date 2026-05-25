//! **R6 R2 batch-A Item 2 (Row D-3-a substantive close)** —
//! `Engine::install_consent_adapter()` returns a ready-to-use
//! [`InstallConsentPolicy`] port that wraps the engine's configured
//! [`CapabilityPolicy`] so the install pipeline's `&dyn` port consults
//! a custom policy's `check_install_consent` end-to-end.
//!
//! ## Why this test exists (L6-r6r2-l6-1 substantive close)
//!
//! Pre-batch-A: the [`CapabilityPolicyInstallConsent`] adapter was
//! minted at R6-R2-FP-B (commit `a9150ef0`) BUT there was no public
//! accessor that auto-built the adapter from the engine's configured
//! policy. Callers had to (a) somehow extract the
//! `Arc<dyn CapabilityPolicy>` from the engine — which is
//! `pub(crate)`-sealed per Class B β — OR (b) reconstruct an
//! adapter from a separately-held `Arc`. Either way the engine's
//! own configured policy was NOT structurally connected to the
//! install pipeline; a custom `CapabilityPolicy` impl passed via
//! `EngineBuilder::capability_policy(...)` had its
//! `check_install_consent` ignored at the install boundary.
//!
//! Post-batch-A: `Engine::install_consent_adapter()` auto-wraps the
//! configured policy (or a `NoAuthBackend` default), so a single line
//! of caller code produces an adapter that observably routes
//! install-time consent through the engine's configured policy.
//!
//! ## §3.6f SUBSTANTIVE-arm test discipline
//!
//! This test:
//! 1. Constructs an `Engine` via the production builder + a custom
//!    `CountingDenyPolicy` impl of `CapabilityPolicy`.
//! 2. Pulls the install-consent adapter via
//!    `Engine::install_consent_adapter()`.
//! 3. Wires it into `InstallPorts.policy` AND drives a full
//!    `install_plugin(...)` call.
//! 4. Asserts: (a) the custom policy's hook was invoked
//!    AT LEAST ONCE with the install record's payload hash + the
//!    plugin-DID string, AND (b) the install fails with typed
//!    `ErrorCode::PluginInstallConsentDenied`.
//!
//! ## Would-FAIL-on-revert
//!
//! Revert `Engine::install_consent_adapter` to wrap
//! `NoAuthBackend::default()` regardless of the configured policy
//! (i.e. drop the `self.policy.as_ref()` branch and always construct
//! a `NoAuthBackend`-wrapping adapter) → the deny arm becomes an
//! admit + the install succeeds → both assertions fire.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy};
use benten_core::Cid;
use benten_engine::EngineBuilder;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, RendererBackend, RendererConfig, SharesPolicy, sign_manifest,
};
use benten_platform_foundation::testing::noop_replay_check;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom `CapabilityPolicy` impl that (a) DENIES every install
/// consent + (b) records the invocation count + arguments observed.
/// Cross-confirms that the engine's adapter routes calls through to
/// this impl rather than substituting `NoAuthBackend`-default-admit.
#[derive(Debug)]
struct CountingDenyPolicy {
    calls: Arc<AtomicUsize>,
    observed_hashes: Arc<Mutex<Vec<[u8; 32]>>>,
    observed_plugin_dids: Arc<Mutex<Vec<String>>>,
}

impl benten_caps::__sealed_for_workspace_tests::Sealed for CountingDenyPolicy {}

impl CapabilityPolicy for CountingDenyPolicy {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        // Not exercised by this test (install pipeline doesn't go
        // through check_write at the consent boundary).
        Ok(())
    }

    fn check_install_consent(&self, hash: &[u8; 32], plugin_did: &str) -> Result<(), CapError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.observed_hashes.lock().unwrap().push(*hash);
        self.observed_plugin_dids
            .lock()
            .unwrap()
            .push(plugin_did.to_string());
        Err(CapError::Denied {
            required: "install:consent".into(),
            entity: plugin_did.to_string(),
        })
    }
}

/// Build a signed manifest fixture for the install pipeline.
fn build_signed_manifest(name: &str, author: &Keypair) -> PluginManifest {
    let mut manifest = PluginManifest {
        plugin_name: name.to_string(),
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        peer_did: author.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("store:notes:read")],
        shares: SharesPolicy::none(),
        renderer_config: Some(RendererConfig {
            output_format: "html_json".to_string(),
            renderer_backends: Some(vec![RendererBackend::BrowserWasm32]),
            hosting_target: None,
            bundle_size_budget_kb: Some(256),
        }),
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    manifest.content_cid = manifest.compute_content_cid();
    manifest.peer_signature = sign_manifest(&manifest, author);
    manifest
}

/// **§3.6f SUBSTANTIVE-arm: production-entry-point invoked, observable
/// consequence, would-FAIL-on-revert.**
///
/// Asserts that a custom `CapabilityPolicy::check_install_consent`
/// passed via `EngineBuilder::capability_policy(...)` is consulted
/// by `install_plugin(...)` AT LEAST ONCE — when the adapter
/// constructed via `engine.install_consent_adapter()` is threaded
/// through `InstallPorts.policy`. Closes L6-r6r2-l6-1 substantively
/// (pre-batch-A: zero production callers of the adapter that wrap
/// the engine's CONFIGURED policy).
#[test]
fn install_consent_adapter_routes_through_configured_capability_policy_end_to_end() {
    let calls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let observed_hashes: Arc<Mutex<Vec<[u8; 32]>>> = Arc::new(Mutex::new(Vec::new()));
    let observed_plugin_dids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let policy = CountingDenyPolicy {
        calls: Arc::clone(&calls),
        observed_hashes: Arc::clone(&observed_hashes),
        observed_plugin_dids: Arc::clone(&observed_plugin_dids),
    };
    let engine = EngineBuilder::new()
        .capability_policy(Box::new(policy))
        .open(":memory:")
        .expect("in-memory engine");

    // PRODUCTION ENTRY POINT: pull the adapter via the new accessor.
    let adapter = engine.install_consent_adapter();

    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = build_signed_manifest("batch-a-item-2", &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode manifest");
    let cid = manifest.content_cid;

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let handle = benten_id::plugin_did::mint();
    let plugin_did = handle.did().clone();
    store.insert(handle).expect("insert plugin-DID");

    let mut record = benten_platform_foundation::InstallRecord {
        manifest_cid: cid,
        plugin_did: plugin_did.clone(),
        consenting_user_did: user_did.clone(),
        user_signature: vec![0u8; 64],
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![0xA5u8; 16],
        granted_caps_bytes: vec![],
    };
    let payload = record.signing_payload();
    record.user_signature = user_kp.sign(&payload).to_bytes().to_vec();

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_check = noop_replay_check();
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_check,
        policy: &adapter,
    };
    let params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &[],
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did,
    };

    let outcome = install_plugin(
        &mut library,
        &mut store,
        &mut ports,
        &params,
        &bytes,
        &cid,
        &record,
        1,
        &|_| None,
    );

    // ASSERTION 1: the custom policy's hook was invoked AT LEAST ONCE.
    // Revert proof: if `install_consent_adapter` wraps
    // `NoAuthBackend::default()` instead of the configured policy,
    // calls stays at 0 here.
    let observed_calls = calls.load(Ordering::SeqCst);
    assert!(
        observed_calls >= 1,
        "LOAD-BEARING: install_plugin MUST invoke the engine's CONFIGURED \
         CapabilityPolicy::check_install_consent via install_consent_adapter; \
         observed calls={observed_calls}"
    );

    // ASSERTION 2: the install denied with typed PluginInstallConsentDenied.
    // Revert proof: if the adapter wraps NoAuthBackend, the install admits.
    assert_eq!(
        outcome.unwrap_err(),
        ErrorCode::PluginInstallConsentDenied,
        "LOAD-BEARING: configured policy's Err must map to typed \
         PluginInstallConsentDenied at the install boundary"
    );

    // ASSERTION 3: the policy observed the install record's payload
    // hash + plugin-DID string (forensic-discrimination).
    let observed_dids = observed_plugin_dids.lock().unwrap();
    assert!(
        observed_dids.iter().any(|d| d == &plugin_did.to_string()),
        "LOAD-BEARING: policy must observe the install record's plugin_did \
         string; observed={observed_dids:?}"
    );
    let observed_h = observed_hashes.lock().unwrap();
    assert!(
        !observed_h.is_empty(),
        "LOAD-BEARING: policy must observe at least one install-record \
         payload hash; observed={observed_h:?}"
    );
}

/// Companion arm: when NO custom policy is configured on the
/// engine, `install_consent_adapter()` wraps a `NoAuthBackend` and
/// the install proceeds past the consent gate (admit-all default
/// per `CapabilityPolicy::check_install_consent` doc).
///
/// Revert proof: if the no-policy branch panicked or returned a
/// non-admitting wrapper, install would fail at consent.
#[test]
fn install_consent_adapter_admits_when_no_policy_configured() {
    let engine = EngineBuilder::new()
        .open(":memory:")
        .expect("in-memory engine");
    let adapter = engine.install_consent_adapter();

    // Calling the adapter directly verifies the admit-all default
    // (a full install_plugin run would also succeed but is overkill
    // for the wrapping-default verification).
    use benten_platform_foundation::install_consent::InstallConsentPolicy;
    let hash = [0u8; 32];
    assert!(
        adapter
            .check_install_consent(&hash, "did:key:zTestDefault")
            .is_ok(),
        "no-policy engine: adapter wraps NoAuthBackend; admit-all default"
    );
}
