//! Phase 2b R3-B — engine_sandbox public-surface integration tests (G7-C).
//!
//! Pin sources: plan §3 G7-C, dx-r1-2b SANDBOX.
//!
//! G7-C surface posture (dx-optimizer corrected):
//!   - DSL composition surface ONLY: `subgraph(...).sandbox({ module,
//!     manifest? | caps? })`.
//!   - NO top-level `engine.sandbox(...)` user-facing API — would
//!     bypass evaluator + Inv-4 + AttributionFrame plumbing.
//!   - Top-level engine surface for sandbox-related work is exclusively
//!     `engine.installModule(manifest, manifestCid)` /
//!     `engine.uninstallModule(cid)` (G10-B owned).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending — DSL composition E2E"]
fn engine_sandbox_end_to_end_via_dsl_composition_only() {
    // Plan §3 G7-C — register a SubgraphSpec built via the DSL
    // `subgraph('handler').sandbox({ module: cid, manifest: 'compute-basic' })`.
    // engine.call('handler', input) routes through the evaluator,
    // which dispatches the SANDBOX primitive.
    //
    // No top-level `engine.sandbox(...)` API is invoked — the
    // composition is what's tested.
    todo!("R5 G7-C — DSL builder + register + engine.call + assertion");
}

#[test]
#[ignore = "Phase 2b G7-C pending — absence pin"]
fn sandbox_no_top_level_engine_sandbox_call_site_exists() {
    // dx-r1-2b SANDBOX surface — anti-regression: the public Rust
    // engine surface (`benten_engine::Engine`) MUST NOT carry a
    // `sandbox` method. Only `install_module` / `uninstall_module`
    // (G10-B owned) and the internal `execute_sandbox_*` plumbing
    // (private).
    //
    // Test:
    //   - Compile a small fixture that attempts `engine.sandbox(...)`;
    //     assert it FAILS to compile (use `compile_fail` doctest or a
    //     `trybuild` harness).
    //   - Alternative: white-box reflective check on the `Engine`
    //     impl — assert no public method named `sandbox` exists.
    //
    // The absence pin is load-bearing for the dx-r1-2b corrected
    // surface — without it, a future PR could re-introduce the
    // top-level method without explicit review.
    todo!("R5 G7-C — trybuild compile_fail or reflective absence assertion");
}
