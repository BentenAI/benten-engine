//! Phase 1 R3 security test — UCAN stub error routing (R1 major #10).
//!
//! Attack class: operator misconfigures `UCANBackend` in Phase 1 believing
//! it's usable (reasonable, because the crate exposes the type and the docs
//! name UCAN as the Phase 3 plan). Every write fails with
//! `CapError::NotImplemented`. The security-auditor named three distinct
//! risks:
//!
//! 1. The error message must NAME the phase (Phase 3) and name an alternative
//!    (NoAuthBackend or a custom `CapabilityPolicy`) so the operator reaches
//!    for the right lever, not "my capability policy is rejecting me".
//! 2. The code must be a DISTINCT `E_CAP_NOT_IMPLEMENTED`, not catch-alled
//!    into `E_CAP_DENIED`. A deny vs. not-implemented distinction is the
//!    difference between "check your grants" and "pick a different backend".
//! 3. In the evaluator, the error must route to `ON_ERROR` — NOT `ON_DENIED`.
//!    Routing a configuration error to the denial path would make Phase 3
//!    operators debug in the wrong direction (they'd audit their grants when
//!    the real problem is backend selection).
//!
//! TDD contract: FAIL at R3 — `UCANBackend`, `CapError::NotImplemented`, the
//! error-code mapping, and the evaluator routing are all Phase 1 deliverables
//! owned by P4 + E3. R5 lands them.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #10 (major)
//! - `.addl/phase-1/r1-triage.md` UCAN-stub disposition
//! - `docs/ERROR-CATALOG.md` `E_CAP_NOT_IMPLEMENTED`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, CapabilityPolicy, UCANBackend, WriteContext};
use benten_core::ErrorCode;

/// The operator-hostile baseline: UCANBackend must cleanly error, not panic.
#[test]
fn ucan_stub_errors_cleanly() {
    let backend = UCANBackend::new();
    let ctx = WriteContext::synthetic_for_test();
    let err = backend
        .check_write(&ctx)
        .expect_err("UCANBackend in Phase 1 must reject all writes");
    assert!(matches!(err, CapError::NotImplemented { .. }));
}

/// The message must name Phase 3 AND an alternative backend the operator can
/// switch to. An opaque "not implemented" reads as a bug report; naming the
/// phase reads as a configuration pointer.
#[test]
fn ucan_stub_error_message_names_phase_and_alternative() {
    let backend = UCANBackend::new();
    let ctx = WriteContext::synthetic_for_test();
    let err = backend.check_write(&ctx).unwrap_err();
    let msg = err.to_string();

    // Phase 3 is the scheduled landing phase for UCAN per `CLAUDE.md` +
    // `docs/FULL-ROADMAP.md`. Operators scanning the message must see it.
    assert!(
        msg.contains("Phase 3"),
        "message must explicitly name Phase 3; got: {msg}"
    );

    // Operators need to know what to configure instead. `NoAuthBackend` is the
    // named alternative in the R1 triage disposition. If the phrasing changes,
    // update both the message and this assertion in lockstep.
    assert!(
        msg.contains("NoAuthBackend"),
        "message must name NoAuthBackend as the interim alternative; got: {msg}"
    );
}

/// The error code must be `E_CAP_NOT_IMPLEMENTED` — DISTINCT from the
/// `E_CAP_DENIED` path. Catch-alling into `Denied` would make the operator
/// audit their grants instead of fixing their backend choice.
#[test]
fn ucan_stub_error_code_is_distinct_from_denied() {
    let backend = UCANBackend::new();
    let ctx = WriteContext::synthetic_for_test();
    let err = backend.check_write(&ctx).unwrap_err();
    assert_eq!(err.code(), ErrorCode::CapNotImplemented);
    assert_ne!(err.code(), ErrorCode::CapDenied);
    assert_ne!(err.code(), ErrorCode::CapDeniedRead);
}

/// The evaluator routing test — critical piece. When a handler in an
/// evaluator with UCANBackend attempts a WRITE, the resulting error MUST
/// route through the `ON_ERROR` typed edge, not `ON_DENIED`. This prevents
/// the misdiagnosis where operators see a config error as an auth failure.
///
/// NOTE: this test depends on the full evaluator. It is defined here (rather
/// than in `benten-eval/tests/`) because the routing contract is the cap
/// crate's responsibility to NAME — the eval crate's responsibility is to
/// honor it. The test wires a minimal handler through `Engine` and asserts
/// the routed edge.
#[test]
fn ucan_stub_error_routes_to_ON_ERROR_not_ON_DENIED() {
    use benten_engine::Engine;
    use benten_engine::testing::{minimal_write_handler, route_of_error};

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(Box::new(UCANBackend::new()))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let handler = minimal_write_handler();
    let handler_id = engine.register_subgraph(&handler).unwrap();

    let result = engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper");
    let taken_edge = route_of_error(&result);

    assert_eq!(
        taken_edge, "ON_ERROR",
        "UCANBackend::NotImplemented must route via ON_ERROR — routing it via \
         ON_DENIED would make operators audit their grants instead of changing \
         their backend. Got: {taken_edge}"
    );
}
