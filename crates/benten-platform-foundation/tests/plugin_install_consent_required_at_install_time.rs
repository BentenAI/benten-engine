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

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    install_plugin currently does NOT bundle the install-record consent gate \
    inside its signature (callers verify_install_record separately). \
    G24-D-FP-1 ships the lifecycle integration where install_plugin walks \
    install_record validation alongside library insert + cap-cascade. Named \
    destination: plan §3 G24-D-FP-1 (plugin_lifecycle::uninstall_plugin + \
    install_plugin lifecycle hardening). HARD RULE 12 clause-(b) BELONGS-NAMED-NOW."]
#[test]
fn install_plugin_without_install_record_surfaces_e_plugin_install_consent_required() {
    // Phase 4-Foundation R5 G24-D-FP-1 un-ignores this. Surface shape:
    //   plugin_lifecycle::install_plugin_with_consent(
    //     library, bytes, expected_cid, installer_shape, install_record,
    //     installed_at, resolver,
    //   ) -> Result; returns ErrorCode::PluginInstallConsentRequired
    // when install_record is None or its signature fails verification.
    //
    // FAILS-IF-NO-OP because the consent gate must explicitly check
    // the install record's presence and user-DID signature validity
    // as part of the install lifecycle (not as a separate caller step).
    panic!(
        "G24-D-FP-1 wires install_plugin's internal consent gate; \
         until then the umbrella test exercises verify_install_record \
         separately"
    );
}
