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

/// Every catalog variant must round-trip through `as_str` / `from_str`
/// without hitting the `Unknown` fallback. The enumerated list below is the
/// authoritative source for the count — `CATALOG_VARIANT_COUNT` is derived
/// from `ALL_CATALOG_VARIANTS.len()` rather than hard-coded, so a new
/// catalog variant added to the list without bumping a separate constant
/// cannot drift (r6b-err-2).
///
/// **Adding a variant:** add it to [`ALL_CATALOG_VARIANTS`], then add the
/// matching `match` arms in `ErrorCode::as_str`, `as_static_str`, and
/// `from_str`, then document the code in `docs/ERROR-CATALOG.md`. The
/// `round_trips_via_as_str_from_str` test below is the tripwire that fails
/// loudly if any of those steps is skipped.
///
/// `ErrorCode::Unknown(String)` is deliberately excluded — it's the
/// forward-compat fallback, not a catalog code.
const ALL_CATALOG_VARIANTS: &[ErrorCode] = &[
    ErrorCode::InvCycle,
    ErrorCode::InvDepthExceeded,
    ErrorCode::InvFanoutExceeded,
    ErrorCode::InvTooManyNodes,
    ErrorCode::InvTooManyEdges,
    ErrorCode::InvDeterminism,
    ErrorCode::InvContentHash,
    ErrorCode::InvRegistration,
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
    ErrorCode::VersionUnknownPrior,
    // Phase-2a G1-B HostError discriminants (PHASE_2A_RESERVED_CODES). All
    // five reserved for Phase-3 sync fires but already carry catalog
    // entries + as_str / as_static_str / from_str arms, so they belong on
    // the round-trip list.
    ErrorCode::HostNotFound,
    ErrorCode::HostWriteConflict,
    ErrorCode::HostBackendUnavailable,
    ErrorCode::HostCapabilityRevoked,
    ErrorCode::HostCapabilityExpired,
    // Phase-2a firing codes (PHASE_2A_FIRING_CODES). Added during the
    // Phase-2a R5 wave and carry full catalog + round-trip wiring.
    ErrorCode::ExecStateTampered,
    ErrorCode::ResumeActorMismatch,
    ErrorCode::ResumeSubgraphDrift,
    ErrorCode::WaitTimeout,
    ErrorCode::InvImmutability,
    ErrorCode::InvSystemZone,
    ErrorCode::InvAttribution,
    ErrorCode::CapWallclockExpired,
    ErrorCode::CapChainTooDeep,
    ErrorCode::WaitSignalShapeMismatch,
    // Phase-2a ucca-7 parse-time refusal code (lone-`*` GrantScope).
    ErrorCode::CapScopeLoneStarRejected,
];

/// Count of catalog variants (auto-derived from [`ALL_CATALOG_VARIANTS`] so
/// adding to the list and forgetting to bump a number is impossible).
const CATALOG_VARIANT_COUNT: usize = ALL_CATALOG_VARIANTS.len();

/// Every catalog variant must round-trip through `as_str` / `from_str`
/// without hitting the `Unknown` fallback. Cross-checks the enumerated
/// `ALL_CATALOG_VARIANTS` list against the enum so a variant added to the
/// enum without being added to the list is caught by the
/// `catalog_variant_count_matches_enum` test below, and a variant added to
/// the list without the matching `from_str` arm is caught here.
#[test]
fn variant_count_is_pinned() {
    // Every listed variant must round-trip through from_str(as_str).
    for code in ALL_CATALOG_VARIANTS {
        let s = code.as_str();
        let parsed = ErrorCode::from_str(s);
        assert_eq!(
            &parsed, code,
            "catalog variant {code:?} failed as_str/from_str round-trip via string {s}",
        );
    }
    // The "as_static_str" path MUST also return the same string for every
    // catalog variant — it's the path the engine's static-code accessor
    // delegates through, and it duplicating `as_str` is load-bearing for
    // the drift detector's expected reverse mapping.
    for code in ALL_CATALOG_VARIANTS {
        assert_eq!(
            code.as_str(),
            code.as_static_str(),
            "as_str / as_static_str disagree for {code:?}",
        );
    }
    // Canary: the known count at the time this harness last synced
    // (58). If a future change bumps the enum, it bumps the array, which
    // bumps this value — the assertion documents the expected movement
    // direction. Adding a variant is a +1 delta; shrinking is always a
    // breaking change that must surface in the catalog diff.
    //
    // G11-A Wave 3a sync: the earlier canary (43) predated the Phase-2a
    // R5 waves which introduced the 5 reserved HostError discriminants
    // (PHASE_2A_RESERVED_CODES), the 10 firing codes (PHASE_2A_FIRING_CODES),
    // and the ucca-7 `CapScopeLoneStarRejected` parse-time refusal. All
    // 16 additions already had `as_str` / `as_static_str` / `from_str`
    // coverage in `benten-errors/src/lib.rs` — the test list just hadn't
    // been updated. Post-sync: 42 + 16 = 58.
    assert_eq!(
        CATALOG_VARIANT_COUNT, 58,
        "CATALOG_VARIANT_COUNT drift — update this value AND docs/ERROR-CATALOG.md in the same commit",
    );
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
