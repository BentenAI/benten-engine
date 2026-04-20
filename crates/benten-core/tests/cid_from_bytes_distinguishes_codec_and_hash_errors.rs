//! Regression tests for the r6b error-arm split on [`Cid::from_bytes`].
//!
//! Before the split every failure returned `CoreError::InvalidCid(_)` which
//! maps to `E_CID_PARSE`, so the catalogued codes `E_CID_UNSUPPORTED_CODEC`
//! and `E_CID_UNSUPPORTED_HASH` had no firing site. The spec-to-code audit
//! (r6b §5.4) flagged this as a catalogued-but-unfired code pattern.
//!
//! These tests pin the new contract:
//!
//! - A well-formed header with the wrong multicodec byte → typed
//!   [`CoreError::CidUnsupportedCodec`] → catalog code
//!   `E_CID_UNSUPPORTED_CODEC`.
//! - A well-formed header with the wrong multihash byte → typed
//!   [`CoreError::CidUnsupportedHash`] → catalog code
//!   `E_CID_UNSUPPORTED_HASH`.
//! - Length / version / digest-length failures stay on
//!   [`CoreError::InvalidCid`] → catalog code `E_CID_PARSE` (no regression
//!   into the new typed arms).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{
    BLAKE3_DIGEST_LEN, CID_LEN, CID_V1, Cid, CoreError, MULTICODEC_DAG_CBOR, MULTIHASH_BLAKE3,
};
use benten_errors::ErrorCode;

fn well_formed_cid_bytes() -> [u8; CID_LEN] {
    let mut buf = [0u8; CID_LEN];
    buf[0] = CID_V1;
    buf[1] = MULTICODEC_DAG_CBOR;
    buf[2] = MULTIHASH_BLAKE3;
    buf[3] = BLAKE3_DIGEST_LEN;
    buf
}

/// Wrong multicodec byte must produce `CidUnsupportedCodec`, not the generic
/// `InvalidCid` parse variant. The multihash byte stays well-formed so the
/// validator definitely reaches the codec check before bailing.
#[test]
fn wrong_multicodec_fires_unsupported_codec() {
    // `raw` (0x55) — a real multicodec that is NOT dag-cbor.
    let mut buf = well_formed_cid_bytes();
    buf[1] = 0x55;
    let err = Cid::from_bytes(&buf).unwrap_err();
    match err {
        CoreError::CidUnsupportedCodec => {}
        other => panic!("expected CoreError::CidUnsupportedCodec, got {other:?}"),
    }
}

/// Wrong multihash byte must produce `CidUnsupportedHash`, not the generic
/// `InvalidCid` parse variant. The multicodec byte stays well-formed so the
/// validator definitely reaches the hash check before bailing.
#[test]
fn wrong_multihash_fires_unsupported_hash() {
    // `sha2-256` (0x12) — a real multihash that is NOT BLAKE3.
    let mut buf = well_formed_cid_bytes();
    buf[2] = 0x12;
    let err = Cid::from_bytes(&buf).unwrap_err();
    match err {
        CoreError::CidUnsupportedHash => {}
        other => panic!("expected CoreError::CidUnsupportedHash, got {other:?}"),
    }
}

/// Wrong codec must map to the `E_CID_UNSUPPORTED_CODEC` catalog code
/// (not `E_CID_PARSE`). Load-bearing for drift-detect and TS binding surface.
#[test]
fn wrong_multicodec_maps_to_unsupported_codec_catalog_code() {
    let mut buf = well_formed_cid_bytes();
    buf[1] = 0x55;
    let err = Cid::from_bytes(&buf).unwrap_err();
    assert_eq!(err.code(), ErrorCode::CidUnsupportedCodec);
}

/// Wrong hash must map to the `E_CID_UNSUPPORTED_HASH` catalog code (not
/// `E_CID_PARSE`). Load-bearing for drift-detect and TS binding surface.
#[test]
fn wrong_multihash_maps_to_unsupported_hash_catalog_code() {
    let mut buf = well_formed_cid_bytes();
    buf[2] = 0x12;
    let err = Cid::from_bytes(&buf).unwrap_err();
    assert_eq!(err.code(), ErrorCode::CidUnsupportedHash);
}

/// Negative control: the length / version / digest-length failure paths
/// must stay on `InvalidCid` → `E_CID_PARSE`. If a future refactor
/// accidentally folds them into the new typed arms, this test catches it.
#[test]
fn length_version_digestlen_errors_remain_cid_parse() {
    // Wrong length.
    let err = Cid::from_bytes(&[]).unwrap_err();
    assert!(matches!(err, CoreError::InvalidCid(_)));
    assert_eq!(err.code(), ErrorCode::CidParse);

    // Wrong version byte.
    let mut buf = well_formed_cid_bytes();
    buf[0] = 0x02;
    let err = Cid::from_bytes(&buf).unwrap_err();
    assert!(matches!(err, CoreError::InvalidCid(_)));
    assert_eq!(err.code(), ErrorCode::CidParse);

    // Wrong digest-length byte.
    let mut buf = well_formed_cid_bytes();
    buf[3] = 16;
    let err = Cid::from_bytes(&buf).unwrap_err();
    assert!(matches!(err, CoreError::InvalidCid(_)));
    assert_eq!(err.code(), ErrorCode::CidParse);
}
