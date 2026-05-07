//! TRANSFORM primitive executor.
//!
//! TRANSFORM evaluates a pure expression against an input Value and surfaces
//! the result on the `"ok"` edge. The expression is stored on the operation
//! Node as the `"expr"` property (a Text value carrying the source string);
//! the input is supplied on the `"input"` property (bound to `$input` in
//! the expression).
//!
//! # Registration-time parse guarantee (5d-J workstream 3)
//!
//! Every TRANSFORM node's `expr` is parsed at `register_subgraph` time via
//! [`crate::invariants::validate_transform_expressions`]; unparseable
//! grammar surfaces `E_TRANSFORM_SYNTAX` during registration, not at call
//! time.
//!
//! # AST cache (Phase-3 G19-E — phase-2-backlog §9.2 CLOSURE)
//!
//! Per Phase-3 R5 wave-7b G19-E, the runtime executor consults the host's
//! [`crate::PrimitiveHost::cached_transform_ast`] hook BEFORE re-parsing
//! the `expr` source on every call. The `benten-engine` host populates
//! the cache at `register_subgraph` / `register_subgraph_replace` time
//! (entries keyed by `(handler_cid, node_id)`); subsequent calls under
//! the same handler version skip the parse entirely. The cache is
//! invalidated when the handler's CID flips (re-register). Defensive
//! re-parse remains the fallback path when `cached_transform_ast`
//! returns `None` (Phase-1 `NullHost` + every test host); observable
//! behaviour is identical either way.
//!
//! Phase-1 contract (R2 §2.5 row 7):
//!
//! - `expr` missing / not a string → routes `ON_ERROR`.
//! - Parse failure → routes `ON_ERROR`, payload carries `E_TRANSFORM_SYNTAX`.
//! - Runtime failure (type mismatch, unbound identifier, etc.) → routes
//!   `ON_ERROR`.
//! - Success → routes `"ok"` with the evaluated [`Value`] on `output`.
//!
//! TRANSFORM is deterministic: identical expression + identical input
//! produces an identical output on every call. This mirrors the content-
//! hash invariant (ENGINE-SPEC §7) and underpins the IVM view determinism.

use benten_core::Value;

use crate::expr::{Expr, eval::Env, eval::eval_with_namespaces, parser::parse};
use crate::{EvalError, OperationNode, PrimitiveHost, StepResult};

/// Execute a TRANSFORM primitive.
///
/// G19-E (phase-2-backlog §9.2 closure): consults
/// [`PrimitiveHost::cached_transform_ast`] for a pre-parsed
/// [`crate::expr::Expr`] before falling through to the per-call parse
/// path. The host's default impl returns `None` (Phase-1 NullHost + every
/// test host); the `benten-engine` impl populates the cache at
/// `register_subgraph` time so repeated calls at the same handler version
/// skip the parse entirely.
///
/// # Errors
///
/// Returns [`EvalError::TransformSyntax`] when the expression fails to
/// parse. Other runtime failures are routed through the `ON_ERROR` edge
/// rather than bubbled as `Err`, matching the [`StepResult`]-as-edge-
/// routing contract used by READ / WRITE.
pub fn execute(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    // G19-E: AST cache fast path. When the host returns a pre-parsed
    // Expr for this node, skip the source-property lookup and the parse
    // step entirely. Falls through to the parse path on cache miss
    // (default trait impl) so Phase-1 NullHost + unit tests keep
    // working unchanged.
    if let Some(cached) = host.cached_transform_ast(&op.id) {
        return run_with_expr(op, cached.as_ref());
    }

    let expr_src = match op.properties.get("expr") {
        Some(Value::Text(s)) => s.clone(),
        _ => return Ok(on_error("TRANSFORM operation missing `expr` property")),
    };

    let expr = match parse(&expr_src) {
        Ok(e) => e,
        Err(parse_err) => {
            // Parse errors are the one case that surfaces a typed Err (so
            // callers / registration-time validation can surface the BNF
            // rejection class with the byte offset). Runtime calls wrap
            // this into an edge result via the evaluator's dispatch.
            return Err(EvalError::TransformSyntax(parse_err.message));
        }
    };

    run_with_expr(op, &expr)
}

/// Evaluate a TRANSFORM `Expr` (cache-hit or fresh-parse) against the
/// node's `input` / `result` properties. Shared by the cached and
/// non-cached paths so behaviour is identical either way.
fn run_with_expr(op: &OperationNode, expr: &Expr) -> Result<StepResult, EvalError> {
    let mut env = Env::with_input(op.properties.get("input").cloned().unwrap_or(Value::Null));
    if let Some(result) = op.properties.get("result") {
        env.set("$result", result.clone());
    }

    match eval_with_namespaces(expr, &mut env) {
        Ok(v) => Ok(StepResult {
            next: None,
            edge_label: "ok".to_string(),
            output: v,
        }),
        Err(err) => Ok(on_error(&err.to_string())),
    }
}

fn on_error(reason: &str) -> StepResult {
    StepResult {
        next: None,
        edge_label: "ON_ERROR".to_string(),
        output: Value::text(reason),
    }
}
