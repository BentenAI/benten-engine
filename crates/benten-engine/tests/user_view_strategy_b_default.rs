// R3-followup (R4-FP B-1) red-phase converted to green by G8-B.
//
//! Phase 2b G8-B — D8-RESOLVED: user views default Strategy::B.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D8-RESOLVED
//!     (`engine.createView(spec)` defaults `'B'` for user views;
//!     the 5 hand-written views stay `Strategy::A` as baselines).
//!   - `.addl/phase-2b/r2-test-landscape.md` §7 row 462 (TS counterpart).
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04 (G8-B coverage gap).
//!
//! Rust-side companion to `packages/engine/test/views.test.ts`. The TS
//! test pins the DSL-resolver surface; this test pins the Rust spec
//! builder contract that both the napi bridge and direct Rust callers
//! depend on.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{UserViewInputPattern, UserViewSpec};

/// `user_view_create_defaults_strategy_b` — D8 + plan §3 G8-B.
///
/// A `UserViewSpec` built without an explicit `.strategy(...)` call
/// MUST default to `Strategy::B`. This is asymmetric vs. the
/// `View::strategy() -> Strategy::A` default for hand-written views
/// (which are Rust-only baselines per D8); user views are the
/// generalized Algorithm B path and must default accordingly.
#[test]
fn user_view_create_defaults_strategy_b() {
    let spec = UserViewSpec::builder()
        .id("user_default_strategy")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        // No .strategy(...) — exercise the default.
        .build()
        .unwrap();

    assert_eq!(
        spec.strategy(),
        benten_ivm::Strategy::B,
        "UserViewSpec MUST default to Strategy::B (D8); \
         Strategy::A is reserved for hand-written Rust views"
    );
}

/// Companion: explicit `Strategy::B` opt-in is identical to the default.
/// Pin against accidental drift where someone adds a "default-strategy
/// auto-select" path (D8 explicitly rejects auto-select).
#[test]
fn user_view_explicit_strategy_b_matches_default() {
    let default_spec = UserViewSpec::builder()
        .id("user_default")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();
    let explicit_spec = UserViewSpec::builder()
        .id("user_explicit")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .unwrap();
    assert_eq!(default_spec.strategy(), explicit_spec.strategy());
    assert_eq!(default_spec.strategy(), benten_ivm::Strategy::B);
}
