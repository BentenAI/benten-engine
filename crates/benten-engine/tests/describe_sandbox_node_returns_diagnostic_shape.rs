//! Phase 2b G7-C — `Engine::describe_sandbox_node` accessor surface pin.
//!
//! Pin source: plan §3 G7-C
//! (`tests/describe_sandbox_node_returns_diagnostic_shape` — ts-r4-3
//! accessor surface pin).
//!
//! The accessor is cfg-gated behind `cfg(any(test, feature =
//! "test-helpers"))` per Phase-2a sec-r6r2-02 discipline so it does
//! NOT appear in the production cdylib symbol set unless the consumer
//! opts into the `test-helpers` feature. This file pins the SHAPE of
//! the accessor's return type — the field set must remain stable so
//! the napi bridge + the TS `engine.describeSandboxNode(...)` surface
//! type-check against a fixed contract while G7-A wires the live
//! lookup.
//!
//! The current Phase-2b body of `describe_sandbox_node` returns an
//! `E_SANDBOX_NODE_UNKNOWN` typed error pending G7-A's lookup wiring
//! (see `crates/benten-engine/src/engine_sandbox.rs`). This test
//! therefore pins the ERROR contract — once G7-A merges, the success
//! branch becomes exercisable and the test should be extended to
//! cover the populated-shape path.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(any(test, feature = "test-helpers"))]

use benten_engine::SandboxNodeDescription;

/// `describe_sandbox_node_returns_diagnostic_shape` — plan §3 G7-C row.
///
/// Pins the SHAPE of [`SandboxNodeDescription`] — the field set the
/// napi bridge + the TS `engine.describeSandboxNode(...)` surface
/// depend on. Field renames or removals here are a breaking-change
/// signal that requires a paired napi bridge + TS `types.ts` edit.
#[test]
fn describe_sandbox_node_diagnostic_shape_field_set_is_stable() {
    // Construct an example value — the constructor exercises every
    // field in the struct so a removed field surfaces as a compile
    // error here rather than as a silent drop on the napi/TS side.
    let example = SandboxNodeDescription {
        module_cid: benten_core::testing::canonical_test_node().cid().unwrap(),
        manifest_id: Some("compute-basic".to_string()),
        fuel: 1_000_000,
        wallclock_ms: 30_000,
        output_limit_bytes: 1_048_576,
        fuel_consumed_high_water: None,
        last_invocation_ms: None,
    };

    // Defaults documented in `docs/SANDBOX-LIMITS.md` §2 (D24 + dx-r1-2b-5).
    assert_eq!(example.fuel, 1_000_000, "fuel default per D24");
    assert_eq!(example.wallclock_ms, 30_000, "wallclock_ms default per D24");
    assert_eq!(
        example.output_limit_bytes, 1_048_576,
        "output_limit_bytes default per D15 trap-loudly"
    );
    assert_eq!(
        example.fuel_consumed_high_water, None,
        "fuel_consumed_high_water None until first invocation"
    );
    assert_eq!(
        example.last_invocation_ms, None,
        "last_invocation_ms None until first call returns"
    );
    assert_eq!(example.manifest_id.as_deref(), Some("compute-basic"));
}

/// Companion — the `manifest_id` field MUST accept `None` for the
/// caps-escape-hatch DSL form (`SandboxArgsByCaps`).
#[test]
fn describe_sandbox_node_manifest_id_none_for_caps_escape_hatch() {
    let example = SandboxNodeDescription {
        module_cid: benten_core::testing::canonical_test_node().cid().unwrap(),
        manifest_id: None,
        fuel: 1_000_000,
        wallclock_ms: 30_000,
        output_limit_bytes: 1_048_576,
        fuel_consumed_high_water: Some(42),
        last_invocation_ms: Some(7),
    };
    assert!(
        example.manifest_id.is_none(),
        "caps-escape-hatch DSL form sets manifest_id = None"
    );
    assert_eq!(example.fuel_consumed_high_water, Some(42));
    assert_eq!(example.last_invocation_ms, Some(7));
}
