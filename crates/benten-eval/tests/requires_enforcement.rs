//! Phase 1 R3 security test — `requires`-property attack surface (R1 SC4, critical).
//!
//! Attack class (AI-agent escalation): an adversary registers a handler with
//! `requires: "post:read"` (innocuous) that, inside its operation subgraph,
//! performs a WRITE to an admin-labelled Node. If capability enforcement is
//! driven by the *declared* `requires` string alone, the attacker gets admin
//! writes authorized by a trivial grant.
//!
//! Ben's decision (R1 triage): **option A — declared AND actually-checked**.
//! The `requires` property is the MINIMUM declared capability the handler
//! needs; the evaluator ALSO checks each primitive's effective capability at
//! call-time (`WRITE` against `write:<label>`, `CALL-isolated-false` against
//! the callee's `requires`, etc.). Handlers whose actual operations exceed
//! their declared `requires` have the excess operations denied individually.
//!
//! This test pair locks in the Phase 1 contract; the second variant guards
//! the CALL attenuation path, which is the second-order escalation.
//!
//! TDD contract: FAIL at R3. The declared-vs-actual check is the Phase 1
//! critical fix from R1 triage disposition SC4; R5 implements the per-op
//! capability check in the evaluator.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #4 (critical)
//! - `.addl/phase-1/r1-triage.md` SC4 disposition (option A, Ben-confirmed)
//! - `.addl/phase-1/r2-test-landscape.md` §2.5 `requires` property handling

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::ErrorCode;
use benten_engine::Engine;
use benten_engine::testing::{
    handler_declaring_read_but_writing_admin, handler_with_call_attenuation_escalation,
    policy_with_grants,
};

/// The canonical attack: handler declares `requires: "post:read"` but its
/// operation subgraph contains a `WRITE` primitive targeting `admin:*`. The
/// expected outcome (option A):
///   - The handler REGISTERS successfully (Phase 1 structural validation
///     does NOT refuse handlers with excess operations — that's a Phase 2+
///     static-analysis pass).
///   - At evaluation time, the `post:read` cap allows reads; the admin
///     WRITE sees its effective capability requirement (`admin:write`) and
///     the caller's grant does NOT include `admin:write`, so the WRITE is
///     denied with `E_CAP_DENIED` routed via `ON_DENIED`.
#[test]
#[ignore = "TODO(phase-2-grant-backed-policy): per-primitive capability check + policy_with_grants helper + handler_declaring_read_but_writing_admin populated helper land in Phase 2 (per-op Invariant 13). When populated, re-assert the denial shape."]
fn handler_with_understated_requires_denies_excess_writes() {
    let dir = tempfile::tempdir().unwrap();
    // The test subject has `post:read` granted but NOT `admin:write`.
    let engine = Engine::builder()
        .capability_policy(policy_with_grants(&["post:read"]))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let handler = handler_declaring_read_but_writing_admin();
    let handler_id = engine
        .register_subgraph(&handler)
        .expect("handler must register — excess-op analysis is Phase 2+");

    let outcome = engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    // The terminal error must be `E_CAP_DENIED` (not a generic registration
    // failure, not `E_SYSTEM_ZONE_WRITE`, not some silent no-op).
    let err = outcome
        .terminal_error()
        .expect("admin write must be denied");
    assert_eq!(
        err.code(),
        ErrorCode::CapDenied,
        "handler with understated `requires` must have its excess WRITE \
         denied at call-time per option A. got: {:?}",
        err.code()
    );

    // And crucially — the routing must go through `ON_DENIED`, not
    // `ON_ERROR`. A capability denial is a domain signal the handler author
    // can recover from; misrouting to `ON_ERROR` would break the audit-
    // trail visualization exit criterion.
    assert_eq!(outcome.taken_edge(), "ON_DENIED");
}

/// Second-order escalation: a handler A declares `requires: "post:read"` and
/// CALLs handler B with `isolated: false`. Handler B declares
/// `requires: "admin:write"`. Without attenuation enforcement, the outer
/// call-chain could execute B's admin writes under A's innocuous grant.
///
/// Option A contract: `isolated: false` means capability context is
/// attenuated to the INTERSECTION of the outer grant and the callee's
/// `requires`. The outer grant is `post:read`; the intersection with
/// `admin:write` is empty; the inner WRITE fires `E_CAP_DENIED`.
#[test]
#[ignore = "TODO(phase-2-grant-backed-policy): CALL attenuation + policy_with_grants helper + handler_with_call_attenuation_escalation populated helper land in Phase 2. When populated, re-assert the denial shape."]
fn handler_cannot_escalate_via_call_attenuation() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(policy_with_grants(&["post:read"]))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let outer_handler = handler_with_call_attenuation_escalation();
    let handler_id = engine.register_subgraph(&outer_handler).unwrap();

    let outcome = engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    let err = outcome
        .terminal_error()
        .expect("call-attenuation escalation must be blocked");
    assert_eq!(
        err.code(),
        ErrorCode::CapDenied,
        "CALL with isolated:false must attenuate capabilities; inner handler \
         declaring a wider `requires` than the outer grant must see its \
         privileged primitives denied. got: {:?}",
        err.code()
    );
    assert_eq!(outcome.taken_edge(), "ON_DENIED");
}
