//! Phase 2a G5-B-ii: runtime attribution threading for Invariant 14.
//!
//! Structural/registration-time declaration lives in
//! [`crate::invariants::attribution`]. This module owns the **runtime** side
//! of phil-1's dual-surface resolution: every [`TraceStep::Step`] emitted by
//! the evaluator carries the [`AttributionFrame`] on top of the current
//! attribution-frame stack.
//!
//! Policy-driven: the stamping is primitive-type agnostic. READ, WRITE,
//! TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT all pass through the same
//! threader so adding a new primitive type does not require opening every
//! executor file. See plan §9.9 (dual-surface resolution) + §3 G5-B-ii.

use benten_core::{Cid, Value};

use crate::{AttributionFrame, EvalError, NullHost, OperationNode, Subgraph, TraceStep};

/// Runtime attribution threader. Given a [`Subgraph`] plus the current
/// [`AttributionFrame`] (the top of the evaluator's frame stack), walks the
/// subgraph and emits one [`TraceStep::Step`] per primitive node with the
/// frame stamped in.
///
/// Phase-2a contract: policy-driven — every primitive that declares
/// `consumes_attribution` (validated at registration by
/// [`crate::invariants::attribution::validate_registration`]) is threaded
/// identically. Boundary variants (`SuspendBoundary` / `ResumeBoundary` /
/// `BudgetExhausted`) do not yet carry `attribution` in the R3 red-phase
/// shape-pin; Phase-2b adds that when the plan §5 "required on every
/// variant" contract is ratified against the frozen shape-pin tests.
///
/// # Errors
/// Returns [`EvalError::Invariant`] carrying
/// [`crate::InvariantViolation::Attribution`] if the subgraph's registration
/// was skipped and an undeclared primitive is encountered at stamp time.
pub fn thread_over_subgraph(
    subgraph: &Subgraph,
    frame: &AttributionFrame,
    _host: &NullHost,
) -> Result<Vec<TraceStep>, EvalError> {
    // Registration-time guard: the public entry point
    // (`invariants::attribution::run_with_attribution_for_test`) validates
    // first. This path is a belt-and-suspenders re-check so a direct caller
    // into `thread_over_subgraph` also fails loudly on an undeclared
    // primitive rather than silently emitting `attribution = None`.
    crate::invariants::attribution::validate_registration(subgraph)?;

    let mut trace: Vec<TraceStep> = Vec::with_capacity(subgraph.nodes().len());
    for node in subgraph.nodes() {
        trace.push(stamp_step(node, frame));
    }
    Ok(trace)
}

/// Build a single `TraceStep::Step` row for `node`, stamping the supplied
/// `frame`. Outputs default to `Value::Null` — the stamper is not running
/// the primitive; it synthesises the trace-row shape so Inv-14 can assert
/// attribution is present on every emitted row.
pub(crate) fn stamp_step(node: &OperationNode, frame: &AttributionFrame) -> TraceStep {
    TraceStep::Step {
        node_id: node.id.clone(),
        duration_us: 1,
        inputs: Value::Null,
        outputs: Value::Null,
        error: None,
        attribution: Some(frame.clone()),
    }
}

/// Construct a canonical non-default `AttributionFrame` for registered
/// handlers whose trace is threaded through this module. Phase-2a derives
/// all three CIDs deterministically from the handler id so tests can assert
/// "non-default" without wiring a principal registry.
#[must_use]
pub(crate) fn default_frame_for_subgraph(subgraph: &Subgraph) -> AttributionFrame {
    let handler_cid = handler_id_to_cid(subgraph.handler_id());
    AttributionFrame {
        actor_cid: synthetic_cid(b"actor:", subgraph.handler_id()),
        handler_cid,
        capability_grant_cid: synthetic_cid(b"grant:", subgraph.handler_id()),
    }
}

fn handler_id_to_cid(handler_id: &str) -> Cid {
    synthetic_cid(b"handler:", handler_id)
}

fn synthetic_cid(prefix: &[u8], id: &str) -> Cid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prefix);
    hasher.update(id.as_bytes());
    Cid::from_blake3_digest(*hasher.finalize().as_bytes())
}
