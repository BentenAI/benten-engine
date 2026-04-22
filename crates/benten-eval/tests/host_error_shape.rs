//! R3 unit tests for G1-B (§9.2): `HostError` struct shape — FROZEN interface.
//!
//! Locked shape per plan §9.2:
//! ```rust
//! pub struct HostError {
//!     pub code: ErrorCode,
//!     pub source: Box<dyn std::error::Error + Send + Sync>,
//!     pub context: Option<String>,
//! }
//! ```
//! Plus typed-error round-trip for every new `E_HOST_*` variant.
//!
//! TDD red-phase: `HostError` does not yet exist in `benten_eval`. Tests will
//! fail to compile until G1-B lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.1).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::HostError;
use std::error::Error as StdError;

fn mock_source() -> Box<dyn StdError + Send + Sync> {
    Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "mock"))
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn host_error_shape_matches_spec() {
    // Constructing with all three fields named must compile; if the layout
    // drifts (renames, removals, reorders that change semantics), this stops
    // compiling.
    let err = HostError {
        code: ErrorCode::HostNotFound,
        source: mock_source(),
        context: Some("mock context".to_string()),
    };
    assert_eq!(err.code, ErrorCode::HostNotFound);
    assert_eq!(err.context.as_deref(), Some("mock context"));
    // Accessing .source via std::error::Error trait must still work.
    let _: &(dyn StdError + Send + Sync) = err.source.as_ref();
}

#[test]
fn host_error_implements_std_error() {
    let err = HostError {
        code: ErrorCode::HostNotFound,
        source: mock_source(),
        context: None,
    };
    // Conformance: HostError must implement std::error::Error. If it doesn't,
    // this coercion fails to compile.
    let _as_err: &dyn StdError = &err;
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn host_error_serde_round_trip_opaque_source() {
    // Wire format per sec-r1-6 / atk-6: code + optional context are serialised;
    // `source` is opaque and MUST NOT appear on the wire.
    let err = HostError {
        code: ErrorCode::HostNotFound,
        source: Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "PRIVATE-CID-bafy-leak",
        )),
        context: Some("public".to_string()),
    };
    let wire = err.to_wire_bytes().expect("wire encode");
    let decoded = HostError::from_wire_bytes(&wire).expect("wire decode");
    assert_eq!(decoded.code, ErrorCode::HostNotFound);
    assert_eq!(decoded.context.as_deref(), Some("public"));

    // Scan the raw wire bytes for the leak marker: must not appear.
    let as_str = String::from_utf8_lossy(&wire);
    assert!(
        !as_str.contains("PRIVATE-CID-bafy-leak"),
        "source must stay opaque on the wire (sec-r1-6 / atk-6)"
    );
}

// ---- Round-trip for every new E_HOST_* variant --------------------
//
// R4 tq-8: the "roundtrip" tests now genuinely round-trip — serialize
// through `to_wire_bytes`, decode through `from_wire_bytes`, and assert
// the code discriminant + context survives. Prior version was construction-
// only; it asserted the as_str value but never exercised the wire format.

fn assert_wire_round_trip(code: ErrorCode, literal: &str, context: Option<&str>) {
    let err = HostError {
        code: code.clone(),
        source: mock_source(),
        context: context.map(ToString::to_string),
    };
    assert_eq!(err.code.as_str(), literal);

    let wire = err.to_wire_bytes().expect("to_wire_bytes");
    let decoded = HostError::from_wire_bytes(&wire).expect("from_wire_bytes");
    assert_eq!(
        decoded.code, code,
        "wire round-trip must preserve ErrorCode for {literal}"
    );
    assert_eq!(
        decoded.context.as_deref(),
        context,
        "wire round-trip must preserve context (or None) for {literal}"
    );
}

#[test]
fn host_error_roundtrip_not_found() {
    assert_wire_round_trip(ErrorCode::HostNotFound, "E_HOST_NOT_FOUND", None);
    assert_wire_round_trip(
        ErrorCode::HostNotFound,
        "E_HOST_NOT_FOUND",
        Some("missing anchor"),
    );
}

#[test]
fn host_error_roundtrip_write_conflict() {
    assert_wire_round_trip(ErrorCode::HostWriteConflict, "E_HOST_WRITE_CONFLICT", None);
}

#[test]
fn host_error_roundtrip_backend_unavailable() {
    assert_wire_round_trip(
        ErrorCode::HostBackendUnavailable,
        "E_HOST_BACKEND_UNAVAILABLE",
        Some("redb is offline"),
    );
}

#[test]
fn host_error_roundtrip_capability_revoked() {
    assert_wire_round_trip(
        ErrorCode::HostCapabilityRevoked,
        "E_HOST_CAPABILITY_REVOKED",
        None,
    );
}

#[test]
fn host_error_roundtrip_capability_expired() {
    assert_wire_round_trip(
        ErrorCode::HostCapabilityExpired,
        "E_HOST_CAPABILITY_EXPIRED",
        Some("ttl=300s elapsed"),
    );
}
