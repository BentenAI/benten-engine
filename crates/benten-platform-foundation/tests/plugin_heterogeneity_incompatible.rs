//! G24-D row pin — heterogeneity contract (ds-r1-8).
//!
//! Phase-4-Meta-Core G-CORE-0 migration (plan §1.A.FROZEN item 7,
//! `docs/future/phase-4-backlog.md §4.33`, HARD-RULE-12 clause-(a)):
//! previously imported the deleted `module_ecosystem::install_plugin`
//! precursor; migrated to the canonical `plugin_lifecycle::
//! install_plugin` (the heterogeneity check is Step 5 of the 11-step
//! pipeline per CLAUDE.md #18).
//!
//! Per docs/PLUGIN-MANIFEST.md §3.1: if a plugin's requires include
//! `host:sandbox:exec` AND the installing peer is a thin-compute-
//! surface (browser / edge per CLAUDE.md #17), install fails with
//! `E_PLUGIN_HETEROGENEITY_INCOMPATIBLE`.

mod common;

use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, SharesPolicy, sign_manifest,
};

/// Build a properly-signed manifest by `author_kp` with the
/// host:sandbox:exec requires-scope. This is the migrated equivalent
/// of `common::manifest_fixtures::manifest_requires_sandbox_exec`,
/// re-signed against a real keypair so the canonical install pipeline's
/// Step 4 (validate_with_clock → verify_peer_signature) accepts the
/// signature + the heterogeneity gate (Step 5) becomes the load-bearing
/// rejection.
fn signed_sandbox_exec_manifest(author: &Keypair) -> PluginManifest {
    let mut manifest = PluginManifest {
        plugin_name: "sandbox-app".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        peer_did: author.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement {
            scope: "host:sandbox:exec".to_string(),
        }],
        shares: SharesPolicy::none(),
        renderer_config: None,
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
fn install_on_thin_compute_surface_with_sandbox_exec_require_fails_with_heterogeneity_error() {
    // SUBSTANTIVE per pim-2 §3.6b: build a properly-signed manifest
    // declaring `host:sandbox:exec`; pass installer_shape=ThinClient.
    // The canonical pipeline's Step 5 heterogeneity gate MUST reject
    // with typed PluginHeterogeneityIncompatible. Would-FAIL if Step 5
    // were skipped on the thin-compute-surface arm.
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = signed_sandbox_exec_manifest(&author);
    assert!(manifest.requires_sandbox_exec());

    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let cid = manifest.content_cid;
    let mut library = PluginLibrary::new();

    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record =
        common::manifest_fixtures::signed_install_record(&user_kp, cid, plugin_did.clone(), 1);

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_replay_check_1 = benten_platform_foundation::testing::noop_replay_check();
    let noauth_policy_1 = benten_platform_foundation::install_consent::AdmitAllInstallConsent;
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_replay_check_1,
        policy: &noauth_policy_1,
    };
    let params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::ThinClient,
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
        1_700_000_000_000_000_000,
        &|_| None,
    );

    let err = result.expect_err("thin-client + sandbox MUST reject");
    assert_eq!(
        err,
        ErrorCode::PluginHeterogeneityIncompatible,
        "Step 5 heterogeneity gate MUST surface typed \
         PluginHeterogeneityIncompatible; would-FAIL if Step 5 \
         skipped on ThinClient + host:sandbox:exec; got {err:?}"
    );

    // Defense-in-depth: rejection leaves library state UNCHANGED.
    assert!(library.is_empty(), "rejected install MUST NOT commit");
}

#[test]
fn full_peer_does_not_trigger_heterogeneity_gate() {
    // Complementary positive arm per pim-2 §3.6b: same manifest +
    // installer_shape=FullPeer; the heterogeneity gate is shape-
    // specific so this arm MUST admit. Would-FAIL if Step 5 fired
    // unconditionally.
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = signed_sandbox_exec_manifest(&author);
    assert!(manifest.requires_sandbox_exec());

    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let cid = manifest.content_cid;
    let mut library = PluginLibrary::new();

    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record =
        common::manifest_fixtures::signed_install_record(&user_kp, cid, plugin_did.clone(), 2);

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let mut noop_replay_check_2 = benten_platform_foundation::testing::noop_replay_check();
    let noauth_policy_2 = benten_platform_foundation::install_consent::AdmitAllInstallConsent;
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_replay_check_2,
        policy: &noauth_policy_2,
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
        1_700_000_000_000_000_000,
        &|_| None,
    );

    // FullPeer does NOT trigger heterogeneity — install admits even
    // for sandbox-exec-requiring manifests.
    assert!(
        outcome.is_ok(),
        "FullPeer install with sandbox-exec require MUST admit; \
         heterogeneity gate is shape-specific; got {outcome:?}"
    );
    assert_eq!(library.len(), 1);
}
