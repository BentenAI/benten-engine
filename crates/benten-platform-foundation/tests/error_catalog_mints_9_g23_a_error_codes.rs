//! R3 Family D RED-PHASE pin for G23-A 9 NEW ErrorCode mint
//! (§3.5g atomic Rust+TS; companion-with-canary per doc-r1-1).
//!
//! Pin source: r2-test-landscape §2.4 ErrorCode list + plan §3 G23-A
//! "NEW ErrorCodes atomically minted Rust + TS per §3.5g".
//!
//! ## §3.5g cross-language rule-mirror
//!
//! Each ErrorCode at G23-A canary lands in:
//!   1. `benten-errors`: `ErrorCode` enum variant + `as_str` arm +
//!      `as_static_str` arm + `from_str` arm.
//!   2. `benten-errors`: variant listed in `ALL_CATALOG_VARIANTS` (used to
//!      derive `CATALOG_VARIANT_COUNT`); pinned at 127 after G23-A canary
//!      (118 + 9).
//!   3. `bindings/napi`: TS-side string-literal union mirror.
//!   4. `docs/ERROR-CATALOG.md`: companion-with-canary entry (NOT bundled at
//!      G26-A per doc-r1-1).
//!
//! This pin asserts shape #1 — every G23-A-minted ErrorCode round-trips
//! through `ErrorCode::from_str` to a NAMED variant (not `Unknown`).

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_errors::ErrorCode;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    9 G23-A ErrorCodes do not exist at HEAD; G23-A wires atomic Rust+TS mint per §3.5g. \
    Currently from_str returns ErrorCode::Unknown for all 9. Closes r2 §2.4 ErrorCode pin."]
fn error_catalog_mints_9_g23_a_error_codes() {
    for code in schema_fixtures::G23_A_ERROR_CODES {
        let parsed = ErrorCode::from_str(code);
        assert!(
            !matches!(parsed, ErrorCode::Unknown(_)),
            "ErrorCode {code} MUST be a named variant post-G23-A; \
             round-trip through from_str returned Unknown — §3.5g atomic mint missing"
        );
        // Round-trip: variant.as_static_str() == code.
        assert_eq!(
            parsed.as_static_str(),
            *code,
            "ErrorCode {code} must round-trip as_static_str → from_str"
        );
    }
}
