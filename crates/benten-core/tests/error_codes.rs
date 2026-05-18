//! `ErrorCode` enum + `CoreError::code()` mapping tests (C7, R2 landscape
//! §2.1 rows 16-17).
//!
//! Every stable ERROR-CATALOG code has a Rust variant; every `CoreError`
//! variant maps to its catalog code.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use std::str::FromStr;

use benten_core::CoreError;
use benten_errors::ErrorCode;

#[test]
fn error_code_as_str_stable_for_inv_cycle() {
    assert_eq!(ErrorCode::InvCycle.as_str(), "E_INV_CYCLE");
}

#[test]
fn error_code_from_str_roundtrips_for_known_code() {
    let code = ErrorCode::from_str("E_INV_CYCLE").expect("recognized code");
    assert_eq!(code, ErrorCode::InvCycle);
}

#[test]
fn error_code_from_str_unknown_is_err_and_preserves_raw_string() {
    // #733: fallible by design — an unrecognized code is a parse error,
    // not a lossy `Unknown`. The raw string is preserved on the error so
    // forward-compat callers can recover `Unknown` explicitly.
    let err = ErrorCode::from_str("E_NOT_A_REAL_CODE")
        .expect_err("unrecognized code must be a parse error");
    assert_eq!(err.as_str(), "E_NOT_A_REAL_CODE");
    let recovered = ErrorCode::from_str("E_NOT_A_REAL_CODE")
        .unwrap_or_else(|e| ErrorCode::Unknown(e.into_inner()));
    assert!(matches!(recovered, ErrorCode::Unknown(s) if s == "E_NOT_A_REAL_CODE"));
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

/// R4 triage (m15) — R1 drift-detector finding: an unrecognized code must
/// preserve the original string, not panic, not lossy-convert. Post-#733
/// the parse is fallible: the raw string is preserved on the parse error,
/// and the forward-compat `Unknown` recovery still renders it verbatim.
#[test]
fn unknown_error_code_preserves_string_not_panic() {
    let arbitrary = "E_SOMETHING_WE_HAVE_NOT_SPECCED_YET";
    let err = ErrorCode::from_str(arbitrary).expect_err("unrecognized code is a parse error");
    assert_eq!(
        err.as_str(),
        arbitrary,
        "payload must be preserved verbatim"
    );
    // And round-trip via the forward-compat `Unknown` recovery: `.as_str()`
    // returns the stored string so downstream printers can render it.
    let code =
        ErrorCode::from_str(arbitrary).unwrap_or_else(|e| ErrorCode::Unknown(e.into_inner()));
    assert_eq!(code.as_str(), arbitrary);
}
