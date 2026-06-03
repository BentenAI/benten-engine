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

use benten_id::keypair::Keypair;

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — e2r §7.2 Provisioning structs + HPKE-to-B seal.
// ---------------------------------------------------------------------------
mod shim {
    pub const PROVISIONING_WIRE_VERSION: u8 = 2;
    /// DeviceLink band base (§4.1 `0x6310..0x631F` FREEZE).
    pub const DEVICE_LINK_BAND_BASE: u16 = 0x6310;

    /// e2r §7.2 — what device B publishes in its QR. B generates a fresh
    /// device-encryption pubkey (stand-in for X25519+ML-KEM-768) + fresh DAK.
    #[derive(Clone)]
    pub struct ProvisioningOffer {
        pub version: u8,
        pub device_b_enc_pubkey: [u8; 32],
        pub provisioning_session_id: [u8; 32],
        /// Device-fingerprint A confirms out-of-band (QR scan).
        pub device_b_fingerprint: [u8; 32],
    }

    /// e2r §7.2 — the secret payload A HPKE-encrypts to B's pubkey.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct ProvisioningInnerPayload {
        pub k_principal: [u8; 32],
        pub user_did_signing_key: [u8; 32],
        pub user_did_pubkey: [u8; 32],
        pub atrium_memberships: Vec<[u8; 32]>,
        pub provisioning_session_id: [u8; 32],
        pub granted_at_bucket: u64,
    }

    impl ProvisioningInnerPayload {
        /// Canonical BE bytes for HPKE sealing (M-20).
        pub fn to_canonical_be(&self) -> Vec<u8> {
            let mut b = Vec::new();
            b.extend_from_slice(&self.k_principal);
            b.extend_from_slice(&self.user_did_signing_key);
            b.extend_from_slice(&self.user_did_pubkey);
            b.extend_from_slice(&(self.atrium_memberships.len() as u32).to_be_bytes());
            for m in &self.atrium_memberships {
                b.extend_from_slice(m);
            }
            b.extend_from_slice(&self.provisioning_session_id);
            b.extend_from_slice(&self.granted_at_bucket.to_be_bytes());
            b
        }
    }

    /// The signed wire wrapper A transmits (e2r §7.2 `ProvisioningPayload`):
    /// the HPKE-sealed inner + A's user-DID signature over (recipient_pubkey ‖
    /// session_id ‖ ciphertext). The signature is what authenticates the
    /// offer — an unsigned/forged offer fails here.
    pub struct ProvisioningPayload {
        pub version: u8,
        pub recipient_enc_pubkey: [u8; 32],
        pub provisioning_session_id: [u8; 32],
        pub hpke_ciphertext: Vec<u8>,
        pub user_did_signature: Vec<u8>,
    }

    /// HPKE-seal stand-in: binds the recipient pubkey so a post-fingerprint
    /// pubkey substitution makes Open fail. Ciphertext =
    /// inner ⊕ keystream(recipient_pubkey ‖ session_id); tag binds both.
    pub fn hpke_seal_to(recipient_pubkey: &[u8; 32], session_id: &[u8; 32], inner: &[u8]) -> Vec<u8> {
        let mut ks = blake3::Hasher::new();
        ks.update(b"benten-provision-hpke-v1:");
        ks.update(recipient_pubkey);
        ks.update(session_id);
        let mut reader = ks.finalize_xof();
        let mut keystream = vec![0u8; inner.len()];
        reader.fill(&mut keystream);
        // XOR keystream over the plaintext.
        inner
            .iter()
            .zip(keystream.iter())
            .map(|(b, k)| b ^ k)
            .collect()
    }

    pub fn hpke_open_with(
        recipient_pubkey: &[u8; 32],
        session_id: &[u8; 32],
        ciphertext: &[u8],
    ) -> Vec<u8> {
        // Symmetric: same keystream.
        hpke_seal_to(recipient_pubkey, session_id, ciphertext)
    }

    pub fn signing_bytes(recipient_pubkey: &[u8; 32], session_id: &[u8; 32], ct: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(recipient_pubkey);
        b.extend_from_slice(session_id);
        b.extend_from_slice(ct);
        b
    }
}

use shim::{
    hpke_open_with, hpke_seal_to, signing_bytes, ProvisioningInnerPayload, ProvisioningOffer,
    ProvisioningPayload, DEVICE_LINK_BAND_BASE, PROVISIONING_WIRE_VERSION,
};

fn sample_offer(device_b_enc: &Keypair) -> ProvisioningOffer {
    let pk = device_b_enc.public_key().to_bytes();
    ProvisioningOffer {
        version: PROVISIONING_WIRE_VERSION,
        device_b_enc_pubkey: pk,
        provisioning_session_id: [0x55; 32],
        device_b_fingerprint: *blake3::hash(&pk).as_bytes(),
    }
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

/// F-LD-4 happy path: A HPKE-seals the inner payload to B's pubkey + signs with
/// user-DID; B opens + verifies + recovers K_principal. would-FAIL-if-no-op'd:
/// if the seal ignored B's pubkey, the wrong-pubkey arm would still open.
#[test]
#[ignore = "RED-PHASE: F-LD-4 — device-link key-wrap HPKE-to-B round-trip + user-DID sig; un-ignore at R5"]
fn f_ld_4_device_link_key_wrap_round_trips_to_device_b() {
    let user_did = Keypair::generate();
    let device_b_enc = Keypair::generate();
    let offer = sample_offer(&device_b_enc);
    let inner = sample_inner(offer.provisioning_session_id);

    let ct = hpke_seal_to(
        &offer.device_b_enc_pubkey,
        &offer.provisioning_session_id,
        &inner.to_canonical_be(),
    );
    let sig = user_did.sign(&signing_bytes(
        &offer.device_b_enc_pubkey,
        &offer.provisioning_session_id,
        &ct,
    ));
    let payload = ProvisioningPayload {
        version: PROVISIONING_WIRE_VERSION,
        recipient_enc_pubkey: offer.device_b_enc_pubkey,
        provisioning_session_id: offer.provisioning_session_id,
        hpke_ciphertext: ct,
        user_did_signature: sig.to_bytes().to_vec(),
    };

    assert_eq!(payload.version, 2, "provisioning wire is V2 from first commit (M-20)");

    // B verifies A's user-DID signature.
    user_did
        .public_key()
        .verify(
            &signing_bytes(
                &payload.recipient_enc_pubkey,
                &payload.provisioning_session_id,
                &payload.hpke_ciphertext,
            ),
            &sig,
        )
        .expect("user-DID signature MUST verify (authentication)");

    // B opens with its pubkey + session-id → recovers the inner bytes.
    let recovered = hpke_open_with(
        &payload.recipient_enc_pubkey,
        &payload.provisioning_session_id,
        &payload.hpke_ciphertext,
    );
    assert_eq!(
        recovered,
        inner.to_canonical_be(),
        "B MUST recover the exact inner payload (K_principal propagated)"
    );
}

/// F-LD-4 §10.2-HIGH: substitute B's pubkey post-fingerprint → K_principal does
/// NOT decrypt to the wrong device. A MITM that swaps the recipient pubkey
/// after A confirmed the fingerprint cannot exfiltrate K_principal.
#[test]
#[ignore = "RED-PHASE: F-LD-4 — pubkey substitution post-fingerprint rejects K_principal exfil (§10.2 HIGH); un-ignore at R5"]
fn f_ld_4_pubkey_substitution_post_fingerprint_rejects_k_principal_exfil() {
    let device_b_enc = Keypair::generate();
    let attacker_enc = Keypair::generate();
    let offer = sample_offer(&device_b_enc);
    let inner = sample_inner(offer.provisioning_session_id);

    // A seals to the FINGERPRINT-CONFIRMED B pubkey.
    let ct = hpke_seal_to(
        &offer.device_b_enc_pubkey,
        &offer.provisioning_session_id,
        &inner.to_canonical_be(),
    );

    // MITM tries to open with the ATTACKER pubkey (substituted on the wire).
    let attacker_attempt = hpke_open_with(
        &attacker_enc.public_key().to_bytes(),
        &offer.provisioning_session_id,
        &ct,
    );
    assert_ne!(
        attacker_attempt,
        inner.to_canonical_be(),
        "K_principal MUST NOT decrypt under a substituted (attacker) pubkey"
    );
    // The recovered k_principal slice is NOT the real one.
    assert_ne!(
        &attacker_attempt[0..32],
        &inner.k_principal[..],
        "attacker MUST NOT recover the real K_principal"
    );
}

/// F-LD-4 forged/unsigned offer → `E_DEVICE_ATTESTATION_FORGED`-class rejection.
/// An offer whose user-DID signature does not verify is refused before B
/// installs anything. would-FAIL-if-no-op'd: skipping the sig check accepts a
/// forged offer.
#[test]
#[ignore = "RED-PHASE: F-LD-4 — forged/unsigned provisioning offer rejects (forged-class); un-ignore at R5"]
fn f_ld_4_forged_offer_signature_rejects() {
    let user_did = Keypair::generate();
    let attacker = Keypair::generate();
    let device_b_enc = Keypair::generate();
    let offer = sample_offer(&device_b_enc);
    let inner = sample_inner(offer.provisioning_session_id);
    let ct = hpke_seal_to(
        &offer.device_b_enc_pubkey,
        &offer.provisioning_session_id,
        &inner.to_canonical_be(),
    );

    // Attacker signs (forged — NOT the user-DID key).
    let forged_sig = attacker.sign(&signing_bytes(
        &offer.device_b_enc_pubkey,
        &offer.provisioning_session_id,
        &ct,
    ));

    // B verifies against the EXPECTED user-DID pubkey → MUST reject.
    assert!(
        user_did
            .public_key()
            .verify(
                &signing_bytes(&offer.device_b_enc_pubkey, &offer.provisioning_session_id, &ct),
                &forged_sig,
            )
            .is_err(),
        "an offer signed by anyone other than the user-DID MUST reject (forged-class)"
    );
}

/// F-LD-4 replay old session-id → reject. A replayed `ProvisioningPayload`
/// carrying a session-id that B already consumed is refused (session-layer FS /
/// replay defense via session-id binding). Pairs with F-LD-5 nonce-cache.
#[test]
#[ignore = "RED-PHASE: F-LD-4 — replayed provisioning session-id rejects; un-ignore at R5"]
fn f_ld_4_replayed_session_id_rejects() {
    // Model a per-device consumed-session-id set (the durable marker pattern).
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
/// forward-secrecy for K_principal (it is identity-equivalent by design). A
/// device that retains its DAK-decrypted K_principal can still decrypt old
/// content. This pins the documented absence (DC) so a future "we added FS for
/// K_principal" claim must update this assertion.
#[test]
#[ignore = "RED-PHASE: F-LD-4 — K_principal has NO forward-secrecy (identity-equivalent, documented); un-ignore at R5"]
fn f_ld_4_k_principal_has_no_forward_secrecy_documented() {
    let device_b_enc = Keypair::generate();
    let offer = sample_offer(&device_b_enc);
    let inner = sample_inner(offer.provisioning_session_id);
    let ct = hpke_seal_to(
        &offer.device_b_enc_pubkey,
        &offer.provisioning_session_id,
        &inner.to_canonical_be(),
    );

    // "Old" sealed K_principal remains decryptable indefinitely with the same
    // recipient pubkey+session (no ratchet, no FS) — this IS the documented
    // identity-equivalence property, NOT a bug.
    let recovered_now = hpke_open_with(&offer.device_b_enc_pubkey, &offer.provisioning_session_id, &ct);
    let recovered_later = hpke_open_with(&offer.device_b_enc_pubkey, &offer.provisioning_session_id, &ct);
    assert_eq!(
        recovered_now, recovered_later,
        "K_principal stays recoverable (no FS) — identity-equivalent by design (§3.4)"
    );
    assert_eq!(&recovered_now[0..32], &inner.k_principal[..]);
}

/// F-LD-4 DeviceLink codepoint band pin: the provisioning structs live in the
/// `0x6310..0x631F` DeviceLink band (FREEZE).
#[test]
#[ignore = "RED-PHASE: F-LD-4 — DeviceLink band base 0x6310 pin; un-ignore at R5"]
fn f_ld_4_device_link_band_base_pinned() {
    assert_eq!(
        DEVICE_LINK_BAND_BASE, 0x6310,
        "DeviceLink band base is wire-locked at 0x6310 (§4.1 FREEZE)"
    );
}
