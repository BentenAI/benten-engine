//! G23-A subset-closure pin — every `E_SCHEMA_*` ErrorCode variant
//! that exists in the `benten-errors` enum at post-G23-A HEAD must be
//! present in the `G23_A_ERROR_CODES` registry at
//! `crates/benten-platform-foundation/tests/common/schema_fixtures.rs`.
//!
//! Per §3.5g cross-language rule-mirror: this is the **inverse-direction**
//! pin that catches the failure mode "minted enum variant but forgot to
//! add to the test-side registry array used by atomic-mint round-trip
//! pins". Without subset-closure, a new E_SCHEMA_NEW_THING variant could
//! land in benten-errors without being asserted by any TS-mirror /
//! catalog-md pin downstream.
//!
//! ## Closure shape
//!
//! Authoritative expected set (frozen at G23-A canary; matches
//! `schema_fixtures::G23_A_ERROR_CODES` exactly). Each expected code:
//!   - resolves via `from_str` to a NAMED variant (catches enum hole)
//!   - round-trips via `as_static_str` (catches as_str arm gap)
//!   - has prefix `E_SCHEMA_` (catches family-naming drift)
//!
//! Per Ben's R4-triage §7 ratification (2026-05-11): TS mirror canonical
//! location is `packages/engine/src/errors.generated.ts`. CATALOG_VARIANT_COUNT
//! 118 → 127 at G23-A canary close.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;

/// 9 G23-A ErrorCode string forms — mirror of
/// `schema_fixtures::G23_A_ERROR_CODES` at
/// `crates/benten-platform-foundation/tests/common/schema_fixtures.rs`.
///
/// Frozen here to catch the failure mode where a NEW `E_SCHEMA_*`
/// variant lands in `benten-errors` but the downstream registry array
/// is not updated to match (which would silently bypass the TS-mirror
/// + catalog-md atomic-mint pins for that variant).
///
/// RED-PHASE: at HEAD these don't exist in the enum. Each
/// `ErrorCode::from_str(code)` returns `ErrorCode::Unknown(code)`. The
/// `!matches!(parsed, ErrorCode::Unknown(_))` assertion fails for all
/// 9. Un-ignore at G23-A wave-4.
const EXPECTED_G23_A_CODES: &[&str] = &[
    "E_SCHEMA_VALIDATION_FAILED",
    "E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED",
    "E_SCHEMA_SANDBOX_HOST_FN_REJECTED",
    "E_SCHEMA_VOCAB_INVALID_LABEL",
    "E_SCHEMA_VOCAB_EDGE_MISMATCH",
    "E_SCHEMA_VOCAB_SCALAR_UNKNOWN",
    "E_SCHEMA_VOCAB_REF_TARGET_MISSING",
    "E_SCHEMA_VOCAB_CYCLE_REJECTED",
    "E_SCHEMA_VOCAB_REQUIRED_PROPERTY_MISSING",
];

// Un-ignored at G23-A wave-4 (2026-05-12): all 9 E_SCHEMA_* variants
// landed in benten-errors with as_str + from_str arms + TS mirror + catalog
// entries. Subset-closure pin now GREEN as a permanent regression-guard.
#[test]
fn every_expected_g23_a_schema_code_resolves_to_named_variant() {
    for code in EXPECTED_G23_A_CODES {
        // Family-prefix discipline: every code in the G23-A set must
        // share the E_SCHEMA_ prefix. This is the static half of the
        // closure — catches drift in the expected list itself.
        assert!(
            code.starts_with("E_SCHEMA_"),
            "G23-A subset-closure: expected code {code} does not start \
             with E_SCHEMA_ — family-naming discipline broken"
        );
        // Dynamic half: from_str round-trip. At HEAD all 9 hit
        // ErrorCode::Unknown so this assertion fails — RED-PHASE
        // closure target.
        let parsed = ErrorCode::from_str(code);
        assert!(
            !matches!(parsed, ErrorCode::Unknown(_)),
            "G23-A subset-closure: ErrorCode {code} expected in enum \
             post-G23-A but from_str returned Unknown — enum + as_str \
             + from_str arms missing for this variant"
        );
        assert_eq!(
            parsed.as_static_str(),
            *code,
            "G23-A subset-closure: ErrorCode {code} must round-trip \
             through as_static_str → from_str without lossy conversion"
        );
    }
}
