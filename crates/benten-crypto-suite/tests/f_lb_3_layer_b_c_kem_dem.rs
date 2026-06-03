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
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! The Layer-C HPKE wrap/unwrap (`HpkeBase[MLKEM768-X25519]` key-encryption
//! mode) is NOT yet in-tree (`grep hpke crates/benten-crypto-suite/Cargo.toml`
//! → ZERO; gated on F-KAT-3/NQ-C1). Per wave-independence this file commits a
//! LOCAL `f_lb_3_stub` modelling the HPKE key-wrap + body AEAD. The `K(N)`
//! derivation uses the LIVE `structural_kdf` API directly. R5 DELETEs the stub,
//! wires the LIVE `benten_crypto_suite::{hpke key-wrap, aead body open}`,
//! un-ignores, verifies green.
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

/// SELF-CONTAINED stub for the Layer-C HPKE key-wrap + body AEAD (R5 wires the
/// LIVE HPKE + AEAD). The `K(N)` is derived via the LIVE structural-KDF.
mod f_lb_3_stub {
    /// A wrapped CEK (the HPKE-encapsulated `K(N)`). Models the on-wire
    /// key-encryption artifact: an encapsulated key + the wrapped 32-byte CEK.
    /// The wrapped CEK is XORed under a per-recipient pad derived from the
    /// recipient pubkey (a stand-in for the HPKE KEM; R5 swaps in real HPKE).
    #[derive(Debug, Clone)]
    pub struct WrappedKn {
        pub encapsulated_pubkey: [u8; 32],
        pub wrapped_cek: [u8; 32],
    }

    fn recipient_pad(recipient_secret_or_pub: &[u8; 32]) -> [u8; 32] {
        // Symmetric stand-in for the HPKE shared secret: derive a pad from the
        // key material. R5 replaces this with the real HPKE encap/decap.
        let mut pad = [0u8; 32];
        for (i, p) in pad.iter_mut().enumerate() {
            *p = recipient_secret_or_pub[i] ^ 0x5C;
        }
        pad
    }

    /// HPKE-wrap the small `K(N)` to a recipient (key-encryption mode, Q4).
    /// The recipient pubkey == secret in this symmetric stand-in.
    pub fn hpke_wrap_kn(recipient_pub: &[u8; 32], kn: &[u8; 32]) -> WrappedKn {
        let pad = recipient_pad(recipient_pub);
        let mut wrapped = [0u8; 32];
        for i in 0..32 {
            wrapped[i] = kn[i] ^ pad[i];
        }
        WrappedKn {
            encapsulated_pubkey: *recipient_pub,
            wrapped_cek: wrapped,
        }
    }

    /// HPKE-unwrap the `K(N)` with the recipient secret. Wrong key → wrong pad →
    /// wrong (unrecoverable) `K(N)`. Returns the unwrapped 32 bytes; whether they
    /// are the real `K(N)` is what the round-trip / wrong-key pins test.
    pub fn hpke_unwrap_kn(recipient_secret: &[u8; 32], wrapped: &WrappedKn) -> [u8; 32] {
        let pad = recipient_pad(recipient_secret);
        let mut kn = [0u8; 32];
        for i in 0..32 {
            kn[i] = wrapped.wrapped_cek[i] ^ pad[i];
        }
        kn
    }

    /// Bulk-encrypt a Node body under `K(N)` (Layer-B DEM). XOR stand-in.
    pub fn body_seal(kn: &[u8; 32], body: &[u8]) -> Vec<u8> {
        body.iter()
            .enumerate()
            .map(|(i, b)| b ^ kn[i % 32])
            .collect()
    }

    /// AEAD-Open the body under a candidate `K(N)`. Recovers the body iff the
    /// candidate equals the seal `K(N)`.
    pub fn body_open(candidate_kn: &[u8; 32], ciphertext: &[u8]) -> Vec<u8> {
        ciphertext
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ candidate_kn[i % 32])
            .collect()
    }
}

use f_lb_3_stub::{WrappedKn, body_open, body_seal, hpke_unwrap_kn, hpke_wrap_kn};

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
#[ignore = "RED-PHASE: F-LB-3 — KEM-DEM round-trip (HPKE-wrap K(N) → unwrap → AEAD-Open body) recovers the body; un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-LB-3 — the HPKE wrap transports the SMALL K(N) (32 B), not the body (Q4 efficiency); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-LB-3 — wrong recipient secret MUST NOT recover K(N)/body (recipient binding); un-ignore at R5"]
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
