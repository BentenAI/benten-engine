// Phase 3 G20-A3 — un-ignored: production code at
// `crates/benten-engine/src/engine_views.rs::register_user_view`
// already rejects Strategy::A + Strategy::C; this file lifts the
// red-phase pin to a green-phase end-to-end driver per dispatch-
// conventions §3.6b (production entry point + observable
// consequence).
//
//! Phase 3 G20-A3 (Phase 2b R4-FP B-1 origin) — D8-RESOLVED:
//! `Engine::register_user_view` REFUSES Strategy::A + Strategy::C
//! at registration time.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D8-RESOLVED
//!     ("REFUSES `'A'` with typed error since hand-written = Rust-only
//!     and not user-registerable from TS").
//!   - `.addl/phase-2b/r2-test-landscape.md` §7 + §8 D8 row.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04.
//!   - `docs/future/phase-3-backlog.md §7.3.A.3` (CLOSED at G20-A3).
//!
//! Each test drives the production entry point
//! `Engine::register_user_view` (§3.6b end-to-end pin) — would
//! FAIL silently if the engine_views.rs strategy match arms ever
//! regressed to silent-accept.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, UserViewInputPattern, UserViewSpec};

/// `user_view_strategy_a_refused_with_typed_error` — D8 + plan §3 G8-B.
///
/// Building a `UserViewSpec` with `.strategy(Strategy::A)` and passing
/// it to `Engine::register_user_view` MUST yield a typed
/// `EngineError::ViewStrategyARefused`.
#[test]
fn user_view_strategy_a_refused_with_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("user_a_attempt")
        .input_pattern(UserViewInputPattern::AnchorPrefix("post".into()))
        .strategy(benten_ivm::Strategy::A)
        .build()
        .expect("UserViewSpec builder constructs (rejection lives at register_user_view)");

    let err = engine
        .register_user_view(spec)
        .expect_err("register_user_view MUST reject Strategy::A for user views");

    let rendered = err.to_string();
    assert!(
        rendered.contains("Strategy::A") || rendered.contains("Strategy A"),
        "expected typed Strategy::A-refused error, got: {rendered}"
    );
    assert!(
        rendered.contains("user_a_attempt"),
        "error must surface the offending view_id, got: {rendered}"
    );
}

/// Companion: `Strategy::C` (reserved for Phase-3 Z-set cancellation
/// per g8-concern-3) MUST also be refused for user views.
#[test]
fn user_view_strategy_c_refused_as_reserved() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("user_c_attempt")
        .input_pattern(UserViewInputPattern::AnchorPrefix("post".into()))
        .strategy(benten_ivm::Strategy::Reserved)
        .build()
        .expect("UserViewSpec builder constructs (rejection lives at register_user_view)");

    let err = engine
        .register_user_view(spec)
        .expect_err("register_user_view MUST reject Strategy::C for user views");

    let rendered = err.to_string();
    assert!(
        rendered.contains("Strategy::C") || rendered.contains("Strategy C"),
        "expected typed Strategy::C-reserved error, got: {rendered}"
    );
    assert!(
        rendered.contains("user_c_attempt"),
        "error must surface the offending view_id, got: {rendered}"
    );
}
