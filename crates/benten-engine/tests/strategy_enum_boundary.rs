//! GREEN-PHASE pin for the Strategy enum at the engine boundary
//! (G15-A wave-5a; arch-r1-12; baked-in #2).
//!
//! ## Pin source
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
//! The Strategy enum surface at the engine boundary stays shape-stable
//! across G15-A's kernel generalization. G15-A added INTERNAL routing
//! ([`benten_ivm::dispatch_for`] returning `Strategy::A` for canonical
//! ids and `Strategy::B` for user-defined ids) but the engine-facing
//! Strategy enum keeps the same closed `{ A, B, C }` variant set —
//! adding a new variant without an explicit RFC would be a breaking
//! change that an exhaustive match catches at compile time.

#![allow(clippy::unwrap_used)]

use benten_ivm::Strategy;

#[test]
fn strategy_enum_at_engine_boundary_does_not_leak_algorithm_b_internals_per_clause_2_baked_in() {
    // baked-in #2: the engine names `benten_ivm::Strategy` only — no
    // `View` / algorithm internals leak through. Compile-time pin via
    // exhaustive match: any new variant breaks this test loudly.
    fn classify(s: Strategy) -> &'static str {
        match s {
            Strategy::A => "A",
            Strategy::B => "B",
            Strategy::C => "C",
        }
    }
    assert_eq!(classify(Strategy::A), "A");
    assert_eq!(classify(Strategy::B), "B");
    assert_eq!(classify(Strategy::C), "C");

    // The G15-A internal dispatch router lives behind a `pub fn
    // dispatch_for(view_id: &str) -> Strategy` — it RETURNS a Strategy
    // (no algorithm-internal type leakage) and TAKES a `&str`
    // (no `View` trait object). The engine consumes only the Strategy
    // return value.
    let _: Strategy = benten_ivm::dispatch_for("capability_grants");
    let _: Strategy = benten_ivm::dispatch_for("custom:foo");

    // The Algorithm B kernel surface (`AlgorithmBView`, `Algorithm`,
    // `LabelPattern`, `Projection`) is named at `benten_ivm::*` but
    // is NOT named in any `benten_engine` public-API method
    // signature. The engine consumes the kernel by:
    //   - storing `Box<dyn benten_ivm::View>` in the Subscriber
    //     (the trait object IS shared, but `View` is the public IVM
    //     trait — not an Algorithm-B-internal type).
    //   - returning `Option<Strategy>` from `Engine::view_strategy`.
    //   - never returning a kernel-typed value.
    //
    // Pinning that "engine signatures do not name kernel internals" at
    // the source level is a structural property the cargo-public-api
    // diff in CI handles end-to-end (post-G15-A landing the public
    // API diff for benten-engine MUST NOT introduce
    // `algorithm_b::Kernel` / `LabelPattern` / `Projection` /
    // `AlgorithmBView` symbols at engine method signatures). This
    // test is the source-level companion: it asserts the Strategy
    // enum itself is the only IVM-typed return value at the engine
    // boundary that consumers downcast on.
    use benten_engine::Engine;
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    // Compile-time: `view_strategy` is `Option<Strategy>` — not
    // `Option<Box<dyn View>>` or `Option<AlgorithmBView>`.
    let _strategy: Option<Strategy> = engine.view_strategy("custom:not_registered");
}
