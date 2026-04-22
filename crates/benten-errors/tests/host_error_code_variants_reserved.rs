//! R3 unit tests: 5 new `HostError` ErrorCode variants reserved in Phase 2a.
//!
//! FROZEN interface (r1-triage §SHAPE-PIN). Each variant is catalog-reserved
//! here; firing sites land in Phase 3. Round-trip `as_str` / `from_str` must
//! already work so drift-detect CI stays green workspace-wide.
//!
//! TDD red-phase: the 5 variants do not yet exist on `ErrorCode`. Tests will
//! fail to compile until G1-B lands them.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.2).

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn host_not_found_roundtrips() {
    let code = ErrorCode::HostNotFound;
    assert_eq!(code.as_str(), "E_HOST_NOT_FOUND");
    assert_eq!(
        ErrorCode::from_str("E_HOST_NOT_FOUND"),
        ErrorCode::HostNotFound
    );
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn host_write_conflict_roundtrips() {
    let code = ErrorCode::HostWriteConflict;
    assert_eq!(code.as_str(), "E_HOST_WRITE_CONFLICT");
    assert_eq!(
        ErrorCode::from_str("E_HOST_WRITE_CONFLICT"),
        ErrorCode::HostWriteConflict
    );
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn host_backend_unavailable_roundtrips() {
    let code = ErrorCode::HostBackendUnavailable;
    assert_eq!(code.as_str(), "E_HOST_BACKEND_UNAVAILABLE");
    assert_eq!(
        ErrorCode::from_str("E_HOST_BACKEND_UNAVAILABLE"),
        ErrorCode::HostBackendUnavailable
    );
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn host_capability_revoked_roundtrips() {
    let code = ErrorCode::HostCapabilityRevoked;
    assert_eq!(code.as_str(), "E_HOST_CAPABILITY_REVOKED");
    assert_eq!(
        ErrorCode::from_str("E_HOST_CAPABILITY_REVOKED"),
        ErrorCode::HostCapabilityRevoked
    );
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn host_capability_expired_roundtrips() {
    let code = ErrorCode::HostCapabilityExpired;
    assert_eq!(code.as_str(), "E_HOST_CAPABILITY_EXPIRED");
    assert_eq!(
        ErrorCode::from_str("E_HOST_CAPABILITY_EXPIRED"),
        ErrorCode::HostCapabilityExpired
    );
}
