//! Phase-4-Foundation R4-FP-1 — T5a LOAD-BEARING pin: post-install
//! install-record drift detected at load-verified.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T5
//! ("Plugin manifest envelope integrity") + defense step 1 (sec-4f-r1-9
//! verify-on-every-load).
//!
//! ## What this pin establishes
//!
//! Per threat-model §T5a + defense step 1: "Install record verified
//! on EVERY load, not just at install — (i) at engine boot, (ii) at
//! per-plugin load on first access, (iii) at per-Atrium-merge
//! boundary." Attacker swaps install-record bytes post-install
//! (writes to manifest store; restarts engine); new install record
//! has wider `requires` consent. User never re-consented.
//!
//! New seam: `crates/benten-platform-foundation/src/manifest_store.rs::
//! ManifestStore::load_verified(plugin_did) -> Result<InstallRecord>`.
//! Verifies user-DID signature on install record at every load.
//!
//! Substantive after G24-D-FP-1 wire-up: exercises the
//! `ManifestStore::load_verified` substantive path against a real
//! Ed25519-signed install record, mutates the bytes, and asserts the
//! drift surfaces as the typed
//! `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` AND user
//! notification is captured (defense-in-depth per threat-model §T5
//! "do not auto-quarantine — surface to user").
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires verification at install only. Manifest-store
//! bytes mutated post-install (file system attack); next engine boot
//! loads the mutated install record without re-verifying; widened
//! `requires` envelope is silently accepted. Layer 2 consent
//! guarantee broken.

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::manifest_store::ManifestStore;
use benten_platform_foundation::plugin_manifest::InstallRecord;
use common::manifest_fixtures::stub_plugin_did;

#[test]
fn plugin_manifest_post_install_record_byte_mutation_detected_at_load_verified() {
    let plugin_did = stub_plugin_did();
    // Real Ed25519 keypair for the user-DID (T5a defense requires real
    // signatures — stub bytes would let the test pass on the wrong
    // path).
    let user = Keypair::generate();
    let user_did = user.public_key().to_did();

    // Build install record signed by user-DID; narrow `requires`
    // envelope (only store:notes:read).
    let manifest_cid = Cid::from_blake3_digest([7u8; 32]);
    let mut original = InstallRecord {
        manifest_cid,
        plugin_did: plugin_did.clone(),
        consenting_user_did: user_did.clone(),
        user_signature: Vec::new(),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![0xABu8; 16],
        granted_caps_bytes: vec![],
    };
    let sig = user.sign(&original.signing_payload());
    original.user_signature = sig.to_bytes().to_vec();

    let mut store = ManifestStore::new();
    store
        .install_plugin(plugin_did.clone(), original.clone())
        .expect("install ok");

    // Baseline: load_verified succeeds.
    let loaded = store
        .load_verified(&plugin_did)
        .expect("baseline load_verified ok");
    assert_eq!(loaded.consenting_user_did, user_did);
    assert!(
        store.captured_user_notifications().is_empty(),
        "Baseline: no drift notifications"
    );

    // Attack: mutate the install record bytes — attacker swaps the
    // nonce (a field signed under the user's key) without holding the
    // user's secret key. The old signature remains in place but no
    // longer verifies over the mutated bytes.
    let mut mutated = original.clone();
    mutated.nonce = vec![0xFFu8; 16];
    // KEY POINT: do NOT re-sign. Attacker has no secret key.
    store
        .simulate_byte_mutation_attack(plugin_did.clone(), mutated)
        .expect("attack simulate ok");

    // T5a LOAD-BEARING: load_verified MUST reject.
    let result = store.load_verified(&plugin_did);
    let err =
        result.expect_err("T5a LOAD-BEARING: post-install install-record drift MUST be detected");
    assert!(
        matches!(err, ErrorCode::PluginInstallRecordUserSignatureInvalid),
        "T5a: must surface typed install-record-invalid error; got {err:?}"
    );

    // Defense-in-depth: user notification surfaced (per threat-model
    // §T5 "ANY mismatch → reject + surface to user").
    let notifications = store.captured_user_notifications();
    assert!(
        notifications
            .iter()
            .any(|n| n.is_install_record_drift_warning(&plugin_did)),
        "T5a: drift detection MUST surface user notification; zero notifications means surfacing path is silent"
    );
}
