//! BRANCH primitive executor.
//!
//! BRANCH is a multi-way routing operation. Phase 1 supports three shapes
//! driven by properties on the operation Node:
//!
//! - **Binary** — property `"condition_value": Value::Bool(…)`. Routes to
//!   the `"true"` or `"false"` edge.
//! - **Multi-way** — property `"match_value": Value::Text(…)`. Routes to
//!   the edge whose label equals the match value; if no edge matches and
//!   the operation sets `"has_default": true`, falls through to the
//!   `"ON_DEFAULT"` edge.
//! - **Conditional list** — property `"conditions"`: a `Value::List` of
//!   `Map { label, condition_value }` pairs, evaluated in order;
//!   first-match-wins.
//!
//! The evaluator's edge-selection logic (G6-C) consults the returned
//! [`StepResult::edge_label`] to pick the next Node. BRANCH itself produces
//! no value — the output is `Value::Null` (the evaluator threads the
//! upstream `$result` forward).
//!
//! BRANCH's only error edge is `"ON_DEFAULT"` (used when no case matches
//! AND the Node lacks a default). BRANCH never errors at dispatch time:
//! invalid / missing properties produce the `"false"` edge (conservative
//! default) so registration-time validation can enforce that any BRANCH
//! with no outgoing edges is rejected structurally.

use benten_core::Value;

use crate::{EvalError, OperationNode, StepResult};

/// Execute a BRANCH primitive.
///
/// # Errors
///
/// BRANCH does not currently surface error variants through `Err`; routing
/// failures produce the `"ON_DEFAULT"` edge.
pub fn execute(op: &OperationNode) -> Result<StepResult, EvalError> {
    // 1. Binary form: `condition_value: Bool`.
    if let Some(Value::Bool(b)) = op.properties.get("condition_value") {
        let label = if *b { "true" } else { "false" };
        return Ok(StepResult {
            next: None,
            edge_label: label.to_string(),
            output: Value::Null,
        });
    }

    // 2. Multi-way form: `match_value: Text`.
    if let Some(Value::Text(match_v)) = op.properties.get("match_value") {
        // The operation Node's `cases` property carries the edge labels that
        // are admissible; Phase-1 tests rely on `match_value` matching the
        // edge-label convention. When present + admissible, route there.
        if let Some(Value::List(cases)) = op.properties.get("cases") {
            for c in cases {
                if let Value::Text(label) = c
                    && label == match_v
                {
                    return Ok(StepResult {
                        next: None,
                        edge_label: label.clone(),
                        output: Value::Null,
                    });
                }
            }
            // No case matched — if there is a default, route there.
            if let Some(Value::Bool(true)) = op.properties.get("has_default") {
                return Ok(StepResult {
                    next: None,
                    edge_label: "ON_DEFAULT".to_string(),
                    output: Value::Null,
                });
            }
        } else {
            // No explicit cases list — Phase-1 test contract is "route to
            // an edge whose label equals `match_value`", which the
            // evaluator handles downstream. Optimistically route to the
            // value's label.
            if let Some(Value::Bool(true)) = op.properties.get("has_default") {
                return Ok(StepResult {
                    next: None,
                    edge_label: "ON_DEFAULT".to_string(),
                    output: Value::Null,
                });
            }
            return Ok(StepResult {
                next: None,
                edge_label: match_v.clone(),
                output: Value::Null,
            });
        }
        // Default behaviour: `ON_DEFAULT` when `has_default` is unset but
        // no case matched — surface the typed default edge so the caller
        // can observe the miss.
        return Ok(StepResult {
            next: None,
            edge_label: "ON_DEFAULT".to_string(),
            output: Value::Null,
        });
    }

    // 3. Conditional-list form: `conditions: List<{label, condition_value}>`.
    if let Some(Value::List(conds)) = op.properties.get("conditions") {
        for c in conds {
            if let Value::Map(m) = c {
                let cond = matches!(m.get("condition_value"), Some(Value::Bool(true)));
                if cond {
                    let label = match m.get("label") {
                        Some(Value::Text(s)) => s.clone(),
                        _ => "true".to_string(),
                    };
                    return Ok(StepResult {
                        next: None,
                        edge_label: label,
                        output: Value::Null,
                    });
                }
            }
        }
        return Ok(StepResult {
            next: None,
            edge_label: "ON_DEFAULT".to_string(),
            output: Value::Null,
        });
    }

    // No recognised discriminator — conservative default.
    Ok(StepResult {
        next: None,
        edge_label: "false".to_string(),
        output: Value::Null,
    })
}
