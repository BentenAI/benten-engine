//! G24-D row + CLAUDE.md #18 Layer 2 — install-time consent required.
//!
//! User reviews `requires` + `shares` at install; consents to envelope.
//! Without an install record carrying valid user-DID signature, the
//! consent gate (`verify_install_record`) fails with typed
//! `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID`.
//!
//! Per pim-2-amendment §3.6b sub-rule 4: this pin exercises the
//! per-finding consent-gate arm specifically (sibling to umbrella
//! `install_record_signed_by_user_did_verifies` which asserts the
//! POSITIVE arm).

mod common;

use benten_errors::ErrorCode;
use benten_platform_foundation::module_ecosystem::verify_install_record;
use benten_platform_foundation::plugin_manifest::InstallRecord;
use common::manifest_fixtures::{stub_cid_zero, stub_user_did};

#[test]
fn install_record_with_zero_byte_signature_rejected_with_typed_consent_error() {
    // SUBSTANTIVE per pim-2 §3.6b + pim-2-amendment sub-rule 4:
    // construct an InstallRecord with zero-byte user_signature;
    // exercise the consent gate (verify_install_record); expect typed
    // PluginInstallRecordUserSignatureInvalid. Would-FAIL if the gate
    // skipped signature length validation (the structural pre-check
    // per arch-r1-3 ErrorCode split).
    let record = InstallRecord {
        manifest_cid: stub_cid_zero(),
        plugin_did: benten_id::did::Did::from_string_unchecked(
            "did:key:z6MkPluginConsentTest".to_string(),
        ),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![0u8; 16],
        consenting_user_did: stub_user_did(),
        user_signature: vec![], // EMPTY signature -> consent gate rejects
        granted_caps_bytes: vec![],
    };

    let err = verify_install_record(&record).expect_err("MUST reject empty signature");
    assert_eq!(
        err,
        ErrorCode::PluginInstallRecordUserSignatureInvalid,
        "consent gate MUST surface typed PluginInstallRecordUserSignatureInvalid; \
         would-FAIL if length check skipped"
    );
}

#[test]
fn install_record_with_malformed_64_byte_signature_rejected_at_consent_gate() {
    // SUBSTANTIVE per pim-2 §3.6b + pim-2-amendment sub-rule 4:
    // signature length passes the 64-byte structural check but the
    // bytes are not a valid Ed25519 signature for the payload; the
    // verify_install_record gate MUST reject. Would-FAIL if the gate
    // only checked length (not actual cryptographic verification).
    let record = InstallRecord {
        manifest_cid: stub_cid_zero(),
        plugin_did: benten_id::did::Did::from_string_unchecked(
            "did:key:z6MkPluginConsentTest".to_string(),
        ),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![0u8; 16],
        consenting_user_did: stub_user_did(),
        // 64 bytes but not a real Ed25519 signature
        user_signature: vec![0xFFu8; 64],
        granted_caps_bytes: vec![],
    };

    let err = verify_install_record(&record).expect_err("MUST reject forged signature");
    assert_eq!(
        err,
        ErrorCode::PluginInstallRecordUserSignatureInvalid,
        "consent gate MUST cryptographically verify; would-FAIL if \
         only structural length check ran"
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn install_plugin_without_install_record_surfaces_e_plugin_install_consent_required() {
    // **R4b-FP-1 Seam 1** un-ignore — substantive: the consent gate
    // is bundled INSIDE `plugin_lifecycle::install_plugin`, not a
    // separate caller step. Two arms per pim-2-amendment §3.6b sub-rule 4:
    //
    //  (a) bad-signature install record → install_plugin rejects with
    //      typed E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID (the
    //      structural ErrorCode-split per arch-r1-3).
    //  (b) valid-signature install record but its `manifest_cid`
    //      mismatches the install path's `expected_cid` → install_plugin
    //      rejects with E_PLUGIN_INSTALL_CONSENT_REQUIRED (consent-
    //      record-substitution defense).
    //
    // Would-FAIL-if-no-op'd: skipping verify_install_record entirely
    // allows arm (a); skipping the cid-binding allows arm (b).
    use benten_id::keypair::Keypair;
    use benten_platform_foundation::plugin_library::PluginLibrary;
    use benten_platform_foundation::plugin_lifecycle::{
        InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
    };

    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    // R6-FP-A-fp caller-mint-first: pre-mint + insert a real plugin-DID
    // handle so install_plugin Step 8's `PluginDidHandleNotPreInserted`
    // check passes. Placeholder DIDs no longer work.
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_placeholder = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);

    // Honestly-signed manifest by alice.
    let manifest = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "consent-gate-test",
        &["store:notes:read"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;

    // ARM (a) — install record carries a 64-byte but cryptographically
    // bogus user signature.
    let bad_record = InstallRecord {
        manifest_cid: expected_cid,
        plugin_did: plugin_did_placeholder.clone(),
        consenting_user_did: user_did.clone(),
        user_signature: vec![0xAAu8; 64], // bogus
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![0u8; 16],
        granted_caps_bytes: vec![],
    };

    let mut library = PluginLibrary::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<benten_id::did::Did> = vec![];
    let mut ctx = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
    };
    let ctx_params = InstallParams {
        now_secs: 1_700_000_000,
        // valid clock (non-sentinel)
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_placeholder,
    };

    let bad_outcome = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &ctx_params,
        &bytes,
        &expected_cid,
        &bad_record,
        1,
        &|_| None,
    );
    let bad_err = bad_outcome.expect_err("install MUST reject bad consent signature");
    assert_eq!(
        bad_err,
        ErrorCode::PluginInstallRecordUserSignatureInvalid,
        "consent gate MUST surface typed PluginInstallRecordUserSignatureInvalid; \
         would-FAIL if install_plugin skipped verify_user_signature"
    );
    assert!(
        library.is_empty(),
        "consent-rejected install MUST NOT commit library state"
    );
    assert!(
        cascade.minted_grants().is_empty(),
        "consent-rejected install MUST NOT mint root grants (fail-closed)"
    );

    // ARM (b) — record has VALID signature but signs over a DIFFERENT
    // manifest CID. Defense vs. consent-record substitution.
    let other_cid = benten_core::Cid::from_blake3_digest([0xCCu8; 32]);
    let mismatched_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        other_cid, // valid signature but over the WRONG cid
        plugin_did_placeholder.clone(),
        7,
    );

    let mut library2 = PluginLibrary::new();
    let mut store2 = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_b = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store2);
    // Rebuild mismatched_record with this run's plugin_did_b so the
    // expected_plugin_did binding holds on the legitimate-Step-8 path
    // — we want the manifest-CID-mismatch to be the SOLE rejection
    // reason in this arm.
    let mismatched_record_b = common::manifest_fixtures::signed_install_record(
        &user_kp,
        other_cid,
        plugin_did_b.clone(),
        7,
    );
    let _ = mismatched_record; // silence unused (replaced by *_b)
    let mut cascade2 = InMemoryInstallCascade::new();
    let mut private_ns2 = InMemoryInstallCascade::new();
    let mut ctx2 = InstallPorts {
        cap_minter: &mut cascade2,
        private_ns: &mut private_ns2,
    };
    let ctx2_params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_b,
    };

    let sub_outcome = install_plugin(
        &mut library2,
        &mut store2,
        &mut ctx2,
        &ctx2_params,
        &bytes,
        &expected_cid,
        &mismatched_record_b,
        1,
        &|_| None,
    );
    let sub_err = sub_outcome.expect_err("install MUST reject substituted consent record");
    // R6-FP-A arch-r6-r1-5 split: manifest-CID-mismatch now surfaces
    // the typed `PluginInstallRecordManifestCidMismatch` (forensic
    // discrimination from the null-consent + the consenting-user +
    // the plugin-DID-binding arms).
    assert_eq!(
        sub_err,
        ErrorCode::PluginInstallRecordManifestCidMismatch,
        "manifest-CID-mismatch on InstallRecord MUST surface \
         PluginInstallRecordManifestCidMismatch (arch-r6-r1-5 split); \
         would-FAIL if the seam skipped the cid-binding check"
    );
    assert!(
        library2.is_empty(),
        "consent-record-substitution rejected install MUST NOT commit"
    );

    // POSITIVE arm (defense-in-depth boundary per pim-2 §3.6b — confirms
    // the seam is not over-strict): with a properly-signed record whose
    // manifest_cid matches expected_cid, install_plugin admits.
    let mut store3 = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_c = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store3);
    let good_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_c.clone(),
        9,
    );
    let mut library3 = PluginLibrary::new();
    let mut cascade3 = InMemoryInstallCascade::new();
    let mut private_ns3 = InMemoryInstallCascade::new();
    let mut ctx3 = InstallPorts {
        cap_minter: &mut cascade3,
        private_ns: &mut private_ns3,
    };
    let ctx3_params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_c,
    };
    let good_outcome = install_plugin(
        &mut library3,
        &mut store3,
        &mut ctx3,
        &ctx3_params,
        &bytes,
        &expected_cid,
        &good_record,
        1,
        &|_| None,
    )
    .expect("properly-signed consent record MUST admit");
    assert_eq!(good_outcome.grants_minted, 1);
    assert_eq!(library3.len(), 1);
}
