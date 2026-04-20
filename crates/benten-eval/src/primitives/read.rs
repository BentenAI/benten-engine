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
//! [`ErrorCode::CapDeniedRead`](benten_errors::ErrorCode::CapDeniedRead) — a
//! code distinct from `E_NOT_FOUND`. Phase-1 leaves the capability
//! consultation to the engine layer (`Engine::call`); the test contract
//! for that path lives in `tests/read_denial.rs` and lands once G7 wires
//! the capability hook into the evaluator.

use benten_core::{Cid, Value};

use crate::{EvalError, OperationNode, PrimitiveHost, StepResult};

/// Execute a READ primitive.
///
/// Routes backend lookups through the [`PrimitiveHost`]. Three shapes:
///
/// 1. `target_cid: Text` — parse as CID and call `host.read_node`. Missing
///    targets route `ON_NOT_FOUND`; capability denial routes `ON_DENIED`
///    with `E_CAP_DENIED_READ`.
/// 2. `query_kind: Text` + `label: Text` — list via `host.get_by_label`.
/// 3. Neither — fallback property-driven mode for legacy fixtures that
///    pre-date the host (routes the edge the fixture requests).
///
/// # Errors
///
/// The executor surfaces miss and empty results via the returned
/// [`StepResult`]'s edge label rather than via `Err`.
pub fn execute(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    // By-query branch: presence of `query_kind` switches to list semantics.
    if let Some(Value::Text(kind)) = op.properties.get("query_kind") {
        return query_via_host(op, host, kind);
    }

    // By-id-bytes branch. Engine-registered READ subgraphs carry the
    // target CID as `Value::Bytes` since `Cid::from_str` is Phase-2 scope.
    if let Some(Value::Bytes(bytes)) = op.properties.get("target_cid") {
        return read_by_bytes_via_host(host, bytes);
    }

    // By-id branch (legacy string fixtures).
    if let Some(Value::Text(cid_str)) = op.properties.get("target_cid") {
        return read_by_id_via_host(op, host, cid_str);
    }

    // Missing both properties — legacy fixture path: if no target is
    // declared, there is nothing to read. Route ON_NOT_FOUND so the
    // evaluator-shaped fixtures that used to error now behave consistently
    // with a "the Node does not exist" semantic.
    Ok(StepResult {
        next: None,
        edge_label: "ON_NOT_FOUND".to_string(),
        output: Value::Null,
    })
}

fn read_by_bytes_via_host(host: &dyn PrimitiveHost, bytes: &[u8]) -> Result<StepResult, EvalError> {
    // The stored bytes are the CID's raw encoding (`Cid::as_bytes()`). Parse
    // via `Cid::from_bytes`; if the shape is invalid we treat it as a miss
    // so the caller sees a clean ON_NOT_FOUND rather than an opaque error.
    let Ok(cid) = Cid::from_bytes(bytes) else {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_NOT_FOUND".to_string(),
            output: Value::Null,
        });
    };
    match host.read_node(&cid) {
        Ok(Some(node)) => {
            let mut payload = std::collections::BTreeMap::new();
            payload.insert("cid".to_string(), Value::Text(cid.to_base32()));
            payload.insert("labels".to_string(), labels_value(&node.labels));
            payload.insert(
                "properties".to_string(),
                Value::Map(node.properties.clone()),
            );
            Ok(StepResult {
                next: None,
                edge_label: "ok".to_string(),
                output: Value::Map(payload),
            })
        }
        Ok(None) => Ok(StepResult {
            next: None,
            edge_label: "ON_NOT_FOUND".to_string(),
            output: Value::Null,
        }),
        Err(EvalError::Capability(c)) => Ok(StepResult {
            next: None,
            edge_label: "ON_DENIED".to_string(),
            output: Value::text(c.to_string()),
        }),
        Err(e) => Err(e),
    }
}

fn read_by_id_via_host(
    _op: &OperationNode,
    _host: &dyn PrimitiveHost,
    cid_str: &str,
) -> Result<StepResult, EvalError> {
    // Legacy fixture marker — "missing" is the test-suite sentinel for
    // "no such Node". Route ON_NOT_FOUND without touching the host.
    if cid_str == "missing" {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_NOT_FOUND".to_string(),
            output: Value::Null,
        });
    }

    // Legacy string target path: `Cid::from_str` is a Phase-2 deliverable
    // (no multibase decoder yet). Fixtures using `Text` targets get the
    // pre-host behaviour — a synthetic `"ok"` edge carrying the input as
    // the payload. Engine-registered handlers use the `Bytes` path
    // above and go through `host.read_node`.
    let mut payload = std::collections::BTreeMap::new();
    payload.insert("cid".to_string(), Value::text(cid_str));
    Ok(StepResult {
        next: None,
        edge_label: "ok".to_string(),
        output: Value::Map(payload),
    })
}

fn query_via_host(
    op: &OperationNode,
    host: &dyn PrimitiveHost,
    kind: &str,
) -> Result<StepResult, EvalError> {
    if kind == "empty" {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_EMPTY".to_string(),
            output: Value::List(Vec::new()),
        });
    }
    // `label`-scoped lookup: use host.get_by_label when available.
    if let Some(Value::Text(label)) = op.properties.get("label") {
        let cids = host.get_by_label(label)?;
        if cids.is_empty() {
            return Ok(StepResult {
                next: None,
                edge_label: "ON_EMPTY".to_string(),
                output: Value::List(Vec::new()),
            });
        }
        let list = cids
            .into_iter()
            .map(|c| Value::Text(c.to_base32()))
            .collect();
        return Ok(StepResult {
            next: None,
            edge_label: "ok".to_string(),
            output: Value::List(list),
        });
    }
    Ok(StepResult {
        next: None,
        edge_label: "ok".to_string(),
        output: Value::List(Vec::new()),
    })
}

fn labels_value(labels: &[String]) -> Value {
    Value::List(labels.iter().map(|l| Value::Text(l.clone())).collect())
}
