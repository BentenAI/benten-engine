//! Phase 2b R3-B (filled in by R5 G7-B) — Inv-7 sandbox-output
//! registration-time check.
//!
//! Pin sources: plan §3 G7-B + plan §4 + D15 trap-loudly.
//!
//! Tests the **static `output_max_bytes` range check** at registration:
//!   - `output_max_bytes = 0` → reject (no physical meaning).
//!   - `output_max_bytes > max_sandbox_output_bytes` → reject.
//!   - `output_max_bytes` of poisoned shape (`Value::Bytes` /
//!     `Value::Text`) → reject.
//!   - `output_max_bytes` within range → accept.
//!   - SANDBOX node with NO `output_max_bytes` property → accept (runtime
//!     uses engine default).
//!
//! Runtime D15 trap-loudly + D17 PRIMARY behavior is exercised in
//! `invariant_7_runtime.rs` which depends on G7-A's primitives::sandbox
//! executor surface (`CountedSink` integration).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{PrimitiveKind, SubgraphBuilder, Value};
use benten_errors::ErrorCode;
use benten_eval::invariants::sandbox_output::DEFAULT_MAX_SANDBOX_OUTPUT_BYTES;
use benten_eval::subgraph_ext::SubgraphBuilderExt;
use benten_eval::{InvariantConfig, InvariantViolation};

/// Build a single-SANDBOX subgraph and stamp the `output_max_bytes`
/// property on the SANDBOX node via the `set_property_for_test` builder
/// helper (the property surface SANDBOX nodes carry pre-G7-A; G7-A's
/// named-manifest DSL will provide the production-side stamp once
/// merged).
fn build_with_output_max(handler_id: &str, prop: Value) -> SubgraphBuilder {
    let mut b = SubgraphBuilder::new(handler_id);
    let entry = b.read("entry");
    let sb = b.push_primitive("sandbox_outprop", PrimitiveKind::Sandbox);
    b.set_property_for_test(sb, "output_max_bytes", prop);
    b.add_edge(entry, sb);
    let _ = b.respond(sb);
    b
}

#[test]
fn invariant_7_output_max_bytes_zero_rejected_at_registration() {
    let b = build_with_output_max("h_zero", Value::Int(0));
    let err = b
        .build_validated()
        .expect_err("output_max_bytes = 0 must reject");
    assert_eq!(
        *err.kind(),
        InvariantViolation::SandboxOutput,
        "expected SandboxOutput, got {:?}",
        err.kind()
    );
    assert_eq!(
        InvariantViolation::SandboxOutput.code(),
        ErrorCode::InvSandboxOutput
    );
}

#[test]
fn invariant_7_output_max_bytes_above_ceiling_rejected_at_registration() {
    // Default ceiling = DEFAULT_MAX_SANDBOX_OUTPUT_BYTES. Declared value
    // ceiling+1 must reject.
    let over = i64::try_from(DEFAULT_MAX_SANDBOX_OUTPUT_BYTES).unwrap_or(i64::MAX) + 1;
    let b = build_with_output_max("h_over", Value::Int(over));
    let err = b
        .build_validated()
        .expect_err("output_max_bytes > ceiling must reject");
    assert_eq!(*err.kind(), InvariantViolation::SandboxOutput);
}

#[test]
fn invariant_7_output_max_bytes_negative_rejected_at_registration() {
    let b = build_with_output_max("h_neg", Value::Int(-1));
    let err = b
        .build_validated()
        .expect_err("negative output_max_bytes must reject");
    assert_eq!(*err.kind(), InvariantViolation::SandboxOutput);
}

#[test]
fn invariant_7_output_max_bytes_poisoned_shape_rejected_at_registration() {
    // A non-Int shape is a poisoned encoding — same discipline as
    // `signal_shape: Value::Bytes` for WAIT.
    let b = build_with_output_max("h_poisoned", Value::Bytes(vec![1, 2, 3]));
    let err = b
        .build_validated()
        .expect_err("poisoned output_max_bytes shape must reject");
    assert_eq!(*err.kind(), InvariantViolation::SandboxOutput);
}

#[test]
fn invariant_7_output_max_bytes_within_range_accepted() {
    let b = build_with_output_max("h_ok", Value::Int(1024));
    b.build_validated()
        .expect("output_max_bytes = 1024 must register cleanly");
}

#[test]
fn invariant_7_output_max_bytes_at_exact_ceiling_accepted() {
    // The ceiling itself is INCLUSIVE — declaring exactly the maximum is
    // legal.
    let cfg = InvariantConfig::default();
    let exact = i64::try_from(cfg.max_sandbox_output_bytes).unwrap_or(i64::MAX);
    let b = build_with_output_max("h_exact", Value::Int(exact));
    b.build_validated()
        .expect("output_max_bytes = ceiling must register cleanly");
}

#[test]
fn invariant_7_sandbox_without_output_max_bytes_property_accepted() {
    // SANDBOX node that omits `output_max_bytes` registers cleanly; the
    // runtime executor uses the engine default ceiling.
    let mut b = SubgraphBuilder::new("h_omit");
    let entry = b.read("entry");
    let sb = b.sandbox(entry, "test:omit");
    let _ = sb;
    b.build_validated()
        .expect("SANDBOX with no output_max_bytes must register cleanly");
}

#[test]
fn invariant_7_inv_config_default_max_is_16_mib() {
    // Pin the default that backs the registration-time ceiling.
    let cfg = InvariantConfig::default();
    assert_eq!(
        cfg.max_sandbox_output_bytes, DEFAULT_MAX_SANDBOX_OUTPUT_BYTES,
        "D15 default ceiling must be 16 MiB"
    );
    assert_eq!(DEFAULT_MAX_SANDBOX_OUTPUT_BYTES, 16 * 1024 * 1024);
}
