//! R3-D RED-PHASE pin (SOURCE-CITE DIAGNOSTIC) for `engine.registerModuleBytes`
//! napi method (G17-C wave-5b; phase-3-backlog §6.6).
//!
//! Pin source: r2-test-landscape §2.5 G17-C
//! `engine_register_module_bytes_napi_source_cite_diagnostic`.
//!
//! ## r4-r2-napi-3 RELABEL (2026-05-05)
//!
//! Per `r4-r2-napi-3` MINOR + recommendation option (a): this test is
//! a SOURCE-CITE DIAGNOSTIC, NOT a load-bearing end-to-end pin per
//! pim-2 §3.6b. The grep-against-source-text shape (`napi_src.contains(
//! "fn register_module_bytes")`) verifies the method's PRESENCE on the
//! Rust source surface but does NOT drive the production-grade entry
//! point with observable behavioral consequence — that contract lives
//! at the LOAD-BEARING end-to-end pin in
//! `packages/engine/test/sandbox.test.ts` (Vitest DSL → napi →
//! engine.registerModuleBytes round-trip exercising the manifest_registry
//! WRITE path). G17-C R5 implementer un-skips both this diagnostic +
//! the load-bearing sandbox.test.ts end-to-end pin together.
//!
//! Treats source-cite-vs-runtime-pin family per r4-r1-napi-3
//! reshape precedent (engine_b_erasure_boundary.rs reshaped to
//! compile-time witness; this companion stays as source-cite diagnostic
//! by explicit labeling). Per pim-2 §3.6b: source-cite diagnostics are
//! useful scaffolding pins but do NOT satisfy the end-to-end pin
//! requirement on their own — the load-bearing pin is at sandbox.test.ts.
//!
//! ## Method-presence diagnostic shape
//!
//! G17-C ships the `register_module_bytes` napi method at
//! `bindings/napi/src/engine.rs::register_module_bytes`. It carries
//! the WRITE side of named-manifest registration:
//!
//! 1. TS DSL caller passes module bytes + name.
//! 2. napi handler validates bytes + computes CID.
//! 3. CID is stored in the engine's `manifest_registry` keyed by
//!    colon-joined name.
//! 4. CID is returned to the caller for downstream subgraph references.
//!
//! Pairs with `crates/benten-engine/tests/manifest_unknown.rs` (READ-
//! AND-VALIDATE side at `register_subgraph` time).
//!
//! Pairs with `packages/engine/test/sandbox.test.ts` (LOAD-BEARING
//! Vitest end-to-end DSL pin per pim-2 §3.6b — exercises the napi
//! method through real production flow).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G17-C wave-5b authors register_module_bytes napi method (source-cite diagnostic per r4-r2-napi-3; load-bearing end-to-end pin in sandbox.test.ts)"]
fn engine_register_module_bytes_napi_source_cite_diagnostic() {
    // phase-3-backlog §6.6 pin. G17-C implementer wires this:
    //
    //   // The napi method is exposed on the Engine surface:
    //   let napi_src = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("src").join("engine.rs")
    //   ).unwrap();
    //
    //   // Method present (snake_case Rust source side):
    //   assert!(napi_src.contains("fn register_module_bytes")
    //         || napi_src.contains("pub fn register_module_bytes"),
    //       "bindings/napi/src/engine.rs must expose register_module_bytes per §6.6");
    //
    //   // The method routes through to engine.register_module:
    //   assert!(napi_src.contains("register_module") || napi_src.contains("manifest_registry"),
    //       "register_module_bytes must dispatch through engine's manifest_registry per §6.6");
    //
    //   // TS-side surface mirror (camelCase in DSL):
    //   let ts_src = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("packages").join("engine")
    //           .join("src").join("engine.ts")
    //   ).unwrap();
    //   assert!(ts_src.contains("registerModuleBytes"),
    //       "packages/engine/src/engine.ts must expose registerModuleBytes per §6.6 + §3.5b doc-coupling");
    //
    // OBSERVABLE consequence: the napi method is reachable from the TS
    // DSL surface. A regression that drops the napi method (or
    // introduces a snake_case TS alias by mistake — pim-2 24th p/c
    // drift class) fails this pin. Pairs with G17-C's
    // `sandbox_handler_args.rs` for the casing-discipline coverage.
    unimplemented!("G17-C wires register_module_bytes napi method + TS engine surface assertion");
}
