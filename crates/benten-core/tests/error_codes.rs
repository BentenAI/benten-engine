//! `ErrorCode` enum + `CoreError::code()` mapping tests (C7, R2 landscape
//! §2.1 rows 16-17).
//!
//! Every stable ERROR-CATALOG code has a Rust variant; every `CoreError`
//! variant maps to its catalog code.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::{CoreError, ErrorCode};

#[test]
fn error_code_as_str_stable_for_inv_cycle() {
    assert_eq!(ErrorCode::InvCycle.as_str(), "E_INV_CYCLE");
}

#[test]
fn error_code_from_str_roundtrips_for_known_code() {
    let code = ErrorCode::from_str("E_INV_CYCLE");
    assert_eq!(code, ErrorCode::InvCycle);
}

#[test]
fn error_code_from_str_unknown_falls_back_to_unknown_variant() {
    let code = ErrorCode::from_str("E_NOT_A_REAL_CODE");
    match code {
        ErrorCode::Unknown(s) => assert_eq!(s, "E_NOT_A_REAL_CODE"),
        other => panic!("expected Unknown, got {other:?}"),
    }
}

#[test]
fn core_error_float_nan_maps_to_value_float_nan_code() {
    assert_eq!(CoreError::FloatNan.code(), ErrorCode::ValueFloatNan);
}

#[test]
fn core_error_float_nonfinite_maps_to_value_float_nonfinite_code() {
    assert_eq!(
        CoreError::FloatNonFinite.code(),
        ErrorCode::ValueFloatNonFinite
    );
}

#[test]
fn core_error_version_branched_maps_to_version_branched_code() {
    assert_eq!(
        CoreError::VersionBranched.code(),
        ErrorCode::VersionBranched
    );
}

#[test]
fn core_error_cid_unsupported_codec_maps_to_cid_unsupported_codec_code() {
    assert_eq!(
        CoreError::CidUnsupportedCodec.code(),
        ErrorCode::CidUnsupportedCodec
    );
}

#[test]
fn core_error_cid_unsupported_hash_maps_to_cid_unsupported_hash_code() {
    assert_eq!(
        CoreError::CidUnsupportedHash.code(),
        ErrorCode::CidUnsupportedHash
    );
}

/// R4 triage (m15) — R1 drift-detector finding: the catch-all
/// `ErrorCode::Unknown(String)` variant must preserve the original string,
/// not panic, not lossy-convert. Without this test a future refactor that
/// drops the payload (e.g. unifying to a single `Unknown` unit variant) would
/// silently lose the code identity.
#[test]
fn unknown_error_code_preserves_string_not_panic() {
    let arbitrary = "E_SOMETHING_WE_HAVE_NOT_SPECCED_YET";
    let code = ErrorCode::from_str(arbitrary);
    match &code {
        ErrorCode::Unknown(s) => assert_eq!(s, arbitrary, "payload must be preserved verbatim"),
        other => panic!("expected Unknown variant, got {other:?}"),
    }
    // And round-trip via as_str: an unknown code's `.as_str()` returns the
    // stored string so downstream printers can render it.
    assert_eq!(code.as_str(), arbitrary);
}
