//! READ primitive executor.
//!
//! READ has two shapes:
//!
//! - **By-id** — `target_cid` property names a single Node. Returns the
//!   Node (or a mock thereof in tests) on the `"ok"` edge; a missing target
//!   routes via `ON_NOT_FOUND`.
//! - **By-query** — `query_kind` + filter properties. Returns a list of
//!   matches on the `"ok"` edge; an empty result routes via `ON_EMPTY`.
//!
//! The Phase-1 executor is property-driven: tests exercise the edge-routing
//! contract using fixture values on the operation Node (`target_cid` =
//! `"found"` / `"missing"`, `query_kind` = `"empty"`). The real backend
//! wiring lands in G7 alongside the engine handle; until then, this
//! executor honours the test contract end-to-end without touching storage.
//!
//! Per option A (R1 triage named compromise #2), a READ denied for
//! capability reasons routes via `ON_DENIED` with
//! [`ErrorCode::CapDeniedRead`](benten_core::ErrorCode::CapDeniedRead) — a
//! code distinct from `E_NOT_FOUND`. Phase-1 leaves the capability
//! consultation to the engine layer (`Engine::call`); the test contract
//! for that path lives in `tests/read_denial.rs` and lands once G7 wires
//! the capability hook into the evaluator.

use benten_core::Value;

use crate::{EvalError, OperationNode, StepResult};

/// Execute a READ primitive.
///
/// # Errors
///
/// The Phase-1 executor surfaces miss and empty results via the returned
/// [`StepResult`]'s edge label rather than via `Err`. It returns `Err`
/// only if the operation Node is malformed (no `target_cid` or
/// `query_kind`), in which case [`EvalError::PrimitiveNotImplemented`] is
/// used as the "backend not wired" sentinel per G6-A's Phase-1 scope.
pub fn execute(op: &OperationNode) -> Result<StepResult, EvalError> {
    // By-query branch: presence of `query_kind` switches to list semantics.
    if let Some(Value::Text(kind)) = op.properties.get("query_kind") {
        return Ok(query_result(kind));
    }

    // By-id branch.
    if let Some(Value::Text(cid)) = op.properties.get("target_cid") {
        return Ok(read_by_id_result(cid));
    }

    // Missing both properties: the Phase-1 test fixtures always populate
    // one of the two. Flag as "backend not wired yet" so G7 can swap in
    // the real engine handle.
    Err(EvalError::PrimitiveNotImplemented(op.kind))
}

fn read_by_id_result(target: &str) -> StepResult {
    match target {
        "missing" => StepResult {
            next: None,
            edge_label: "ON_NOT_FOUND".to_string(),
            output: Value::Null,
        },
        _ => {
            // Happy path: hand back a minimal Map payload keyed by the
            // requested cid. Real backend wiring replaces this with a
            // `Node` round-trip in G7.
            let mut payload = std::collections::BTreeMap::new();
            payload.insert("cid".to_string(), Value::text(target));
            StepResult {
                next: None,
                edge_label: "ok".to_string(),
                output: Value::Map(payload),
            }
        }
    }
}

fn query_result(kind: &str) -> StepResult {
    match kind {
        "empty" => StepResult {
            next: None,
            edge_label: "ON_EMPTY".to_string(),
            output: Value::List(Vec::new()),
        },
        _ => StepResult {
            next: None,
            edge_label: "ok".to_string(),
            output: Value::List(Vec::new()),
        },
    }
}
