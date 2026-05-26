//! **R6 R2 FP-B (L7-R2-F-2 closure)** — §4.37 InstallRecord replay-
//! defense exercised THROUGH `install_plugin(...)` end-to-end (not
//! just `InstallRecordReplayStore::record_and_check` direct).
//!
//! Pre-FP-B the §4.37 coverage at
//! `crates/benten-engine/tests/g_core_8_install_record_replay_atomic_record_and_check.rs`
//! exercised `InstallRecordReplayStore::record_and_check` in isolation
//! — none of its arms drove `install_plugin(...)` to verify the typed
//! `PluginInstallRecordAlreadyApplied` pipeline-rejection fires at the
//! Step-3b replay-check gate.
//!
//! This file pins the production-arm end-to-end: wire a closure over
//! a real `InstallRecordReplayStore::record_and_check` as the
//! `InstallPorts.install_record_replay_check`, run `install_plugin`
//! twice with the same install-record bytes, observe the second call
//! returns `Err(PluginInstallRecordAlreadyApplied)` pre-mint + the
//! library only contains the first install's entry.
//!
//! Per pim-18 §3.6f SUBSTANTIVE-arm-not-SHAPE: the test exercises the
//! production `install_plugin(...)` entry point + the canonical
//! closure pattern documented in the §S2 docstring (R6 R2 FP-B's
//! phantom-cite delete replaced
//! `make_engine_replay_check_closure(...)` with the inline closure
//! shape this test demonstrates).

#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;

use benten_core::Cid;
use benten_engine::install_record_replay::InstallRecordReplayStore;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::install_consent::AdmitAllInstallConsent;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, RendererBackend, RendererConfig, SharesPolicy, sign_manifest,
};

/// End-to-end pin: a real `InstallRecordReplayStore::record_and_check`
/// wired through `install_plugin(...)` rejects the second presentation
/// of the same install-record with typed
/// `PluginInstallRecordAlreadyApplied` pre-mint.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "Linear end-to-end positive-then-negative test wiring v1 install \
              + same-record second-presentation through install_plugin(...) — \
              boundary pin for §4.37 TOCTOU defense. Inlining keeps the v1 \
              admit + v1-replay reject ordering audit-able as a single test \
              body; helper-extraction would split the would-FAIL-on-revert \
              assertion across helpers and obscure the test's intent."
)]
fn install_record_replay_through_install_plugin_rejects_second_presentation() {
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Honest manifest.
    let mut manifest = PluginManifest {
        plugin_name: "replay-defense-test-plugin".to_string(),
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

    // PRODUCTION wiring shape (the §S2 docstring's canonical pattern):
    // wrap a real `InstallRecordReplayStore` in a closure satisfying
    // `InstallRecordReplayCheckFn`. Identical to what a production
    // engine call site would do.
    let replay_store = Arc::new(InstallRecordReplayStore::new());
    let store_clone = replay_store.clone();
    let mut replay_check =
        move |hash: &[u8; 32]| -> Result<(), ErrorCode> { store_clone.record_and_check(*hash) };

    // First install.
    let admit_policy_1 = AdmitAllInstallConsent;
    let mut cascade_1 = InMemoryInstallCascade::new();
    let mut private_ns_1 = InMemoryInstallCascade::new();
    let mut ports_1 = InstallPorts {
        cap_minter: &mut cascade_1,
        private_ns: &mut private_ns_1,
        install_record_replay_check: &mut replay_check,
        policy: &admit_policy_1,
    };
    let params_1 = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &[],
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did,
    };
    let first = install_plugin(
        &mut library,
        &mut store,
        &mut ports_1,
        &params_1,
        &bytes,
        &cid,
        &install_record,
        1,
        &|_| None,
    );
    assert!(
        first.is_ok(),
        "first install MUST succeed (replay store is empty): {first:?}"
    );
    assert_eq!(
        library.len(),
        1,
        "first install MUST commit one library entry"
    );

    // SECOND install (same install_record bytes) — replay defense
    // fires.
    let admit_policy_2 = AdmitAllInstallConsent;
    let mut cascade_2 = InMemoryInstallCascade::new();
    let mut private_ns_2 = InMemoryInstallCascade::new();
    let mut ports_2 = InstallPorts {
        cap_minter: &mut cascade_2,
        private_ns: &mut private_ns_2,
        install_record_replay_check: &mut replay_check,
        policy: &admit_policy_2,
    };
    let params_2 = InstallParams {
        now_secs: 1_700_000_001,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &[],
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did,
    };
    let second = install_plugin(
        &mut library,
        &mut store,
        &mut ports_2,
        &params_2,
        &bytes,
        &cid,
        &install_record,
        2,
        &|_| None,
    );

    // SUBSTANTIVE: the §4.37 closure-routed-through-install_plugin
    // fires. Would-FAIL-on-revert if the install_plugin Step-3b
    // replay-check is removed: the second install would be admitted +
    // a duplicate library entry would land (mint-twice violation of
    // §4.37 TOCTOU contract).
    assert_eq!(
        second,
        Err(ErrorCode::PluginInstallRecordAlreadyApplied),
        "§4.37 replay-defense: second install with same record MUST be \
         rejected with typed PluginInstallRecordAlreadyApplied; got: \
         {second:?}"
    );

    // Library state unchanged — second install does NOT commit.
    assert_eq!(
        library.len(),
        1,
        "§4.37 replay rejection MUST leave library unchanged (one \
         entry from the first install only)"
    );
}
