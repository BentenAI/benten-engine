//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pins for the
//! UCAN-Varsig-v1 header + the signature-envelope-per-codepoint table
//! (Rows D-9 / D-79).
//!
//! # What this pins
//!
//! Per Row D-9, the codepoint-*integer-value* pin + the AAD-layout pin
//! already landed (`canonical_bytes_v1_codepoints_and_aad.rs`). This
//! file adds the two remaining ABSOLUTE hex byte-pins that Row D-9 named
//! MISSING for this crate:
//!
//! 3. **UCAN-Varsig v1 header** — the Varsig-framed signature header
//!    (`[magic 0xb5 | version 0x01 | codepoint u16-BE | payload]`).
//! 6b. **signature-envelope per codepoint** — one canonical hex per
//!    SHIPPED signature codepoint (0x0001 LAMPS hybrid + 0x0002
//!    classical-only). The reserved 0x0003 arm is NOT wire-emittable
//!    at v1-beta (typed-rejected at `encode_hybrid`'s upstream
//!    dispatch), so its wire-format is pinned via the with-raw-codepoint
//!    header shape at the encoder boundary only.
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! A real hybrid signature is NOT byte-deterministic (ML-DSA-65 signing
//! is hedged/randomized + keygen uses `OsRng`). So the fixtures below do
//! NOT *produce* signatures — they construct a `HybridSignature` from
//! FIXED classical + FIXED pq bytes via the public
//! `HybridSignature::from_parts_internal`, then pin the deterministic
//! wire ENCODING (`to_wire_bytes` / `UcanVarsigV1Header::encode_hybrid`).
//! That is precisely the frozen wire-format surface: the magic byte, the
//! version byte, the BIG-ENDIAN codepoint (M-19), and the ML-DSA-FIRST
//! concat order. Any drift in those fails the pin.
//!
//! The fixtures use short FIXED "signature" halves (0x11.. classical,
//! 0x22.. pq) rather than the real 64-B / 3309-B lengths so the golden
//! hex is compact + human-auditable; the header framing bytes (magic /
//! version / codepoint / concat-order) are identical regardless of
//! payload length, so the framing freeze is fully exercised.

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::codepoint::SigCodepoint;
use benten_crypto_suite::sig::HybridSignature;
use benten_crypto_suite::varsig::UcanVarsigV1Header;

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// UCAN-Varsig v1 header — ABSOLUTE golden hex for the 0x0001 hybrid
/// default framing.
///
/// Fixture: `HybridSignature::from_parts_internal(0x0001, classical =
/// [0x11; 4], pq = [0x22; 6])`. The payload is the ML-DSA-FIRST concat
/// `pq || classical` = `222222222222 11111111`. The header prepends
/// `b5` (magic) `01` (version) `0001` (codepoint BE).
///
/// Expected total layout: `b5 01 0001 || 222222222222 || 11111111`.
///
/// would-FAIL-on-drift: flipping the magic, the version, the codepoint
/// endianness (LE would be `0100`), OR the concat order (classical-first
/// would put `11111111` before `222222222222`) all change these bytes.
#[test]
fn varsig_v1_header_hybrid_0x0001_golden_hex() {
    let sig = HybridSignature::from_parts_internal(
        SigCodepoint::HYBRID_ED25519_MLDSA65,
        vec![0x11; 4], // classical (Ed25519 tradSig) fixed bytes
        vec![0x22; 6], // pq (ML-DSA-65 mldsaSig) fixed bytes
    );
    let header = UcanVarsigV1Header::encode_hybrid(&sig);
    let got = to_hex(header.as_bytes());

    // magic b5 | version 01 | codepoint 0001 (BE) | pq(6) 22..22 | classical(4) 11..11
    let expected = "b5010001222222222222 11111111".replace(' ', "");
    assert_eq!(got, expected, "Varsig v1 hybrid header framing drifted");
}

/// UCAN-Varsig v1 header — ABSOLUTE golden hex for the 0x0002
/// classical-only downgrade framing.
///
/// Fixture: `HybridSignature::from_parts_internal(0x0002, classical =
/// [0x33; 5], pq = [])`. The ML-DSA-first concat degenerates to the bare
/// classical bytes (empty pq). Header = `b5 01 0002 || 3333333333`.
///
/// would-FAIL-on-drift: codepoint endianness (LE `0200`), magic, version.
#[test]
fn varsig_v1_header_classical_0x0002_golden_hex() {
    let sig = HybridSignature::from_parts_internal(
        SigCodepoint::CLASSICAL_ED25519,
        vec![0x33; 5], // classical-only
        Vec::new(),    // no pq half
    );
    let header = UcanVarsigV1Header::encode_hybrid(&sig);
    let got = to_hex(header.as_bytes());

    let expected = "b5010002 3333333333".replace(' ', "");
    assert_eq!(
        got, expected,
        "Varsig v1 classical-only header framing drifted"
    );
}

/// Signature-envelope per codepoint — `HybridSignature::to_wire_bytes`
/// ABSOLUTE golden hex (the payload the Varsig header wraps), one per
/// shipped sig codepoint.
///
/// This pins the ML-DSA-FIRST concat contract at the payload layer
/// independent of the Varsig framing (a future refactor that keeps the
/// Varsig header stable but reorders the concat would still fail here).
#[test]
fn sig_envelope_to_wire_bytes_per_codepoint_golden_hex() {
    // 0x0001 hybrid: pq(0x22 x6) FIRST, then classical(0x11 x4).
    let hybrid = HybridSignature::from_parts_internal(
        SigCodepoint::HYBRID_ED25519_MLDSA65,
        vec![0x11; 4],
        vec![0x22; 6],
    );
    let hybrid_hex = to_hex(&hybrid.to_wire_bytes());
    let expected_hybrid = "222222222222 11111111".replace(' ', "");
    assert_eq!(
        hybrid_hex, expected_hybrid,
        "0x0001 signature-envelope wire (ML-DSA-first concat) drifted"
    );

    // 0x0002 classical-only: bare classical bytes (empty pq).
    let classical = HybridSignature::from_parts_internal(
        SigCodepoint::CLASSICAL_ED25519,
        vec![0x33; 5],
        Vec::new(),
    );
    let classical_hex = to_hex(&classical.to_wire_bytes());
    let expected_classical = "3333333333";
    assert_eq!(
        classical_hex, expected_classical,
        "0x0002 signature-envelope wire (bare classical) drifted"
    );
}
