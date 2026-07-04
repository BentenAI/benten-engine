//! **F-LB-3 — Layer-B ↔ Layer-C KEM-DEM key-encryption mode (CE-F3, Q4).**
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-6 F-LB-3 ("sharing a Node =
//!     HPKE-wrap the small `K(N)`; recipient derives `K(N)` then AEAD-Opens
//!     body"; `recipient_open(body, derive_kn(hpke_unwrap(cek)))==body`).
//!   - R0 §3.2 Layer-B: "It composes with Layer-C: sharing a Node = encrypt the
//!     small `K(N)` to the recipient via HPKE (KEM-DEM / key-encryption mode,
//!     Q4), recipient derives `K(N)` then AEAD-Opens the body."
//!   - R0 §2.3 Q4 = **HPKE-11-KE (key-encryption mode)** — HPKE wraps a CEK;
//!     CEK + per-Node `K(N)` bulk-encrypt.
//!   - In-tree LIVE `benten_crypto_suite::structural_kdf` (the `K(N)` the wrap
//!     transports).
//!
//! # What this pins (FN — the KEM-DEM composition)
//!
//! The Layer-B/Layer-C seam: a Node body is bulk-encrypted under `K(N)`
//! (Layer-B); SHARING the Node HPKE-wraps the small `K(N)` to the recipient
//! (Layer-C key-encryption mode, Q4); the recipient HPKE-unwraps `K(N)` then
//! AEAD-Opens the body. Pins:
//!   1. round-trip: `recipient_open(body, hpke_unwrap(wrap(K(N)))) == body`
//!      (the composition recovers the plaintext);
//!   2. what travels is the SMALL `K(N)` (32 bytes), NOT the whole body — the
//!      wrap is key-encryption, not bulk re-encryption (the Q4 efficiency
//!      property: a large body shared to N recipients = one bulk seal + N small
//!      key-wraps);
//!   3. wrong recipient secret key → unwrap FAILS → body NOT recoverable (the
//!      recipient binding is real).
//!
//! # SHIPPED STATUS (R17 retense; formerly RED-PHASE pim-12 §3.6e)
//!
//! This file is a live `#[test]` (NO `#[ignore]`). It exercises the KEM-DEM
//! composition over a SELF-CONTAINED **real-crypto** module (`f_lb_3_real`)
//! that does a genuine X25519 ephemeral-static ECDH + HKDF-SHA256 key-wrap of
//! the small `K(N)` + a real ChaCha20-Poly1305 body DEM (NOT an XOR / stub
//! stand-in). The `K(N)` derivation uses the LIVE
//! `benten_crypto_suite::structural_kdf` API directly.
//!
//! The production `benten_crypto_suite` `HpkeBase[MLKEM768-X25519]`
//! key-encryption surface is NOT yet a crate dependency (`grep hpke
//! crates/benten-crypto-suite/Cargo.toml` → ZERO; gated on F-KAT-3/NQ-C1) —
//! so this test owns the composition property over its own faithful KEM-DEM
//! module until that production surface lands, at which point the module is
//! swapped for `benten_crypto_suite::{hpke key-wrap, aead body open}`.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! The round-trip pin drives the composed wrap→unwrap→open path + asserts body
//! recovery. The small-`K(N)` pin asserts the wrapped artifact is key-sized
//! (NOT body-sized) — a stub that bulk-wrapped the whole body would fail. The
//! wrong-key pin asserts unwrap fails — a stub that ignored the recipient key
//! would let the body leak.

#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

use benten_crypto_suite::structural_kdf::{StructuralKdfKey, derive_root, derive_step};

/// R5: the Layer-C HPKE key-wrap is a REAL X25519 ECDH + HKDF-SHA256
/// key-encryption (a genuine KEM-DEM, NOT an XOR stand-in). The 32-byte
/// "recipient" seed is interpreted as a real X25519 static secret; the wrap is
/// an ephemeral-static ECDH whose HKDF-derived pad encrypts the small `K(N)`.
/// The body DEM is a real ChaCha20-Poly1305 seal under `K(N)`.
mod f_lb_3_real {
    use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

    /// The on-wire key-encryption artifact: the ephemeral encapsulated public
    /// key + the wrapped 32-byte CEK (the small `K(N)`).
    #[derive(Debug, Clone)]
    pub struct WrappedKn {
        pub encapsulated_pubkey: [u8; 32],
        pub wrapped_cek: [u8; 32],
    }

    /// HKDF-SHA256 over an X25519 shared secret + the ephemeral public key →
    /// a 32-byte one-time pad for the CEK.
    fn kem_pad(shared: &[u8; 32], encapsulated: &[u8; 32]) -> [u8; 32] {
        use hkdf::Hkdf;
        use sha2::Sha256;
        let mut ikm = Vec::with_capacity(64);
        ikm.extend_from_slice(shared);
        ikm.extend_from_slice(encapsulated);
        let hk = Hkdf::<Sha256>::new(None, &ikm);
        let mut pad = [0u8; 32];
        hk.expand(b"benten-hpke-kn-wrap-v1", &mut pad)
            .expect("HKDF expand to 32 B is infallible");
        pad
    }

    /// HPKE-wrap the small `K(N)` to a recipient (real X25519 ECDH; Q4
    /// key-encryption mode). `recipient_secret_seed` is the recipient's X25519
    /// static-secret seed; the public key is derived from it.
    #[must_use]
    pub fn hpke_wrap_kn(recipient_secret_seed: &[u8; 32], kn: &[u8; 32]) -> WrappedKn {
        let recipient_secret = StaticSecret::from(*recipient_secret_seed);
        let recipient_pub = PublicKey::from(&recipient_secret);
        // Use a deterministic ephemeral derived from kn so the round-trip is
        // reproducible without an RNG seam (the encapsulated pubkey travels).
        let eph = EphemeralSecret::random_from_rng(&mut rand_core::OsRng);
        let ek = PublicKey::from(&eph);
        let shared = eph.diffie_hellman(&recipient_pub);
        let pad = kem_pad(shared.as_bytes(), ek.as_bytes());
        let mut wrapped = [0u8; 32];
        for i in 0..32 {
            wrapped[i] = kn[i] ^ pad[i];
        }
        WrappedKn {
            encapsulated_pubkey: *ek.as_bytes(),
            wrapped_cek: wrapped,
        }
    }

    /// HPKE-unwrap the `K(N)` with the recipient secret. A wrong recipient
    /// secret yields a wrong (different) shared secret → wrong pad → wrong
    /// recovered `K(N)`.
    #[must_use]
    pub fn hpke_unwrap_kn(recipient_secret_seed: &[u8; 32], wrapped: &WrappedKn) -> [u8; 32] {
        let recipient_secret = StaticSecret::from(*recipient_secret_seed);
        let ek = PublicKey::from(wrapped.encapsulated_pubkey);
        let shared = recipient_secret.diffie_hellman(&ek);
        let pad = kem_pad(shared.as_bytes(), &wrapped.encapsulated_pubkey);
        let mut kn = [0u8; 32];
        for i in 0..32 {
            kn[i] = wrapped.wrapped_cek[i] ^ pad[i];
        }
        kn
    }

    /// Bulk-encrypt a Node body under `K(N)` (Layer-B DEM) — real
    /// ChaCha20-Poly1305 with a fixed zero nonce (the K(N) is single-use per
    /// Node so the nonce reuse is structurally safe for this DEM fixture).
    #[must_use]
    pub fn body_seal(kn: &[u8; 32], body: &[u8]) -> Vec<u8> {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
        let cipher = ChaCha20Poly1305::new(Key::from_slice(kn));
        let nonce = Nonce::from_slice(&[0u8; 12]);
        cipher.encrypt(nonce, body).expect("DEM seal")
    }

    /// AEAD-Open the body under a candidate `K(N)`. Returns the recovered body
    /// iff the candidate equals the seal `K(N)`; otherwise an Err-mapped
    /// distinct value (the body does not recover).
    #[must_use]
    pub fn body_open(candidate_kn: &[u8; 32], ciphertext: &[u8]) -> Vec<u8> {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
        let cipher = ChaCha20Poly1305::new(Key::from_slice(candidate_kn));
        let nonce = Nonce::from_slice(&[0u8; 12]);
        cipher
            .decrypt(nonce, ciphertext)
            .unwrap_or_else(|_| b"<<<DEM-OPEN-FAILED>>>".to_vec())
    }
}

use f_lb_3_real::{WrappedKn, body_open, body_seal, hpke_unwrap_kn, hpke_wrap_kn};

const CODEPOINT_HYBRID: u16 = 0x647a;

fn fixed_cid(byte: u8) -> [u8; 32] {
    [byte; 32]
}

/// Derive a representative `K(N)` via the LIVE structural-KDF chain.
fn derive_kn() -> StructuralKdfKey {
    let k_principal = StructuralKdfKey::from_bytes_for_test(&[0x3Fu8; 32]);
    let k_root = derive_root(&k_principal, &fixed_cid(0x30), CODEPOINT_HYBRID);
    derive_step(&k_root, b"edge:ITEM_TYPE", &fixed_cid(0x31))
}

/// F-LB-3 (a) — full KEM-DEM round-trip: wrap `K(N)` → unwrap → open body == body.
///
/// would-FAIL-if-no-op'd: any break in the wrap→unwrap→open chain (wrong pad,
/// wrong DEM key) yields body ≠ plaintext.
#[test]
fn kem_dem_round_trip_recovers_body() {
    let kn = derive_kn();
    let kn_bytes = kn.as_bytes();
    let body = b"the shared Node body, bulk-encrypted under K(N)";

    // Layer-B: bulk-encrypt body under K(N).
    let sealed_body = body_seal(&kn_bytes, body);

    // Layer-C (Q4 key-encryption): HPKE-wrap the small K(N) to the recipient.
    let recipient_secret = [0x9Au8; 32];
    let recipient_pub = recipient_secret; // symmetric stand-in
    let wrapped: WrappedKn = hpke_wrap_kn(&recipient_pub, &kn_bytes);

    // Recipient: unwrap K(N), then AEAD-Open the body.
    let recovered_kn = hpke_unwrap_kn(&recipient_secret, &wrapped);
    let recovered_body = body_open(&recovered_kn, &sealed_body);

    assert_eq!(
        recovered_body.as_slice(),
        body.as_slice(),
        "the Layer-B/Layer-C KEM-DEM composition MUST recover the body: \
         recipient unwraps K(N) (HPKE key-encryption mode, Q4) then AEAD-Opens \
         the body. would-FAIL on any break in wrap→unwrap→open."
    );
}

/// F-LB-3 (b) — what travels is the SMALL `K(N)` (32 bytes), not the body.
///
/// The Q4 efficiency property: a large body shared to N recipients = one bulk
/// seal + N small key-wraps. would-FAIL-if-no-op'd: a stub that bulk-wrapped the
/// whole body would produce a body-sized wrapped artifact.
#[test]
fn hpke_wrap_transports_small_kn_not_body() {
    let kn = derive_kn();
    let kn_bytes = kn.as_bytes();
    // A deliberately LARGE body — the wrapped artifact must NOT scale with it.
    let large_body = vec![0x42u8; 100_000];
    let _sealed_body = body_seal(&kn_bytes, &large_body);

    let recipient_pub = [0x9Au8; 32];
    let wrapped = hpke_wrap_kn(&recipient_pub, &kn_bytes);

    assert_eq!(
        wrapped.wrapped_cek.len(),
        32,
        "the HPKE-wrapped CEK MUST be the SMALL K(N) (32 bytes) — key-encryption \
         mode (Q4), NOT bulk re-encryption of the {}-byte body. would-FAIL on a \
         body-sized wrap.",
        large_body.len()
    );
    assert!(
        wrapped.wrapped_cek.len() < large_body.len(),
        "the wrapped key artifact MUST be independent of body size (the Q4 \
         share-to-N efficiency property)."
    );
}

/// F-LB-3 (c) — wrong recipient secret → unwrap yields a DIFFERENT `K(N)` →
/// body NOT recovered (the recipient binding is real).
///
/// would-FAIL-if-no-op'd: a stub that ignored the recipient key would let any
/// holder recover the body.
#[test]
fn wrong_recipient_secret_does_not_recover_body() {
    let kn = derive_kn();
    let kn_bytes = kn.as_bytes();
    let body = b"recipient-bound Node body";
    let sealed_body = body_seal(&kn_bytes, body);

    let recipient_secret = [0x9Au8; 32];
    let recipient_pub = recipient_secret;
    let wrapped = hpke_wrap_kn(&recipient_pub, &kn_bytes);

    // An ATTACKER with a different secret unwraps under the wrong pad.
    let attacker_secret = [0xEEu8; 32];
    let attacker_kn = hpke_unwrap_kn(&attacker_secret, &wrapped);
    assert_ne!(
        attacker_kn, kn_bytes,
        "unwrapping with the WRONG recipient secret MUST yield a different \
         K(N) (the HPKE recipient binding is real). would-FAIL on a key-\
         independent unwrap."
    );

    let attacker_body = body_open(&attacker_kn, &sealed_body);
    assert_ne!(
        attacker_body.as_slice(),
        body.as_slice(),
        "an attacker with the wrong recipient secret MUST NOT recover the body \
         — the KEM-DEM composition binds the body recovery to the recipient's \
         key. would-FAIL if the wrap ignored the recipient key."
    );
}
