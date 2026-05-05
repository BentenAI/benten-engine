//! R3-D RED-PHASE pins for 24th p/c drift acceptance criterion —
//! Rust eval-side casing discipline (G17-C wave 5b; pim-2 LOAD-BEARING;
//! phase-3-backlog §6.6 + §3.6b).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-C + §3.D 24th p/c drift):
//!
//! - `tests/sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
//!   — pim-2 end-to-end pin (24th p/c drift)
//! - `tests/sandbox_per_handler_output_limit_camel_case_dsl_round_trips`
//!   — 24th p/c drift (RECALIBRATED at R4-FP per r4-r1-wsa-1 BLOCKER —
//!   canonical eval-side property is `output_limit`, NOT
//!   `output_limit_bytes`; see verification grep at
//!   `crates/benten-engine/src/primitive_host.rs:877` +
//!   `crates/benten-dsl-compiler/src/lib.rs:761+765`).
//! - `tests/sandbox_per_handler_property_name_does_not_drift_across_dsl_compiler_and_primitive_host`
//!   — r4-r1-wsa-1 architectural drift-pin (NEW at R4-FP — defends
//!   against the 25th p/c drift recurrence by asserting both producer
//!   sites name the same property string).
//!
//! ## 24th p/c drift acceptance criterion (per phase-3-backlog §6.6)
//!
//! Phase-2b SANDBOX named-manifest carried 23 producer-consumer drifts
//! in named storage; the 24th was identified at phase-2b-close R6 R6
//! via Ben's HARD RULE call as BELONGS-NAMED-NOW: the per-handler
//! `wallclockMs` (camelCase, DSL) ↔ `wallclock_ms` (snake_case, eval)
//! plus `outputLimitBytes` (camelCase, DSL — `Bytes` for type-clarity)
//! ↔ `output_limit` (snake_case, eval — NO `_bytes` suffix; symmetric
//! with `wallclock_ms` which carries no `_milliseconds` suffix).
//!
//! The acceptance criterion is end-to-end:
//!
//! 1. TS DSL builder accepts `{ wallclockMs: 100, outputLimitBytes: 4096 }`.
//! 2. The DSL serializer translates to snake_case before crossing the
//!    napi boundary (`packages/engine/src/dsl.ts` — G17-C mints the
//!    translateSandboxArgs helper per §6.6, mirrors PR #76 translateWaitArgs
//!    precedent). Specifically: `outputLimitBytes` → `output_limit`
//!    (drops `Bytes` suffix on the eval-side per the canonical name).
//! 3. The napi argv arrives with snake_case keys (`wallclock_ms` +
//!    `output_limit`).
//! 4. The eval-side parser reads `wallclock_ms` + `output_limit`
//!    correctly (existing surface; `primitive_host.rs:877` reads
//!    `op.properties.get("output_limit")`).
//! 5. SANDBOX execution observes the configured ceiling.
//!
//! ## Why the 25th-drift recurrence (r4-r1-wsa-1)
//!
//! At R3-D the snake_case target was authored as `output_limit_bytes`
//! by symmetry with `wallclockMs` → `wallclock_ms` (preserving every
//! token of the camelCase form). But the eval-side reader at
//! `primitive_host.rs:877` reads `op.properties.get("output_limit")`
//! and the DSL-compiler at `dsl-compiler/src/lib.rs:761+765` writes
//! `output_limit: 65536` (no `_bytes`). The 25th p/c drift would have
//! materialized inside the 24th-drift fix: the test pin would request
//! a translation that drops the value into a property name eval ignores,
//! causing the OutputOverflow assertion to pass by default-fallthrough
//! rather than by the 4096-byte ceiling actually being applied.
//!
//! Pim-8 (mirror-precedent overshoot per §3.6c) hit a second time after
//! Phase-2b PR #76. Recurrence-watch implication noted in r4-r1
//! findings; codified inline at R4-FP via the architectural drift-pin
//! `..._does_not_drift_across_dsl_compiler_and_primitive_host`.
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
    //       "output_limit": 4096,
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
#[ignore = "RED-PHASE: G17-C wave 5b 24th p/c drift output-limit round-trip (canonical eval-side name = `output_limit`, NOT `output_limit_bytes` — see r4-r1-wsa-1 + primitive_host.rs:877 + dsl-compiler/src/lib.rs:761+765)"]
fn sandbox_per_handler_output_limit_camel_case_dsl_round_trips() {
    // 24th p/c drift sibling pin. RECALIBRATED at R4-FP per r4-r1-wsa-1
    // BLOCKER — the eval-side canonical property is `output_limit`,
    // NOT `output_limit_bytes`. The TS-side DSL surface remains
    // `outputLimitBytes` (camelCase, with `Bytes` for type-clarity);
    // translateSandboxArgs MAPS `outputLimitBytes` → `output_limit`
    // (drops the `Bytes` token), parallel to how `wallclockMs` →
    // `wallclock_ms` preserves every token only because the eval-side
    // canonical ALSO carries `ms`.
    //
    // G17-C implementer wires this:
    //
    //   // Same translation chain as wallclock — but for output limit:
    //   let argv_after_napi_xlate = serde_json::json!({
    //       "manifest_name": "compute:safe-default",
    //       "output_limit": 4096,   // <-- canonical eval-side key
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
    //      defaulted to the eval-side default 1 MB ceiling).
    //   3. The translator emits the WRONG snake_case target (e.g.
    //      `output_limit_bytes` instead of `output_limit`); eval-side
    //      reader at primitive_host.rs:877 silently drops the value
    //      and the OutputOverflow assertion would still fire by
    //      default-fallthrough — the 25th p/c drift recurrence shape.
    //      The companion architectural drift-pin
    //      `..._does_not_drift_across_dsl_compiler_and_primitive_host`
    //      (below) defends against case (3) directly.
    //
    // Distinct end-to-end consequence per pim-2 §3.6b.
    unimplemented!("G17-C wires 24th p/c drift end-to-end assertion for output_limit axis");
}

#[test]
#[ignore = "RED-PHASE: G17-C wave 5b — architectural drift-pin (r4-r1-wsa-1; defends against 25th p/c drift recurrence)"]
fn sandbox_per_handler_property_name_does_not_drift_across_dsl_compiler_and_primitive_host() {
    // r4-r1-wsa-1 architectural drift-pin. Defends against the 25th
    // p/c drift recurrence shape by asserting both PRODUCER sites name
    // the SAME property string. If a refactor renames `output_limit` on
    // one side but not the other (or introduces `output_limit_bytes` on
    // one side and not the other), this pin fires.
    //
    // G17-C implementer wires this (or G20-B docs-sweep agent if it
    // composes with adjacent drift detectors):
    //
    //   // Read both producer sites:
    //   let primitive_host = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("benten-engine").join("src").join("primitive_host.rs")
    //   ).unwrap();
    //   let dsl_compiler = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("benten-dsl-compiler").join("src").join("lib.rs")
    //   ).unwrap();
    //
    //   // Both name the canonical SANDBOX per-handler ceiling property
    //   // as `output_limit` (NOT `output_limit_bytes`). Symmetric with
    //   // `wallclock_ms` (NOT `wallclock_milliseconds`).
    //   assert!(primitive_host.contains("\"output_limit\""),
    //       "primitive_host.rs MUST read SANDBOX per-handler `output_limit` property \
    //        per r4-r1-wsa-1 architectural drift-pin");
    //   assert!(dsl_compiler.contains("output_limit:") || dsl_compiler.contains("\"output_limit\""),
    //       "dsl-compiler/lib.rs MUST emit SANDBOX per-handler `output_limit` property \
    //        per r4-r1-wsa-1 architectural drift-pin");
    //
    //   // The non-canonical drift form is FORBIDDEN at both sites:
    //   assert!(!primitive_host.contains("\"output_limit_bytes\""),
    //       "primitive_host.rs MUST NOT read `output_limit_bytes` (drift form rejected at \
    //        R4-FP per r4-r1-wsa-1 — canonical eval-side name is `output_limit`)");
    //   assert!(!dsl_compiler.contains("output_limit_bytes:") && !dsl_compiler.contains("\"output_limit_bytes\""),
    //       "dsl-compiler/lib.rs MUST NOT emit `output_limit_bytes` (drift form rejected at \
    //        R4-FP per r4-r1-wsa-1 — canonical name is `output_limit`)");
    //
    //   // Symmetric assertion for wallclock — same property name on both sides:
    //   assert!(primitive_host.contains("\"wallclock_ms\""),
    //       "primitive_host.rs MUST read `wallclock_ms` per producer-consumer parity");
    //   assert!(dsl_compiler.contains("wallclock_ms:") || dsl_compiler.contains("\"wallclock_ms\""),
    //       "dsl-compiler/lib.rs MUST emit `wallclock_ms` per producer-consumer parity");
    //
    // OBSERVABLE consequence: a refactor that renames the property on
    // ONE side without updating the other fires this pin BEFORE the
    // 24th-drift end-to-end pin's default-fallthrough false-pass shape
    // can mask the regression. Defends r4-r1-wsa-1 + the pim-8
    // mirror-precedent-overshoot recurrence-watch.
    unimplemented!(
        "G17-C wires architectural drift-pin asserting `output_limit` + `wallclock_ms` consistency across primitive_host.rs + dsl-compiler/src/lib.rs"
    );
}
