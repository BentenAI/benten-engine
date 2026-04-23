//! Phase 2a G5-B-ii / E12 / phil-1: Invariant-14 — every `TraceStep` carries
//! an `AttributionFrame`.
//!
//! This file owns the **structural / registration-time** half of phil-1's
//! dual-surface resolution. Each primitive-type in a registered subgraph
//! MUST declare whether it consumes attribution. Absence is a registration-
//! time error. See plan §9.9 + §3 G5-B-ii.
//!
//! Runtime threading — stamping the current `AttributionFrame` onto every
//! emitted `TraceStep` — lives in [`crate::evaluator::attribution`].

use benten_core::Value;

use crate::{
    EvalError, InvariantViolation, NullHost, OperationNode, PrimitiveKind, Subgraph, TraceStep,
};

/// Property key that each [`OperationNode`] in a registered subgraph must
/// carry. The value MUST be [`Value::Bool`] — `true` declares the primitive
/// consumes attribution (the Phase-2a default for every kind); `false`
/// opts it out. Absence of the key fails registration with
/// [`crate::ErrorCode::InvAttribution`].
pub const ATTRIBUTION_PROPERTY_KEY: &str = "attribution";

/// Registration-time declaration validator. Rejects a subgraph whose
/// primitives fail to declare their attribution source.
///
/// A primitive declares attribution by setting the [`ATTRIBUTION_PROPERTY_KEY`]
/// property to [`Value::Bool`] on its [`OperationNode`]. Any other type (or
/// absence of the key) is a registration-time failure. The structural shape
/// is validated here; the runtime threader in
/// [`crate::evaluator::attribution`] can then rely on the declaration being
/// well-formed.
///
/// # Errors
/// Returns [`EvalError::Invariant`] carrying [`InvariantViolation::Attribution`]
/// (catalog code `E_INV_ATTRIBUTION`) when any [`OperationNode`] in the
/// subgraph fails to declare attribution.
pub fn validate_registration(subgraph: &Subgraph) -> Result<(), EvalError> {
    for node in subgraph.nodes() {
        if !node_declares_attribution(node) {
            return Err(EvalError::Invariant(InvariantViolation::Attribution));
        }
    }
    Ok(())
}

/// Back-compat shim for the `invariant_14_attribution` test — forwards to
/// [`validate_registration`].
///
/// # Errors
/// Forwards to [`validate_registration`].
pub fn validate(sg: &Subgraph) -> Result<(), EvalError> {
    validate_registration(sg)
}

/// True when `node` carries a well-formed attribution declaration.
fn node_declares_attribution(node: &OperationNode) -> bool {
    matches!(
        node.property(ATTRIBUTION_PROPERTY_KEY),
        Some(Value::Bool(_))
    )
}

/// Test harness: run a 5-step handler with attribution stamping.
///
/// Validates the registration-time declaration, then threads a synthesised
/// [`crate::AttributionFrame`] (derived from the handler id) onto every
/// emitted [`TraceStep`]. The synthesised frame is non-default — all three
/// CIDs are BLAKE3-of-handler-id variants — so the
/// `invariant_14_attribution_every_trace_step` non-zero-attribution
/// assertion fires.
///
/// # Errors
/// Returns [`EvalError::Invariant`] when the subgraph fails
/// [`validate_registration`].
pub fn run_with_attribution_for_test(
    subgraph: &Subgraph,
    host: &NullHost,
) -> Result<Vec<TraceStep>, EvalError> {
    validate_registration(subgraph)?;
    let frame = crate::evaluator::attribution::default_frame_for_subgraph(subgraph);
    crate::evaluator::attribution::thread_over_subgraph(subgraph, &frame, host)
}

/// Test harness: build a 5-primitive handler whose nodes declare their
/// attribution source. Used by
/// `invariant_14_attribution_every_trace_step`.
#[must_use]
pub fn build_five_step_handler_for_test() -> Subgraph {
    let mut sg = Subgraph::new("inv14:five_step");
    for (idx, kind) in [
        PrimitiveKind::Read,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Write,
        PrimitiveKind::Respond,
    ]
    .into_iter()
    .enumerate()
    {
        let id = format!("n{idx}");
        let op =
            OperationNode::new(id, kind).with_property(ATTRIBUTION_PROPERTY_KEY, Value::Bool(true));
        sg = sg.with_node(op);
    }
    sg
}

/// Test harness: build a subgraph whose primitives intentionally omit
/// attribution declaration. `validate_registration` must reject with
/// [`crate::ErrorCode::InvAttribution`]. Used by
/// `invariant_14_missing_attribution_is_registration_error`.
#[must_use]
pub fn build_subgraph_with_undeclared_attribution_for_test() -> Subgraph {
    // First node deliberately omits the attribution property.
    let bad = OperationNode::new("n0_no_attr", PrimitiveKind::Read);
    let declared = OperationNode::new("n1", PrimitiveKind::Respond)
        .with_property(ATTRIBUTION_PROPERTY_KEY, Value::Bool(true));
    Subgraph::new("inv14:undeclared")
        .with_node(bad)
        .with_node(declared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorCode;

    #[test]
    fn five_step_handler_passes_registration() {
        let sg = build_five_step_handler_for_test();
        validate_registration(&sg).expect("declared subgraph must validate");
    }

    #[test]
    fn undeclared_subgraph_surfaces_inv_attribution_code() {
        let sg = build_subgraph_with_undeclared_attribution_for_test();
        let err = validate_registration(&sg).expect_err("undeclared must reject");
        assert_eq!(err.code(), ErrorCode::InvAttribution);
    }

    #[test]
    fn run_threads_non_default_attribution() {
        let sg = build_five_step_handler_for_test();
        let trace = run_with_attribution_for_test(&sg, &NullHost).expect("trace");
        assert_eq!(trace.len(), 5);
        for step in &trace {
            let attr = step.attribution().expect("every step carries attribution");
            assert!(
                !attr.handler_cid.as_bytes().iter().all(|b| *b == 0),
                "handler_cid must be non-default"
            );
        }
    }
}
