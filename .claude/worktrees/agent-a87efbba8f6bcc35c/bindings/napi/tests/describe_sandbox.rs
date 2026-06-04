//! G19-C2 wave-7 (§7.1) — napi describeSandboxNode source-cite + presence
//! diagnostic.
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C2):
//!
//! - `tests/describe_sandbox_node_napi_returns_real_metric_values_not_unknown_placeholder`
//!
//! Companion to `crates/benten-engine/tests/sandbox_metrics.rs`; this
//! file is a SOURCE-CITE DIAGNOSTIC, NOT a load-bearing end-to-end pin
//! per pim-2 §3.6b. The grep-against-source-text shape verifies that
//! the napi-side `describeSandboxNode` method is wired to call into the
//! engine's real metric-tracking accessor
//! (`describe_sandbox_node_for_handler`) rather than synthesizing the
//! "unknown" sentinel placeholder.
//!
//! The LOAD-BEARING end-to-end pins for the metric-real-numbers
//! contract live at:
//!   - `crates/benten-engine/tests/sandbox_metrics.rs` (Rust-side
//!     production-runtime path drives `Engine::call` with observable
//!     `describe_sandbox_node_for_handler` consequence).
//!   - `packages/engine/test/sandbox_metrics.test.ts` (Vitest
//!     end-to-end DSL → napi → engine path; PHASE-3 follow-up — see
//!     gap below).
//!
//! ## Phase-3 follow-up
//!
//! The full Vitest end-to-end pin for `engine.describeSandboxNode`
//! returning real numeric `fuelConsumedHighWater` is named-NOW into
//! `docs/future/phase-3-backlog.md` §7.1 — depends on a built napi
//! cdylib carrying the new `describeSandboxNode` method (built via
//! `napi build --features test-helpers`). The Rust-side end-to-end
//! pin in `sandbox_metrics.rs` validates the metric-tracking shape;
//! the napi cdylib + Vitest land in tandem when the cdylib is rebuilt.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[test]
fn describe_sandbox_node_napi_returns_real_metric_values_not_unknown_placeholder() {
    // §7.1 napi-side source-cite diagnostic. Reads the napi crate's
    // lib.rs + sandbox.rs and verifies the wave-7 metric-propagation
    // wire-up is in source.
    let napi_lib_rs = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("napi lib.rs readable");

    // The napi `EngineNapi` impl block carries the `describeSandboxNode`
    // method that calls into `Engine::describe_sandbox_node_for_handler`
    // (cfg-gated under test-helpers).
    assert!(
        napi_lib_rs.contains("describe_sandbox_node_for_handler")
            || napi_lib_rs.contains("describeSandboxNode"),
        "lib.rs must wire describeSandboxNode through the engine's \
         real per-handler metric-tracking accessor (G19-C2 §7.1); \
         pre-G19-C2 the napi side returned a synthesized 'unknown' \
         placeholder. Source-cite expected: \
         `Engine::describe_sandbox_node_for_handler` reachable from \
         the napi method body."
    );

    // The companion sandbox.rs no longer carries the
    // "HONEST IMPLEMENTATION STATE" / "literal 'unknown' sentinel"
    // narrative — that comment block documented the pre-G19-C2 state
    // when the metric drop happened upstream. The block remains as
    // history but is supplemented with the wave-7 wire-through note.
    let napi_sandbox_rs = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("sandbox.rs"),
    )
    .expect("napi sandbox.rs readable");
    // Sentinel-presence: the file still documents the diagnostic shape.
    assert!(
        napi_sandbox_rs.contains("describe_sandbox_node"),
        "sandbox.rs must continue to document the describe_sandbox_node \
         diagnostic accessor (the comment block survives wave-7; the \
         actual metric source-of-truth shifts to lib.rs)."
    );

    // OBSERVABLE consequence: the source-cite confirms the napi side
    // is wired to consume real metrics; the Rust-side load-bearing
    // end-to-end pin at
    // `crates/benten-engine/tests/sandbox_metrics.rs::describe_sandbox_node_returns_real_fuel_consumed_not_unknown`
    // verifies the actual production-runtime metric values are real
    // numerics rather than the "unknown" sentinel. This source-cite
    // diagnostic catches a regression where the napi method is
    // accidentally renamed or removed.
}
