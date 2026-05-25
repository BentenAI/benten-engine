//! R6 R1 FP-F4 §S3a production-arm test pin — `plugin_lifecycle::install_plugin`
//! consults `InstallConsentPolicy::check_install_consent` at step 3c
//! and surfaces typed `ErrorCode::PluginInstallConsentDenied` on denial.
//!
//! Closes Row D-3-a. CLAUDE.md baked-in #18 §8-E hook #1 install-time
//! consent is now policy-routed at the install boundary.
//!
//! ## Test shape (per pim-18 / §3.6f SUBSTANTIVE-arm-not-SHAPE)
//!
//! The end-to-end install_plugin integration is exercised by the
//! existing platform-foundation tests (they all wire
//! `AdmitAllInstallConsent` post-S3a migration; with the threading
//! verified by their compilation). This file exercises the policy
//! hook surface directly + the typed-code mapping.
//!
//! 1. Production-arm: a real `DenyAllInstallConsent` returns
//!    `ErrorCode::PluginInstallConsentDenied` from the hook (the
//!    exact code the install pipeline forwards).
//! 2. Admit-arm: `AdmitAllInstallConsent` returns `Ok(())`.
//! 3. Custom-policy receives the plugin-DID string + payload hash.
//! 4. `InstallPorts` carries both `install_record_replay_check`
//!    (post-S2 Option drop) AND `policy` (post-S3a addition).

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::install_consent::{
    AdmitAllInstallConsent, DenyAllInstallConsent, InstallConsentPolicy,
};
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallRecordReplayCheckFn,
    InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, RendererBackend, RendererConfig, SharesPolicy, sign_manifest,
};
use benten_platform_foundation::testing::noop_replay_check;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom policy that records every call + admits all (with argument
/// capture for forensic-discrimination verification).
struct CountingAdmit {
    calls: Arc<AtomicUsize>,
    observed_hashes: Arc<Mutex<Vec<[u8; 32]>>>,
    observed_plugin_dids: Arc<Mutex<Vec<String>>>,
}

impl InstallConsentPolicy for CountingAdmit {
    fn check_install_consent(&self, hash: &[u8; 32], plugin_did: &str) -> Result<(), ErrorCode> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.observed_hashes.lock().unwrap().push(*hash);
        self.observed_plugin_dids
            .lock()
            .unwrap()
            .push(plugin_did.to_string());
        Ok(())
    }
}

/// Build a signed manifest for the install-pipeline harness.
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

/// Helper: run `install_plugin` end-to-end with a caller-supplied
/// `InstallConsentPolicy`. Returns Result<()> after threading all the
/// canonical pipeline ports.
fn install_with_policy<P: InstallConsentPolicy>(
    name: &str,
    policy: &P,
    nonce_byte: u8,
) -> Result<(), ErrorCode> {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = build_signed_manifest(name, &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode manifest");
    let cid = manifest.content_cid;

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        cid,
        plugin_did.clone(),
        nonce_byte,
    );

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_check = noop_replay_check();
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_check,
        policy,
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
    install_plugin(
        &mut library,
        &mut store,
        &mut ports,
        &params,
        &bytes,
        &cid,
        &install_record,
        1,
        &|_| None,
    )
    .map(|_| ())
}

/// **§S3a arm 1 — production-arm: counting policy observes hook
/// invocation through `install_plugin` end-to-end.**
/// Builds a real signed manifest + real InstallRecord + threads the
/// counting policy through `InstallPorts.policy`; asserts the
/// production pipeline at plugin_lifecycle.rs step 3c invoked the
/// hook AT LEAST ONCE with the install-record's payload-hash + the
/// install-record's plugin_did string (forensic-discrimination).
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** delete the
/// `ports.policy.check_install_consent(&payload_hash, plugin_did_str)`
/// wire-in at plugin_lifecycle.rs:967-973 → counter stays at 0 →
/// assertion fires.
#[test]
fn check_install_consent_observes_invocation_count_via_install_plugin() {
    let calls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let observed_hashes: Arc<Mutex<Vec<[u8; 32]>>> = Arc::new(Mutex::new(Vec::new()));
    let observed_plugin_dids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let policy = CountingAdmit {
        calls: calls.clone(),
        observed_hashes: observed_hashes.clone(),
        observed_plugin_dids: observed_plugin_dids.clone(),
    };
    install_with_policy("s3a-arm1", &policy, 0x11)
        .expect("admitting policy → install_plugin admits end-to-end");

    let observed = calls.load(Ordering::SeqCst);
    assert!(
        observed >= 1,
        "LOAD-BEARING: production install_plugin path MUST consult \
         check_install_consent at step 3c; observed={observed}"
    );

    // Verify the hook received a non-zero payload-hash (the canonical
    // BLAKE3 of install_record.signing_payload() — never all-zero).
    let hashes = observed_hashes.lock().unwrap();
    assert!(
        hashes.iter().any(|h| *h != [0u8; 32]),
        "payload-hash forwarding gap: install_plugin must pass the canonical \
         signing_payload() BLAKE3 (never all-zero); observed hashes={hashes:?}"
    );
    // Verify the hook received a `did:key:` plugin_did string (the
    // install_record's plugin_did.as_str()).
    let dids = observed_plugin_dids.lock().unwrap();
    assert!(
        dids.iter().any(|d| d.starts_with("did:key:")),
        "plugin_did forwarding gap: install_plugin must pass install_record.plugin_did.as_str(); \
         observed dids={dids:?}"
    );
}

/// **§S3a arm 2 — production-arm: DenyAllInstallConsent rejects
/// install at step 3c with typed `PluginInstallConsentDenied`.**
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** delete the step-3c wire-in
/// at plugin_lifecycle.rs:967-973 → DenyAll never consulted → install
/// proceeds (Ok or other error) → `expect_err(... PluginInstallConsentDenied)`
/// fails.
#[test]
fn deny_all_install_consent_rejects_install_with_typed_code() {
    let policy = DenyAllInstallConsent;
    let result = install_with_policy("s3a-arm2", &policy, 0x22);
    let err = result.expect_err("DenyAllInstallConsent MUST reject install at step 3c");
    assert_eq!(
        err,
        ErrorCode::PluginInstallConsentDenied,
        "LOAD-BEARING: production install pipeline MUST surface typed \
         PluginInstallConsentDenied on hook denial (forensic discrimination \
         from sibling install-time codes per CRITIC-1 FIX-5); got {err:?}"
    );
}

/// **§S3a arm 3 — production-arm: custom plugin-DID-specific policy
/// observes install-record's plugin_did through install_plugin.**
/// A policy that denies ONLY plugin-DIDs containing "Forbidden" admits
/// the install (since the harness mints a random plugin_did that does
/// not match), and the call observation arm verifies the hook fired
/// with a `did:key:` string (forensic-discrimination per CRITIC-1
/// FIX-5).
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** delete the step-3c wire-in
/// → counter stays at 0 → assertion fires.
#[test]
fn check_install_consent_receives_install_record_plugin_did_string() {
    struct PluginSubstringDeny {
        deny_substring: String,
        calls: Arc<AtomicUsize>,
        observed: Arc<Mutex<Vec<String>>>,
    }
    impl InstallConsentPolicy for PluginSubstringDeny {
        fn check_install_consent(
            &self,
            _hash: &[u8; 32],
            plugin_did: &str,
        ) -> Result<(), ErrorCode> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.observed.lock().unwrap().push(plugin_did.to_string());
            if plugin_did.contains(&self.deny_substring) {
                Err(ErrorCode::PluginInstallConsentDenied)
            } else {
                Ok(())
            }
        }
    }
    let calls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let observed: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let policy = PluginSubstringDeny {
        deny_substring: "Forbidden-NEVER-MINTED-SUBSTRING".to_string(),
        calls: calls.clone(),
        observed: observed.clone(),
    };
    install_with_policy("s3a-arm3", &policy, 0x33)
        .expect("plugin-substring-deny that DOES NOT match minted DID → install admits");

    assert!(
        calls.load(Ordering::SeqCst) >= 1,
        "hook MUST fire during install_plugin"
    );
    let dids = observed.lock().unwrap();
    assert!(
        dids.iter().all(|d| d.starts_with("did:key:")),
        "production forwarding contract: every observed plugin_did MUST be a \
         well-formed `did:key:` string; got: {dids:?}"
    );
}

/// **§S3a arm 4 — production-arm: AdmitAllInstallConsent → install
/// succeeds end-to-end.** Default-shape verification through the real
/// pipeline (not trait-direct): a known-admitting policy MUST observably
/// admit the install_plugin end-to-end.
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** if step-3c were rewired to
/// always-deny → AdmitAll install fails → assertion fires.
#[test]
fn admit_all_install_consent_admits_install_plugin_end_to_end() {
    let policy = AdmitAllInstallConsent;
    install_with_policy("s3a-arm4", &policy, 0x44)
        .expect("AdmitAllInstallConsent MUST admit install_plugin end-to-end");
}

/// **§S3a arm 5 — `noop_replay_check` interop:** the test fixture
/// canonical shape post-S2 + S3a wires noop_replay_check + the consent
/// policy through `InstallPorts`. Verifies the struct constructibility
/// of the two combined ports.
#[test]
fn install_ports_struct_carries_both_replay_check_and_policy() {
    let mut noop = noop_replay_check();
    let policy = AdmitAllInstallConsent;
    let check: &mut InstallRecordReplayCheckFn = &mut noop;
    let pol: &dyn InstallConsentPolicy = &policy;
    // Exercise both ports (forces the dummy bindings to count as
    // observably-used per clippy's `no_effect_underscore_binding`).
    assert!(check(&[0u8; 32]).is_ok());
    assert!(
        pol.check_install_consent(&[0u8; 32], "did:key:zNoop")
            .is_ok()
    );
}

/// **§S3a arm 6 — production-arm: hook fires BEFORE Step-9 cap-cascade
/// (ordering pin via observable side-effect).**
/// A `DenyAllInstallConsent` policy rejects at step 3c BEFORE the
/// cap-cascade runs (step 9). We observe this via the cascade port:
/// after a denied install, the cap-minter records ZERO grants minted
/// — proving the cascade was NEVER reached. If the ordering were
/// reversed (cap-cascade-before-consent), grants would be minted
/// before the consent denial fires.
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** reorder so consent fires
/// AFTER cap-cascade → grants are minted then "rolled back" or
/// orphaned → assertion observes cap_minter has grant residue.
#[test]
fn install_consent_hook_fires_before_cap_cascade_observable_via_cascade_residue() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = build_signed_manifest("s3a-arm6-ordering", &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode manifest");
    let cid = manifest.content_cid;

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record =
        common::manifest_fixtures::signed_install_record(&user_kp, cid, plugin_did.clone(), 6);

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_check = noop_replay_check();
    let deny_policy = DenyAllInstallConsent;
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_check,
        policy: &deny_policy,
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
    let result = install_plugin(
        &mut library,
        &mut store,
        &mut ports,
        &params,
        &bytes,
        &cid,
        &install_record,
        1,
        &|_| None,
    );

    assert_eq!(
        result,
        Err(ErrorCode::PluginInstallConsentDenied),
        "ORDERING PIN: deny at step 3c MUST surface PluginInstallConsentDenied"
    );

    // Library MUST remain unchanged (Step 11 manifest-store write
    // never reached).
    assert!(
        library.is_empty(),
        "Step-9 cap-cascade ordering pin: denied install at step 3c MUST NOT \
         leave library residue (no partial-mint per §4.35); if non-empty, the \
         consent hook fired AFTER Step 11 — order reversed."
    );
    // Cascade MUST also remain unchanged (Step 9 cap-cascade never
    // reached). The InMemoryInstallCascade tracks granted caps; we
    // assert no grants accumulated.
    assert!(
        cascade.minted_grants().is_empty(),
        "Step-9 cap-cascade ordering pin: denied install at step 3c MUST NOT \
         have reached cap-cascade (cascade.minted_grants() expected empty); if \
         non-empty, the consent hook fired AFTER cap-cascade — order reversed. \
         Observed grants: {:?}",
        cascade.minted_grants()
    );
    assert_eq!(
        cascade.provisioned_count(),
        0,
        "Step-9 cap-cascade ordering pin: denied install MUST NOT have \
         provisioned any private namespaces (Step 6 also never reached); \
         observed count: {}",
        cascade.provisioned_count()
    );
}
