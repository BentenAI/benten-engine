//! Phase-4-Foundation R4-FP-1 — T5b LOAD-BEARING pin: plugin content
//! substitution at install rejected (peer-DID signature check).
//!
//! Phase-4-Meta-Core G-CORE-0 migration (plan §1.A.FROZEN item 7,
//! `docs/future/phase-4-backlog.md §4.33`, HARD-RULE-12 clause-(a)):
//! the first two arms previously imported the deleted
//! `module_ecosystem::install_plugin` precursor; all three arms now
//! route through the canonical `plugin_lifecycle::install_plugin`
//! (Step 4 `validate_with_clock → verify_peer_signature` is the
//! load-bearing peer-signature defense).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T5
//! (T5b sub-threat) + defense step 3 (DID resolution for peer-DID at
//! install).
//!
//! ## What this pin establishes
//!
//! Per threat-model §T5b: "Attacker intercepts plugin content delivery
//! and substitutes bytes signed by a DIFFERENT peer-DID. If install
//! flow doesn't validate the peer-DID signature against the user's
//! known-trusted-author list, user installs hostile content under the
//! same human-readable name."
//!
//! Defense: the canonical install pipeline Step 4 (validate_with_clock,
//! which delegates to verify_peer_signature internally) verifies the
//! signature was produced by the claimed peer-DID's secret key. A
//! forged-claim signature (where bytes were signed by a different key
//! than the claimed `peer_did`) FAILS verification and surfaces typed
//! `E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID`.
//!
//! Sibling to `plugin_content_carries_peer_did_signature_for_provenance.rs`
//! (existing R3 pin) — that pins the POSITIVE arm; this pin is the
//! NEGATIVE rejection arm per pim-2 §3.6b sub-rule 4.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires content-CID match check but skips peer-DID
//! signature verification. Attacker constructs hostile content; signs
//! with attacker's key; manifest claims peer-DID = alice (a trusted
//! author); install flow accepts because CID matches own bytes.
//! Substitution attack succeeds.

#![allow(clippy::unwrap_used)]

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

#[test]
fn plugin_install_with_content_substituted_by_attacker_peer_did_rejected() {
    // SUBSTANTIVE per pim-2 §3.6b: build a manifest CLAIMING alice's
    // peer_did but sign with attacker's keypair. The canonical install
    // pipeline's Step 4 verify_peer_signature MUST reject with typed
    // PluginContentPeerSignatureInvalid. Would-FAIL if Step 4 skipped
    // peer-DID signature verification (T5b substitution attack
    // succeeds silently).
    let alice = Keypair::generate();
    let attacker = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    assert_ne!(
        alice.public_key().to_did(),
        attacker.public_key().to_did(),
        "test setup: distinct keypairs"
    );

    // Hostile manifest: claims peer_did = alice's DID, but actually
    // signed by attacker.
    let mut manifest = PluginManifest {
        plugin_name: "popular-plugin".to_string(),
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        // ATTACK: claim alice authored this
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
    // ATTACK: sign with attacker's key (not alice's).
    manifest.peer_signature = sign_manifest(&manifest, &attacker);

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
        &cid,
        &install_record,
        1,
        &|_| None,
    );

    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("T5b: substitution attack MUST be rejected"),
    };

    // SUBSTANTIVE: typed peer-signature-invalid (per arch-r1-3
    // ErrorCode split). Would-FAIL if verify_peer_signature was a
    // no-op (CID matches own bytes, but key-vs-claimed-DID mismatch
    // is the load-bearing check).
    assert_eq!(
        err,
        ErrorCode::PluginContentPeerSignatureInvalid,
        "T5b LOAD-BEARING: forged-claim attack MUST surface typed \
         PluginContentPeerSignatureInvalid"
    );

    // Defense-in-depth: rejected install does NOT commit state.
    assert!(library.is_empty(), "T5b: rejected install MUST NOT commit");
}

#[test]
fn plugin_install_admits_bytes_when_peer_did_matches_signing_key() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: complementary positive arm
    // - if claimed peer_did matches the actual signing key, the
    // canonical install pipeline admits. Would-FAIL if Step 4
    // verify_peer_signature was over-strict.
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let mut manifest = PluginManifest {
        plugin_name: "honest-plugin".to_string(),
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
    assert!(
        outcome.is_ok(),
        "claimed-peer-DID-matches-signature MUST admit: {outcome:?}"
    );
    assert_eq!(library.len(), 1);
}

#[test]
#[allow(clippy::too_many_lines)]
fn unknown_author_install_surfaces_e_plugin_author_not_trusted_for_user_prompt() {
    // **R4b-FP-1 Seam 1** un-ignore — substantive trust-list arm in
    // plugin_lifecycle::install_plugin. Two arms per pim-2 §3.6b:
    //  (a) NEGATIVE — alice authors; trust-list does NOT contain alice
    //      → typed PluginAuthorNotTrusted.
    //  (b) POSITIVE — same manifest, trust-list contains alice → admits.
    let alice = Keypair::generate();
    let trusted_author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Honestly-signed manifest by alice.
    let mut manifest = PluginManifest {
        plugin_name: "unknown-author-test".to_string(),
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        peer_did: alice.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("store:notes:read")],
        shares: SharesPolicy::none(),
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    manifest.content_cid = manifest.compute_content_cid();
    manifest.peer_signature = sign_manifest(&manifest, &alice);

    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;

    // R6-FP-A-fp caller-mint-first: pre-mint + insert real plugin-DID
    // handles per the new install_plugin Step 8 contract.
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_unknown = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_unknown.clone(),
        2,
    );

    // ARM (a) — trust-list does NOT contain alice.
    let mut library = PluginLibrary::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list = vec![trusted_author.public_key().to_did()];
    let mut noop_replay_check_3 = benten_platform_foundation::testing::noop_replay_check();
    let mut ctx = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut noop_replay_check_3,
    };
    let ctx_params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_unknown,
    };
    let attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &ctx_params,
        &bytes,
        &expected_cid,
        &install_record,
        1,
        &|_| None,
    );
    let err = attempt.expect_err("unknown-author install MUST be rejected by trust-list");
    assert_eq!(
        err,
        ErrorCode::PluginAuthorNotTrusted,
        "Seam 1 trust-list arm: unknown-author install MUST surface typed \
         PluginAuthorNotTrusted; would-FAIL if seam skipped trust-list check"
    );
    assert!(
        library.is_empty(),
        "trust-list-rejected install MUST NOT commit"
    );

    // ARM (b) — POSITIVE: trust-list contains alice.
    let mut library2 = PluginLibrary::new();
    let mut store2 = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_arm_b = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store2);
    let install_record_arm_b = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_arm_b.clone(),
        3,
    );
    let mut cascade2 = InMemoryInstallCascade::new();
    let mut private_ns2 = InMemoryInstallCascade::new();
    let trust_list_with_alice = vec![alice.public_key().to_did()];
    let mut noop_replay_check_4 = benten_platform_foundation::testing::noop_replay_check();
    let mut ctx2 = InstallPorts {
        cap_minter: &mut cascade2,
        private_ns: &mut private_ns2,
        install_record_replay_check: &mut noop_replay_check_4,
    };
    let ctx2_params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list_with_alice,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_arm_b,
    };
    install_plugin(
        &mut library2,
        &mut store2,
        &mut ctx2,
        &ctx2_params,
        &bytes,
        &expected_cid,
        &install_record_arm_b,
        1,
        &|_| None,
    )
    .expect("alice-in-trust-list install MUST admit");
    assert_eq!(library2.len(), 1);
}
