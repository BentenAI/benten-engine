//! TF-3a pin (G-CORE-3a Gate 20 — R4-FP-1 closure) — UCAN Varsig v1
//! header SHAPE assertion: the header carries the hybrid signature with
//! a parseable multiformats prefix advertising the hybrid codepoint.
//!
//! ADDL R4-FP-1 (corpus-edit; closes R4.1 L4 M-5 + cross-confirmed
//! L1-MINOR-3) — Phase-4-Meta-Core G-CORE-3a. Pin sources:
//!   - `R2-test-landscape.md` §6 Gate 20 (UCAN Varsig v1 header carries
//!     the hybrid): "Hybrid-sig substrate is GREEN at main via tf2_*,
//!     but the *Varsig-header-shape* assertion (the load-bearing
//!     CLAUDE.md #5 'multiformats framing IS the permanent commitment')
//!     is absent. Without it, R5 could ship raw concatenated bytes
//!     that wire-break Varsig-aware peers."
//!   - `.addl/phase-4-meta/R4.1-TRIAGE.md` finding M-5 + m-3 (L4 wire-
//!     format-conformance + L1 coverage cross-confirmed, FIX-NOW):
//!     "Gate 20 has NO direct R3 pin."
//!   - CLAUDE.md baked-in #5 (the *permanent* commitment is the
//!     self-describing **multiformats framing** — NOT any one algorithm
//!     — `CIDv1/multihash/multicodec/did:key/UCAN-Varsig`). The
//!     existing `tf2_codepoint_dispatch_typed_unsupported_classical_
//!     downgrade.rs::tf2_ucan_varsig_v1_hybrid_header_round_trip` tests
//!     the round-trip + typed-unsupported behavior; THIS file tests the
//!     load-bearing *header-shape contract*: the magic byte, the version
//!     byte, the codepoint position bytes, and the payload-after-prefix
//!     layout are parseable to a `Varsig-aware peer` byte-by-byte.
//!
//! # NOT a RED-PHASE staged-pin (verify-stays regression-guard at HEAD)
//!
//! The `UcanVarsigV1Header` + `HybridSignature` + `SignatureSuite`
//! surfaces are SHIPPED at HEAD `c9c11c56` (per §3.5n ground-truth-
//! verified at `crates/benten-crypto-suite/src/varsig.rs:33` +
//! `:62` + `:71`). This pin is a regression-guard against the
//! permanent-commitment-property: a future change must not silently
//! collapse the header to raw concatenated bytes (which would compile
//! green + round-trip green within Benten but wire-break Varsig-aware
//! peers outside).
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! The pin exercises the LIVE production wire-format (no stubs). The
//! assertion enumerates the multiformats prefix byte-by-byte:
//!   1. Byte 0 = `0xb5` (Varsig magic byte; load-bearing identifier)
//!   2. Byte 1 = `0x01` (Varsig v1 version byte)
//!   3. Bytes 2-3 = the signature codepoint, little-endian u16
//!   4. Bytes 4+ = the signature payload
//!
//! The HYBRID arm asserts bytes 2-3 == `SigCodepoint::HYBRID_ED25519_MLDSA65`
//! (the v1-beta default codepoint per CLAUDE.md baked-in #5 reframe).
//! The NEGATIVE arm asserts a raw-concatenated-bytes layout WITHOUT the
//! magic prefix would FAIL the parse (the Gate-20 contract that a
//! Varsig-aware peer can distinguish a properly-framed header from raw
//! bytes).
//!
//! # §3.13 per-test-static decomposition
//!
//! No process-scoped shared state. Each test mints its own keypair +
//! per-test message bytes.
//!
//! # §3.5g cross-language note
//!
//! Couples with the TS-side pin
//! `bindings/napi/__tests__/authorization_grant_ts_round_trip.spec.ts`
//! second `it()` block (the binding_sig field round-trips a Varsig-
//! multiformats header) — the Rust-side here is the source-of-truth
//! shape; the TS-side asserts the napi serialization preserves it
//! byte-identically.

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::SignatureSuite;
use benten_crypto_suite::codepoint::SigCodepoint;
use benten_crypto_suite::varsig::{UcanVarsigV1Header, VarsigError};

const VARSIG_MAGIC_BYTE: u8 = 0xb5;
const VARSIG_V1_VERSION_BYTE: u8 = 0x01;

/// Gate 20 (positive) — the Varsig v1 header byte-layout encodes the
/// hybrid signature with the load-bearing multiformats prefix
/// (magic + version + codepoint).
#[test]
fn tf3a_ucan_varsig_v1_header_layout_carries_hybrid_codepoint() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"Gate-20 Varsig-header-shape: hybrid signature on the wire";
    let sig = suite.sign(&kp, msg);

    let header = UcanVarsigV1Header::encode_hybrid(&sig);
    let bytes = header.as_bytes();

    // The header MUST be parseable to a Varsig-aware peer byte-by-byte.
    // The load-bearing property per CLAUDE.md baked-in #5: the multi-
    // formats framing IS the permanent commitment; a peer that knows
    // only "this is Varsig-v1" can locate the codepoint without
    // pre-coordinating on the algorithm.
    assert!(
        bytes.len() >= 4,
        "Gate 20: Varsig v1 header MUST be at least 4 bytes (magic + \
         version + codepoint); got {} bytes",
        bytes.len()
    );

    // Byte 0 — magic byte (multiformats identifier).
    assert_eq!(
        bytes[0], VARSIG_MAGIC_BYTE,
        "Gate 20: byte-0 of the Varsig v1 header MUST be the magic byte \
         (0xb5); load-bearing identifier for a Varsig-aware peer; got 0x{:02x}",
        bytes[0]
    );

    // Byte 1 — version byte.
    assert_eq!(
        bytes[1], VARSIG_V1_VERSION_BYTE,
        "Gate 20: byte-1 of the Varsig v1 header MUST be the v1 version \
         byte (0x01); got 0x{:02x}",
        bytes[1]
    );

    // Bytes 2-3 — codepoint, little-endian u16.
    let codepoint_bytes = [bytes[2], bytes[3]];
    let codepoint_raw = u16::from_le_bytes(codepoint_bytes);
    assert_eq!(
        codepoint_raw,
        SigCodepoint::HYBRID_ED25519_MLDSA65.raw(),
        "Gate 20: bytes 2-3 of the v1-beta-default Varsig header MUST \
         carry the hybrid Ed25519⊕ML-DSA-65 codepoint (the v1-beta \
         default per CLAUDE.md baked-in #5 reframe; codepoint advertises \
         the hybrid algorithm so a Varsig-aware peer can dispatch \
         without pre-coordinating)"
    );

    // Sanity — the header's own advertises_hybrid() agrees with our
    // direct-byte read (defense-in-depth against a future divergence
    // between the helper + the wire layout).
    assert!(
        header.advertises_hybrid(),
        "Gate 20: header.advertises_hybrid() MUST agree with the direct \
         byte-2/3 codepoint read (consistency guard against a future \
         divergence between the convenience helper and the wire bytes)"
    );

    // Payload-after-prefix — the signature payload starts at byte 4.
    // The exact length is codepoint-dispatched (no hardcoded Ed25519
    // 32/64 assumption per CLAUDE.md baked-in #5); we assert ONLY that
    // SOME payload follows the prefix (the load-bearing "the header is
    // not just a sentinel; it actually carries the hybrid bytes" check).
    assert!(
        bytes.len() > 4,
        "Gate 20: header bytes MUST extend past the 4-byte prefix into \
         the hybrid signature payload (would-FAIL if the encoder \
         silently dropped the payload — a regression that would compile \
         green within Benten but wire-break peer verification)"
    );

    // Decode round-trip — the live decoder MUST recover the header to
    // the same hybrid signature shape (regression-guard for the
    // permanent-commitment property).
    let decoded = UcanVarsigV1Header::decode(bytes)
        .expect("Gate 20: a properly-framed Varsig v1 hybrid header MUST decode (the load-bearing parseability property)");
    assert_eq!(
        decoded.codepoint().raw(),
        SigCodepoint::HYBRID_ED25519_MLDSA65.raw(),
        "Gate 20: decoded codepoint MUST equal encoded codepoint \
         (round-trip property; would-FAIL on any encoder/decoder drift)"
    );
}

/// Gate 20 (negative) — raw-concatenated-bytes WITHOUT the Varsig
/// magic prefix MUST fail the parse. This is the load-bearing
/// "permanent commitment IS the framing" property: a Varsig-aware peer
/// must be able to *distinguish* a properly-framed header from raw
/// bytes; if the decoder silently accepted raw bytes, the framing
/// commitment would be vacuous.
#[test]
fn tf3a_ucan_varsig_v1_raw_bytes_without_magic_fail_to_parse() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"Gate-20 negative: raw bytes without Varsig framing";
    let sig = suite.sign(&kp, msg);

    // Build raw-concatenated bytes WITHOUT the multiformats prefix.
    // The shape is "what a sloppy implementer might emit if they
    // forgot to wrap the signature in a Varsig header" — i.e. the
    // failure class Gate 20 defends against.
    let header = UcanVarsigV1Header::encode_hybrid(&sig);
    let framed = header.as_bytes();
    let raw_payload = &framed[4..]; // strip the 4-byte prefix

    // A Varsig-aware peer decoding raw_payload MUST fail. The exact
    // typed-error arm depends on what the first byte happens to be:
    //   - If raw_payload[0] != VARSIG_MAGIC_BYTE → `VarsigError::BadMagic`
    //   - If raw_payload[0] == VARSIG_MAGIC_BYTE (vanishingly rare random
    //     coincidence on a real signature) → some downstream parse
    //     failure (truncated / unsupported version / unknown codepoint
    //     / payload-length mismatch).
    let outcome = UcanVarsigV1Header::decode(raw_payload);
    assert!(
        outcome.is_err(),
        "Gate 20 negative: a Varsig-aware peer MUST FAIL to decode \
         raw-concatenated bytes that lack the multiformats prefix. \
         Silent acceptance would make the multiformats-framing-as- \
         permanent-commitment property (CLAUDE.md baked-in #5) \
         VACUOUS — the whole point is that a peer can distinguish a \
         framed header from raw bytes. Got Ok: {outcome:?}"
    );

    // For the overwhelmingly-common case raw_payload[0] != 0xb5, the
    // typed error MUST be the `BadMagic` arm (NOT a silent classical
    // fallback to bare-Ed25519 parsing — that would be the exact
    // never-silent-fallback violation per CLAUDE.md baked-in #5).
    if raw_payload[0] != VARSIG_MAGIC_BYTE {
        assert!(
            matches!(outcome, Err(VarsigError::BadMagic { .. })),
            "Gate 20 negative: when raw bytes lack the 0xb5 magic, the \
             decoder MUST surface BadMagic (typed error), not silently \
             fall back to a classical-Ed25519 parse. Got {outcome:?}"
        );
    }
}
