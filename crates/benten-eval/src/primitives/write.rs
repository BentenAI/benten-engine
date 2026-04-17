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

use benten_core::Value;

use crate::{EvalError, OperationNode, StepResult};

/// Execute a WRITE primitive.
///
/// # Errors
///
/// Returns [`EvalError::PrimitiveNotImplemented`] if the `op` property is
/// absent or carries an unrecognized discriminant. Version-conflict
/// failures (the CAS path) are routed via `ON_CONFLICT` rather than
/// returned as `Err`; see the module docs.
pub fn execute(op: &OperationNode) -> Result<StepResult, EvalError> {
    let mode = match op.properties.get("op") {
        Some(Value::Text(t)) => t.as_str(),
        _ => return Err(EvalError::PrimitiveNotImplemented(op.kind)),
    };

    match mode {
        "create" | "update" | "delete" => Ok(ok_step(mode)),
        "cas" => Ok(cas_step(op)),
        _ => Err(EvalError::PrimitiveNotImplemented(op.kind)),
    }
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
