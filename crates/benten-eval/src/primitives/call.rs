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
//! TODO(phase-4-meta — backlog §4.75; CALL depth counter + multiplicative budget
//! propagation): add an `Evaluator.call_depth: usize` counter that
//! increments on CALL entry and decrements on callee terminate, and
//! propagate remaining iteration budget multiplicatively through the
//! CALL boundary. Mini-review findings `g6-cag-5` and the
//! ITERATE/CALL observability concern in `g6-cr-10`. Carried from
//! Phase-2 generic marker.

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
    ) && *elapsed > *timeout
    {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_LIMIT".to_string(),
            output: Value::Null,
        });
    }

    // Phase-3 G21-T1: typed-CALL fork. When `target` starts with the
    // reserved `engine:typed:` namespace, route to the typed-CALL
    // dispatch surface instead of the user handler registry. The
    // typed-CALL registry is closed (10 ops); see `crate::typed_call`
    // for the full enumeration + per-op cap requirements.
    if let Some(Value::Text(target)) = op.properties.get("target")
        && let Some(typed_op_name) = target.strip_prefix(crate::typed_call::TYPED_CALL_PREFIX)
    {
        return execute_typed_call(op, host, typed_op_name);
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

/// Phase-3 G21-T1: typed-CALL dispatch from the CALL primitive.
///
/// Recognises the op name (the trailing segment after
/// [`crate::typed_call::TYPED_CALL_PREFIX`]), validates the input
/// shape ([`crate::typed_call::TypedCallOp::validate_input`]), then
/// routes through [`PrimitiveHost::dispatch_typed_call`] (which the
/// engine impl wires to `benten-id` / `benten-core`).
///
/// The CALL Operation Node carries the typed-CALL input under its
/// `input` property as a `Value::Map`. A missing or non-Map `input`
/// surfaces `E_TYPED_CALL_INVALID_INPUT` per the catalog. A
/// dispatching grant lacking the per-op required cap surfaces
/// `E_TYPED_CALL_CAP_DENIED` (routed via the host's
/// `check_capability` hook).
///
/// Routing:
///   - `Ok(_)` → `"ok"` edge with the op's typed result Value.
///   - `EvalError::TypedCallCapDenied` → `"ON_DENIED"` edge (cap-
///     denial family routing per `routed_edge_label`).
///   - `EvalError::TypedCallUnknownOp` /
///     `EvalError::TypedCallInvalidInput` /
///     `EvalError::TypedCallDispatchError` → returned via `Err` so
///     the evaluator's existing typed-error path produces the
///     `ON_ERROR`-routed terminal step.
fn execute_typed_call(
    op: &OperationNode,
    host: &dyn PrimitiveHost,
    typed_op_name: &str,
) -> Result<StepResult, EvalError> {
    let typed_op = match crate::typed_call::TypedCallOp::parse(typed_op_name) {
        Some(op) => op,
        None => {
            return Err(EvalError::TypedCallUnknownOp {
                op_name: typed_op_name.to_string(),
            });
        }
    };

    // Per-op cap-check via the host. Default-permit under
    // NoAuthBackend; UCAN backend gates per chain claim. Map a
    // generic cap-denial back to a typed `TypedCallCapDenied` so the
    // ErrorCode catalog code is the typed-CALL-specific
    // `E_TYPED_CALL_CAP_DENIED` (not the broader `E_CAP_DENIED`).
    if let Err(e) = host.check_capability(typed_op.required_cap(), None) {
        if matches!(e, EvalError::Capability(_)) {
            return Ok(StepResult {
                next: None,
                edge_label: "ON_DENIED".to_string(),
                output: Value::text(format!(
                    "typed-CALL cap-denied: {} requires {}",
                    typed_op.name(),
                    typed_op.required_cap()
                )),
            });
        }
        return Err(e);
    }

    // Resolve the input. The CALL Operation Node carries the typed-CALL
    // input under `input` as a `Value::Map`. Empty input is permitted
    // for ops whose schema allows `{}` (e.g. `keypair_generate`).
    let default_input = Value::Map(alloc::collections::BTreeMap::new());
    let input_value = op.properties.get("input").unwrap_or(&default_input);

    // Shape-check the input against the op's per-op schema. This
    // happens BEFORE dispatch so a malformed call has zero observable
    // side effect.
    typed_op.validate_input(input_value)?;

    // Host-side dispatch. Returns a Value carrying the typed result
    // (per-op output schema documented at TypedCallOp).
    match host.dispatch_typed_call(typed_op, input_value) {
        Ok(v) => Ok(StepResult {
            next: None,
            edge_label: "ok".to_string(),
            output: v,
        }),
        Err(EvalError::TypedCallCapDenied { op_name, required }) => Ok(StepResult {
            next: None,
            edge_label: "ON_DENIED".to_string(),
            output: Value::text(format!(
                "typed-CALL cap-denied: {op_name} requires {required}"
            )),
        }),
        Err(e) => Err(e),
    }
}

extern crate alloc;
