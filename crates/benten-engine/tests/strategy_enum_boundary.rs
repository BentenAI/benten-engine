//! R3-C RED-PHASE pin for the Strategy enum at the engine boundary
//! (G15-A wave-5a; arch-r1-12; baked-in #2).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-A row
//!   `strategy_enum_at_engine_boundary_does_not_leak_algorithm_b_internals_per_clause_2_baked_in`.
//! - `arch-r1-12` (architectural pin: Strategy enum is the
//!   load-bearing seam between engine and IVM).
//! - CLAUDE.md baked-in #2 (sharpened at R6-R3 r6-r3-arch-8): "the
//!   engine names `benten_ivm::Strategy` as the dispatch type but no
//!   `View` / algorithm internals leak through; `benten-ivm` depends
//!   on `benten-graph::ChangeSubscriber`, never the reverse").
//!
//! ## What this pins
//!
//! The Strategy enum surface at the engine boundary MUST stay shape-
//! stable across G15-A's kernel generalization. G15-A may add
//! INTERNAL routing (Strategy::A canonical fast-path vs Strategy::B
//! generalized) but the engine-facing Strategy enum keeps the same
//! variants + same shape — no new variants, no fields exposed at the
//! engine boundary that leak Algorithm-B implementation choices.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale
//! `"RED-PHASE: G15-A wave-5a preserves Strategy boundary"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — arch-r1-12 — Strategy enum boundary stable"]
fn strategy_enum_at_engine_boundary_does_not_leak_algorithm_b_internals_per_clause_2_baked_in() {
    // arch-r1-12 + baked-in #2 pin. Concrete assertion shape (post
    // G15-A landing):
    //
    //   // The engine-facing Strategy enum has the same variants
    //   // pre and post G15-A:
    //   use benten_ivm::Strategy;
    //   let _a: Strategy = Strategy::A;
    //   let _b: Strategy = Strategy::B;
    //   // Adding a new variant without explicit RFC = compile-time
    //   // error in any code that exhaustively matches on Strategy
    //   // (the engine, the napi binding, etc.). This test compiles
    //   // an exhaustive match without #[non_exhaustive] handling:
    //   fn classify(s: Strategy) -> &'static str {
    //       match s {
    //           Strategy::A => "A",
    //           Strategy::B => "B",
    //           Strategy::C => "C",  // reserved per Phase-2b
    //       }
    //   }
    //   assert_eq!(classify(Strategy::A), "A");
    //   assert_eq!(classify(Strategy::B), "B");
    //   assert_eq!(classify(Strategy::C), "C");
    //
    //   // No `View` trait or algorithm-internal type appears in any
    //   // engine public signature. We grep the engine public API
    //   // (from cargo-public-api) for `View`, `Algorithm`, `Kernel`,
    //   // and assert NONE appear at function/method signature
    //   // boundaries (only `Strategy` is permitted).
    //
    // OBSERVABLE consequence: a future refactor that, e.g., exposes
    // `benten_ivm::algorithm_b::Kernel` in an engine method
    // signature fails this test. Defends against architectural leaks
    // that would tie the engine to a specific IVM algorithm choice.
    unimplemented!("G15-A wires Strategy enum boundary stability + public-API leak audit");
}
