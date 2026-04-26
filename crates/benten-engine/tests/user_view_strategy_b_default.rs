#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G8-B
// `engine.create_view` + Strategy::B default for user views.
//
//! Phase 2b R4-FP (B-1) — D8-RESOLVED: user views default Strategy::B.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D8-RESOLVED
//!     (`engine.createView(spec)` defaults `'B'` for user views;
//!     the 5 hand-written views stay `Strategy::A` as baselines).
//!   - `.addl/phase-2b/r2-test-landscape.md` §7 row 462 (TS counterpart).
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04 (G8-B coverage gap).
//!
//! Rust-side companion to `packages/engine/test/views.test.ts`. The TS
//! test pins the DSL surface; this test pins the Rust surface contract
//! that the napi bridge calls into.
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use benten_engine::Engine;

/// `user_view_create_defaults_strategy_b` — D8 + plan §3 G8-B.
///
/// A `UserViewSpec` built without an explicit `.strategy(...)` call
/// MUST default to `Strategy::B`. This is asymmetric vs. the
/// `View::strategy() -> Strategy::A` default for hand-written views
/// (which are Rust-only baselines per D8); user views are the
/// generalized Algorithm B path and must default accordingly.
#[test]
#[ignore = "Phase 2b G8-B pending — UserViewSpec::default strategy unimplemented"]
fn user_view_create_defaults_strategy_b() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo:
    //   let spec = UserViewSpec::builder()
    //       .id("user_default_strategy")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       // NO .strategy(...) call — exercise the default.
    //       .build()
    //       .unwrap();
    //   assert_eq!(
    //       spec.strategy(),
    //       benten_ivm::Strategy::B,
    //       "UserViewSpec MUST default to Strategy::B (D8); \
    //        Strategy::A is reserved for hand-written Rust views"
    //   );
    //   engine.create_view(spec).expect("create_view succeeds with default B");
    todo!("R5 G8-B — UserViewSpec default strategy assertion");
}

/// Companion: explicit `Strategy::B` opt-in is identical to the default.
/// Pin against accidental drift where someone adds a "default-strategy
/// auto-select" path (D8 explicitly rejects auto-select).
#[test]
#[ignore = "Phase 2b G8-B pending"]
fn user_view_explicit_strategy_b_matches_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo:
    //   let default_spec = UserViewSpec::builder()
    //       .id("user_default")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       .build().unwrap();
    //   let explicit_spec = UserViewSpec::builder()
    //       .id("user_explicit")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       .strategy(Strategy::B)
    //       .build().unwrap();
    //   assert_eq!(default_spec.strategy(), explicit_spec.strategy());
    todo!("R5 G8-B — default strategy matches explicit B opt-in");
}
