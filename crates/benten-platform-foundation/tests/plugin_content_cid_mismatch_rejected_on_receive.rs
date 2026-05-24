//! G24-D row pin — pull-model CID verification on receive.
//!
//! Phase-4-Meta-Core G-CORE-0 migration (plan §1.A.FROZEN item 7,
//! `docs/future/phase-4-backlog.md §4.33`, HARD-RULE-12 clause-(a)):
//! this file was previously importing the deleted
//! `module_ecosystem::install_plugin` precursor; migrated to the
//! canonical `plugin_lifecycle::install_plugin` (11-step pipeline with
//! full Layer-1 cap cascade + Layer-2 consent + Layer-3 envelope per
//! CLAUDE.md #18). The CID-mismatch arm is exercised by Step 1 of the
//! canonical pipeline (decode + verify content-CID).
//!
//! Per docs/PLUGIN-MANIFEST.md §4.1 step 2(a): receiver verifies bytes
//! hash to declared content-CID. Mismatch surfaces
//! `E_PLUGIN_CONTENT_CID_MISMATCH`.
//!
//! Defends against T6a substitution-at-transit attacks.

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, RendererBackend, RendererConfig, SharesPolicy, sign_manifest,
};

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

#[test]
fn install_path_rejects_bytes_with_announced_cid_mismatch_with_typed_error() {
    // SUBSTANTIVE per pim-2 §3.6b: build a real signed manifest;
    // pass `plugin_lifecycle::install_plugin` a DIFFERENT claimed CID.
    // Expect typed PluginContentCidMismatch. Would-FAIL if Step 1 of
    // the canonical pipeline skipped CID verification.
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = build_signed_manifest("test-app", &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");

    let mut library = PluginLibrary::new();
    // Claim a CID that does NOT match the manifest's actual CID.
    let bogus_cid = Cid::from_blake3_digest([0xEEu8; 32]);

    // Caller-mint-first per CLAUDE.md #18 + the canonical install
    // pipeline Step 8 contract.
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        bogus_cid,
        plugin_did.clone(),
        1,
    );

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_replay_check_1 = benten_platform_foundation::testing::noop_replay_check();
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_replay_check_1,
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
        &bogus_cid,
        &install_record,
        1,
        &|_| None,
    );
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("MUST reject CID mismatch"),
    };

    assert_eq!(
        err,
        ErrorCode::PluginContentCidMismatch,
        "install path MUST surface typed PluginContentCidMismatch; \
         would-FAIL if Step 1 CID verification skipped"
    );

    // Defense-in-depth: rejection leaves library state UNCHANGED.
    assert!(library.is_empty(), "rejected install MUST NOT commit");
}

#[test]
fn install_path_admits_bytes_when_announced_cid_matches_signed_manifest() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: complementary positive arm
    // — the CID verification check is not over-strict; matching CID
    // admits. Would-FAIL if verification rejected even matched CIDs.
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = build_signed_manifest("ok-app", &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");

    let mut library = PluginLibrary::new();
    let cid = manifest.content_cid;

    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record =
        common::manifest_fixtures::signed_install_record(&user_kp, cid, plugin_did.clone(), 2);

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_replay_check_2 = benten_platform_foundation::testing::noop_replay_check();
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_replay_check_2,
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
        &install_record,
        1,
        &|_| None,
    );
    assert!(outcome.is_ok(), "matched CID MUST admit: {outcome:?}");
    assert_eq!(library.len(), 1, "library now holds the entry");
}
