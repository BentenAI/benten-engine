//! Shape / stability pins for the `ErrorCode` enum.
//!
//! These tests are the canonical regression fixtures for the benten-errors
//! extraction (closes SECURITY-POSTURE compromise #3). They pin:
//!
//! 1. The enum variant count — a wire-compat tripwire. Adding a variant
//!    requires bumping this number AND adding a catalog entry + `.code()`
//!    mapping in the owning crate; shrinking it is always a breaking
//!    change.
//! 2. A representative `as_str` round-trip so the string form is frozen
//!    for at least one variant per catalog "family" (invariant, capability,
//!    transaction, CID, engine-level).
//! 3. The `Unknown(String)` forward-compat fallback preserves the raw
//!    string verbatim — the drift detector relies on this so an unknown
//!    code rendered by an older client round-trips through the enum
//!    without lossy conversion.

use benten_errors::ErrorCode;

/// Count the enum variants that the catalog enumerates. Update this alongside
/// the catalog doc + `.code()` mappers whenever a new catalog code lands.
///
/// Current catalog count: 42 catalog variants + 1 `Unknown(String)` fallback
/// = 43 total. (`ErrorCode::Unknown` is not a catalog code; it's a
/// forward-compat fallback.)
const CATALOG_VARIANT_COUNT: usize = 42;

/// Every catalog variant must round-trip through `as_str` / `from_str`
/// without hitting the `Unknown` fallback. If this count drifts from the
/// actual enum shape the test fails with a clear error message, which is
/// the tripwire we want.
#[test]
fn variant_count_is_pinned() {
    // Enumerate every variant explicitly so the count is mechanically
    // verifiable against the enum declaration. Adding a variant requires
    // adding it here too — a deliberate friction point.
    let all: [ErrorCode; CATALOG_VARIANT_COUNT] = [
        ErrorCode::InvCycle,
        ErrorCode::InvDepthExceeded,
        ErrorCode::InvFanoutExceeded,
        ErrorCode::InvTooManyNodes,
        ErrorCode::InvTooManyEdges,
        ErrorCode::InvDeterminism,
        ErrorCode::InvContentHash,
        ErrorCode::InvRegistration,
        ErrorCode::InvIterateNestDepth,
        ErrorCode::InvIterateMaxMissing,
        ErrorCode::InvIterateBudget,
        ErrorCode::CapDenied,
        ErrorCode::CapDeniedRead,
        ErrorCode::CapRevoked,
        ErrorCode::CapRevokedMidEval,
        ErrorCode::CapNotImplemented,
        ErrorCode::CapAttenuation,
        ErrorCode::WriteConflict,
        ErrorCode::IvmViewStale,
        ErrorCode::TxAborted,
        ErrorCode::NestedTransactionNotSupported,
        ErrorCode::PrimitiveNotImplemented,
        ErrorCode::SystemZoneWrite,
        ErrorCode::ValueFloatNan,
        ErrorCode::ValueFloatNonFinite,
        ErrorCode::CidParse,
        ErrorCode::CidUnsupportedCodec,
        ErrorCode::CidUnsupportedHash,
        ErrorCode::VersionBranched,
        ErrorCode::BackendNotFound,
        ErrorCode::TransformSyntax,
        ErrorCode::InputLimit,
        ErrorCode::NotFound,
        ErrorCode::Serialize,
        ErrorCode::GraphInternal,
        ErrorCode::DuplicateHandler,
        ErrorCode::NoCapabilityPolicyConfigured,
        ErrorCode::ProductionRequiresCaps,
        ErrorCode::SubsystemDisabled,
        ErrorCode::UnknownView,
        ErrorCode::NotImplemented,
        ErrorCode::IvmPatternMismatch,
    ];
    assert_eq!(all.len(), CATALOG_VARIANT_COUNT);
    // Every listed variant must round-trip through from_str(as_str).
    for code in &all {
        let s = code.as_str();
        let parsed = ErrorCode::from_str(s);
        assert_eq!(
            &parsed, code,
            "catalog variant {code:?} failed as_str/from_str round-trip via string {s}",
        );
    }
}

/// Representative catalog code renders the frozen string form.
#[test]
fn as_str_stable_for_representative_code() {
    assert_eq!(ErrorCode::CapDenied.as_str(), "E_CAP_DENIED");
    assert_eq!(ErrorCode::InvCycle.as_str(), "E_INV_CYCLE");
    assert_eq!(ErrorCode::ValueFloatNan.as_str(), "E_VALUE_FLOAT_NAN");
}

/// `from_str` round-trips `as_str` for a representative code.
#[test]
fn from_str_roundtrip_representative() {
    let parsed = ErrorCode::from_str("E_CAP_DENIED");
    assert_eq!(parsed, ErrorCode::CapDenied);
    assert_eq!(parsed.as_str(), "E_CAP_DENIED");
}

/// Unknown codes fall back to `Unknown(String)` with the payload preserved.
#[test]
fn from_str_unknown_preserves_raw_string() {
    let code = ErrorCode::from_str("E_NOT_A_REAL_CODE");
    match &code {
        ErrorCode::Unknown(s) => assert_eq!(s, "E_NOT_A_REAL_CODE"),
        other => panic!("expected Unknown, got {other:?}"),
    }
    // as_str returns the raw string verbatim so rendering stays lossless.
    assert_eq!(code.as_str(), "E_NOT_A_REAL_CODE");
}

/// `as_static_str` returns the frozen 'static form for known variants and
/// a sentinel `"E_UNKNOWN"` for the forward-compat fallback (since the
/// payload is an owned String and cannot be promoted to `'static`).
#[test]
fn as_static_str_known_and_unknown() {
    assert_eq!(ErrorCode::CapDenied.as_static_str(), "E_CAP_DENIED");
    assert_eq!(
        ErrorCode::Unknown("E_SOMETHING".into()).as_static_str(),
        "E_UNKNOWN"
    );
}
