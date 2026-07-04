//! Phase-4-Meta-Core R11-fix M-2b — signed-surface gap closure pin.
//!
//! `InstallRecord::signing_payload()` binds `granted_caps_bytes` into the
//! user's consent signature (canonical length-prefixed). Before this fix the
//! payload covered only `manifest_cid + timestamp + nonce + plugin_did_bytes`,
//! so the granted capabilities were attacker-mutable independent of the user's
//! consent signature — a store / filesystem tamper that appends an
//! un-consented cap would still pass `verify_user_signature()` →
//! capability-escalation.
//!
//! ## Would-FAIL-on-revert
//!
//! Revert the M-2b `granted_caps_bytes` length-prefix in `signing_payload()`
//! and these two arms flip:
//!   - `install_record_granted_caps_tamper_fails_signature_verify` would PASS
//!     signature verification on the tampered record (the escalation bug).
//!   - `install_record_signing_payload_binds_granted_caps` would observe an
//!     identical payload for two records that differ only in
//!     `granted_caps_bytes` (the payload would not be injective over caps).

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_manifest::{CapRequirement, InstallRecord};

/// DAG-CBOR encode one `CapRequirement` the way a real consent record carries
/// it in `granted_caps_bytes` (per `install_record_covers_required_caps`).
fn cap_bytes(scope: &str) -> Vec<u8> {
    serde_ipld_dagcbor::to_vec(&CapRequirement::new(scope)).unwrap()
}

/// A user signs an install record consenting to a NARROW set of granted caps;
/// an attacker appends an un-consented cap to `granted_caps_bytes` without
/// re-signing. Verification MUST reject (the granted caps are bound into the
/// signature).
#[test]
fn install_record_granted_caps_tamper_fails_signature_verify() {
    let user = Keypair::generate();
    let user_did = user.public_key().to_did();

    // Build the plugin-DID via a real minted keypair so `plugin_did.as_str()`
    // is a resolvable did:key (matches production install-record shape).
    let plugin_kp = Keypair::generate();
    let plugin_did = plugin_kp.public_key().to_did();

    // User consents to ONE granted cap (store:notes:read).
    let mut record = InstallRecord {
        manifest_cid: Cid::from_blake3_digest([7u8; 32]),
        plugin_did: plugin_did.clone(),
        consenting_user_did: user_did.clone(),
        user_signature: Vec::new(),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![0xABu8; 16],
        granted_caps_bytes: vec![cap_bytes("store:notes:read")],
    };
    record.user_signature = user.sign(&record.signing_payload()).to_bytes().to_vec();

    // Baseline: the honestly-signed record verifies.
    record
        .verify_user_signature()
        .expect("baseline: honestly-signed record verifies");

    // Attack: append an un-consented, higher-authority cap. Attacker holds no
    // secret key, so the old signature stays in place.
    let mut tampered = record.clone();
    tampered.granted_caps_bytes.push(cap_bytes("store:*:write"));
    // KEY POINT: do NOT re-sign.

    let err = tampered
        .verify_user_signature()
        .expect_err("M-2b: granted_caps_bytes tamper MUST fail signature verify");
    assert!(
        matches!(
            err,
            benten_errors::ErrorCode::PluginInstallRecordUserSignatureInvalid
        ),
        "M-2b: tamper must surface typed install-record-invalid; got {err:?}"
    );
}

/// `signing_payload()` is injective over `granted_caps_bytes`: two records
/// that differ ONLY in their granted caps produce different payloads. Also
/// pins the length-prefix boundary against the `[[a,b],[c]]` vs `[[a],[b,c]]`
/// concatenation-collision class.
#[test]
fn install_record_signing_payload_binds_granted_caps() {
    let plugin_kp = Keypair::generate();
    let plugin_did = plugin_kp.public_key().to_did();
    let user_did = Keypair::generate().public_key().to_did();

    let base = InstallRecord {
        manifest_cid: Cid::from_blake3_digest([9u8; 32]),
        plugin_did: plugin_did.clone(),
        consenting_user_did: user_did.clone(),
        user_signature: Vec::new(),
        timestamp_stub_nanos: 42,
        nonce: vec![0u8; 16],
        granted_caps_bytes: vec![],
    };

    let with_read = InstallRecord {
        granted_caps_bytes: vec![cap_bytes("store:notes:read")],
        ..base.clone()
    };
    let with_write = InstallRecord {
        granted_caps_bytes: vec![cap_bytes("store:*:write")],
        ..base.clone()
    };

    // Empty vs one-cap: payloads differ (caps are bound, not dropped).
    assert_ne!(
        base.signing_payload(),
        with_read.signing_payload(),
        "M-2b: adding a granted cap MUST change the signing payload"
    );
    // Same count, different cap bytes: payloads differ.
    assert_ne!(
        with_read.signing_payload(),
        with_write.signing_payload(),
        "M-2b: a different granted cap MUST change the signing payload"
    );

    // Length-prefix boundary injectivity: [[a, b], [c]] vs [[a], [b, c]] with
    // the SAME concatenated cap bytes must NOT collide.
    let a = cap_bytes("store:a:read");
    let b = cap_bytes("store:b:read");
    let c = cap_bytes("store:c:read");
    let mut ab = a.clone();
    ab.extend_from_slice(&b);
    let mut bc = b.clone();
    bc.extend_from_slice(&c);

    let split_1 = InstallRecord {
        granted_caps_bytes: vec![ab, c.clone()],
        ..base.clone()
    };
    let split_2 = InstallRecord {
        granted_caps_bytes: vec![a, bc],
        ..base.clone()
    };
    assert_ne!(
        split_1.signing_payload(),
        split_2.signing_payload(),
        "M-2b: length-prefixing MUST prevent a boundary-shifting caps collision"
    );
}
