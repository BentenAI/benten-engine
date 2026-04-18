//! Phase 1 R3 security test — `requires` is checked per-primitive at call-time.
//!
//! Attack class companion to `requires_enforcement.rs`: this file narrows in
//! on the load-bearing CONTRACT (option A, Ben-confirmed in R1 triage SC4):
//! the capability hook is consulted PER PRIMITIVE with the primitive's
//! *effective* capability requirement — NOT a single check against the
//! handler's declared `requires` at entry.
//!
//! If R5 implements the check "once at call entry, against `requires`", the
//! happy-path tests in `requires_enforcement.rs` could pass by accident (the
//! denial still fires because the top-level `requires` doesn't cover the
//! write). This file isolates the contract by building a handler whose
//! *overall* declared `requires` IS satisfied but whose INTERNAL ordering
//! contains a denied operation in the middle.
//!
//! The test asserts the capability hook is called MULTIPLE TIMES — once per
//! capability-requiring primitive, not once per handler call. This is the
//! property that makes the Phase 1 default actually safe for Phase 6 AI
//! agents, not merely nominally safe.
//!
//! TDD contract: FAIL at R3. R5 lands the per-primitive hook in the
//! evaluator.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #4 (critical)
//! - `.addl/phase-1/r1-triage.md` SC4 — option A
//! - `.addl/phase-1/r2-test-landscape.md` §2.5 `requires_checked_at_primitive_not_just_declaration`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_engine::testing::{counting_capability_policy, handler_with_read_write_read_sequence};

/// The handler performs READ → WRITE → READ in sequence. All three
/// operations are under the declared `requires`. The contract says the
/// capability hook must be called THREE times — once per primitive — not
/// once-at-entry.
///
/// Why this matters: if the implementation checks only-at-entry, a future
/// revocation between op 1 and op 3 will not be seen (bad — that's what
/// `capability_revoked_mid_iteration_denies_subsequent_batches` locks in).
/// More immediately, a handler whose declared `requires` is technically
/// satisfiable but whose INTERNAL ops drift from declaration over time
/// (e.g. refactor adds a WRITE) won't be re-audited. Per-primitive checking
/// makes declaration ↔ execution drift observable.
#[test]
#[ignore = "TODO(phase-2-grant-backed-policy): per-primitive capability check + counting_capability_policy real handler + handler_with_read_write_read_sequence populated helper land in Phase 2 (per-op Invariant 13). When populated, assert delta == 3."]
fn requires_checked_at_primitive_not_just_declaration() {
    let dir = tempfile::tempdir().unwrap();
    let policy = counting_capability_policy();
    let counter = policy.call_counter();

    let engine = Engine::builder()
        .capability_policy(Box::new(policy))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let handler = handler_with_read_write_read_sequence();
    let handler_id = engine.register_subgraph(&handler).unwrap();

    let before = counter.load();
    engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper")
        .assert_success();
    let after = counter.load();

    let delta = after - before;
    assert_eq!(
        delta, 3,
        "capability policy must be consulted once per capability-requiring \
         primitive (READ → WRITE → READ = 3 calls). Got {delta}. If this is \
         1, the evaluator is checking only-at-entry — a contract violation \
         that breaks the TOCTOU batching model in `toctou_iteration.rs` and \
         the per-primitive guarantee in `requires_enforcement.rs`."
    );
}
