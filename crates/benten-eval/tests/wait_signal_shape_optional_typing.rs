//! Edge-case tests: WAIT signal-shape optional typing (DX addendum).
//!
//! R2 landscape §2.5.3 row "WAIT signal-shape optional typing".
//!
//! The WAIT signal variant may optionally declare a `signal_shape` (a
//! Value-shape schema) that constrains the `signal_value` accepted at
//! resume. If present and the resume-time value doesn't match, the
//! evaluator surfaces a typed error (`E_INV_REGISTRATION`-class for shape
//! mismatch routed through `ON_ERROR`). If absent, any Value is accepted.
//!
//! Concerns pinned:
//! - `signal_shape` absent → resume accepts any Value payload.
//! - `signal_shape` present + value matches → resume succeeds.
//! - `signal_shape` present + value shape mismatch → typed error at resume.
//! - Decode-failure-not-panic: an ill-formed `signal_shape` property rejects
//!   at `build_validated`, not at resume.
//!
//! R3 red-phase contract: R5 (G3-B) lands `SubgraphBuilder::wait_signal_typed`
//! plus shape-validation at resume. Tests compile; they fail because the API
//! does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes per R1 triage."
)]

use benten_core::Value;
use benten_errors::ErrorCode;
use benten_eval::{
    EvalContext, MockTimeSource, Outcome, SignalShape, SubgraphBuilder, WaitResumeSignal,
};
use std::time::Duration;

fn subgraph_with_optional_shape(shape: Option<SignalShape>) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new("wait_shape_opt");
    let start = sb.read("x");
    let w = match shape {
        Some(s) => sb.wait_signal_typed(start, "go", s),
        None => sb.wait_signal(start, "go"),
    };
    sb.respond(w);
    sb.build_validated().expect("validation")
}

#[test]
fn wait_signal_shape_defaults_untyped_accepts_any_value() {
    // No shape declared → resume accepts any Value payload.
    let sg = subgraph_with_optional_shape(None);
    let clock = MockTimeSource::at(Duration::ZERO);
    let mut ctx = EvalContext::with_clock(clock);
    let handle = match benten_eval::evaluate(&sg, &mut ctx, Value::unit()) {
        Outcome::Suspended(h) => h,
        other => panic!("must suspend, got {other:?}"),
    };
    // Arbitrary payload shape — Int.
    let result = benten_eval::resume(
        &sg,
        &mut ctx,
        handle,
        WaitResumeSignal::signal("go", Value::Int(42)),
    );
    assert!(
        matches!(result, Outcome::Complete(_)),
        "untyped shape must accept any Value, got {result:?}"
    );
}

#[test]
fn wait_signal_shape_validates_against_schema_when_set() {
    // Shape declares `Map<{ count: Int }>`; resume with a matching-shape
    // value completes.
    let shape = SignalShape::map_of([("count", SignalShape::int())]);
    let sg = subgraph_with_optional_shape(Some(shape));

    let clock = MockTimeSource::at(Duration::ZERO);
    let mut ctx = EvalContext::with_clock(clock);

    let handle = match benten_eval::evaluate(&sg, &mut ctx, Value::unit()) {
        Outcome::Suspended(h) => h,
        other => panic!("must suspend, got {other:?}"),
    };
    let matching = Value::map_of([("count", Value::Int(7))]);
    let result = benten_eval::resume(
        &sg,
        &mut ctx,
        handle,
        WaitResumeSignal::signal("go", matching),
    );
    assert!(
        matches!(result, Outcome::Complete(_)),
        "matching-shape value must complete, got {result:?}"
    );
}

#[test]
#[ignore = "Phase-2b: runtime signal-shape check not yet implemented. WaitSignalShapeMismatch is shape-reserved per docs/ERROR-CATALOG.md (reachability: ignore annotation in Wave-3b EXPECTED-RED-CLOSEOUT). Un-ignore when the runtime path lands."]
fn wait_signal_shape_mismatch_fires_typed_error_routed_on_error() {
    // Shape declares `Int`; resume with `Text` shape mismatches → typed
    // error, routed through ON_ERROR.
    let shape = SignalShape::int();
    let sg = subgraph_with_optional_shape(Some(shape));

    let clock = MockTimeSource::at(Duration::ZERO);
    let mut ctx = EvalContext::with_clock(clock);
    let handle = match benten_eval::evaluate(&sg, &mut ctx, Value::unit()) {
        Outcome::Suspended(h) => h,
        other => panic!("must suspend"),
    };
    let result = benten_eval::resume(
        &sg,
        &mut ctx,
        handle,
        WaitResumeSignal::signal("go", Value::text("not_an_int")),
    );
    let err = match result {
        Outcome::Err(e) => e,
        other => panic!("shape mismatch must fail, got {other:?}"),
    };
    // Shape-mismatch is a runtime-registration-boundary error.
    assert_eq!(
        err.code(),
        ErrorCode::InvRegistration,
        "signal_shape mismatch must fire E_INV_REGISTRATION (runtime shape check), got {:?}",
        err.code()
    );
    assert_eq!(
        err.routed_edge_label(),
        Some("ON_ERROR"),
        "shape-mismatch must route via ON_ERROR, not ON_DENIED"
    );
}

#[test]
fn wait_signal_shape_malformed_at_build_rejects_early() {
    // If a builder inserts a malformed SignalShape (e.g. directly mutating
    // the WAIT Node's `signal_shape` property to non-schema bytes), the
    // build_validated path must reject at registration, not at resume.
    let mut sb = SubgraphBuilder::new("wait_shape_malformed");
    let start = sb.read("x");
    let w = sb.wait_signal(start, "go");
    // Directly poison the `signal_shape` property with a non-schema Value.
    sb.set_property_for_test(w, "signal_shape", Value::Bytes(vec![0xff, 0xee]));
    sb.respond(w);

    let err = sb
        .build_validated()
        .expect_err("malformed signal_shape must fail at build_validated");
    assert_eq!(err.code(), ErrorCode::InvRegistration);
}
