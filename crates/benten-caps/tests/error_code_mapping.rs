//! `cap_error_codes_match_catalog` — every `CapError` variant maps to the
//! right ERROR-CATALOG stable code (P5, C7 — R2 landscape §2.4 row 5).
//!
//! R3 writer: `rust-test-writer-unit`.
//! Codes fired: `E_CAP_DENIED`, `E_CAP_DENIED_READ`, `E_CAP_REVOKED_MID_EVAL`,
//! `E_CAP_NOT_IMPLEMENTED`, `E_CAP_ATTENUATION`.

#![allow(clippy::unwrap_used)]

use benten_caps::CapError;
use benten_core::ErrorCode;

#[test]
fn cap_error_codes_match_catalog() {
    assert_eq!(CapError::Denied.code(), ErrorCode::CapDenied);
    assert_eq!(CapError::DeniedRead.code(), ErrorCode::CapDeniedRead);
    assert_eq!(
        CapError::RevokedMidEval.code(),
        ErrorCode::CapRevokedMidEval
    );
    assert_eq!(
        CapError::NotImplemented.code(),
        ErrorCode::CapNotImplemented
    );
    assert_eq!(CapError::Attenuation.code(), ErrorCode::CapAttenuation);
}

#[test]
fn cap_error_display_messages_are_nonempty() {
    use std::error::Error;
    let errs: [CapError; 5] = [
        CapError::Denied,
        CapError::DeniedRead,
        CapError::RevokedMidEval,
        CapError::NotImplemented,
        CapError::Attenuation,
    ];
    for e in &errs {
        let msg = e.to_string();
        assert!(!msg.is_empty(), "Display for {e:?} must be nonempty");
        // No chained source in Phase 1 stubs — all variants are leaf errors.
        assert!(e.source().is_none());
    }
}
