//! R3 unit tests for G5-B-ii / E12 / phil-1: Invariant-14 — every `TraceStep`
//! carries a non-empty `AttributionFrame`.
//!
//! Plus dual-surface placement pin (structural declaration in
//! `benten_eval::invariants::attribution`; runtime threading in
//! `benten_engine::evaluator_attribution`).
//!
//! TDD red-phase: the attribution threading does not yet fire in
//! `Evaluator::run_with_trace`. Tests fail until G5-B-ii lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.5 E12).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::invariants::attribution;
use benten_eval::{NullHost, TraceStep};

#[test]
fn invariant_14_attribution_every_trace_step() {
    // Run a 5-step handler through the evaluator. Every emitted TraceStep
    // must carry a non-empty attribution field.
    let subgraph = attribution::build_five_step_handler_for_test();
    let trace =
        attribution::run_with_attribution_for_test(&subgraph, &NullHost).expect("trace run");

    assert!(
        !trace.is_empty(),
        "5-step handler must emit at least one trace step"
    );

    for (idx, step) in trace.iter().enumerate() {
        let attr = step
            .attribution()
            .unwrap_or_else(|| panic!("step {idx} missing attribution (Inv-14)"));
        // Non-empty attribution: actor_cid should not be zero-default
        // AND must have been threaded from the handler frame.
        assert!(
            !attr.actor_cid.as_bytes().iter().all(|b| *b == 0)
                || !attr.handler_cid.as_bytes().iter().all(|b| *b == 0),
            "step {idx} attribution must be non-default (got {attr:?})"
        );
        let _ = step; // silence unused-var if TraceStep::attribution is a method on step
    }
}

#[test]
fn invariant_14_missing_attribution_is_registration_error() {
    // A primitive that fails to declare its attribution source at definition
    // time must surface `E_INV_ATTRIBUTION` at registration.
    let subgraph = attribution::build_subgraph_with_undeclared_attribution_for_test();
    let err = attribution::validate_registration(&subgraph)
        .expect_err("undeclared attribution must reject");
    assert_eq!(err.code(), ErrorCode::InvAttribution);
}

#[test]
fn invariant_14_structural_declaration_in_invariants_module() {
    // phil-1 dual-surface placement: structural declaration lives in
    // `benten_eval::invariants::attribution`. The import above exists; the
    // fn-pointer coercion is a compile-time file-location check.
    let fn_ptr: fn(&benten_eval::Subgraph) -> Result<(), benten_eval::EvalError> =
        attribution::validate_registration;
    let _ = fn_ptr;

    // Sanity: the run helper lives alongside the validator in the same module.
    let _trace_ty = std::any::type_name::<Vec<TraceStep>>();
}
