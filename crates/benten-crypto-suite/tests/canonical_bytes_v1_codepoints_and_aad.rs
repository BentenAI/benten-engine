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
use benten_crypto_suite::sig::SignatureSuite;
use benten_crypto_suite::varsig::{UcanVarsigV1Header, VarsigError};

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
    assert_eq!(
        HashCodepoint::SHA2_512_256.raw(),
        0x1015,
        "SHA2_512_256 wire-locked at 0x1015 (pre-blessed agile fallback per V1-FROZEN row 6)"
    );
    assert_eq!(
        HashCodepoint::SHA3_256.raw(),
        0x16,
        "SHA3_256 wire-locked at 0x16 (pre-blessed agile fallback per V1-FROZEN row 6)"
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
/// F3 ratification (R6 R1 fix-pass — Fork 1 retraction supersedes the
/// earlier 2-tuple layout). The AAD now binds 4 segments per
/// `crates/benten-crypto-suite/src/aead.rs::aad_per_chunk`.
///
/// **F4-004 / M-19 BIG-ENDIAN migration (RED-PHASE until R5/Wave-0).**
/// The M-19 freeze ratifies that ALL multiformats-framed integer wire/AAD
/// fields are **BIG-ENDIAN** (R0.3 §4.1 row "BE endianness"; Q2/U7). The
/// per-chunk AAD's `chunk_index` (u64) + `total_chunks` (u32) are named
/// M-19 sites. This pin therefore freezes the **BIG-ENDIAN** layout; at
/// HEAD the production `aad_per_chunk` (`aead.rs:244,245`) still emits
/// LITTLE-ENDIAN, so this test is gated `#[ignore]` and the W0/M-19 step
/// that flips the production encoder to BE un-ignores it. This leaves
/// exactly ONE canonical big-endian per-chunk-AAD pin across the corpus
/// (the prior on-main LE assertion is migrated here, not duplicated).
///
/// The exact post-M-19 byte layout is:
///   `b"benten-aead:chunk:" || plaintext_cid || chunk_index.to_be_bytes() || total_chunks.to_be_bytes()`
///
/// The `total_chunks: u32` big-endian segment closes the
/// cross-chunk-truncation attack — an attacker who truncates a 10-chunk
/// ciphertext to 5 chunks cannot fabricate per-chunk AAD-matching tags
/// because the seal-time AAD committed to `total_chunks=10`.
///
/// Any change to the segment order, encoding, or endianness is a wire-format
/// break; this test fails first to signal the freeze-discipline
/// coupling. would-FAIL-if-no-op'd: while the production encoder still
/// emits LE (or if a future refactor reverts to LE), the BE assertions
/// below fail (the explicit `assert_ne!` BE-not-LE guard makes the
/// endianness distinction load-bearing).
#[test]
#[ignore = "RED-PHASE: F4-004 / M-19 — per-chunk AAD MUST be BIG-ENDIAN (aead.rs:244,245 is LE today); un-ignore at R5/Wave-0 when the M-19 step flips the production encoder to BE"]
fn aad_per_chunk_canonical_layout_pinned() {
    // Synthetic plaintext_cid + chunk_index + total_chunks. The chosen
    // values have DISTINCT big-endian and little-endian byte orders, so the
    // endianness assertions actually discriminate the two.
    let plaintext_cid = [0xAA_u8; 32]; // 32-byte CID hash payload
    let chunk_index: u64 = 0x0123_4567_89AB_CDEF;
    let total_chunks: u32 = 0xDEAD_BEEF;

    let aad = aad_per_chunk(&plaintext_cid, chunk_index, total_chunks);

    // Verify layout (4 segments): tag || plaintext_cid || chunk_index BE || total_chunks BE.
    let tag = b"benten-aead:chunk:";
    let expected_len = tag.len() + plaintext_cid.len() + 8 + 4;
    assert_eq!(
        aad.len(),
        expected_len,
        "AAD layout regression — expected (tag || cid || u64-BE chunk_index || u32-BE total_chunks), total {} bytes; got {} bytes",
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

    // Segment 3: chunk_index encoded as BIG-endian u64 (M-19).
    assert_eq!(
        &aad[tag.len() + plaintext_cid.len()..tag.len() + plaintext_cid.len() + 8],
        &chunk_index.to_be_bytes(),
        "AAD chunk_index encoding changed — MUST be BIG-ENDIAN per M-19 (wire-format break)"
    );

    // Segment 4 (F3): total_chunks encoded as BIG-endian u32 (M-19).
    assert_eq!(
        &aad[aad.len() - 4..],
        &total_chunks.to_be_bytes(),
        "AAD total_chunks encoding regression — MUST be BIG-ENDIAN per M-19 (F3 wire-format break)"
    );

    // Explicit hex pin: with cid = 32x 0xAA + chunk_index = 0x0123456789ABCDEF +
    // total_chunks = 0xDEADBEEF, expected tail (BIG-ENDIAN per M-19) =
    //   [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF] (u64-BE chunk_index)
    //   || [0xDE, 0xAD, 0xBE, 0xEF]                       (u32-BE total_chunks)
    let expected_chunk_idx_bytes = [0x01_u8, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
    let expected_total_chunks_bytes = [0xDE_u8, 0xAD, 0xBE, 0xEF];
    assert_eq!(
        &aad[aad.len() - 12..aad.len() - 4],
        &expected_chunk_idx_bytes,
        "AAD u64-BE chunk_index encoding regression — must use to_be_bytes() not to_le_bytes() (M-19)"
    );
    assert_eq!(
        &aad[aad.len() - 4..],
        &expected_total_chunks_bytes,
        "AAD u32-BE total_chunks encoding regression — must use to_be_bytes() not to_le_bytes() (M-19)"
    );

    // would-FAIL guard: the bytes are NOT in little-endian order (the
    // pre-M-19 aead.rs:244,245 layout is migrated away). This makes the
    // endianness flip load-bearing — a reversion to LE fails here.
    assert_ne!(
        &aad[tag.len() + plaintext_cid.len()..tag.len() + plaintext_cid.len() + 8],
        &chunk_index.to_le_bytes(),
        "AAD chunk_index MUST NOT be little-endian (M-19 migrates aead.rs:244 LE → BE)"
    );
    assert_ne!(
        &aad[aad.len() - 4..],
        &total_chunks.to_le_bytes(),
        "AAD total_chunks MUST NOT be little-endian (M-19 migrates aead.rs:245 LE → BE)"
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
        matches!(
            outcome,
            Err(UnsupportedAlgorithm::Signature { codepoint: 0x0003 })
        ),
        "0x0003 must stay typed-rejected at v1-beta default dispatcher (C11b safety gate); got {outcome:?}"
    );

    // Cipher 0x647b (HYBRID_MLKEM768_HQC reserved) — typed-rejected.
    let outcome = CipherSuiteCodepoint::HYBRID_MLKEM768_HQC.resolve();
    assert!(
        matches!(
            outcome,
            Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647b })
        ),
        "0x647b must stay typed-rejected at v1-beta default dispatcher; got {outcome:?}"
    );

    // Cipher 0x647c (PURE_PQ_MLKEM768_ONLY reserved swap-matrix arm) —
    // typed-rejected at the dispatcher; reachable only via audit-gated
    // try_pure_pq_sole_trust_path constructor.
    let outcome = CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY.resolve();
    assert!(
        matches!(
            outcome,
            Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647c })
        ),
        "0x647c must stay typed-rejected at v1-beta default dispatcher; got {outcome:?}"
    );
}

/// L1-crypto-r2-4 closure — sub-pins (b) `SignatureSuite::resolve_codepoint`
/// + (c) `UcanVarsigV1Header::decode` for 0x0003. The L1-MAJOR-1 fix-pass
/// shipped sub-pin (a) (`SigCodepoint::resolve`) at the upstream dispatcher;
/// these two sub-pins explicitly assert typed-reject at EVERY downstream
/// dispatcher entry per the freeze-discipline wording-mutation guard
/// (§3.5g). A future-refactor that decouples either surface from the
/// central SigCodepoint::resolve dispatcher (e.g. introduces a fast-path
/// special-case) would slip past sub-pin (a) but fail one of these.
#[test]
fn reserved_signature_codepoint_typed_rejected_at_every_dispatcher_entry() {
    use benten_crypto_suite::error::UnsupportedAlgorithm;

    // Sub-pin (b): SignatureSuite::resolve_codepoint(0x0003) → typed-reject.
    // (SignatureSuite is not Debug; assert via match arm rather than {:?}).
    match SignatureSuite::resolve_codepoint(SigCodepoint::HYBRID_MLDSA65_SLHDSA) {
        Err(UnsupportedAlgorithm::Signature { codepoint: 0x0003 }) => {}
        Err(other) => panic!(
            "SignatureSuite::resolve_codepoint(0x0003) must typed-reject as UnsupportedAlgorithm::Signature {{ codepoint: 0x0003 }} at v1-beta (C11b safety gate); got Err({other:?})"
        ),
        Ok(_) => panic!(
            "SignatureSuite::resolve_codepoint(0x0003) must typed-reject at v1-beta (C11b safety gate); got Ok(_)"
        ),
    }

    // Sub-pin (c): UcanVarsigV1Header::decode of a header carrying 0x0003
    // → typed-reject.
    let header = UcanVarsigV1Header::with_raw_codepoint_for_test(0x0003);
    let outcome = UcanVarsigV1Header::decode(header.as_bytes());
    assert!(
        matches!(
            outcome,
            Err(VarsigError::UnsupportedCodepoint {
                codepoint: 0x0003,
                ..
            })
        ),
        "UcanVarsigV1Header::decode of a 0x0003-bearing header must typed-reject at v1-beta (C11b safety gate); got {outcome:?}"
    );
}
