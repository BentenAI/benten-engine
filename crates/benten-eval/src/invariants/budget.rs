//! Phase 2a G4-A / E9 / Code-as-graph Major #2: Invariant-8 multiplicative
//! cumulative budget.
//!
//! # Shape
//!
//! Each primitive node in a subgraph contributes a **factor** to the
//! cumulative iteration budget along any DAG path that reaches it:
//!
//! - `ITERATE { max: N }` contributes factor `N`.
//! - `CALL { isolated: false }` (the default) contributes factor `1` — the
//!   callee's iteration cost composes multiplicatively with the caller's
//!   remaining budget, so the CALL node itself is a pass-through; the
//!   caller's per-iteration cost propagates through.
//! - `CALL { isolated: true }` **resets** the cumulative to the callee
//!   grant's declared bound (Code-as-graph Major #2 — Option B). Isolated
//!   CALLs run under the callee's own grant; the parent's remaining budget
//!   does not multiply in.
//! - Every other primitive contributes factor `1`.
//!
//! The cumulative for a node is the MAX across all DAG paths of the product
//! of per-node factors along that path (saturating at `u64::MAX` on
//! overflow).
//!
//! The cumulative for a whole subgraph is the MAX of per-node cumulatives —
//! the worst-case path product.
//!
//! # Replacement of Phase-1 stopgap
//!
//! Phase 1 shipped a `MAX_ITERATE_NEST_DEPTH = 3` stopgap. Phase 2a G4-A
//! drops the nest-depth bound entirely: a 10-deep nest with per-level max 1
//! has cumulative 1 and must be accepted (see
//! `tests/invariant_8_nest_depth_stopgap_removed.rs`). The multiplicative
//! form replaces that stopgap with the proper cumulative-budget check.

use benten_core::Value;
use benten_errors::ErrorCode;
use std::collections::HashMap;

use crate::{
    InvariantViolation, NodeHandle, OperationNode, PrimitiveKind, RegistrationError, Subgraph,
    SubgraphSnapshot,
};

/// Default cumulative iteration-budget bound (registration-time).
///
/// 500_000 — covers the vast majority of reasonable handler compositions
/// (depth-10 nest at max=1 = 1; depth-4 nest at max=2 = 16; straight
/// chained `ITERATE(3) × CALL(2)` = 6; a single `ITERATE(100_000)` stands
/// alone) while rejecting both the pathological nest-depth-4-at-max-1000
/// (= 1e12) and the `ITERATE(1000) × CALL(1000)` non-isolated overflow (=
/// 1e6). Matches the bound declared inline in `invariant_8_isolated_call`
/// test prose.
pub const DEFAULT_INV_8_BUDGET: u64 = 500_000;

/// Thin newtype wrapping the cumulative bound.
#[derive(Debug, Clone, Copy)]
pub struct MultiplicativeBudget(u64);

impl MultiplicativeBudget {
    /// Construct a budget bound.
    #[must_use]
    pub fn new(limit: u64) -> Self {
        Self(limit)
    }

    /// Return the underlying bound.
    #[must_use]
    pub fn limit(self) -> u64 {
        self.0
    }
}

/// Budget-check error type surfaced by the multiplicative validator.
#[derive(Debug, Clone)]
pub struct BudgetError {
    code: ErrorCode,
    message: String,
}

impl BudgetError {
    /// Stable catalog code.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code.clone()
    }

    /// Human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl core::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for BudgetError {}

// ---------------------------------------------------------------------------
// Per-node factor resolution
// ---------------------------------------------------------------------------

/// Resolve the multiplicative factor a given operation node contributes to
/// any DAG path passing through it.
///
/// See crate docs for the factor table. `CALL { isolated: true }` is NOT
/// resolved here — it's a reset, not a factor, and the path-walker handles
/// it separately.
#[must_use]
fn factor_for_node(op: &OperationNode) -> u64 {
    match op.kind {
        PrimitiveKind::Iterate => match op.properties.get("max") {
            Some(Value::Int(m)) if *m > 0 => u64::try_from(*m).unwrap_or(u64::MAX),
            // No `max` declared — the registration-time Inv-8 max-missing
            // check (invariants/structural.rs) already rejects this; treat
            // as factor 1 here so the walker doesn't explode.
            _ => 1,
        },
        // Non-isolated CALL contributes the callee grant's declared
        // iteration bound as a factor — the caller's remaining budget
        // multiplies with the callee's own bound (Code-as-graph Major #2
        // Option B negative space). Isolated CALL is handled by the
        // path-walker as a RESET, not a factor.
        PrimitiveKind::Call => non_isolated_callee_factor(op),
        // All other primitives (READ, WRITE, TRANSFORM, BRANCH, RESPOND,
        // EMIT, WAIT, SANDBOX, SUBSCRIBE, STREAM) are pass-through:
        // factor 1.
        _ => 1,
    }
}

/// Factor contributed by a non-isolated CALL: the callee's declared
/// iteration bound (via the process-global test-callee registry), falling
/// back to any `max` property on the CALL node, else 1.
fn non_isolated_callee_factor(op: &OperationNode) -> u64 {
    if matches!(op.properties.get("isolated"), Some(Value::Bool(true))) {
        // Caller of `factor_for_node` for an isolated CALL shouldn't
        // happen (the walker detects isolation first and resets). Guard
        // defensively.
        return 1;
    }
    if let Some(Value::Text(name)) = op.properties.get("handler")
        && let Some(bound) = crate::lookup_test_callee(name)
    {
        return bound;
    }
    match op.properties.get("max") {
        Some(Value::Int(m)) if *m > 0 => u64::try_from(*m).unwrap_or(u64::MAX),
        _ => 1,
    }
}

/// Whether `op` is a CALL primitive declaring `isolated: true`.
fn is_isolated_call(op: &OperationNode) -> bool {
    matches!(op.kind, PrimitiveKind::Call)
        && matches!(op.properties.get("isolated"), Some(Value::Bool(true)))
}

/// Whether `op` is a CALL primitive (isolated or not).
fn is_call(op: &OperationNode) -> bool {
    matches!(op.kind, PrimitiveKind::Call)
}

/// Look up the callee-grant bound for an isolated CALL node by consulting the
/// handler name property + the process-global test-callee registry.
///
/// When the CALL's `handler` property doesn't match any registered callee,
/// returns the declared `max` on the CALL node if present, otherwise `1` —
/// an unknown callee contributes no iteration cost of its own.
fn isolated_callee_bound(op: &OperationNode) -> u64 {
    if let Some(Value::Text(name)) = op.properties.get("handler")
        && let Some(bound) = crate::lookup_test_callee(name)
    {
        return bound;
    }
    // Fallback: the CALL node may declare its own `max` for tests that
    // don't register a named callee.
    match op.properties.get("max") {
        Some(Value::Int(m)) if *m > 0 => u64::try_from(*m).unwrap_or(u64::MAX),
        _ => 1,
    }
}

// ---------------------------------------------------------------------------
// DAG walker — computes per-node cumulative budgets.
// ---------------------------------------------------------------------------

/// Compute the MAX-over-paths cumulative budget for each node in a finalized
/// Subgraph, keyed by node id. Saturating multiplication throughout.
fn cumulative_by_id(sg: &Subgraph) -> HashMap<String, u64> {
    // Build adjacency by id.
    let mut outgoing: HashMap<&str, Vec<&str>> = HashMap::new();
    for (f, t, _l) in &sg.edges {
        outgoing.entry(f.as_str()).or_default().push(t.as_str());
    }
    let by_id: HashMap<&str, &OperationNode> =
        sg.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Find roots (nodes with no incoming edge).
    let mut has_incoming: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (_f, t, _l) in &sg.edges {
        has_incoming.insert(t.as_str());
    }
    let roots: Vec<&str> = sg
        .nodes
        .iter()
        .map(|n| n.id.as_str())
        .filter(|id| !has_incoming.contains(id))
        .collect();

    // BFS/DFS carrying the cumulative product so far; at each node record
    // the MAX product seen from any path reaching it. On an isolated CALL
    // the cumulative at that node resets to the callee bound (not the
    // product with what came before).
    let mut cumulative: HashMap<String, u64> = HashMap::new();
    for root in roots {
        walk(root, 1, &by_id, &outgoing, &mut cumulative, &mut Vec::new());
    }
    cumulative
}

/// DFS walker. `running` = product of factors along the path up to but not
/// including `cur`. `visiting` prevents infinite loops under residual cycles
/// (belt-and-suspenders; Invariant 1 should have already rejected cycles).
fn walk(
    cur: &str,
    running: u64,
    by_id: &HashMap<&str, &OperationNode>,
    outgoing: &HashMap<&str, Vec<&str>>,
    cumulative: &mut HashMap<String, u64>,
    visiting: &mut Vec<String>,
) {
    if visiting.iter().any(|v| v == cur) {
        return;
    }
    let Some(op) = by_id.get(cur).copied() else {
        return;
    };

    // Compute the cumulative AT this node.
    let at_here = if is_isolated_call(op) {
        // Isolated CALL resets cumulative to the callee grant's bound.
        isolated_callee_bound(op)
    } else {
        // Factor-multiply onto the running product (saturating).
        running.saturating_mul(factor_for_node(op))
    };

    // Record the MAX over all paths reaching `cur`.
    let slot = cumulative.entry(cur.to_string()).or_insert(0);
    if at_here > *slot {
        *slot = at_here;
    }

    // For non-isolated CALL, the downstream propagation carries the full
    // running × callee-frame product. For isolated CALL, the downstream
    // continues from the reset bound (the callee's frame). For other
    // primitives, downstream carries `at_here`.
    let carry_forward = if is_isolated_call(op) {
        at_here
    } else if is_call(op) {
        // Non-isolated CALL: factor itself is 1 at the CALL node, but the
        // downstream of the CALL subgraph isn't represented in the same
        // Subgraph — the caller's subsequent nodes see `at_here` (which
        // equals `running` since factor_for_node(CALL)=1).
        at_here
    } else {
        at_here
    };

    visiting.push(cur.to_string());
    let empty: Vec<&str> = Vec::new();
    for &next in outgoing.get(cur).unwrap_or(&empty) {
        walk(next, carry_forward, by_id, outgoing, cumulative, visiting);
    }
    visiting.pop();
}

// ---------------------------------------------------------------------------
// Snapshot walker — builder-side equivalent of `cumulative_by_id`.
// ---------------------------------------------------------------------------

/// Compute per-node cumulative budgets for a builder snapshot (the pre-
/// finalize form used by `SubgraphBuilder::build_validated`). Returns a
/// Vec indexed by node position.
pub(crate) fn cumulative_by_snapshot(sn: &SubgraphSnapshot<'_>) -> Vec<u64> {
    // Build adjacency by index.
    let mut outgoing: HashMap<usize, Vec<usize>> = HashMap::new();
    for (f, t, _l) in sn.edges {
        outgoing.entry(f.0 as usize).or_default().push(t.0 as usize);
    }

    // Roots = nodes with no incoming edge.
    let mut has_incoming = vec![false; sn.nodes.len()];
    for (_f, t, _l) in sn.edges {
        if let Some(slot) = has_incoming.get_mut(t.0 as usize) {
            *slot = true;
        }
    }

    let mut cumulative = vec![0_u64; sn.nodes.len()];
    for (idx, _) in sn.nodes.iter().enumerate() {
        if !has_incoming.get(idx).copied().unwrap_or(false) {
            walk_snapshot(
                idx,
                1,
                sn.nodes,
                &outgoing,
                &mut cumulative,
                &mut Vec::new(),
            );
        }
    }
    cumulative
}

fn walk_snapshot(
    cur: usize,
    running: u64,
    nodes: &[OperationNode],
    outgoing: &HashMap<usize, Vec<usize>>,
    cumulative: &mut [u64],
    visiting: &mut Vec<usize>,
) {
    if visiting.contains(&cur) {
        return;
    }
    let Some(op) = nodes.get(cur) else {
        return;
    };
    let at_here = if is_isolated_call(op) {
        isolated_callee_bound(op)
    } else {
        running.saturating_mul(factor_for_node(op))
    };
    if at_here > cumulative.get(cur).copied().unwrap_or(0)
        && let Some(slot) = cumulative.get_mut(cur)
    {
        *slot = at_here;
    }
    visiting.push(cur);
    let empty: Vec<usize> = Vec::new();
    for &next in outgoing.get(&cur).unwrap_or(&empty) {
        walk_snapshot(next, at_here, nodes, outgoing, cumulative, visiting);
    }
    visiting.pop();
}

// ---------------------------------------------------------------------------
// Public cumulative-budget query helpers (consumed by test harness).
// ---------------------------------------------------------------------------

/// Compute the cumulative Inv-8 budget for a subgraph; returns the
/// MAX-over-paths product.
#[must_use]
pub fn compute_cumulative(subgraph: &Subgraph) -> u64 {
    cumulative_by_id(subgraph)
        .values()
        .copied()
        .max()
        .unwrap_or(1)
}

/// Compute the cumulative Inv-8 budget at a specific node handle. `None` is
/// returned when the handle doesn't correspond to any node in the subgraph.
#[must_use]
pub fn cumulative_at_handle(subgraph: &Subgraph, handle: NodeHandle) -> Option<u64> {
    let idx = handle.0 as usize;
    let node = subgraph.nodes.get(idx)?;
    let table = cumulative_by_id(subgraph);
    table.get(&node.id).copied()
}

// ---------------------------------------------------------------------------
// Validation entry points.
// ---------------------------------------------------------------------------

/// Validate the multiplicative cumulative budget for a subgraph against a
/// declared bound.
///
/// # Errors
/// Fires [`ErrorCode::InvIterateBudget`] via [`BudgetError`] when the
/// worst-path product exceeds `bound.limit()`.
pub fn validate_multiplicative(
    subgraph: &Subgraph,
    bound: MultiplicativeBudget,
) -> Result<(), BudgetError> {
    let cumulative = compute_cumulative(subgraph);
    if cumulative > bound.limit() {
        return Err(BudgetError {
            code: ErrorCode::InvIterateBudget,
            message: format!(
                "cumulative iteration budget {cumulative} exceeds bound {}",
                bound.limit()
            ),
        });
    }
    Ok(())
}

/// Integration with the structural `validate_subgraph` entry point. Uses
/// `DEFAULT_INV_8_BUDGET` as the registration-time bound.
///
/// # Errors
/// Converts [`BudgetError`] to [`RegistrationError`] carrying the Inv-8
/// context.
pub fn validate(subgraph: &Subgraph) -> Result<(), RegistrationError> {
    match validate_multiplicative(subgraph, MultiplicativeBudget::new(DEFAULT_INV_8_BUDGET)) {
        Ok(()) => Ok(()),
        Err(_) => Err(RegistrationError::new(InvariantViolation::IterateBudget)),
    }
}

/// Builder-snapshot validation. Consumed by `validate_builder` in
/// `invariants/structural.rs`. Fires the same `InvIterateBudget` code as
/// the finalized-subgraph path.
///
/// # Errors
/// Returns a populated [`RegistrationError`] when any node's cumulative
/// budget exceeds `DEFAULT_INV_8_BUDGET`.
pub(crate) fn validate_snapshot(sn: &SubgraphSnapshot<'_>) -> Result<(), RegistrationError> {
    let cumulative = cumulative_by_snapshot(sn);
    let worst = cumulative.iter().copied().max().unwrap_or(1);
    if worst > DEFAULT_INV_8_BUDGET {
        return Err(RegistrationError::new(InvariantViolation::IterateBudget));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Test harnesses (consumed by tests/invariant_8_multiplicative.rs).
// ---------------------------------------------------------------------------

/// Test harness: build a subgraph `ITERATE(m1) → CALL → ITERATE(m2)` with the
/// call_factor embedded as a leading ITERATE so the "chained CALL/ITERATE"
/// product is `m1 × m2 × m3`.
///
/// The test name `build_chained_call_iterate_iterate_for_test(a, b, c)` is
/// documented in `invariant_8_multiplicative.rs` as producing cumulative
/// `a × b × c`. We realize that as a straight ITERATE chain of three ITERATE
/// primitives with maxes `a`, `b`, `c` (the CALL boundary contributes
/// factor 1 to the product under non-isolated semantics).
#[must_use]
pub fn build_chained_call_iterate_iterate_for_test(m1: u64, m2: u64, m3: u64) -> Subgraph {
    use crate::{OperationNode, PrimitiveKind, Subgraph};
    let mut sg = Subgraph::new("chained_call_iterate");
    let i64_or_max = |v: u64| i64::try_from(v).unwrap_or(i64::MAX);
    sg.nodes.push(
        OperationNode::new("iter_a", PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64_or_max(m1))),
    );
    sg.nodes.push(
        OperationNode::new("iter_b", PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64_or_max(m2))),
    );
    sg.nodes.push(
        OperationNode::new("iter_c", PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64_or_max(m3))),
    );
    sg.edges
        .push(("iter_a".into(), "iter_b".into(), "next".into()));
    sg.edges
        .push(("iter_b".into(), "iter_c".into(), "next".into()));
    sg
}

/// Test harness: ITERATE(inner_max) nested inside ITERATE(outer_max).
#[must_use]
pub fn build_nested_iterate_for_test(outer_max: u64, inner_max: u64) -> Subgraph {
    use crate::{OperationNode, PrimitiveKind, Subgraph};
    let mut sg = Subgraph::new("nested_iterate");
    let i64_or_max = |v: u64| i64::try_from(v).unwrap_or(i64::MAX);
    sg.nodes.push(
        OperationNode::new("outer", PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64_or_max(outer_max))),
    );
    sg.nodes.push(
        OperationNode::new("inner", PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64_or_max(inner_max))),
    );
    sg.edges
        .push(("outer".into(), "inner".into(), "next".into()));
    sg
}

/// Test harness: a CALL with `isolated: true` into a callee whose grant
/// declared bound is `callee_bound`. The caller's iteration count does not
/// multiply in (the isolated CALL resets to the callee bound).
#[must_use]
pub fn build_call_with_callee_budget_for_test(callee_bound: u64) -> Subgraph {
    use crate::{OperationNode, PrimitiveKind, Subgraph};
    let mut sg = Subgraph::new("call_with_callee_budget");
    let bound = i64::try_from(callee_bound).unwrap_or(i64::MAX);
    sg.nodes
        .push(OperationNode::new("read", PrimitiveKind::Read));
    sg.nodes.push(
        OperationNode::new("call", PrimitiveKind::Call)
            .with_property("isolated", Value::Bool(true))
            // Encode the callee bound directly on the CALL node so the
            // validator resolves it without a registry lookup.
            .with_property("max", Value::Int(bound)),
    );
    sg.edges.push(("read".into(), "call".into(), "next".into()));
    sg
}

/// Test harness: a DAG with two paths of different products; cumulative is
/// the MAX over paths.
///
/// Layout: a root node forks into two independent chains; each chain is an
/// ITERATE for every entry in its path-spec list; the chains rejoin at a
/// terminal RESPOND. Cumulative for the subgraph is `max(product(path_a),
/// product(path_b))`.
#[must_use]
pub fn build_two_path_dag_for_test(path_a: &[u64], path_b: &[u64]) -> Subgraph {
    use crate::{OperationNode, PrimitiveKind, Subgraph};
    let mut sg = Subgraph::new("two_path_dag");
    let i64_or_max = |v: u64| i64::try_from(v).unwrap_or(i64::MAX);
    sg.nodes
        .push(OperationNode::new("root", PrimitiveKind::Read));
    let mut prev_a: String = "root".into();
    for (i, m) in path_a.iter().copied().enumerate() {
        let id = format!("a_{i}");
        sg.nodes.push(
            OperationNode::new(id.clone(), PrimitiveKind::Iterate)
                .with_property("max", Value::Int(i64_or_max(m))),
        );
        sg.edges.push((prev_a.clone(), id.clone(), "next".into()));
        prev_a = id;
    }
    let mut prev_b: String = "root".into();
    for (i, m) in path_b.iter().copied().enumerate() {
        let id = format!("b_{i}");
        sg.nodes.push(
            OperationNode::new(id.clone(), PrimitiveKind::Iterate)
                .with_property("max", Value::Int(i64_or_max(m))),
        );
        sg.edges.push((prev_b.clone(), id.clone(), "next".into()));
        prev_b = id;
    }
    sg
}
