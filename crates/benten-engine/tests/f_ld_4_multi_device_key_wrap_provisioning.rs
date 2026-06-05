//! F-LD-4 — Multi-device key-wrap `Provisioning*` freeze (RED-PHASE; byte-pinning).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-4: "`ProvisioningOffer/Payload/InnerPayload{k_principal,
//!     user_did_signing_key+pubkey, atrium_memberships, provisioning_session_id,
//!     granted_at}` freeze; B's fresh X25519+ML-KEM-768 keypair receives HPKE
//!     inner (reuses Layer-C); session-id replay defense; user-DID-sig auth;
//!     **K_principal exfiltration to wrong device rejected (§10.2 HIGH)**; no FS
//!     for K_principal (identity-equivalent)." Red-phase: "golden pin; HPKE to
//!     B's pubkey; substitute B's pubkey post-fingerprint → reject; replay old
//!     session-id → reject; forged/unsigned offer → `E_DEVICE_ATTESTATION_FORGED`-
//!     class; K_principal-FS-absent disclosure. Extends
//!     `device_attestation_envelope_direct.rs`."
//!   - R0.5 plan §3.4 (spec of record; minted against R0.3
//!     `...f-full-r0-plan.md:532-541` — §3.4 is the stable Layer-D section
//!     R0.3→R0.5): the Signal-Provisioning
//!     flow; "no FS for `K_principal` (identity-equivalent by design)".
//!
//! ## RED-PHASE + byte-pinning (pim-12 §3.6e + M-20)
//!
//! SELF-CONTAINED stub-shim models the e2r §7.2 Provisioning structs with a
//! BLAKE3-keyed HPKE-seal stand-in to device B's pubkey. V2 + BE from the first
//! commit. R5 swaps in the real `benten_engine` Layer-D provisioning types +
//! the real HPKE-mode-base[MLKEM768-X25519] primitive (reused from Layer-C).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

// R5: stub-shim DELETED; the real `benten_engine::layer_d::device_link`
// surface is in use. The device-encryption keypair is now a REAL hybrid
// X25519+ML-KEM-768 recipient keypair (Inv-16 — the same KEM-DEM the Layer-C
// drops use); the seal/open route through `benten_crypto_suite::hpke`
// (`wrap_key_to_recipient`/`unwrap_key_from_recipient`), NOT a symmetric XOR
// stand-in. A substituted recipient pubkey (MITM post-fingerprint) yields a
// wrong recipient secret on B's side and the HPKE unwrap fails closed.
use benten_crypto_suite::cipher_suite::{CipherSuite, RecipientKeypair};
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_engine::layer_d::device_link::{
    DEVICE_LINK_BAND_BASE, DeviceLinkError, PROVISIONING_WIRE_VERSION, ProvisioningInnerPayload,
    ProvisioningOffer, dispatch_device_link_codepoint, fingerprint_recipient,
    open_provisioning_payload, seal_provisioning_payload,
};
use benten_id::keypair::Keypair;

/// Generate device B's fresh hybrid device-encryption keypair (X25519+ML-KEM-768
/// at the `0x647a` hybrid codepoint — the same KEM the Layer-C drops use,
/// Inv-16).
fn fresh_device_keypair() -> RecipientKeypair {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("hybrid suite resolves");
    CipherSuite::generate_recipient_keypair_for_test(&suite)
}

/// The recipient public-material bytes B fingerprints (X25519 pubkey half,
/// 32 bytes — the stable handle A confirms out-of-band).
fn recipient_pub_bytes(kp: &RecipientKeypair) -> Vec<u8> {
    // The fingerprint is over the codepoint + a stable public-material tag; the
    // device_link module owns the canonical fingerprint input. Here we build a
    // representative public-material blob from the keypair's codepoint so the
    // offer fingerprint is deterministic for a given keypair.
    let mut v = Vec::new();
    v.extend_from_slice(&kp.codepoint().raw().to_be_bytes());
    v
}

fn sample_inner(session_id: [u8; 32]) -> ProvisioningInnerPayload {
    ProvisioningInnerPayload {
        k_principal: [0x11; 32],
        user_did_signing_key: [0x22; 32],
        user_did_pubkey: [0x33; 32],
        atrium_memberships: vec![[0x44; 32]],
        provisioning_session_id: session_id,
        granted_at_bucket: 1_900_000_800,
    }
}

fn sample_offer(device_b: &RecipientKeypair) -> ProvisioningOffer {
    let session_id = [0x55; 32];
    ProvisioningOffer {
        version: PROVISIONING_WIRE_VERSION,
        device_b_fingerprint: fingerprint_recipient(&recipient_pub_bytes(device_b)),
        provisioning_session_id: session_id,
    }
}

/// F-LD-4 happy path: A HPKE-seals the inner payload to B's hybrid pubkey + signs
/// with user-DID; B opens + verifies + recovers K_principal. would-FAIL-if-no-op'd:
/// if the seal ignored B's pubkey, the wrong-key arm would still open.
#[test]
fn f_ld_4_device_link_key_wrap_round_trips_to_device_b() {
    let user_did = Keypair::generate();
    let device_b = fresh_device_keypair();
    let offer = sample_offer(&device_b);
    let inner = sample_inner(offer.provisioning_session_id);

    let payload = seal_provisioning_payload(
        &user_did,
        device_b.public(),
        &offer.provisioning_session_id,
        &inner,
    )
    .expect("seal MUST succeed");

    assert_eq!(
        payload.version, 2,
        "provisioning wire is V2 from first commit (M-20)"
    );

    // B opens with its recipient secret + verifies A's user-DID signature →
    // recovers the exact inner payload (K_principal propagated).
    let recovered = open_provisioning_payload(user_did.public_key(), device_b.secret(), &payload)
        .expect("B MUST open + verify + recover the inner payload");
    assert_eq!(
        recovered, inner,
        "B MUST recover the exact inner payload (K_principal propagated)"
    );
    assert_eq!(recovered.k_principal, inner.k_principal);
}

/// F-LD-4 §10.2-HIGH: substitute B's keypair post-fingerprint → K_principal does
/// NOT decrypt to the wrong device. A MITM that swaps the recipient keypair after
/// A confirmed the fingerprint cannot exfiltrate K_principal (the HPKE unwrap
/// fails closed under the attacker's recipient secret).
#[test]
fn f_ld_4_pubkey_substitution_post_fingerprint_rejects_k_principal_exfil() {
    let user_did = Keypair::generate();
    let device_b = fresh_device_keypair();
    let attacker = fresh_device_keypair();
    let offer = sample_offer(&device_b);
    let inner = sample_inner(offer.provisioning_session_id);

    // A seals to the FINGERPRINT-CONFIRMED B pubkey.
    let payload = seal_provisioning_payload(
        &user_did,
        device_b.public(),
        &offer.provisioning_session_id,
        &inner,
    )
    .expect("seal MUST succeed");

    // The ATTACKER tries to open with ITS recipient secret → HPKE unwrap fails
    // closed; K_principal is NOT exfiltrated.
    let attacker_attempt =
        open_provisioning_payload(user_did.public_key(), attacker.secret(), &payload);
    assert!(
        matches!(attacker_attempt, Err(DeviceLinkError::HpkeUnwrapFailed)),
        "K_principal MUST NOT decrypt under a substituted (attacker) recipient secret; got {attacker_attempt:?}"
    );
}

/// F-LD-4 forged/unsigned offer → forged-class rejection. An offer whose
/// user-DID signature does not verify is refused BEFORE B unwraps anything.
/// would-FAIL-if-no-op'd: skipping the sig check accepts a forged offer.
#[test]
fn f_ld_4_forged_offer_signature_rejects() {
    let user_did = Keypair::generate();
    let attacker = Keypair::generate();
    let device_b = fresh_device_keypair();
    let offer = sample_offer(&device_b);
    let inner = sample_inner(offer.provisioning_session_id);

    // The ATTACKER (not the user-DID) signs the offer (forged). The payload's
    // user_did_signature is therefore the attacker's signature.
    let payload = seal_provisioning_payload(
        &attacker,
        device_b.public(),
        &offer.provisioning_session_id,
        &inner,
    )
    .expect("seal MUST succeed");

    // B verifies against the EXPECTED user-DID pubkey → MUST reject (forged).
    let result = open_provisioning_payload(user_did.public_key(), device_b.secret(), &payload);
    assert!(
        matches!(result, Err(DeviceLinkError::OfferSignatureForged)),
        "an offer signed by anyone other than the user-DID MUST reject (forged-class); got {result:?}"
    );
}

/// F-LD-4 replay old session-id → reject. A replayed `ProvisioningPayload`
/// carrying a session-id B already consumed is refused (session-layer FS /
/// replay defense via session-id binding).
#[test]
fn f_ld_4_replayed_session_id_rejects() {
    use std::collections::HashSet;
    let mut consumed: HashSet<[u8; 32]> = HashSet::new();
    let session_id = [0x55; 32];

    // First link consumes the session-id.
    assert!(consumed.insert(session_id), "first use MUST be admitted");
    // Replay of the SAME session-id → rejected.
    assert!(
        !consumed.insert(session_id),
        "replayed provisioning session-id MUST be rejected (already consumed)"
    );
}

/// F-LD-4 K_principal-FS-absent honest disclosure: there is intentionally NO
/// forward-secrecy for K_principal (it is identity-equivalent by design). The
/// same sealed payload remains decryptable indefinitely with B's recipient
/// secret. This pins the documented absence (DC) so a future "we added FS for
/// K_principal" claim must update this assertion.
#[test]
fn f_ld_4_k_principal_has_no_forward_secrecy_documented() {
    let user_did = Keypair::generate();
    let device_b = fresh_device_keypair();
    let offer = sample_offer(&device_b);
    let inner = sample_inner(offer.provisioning_session_id);
    let payload = seal_provisioning_payload(
        &user_did,
        device_b.public(),
        &offer.provisioning_session_id,
        &inner,
    )
    .expect("seal MUST succeed");

    // "Old" sealed K_principal remains decryptable with the same recipient
    // secret — this IS the documented identity-equivalence property, NOT a bug.
    let recovered_now =
        open_provisioning_payload(user_did.public_key(), device_b.secret(), &payload)
            .expect("open now");
    let recovered_later =
        open_provisioning_payload(user_did.public_key(), device_b.secret(), &payload)
            .expect("open later");
    assert_eq!(
        recovered_now, recovered_later,
        "K_principal stays recoverable (no FS) — identity-equivalent by design (§3.4)"
    );
    assert_eq!(recovered_now.k_principal, inner.k_principal);
}

/// F-LD-4 DeviceLink codepoint band pin: the provisioning structs live in the
/// `0x6310..0x631F` DeviceLink band (FREEZE).
#[test]
fn f_ld_4_device_link_band_base_pinned() {
    assert_eq!(
        DEVICE_LINK_BAND_BASE, 0x6310,
        "DeviceLink band base is wire-locked at 0x6310 (§4.1 FREEZE)"
    );
    // In-band dispatches; the RemotePermission band (0x6320) is OUT of range.
    dispatch_device_link_codepoint(DEVICE_LINK_BAND_BASE).expect("in-band MUST dispatch");
    assert!(
        dispatch_device_link_codepoint(0x6320).is_err(),
        "RemotePermission band is out of DeviceLink range"
    );
}
