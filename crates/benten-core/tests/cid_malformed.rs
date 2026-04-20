//! Edge-case tests for malformed CID byte inputs.
//!
//! Covers error codes:
//! - `E_CID_PARSE` — structurally invalid CID bytes (length, version)
//! - `E_CID_UNSUPPORTED_CODEC` — CID uses a multicodec other than `dag-cbor` (0x71)
//! - `E_CID_UNSUPPORTED_HASH` — CID uses a multihash other than BLAKE3 (0x1e)
//!
//! These tests honestly say "no" to boundary inputs. Adversarial malformation
//! (fuzzing, DoS-shaped CIDs) is owned by rust-test-writer-security.
//!
//! R3 contract: these tests fail today because the Phase-1 error-code enum
//! (`benten_errors::ErrorCode`) does not yet exist and `Cid::from_bytes` currently
//! returns a free-form `&'static str`. R5 lands the `ErrorCode` surface and
//! wires each parse failure to the canonical code. The assertions below pin
//! the mapping so R5 cannot drift.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{
    BLAKE3_DIGEST_LEN, CID_LEN, CID_V1, Cid, CoreError, MULTICODEC_DAG_CBOR, MULTIHASH_BLAKE3,
};

/// Build a syntactically well-formed Cid byte buffer with override slots so
/// individual tests can corrupt exactly one field.
fn well_formed_cid_bytes() -> [u8; CID_LEN] {
    let mut buf = [0u8; CID_LEN];
    buf[0] = CID_V1;
    buf[1] = MULTICODEC_DAG_CBOR;
    buf[2] = MULTIHASH_BLAKE3;
    buf[3] = BLAKE3_DIGEST_LEN;
    // digest bytes are zero; content doesn't matter for header validation
    buf
}

/// Helper: a structural parse failure must be `CoreError::InvalidCid(_)`
/// (maps to `E_CID_PARSE`).
fn assert_core_error_is_invalid_cid(err: &CoreError) {
    match err {
        CoreError::InvalidCid(_) => {}
        other => panic!("expected CoreError::InvalidCid, got {other:?}"),
    }
}

/// Helper: a catalogued-but-typed parse failure — any of the three
/// `from_bytes` failure classes is acceptable for inputs where multiple
/// bytes are corrupted at once (the checker short-circuits on the first
/// mismatch).
fn assert_core_error_is_cid_class(err: &CoreError) {
    match err {
        CoreError::InvalidCid(_)
        | CoreError::CidUnsupportedCodec
        | CoreError::CidUnsupportedHash => {}
        other => panic!(
            "expected CoreError::{{InvalidCid, CidUnsupportedCodec, CidUnsupportedHash}}, \
             got {other:?}"
        ),
    }
}

#[test]
fn cid_parse_errors() {
    // E_CID_PARSE — empty input
    let err = Cid::from_bytes(&[]).unwrap_err();
    assert_core_error_is_invalid_cid(&err);

    // E_CID_PARSE — truncated (header-only, no digest)
    let err = Cid::from_bytes(&[
        CID_V1,
        MULTICODEC_DAG_CBOR,
        MULTIHASH_BLAKE3,
        BLAKE3_DIGEST_LEN,
    ])
    .unwrap_err();
    assert_core_error_is_invalid_cid(&err);

    // E_CID_PARSE — oversized (legitimate CID with one trailing byte)
    let mut oversized = well_formed_cid_bytes().to_vec();
    oversized.push(0x00);
    let err = Cid::from_bytes(&oversized).unwrap_err();
    assert_core_error_is_invalid_cid(&err);

    // E_CID_PARSE — wrong CID version byte (CIDv0 is a raw multihash,
    // Benten only accepts CIDv1).
    let mut buf = well_formed_cid_bytes();
    buf[0] = 0x12; // CIDv0 starts with a raw multihash, which is not 0x01
    let err = Cid::from_bytes(&buf).unwrap_err();
    assert_core_error_is_invalid_cid(&err);

    // E_CID_PARSE — digest-length byte advertises a different length
    // from the wire-format fixed length. (Benten's CIDs are always
    // BLAKE3-256, so any digest length other than 32 must error.)
    let mut buf = well_formed_cid_bytes();
    buf[3] = 16; // lies: says 16-byte digest while the buffer is sized for 32
    let err = Cid::from_bytes(&buf).unwrap_err();
    assert_core_error_is_invalid_cid(&err);
}

#[test]
fn cid_unsupported_codec() {
    // E_CID_UNSUPPORTED_CODEC — replace `dag-cbor` (0x71) with `raw` (0x55)
    // or `dag-json` (0x0129 truncated). Both must be rejected because Benten
    // CIDs are dag-cbor-only in Phase 1. Post r6b the error is typed
    // `CidUnsupportedCodec` rather than folding to `InvalidCid`.
    for rogue_codec in [0x55, 0x70, 0x00, 0xff] {
        let mut buf = well_formed_cid_bytes();
        buf[1] = rogue_codec;
        let err = Cid::from_bytes(&buf).unwrap_err();
        match err {
            CoreError::CidUnsupportedCodec => {}
            other => panic!("expected CoreError::CidUnsupportedCodec, got {other:?}"),
        }
    }
}

#[test]
fn cid_unsupported_hash() {
    // E_CID_UNSUPPORTED_HASH — replace BLAKE3 (0x1e) with SHA-256 (0x12),
    // SHA-512 (0x13), or Keccak-256 (0x1b). All must be rejected —
    // Benten CIDs are BLAKE3-only. Post r6b the error is typed
    // `CidUnsupportedHash` rather than folding to `InvalidCid`.
    for rogue_hash in [0x12, 0x13, 0x1b, 0x00, 0xff] {
        let mut buf = well_formed_cid_bytes();
        buf[2] = rogue_hash;
        let err = Cid::from_bytes(&buf).unwrap_err();
        match err {
            CoreError::CidUnsupportedHash => {}
            other => panic!("expected CoreError::CidUnsupportedHash, got {other:?}"),
        }
    }
}

#[test]
fn cid_parse_rejects_noncontiguous_corruption() {
    // Belt-and-suspenders: corrupt *every* header byte position. None of
    // these produce a valid Benten CID, and none may panic. Accepts any of
    // the three typed failure classes (r6b split: version/length corruption
    // still hits `InvalidCid`; multicodec corruption hits
    // `CidUnsupportedCodec`; multihash corruption hits `CidUnsupportedHash`).
    for idx in 0..4usize {
        let mut buf = well_formed_cid_bytes();
        buf[idx] = buf[idx].wrapping_add(1);
        let err = Cid::from_bytes(&buf).unwrap_err();
        assert_core_error_is_cid_class(&err);
    }
}
