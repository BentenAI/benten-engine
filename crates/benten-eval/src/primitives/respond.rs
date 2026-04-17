//! RESPOND primitive executor.
//!
//! RESPOND is a terminal leaf: it returns the handler's user-facing
//! payload (status + body) and signals the evaluator to stop walking the
//! subgraph. Per ENGINE-SPEC §5, a RESPOND Node has no outgoing
//! evaluator edges; the evaluator observes `next == None` on the
//! [`StepResult`] and unwinds.
//!
//! The payload shape is a `Value::Map` with at least a `body` entry. When
//! the operation Node supplies a `status` property, it's mirrored into
//! the payload so handlers can return HTTP-like (status, body) tuples.
//! The map is deliberately flexible — different handler shapes (HTTP,
//! subgraph CALL return values, event-handler fire-and-forget
//! acknowledgements) reuse this same wrapper.
//!
//! RESPOND advertises its terminal-ness via the `"terminal"` edge label.
//! The evaluator's stack unwind logic (G6-C) keys off that label to pop
//! frames; G7's engine-level `Engine::call` wraps the final `StepResult`
//! into a user-facing `Outcome` with the payload surfaced on
//! `outcome.response()`.

use std::collections::BTreeMap;

use benten_core::Value;

use crate::{EvalError, OperationNode, StepResult};

/// Execute a RESPOND primitive.
///
/// # Errors
///
/// RESPOND does not currently surface any error variants; the function
/// signature preserves the dispatcher shape used by the other
/// executors.
pub fn execute(op: &OperationNode) -> Result<StepResult, EvalError> {
    let mut payload = BTreeMap::new();

    if let Some(status) = op.properties.get("status") {
        payload.insert("status".to_string(), status.clone());
    }
    if let Some(body) = op.properties.get("body") {
        payload.insert("body".to_string(), body.clone());
    }

    Ok(StepResult {
        next: None,
        edge_label: "terminal".to_string(),
        output: Value::Map(payload),
    })
}
