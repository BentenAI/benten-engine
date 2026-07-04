//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pins for the
//! encryption-envelope-per-codepoint table (Rows D-9 / D-79).
//!
//! Row D-9 names "encryption-envelope per codepoint" as a MISSING byte
//! pin: one canonical hex per frozen encryption codepoint. The Layer-C
//! envelope codepoint band lives in `benten_drop::layer_c`:
//!
//! - `0x6500` `LAYER_C_DROP` (plaintext-sender, non-default)
//! - `0x6510` `DROP_TO_RECIPIENT_SEALED_SENDER` (v1-beta DEFAULT)
//! - `0x6520` `LAYER_C_DROP_MULTI_RECIPIENT` (group multi-stanza)
//! - `0x6610` MembershipSet group multi-stanza
//! - `0x647a` `HYBRID_X25519_MLKEM768` (the cipher codepoint — the
//!   ChaCha20-Poly1305-under-X-Wing AEAD envelope)
//!
//! # What surface pins each codepoint
//!
//! The codepoint is bound BIG-ENDIAN into the sender-auth binding message
//! `M_auth` (`build_m_auth`), so pinning `build_m_auth` over a fixed
//! `SenderAuthBinding` with each `envelope_codepoint` gives ONE canonical
//! hex per band — the same function seal + verify both run, so a drift in
//! the codepoint-binding position/endianness fails the pin. The 0x6510
//! DEFAULT band ALSO gets its on-wire AAD serializer pinned
//! (`sealed_aad::serialize_sealed_sender_aad`). The 0x647a cipher
//! codepoint is pinned at `AeadEnvelope::to_wire_bytes` (BE codepoint,
//! M-19).
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! Every surface here is a pure deterministic serializer over FIXED
//! inputs (fixed body-CID, fixed sender-DID, fixed commitments, fixed
//! nonce). Hex captured via throwaway compute + pasted below.
//!
//! would-FAIL-on-drift: a codepoint value change, a BE→LE endianness
//! flip, or a field-order change all change these bytes.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_crypto_suite::aead::AeadEnvelope;
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_drop::layer_c::{
    LAYER_C_DROP, LAYER_C_DROP_MULTI_RECIPIENT, SENDER_AUTH_SIG_CODEPOINT, SenderAuthBinding,
    build_m_auth, sealed_aad,
};

/// The MembershipSet group multi-stanza envelope codepoint (0x6610).
const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;
/// The Sealed-Sender DEFAULT codepoint (0x6510).
const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Fixed canonical 36-byte CIDv1 (self-describing) for the body_cid.
fn fixed_body_cid_bytes() -> Vec<u8> {
    Cid::from_blake3_digest([0xCC; 32]).as_bytes().to_vec()
}

/// A fixed `SenderAuthBinding` with the given envelope codepoint. All
/// other fields fixed so the only variation across the table rows is the
/// `envelope_codepoint` (isolating the codepoint-binding freeze).
fn fixed_m_auth_hex(envelope_codepoint: u16) -> String {
    let body_cid = fixed_body_cid_bytes();
    let sender_did = b"did:key:zFixedSenderForBytePin".to_vec();
    let audience_commitment = vec![0xBB; 32];
    let generations: Vec<u32> = vec![7];
    let binding = SenderAuthBinding {
        sig_codepoint: SENDER_AUTH_SIG_CODEPOINT,
        envelope_codepoint,
        sender_did: &sender_did,
        body_cid: &body_cid,
        audience_commitment: &audience_commitment,
        generations: &generations,
        stanza_count: 3,
        body_aad_digest: [0xEE; 32],
    };
    to_hex(&build_m_auth(&binding))
}

/// Encryption-envelope per codepoint — `M_auth` codepoint-binding golden
/// hex for each Layer-C band. One canonical hex per envelope codepoint.
#[test]
fn encryption_envelope_m_auth_per_codepoint_golden_hex() {
    // The rows differ ONLY in the 2-byte envelope_codepoint (BE) bound
    // after `SENDER_AUTH_DOMAIN || sig_codepoint(0x0001)`.
    // 0x6500 — plaintext-sender drop (non-default).
    assert_eq!(
        fixed_m_auth_hex(LAYER_C_DROP),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000165000000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6500 M_auth codepoint-binding drifted"
    );
    // 0x6510 — Sealed-Sender DEFAULT.
    assert_eq!(
        fixed_m_auth_hex(DROP_TO_RECIPIENT_SEALED_SENDER),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000165100000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6510 M_auth codepoint-binding drifted"
    );
    // 0x6520 — group multi-recipient.
    assert_eq!(
        fixed_m_auth_hex(LAYER_C_DROP_MULTI_RECIPIENT),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000165200000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6520 M_auth codepoint-binding drifted"
    );
    // 0x6610 — MembershipSet group multi-stanza.
    assert_eq!(
        fixed_m_auth_hex(MEMBERSHIP_SET_GROUP_MULTI_STANZA),
        "62656e74656e2f6c617965722d632f7365616c65642d73656e6465722d6f726967696e2d617574682f7631000166100000001e6469643a6b65793a7a466978656453656e646572466f724279746550696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000020bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000000010000000700000003eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "0x6610 M_auth codepoint-binding drifted"
    );
}

/// The DEFAULT 0x6510 on-wire envelope AAD serializer — ABSOLUTE golden
/// hex (`aad_version u8 | codepoint u16-BE | aud_len u32-BE | audience |
/// body_cid(36) | recipient_key_gen u32-BE`).
#[test]
fn sealed_sender_0x6510_on_wire_aad_golden_hex() {
    let aad = sealed_aad::SealedSenderAad {
        aad_version: sealed_aad::AAD_VERSION,
        codepoint: sealed_aad::DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        audience_did: b"did:key:zFixedAudienceForPin".to_vec(),
        body_cid: fixed_body_cid_bytes(),
        recipient_key_generation: 9,
    };
    let got = to_hex(&sealed_aad::serialize_sealed_sender_aad(&aad));
    assert_eq!(
        got,
        "0165100000001c6469643a6b65793a7a466978656441756469656e6365466f7250696e01711e20cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc00000009",
        "0x6510 on-wire Sealed-Sender AAD framing drifted"
    );
}

/// The 0x647a cipher codepoint — `AeadEnvelope::to_wire_bytes` framing
/// (the ChaCha20-Poly1305-under-X-Wing envelope). Pins the BE codepoint
/// (M-19) at the encryption-envelope layer.
#[test]
fn cipher_codepoint_0x647a_envelope_golden_hex() {
    let env = AeadEnvelope {
        format_version: 0x01,
        cipher_codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        nonce: vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ],
        ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };
    let got = to_hex(&env.to_wire_bytes());
    assert_eq!(
        got, "ae01647a0c0102030405060708090a0b0cdeadbeef",
        "0x647a cipher-codepoint envelope framing drifted"
    );
}
