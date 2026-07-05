//! Phase-4-Meta-Core F-INJ-1 — `InstallRecord::signing_payload()` whole-
//! pre-image injectivity pin (nonce → plugin_did seam).
//!
//! BEFORE this fix the payload concatenated `nonce` and `plugin_did_bytes`
//! with NO length prefix:
//!   `manifest_cid[36] || timestamp[8] || nonce || plugin_did_bytes || caps`
//! so the `nonce → plugin_did` seam was AMBIGUOUS: shifting a byte across it
//! yields a DISTINCT `(nonce, plugin_did)` tuple with IDENTICAL payload bytes
//! (and therefore identical BLAKE3). That breaks (a) signature injectivity —
//! one Ed25519 signature verifies for two different records — and (b) the
//! §4.37 replay store keyed on `blake3(signing_payload())`
//! (`plugin_lifecycle.rs::consume_offline`-adjacent replay-check), which
//! would collide the two records.
//!
//! The fix length-prefixes BOTH `nonce` and `plugin_did_bytes` with a BE-u32
//! byte-length (matching the M-2b `granted_caps_bytes` discipline already in
//! the same fn), making the whole pre-image injective.
//!
//! ## Would-FAIL-on-revert
//!
//! Revert the F-INJ-1 nonce/plugin_did length-prefixes in `signing_payload()`
//! and both assertions below flip: the reproduced seam-split pair produces an
//! IDENTICAL `signing_payload()` AND an IDENTICAL `blake3(signing_payload())`,
//! so `assert_ne!` fails.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_id::did::Did;
use benten_platform_foundation::plugin_manifest::InstallRecord;

/// Build an `InstallRecord` with an explicit `nonce` + `plugin_did` string,
/// holding every other field constant so ONLY the nonce/plugin_did seam
/// varies between the two colliding candidates.
fn record_with(nonce: Vec<u8>, plugin_did_str: &str) -> InstallRecord {
    InstallRecord {
        manifest_cid: Cid::from_blake3_digest([0x33u8; 32]),
        // Test-fixture DID: `signing_payload()` reads `plugin_did.as_str()`
        // verbatim (no resolution), so an arbitrary string is the exact
        // pre-image the seam-collision is built against.
        plugin_did: Did::from_string_for_test_fixture(plugin_did_str.to_string()),
        consenting_user_did: Did::from_string_for_test_fixture("did:key:zUSER".to_string()),
        user_signature: Vec::new(),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce,
        granted_caps_bytes: vec![],
    }
}

/// The REPRODUCED nonce → plugin_did seam collision:
///   A = { nonce = [0x01; 16],        plugin_did = "did:key:zAAA_TAIL" }
///   B = { nonce = [0x01; 16, 0x64],  plugin_did =  "id:key:zAAA_TAIL" }
/// (`0x64` is the ASCII `d`). Under the OLD bare-concatenation layout the
/// two `nonce || plugin_did_bytes` regions are byte-identical — the `d`
/// merely shifts from the head of `plugin_did` into the tail of `nonce` —
/// so both records hash the SAME payload despite being distinct tuples.
/// After the length-prefix fix the `len(nonce)` field (16 vs 17) alone
/// disambiguates them.
#[test]
fn f_inj_1_nonce_plugin_did_seam_is_injective() {
    let record_a = record_with(vec![0x01u8; 16], "did:key:zAAA_TAIL");

    let mut nonce_b = vec![0x01u8; 16];
    nonce_b.push(0x64); // ASCII 'd'
    let record_b = record_with(nonce_b, "id:key:zAAA_TAIL");

    // Sanity: the two records really are DISTINCT tuples (not the same
    // record built twice) — the seam split is the only difference.
    assert_ne!(
        record_a.nonce, record_b.nonce,
        "fixture sanity: the two candidates must carry different nonces",
    );
    assert_ne!(
        record_a.plugin_did.as_str(),
        record_b.plugin_did.as_str(),
        "fixture sanity: the two candidates must carry different plugin_dids",
    );

    // (a) signature-injectivity: the two DISTINCT records must produce
    //     DISTINCT signing payloads (so one Ed25519 signature cannot verify
    //     for both). REVERT -> identical payloads -> this fails.
    let payload_a = record_a.signing_payload();
    let payload_b = record_b.signing_payload();
    assert_ne!(
        payload_a, payload_b,
        "F-INJ-1: the nonce/plugin_did seam split MUST produce distinct \
         signing_payload() bytes (whole pre-image is length-prefixed)",
    );

    // (b) §4.37 replay-store keyed on blake3(signing_payload()): the two
    //     records must produce DISTINCT hashes so the replay store does not
    //     collide them. REVERT -> identical blake3 -> this fails.
    let hash_a = *blake3::hash(&payload_a).as_bytes();
    let hash_b = *blake3::hash(&payload_b).as_bytes();
    assert_ne!(
        hash_a, hash_b,
        "F-INJ-1: distinct records MUST produce distinct blake3(signing_payload) \
         so the §4.37 replay store does not collide them",
    );
}

/// A signature honestly minted over record A must NOT verify against the
/// seam-collided record B. Under the OLD layout the shared payload meant the
/// SAME signature verified for both (the injectivity break in the wild);
/// after the fix the payloads differ so B's verify fails.
#[test]
fn f_inj_1_signature_does_not_cross_the_seam() {
    use benten_id::keypair::Keypair;

    let user = Keypair::generate();
    let user_did = user.public_key().to_did();

    // Record A signed honestly by the user.
    let mut record_a = record_with(vec![0x01u8; 16], "did:key:zAAA_TAIL");
    record_a.consenting_user_did = user_did.clone();
    record_a.user_signature = user.sign(&record_a.signing_payload()).to_bytes().to_vec();
    record_a
        .verify_user_signature()
        .expect("baseline: honestly-signed record A verifies");

    // Record B is the seam-shifted twin. Copy A's signature verbatim (the
    // attacker holds no key). Under the OLD layout A and B share a payload,
    // so A's signature verified for B (cross-seam signature reuse). The fix
    // makes the payloads distinct, so verify MUST reject.
    let mut nonce_b = vec![0x01u8; 16];
    nonce_b.push(0x64);
    let mut record_b = record_with(nonce_b, "id:key:zAAA_TAIL");
    record_b.consenting_user_did = user_did;
    record_b.user_signature = record_a.user_signature.clone();

    let err = record_b
        .verify_user_signature()
        .expect_err("F-INJ-1: A's signature MUST NOT verify for the seam-shifted twin B");
    assert!(
        matches!(
            err,
            benten_errors::ErrorCode::PluginInstallRecordUserSignatureInvalid
        ),
        "F-INJ-1: cross-seam signature reuse must surface typed \
         install-record-invalid; got {err:?}",
    );
}
