//! G-CORE-9 R1 fix-pass — Bundle 5 wire-format byte-pins.
//!
//! Hex-pinned byte-pin tests for the freeze-load-bearing wire-format
//! surfaces named in V1-FROZEN-INTERFACE.md item 4. Per the G-CORE-9
//! R1 triage Bundle 5 PARTIAL disposition, this file ships the
//! highest-leverage pins (codepoint integer values + per-chunk AEAD
//! AAD layout); the remaining 6 byte-pin tests for SnapshotBlob v2 /
//! UCAN-Varsig / AuthorizationGrant CBOR / Drop bundle CBOR /
//! encryption-envelope per codepoint / signature-envelope per
//! codepoint are deferred to G-COMP-1 per
//! `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-9.
//!
//! Closes L11-MAJOR-4 (codepoint integer values not pinned) +
//! L11-MAJOR-1 partial (the AAD-layout doc-vs-code drift is closed
//! at the doc side per Fork 1; this test pins the code side so future
//! work can't silently augment to a 3-tuple without updating both
//! sides + the failing pin signals).

use benten_crypto_suite::aead::aad_per_chunk;
use benten_crypto_suite::codepoint::{CipherSuiteCodepoint, HashCodepoint, SigCodepoint};

/// L11-MAJOR-4 closure — codepoint table integer values frozen.
///
/// Any value drift here = wire-format break. The v1-FROZEN codepoint
/// table at V1-FROZEN-INTERFACE.md row 6 is the canonical reference;
/// this test pins each integer value at the type level.
#[test]
fn codepoint_table_integer_values_pinned() {
    // Signature codepoints (V1-FROZEN row 6).
    assert_eq!(
        SigCodepoint::HYBRID_ED25519_MLDSA65.raw(),
        0x0001,
        "HYBRID_ED25519_MLDSA65 wire-locked at 0x0001 (v1-beta DEFAULT per NF-4)"
    );
    assert_eq!(
        SigCodepoint::CLASSICAL_ED25519.raw(),
        0x0002,
        "CLASSICAL_ED25519 wire-locked at 0x0002 (non-default downgrade)"
    );
    assert_eq!(
        SigCodepoint::HYBRID_MLDSA65_SLHDSA.raw(),
        0x0003,
        "HYBRID_MLDSA65_SLHDSA wire-locked at 0x0003 (reserved swap-matrix arm; typed-rejected by default per C11b)"
    );

    // Hash codepoints (V1-FROZEN row 6).
    assert_eq!(
        HashCodepoint::BLAKE3.raw(),
        0x1e,
        "BLAKE3 wire-locked at 0x1e (v1-beta DEFAULT)"
    );

    // Cipher-suite codepoints (V1-FROZEN row 6).
    assert_eq!(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw(),
        0x647a,
        "HYBRID_X25519_MLKEM768 wire-locked at 0x647a (v1-beta DEFAULT per X-Wing combiner)"
    );
    assert_eq!(
        CipherSuiteCodepoint::CLASSICAL_X25519.raw(),
        0x6400,
        "CLASSICAL_X25519 wire-locked at 0x6400 (non-default classical-only downgrade)"
    );
    assert_eq!(
        CipherSuiteCodepoint::NONE_PLAINTEXT.raw(),
        0x0000,
        "NONE_PLAINTEXT wire-locked at 0x0000 (non-default plaintext-partition downgrade)"
    );
    assert_eq!(
        CipherSuiteCodepoint::HYBRID_MLKEM768_HQC.raw(),
        0x647b,
        "HYBRID_MLKEM768_HQC wire-locked at 0x647b (reserved-unimplemented NF-1 KEM end-state)"
    );
    assert_eq!(
        CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY.raw(),
        0x647c,
        "PURE_PQ_MLKEM768_ONLY wire-locked at 0x647c (reserved swap-matrix arm; typed-rejected per C11b)"
    );
}

/// L11-MAJOR-1 closure (code-side; doc-side closure is Bundle 7) —
/// per-chunk AEAD AAD layout pinned at the as-shipped 2-arg shape per
/// Fork 1 ratification (doc retracted from 3-tuple to 2-tuple).
///
/// The exact byte layout is:
///   `b"benten-aead:chunk:" || plaintext_cid || chunk_index.to_le_bytes()`
///
/// Any change to add `total_chunks` or any other component is a
/// wire-format break; this test fails first to signal the
/// freeze-discipline coupling.
#[test]
fn aad_per_chunk_canonical_layout_pinned() {
    // Synthetic plaintext_cid + chunk_index.
    let plaintext_cid = [0xAA_u8; 32]; // 32-byte CID hash payload
    let chunk_index: u64 = 0x0123_4567_89AB_CDEF;

    let aad = aad_per_chunk(&plaintext_cid, chunk_index);

    // Verify layout (3 segments): tag || plaintext_cid || chunk_index LE.
    let tag = b"benten-aead:chunk:";
    let expected_len = tag.len() + plaintext_cid.len() + 8;
    assert_eq!(
        aad.len(),
        expected_len,
        "AAD layout regression — expected (tag || cid || u64-LE), total {} bytes; got {} bytes",
        expected_len,
        aad.len()
    );

    // Segment 1: domain-separation tag.
    assert_eq!(
        &aad[..tag.len()],
        tag,
        "AAD domain-separation tag changed — wire-format break"
    );

    // Segment 2: plaintext CID bytes (32 bytes for this synthetic).
    assert_eq!(
        &aad[tag.len()..tag.len() + plaintext_cid.len()],
        &plaintext_cid,
        "AAD plaintext_cid binding changed — wire-format break"
    );

    // Segment 3: chunk_index encoded as little-endian u64.
    assert_eq!(
        &aad[tag.len() + plaintext_cid.len()..],
        &chunk_index.to_le_bytes(),
        "AAD chunk_index encoding changed — wire-format break"
    );

    // Explicit hex pin: with cid = 32x 0xAA + chunk_index = 0x0123456789ABCDEF,
    // expected bytes = b"benten-aead:chunk:" || [0xAA]*32 || [0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]
    let expected_tail = [0xEF_u8, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01];
    assert_eq!(
        &aad[aad.len() - 8..],
        &expected_tail,
        "AAD u64-LE encoding regression — must use to_le_bytes() not to_be_bytes()"
    );
}

/// L11-MINOR-3 closure — typed-reject regression-guard for the
/// reserved/unsupported codepoints at the dispatcher boundaries.
/// Old codepoints supported forever; reserved codepoints stay
/// typed-rejected. A regression that silently routes 0x0003 / 0x647b
/// / 0x647c to a real implementation = downgrade-attack vector.
#[test]
fn reserved_codepoints_stay_typed_rejected_at_v1_beta() {
    use benten_crypto_suite::error::UnsupportedAlgorithm;

    // Signature 0x0003 (HYBRID_MLDSA65_SLHDSA NF-1 end-state) — typed-rejected
    // by default at v1-beta per C11b safety gate.
    let outcome = SigCodepoint::HYBRID_MLDSA65_SLHDSA.resolve();
    assert!(
        matches!(outcome, Err(UnsupportedAlgorithm::Signature { codepoint: 0x0003 })),
        "0x0003 must stay typed-rejected at v1-beta default dispatcher (C11b safety gate); got {outcome:?}"
    );

    // Cipher 0x647b (HYBRID_MLKEM768_HQC reserved) — typed-rejected.
    let outcome = CipherSuiteCodepoint::HYBRID_MLKEM768_HQC.resolve();
    assert!(
        matches!(outcome, Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647b })),
        "0x647b must stay typed-rejected at v1-beta default dispatcher; got {outcome:?}"
    );

    // Cipher 0x647c (PURE_PQ_MLKEM768_ONLY reserved swap-matrix arm) —
    // typed-rejected at the dispatcher; reachable only via audit-gated
    // try_pure_pq_sole_trust_path constructor.
    let outcome = CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY.resolve();
    assert!(
        matches!(outcome, Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647c })),
        "0x647c must stay typed-rejected at v1-beta default dispatcher; got {outcome:?}"
    );
}
