//! Edge-case tests for ENGINE-SPEC §4 Invariants 9, 10, and 12.
//!
//! - Invariant 9: determinism classification per operation type
//! - Invariant 10: content-addressed hash per subgraph
//! - Invariant 12: registration-time structural validation (catch-all code)
//!
//! Covers error codes:
//! - `E_INV_DETERMINISM` — a non-deterministic primitive appears in a context
//!   declared deterministic (e.g. EMIT in a read-only view; SANDBOX inside
//!   a deterministic handler).
//! - `E_INV_CONTENT_HASH` — stored subgraph's computed hash ≠ its stored key.
//!   Indicates on-disk corruption or format drift.
//! - `E_INV_REGISTRATION` — catch-all when multiple invariants fire and
//!   no single `E_INV_*` code is precise enough. The `violated_invariants`
//!   list must enumerate every violation.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{ErrorCode, SubgraphBuilder};

#[test]
fn rejects_nondet_in_det_context() {
    // Invariant 9 boundary: SANDBOX is nondeterministic by classification.
    // A handler declared `deterministic: true` (e.g. an IVM-target view)
    // must reject any SANDBOX primitive inside it at registration time.
    let mut sb = SubgraphBuilder::new("det_context_with_sandbox");
    sb.declare_deterministic(true);
    let root = sb.read("r");
    // SANDBOX is type-defined but its executor is Phase 2. Structural
    // validation however still runs (that's the whole point of shipping
    // all 12 primitive types in Phase 1).
    let _sandboxed = sb.sandbox(root, "some-wasm-module");
    let err = sb
        .build_validated()
        .expect_err("SANDBOX in deterministic context must fail");
    assert_eq!(err.code(), ErrorCode::InvDeterminism);
}

#[test]
fn accepts_det_primitives_in_det_context() {
    // Positive boundary pair: READ, WRITE (via CAS), TRANSFORM, BRANCH,
    // ITERATE, CALL, RESPOND, EMIT are all classified so that they can
    // appear in a deterministic context. (EMIT's determinism depends on
    // whether the emitted event is local or remote; the registration
    // check accepts local-only EMIT in deterministic contexts per
    // ENGINE-SPEC §3.)
    let mut sb = SubgraphBuilder::new("det_context_pure");
    sb.declare_deterministic(true);
    let root = sb.read("r");
    let t = sb.transform(root, "$input + 1");
    let _ = sb.respond(t);
    let _sg = sb
        .build_validated()
        .expect("deterministic context with only det primitives must pass");
}

#[test]
fn rejects_content_hash_mismatch() {
    // Invariant 10: stored-but-mis-hashed subgraph. We inject bytes
    // that have been tampered with post-storage — the computed CID
    // no longer matches the key under which the bytes live.
    let sg = SubgraphBuilder::new("will_be_corrupted")
        .read("r")
        .build_validated_for_corruption_test();
    let cid = sg.cid().unwrap();
    let mut bytes = sg.canonical_bytes().unwrap();
    // Flip a single bit. Now the stored bytes no longer hash to `cid`.
    let last = bytes.last_mut().unwrap();
    *last ^= 0x01;

    // The load path verifies the hash; mismatch fires E_INV_CONTENT_HASH.
    let err = benten_eval::Subgraph::load_verified(&cid, &bytes)
        .expect_err("hash mismatch must be detected on load");
    assert_eq!(err.code(), ErrorCode::InvContentHash);

    // Context includes expected and actual.
    assert!(err.expected_cid().is_some());
    assert!(err.actual_cid().is_some());
}

#[test]
fn registration_catch_all_populates_violated_list() {
    // Invariant 12: registration-time catch-all. A subgraph that violates
    // multiple invariants (cycle + fan-out-exceeded) can be reported via
    // E_INV_REGISTRATION with the violated list naming all of them.
    //
    // The checker is free to fail-fast on the first; but if it aggregates,
    // E_INV_REGISTRATION is the code that carries the multi-fault report.
    //
    // We exercise the aggregating-mode behaviour via a test-only API.
    let cap = benten_eval::limits::DEFAULT_MAX_FANOUT;
    let mut sb = SubgraphBuilder::new("multi_violation");
    let root = sb.read("root");

    // Violation 1: cycle.
    sb.add_edge(root, root);
    // Violation 2: fan-out-exceeded.
    for _ in 0..(cap + 1) {
        let _arm = sb.transform(root, "$input");
    }

    let err = sb
        .build_validated_aggregate_all()
        .expect_err("aggregate check must report multiple violations");
    assert_eq!(err.code(), ErrorCode::InvRegistration);

    let violations = err
        .violated_invariants()
        .expect("aggregate path populates list");
    assert!(violations.contains(&1), "Invariant 1 (cycle) must appear");
    assert!(violations.contains(&3), "Invariant 3 (fan-out) must appear");
}

#[test]
fn single_violation_uses_specific_code_not_catch_all() {
    // Partner boundary: when exactly one invariant fires, the error must
    // use the SPECIFIC code (E_INV_CYCLE, E_INV_FANOUT_EXCEEDED, ...),
    // NOT the generic E_INV_REGISTRATION. The catch-all is reserved
    // for aggregate reporting.
    let mut sb = SubgraphBuilder::new("single_violation_cycle_only");
    let root = sb.read("root");
    sb.add_edge(root, root);
    let err = sb.build_validated().expect_err("cycle only");
    assert_eq!(
        err.code(),
        ErrorCode::InvCycle,
        "single-violation path must use the specific code, not E_INV_REGISTRATION"
    );
}
