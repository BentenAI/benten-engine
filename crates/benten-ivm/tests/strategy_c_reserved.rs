#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! `Strategy::C` (Z-set / DBSP cancellation) is reserved-not-implemented in
//! Phase 2b (g8-concern-3).
//!
//! Pin source: `r1-ivm-algorithm.json` g8-concern-3 — Z-set cancellation is
//! deferred to Phase 3+; the variant exists so the catalog of options is
//! complete and stable, but constructing a Strategy::C view returns a typed
//! `E_IVM_STRATEGY_NOT_IMPLEMENTED` error with a deferral message that names
//! the phase target.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §1.6 row 12.
//!
//! Future surface (G8-A introduces):
//! - `benten_errors::ErrorCode::IvmStrategyNotImplemented` + string
//!   `"E_IVM_STRATEGY_NOT_IMPLEMENTED"`.
//! - `benten_ivm::testing::testing_construct_view_with_strategy(Strategy::C)`
//!   returns `Err(ViewError::StrategyNotImplemented { strategy: Strategy::C, deferred_to_phase: "Phase 3+" })`
//!   — the `Result` shape is the public contract.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;
use benten_ivm::Strategy;
use benten_ivm::testing::try_construct_view_with_strategy;
use benten_ivm::view::ViewError;

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn algorithm_b_strategy_c_reserved_no_implementation_in_2b() {
    // Strategy::A + Strategy::B are constructable; Strategy::C is not.
    let a = try_construct_view_with_strategy(Strategy::A);
    let b = try_construct_view_with_strategy(Strategy::B);
    let c = try_construct_view_with_strategy(Strategy::C);

    assert!(
        a.is_ok(),
        "Strategy::A must construct cleanly (the 5 hand-written views)"
    );
    assert!(
        b.is_ok(),
        "Strategy::B must construct cleanly (Algorithm B generalized)"
    );

    let err = c.expect_err(
        "Strategy::C is RESERVED in Phase 2b — must surface a typed error, \
         not panic or silently fall back to A/B",
    );

    match &err {
        ViewError::StrategyNotImplemented {
            strategy,
            deferred_to_phase,
        } => {
            assert_eq!(*strategy, Strategy::C);
            assert!(
                deferred_to_phase.contains("Phase 3"),
                "deferral message must name the Phase-3+ target so operators \
                 know when this becomes available; got `{deferred_to_phase}`"
            );
        }
        other => panic!(
            "expected ViewError::StrategyNotImplemented, got `{other:?}` — \
             the Strategy::C reserved-not-implemented contract requires a \
             dedicated variant so callers can match exhaustively."
        ),
    }

    // Stable catalog code wired so cross-language consumers see the same
    // `E_IVM_STRATEGY_NOT_IMPLEMENTED` string at every boundary.
    assert_eq!(err.code(), ErrorCode::IvmStrategyNotImplemented);
    assert_eq!(err.code().as_str(), "E_IVM_STRATEGY_NOT_IMPLEMENTED");
}
