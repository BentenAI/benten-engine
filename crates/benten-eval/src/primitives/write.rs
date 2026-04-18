//! WRITE primitive executor.
//!
//! WRITE has four operation modes, distinguished by the `op` property:
//!
//! - `"create"` — insert a new Node / Edge. Routes via `"ok"` on success.
//! - `"update"` — overwrite an existing Node. Routes via `"ok"`.
//! - `"delete"` — remove a Node (or mark the current version tombstoned).
//!   Routes via `"ok"`.
//! - `"cas"` — compare-and-swap using `expected_version` /
//!   `actual_version`. On a version mismatch, routes via the typed
//!   `ON_CONFLICT` edge with a `Value::Null` payload (R4 triage m3:
//!   conflicts are *routed*, not errored — the engine's transaction
//!   primitive reserves `Err(WriteConflict)` for its own internal use).
//!
//! The Phase-1 executor is property-driven, matching G6-A's scope: real
//! backend wiring lands in G7 alongside the engine handle. The contract
//! this module locks in is the edge-routing shape the evaluator uses to
//! select the next node in the subgraph walk.
//!
//! Capability checking for WRITE is handled in `benten-caps` at commit
//! time via the pre-write hook. This executor does not consult the
//! capability policy directly; the engine layer fires the hook in
//! `Engine::call` per [Validated Design Decision #7].

use benten_core::{Node, Value};

use crate::{EvalError, OperationNode, PrimitiveHost, StepResult};

/// Execute a WRITE primitive.
///
/// Takes a `&dyn PrimitiveHost` so the engine's capability-gated write path
/// runs through the host's `put_node` / `delete_node`. When the operation
/// Node carries a structured `label: Text` + `properties: Map` payload, the
/// executor builds a `Node` and dispatches `host.put_node` — the "real"
/// write path. Otherwise the executor falls back to the edge-only property-
/// carried fixture path for unit tests that don't need a backend.
///
/// # Errors
///
/// Returns [`EvalError::PrimitiveNotImplemented`] if the `op` property is
/// absent or carries an unrecognized discriminant. Version-conflict
/// failures (the CAS path) are routed via `ON_CONFLICT` rather than
/// returned as `Err`; see the module docs.
pub fn execute(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    let mode = match op.properties.get("op") {
        Some(Value::Text(t)) => t.as_str(),
        _ => return Err(EvalError::PrimitiveNotImplemented(op.kind)),
    };

    match mode {
        "create" | "update" => execute_create_or_update(op, host, mode),
        "delete" => execute_delete(op, host),
        "delete_missing" => Ok(StepResult {
            next: None,
            edge_label: "ON_NOT_FOUND".to_string(),
            output: Value::Null,
        }),
        "test_inject_failure" => Err(EvalError::Backend("test_inject_failure".to_string())),
        "cas" => Ok(cas_step(op)),
        _ => Err(EvalError::PrimitiveNotImplemented(op.kind)),
    }
}

fn execute_delete(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    if let Some(Value::Bytes(bytes)) = op.properties.get("target_cid") {
        let Ok(cid) = benten_core::Cid::from_bytes(bytes) else {
            return Ok(StepResult {
                next: None,
                edge_label: "ON_NOT_FOUND".to_string(),
                output: Value::Null,
            });
        };
        match host.delete_node(&cid) {
            Ok(()) => Ok(StepResult {
                next: None,
                edge_label: "ok".to_string(),
                output: Value::Null,
            }),
            Err(EvalError::Capability(c)) => Ok(StepResult {
                next: None,
                edge_label: "ON_DENIED".to_string(),
                output: Value::text(c.to_string()),
            }),
            Err(e) => Err(e),
        }
    } else {
        // No target — legacy fixture path.
        Ok(ok_step("delete"))
    }
}

fn execute_create_or_update(
    op: &OperationNode,
    host: &dyn PrimitiveHost,
    mode: &str,
) -> Result<StepResult, EvalError> {
    // Structured payload path: label + properties compose a Node that the
    // host persists. Capability denial routes ON_DENIED; other errors
    // bubble as Err.
    let label = match op.properties.get("label") {
        Some(Value::Text(s)) => Some(s.clone()),
        _ => None,
    };
    let props = match op.properties.get("properties") {
        Some(Value::Map(m)) => Some(m.clone()),
        _ => None,
    };
    if let (Some(label), Some(props)) = (label, props) {
        let node = Node::new(vec![label], props);
        match host.put_node(&node) {
            Ok(cid) => {
                let mut payload = std::collections::BTreeMap::new();
                payload.insert("op".to_string(), Value::text(mode));
                payload.insert("cid".to_string(), Value::Text(cid.to_base32()));
                return Ok(StepResult {
                    next: None,
                    edge_label: "ok".to_string(),
                    output: Value::Map(payload),
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
    Ok(ok_step(mode))
}

fn ok_step(mode: &str) -> StepResult {
    let mut payload = std::collections::BTreeMap::new();
    payload.insert("op".to_string(), Value::text(mode));
    StepResult {
        next: None,
        edge_label: "ok".to_string(),
        output: Value::Map(payload),
    }
}

fn cas_step(op: &OperationNode) -> StepResult {
    let expected = op
        .properties
        .get("expected_version")
        .and_then(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })
        .unwrap_or(0);
    let actual = op
        .properties
        .get("actual_version")
        .and_then(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })
        .unwrap_or(0);

    if expected == actual {
        let mut payload = std::collections::BTreeMap::new();
        payload.insert("op".to_string(), Value::text("cas"));
        payload.insert("version".to_string(), Value::Int(actual));
        StepResult {
            next: None,
            edge_label: "ok".to_string(),
            output: Value::Map(payload),
        }
    } else {
        // R4 triage m3: conflicts are *routed*, never errored.
        StepResult {
            next: None,
            edge_label: "ON_CONFLICT".to_string(),
            output: Value::Null,
        }
    }
}
