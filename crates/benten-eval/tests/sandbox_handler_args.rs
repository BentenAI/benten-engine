//! R3-D RED-PHASE pins for 24th p/c drift acceptance criterion —
//! Rust eval-side casing discipline (G17-C wave 5b; pim-2 LOAD-BEARING;
//! phase-3-backlog §6.6 + §3.6b).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-C + §3.D 24th p/c drift):
//!
//! - `tests/sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
//!   — pim-2 end-to-end pin (24th p/c drift)
//! - `tests/sandbox_per_handler_output_limit_bytes_camel_case_dsl_round_trips`
//!   — 24th p/c drift
//!
//! ## 24th p/c drift acceptance criterion (per phase-3-backlog §6.6)
//!
//! Phase-2b SANDBOX named-manifest carried 23 producer-consumer drifts
//! in named storage; the 24th was identified at phase-2b-close R6 R6
//! via Ben's HARD RULE call as BELONGS-NAMED-NOW: the per-handler
//! `wallclockMs` (camelCase, DSL) ↔ `wallclock_ms` (snake_case, eval)
//! plus `outputLimitBytes` ↔ `output_limit_bytes`.
//!
//! The acceptance criterion is end-to-end:
//!
//! 1. TS DSL builder accepts `{ wallclockMs: 100, outputLimitBytes: 4096 }`.
//! 2. The DSL serializer translates to snake_case before crossing the
//!    napi boundary (`packages/engine/src/dsl.ts` — G17-C mints the
//!    translateSandboxArgs helper per §6.6, mirrors PR #76 translateWaitArgs
//!    precedent).
//! 3. The napi argv arrives with snake_case keys.
//! 4. The eval-side parser reads `wallclock_ms` + `output_limit_bytes`
//!    correctly (existing surface).
//! 5. SANDBOX execution observes the configured ceiling.
//!
//! ## Rust-side pin shape
//!
//! This file pins the Rust eval-side observable end of the round-trip.
//! The TS-side test (`packages/engine/test/sandbox_handler_args.test.ts`)
//! pins the DSL serializer.
//!
//! Pairs with G17-C's `register_module_bytes` napi method test +
//! `manifest_unknown.rs` (engine-side surface) for the full §6.6
//! deliverable coverage.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-C wave 5b authors translateSandboxArgs DSL helper + un-ignores eval-side assertion (pim-2 LOAD-BEARING)"]
fn sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case() {
    // pim-2 LOAD-BEARING + 24th p/c drift acceptance criterion pin.
    // G17-C implementer wires this:
    //
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef};
    //
    //   // Snake-case keys arrive at the eval-side (per the DSL
    //   // serializer translation):
    //   let argv_after_napi_xlate = serde_json::json!({
    //       "manifest_name": "compute:safe-default",
    //       "wallclock_ms": 100,
    //       "output_limit_bytes": 4096,
    //   });
    //
    //   // Sandbox config built from the (already-translated) argv:
    //   let config: SandboxConfig = serde_json::from_value(argv_after_napi_xlate).unwrap();
    //   let sandbox = Sandbox::new(config);
    //
    //   // The wallclock ceiling reaches the running SANDBOX:
    //   //   (un-ignore the existing test
    //   //    `crates/benten-eval/tests/sandbox_wallclock.rs` per §6.6)
    //   let result = sandbox.execute(/* fixture: long-running guest */);
    //   assert!(matches!(
    //       result.unwrap_err(),
    //       benten_eval::SandboxError::WallclockExceeded { limit_ms: 100 }
    //   ));
    //
    // OBSERVABLE consequence: the wallclock ceiling specified in
    // camelCase at the DSL surface is OBSERVED at the SANDBOX guest
    // boundary. A regression that loses the translation (e.g. by
    // dropping a translateSandboxArgs call site, or by adding a new
    // arg to the manifest schema without updating the translator)
    // surfaces here as a failed wallclock-trip assertion.
    //
    // Defends pim-2 §3.6b end-to-end pin requirement directly.
    unimplemented!(
        "G17-C wires 24th p/c drift end-to-end assertion: camelCase DSL → snake_case eval → wallclock observed"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-C wave 5b 24th p/c drift output-limit-bytes round-trip"]
fn sandbox_per_handler_output_limit_bytes_camel_case_dsl_round_trips() {
    // 24th p/c drift sibling pin. G17-C implementer:
    //
    //   // Same translation chain as wallclock — but for output limit:
    //   let argv_after_napi_xlate = serde_json::json!({
    //       "manifest_name": "compute:safe-default",
    //       "output_limit_bytes": 4096,
    //   });
    //
    //   let config: SandboxConfig = serde_json::from_value(argv_after_napi_xlate).unwrap();
    //   let sandbox = Sandbox::new(config);
    //
    //   // The output ceiling is enforced — guest that emits >4096
    //   // bytes traps with output overflow:
    //   let result = sandbox.execute(/* fixture: emits 8KB */);
    //   assert!(matches!(
    //       result.unwrap_err(),
    //       benten_eval::SandboxError::OutputOverflow(_)
    //   ));
    //
    // OBSERVABLE consequence: parallel to wallclock — distinct
    // observable axis (output ceiling vs time ceiling). Both
    // distinct end-to-end pins guard against:
    //
    //   1. The translator forgets ONE of the camelCase keys (e.g.
    //      added wallclockMs but forgot outputLimitBytes).
    //   2. The eval-side parser silently ignores an unknown key (the
    //      `OutputOverflow` would never fire because the ceiling
    //      defaulted).
    //
    // Distinct end-to-end consequence per pim-2 §3.6b.
    unimplemented!("G17-C wires 24th p/c drift end-to-end assertion for output_limit_bytes axis");
}
