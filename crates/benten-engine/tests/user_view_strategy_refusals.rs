//! Phase 2b G8-B (D8-RESOLVED): typed-error refusal tests for
//! `Engine::create_user_view`.
//!
//! `Strategy::A` is reserved for the 5 hand-written Phase-1 IVM views
//! (Rust-only). User-registered views must use `Strategy::B` (the
//! default per D8). `Strategy::C` is reserved for Phase 3+ Z-set / DBSP
//! cancellation and is refused at registration time in Phase 2b.
//!
//! These tests pin the engine boundary — `create_user_view` returns
//! `EngineError::ViewStrategyARefused` / `EngineError::ViewStrategyCReserved`
//! BEFORE any subscriber side-effect. The error.code() round-trips to the
//! catalog string `E_VIEW_STRATEGY_A_REFUSED` /
//! `E_VIEW_STRATEGY_C_RESERVED` so cross-language consumers (TS bindings
//! via napi) see the same string.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{
    Engine, EngineError, ErrorCode, UserViewInputPattern, UserViewSpec,
};

#[test]
fn user_view_strategy_a_refused_at_registration() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("user_strategy_a_attempt")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .strategy(benten_ivm::Strategy::A)
        .build()
        .unwrap();

    let err = engine
        .create_user_view(spec)
        .expect_err("Strategy::A must be refused at registration");

    match &err {
        EngineError::ViewStrategyARefused { view_id } => {
            assert_eq!(view_id, "user_strategy_a_attempt");
        }
        other => panic!("expected ViewStrategyARefused, got {other:?}"),
    }
    assert_eq!(err.code(), ErrorCode::ViewStrategyARefused);
    assert_eq!(err.code_as_str(), "E_VIEW_STRATEGY_A_REFUSED");
}

#[test]
fn user_view_strategy_c_reserved_at_registration() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("user_strategy_c_attempt")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .strategy(benten_ivm::Strategy::C)
        .build()
        .unwrap();

    let err = engine
        .create_user_view(spec)
        .expect_err("Strategy::C must be refused at registration");

    match &err {
        EngineError::ViewStrategyCReserved { view_id } => {
            assert_eq!(view_id, "user_strategy_c_attempt");
        }
        other => panic!("expected ViewStrategyCReserved, got {other:?}"),
    }
    assert_eq!(err.code(), ErrorCode::ViewStrategyCReserved);
    assert_eq!(err.code_as_str(), "E_VIEW_STRATEGY_C_RESERVED");
}

#[test]
fn user_view_default_strategy_b_is_accepted() {
    // D8-RESOLVED: builder default is Strategy::B for user views; the
    // accepted path returns a CID for the persisted view-definition Node.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("user_default_b")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        // No .strategy(...) — exercises the D8 default (B).
        .build()
        .unwrap();
    assert_eq!(spec.strategy(), benten_ivm::Strategy::B);

    let cid = engine
        .create_user_view(spec)
        .expect("default Strategy::B must be accepted");
    // Node CID is content-addressed; assert it round-trips through the
    // base32 encoder so we know a real Node was persisted.
    let s = cid.to_base32();
    assert!(s.starts_with('b'), "CID base32 prefix expected: {s}");
}

#[test]
fn user_view_explicit_strategy_b_matches_default_acceptance() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let explicit = UserViewSpec::builder()
        .id("user_explicit_b")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();
    let _ = engine
        .create_user_view(explicit)
        .expect("explicit Strategy::B accepted");

    let default = UserViewSpec::builder()
        .id("user_default_b_again")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();
    let _ = engine
        .create_user_view(default)
        .expect("default Strategy::B accepted");
}

#[test]
fn user_view_subsystem_disabled_when_ivm_off() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .without_ivm()
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("user_when_ivm_off")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();

    let err = engine
        .create_user_view(spec)
        .expect_err("create_user_view must refuse when IVM disabled");
    match err {
        EngineError::SubsystemDisabled { subsystem } => assert_eq!(subsystem, "ivm"),
        other => panic!("expected SubsystemDisabled, got {other:?}"),
    }
}
