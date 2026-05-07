// Phase-3 G20-A2 (D12 wave-8a) — D12 WAIT TTL error-catalog drift detector.
//
// `phase_2b_landed` cfg gate retired at G20-A2 wave-8a — the
// `WaitTtlExpired` + `WaitTtlInvalid` variants now exist as production
// catalog entries.
//
// Mirrors the R3-B `sandbox_codes_present.rs` + R3-A `stream_codes_present.rs`
// drift-detector pattern: assert the new codes round-trip through
// `as_str` / `from_str`, and assert no deprecated alias survives.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;

/// `e_wait_ttl_expired_present_in_catalog` — D12 + R2 row 525.
#[test]
fn e_wait_ttl_expired_present_in_catalog() {
    let code = ErrorCode::WaitTtlExpired;
    assert_eq!(code.as_str(), "E_WAIT_TTL_EXPIRED");
    assert_eq!(
        ErrorCode::from_str("E_WAIT_TTL_EXPIRED"),
        ErrorCode::WaitTtlExpired
    );
}

/// `e_wait_ttl_invalid_present_in_catalog` — D12 + R2 row 526.
#[test]
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
fn wait_ttl_no_deprecated_aliases() {
    // None of these should round-trip to the new variants — they MUST
    // collapse to a different (or unknown) variant.
    for alias in &[
        "E_WAIT_EXPIRED",
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
    // E_WAIT_TIMEOUT is a distinct existing variant; it MUST parse to
    // ErrorCode::WaitTimeout (NOT to WaitTtlExpired).
    assert_eq!(
        ErrorCode::from_str("E_WAIT_TIMEOUT"),
        ErrorCode::WaitTimeout
    );
    assert!(
        !matches!(
            ErrorCode::from_str("E_WAIT_TIMEOUT"),
            ErrorCode::WaitTtlExpired
        ),
        "E_WAIT_TIMEOUT MUST NOT alias E_WAIT_TTL_EXPIRED — distinct semantics"
    );

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
