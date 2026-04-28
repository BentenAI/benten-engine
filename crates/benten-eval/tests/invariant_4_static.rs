//! Phase 2b R3-B (filled in by R5 G7-B) — Inv-4 sandbox-depth
//! registration-time check.
//!
//! Pin source: plan §3 G7-B + D20-RESOLVED.
//!
//! Tests the **static SubgraphSpec analysis** path of Inv-4: a subgraph
//! whose longest SANDBOX-bearing chain exceeds
//! `InvariantConfig::max_sandbox_nest_depth` (default 4) is rejected at
//! `SubgraphBuilder::build_validated` time with
//! `InvariantViolation::SandboxDepth` → `ErrorCode::InvSandboxDepth`.
//!
//! This path is purely structural — it does NOT require G7-A's wasmtime
//! runtime; it walks the OperationNode call-graph and counts SANDBOX
//! primitives. The runtime depth check (TRANSFORM-computed targets) is
//! covered by `invariant_4_runtime.rs` which DOES need G7-A's
//! `primitives::sandbox` executor merged.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::SubgraphBuilder;
use benten_errors::ErrorCode;
use benten_eval::subgraph_ext::SubgraphBuilderExt;
use benten_eval::{InvariantConfig, InvariantViolation};

/// Build a subgraph whose call-graph has `n` nested SANDBOX primitives in
/// a single linear chain (sandbox₀ → sandbox₁ → … → sandboxₙ₋₁). The
/// longest SANDBOX-only chain length equals `n`.
///
/// Each SANDBOX gets a distinct `module` property so the OperationNode
/// content-hash differs per node (otherwise two structurally-identical
/// nodes would collapse under deduplication paths in some passes).
fn build_sandbox_chain(handler_id: &str, n: usize) -> SubgraphBuilder {
    let mut b = SubgraphBuilder::new(handler_id);
    let mut prev = b.read("entry");
    for i in 0..n {
        let module = format!("test:sandbox_{i}");
        prev = b.sandbox(prev, &module);
    }
    let _ = b.respond(prev);
    b
}

#[test]
fn invariant_4_sandbox_nest_depth_rejected_at_registration() {
    // Default config max = 4. A 5-sandbox chain trips static analysis.
    let b = build_sandbox_chain("h_depth_5", 5);
    let err = b
        .build_validated()
        .expect_err("depth-5 SANDBOX chain must reject");
    assert_eq!(
        *err.kind(),
        InvariantViolation::SandboxDepth,
        "expected SandboxDepth invariant violation, got {:?}",
        err.kind()
    );
    assert_eq!(
        InvariantViolation::SandboxDepth.code(),
        ErrorCode::InvSandboxDepth
    );
}

#[test]
fn invariant_4_sandbox_nest_depth_accepts_at_default_ceiling() {
    // 4-sandbox chain at the default ceiling registers cleanly.
    let b = build_sandbox_chain("h_depth_4", 4);
    b.build_validated()
        .expect("depth-4 SANDBOX chain must register at default ceiling");
}

#[test]
fn invariant_4_sandbox_no_sandbox_chain_registers() {
    // Sanity: a chain with no SANDBOX primitives is unaffected.
    let mut b = SubgraphBuilder::new("h_no_sandbox");
    let r = b.read("entry");
    let t = b.transform(r, "x");
    let _ = b.respond(t);
    b.build_validated()
        .expect("non-SANDBOX subgraph must register");
}

#[test]
fn invariant_4_single_sandbox_registers_under_default_ceiling() {
    // depth-1 SANDBOX (typical handler shape) registers cleanly.
    let b = build_sandbox_chain("h_depth_1", 1);
    b.build_validated()
        .expect("depth-1 SANDBOX must register under default ceiling");
}

#[test]
fn invariant_4_inv_config_default_max_is_four() {
    // Pin the default that backs the registration-time test above.
    let cfg = InvariantConfig::default();
    assert_eq!(
        cfg.max_sandbox_nest_depth, 4,
        "D20-RESOLVED — default max_sandbox_nest_depth must remain 4"
    );
}
