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

use benten_crypto_suite::aead::{aad_per_chunk, aad_per_recipe, aad_whole_content};
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
/// **F4-004 / M-19 BIG-ENDIAN migration (LANDED).**
/// The M-19 freeze ratifies that ALL multiformats-framed integer wire/AAD
/// fields are **BIG-ENDIAN** (R0.5 §4.1 row "BE endianness"; Q2/U7). The
/// per-chunk AAD's `chunk_index` (u64) + `total_chunks` (u32) are named
/// M-19 sites. This pin freezes the **BIG-ENDIAN** layout; the production
/// `aad_per_chunk` (`aead.rs`) emits BIG-ENDIAN (the M-19 BE migration has
/// landed — see the `to_be_bytes()` sites). This is the single canonical
/// big-endian per-chunk-AAD pin across the corpus.
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
/// coupling. would-FAIL-if-no-op'd: if a future refactor reverts the
/// production encoder to LE, the BE assertions below fail (the explicit
/// `assert_ne!` BE-not-LE guard makes the endianness distinction
/// load-bearing).
#[test]
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

/// E-07 closure — the whole-content AEAD AAD layout, which had NO golden.
///
/// This is the arm that seals every payload under `WHOLE_CONTENT_AEAD_THRESHOLD`
/// (64 KiB): the COMMON case. The falsification sweep swapped its two segments
/// at `aead.rs:234-236` and the crate stayed 210/210 + 14/14 PASS. Nothing
/// compared the produced AAD against an expected byte layout —
/// `aad_whole_content` is a seal-local builder with no decoder, so seal and open
/// agree bilaterally on ANY segment order and every round-trip test stays green.
/// The sibling `aad_per_chunk_canonical_layout_pinned` above already had this
/// shape; this arm mirrors it onto the unpinned whole-content builder.
///
/// MUTATION THAT MUST MAKE THIS FAIL: in `aead.rs::aad_whole_content`, swap the
/// two pushes so `aad.extend_from_slice(plaintext_cid);` runs before
/// `aad.extend_from_slice(AEAD_WHOLE_CONTEXT);`. That mutation preserves the
/// total length exactly, so ONLY the positional segment assertions below catch
/// it.
#[test]
fn aad_whole_content_canonical_layout_pinned() {
    // A distinctive CID fill so a segment swap produces a readable failure
    // message (the positional asserts would catch an all-zero CID too).
    let plaintext_cid = [0xAA_u8; 32];

    let aad = aad_whole_content(&plaintext_cid);

    // Layout (2 segments): tag || plaintext_cid.
    let tag = b"benten-aead:whole:";
    let expected_len = tag.len() + plaintext_cid.len();
    assert_eq!(
        aad.len(),
        expected_len,
        "whole-content AAD layout regression — expected (tag || cid), total {} bytes; got {}",
        expected_len,
        aad.len()
    );

    // Segment 1: the domain-separation tag LEADS. A whole-content seal can never
    // be reinterpreted as a per-chunk or per-Recipe seal only because this
    // distinct prefix comes first — that property is positional, not just
    // present-somewhere.
    assert_eq!(
        &aad[..tag.len()],
        tag,
        "whole-content AAD MUST LEAD with the `benten-aead:whole:` domain-separation \
         tag — a swap to (cid || tag) preserves length and is invisible to every \
         seal/open round-trip (E-07)"
    );

    // Segment 2: the plaintext CID FOLLOWS (§1.A.FROZEN item 15(g)
    // AAD-binds-plaintext-CID rebinding defense).
    assert_eq!(
        &aad[tag.len()..],
        &plaintext_cid,
        "whole-content AAD MUST bind the plaintext CID immediately after the tag — \
         rebinding-attack defense per §1.A.FROZEN item 15(g)"
    );

    // would-FAIL guard (mirrors the per-chunk pin's `assert_ne!` arm): the
    // swapped concatenation is a DIFFERENT byte string. If this ever fires, the
    // two asserts above are not actually comparing bytes.
    let swapped: Vec<u8> = plaintext_cid
        .iter()
        .copied()
        .chain(tag.iter().copied())
        .collect();
    assert_ne!(
        aad, swapped,
        "the swapped-segment layout (cid || tag) MUST NOT equal the canonical layout (E-07)"
    );

    // Cross-arm separation: the whole-content prefix must not have converged
    // with its two siblings. If the three tags ever collided, a per-chunk seal
    // could be replayed as a whole-content seal at the same CID.
    assert_ne!(
        &aad[..tag.len()],
        b"benten-aead:chunk:",
        "the whole-content AAD tag MUST stay distinct from the per-chunk tag"
    );
}

/// E-08 closure — the per-Recipe AEAD AAD layout, which had NO golden.
///
/// `aad_per_recipe` binds `(plaintext_cid, recipe_index, total_recipes)` and
/// BOTH integers are `u32`. Swapping the two fields at `aead.rs:306-307` is
/// therefore length-preserving AND endianness-preserving: neither a size check
/// nor an anti-little-endian guard can see it. The falsification sweep made
/// exactly that swap and the crate stayed 210/210 + 14/14 PASS.
///
/// TWO MUTATIONS MUST MAKE THIS FAIL:
///   1. FIELD SWAP — in `aead.rs::aad_per_recipe`, emit
///      `total_recipes.to_be_bytes()` before `recipe_index.to_be_bytes()`.
///      Caught only because the two probe values below are DISTINCT; a fixture
///      using `(1, 1)` would sail straight through.
///   2. ENDIANNESS REVERT — `to_le_bytes()` in place of `to_be_bytes()`
///      (against the M-19 BE freeze). Caught by the `assert_ne!` arms.
#[test]
fn aad_per_recipe_canonical_layout_pinned() {
    // Fixture discipline: the two u32 probes MUST differ from each other (so a
    // field SWAP is observable) and each must have distinct BE/LE byte orders
    // (so an endianness revert is observable). `recipe_index < total_recipes`
    // keeps the fixture semantically honest.
    let plaintext_cid = [0xAA_u8; 32];
    let recipe_index: u32 = 0x0123_4567;
    let total_recipes: u32 = 0x89AB_CDEF;
    assert_ne!(
        recipe_index, total_recipes,
        "fixture precondition: the two u32 probes MUST differ or a field swap is invisible"
    );

    let aad = aad_per_recipe(&plaintext_cid, recipe_index, total_recipes);

    // Layout (4 segments): tag || plaintext_cid || recipe_index BE || total_recipes BE.
    let tag = b"benten-aead:recipe:";
    let expected_len = tag.len() + plaintext_cid.len() + 4 + 4;
    assert_eq!(
        aad.len(),
        expected_len,
        "per-Recipe AAD layout regression — expected (tag || cid || u32-BE recipe_index \
         || u32-BE total_recipes), total {} bytes; got {}",
        expected_len,
        aad.len()
    );

    // Segment 1: domain-separation tag, distinct from the whole-content and
    // per-chunk tags so a per-Recipe seal can never be reinterpreted as either.
    assert_eq!(
        &aad[..tag.len()],
        tag,
        "per-Recipe AAD domain-separation tag changed — wire-format break"
    );

    // Segment 2: plaintext CID.
    assert_eq!(
        &aad[tag.len()..tag.len() + plaintext_cid.len()],
        &plaintext_cid,
        "per-Recipe AAD plaintext_cid binding changed — wire-format break"
    );

    // Segment 3: recipe_index FIRST, big-endian (M-19).
    assert_eq!(
        &aad[aad.len() - 8..aad.len() - 4],
        &recipe_index.to_be_bytes(),
        "per-Recipe AAD MUST bind recipe_index FIRST as u32-BE — a swap with \
         total_recipes is length- and endianness-preserving and is caught ONLY here (E-08)"
    );

    // Segment 4: total_recipes SECOND, big-endian (M-19).
    assert_eq!(
        &aad[aad.len() - 4..],
        &total_recipes.to_be_bytes(),
        "per-Recipe AAD MUST bind total_recipes SECOND as u32-BE — the inter-Recipe \
         truncation defense commits to the seal-time count (E-08)"
    );

    // Explicit hex pin: with recipe_index = 0x01234567 and
    // total_recipes = 0x89ABCDEF the expected BIG-ENDIAN tail is
    //   [0x01, 0x23, 0x45, 0x67] || [0x89, 0xAB, 0xCD, 0xEF]
    let expected_recipe_index_bytes = [0x01_u8, 0x23, 0x45, 0x67];
    let expected_total_recipes_bytes = [0x89_u8, 0xAB, 0xCD, 0xEF];
    assert_eq!(
        &aad[aad.len() - 8..aad.len() - 4],
        &expected_recipe_index_bytes,
        "per-Recipe AAD u32-BE recipe_index encoding regression — must use to_be_bytes() (M-19)"
    );
    assert_eq!(
        &aad[aad.len() - 4..],
        &expected_total_recipes_bytes,
        "per-Recipe AAD u32-BE total_recipes encoding regression — must use to_be_bytes() (M-19)"
    );

    // would-FAIL guard 1 (endianness): neither integer is little-endian.
    assert_ne!(
        &aad[aad.len() - 8..aad.len() - 4],
        &recipe_index.to_le_bytes(),
        "per-Recipe AAD recipe_index MUST NOT be little-endian (M-19)"
    );
    assert_ne!(
        &aad[aad.len() - 4..],
        &total_recipes.to_le_bytes(),
        "per-Recipe AAD total_recipes MUST NOT be little-endian (M-19)"
    );

    // would-FAIL guard 2 (field swap): the recipe_index slot must not hold
    // total_recipes. This is the arm the falsification sweep's swap defeats
    // everywhere else — same width, same endianness, same length.
    assert_ne!(
        &aad[aad.len() - 8..aad.len() - 4],
        &total_recipes.to_be_bytes(),
        "per-Recipe AAD field ORDER regression — total_recipes is sitting in the \
         recipe_index slot (E-08: both fields are u32, so only distinct probe \
         values expose this)"
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
