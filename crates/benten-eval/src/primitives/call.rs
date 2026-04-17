//! CALL primitive executor.
//!
//! CALL invokes another handler subgraph with optional capability
//! attenuation and a bounded timeout. The Phase-1 executor is property-
//! driven (matching the other primitives):
//!
//! - `target: Text` — the handler_id to invoke.
//! - `parent_scope: Text` / `child_scope: Text` — capability-attenuation
//!   scopes. When both are present, the executor performs the subset check
//!   via [`benten_caps::check_attenuation`] and routes `ON_DENIED` on
//!   rejection.
//! - `timeout_ms: Int` / `elapsed_ms: Int` — when elapsed exceeds timeout,
//!   routes `ON_LIMIT`. The real engine handle (G7) supplies `elapsed_ms`
//!   from the evaluator's trace; Phase-1 tests stage both values on the
//!   operation Node.
//! - `isolated: Bool` — whether the callee runs in a fresh binding scope.
//!
//! Phase-1 scope: the executor does not actually invoke a callee subgraph
//! (no engine handle available); it validates the attenuation + timeout
//! preconditions and surfaces the edge-routing decision. G7 wires up the
//! real invocation path.
//!
//! CALL-depth tracking (invariant 8) is handled by the iterative
//! evaluator's stack accounting in G6-C; CALL itself is unaware of depth.
//!
//! TODO(R4b / G7): when G7 wires real callee-subgraph invocation, add an
//! `Evaluator.call_depth: usize` counter that increments on CALL entry
//! and decrements on callee terminate, and propagate remaining iteration
//! budget multiplicatively through the CALL boundary. Mini-review
//! findings `g6-cag-5` and the ITERATE/CALL observability concern in
//! `g6-cr-10`.

use benten_core::Value;

use crate::{EvalError, OperationNode, StepResult};

/// Execute a CALL primitive.
///
/// # Errors
///
/// Does not surface errors via `Err`; attenuation / timeout failures route
/// through the typed error edges `ON_DENIED` / `ON_LIMIT`.
pub fn execute(op: &OperationNode) -> Result<StepResult, EvalError> {
    // Attenuation check.
    if let (Some(Value::Text(parent)), Some(Value::Text(child))) = (
        op.properties.get("parent_scope"),
        op.properties.get("child_scope"),
    ) {
        let parent_scope = match benten_caps::GrantScope::parse(parent) {
            Ok(s) => s,
            Err(_) => return Ok(denied()),
        };
        let child_scope = match benten_caps::GrantScope::parse(child) {
            Ok(s) => s,
            Err(_) => return Ok(denied()),
        };
        if benten_caps::check_attenuation(&parent_scope, &child_scope).is_err() {
            return Ok(denied());
        }
    }

    // Timeout check.
    if let (Some(Value::Int(timeout)), Some(Value::Int(elapsed))) = (
        op.properties.get("timeout_ms"),
        op.properties.get("elapsed_ms"),
    ) {
        if *elapsed > *timeout {
            return Ok(StepResult {
                next: None,
                edge_label: "ON_LIMIT".to_string(),
                output: Value::Null,
            });
        }
    }

    // Happy path: the evaluator (G7) threads the callee's RESPOND payload
    // through. Phase-1 surface hands back a null placeholder.
    Ok(StepResult {
        next: None,
        edge_label: "ok".to_string(),
        output: Value::Null,
    })
}

fn denied() -> StepResult {
    StepResult {
        next: None,
        edge_label: "ON_DENIED".to_string(),
        output: Value::Null,
    }
}
