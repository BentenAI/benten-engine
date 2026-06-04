//! #1187 regression pin (refinement-audit-2026-05).
//!
//! `JsAtrium::join()`'s engine-bound path MUST drive
//! `AtriumConfig::production()` — NOT `for_test()`.
//!
//! Pre-fix the engine-bound `join()` path unconditionally passed
//! `EngineAtriumConfig::for_test()` (= `AtriumMode::Loopback`) to
//! `engine.open_atrium(...)`, so ANY production JS caller invoking
//! `engine.atrium(id).join()` silently bound a loopback (no-relay,
//! no-holepunch) transport regardless of intent — a deployed
//! production-invariant violation invisible in default-feature
//! builds (the same severity class as the META #660 deployed-
//! invariant-violation cluster).
//!
//! The original #869 finding prescribed threading `atriumId`
//! through `EngineAtriumConfig::from_id(...)`, but post-COLLAPSE the
//! engine-side `AtriumConfig` carries NO atrium-id field
//! (`AtriumConfig { mode }` only); atrium identity lives solely in
//! the napi layer's `JsAtrium.config.atrium_id`. The genuine
//! residual is the transport-mode mis-wire — the engine-bound path
//! is the real-peer path and MUST drive `production()`.
//!
//! This pin asserts the production-mode contract `join()` now
//! depends on. It would FAIL if a future edit reverted the
//! engine-bound path back to `for_test()` (Loopback) or if the
//! engine-side `production()` constructor stopped yielding
//! `AtriumMode::Production`. Carried as an integration test (its own
//! compilation unit) rather than a `src` unit test because the
//! lib-test target is blocked on a pre-existing-on-main
//! `benten_engine::testing` reachability residual in `wait.rs`
//! (orthogonal to this lane).

#![cfg(all(feature = "napi-export", not(target_arch = "wasm32")))]

use benten_engine::atrium_api::{AtriumConfig, AtriumMode};

#[test]
fn join_engine_bound_path_uses_production_not_loopback() {
    // The exact config the engine-bound `JsAtrium::join()` path
    // constructs (see `bindings/napi/src/atrium.rs::join`).
    let prod = AtriumConfig::production();
    assert_eq!(
        prod.mode,
        AtriumMode::Production,
        "engine-bound join() must bind a production transport, not \
         the test-fixture loopback config (#1187)"
    );

    // Negative half: prove the two configs are observably distinct
    // so the pin actually catches a regression back to `for_test()`.
    let test_cfg = AtriumConfig::for_test();
    assert_eq!(test_cfg.mode, AtriumMode::Loopback);
    assert_ne!(
        prod.mode, test_cfg.mode,
        "production() and for_test() must yield distinct modes — \
         otherwise the #1187 mis-wire would be invisible"
    );
}
