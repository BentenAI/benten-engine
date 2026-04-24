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

/// G4-A mini-review M1: an unknown-callee rejection surfaced by the
/// multiplicative walker. A CALL node that names a `handler` which is not
/// resolvable to a cumulative bound at registration time is treated as an
/// adversarial / misconfigured subgraph; the walker fails the subgraph
/// rather than silently defaulting to factor 1.
#[derive(Debug, Clone)]
pub(crate) struct UnknownCallee {
    /// The CALL node id that references the unregistered callee.
    pub(crate) op_id: String,
    /// The `handler` property value that could not be resolved.
    pub(crate) handler: String,
}

/// Resolve the multiplicative factor a given operation node contributes to
/// any DAG path passing through it.
///
/// See crate docs for the factor table. `CALL { isolated: true }` is NOT
/// resolved here — it's a reset, not a factor, and the path-walker handles
/// it separately.
///
/// Returns `Err(UnknownCallee)` for a non-isolated CALL whose `handler`
/// property names a callee not registered via `register_test_callee` and
/// which does not carry a directly-encoded `max` on the CALL node itself
/// (G4-A mini-review M1).
fn factor_for_node(op: &OperationNode) -> Result<u64, UnknownCallee> {
    match op.kind {
        PrimitiveKind::Iterate => match op.properties.get("max") {
            Some(Value::Int(m)) if *m > 0 => Ok(u64::try_from(*m).unwrap_or(u64::MAX)),
            // No `max` declared — the registration-time Inv-8 max-missing
            // check (invariants/structural.rs) already rejects this; treat
            // as factor 1 here so the walker doesn't explode.
            _ => Ok(1),
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
        _ => Ok(1),
    }
}

/// Factor contributed by a non-isolated CALL.
///
/// Resolution order (G4-A mini-review M1):
/// 1. If the CALL declares a `handler` property, it MUST resolve to a
///    registered callee bound — otherwise return `Err(UnknownCallee)`.
///    This is the adversarial case the M1 fix addresses: a handler
///    declaring a named callee that was never registered cannot silently
///    default to factor 1 (or to the CALL's own `max`), because that
///    would let a malicious subgraph bypass Inv-8 by claiming a cheap
///    callee and then pointing at an expensive one at runtime.
/// 2. If the CALL declares no `handler` property but carries a `max`,
///    use the directly-encoded bound (test-harness shorthand used by
///    `build_call_with_callee_budget_for_test`).
/// 3. If the CALL declares neither, fall back to factor 1. This covers
///    the legacy `SubgraphBuilder::call` surface (which ignores its
///    `_handler` arg) used by depth + structural tests that build CALL
///    nodes purely to exercise non-Inv-8 checks. The handler-less CALL
///    carries no iteration cost of its own — the iteration-cost attack
///    surface is gated on declaring a named callee.
fn non_isolated_callee_factor(op: &OperationNode) -> Result<u64, UnknownCallee> {
    if matches!(op.properties.get("isolated"), Some(Value::Bool(true))) {
        // Caller of `factor_for_node` for an isolated CALL shouldn't
        // happen (the walker detects isolation first and resets). Guard
        // defensively.
        return Ok(1);
    }
    if let Some(Value::Text(name)) = op.properties.get("handler") {
        return crate::lookup_test_callee(name).ok_or_else(|| UnknownCallee {
            op_id: op.id.clone(),
            handler: name.clone(),
        });
    }
    match op.properties.get("max") {
        Some(Value::Int(m)) if *m > 0 => Ok(u64::try_from(*m).unwrap_or(u64::MAX)),
        _ => Ok(1),
    }
}

/// Whether `op` is a CALL primitive declaring `isolated: true`.
fn is_isolated_call(op: &OperationNode) -> bool {
    matches!(op.kind, PrimitiveKind::Call)
        && matches!(op.properties.get("isolated"), Some(Value::Bool(true)))
}

/// Look up the callee-grant bound for an isolated CALL node.
///
/// Same resolution rules as [`non_isolated_callee_factor`] (registry
/// lookup when `handler` is named → rejection on miss; `max` on the CALL
/// node when no handler is named; fallback 1 when neither is declared).
///
/// G4-A mini-review M1: an unknown NAMED callee at an isolated-CALL
/// boundary is a registration-time rejection, not a silent-default-to-1
/// — an adversarial handler declaring `isolated: true` with
/// `handler: "<unregistered>"` must not bypass Inv-8 by pointing at an
/// undisclosed callee. Handler-LESS isolated CALLs with no `max`
/// contribute factor 1 (no iteration cost is the honest fallback when
/// the CALL site declares no cost at all).
fn isolated_callee_bound(op: &OperationNode) -> Result<u64, UnknownCallee> {
    if let Some(Value::Text(name)) = op.properties.get("handler") {
        return crate::lookup_test_callee(name).ok_or_else(|| UnknownCallee {
            op_id: op.id.clone(),
            handler: name.clone(),
        });
    }
    match op.properties.get("max") {
        Some(Value::Int(m)) if *m > 0 => Ok(u64::try_from(*m).unwrap_or(u64::MAX)),
        _ => Ok(1),
    }
}

// ---------------------------------------------------------------------------
// DAG walker — computes per-node cumulative budgets.
// ---------------------------------------------------------------------------

/// Compute the MAX-over-paths cumulative budget for each node in a finalized
/// Subgraph, keyed by node id. Saturating multiplication throughout.
///
/// Returns `Err(UnknownCallee)` when any CALL node on the walked path
/// names an unresolvable callee (G4-A mini-review M1).
fn cumulative_by_id(sg: &Subgraph) -> Result<HashMap<String, u64>, UnknownCallee> {
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
        walk(root, 1, &by_id, &outgoing, &mut cumulative, &mut Vec::new())?;
    }
    Ok(cumulative)
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
) -> Result<(), UnknownCallee> {
    if visiting.iter().any(|v| v == cur) {
        return Ok(());
    }
    let Some(op) = by_id.get(cur).copied() else {
        return Ok(());
    };

    // Compute the cumulative AT this node.
    let at_here = if is_isolated_call(op) {
        // Isolated CALL resets cumulative to the callee grant's bound.
        isolated_callee_bound(op)?
    } else {
        // Factor-multiply onto the running product (saturating).
        running.saturating_mul(factor_for_node(op)?)
    };

    // Record the MAX over all paths reaching `cur`.
    let slot = cumulative.entry(cur.to_string()).or_insert(0);
    if at_here > *slot {
        *slot = at_here;
    }

    // Determine the product propagated to downstream nodes.
    //
    // G4-A mini-review M2: an isolated-CALL does NOT taint the caller's
    // post-CALL path with the callee's bound. The callee runs under its
    // own grant, so the caller's nodes reached AFTER the CALL must
    // continue from the caller's PRE-CALL running product. A shape like
    // `ITERATE(10) → isolated-CALL(callee-bound=5) → ITERATE(3)` therefore
    // propagates carry_forward=10 (not 5) past the CALL, yielding
    // cumulative=30 at the trailing ITERATE — NOT 15.
    //
    // For a non-isolated CALL, `at_here = running × callee_factor`; the
    // caller's post-CALL path legitimately inherits that product
    // (non-isolated semantics = budget composes multiplicatively through
    // the CALL boundary).
    //
    // For every other primitive, `at_here == running × factor_for_node`,
    // which is also what downstream should see.
    let carry_forward = if is_isolated_call(op) {
        running
    } else {
        at_here
    };

    visiting.push(cur.to_string());
    let empty: Vec<&str> = Vec::new();
    for &next in outgoing.get(cur).unwrap_or(&empty) {
        walk(next, carry_forward, by_id, outgoing, cumulative, visiting)?;
    }
    visiting.pop();
    Ok(())
}

// ---------------------------------------------------------------------------
// Snapshot walker — builder-side equivalent of `cumulative_by_id`.
// ---------------------------------------------------------------------------

/// Compute per-node cumulative budgets for a builder snapshot (the pre-
/// finalize form used by `SubgraphBuilder::build_validated`). Returns a
/// Vec indexed by node position.
///
/// Returns `Err(UnknownCallee)` when any CALL node names an unresolvable
/// callee (G4-A mini-review M1, mirroring `cumulative_by_id`).
pub(crate) fn cumulative_by_snapshot(sn: &SubgraphSnapshot<'_>) -> Result<Vec<u64>, UnknownCallee> {
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
            )?;
        }
    }
    Ok(cumulative)
}

fn walk_snapshot(
    cur: usize,
    running: u64,
    nodes: &[OperationNode],
    outgoing: &HashMap<usize, Vec<usize>>,
    cumulative: &mut [u64],
    visiting: &mut Vec<usize>,
) -> Result<(), UnknownCallee> {
    if visiting.contains(&cur) {
        return Ok(());
    }
    let Some(op) = nodes.get(cur) else {
        return Ok(());
    };
    let at_here = if is_isolated_call(op) {
        isolated_callee_bound(op)?
    } else {
        running.saturating_mul(factor_for_node(op)?)
    };
    if at_here > cumulative.get(cur).copied().unwrap_or(0)
        && let Some(slot) = cumulative.get_mut(cur)
    {
        *slot = at_here;
    }
    // G4-A mini-review M2: isolated-CALL must NOT leak the callee bound
    // into the caller's post-CALL path — mirror the finalized-subgraph
    // walker (see `walk` above).
    let carry_forward = if is_isolated_call(op) {
        running
    } else {
        at_here
    };
    visiting.push(cur);
    let empty: Vec<usize> = Vec::new();
    for &next in outgoing.get(&cur).unwrap_or(&empty) {
        walk_snapshot(next, carry_forward, nodes, outgoing, cumulative, visiting)?;
    }
    visiting.pop();
    Ok(())
}

// ---------------------------------------------------------------------------
// Public cumulative-budget query helpers (consumed by test harness).
// ---------------------------------------------------------------------------

/// Compute the cumulative Inv-8 budget for a subgraph; returns the
/// MAX-over-paths product.
///
/// A CALL node with an unresolvable callee (no registered bound and no
/// directly-encoded `max`) saturates this computation to `u64::MAX`, which
/// reliably exceeds any configured Inv-8 bound and surfaces at the
/// validator layer as a registration rejection (G4-A mini-review M1). The
/// typed rejection with handler-id context flows through the validator
/// entry points below.
#[must_use]
pub fn compute_cumulative(subgraph: &Subgraph) -> u64 {
    match cumulative_by_id(subgraph) {
        Ok(table) => table.values().copied().max().unwrap_or(1),
        Err(_) => u64::MAX,
    }
}

/// Compute the cumulative Inv-8 budget at a specific node handle. `None` is
/// returned when the handle doesn't correspond to any node in the subgraph
/// or when the subgraph contains an unresolvable callee (in which case the
/// validator path is the canonical surface for the rejection).
#[must_use]
pub fn cumulative_at_handle(subgraph: &Subgraph, handle: NodeHandle) -> Option<u64> {
    let idx = handle.0 as usize;
    let node = subgraph.nodes.get(idx)?;
    let table = cumulative_by_id(subgraph).ok()?;
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
/// worst-path product exceeds `bound.limit()`, or
/// [`ErrorCode::InvRegistration`] when a CALL node references an
/// unresolvable callee bound (G4-A mini-review M1 — an unknown callee is a
/// registration mistake or adversarial bypass attempt, not a factor-1
/// default).
pub fn validate_multiplicative(
    subgraph: &Subgraph,
    bound: MultiplicativeBudget,
) -> Result<(), BudgetError> {
    let table = match cumulative_by_id(subgraph) {
        Ok(t) => t,
        Err(unk) => {
            return Err(BudgetError {
                code: ErrorCode::InvRegistration,
                message: format!(
                    "CALL node {:?} references unresolvable callee {:?} \
                     — isolated/non-isolated CALL requires the callee's \
                     cumulative bound be declared at registration time",
                    unk.op_id, unk.handler
                ),
            });
        }
    };
    let cumulative = table.values().copied().max().unwrap_or(1);
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
/// context. Unknown-callee is mapped to
/// [`InvariantViolation::Registration`] (→ `E_INV_REGISTRATION`) rather
/// than `InvariantViolation::IterateBudget`; a missing callee declaration
/// is a structural flaw, not a budget-overflow.
pub fn validate(subgraph: &Subgraph) -> Result<(), RegistrationError> {
    match validate_multiplicative(subgraph, MultiplicativeBudget::new(DEFAULT_INV_8_BUDGET)) {
        Ok(()) => Ok(()),
        Err(be) if be.code() == ErrorCode::InvRegistration => {
            Err(RegistrationError::new(InvariantViolation::Registration))
        }
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
    let cumulative = match cumulative_by_snapshot(sn) {
        Ok(v) => v,
        Err(_) => {
            // G4-A mini-review M1: unknown callee at the builder-snapshot
            // path is a registration-layer rejection, not a budget
            // overflow.
            return Err(RegistrationError::new(InvariantViolation::Registration));
        }
    };
    let worst = cumulative.iter().copied().max().unwrap_or(1);
    if worst > DEFAULT_INV_8_BUDGET {
        return Err(RegistrationError::new(InvariantViolation::IterateBudget));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Test harnesses (consumed by tests/invariant_8_multiplicative.rs).
// ---------------------------------------------------------------------------

/// Test harness: build a subgraph `ITERATE(m1) → CALL(non-isolated) →
/// ITERATE(m3)` where the CALL's callee has declared bound `call_factor`.
///
/// G4-A mini-review C2: the prior harness collapsed the call-factor into
/// a third ITERATE, which meant
/// `prop_invariant_8_multiplicative_exact` exercised zero CALL code paths
/// (`non_isolated_callee_factor`, `isolated_callee_bound`, or the `walk`
/// isolation branch). This version threads the call-factor through a real
/// CALL node: the middle node is `PrimitiveKind::Call` with
/// `isolated: false` and a registered callee whose bound is `call_factor`.
/// The proptest (and unit tests calling this helper) must
/// [`crate::register_test_callee`] that callee first — the M1 fallback now
/// rejects unknown callees at registration time.
///
/// Caller contract: the callee name returned by this function (encoded on
/// the CALL node's `handler` property) is derived from `call_factor` so
/// each invocation with a distinct factor pre-registers a distinct entry.
/// Caller must call `register_test_callee(name, call_factor)` before
/// validating the subgraph.
#[must_use]
pub fn build_chained_call_iterate_iterate_for_test(m1: u64, call_factor: u64, m3: u64) -> Subgraph {
    use crate::{OperationNode, PrimitiveKind, Subgraph};
    let mut sg = Subgraph::new("chained_call_iterate");
    let i64_or_max = |v: u64| i64::try_from(v).unwrap_or(i64::MAX);
    let callee_name = callee_name_for_factor(call_factor);
    sg.nodes.push(
        OperationNode::new("iter_a", PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64_or_max(m1))),
    );
    sg.nodes.push(
        OperationNode::new("call_mid", PrimitiveKind::Call)
            .with_property("handler", Value::text(callee_name))
            .with_property("isolated", Value::Bool(false)),
    );
    sg.nodes.push(
        OperationNode::new("iter_c", PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64_or_max(m3))),
    );
    sg.edges
        .push(("iter_a".into(), "call_mid".into(), "next".into()));
    sg.edges
        .push(("call_mid".into(), "iter_c".into(), "next".into()));
    sg
}

/// Test harness: deterministic callee name derived from the declared
/// bound. Using the bound as a suffix lets the proptest pre-register
/// every distinct call_factor exactly once and share registrations
/// across test runs in the same process. Consumers must
/// `register_test_callee(&callee_name_for_factor(factor), factor)` before
/// validating the subgraph.
#[must_use]
pub fn callee_name_for_factor(factor: u64) -> String {
    format!("chained_callee_factor_{factor}")
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
