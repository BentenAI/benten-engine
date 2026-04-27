//! 12 primitive types defined (E1, G6-A — R2 landscape §2.5 row 1).
//!
//! Each of the 12 PrimitiveKind variants exists, has a determinism
//! classification, and has an error-edge set. Phase 2 primitives (WAIT,
//! STREAM, SUBSCRIBE, SANDBOX) are type-defined so subgraphs containing them
//! survive registration.
//!
//! R3 writer: `rust-test-writer-unit`.
//! Codes fired: `E_PRIMITIVE_NOT_IMPLEMENTED` (via call-time rejection).

#![allow(clippy::unwrap_used)]

use benten_eval::PrimitiveKind;

#[test]
fn all_twelve_primitive_variants_exist() {
    let all = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ];
    assert_eq!(all.len(), 12);
    // Uniqueness (via debug-string equality check).
    use std::collections::HashSet;
    let s: HashSet<_> = all.iter().map(|k| format!("{k:?}")).collect();
    assert_eq!(s.len(), 12);
}

#[test]
fn phase_1_executable_subset_is_eight_primitives() {
    let all = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ];
    // R4 triage (m2): set equality rather than count-plus-containment. The
    // former accepted any 8-subset that happened to include the canonical
    // eight; the latter closes that loophole.
    use std::collections::HashSet;
    let executable: HashSet<_> = all
        .iter()
        .filter(|k| k.is_phase_1_executable())
        .copied()
        .map(|k| format!("{k:?}"))
        .collect();
    let expected: HashSet<String> = [
        "Read",
        "Write",
        "Transform",
        "Branch",
        "Iterate",
        "Call",
        "Respond",
        "Emit",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();
    assert_eq!(
        executable, expected,
        "phase-1 executable set must be exactly the eight canonical primitives"
    );
}

#[test]
fn phase_2_primitives_are_not_phase_1_executable() {
    assert!(!PrimitiveKind::Wait.is_phase_1_executable());
    assert!(!PrimitiveKind::Stream.is_phase_1_executable());
    assert!(!PrimitiveKind::Subscribe.is_phase_1_executable());
    assert!(!PrimitiveKind::Sandbox.is_phase_1_executable());
}

#[test]
fn deterministic_primitives_match_spec() {
    assert!(PrimitiveKind::Read.is_deterministic());
    assert!(PrimitiveKind::Write.is_deterministic());
    assert!(PrimitiveKind::Transform.is_deterministic());
    assert!(PrimitiveKind::Branch.is_deterministic());
    assert!(PrimitiveKind::Iterate.is_deterministic());
    assert!(PrimitiveKind::Call.is_deterministic());
    assert!(PrimitiveKind::Respond.is_deterministic());
}

#[test]
fn non_deterministic_primitives_match_spec() {
    assert!(!PrimitiveKind::Emit.is_deterministic());
    assert!(!PrimitiveKind::Wait.is_deterministic());
    assert!(!PrimitiveKind::Sandbox.is_deterministic());
    assert!(!PrimitiveKind::Subscribe.is_deterministic());
    assert!(!PrimitiveKind::Stream.is_deterministic());
}

#[test]
fn read_primitive_error_edges_include_on_not_found() {
    assert!(PrimitiveKind::Read.error_edges().contains(&"ON_NOT_FOUND"));
    assert!(PrimitiveKind::Read.error_edges().contains(&"ON_EMPTY"));
    assert!(PrimitiveKind::Read.error_edges().contains(&"ON_DENIED"));
}

#[test]
fn write_primitive_error_edges_include_on_conflict() {
    assert!(PrimitiveKind::Write.error_edges().contains(&"ON_CONFLICT"));
    assert!(PrimitiveKind::Write.error_edges().contains(&"ON_DENIED"));
}

#[test]
fn iterate_primitive_error_edges_include_on_limit() {
    assert!(PrimitiveKind::Iterate.error_edges().contains(&"ON_LIMIT"));
}

/// Mini-review finding g6-opl-1: CALL's timeout path routes `ON_LIMIT`
/// (see `primitives/call.rs`), so the structural validator's allowlist
/// must include it or a valid CALL subgraph would be rejected at
/// registration-time edge-label validation.
#[test]
fn call_primitive_error_edges_include_on_limit_denied_error() {
    let edges = PrimitiveKind::Call.error_edges();
    assert!(edges.contains(&"ON_LIMIT"), "CALL must expose ON_LIMIT");
    assert!(edges.contains(&"ON_DENIED"), "CALL must expose ON_DENIED");
    assert!(edges.contains(&"ON_ERROR"), "CALL must expose ON_ERROR");
}

/// Covered by `covers_error_code[E_PRIMITIVE_NOT_IMPLEMENTED]` entry
/// "phase_two_primitives_return_not_implemented_at_call_time".
///
/// Phase-2b G6-A scope update: STREAM + SUBSCRIBE now have real executors
/// (wave-4 G6-A landing), so they no longer return
/// `PrimitiveNotImplemented`. The remaining Phase-2 primitives (`Wait`,
/// `Sandbox`) keep their `PrimitiveNotImplemented` reject posture until G3
/// / G7 land their executors.
#[test]
fn phase_two_primitives_return_not_implemented_at_call_time() {
    use benten_eval::{EvalError, Evaluator, NullHost, OperationNode};

    let mut ev = Evaluator::new();
    for kind in [PrimitiveKind::Wait, PrimitiveKind::Sandbox] {
        let op = OperationNode::new(format!("op_{kind:?}"), kind);
        let err = ev
            .step(&op, &NullHost)
            .expect_err("Phase-2 primitive must error at call time");
        assert!(
            matches!(err, EvalError::PrimitiveNotImplemented(k) if k == kind),
            "expected PrimitiveNotImplemented({kind:?}), got {err:?}"
        );
    }
}
