//! r6-err-4 / r6-err-10 regressions:
//! - Every `EngineError` variant that wraps a synthesized catalog code now
//!   routes through a first-class `ErrorCode` enum variant (no more
//!   `ErrorCode::Unknown("E_XXX".into())` placeholders).
//! - `EngineError::code()` delegates to `ErrorCode::as_static_str()` — the
//!   engine no longer duplicates the match arms from `benten-core`.

use benten_core::ErrorCode;
use benten_engine::EngineError;

#[test]
fn duplicate_handler_routes_to_catalog_variant() {
    let err = EngineError::DuplicateHandler {
        handler_id: "create_post".into(),
    };
    assert_eq!(err.error_code(), ErrorCode::DuplicateHandler);
    assert_eq!(err.code(), "E_DUPLICATE_HANDLER");
    assert!(!matches!(err.error_code(), ErrorCode::Unknown(_)));
}

#[test]
fn no_capability_policy_configured_routes_to_catalog_variant() {
    let err = EngineError::NoCapabilityPolicyConfigured;
    assert_eq!(err.error_code(), ErrorCode::NoCapabilityPolicyConfigured);
    assert_eq!(err.code(), "E_NO_CAPABILITY_POLICY_CONFIGURED");
}

#[test]
fn production_requires_caps_routes_to_catalog_variant() {
    let err = EngineError::ProductionRequiresCaps;
    assert_eq!(err.error_code(), ErrorCode::ProductionRequiresCaps);
    assert_eq!(err.code(), "E_PRODUCTION_REQUIRES_CAPS");
}

#[test]
fn subsystem_disabled_routes_to_catalog_variant() {
    let err = EngineError::SubsystemDisabled { subsystem: "ivm" };
    assert_eq!(err.error_code(), ErrorCode::SubsystemDisabled);
    assert_eq!(err.code(), "E_SUBSYSTEM_DISABLED");
}

#[test]
fn unknown_view_routes_to_catalog_variant() {
    let err = EngineError::UnknownView {
        view_id: "not_a_view".into(),
    };
    assert_eq!(err.error_code(), ErrorCode::UnknownView);
    assert_eq!(err.code(), "E_UNKNOWN_VIEW");
}

#[test]
fn not_implemented_routes_to_catalog_variant() {
    let err = EngineError::NotImplemented {
        feature: "engine.trace",
    };
    assert_eq!(err.error_code(), ErrorCode::NotImplemented);
    assert_eq!(err.code(), "E_NOT_IMPLEMENTED");
}

#[test]
fn code_delegates_to_as_static_str() {
    // r6-err-10: `EngineError::code()` used to duplicate a `static_for`
    // match over ErrorCode; now it delegates to
    // `ErrorCode::as_static_str` so drift between the two is impossible.
    let err = EngineError::Other {
        code: ErrorCode::WriteConflict,
        message: "race".into(),
    };
    assert_eq!(err.code(), ErrorCode::WriteConflict.as_static_str());
    assert_eq!(err.code(), "E_WRITE_CONFLICT");
}
