//! G4-A mini-review M1: a CALL node that names an unregistered callee
//! must be rejected at registration time with `E_INV_REGISTRATION`.
//!
//! The prior fallback silently treated an unresolvable callee as factor
//! 1 (for non-isolated CALL) or `max`-or-1 (for isolated CALL). That
//! under-counted cumulative — an adversarial handler declaring
//! `isolated: true` without registering the callee bypassed Inv-8
//! entirely. The fix makes unknown callee a typed registration error
//! (`InvariantViolation::Registration` → `E_INV_REGISTRATION`), matching
//! the catalog's existing code rather than churning in a new one.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes per R1 triage."
)]

use benten_eval::{ErrorCode, SubgraphBuilder};

#[test]
fn invariant_8_unknown_callee_rejects_at_registration_non_isolated() {
    // Non-isolated CALL into a callee that was NEVER registered.
    // Pre-M1: silent fallback to factor 1, subgraph accepted.
    // Post-M1: rejected with E_INV_REGISTRATION.
    let mut sb = SubgraphBuilder::new("unknown_callee_non_isolated");
    let root = sb.read("input");
    let _call = sb.call_with_isolated(root, "never_registered_callee", false);
    sb.respond(root);
    let err = sb
        .build_validated()
        .expect_err("unknown callee must reject at registration");
    assert_eq!(
        err.code(),
        ErrorCode::InvRegistration,
        "unknown non-isolated callee must fire E_INV_REGISTRATION, got {:?}",
        err.code()
    );
}

#[test]
fn invariant_8_unknown_callee_rejects_at_registration_isolated() {
    // Isolated CALL into a callee that was NEVER registered.
    // Pre-M1: silent fallback to CALL.max or 1; adversarial subgraph
    // declaring `isolated: true` with no registered callee could
    // bypass Inv-8 entirely.
    // Post-M1: rejected with E_INV_REGISTRATION.
    let mut sb = SubgraphBuilder::new("unknown_callee_isolated");
    let root = sb.read("input");
    let _call = sb.call_with_isolated(root, "never_registered_isolated_callee", true);
    sb.respond(root);
    let err = sb
        .build_validated()
        .expect_err("unknown isolated callee must reject at registration");
    assert_eq!(
        err.code(),
        ErrorCode::InvRegistration,
        "unknown isolated callee must fire E_INV_REGISTRATION, got {:?}",
        err.code()
    );
}

#[test]
fn invariant_8_registered_callee_is_accepted() {
    // Sanity check: registering the callee first lets the same subgraph
    // through. This anchors the rejection to the registry-hit path, not
    // to some unrelated structural issue in the builder.
    let callee = "registered_callee_for_m1_sanity";
    benten_eval::register_test_callee(callee, 5);
    let mut sb = SubgraphBuilder::new("known_callee_isolated");
    let root = sb.read("input");
    let _call = sb.call_with_isolated(root, callee, true);
    sb.respond(root);
    sb.build_validated()
        .expect("registered callee must accept (5 ≤ default Inv-8 bound)");
}

/// Runtime companion to the cfg-gate on `register_test_callee` (G4-A
/// mini-review C1). Integration tests run with `cfg(debug_assertions)`
/// active, so the helper is reachable here; in a release build
/// (`debug_assertions = false`, `testing` feature unset) the symbol
/// disappears entirely and adversarial release-mode code cannot pre-seed
/// the registry that Inv-8 consults at registration time.
///
/// This is a compile-time guard, not a runtime one — the closest the
/// test harness can get to asserting the gate without driving a release
/// build from within a test is demonstrating the symbol is reachable
/// here (and documenting the cfg predicate that gates it).
#[test]
fn register_test_callee_is_callable_under_test_cfg() {
    // This compiles iff the cfg gate matches test profiles — which
    // locks in the C1 soundness property: if the gate regressed to
    // always-on `pub`, the test would still compile; if it regressed to
    // `feature = "testing"` only, integration tests would stop
    // compiling and this test would fail first.
    benten_eval::register_test_callee("c1_gate_reachability_probe", 1);
    // No assertion needed — the compile-reachability IS the assertion.
}
