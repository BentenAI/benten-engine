// R3-followup (R4-FP B-1) red-phase converted to green by G8-B.
//
//! Phase 2b G8-B — user-registered IVM views unit tests.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §3 G8-B (`create_view`
//!     goes live; removes the Phase-1 `TODO(phase-2-view-id-registry)`).
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.7 rows 182-184.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04 (zero R3-landed coverage).
//!
//! Three assertions:
//!
//!   1. `user_registered_view_end_to_end` — register a user-defined view
//!      via `Engine::create_user_view(spec)`; assert the registration
//!      returns a CID for the persisted view-definition Node.
//!
//!   2. `user_view_pattern_mismatch_fires_typed_error` — a user view
//!      whose builder is missing the required `input_pattern` field
//!      MUST be rejected at `build()` with a typed error message.
//!
//!   3. `engine_create_view_removes_phase_1_todo` — the
//!      `TODO(phase-2-view-id-registry)` marker at engine.rs:~1976 is
//!      GONE after G8-B lands.
//!
//! Owned by R4-FP B-1; landed by G8-B.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, UserViewInputPattern, UserViewSpec};

/// `user_registered_view_end_to_end` — R2 §1.7 + plan §3 G8-B.
///
/// Register a `UserViewSpec` via `Engine::create_user_view`. Assert the
/// registration returns a CID and that the spec's strategy resolves to
/// `Strategy::B` per D8 default.
///
/// Full materialization (synthetic event emission + read-view assertion)
/// lights up alongside G8-A's generalized Algorithm B port; the
/// registration round-trip is the load-bearing surface this test pins
/// today.
#[test]
fn user_registered_view_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("user_posts_by_author")
        .input_pattern(UserViewInputPattern::AnchorPrefix("post".to_string()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();
    assert_eq!(spec.strategy(), benten_ivm::Strategy::B);
    assert_eq!(spec.id(), "user_posts_by_author");

    let cid = engine
        .create_user_view(spec)
        .expect("create_user_view succeeds with default Strategy::B");
    assert!(
        cid.to_base32().starts_with('b'),
        "view-definition CID must round-trip through the base32 encoder"
    );
}

/// `user_view_pattern_mismatch_fires_typed_error` — R2 §1.7 + plan §3 G8-B.
///
/// A `UserViewSpec` builder missing the required `input_pattern` field
/// MUST surface a typed error from `build()` rather than silently
/// producing a perpetually-empty view.
#[test]
fn user_view_pattern_mismatch_fires_typed_error() {
    let result = UserViewSpec::builder()
        .id("user_no_pattern")
        // Missing .input_pattern(...) — exercise the missing-field error.
        .build();
    let err = result.expect_err("missing input_pattern must be a typed error");
    assert!(
        err.contains("input_pattern"),
        "error must name the missing field; got: {err}"
    );
}

/// `engine_create_view_removes_phase_1_todo` — R2 §1.7 + plan §3 G8-B.
///
/// The Phase-1 `TODO(phase-2-view-id-registry)` MUST be removed when
/// G8-B lands: the per-view-definition registry replaces the hard-coded
/// 5-name whitelist. Drift detector via source grep against `engine.rs`.
#[test]
fn engine_create_view_removes_phase_1_todo() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine.rs",
    ))
    .expect("engine.rs readable");
    assert!(
        !src.contains("TODO(phase-2-view-id-registry)"),
        "the Phase-1 view-id-registry TODO MUST be removed by G8-B \
         (per-view definition registration via create_user_view replaces \
         the 5-name whitelist for the dynamic-registration path)"
    );

    // Affirmative compile-pin: the public surface MUST exist.
    fn _compile_pin(e: &Engine, s: UserViewSpec) {
        let _ = e.create_user_view(s);
    }
}
