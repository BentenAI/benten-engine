//! **R6 R2 FP-B (L7-R2-F-3 closure)** — §S3a deny-arm exercised THROUGH
//! `install_plugin(...)` end-to-end (not just `DenyAllInstallConsent`
//! direct trait-call).
//!
//! Pre-FP-B the §S3a coverage at
//! `r6_r1_fp_f4_s3a_install_pipeline_consults_check_install_consent.rs`
//! exercised the trait surface in isolation — none of its arms drove
//! `install_plugin(...)` to verify the typed
//! `PluginInstallConsentDenied` pipeline-rejection actually fires +
//! state stays uncommitted. This file pins the production-arm end-to-
//! end: wire `DenyAllInstallConsent` as the `InstallPorts.policy`,
//! observe `install_plugin` returns `Err(PluginInstallConsentDenied)`
//! pre-mint + the library + cascade observe ZERO state mutation.
//!
//! Per pim-18 §3.6f SUBSTANTIVE-arm-not-SHAPE: the test exercises the
//! `install_plugin(...)` production entry point with a real manifest +
//! real signed install-record; would-FAIL-on-revert if §S3a wiring is
//! removed (the deny would be silently bypassed + install commits).

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::install_consent::DenyAllInstallConsent;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, RendererBackend, RendererConfig, SharesPolicy, sign_manifest,
};

/// End-to-end pin: wiring `DenyAllInstallConsent` through
/// `install_plugin(...)` surfaces typed `PluginInstallConsentDenied`
/// pre-mint + leaves the library uncommitted.
#[test]
fn deny_all_install_consent_through_install_plugin_rejects_pre_mint() {
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Honest manifest (peer_did matches signing key — so we know any
    // rejection comes from §S3a, not Step-4 peer-sig verify).
    let mut manifest = PluginManifest {
        plugin_name: "consent-deny-test-plugin".to_string(),
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        peer_did: alice.public_key().to_did(),
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
    manifest.peer_signature = sign_manifest(&manifest, &alice);

    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let cid = manifest.content_cid;

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record =
        common::manifest_fixtures::signed_install_record(&user_kp, cid, plugin_did.clone(), 1);

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_replay_check = benten_platform_foundation::testing::noop_replay_check();
    let deny_policy = DenyAllInstallConsent;
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_replay_check,
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

    // SUBSTANTIVE: the §S3a hook fires + rejects with the typed code.
    // Would-FAIL-on-revert if the install_plugin call site at step 3c
    // is removed: deny would be silently bypassed.
    assert_eq!(
        result,
        Err(ErrorCode::PluginInstallConsentDenied),
        "§S3a: DenyAllInstallConsent wired through install_plugin MUST \
         surface typed PluginInstallConsentDenied; pre-FP-B coverage \
         only exercised the trait in isolation. Got: {result:?}"
    );

    // §4.35 zero partial-mint atomicity defense: rejected install
    // does NOT commit state.
    assert!(
        library.is_empty(),
        "§S3a denial MUST leave library uncommitted (zero partial-mint)"
    );
}
