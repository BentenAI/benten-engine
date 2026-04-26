#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G12-E
// error-catalog additions (E_WAIT_TTL_EXPIRED + E_WAIT_TTL_INVALID).
//
//! Phase 2b R4-FP (B-1) — D12 WAIT TTL error-catalog drift detector.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D12 (new
//!     `E_WAIT_TTL_EXPIRED` + `E_WAIT_TTL_INVALID` error codes).
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.10 + §8.1 rows 525-526.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-06.
//!
//! Mirrors the R3-B `sandbox_codes_present.rs` + R3-A `stream_codes_present.rs`
//! drift-detector pattern: assert the new codes round-trip through
//! `as_str` / `from_str`, and assert no deprecated alias survives.
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;

/// `e_wait_ttl_expired_present_in_catalog` — D12 + R2 row 525.
///
/// `E_WAIT_TTL_EXPIRED` MUST exist as a stable variant in `ErrorCode`
/// and round-trip through `from_str` / `as_str`.
#[test]
#[ignore = "Phase 2b G12-E pending — depends on ErrorCode::WaitTtlExpired variant"]
fn e_wait_ttl_expired_present_in_catalog() {
    let code = ErrorCode::WaitTtlExpired;
    assert_eq!(code.as_str(), "E_WAIT_TTL_EXPIRED");
    assert_eq!(
        ErrorCode::from_str("E_WAIT_TTL_EXPIRED"),
        ErrorCode::WaitTtlExpired
    );
}

/// `e_wait_ttl_invalid_present_in_catalog` — D12 + R2 row 526.
///
/// `E_WAIT_TTL_INVALID` MUST exist as a stable variant in `ErrorCode`
/// and round-trip. Fired by the registration-time validation when
/// `ttl_hours: 0` or `ttl_hours > 720`.
#[test]
#[ignore = "Phase 2b G12-E pending — depends on ErrorCode::WaitTtlInvalid variant"]
fn e_wait_ttl_invalid_present_in_catalog() {
    let code = ErrorCode::WaitTtlInvalid;
    assert_eq!(code.as_str(), "E_WAIT_TTL_INVALID");
    assert_eq!(
        ErrorCode::from_str("E_WAIT_TTL_INVALID"),
        ErrorCode::WaitTtlInvalid
    );
}

/// Anti-rename guard: assert no plausible deprecated alias for the new
/// variants leaks back into the catalog. CLAUDE.md non-negotiable
/// rule #5: no deprecated aliases or backward-compat shims.
#[test]
#[ignore = "Phase 2b G12-E pending — anti-rename guard"]
fn wait_ttl_no_deprecated_aliases() {
    // None of these should round-trip to the new variants — they MUST
    // collapse to ErrorCode::Unknown(_).
    for alias in &[
        "E_WAIT_EXPIRED",
        "E_WAIT_TIMEOUT",
        "E_WAIT_TTL_TIMEOUT",
        "E_WAIT_DEADLINE_EXCEEDED",
    ] {
        let parsed = ErrorCode::from_str(alias);
        assert!(
            !matches!(parsed, ErrorCode::WaitTtlExpired),
            "{alias} MUST NOT alias E_WAIT_TTL_EXPIRED (no deprecated \
             aliases per CLAUDE.md rule #5)"
        );
    }
    for alias in &[
        "E_WAIT_TTL_BAD",
        "E_WAIT_TTL_OUT_OF_RANGE",
        "E_WAIT_INVALID_TTL",
    ] {
        let parsed = ErrorCode::from_str(alias);
        assert!(
            !matches!(parsed, ErrorCode::WaitTtlInvalid),
            "{alias} MUST NOT alias E_WAIT_TTL_INVALID"
        );
    }
}
