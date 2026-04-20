//! `cap_error_codes_match_catalog` — every `CapError` variant maps to the
//! right ERROR-CATALOG stable code (P5, C7 — R2 landscape §2.4 row 5).
//!
//! R3 writer: `rust-test-writer-unit`.
//! Codes fired: `E_CAP_DENIED`, `E_CAP_DENIED_READ`, `E_CAP_REVOKED_MID_EVAL`,
//! `E_CAP_NOT_IMPLEMENTED`, `E_CAP_ATTENUATION`.

#![allow(clippy::unwrap_used)]

use benten_caps::CapError;
use benten_errors::ErrorCode;

#[test]
fn cap_error_codes_match_catalog() {
    // Note (G4 mini-review g4-cr-5): `Denied` was consolidated from a
    // unit variant + `DeniedDetail` struct variant into a single struct
    // variant with `required` / `entity` strings (both empty here, since
    // this test cares about the code mapping, not the payload).
    assert_eq!(
        CapError::Denied {
            required: String::new(),
            entity: String::new()
        }
        .code(),
        ErrorCode::CapDenied
    );
    assert_eq!(
        CapError::DeniedRead {
            required: String::new(),
            entity: String::new()
        }
        .code(),
        ErrorCode::CapDeniedRead
    );
    assert_eq!(
        CapError::RevokedMidEval.code(),
        ErrorCode::CapRevokedMidEval
    );
    // `NotImplemented` now carries `backend` + `lands_in_phase` fields
    // (g4-cr-6). The code mapping is unchanged.
    assert_eq!(
        CapError::NotImplemented {
            backend: "UCANBackend",
            lands_in_phase: 3
        }
        .code(),
        ErrorCode::CapNotImplemented
    );
    assert_eq!(CapError::Attenuation.code(), ErrorCode::CapAttenuation);
}

#[test]
fn cap_error_display_messages_are_nonempty() {
    use std::error::Error;
    let errs: [CapError; 5] = [
        CapError::Denied {
            required: String::new(),
            entity: String::new(),
        },
        CapError::DeniedRead {
            required: String::new(),
            entity: String::new(),
        },
        CapError::RevokedMidEval,
        CapError::NotImplemented {
            backend: "UCANBackend",
            lands_in_phase: 3,
        },
        CapError::Attenuation,
    ];
    for e in &errs {
        let msg = e.to_string();
        assert!(!msg.is_empty(), "Display for {e:?} must be nonempty");
        // No chained source in Phase 1 stubs — all variants are leaf errors.
        assert!(e.source().is_none());
    }
}
