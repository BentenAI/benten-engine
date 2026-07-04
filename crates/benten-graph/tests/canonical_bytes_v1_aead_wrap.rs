//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pins for the
//! per-chunk / per-Node AEAD storage-envelope wire formats (Rows D-9 /
//! D-79).
//!
//! Per Row D-9, the per-chunk-AAD-*layout* pin already landed
//! (`benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs`).
//! This file adds the MISSING ABSOLUTE hex byte-pins for the two frozen
//! encoders on the storage path:
//!
//! - `AeadEnvelope::to_wire_bytes()` — the cipher-suite envelope framing
//!   (`magic 0xae | format_version | cipher_codepoint u16-BE | nonce_len
//!   | nonce | ciphertext`).
//! - `encode_encrypted_node()` — the G-CORE-3d storage envelope framing
//!   (`magic 0x3d | variant tag | plaintext_cid(36) | AeadEnvelope wire`)
//!   for the `Whole` arm.
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! A real `wrap()` call is NON-deterministic (random ChaCha20-Poly1305
//! nonce). So the fixtures do NOT call `wrap()`; they construct an
//! `AeadEnvelope` DIRECTLY from FIXED fields (fixed nonce + fixed
//! ciphertext bytes) via the public struct, then pin the deterministic
//! wire encoding. That is exactly the frozen envelope framing surface.
//! The plaintext CID is a fixed BLAKE3-digest CID. Hex captured via
//! throwaway compute + pasted below.
//!
//! would-FAIL-on-drift: magic byte, format-version, codepoint endianness
//! (M-19 BE; LE would swap `647a`→`7a64`), nonce-length byte, field
//! order, or the storage variant tag all change these bytes.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_crypto_suite::aead::AeadEnvelope;
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_graph::aead_wrap::{EncryptedNode, encode_encrypted_node};

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Fixed CID from a constant BLAKE3 digest (0xAA x32) — deterministic.
fn fixed_cid() -> Cid {
    Cid::from_blake3_digest([0xAA; 32])
}

/// A fixed `AeadEnvelope` at the default 0x647a codepoint with a fixed
/// 12-byte nonce + fixed ciphertext. All fields literal so the wire
/// bytes are deterministic.
fn fixed_envelope() -> AeadEnvelope {
    AeadEnvelope {
        format_version: 0x01,
        cipher_codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, // 0x647a
        nonce: vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ],
        ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
    }
}

#[test]
fn aead_envelope_to_wire_bytes_golden_hex_0x647a() {
    let got = to_hex(&fixed_envelope().to_wire_bytes());
    // ae(magic) 01(fmt) 647a(codepoint BE) 0c(nonce_len=12)
    //   || 010203..0c (nonce) || deadbeef (ciphertext)
    let expected = "ae01647a0c0102030405060708090a0b0cdeadbeef";
    assert_eq!(
        got, expected,
        "AeadEnvelope wire framing drifted from the frozen v1-beta bytes"
    );
}

#[test]
fn encode_encrypted_node_whole_golden_hex() {
    let node = EncryptedNode::Whole {
        plaintext_cid: fixed_cid(),
        envelope: fixed_envelope(),
    };
    let bytes = encode_encrypted_node(&node).unwrap();
    let got = to_hex(&bytes);
    // 3d(storage magic) 00(Whole variant) || plaintext_cid(36) ||
    //   AeadEnvelope::to_wire_bytes()
    let expected = "3d0001711e20aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaae01647a0c0102030405060708090a0b0cdeadbeef";
    assert_eq!(
        got, expected,
        "encode_encrypted_node(Whole) storage framing drifted from the frozen v1-beta bytes"
    );
}
