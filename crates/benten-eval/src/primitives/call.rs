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
//! Named Compromise #1 (CALL-entry half): the executor consults
//! `PrimitiveHost::check_capability` before attenuation / timeout /
//! dispatch. A grant revoked between the outer handler's registration
//! and the CALL entry is surfaced through the `ON_DENIED` typed edge
//! with the policy's error code in the edge payload — same shape as a
//! mid-iteration revocation observed at an ITERATE batch boundary.
//!
//! TODO(phase-2): add an `Evaluator.call_depth: usize` counter that
//! increments on CALL entry and decrements on callee terminate, and
//! propagate remaining iteration budget multiplicatively through the
//! CALL boundary. Mini-review findings `g6-cag-5` and the ITERATE/CALL
//! observability concern in `g6-cr-10`.

use benten_core::{Node, Value};

use crate::{EvalError, OperationNode, PrimitiveHost, StepResult};

/// Execute a CALL primitive.
///
/// Takes `&dyn PrimitiveHost` so the engine can recursively dispatch the
/// callee handler via `host.call_handler`. Attenuation and timeout remain
/// property-driven (R3 tests stage both on the Node).
///
/// # Errors
///
/// Does not surface errors via `Err`; attenuation / timeout failures route
/// through the typed error edges `ON_DENIED` / `ON_LIMIT`.
pub fn execute(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    // Compromise #1 closure (CALL-entry cap refresh). Before any
    // attenuation or dispatch work, consult the configured capability
    // policy so a grant revoked between the outer handler's registration
    // and the CALL entry is observed immediately — not deferred to the
    // callee's first per-commit check. The `required` string prefers the
    // declared `child_scope` (the scope the callee will run under) and
    // falls back to `requires` or `"call"` so a host that keys off a
    // specific scope sees the most precise identifier available.
    let required_scope: String = match (
        op.properties.get("child_scope"),
        op.properties.get("requires"),
    ) {
        (Some(Value::Text(s)), _) => s.clone(),
        (_, Some(Value::Text(s))) => s.clone(),
        _ => "call".to_string(),
    };
    if let Err(EvalError::Capability(c)) = host.check_capability(&required_scope, None) {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_DENIED".to_string(),
            output: Value::text(c.to_string()),
        });
    }

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

    // Dispatch through the host when a `target` + `op` are staged. Real
    // handler-to-handler dispatch. Capability denial routes ON_DENIED.
    if let (Some(Value::Text(target)), Some(Value::Text(callee_op))) =
        (op.properties.get("target"), op.properties.get("call_op"))
    {
        let input = Node::empty();
        match host.call_handler(target, callee_op, input) {
            Ok(v) => {
                return Ok(StepResult {
                    next: None,
                    edge_label: "ok".to_string(),
                    output: v,
                });
            }
            Err(EvalError::Capability(c)) => {
                return Ok(StepResult {
                    next: None,
                    edge_label: "ON_DENIED".to_string(),
                    output: Value::text(c.to_string()),
                });
            }
            Err(e) => return Err(e),
        }
    }

    // Legacy no-target path: Phase-1 fixtures that only stage attenuation /
    // timeout properties still work — they hit the happy path with a null
    // placeholder.
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
