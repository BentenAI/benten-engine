//! G23-B subset-closure pin — every `E_MATERIALIZER_*` ErrorCode variant
//! that exists in the `benten-errors` enum at post-G23-B HEAD must be
//! present in the `G23_B_ERROR_CODES` registry at
//! `crates/benten-platform-foundation/tests/common/materializer_fixtures.rs`.
//!
//! Per §3.5g cross-language rule-mirror: this is the **inverse-direction**
//! pin that catches the failure mode "minted enum variant but forgot to
//! add to the test-side registry array used by atomic-mint round-trip
//! pins". Without subset-closure, a new E_MATERIALIZER_NEW_THING variant
//! could land in benten-errors without being asserted by any TS-mirror /
//! catalog-md pin downstream.
//!
//! ## Closure shape
//!
//! Authoritative expected set (frozen at G23-B canary; matches
//! `materializer_fixtures::G23_B_ERROR_CODES` exactly). Each expected
//! code:
//!   - resolves via `from_str` to a NAMED variant (catches enum hole)
//!   - round-trips via `as_static_str` (catches as_str arm gap)
//!   - has prefix `E_MATERIALIZER_` (catches family-naming drift)
//!
//! Per Ben's R4-triage §7 ratification (2026-05-11): TS mirror canonical
//! location is `packages/engine/src/errors.generated.ts`. CATALOG_VARIANT_COUNT
//! 127 → 130 at G23-B canary close.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;

/// 3 G23-B ErrorCode string forms — mirror of
/// `materializer_fixtures::G23_B_ERROR_CODES` at
/// `crates/benten-platform-foundation/tests/common/materializer_fixtures.rs`.
///
/// Frozen here to catch the failure mode where a NEW
/// `E_MATERIALIZER_*` variant lands in `benten-errors` but the
/// downstream registry array is not updated to match.
///
/// RED-PHASE: at HEAD these don't exist in the enum. Un-ignore at
/// G23-B wave-5.
const EXPECTED_G23_B_CODES: &[&str] = &[
    "E_MATERIALIZER_CAP_DENIED",
    "E_MATERIALIZER_SCHEMA_MISMATCH",
    "E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE",
];

#[test]
fn every_expected_g23_b_materializer_code_resolves_to_named_variant() {
    for code in EXPECTED_G23_B_CODES {
        // Family-prefix discipline: every code in the G23-B set must
        // share the E_MATERIALIZER_ prefix.
        assert!(
            code.starts_with("E_MATERIALIZER_"),
            "G23-B subset-closure: expected code {code} does not start \
             with E_MATERIALIZER_ — family-naming discipline broken"
        );
        // Dynamic half: from_str round-trip.
        let parsed = ErrorCode::from_str(code);
        assert!(
            !matches!(parsed, ErrorCode::Unknown(_)),
            "G23-B subset-closure: ErrorCode {code} expected in enum \
             post-G23-B but from_str returned Unknown — enum + as_str \
             + from_str arms missing for this variant"
        );
        assert_eq!(
            parsed.as_static_str(),
            *code,
            "G23-B subset-closure: ErrorCode {code} must round-trip \
             through as_static_str → from_str without lossy conversion"
        );
    }
}
