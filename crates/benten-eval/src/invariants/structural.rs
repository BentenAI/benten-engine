//! Phase-1 structural invariants (1, 2, 3, 5, 6, 9, 10, 12).
//!
//! Pure code-move from the former single-file `invariants.rs` per plan §3
//! G1-A. Phase-2a invariants 8 (multiplicative budget), 11 (system zone), 13
//! (immutability), 14 (attribution) live in sibling files in this directory.
//!
//! Per ENGINE-SPEC §4 and the Phase-1 scope reconciliation in
//! `.addl/phase-1/00-implementation-plan.md` §3 G6-C, this module enforces:
//!
//! - **1 — DAG-ness** (back-edge detection via DFS coloring)
//! - **2 — max depth** (longest path in the subgraph)
//! - **3 — max fan-out per node** (both explicit outgoing edges and the
//!   logical fan-out of `iterate_parallel(max)` siblings)
//! - **5 — max total nodes** (default 4096)
//! - **6 — max total edges** (default 8192)
//! - **8 (Phase-1 stopgap)** — max ITERATE nesting depth (R1 named
//!   compromise; hardcoded to 3 per `DEFAULT_MAX_ITERATE_NEST_DEPTH`).
//!   Phase-2a G4-A replaces this stopgap with the multiplicative
//!   cumulative-budget path in `invariants/budget.rs`.
//! - **9 — determinism classification** (any non-deterministic primitive
//!   inside a handler declared `deterministic: true` is rejected)
//! - **10 — content-addressed CID** (byte encoding is order-independent:
//!   the same node/edge set in a different construction order produces the
//!   same CID, per R1 triage)
//! - **12 — registration-time catch-all** (aggregate-mode reporting across
//!   two or more simultaneous violations)
//!
//! Per Validated Design Decision #5, the CID is BLAKE3 over a canonical
//! DAG-CBOR encoding; the canonicalizer sorts nodes and edges before
//! encoding so CID is invariant under construction order.

use benten_core::{CoreError, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::expr::parser;
use crate::{
    EvalError, InvariantConfig, InvariantViolation, NodeHandle, OperationNode, PrimitiveKind,
    RegistrationError, Subgraph, SubgraphSnapshot,
};

/// Parse every TRANSFORM operation-node's `expr` property at registration
/// time. 5d-J workstream 3: gives the fail-fast guarantee that an
/// unparseable expression surfaces at `register_subgraph` rather than
/// waiting for `engine.call` to trip the parser mid-dispatch.
///
/// # Errors
///
/// Returns [`EvalError::TransformSyntax`] carrying the parser's diagnostic
/// message when any TRANSFORM node fails to parse.
pub fn validate_transform_expressions(sg: &Subgraph) -> Result<(), EvalError> {
    for node in &sg.nodes {
        if !matches!(node.kind, PrimitiveKind::Transform) {
            continue;
        }
        let Some(Value::Text(src)) = node.properties.get("expr") else {
            // Missing `expr` is a runtime `ON_ERROR` (route) rather than a
            // hard registration reject — a TRANSFORM node without an expr
            // is legal but useless. Runtime path handles it.
            continue;
        };
        parser::parse(src).map_err(|e| EvalError::TransformSyntax(e.message))?;
    }
    Ok(())
}

/// Validate a finalized [`Subgraph`] (the post-builder, post-edge-
/// materialization form) against the structural invariants.
///
/// Used by [`Subgraph::validate`](crate::Subgraph::validate) when the caller
/// already has a `Subgraph` (e.g. after round-tripping through storage).
/// `SubgraphBuilder::build_validated` takes the richer builder-snapshot path
/// via `validate_builder` (crate-private).
///
/// # Errors
///
/// Returns a [`RegistrationError`] carrying the first invariant violation
/// encountered. `aggregate=true` runs every check and reports all failures
/// via [`InvariantViolation::Registration`]-style context.
pub fn validate_subgraph(
    sg: &Subgraph,
    config: &InvariantConfig,
    aggregate: bool,
) -> Result<(), RegistrationError> {
    // Project the Subgraph back onto a snapshot-shaped view. Edges use node
    // ids as strings so the checker walks the same primary key the storage
    // layer uses.
    let mut violations: Vec<InvariantViolation> = Vec::new();
    let mut out = RegistrationError::new(InvariantViolation::Cycle);

    // Invariant 5 — node count.
    let max_nodes = usize::try_from(config.max_nodes).unwrap_or(usize::MAX);
    if sg.nodes.len() > max_nodes {
        out.nodes_actual = Some(sg.nodes.len());
        out.nodes_max = Some(max_nodes);
        violations.push(InvariantViolation::TooManyNodes);
        if !aggregate {
            return Err(finalize(out, violations));
        }
    }

    // Invariant 6 — edge count.
    let max_edges = usize::try_from(config.max_edges).unwrap_or(usize::MAX);
    if sg.edges.len() > max_edges {
        out.edges_actual = Some(sg.edges.len());
        out.edges_max = Some(max_edges);
        violations.push(InvariantViolation::TooManyEdges);
        if !aggregate {
            return Err(finalize(out, violations));
        }
    }

    // Build string-keyed adjacency.
    let mut outgoing: HashMap<&str, Vec<&str>> = HashMap::new();
    for (f, t, _l) in &sg.edges {
        outgoing.entry(f.as_str()).or_default().push(t.as_str());
    }
    let node_ids: HashSet<&str> = sg.nodes.iter().map(|n| n.id.as_str()).collect();

    // Invariant 1 — DAG-ness.
    if let Some(path) = find_cycle(&sg.nodes, &outgoing) {
        out.cycle_path = Some(path);
        violations.push(InvariantViolation::Cycle);
        if !aggregate {
            return Err(finalize(out, violations));
        }
    }

    // Invariant 3 — per-node fan-out.
    let max_fanout = usize::try_from(config.max_fanout).unwrap_or(usize::MAX);
    for n in &sg.nodes {
        let explicit = outgoing.get(n.id.as_str()).map_or(0, Vec::len);
        let parallel = node_parallel_fanout(n);
        let fan = explicit.max(parallel);
        if fan > max_fanout {
            out.fanout_actual = Some(fan);
            out.fanout_max = Some(max_fanout);
            out.fanout_node_id = Some(n.id.clone());
            violations.push(InvariantViolation::FanoutExceeded);
            if !aggregate {
                return Err(finalize(out, violations));
            }
            break;
        }
    }

    // Invariant 2 — max depth (longest path). Assumes DAG; if cycle was
    // reported, skip to avoid infinite walks.
    if !violations.contains(&InvariantViolation::Cycle) {
        let max_depth = usize::try_from(config.max_depth).unwrap_or(usize::MAX);
        if let Some(longest) = longest_path(&sg.nodes, &outgoing)
            && longest.len() > max_depth
        {
            out.depth_actual = Some(longest.len());
            out.depth_max = Some(max_depth);
            out.longest_path = Some(longest);
            violations.push(InvariantViolation::DepthExceeded);
            if !aggregate {
                return Err(finalize(out, violations));
            }
        }
    }

    // Invariant 9 — determinism. 5d-J workstream 4: the finalized
    // Subgraph now carries the `deterministic` flag (still in-memory
    // only; DAG-CBOR serialization is Phase-2 scope per the earlier
    // `load_verified` decoder TODO), so a builder-to-finalized
    // round-trip re-runs the same per-primitive classification check
    // the builder-snapshot path applied. See mini-review findings
    // `g6-cag-4` / `g6-opl-5` for the earlier state.
    if sg.deterministic {
        for n in &sg.nodes {
            if !n.kind.is_deterministic() {
                violations.push(InvariantViolation::Determinism);
                if !aggregate {
                    let mut err = RegistrationError::new(InvariantViolation::Determinism);
                    err.fanout_node_id = Some(n.id.clone());
                    return Err(err);
                }
                break;
            }
        }
    }

    // Edge references to undeclared nodes don't fire a typed invariant in
    // Phase 1; the structural-catch-all path handles it in Phase 2.
    drop(node_ids);

    if violations.is_empty() {
        Ok(())
    } else {
        Err(finalize(out, violations))
    }
}

/// Validate a builder snapshot. This is the primary path exercised by
/// `SubgraphBuilder::build_validated*` because the snapshot carries
/// builder-only metadata (iterate-nest depth, parallel fanout, determinism
/// flag) the finalized [`Subgraph`] loses.
///
/// # Errors
///
/// See [`validate_subgraph`].
pub(crate) fn validate_builder(
    sn: &SubgraphSnapshot<'_>,
    config: &InvariantConfig,
    aggregate: bool,
) -> Result<(), RegistrationError> {
    let mut violations: Vec<InvariantViolation> = Vec::new();
    let mut out = RegistrationError::new(InvariantViolation::Cycle);

    // Invariant 5 — node count.
    let max_nodes = usize::try_from(config.max_nodes).unwrap_or(usize::MAX);
    if sn.nodes.len() > max_nodes {
        out.nodes_actual = Some(sn.nodes.len());
        out.nodes_max = Some(max_nodes);
        violations.push(InvariantViolation::TooManyNodes);
        if !aggregate {
            return Err(finalize(out, violations));
        }
    }

    // Invariant 6 — edge count, includes synthetic extra edges.
    let max_edges = usize::try_from(config.max_edges).unwrap_or(usize::MAX);
    let total_edges = sn.edges.len() + sn.extra_edges;
    if total_edges > max_edges {
        out.edges_actual = Some(total_edges);
        out.edges_max = Some(max_edges);
        violations.push(InvariantViolation::TooManyEdges);
        if !aggregate {
            return Err(finalize(out, violations));
        }
    }

    // Adjacency by handle index.
    let mut outgoing: HashMap<usize, Vec<usize>> = HashMap::new();
    for (f, t, _l) in sn.edges {
        outgoing.entry(f.0 as usize).or_default().push(t.0 as usize);
    }

    // Invariant 1 — cycle.
    if let Some(path) = find_cycle_indices(sn.nodes.len(), &outgoing) {
        out.cycle_path = Some(
            path.into_iter()
                .map(|i| {
                    sn.nodes
                        .get(i)
                        .map_or_else(|| format!("n{i}"), |n| n.id.clone())
                })
                .collect(),
        );
        violations.push(InvariantViolation::Cycle);
        if !aggregate {
            return Err(finalize(out, violations));
        }
    }

    // Invariant 3 — per-node fan-out (explicit outgoing + parallel forks).
    let max_fanout = usize::try_from(config.max_fanout).unwrap_or(usize::MAX);
    for (idx, n) in sn.nodes.iter().enumerate() {
        let explicit = outgoing.get(&idx).map_or(0, Vec::len);
        let parallel = sn.parallel_fanout.get(idx).copied().unwrap_or(1);
        let fan = explicit.max(parallel);
        if fan > max_fanout {
            out.fanout_actual = Some(fan);
            out.fanout_max = Some(max_fanout);
            out.fanout_node_id = Some(n.id.clone());
            violations.push(InvariantViolation::FanoutExceeded);
            if !aggregate {
                return Err(finalize(out, violations));
            }
            break;
        }
    }

    // Invariant 8 (Phase-1 stopgap) — max ITERATE nesting depth. The builder
    // tracks the chain depth on each node; exceeding the cap fires here.
    let max_nest = usize::try_from(config.max_iterate_nest_depth).unwrap_or(usize::MAX);
    let mut worst: (usize, usize) = (0, 0);
    for (idx, depth) in sn.iterate_depth.iter().copied().enumerate() {
        if depth > worst.0 {
            worst = (depth, idx);
        }
    }
    if worst.0 > max_nest {
        out.iterate_nest_depth_actual = Some(worst.0);
        out.iterate_nest_depth_max = Some(max_nest);
        // Walk back the chain via reverse-adjacency to produce a path string.
        out.iterate_nest_path = Some(
            sn.nodes
                .iter()
                .enumerate()
                .filter_map(|(i, n)| {
                    if sn.iterate_depth.get(i).copied().unwrap_or(0) > 0
                        && matches!(n.kind, PrimitiveKind::Iterate)
                    {
                        Some(n.id.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        );
        violations.push(InvariantViolation::IterateNestDepth);
        if !aggregate {
            return Err(finalize(out, violations));
        }
    }

    // Invariant 2 — max depth. Depth = count of CALL primitives on the
    // longest path (per `docs/ENGINE-SPEC.md` §4 and R1 triage:
    // handler-call depth is what the capability grant configures, not raw
    // longest-path length). Skipped if we've reported a cycle (walk would
    // not terminate on an arbitrary SCC).
    if !violations.contains(&InvariantViolation::Cycle) {
        let max_depth = usize::try_from(config.max_depth).unwrap_or(usize::MAX);
        if let Some(longest) = longest_path_indices(sn.nodes.len(), &outgoing) {
            let call_count = longest
                .iter()
                .filter(|i| {
                    sn.nodes
                        .get(**i)
                        .is_some_and(|n| matches!(n.kind, PrimitiveKind::Call))
                })
                .count();
            if call_count > max_depth {
                out.depth_actual = Some(call_count);
                out.depth_max = Some(max_depth);
                out.longest_path = Some(
                    longest
                        .into_iter()
                        .map(|i| {
                            sn.nodes
                                .get(i)
                                .map_or_else(|| format!("n{i}"), |n| n.id.clone())
                        })
                        .collect(),
                );
                violations.push(InvariantViolation::DepthExceeded);
                if !aggregate {
                    return Err(finalize(out, violations));
                }
            }
        }
    }

    // Invariant 9 — determinism. A handler declared deterministic rejects
    // any primitive whose classification is non-deterministic.
    if sn.deterministic {
        for n in sn.nodes {
            if !n.kind.is_deterministic() {
                violations.push(InvariantViolation::Determinism);
                if !aggregate {
                    let mut err = RegistrationError::new(InvariantViolation::Determinism);
                    err.fanout_node_id = Some(n.id.clone());
                    return Err(err);
                }
                break;
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(finalize(out, violations))
    }
}

fn finalize(mut err: RegistrationError, violations: Vec<InvariantViolation>) -> RegistrationError {
    if violations.len() >= 2 {
        err.kind = InvariantViolation::Cycle; // placeholder; overwritten below
        let list: Vec<u8> = violations.iter().map(invariant_number).collect();
        err.violated_invariants = Some(list);
        // Per the `registration_catch_all_populates_violated_list` test, the
        // code becomes E_INV_REGISTRATION — encoded as a new variant kind.
        err.kind = InvariantViolation::Registration;
    } else if let Some(v) = violations.into_iter().next() {
        err.kind = v;
    }
    err
}

fn invariant_number(v: &InvariantViolation) -> u8 {
    match v {
        InvariantViolation::Cycle => 1,
        InvariantViolation::DepthExceeded => 2,
        InvariantViolation::FanoutExceeded => 3,
        InvariantViolation::TooManyNodes => 5,
        InvariantViolation::TooManyEdges => 6,
        InvariantViolation::Determinism => 9,
        InvariantViolation::ContentHash => 10,
        InvariantViolation::Registration => 12,
        InvariantViolation::IterateMaxMissing
        | InvariantViolation::IterateNestDepth
        | InvariantViolation::IterateBudget => 8,
        InvariantViolation::Attribution => 14,
        InvariantViolation::Immutability => 13,
    }
}

// ---------------------------------------------------------------------------
// Graph algorithms — DFS cycle detection + DAG longest path.
// ---------------------------------------------------------------------------

fn find_cycle(nodes: &[OperationNode], outgoing: &HashMap<&str, Vec<&str>>) -> Option<Vec<String>> {
    let mut color: HashMap<&str, u8> = HashMap::new(); // 0=unvisited,1=grey,2=black
    for n in nodes {
        if color.get(n.id.as_str()).copied().unwrap_or(0) == 0 {
            let mut stack: Vec<(&str, usize)> = vec![(n.id.as_str(), 0)];
            let mut path: Vec<&str> = Vec::new();
            color.insert(n.id.as_str(), 1);
            path.push(n.id.as_str());
            while let Some(&(cur, idx)) = stack.last() {
                let empty: Vec<&str> = Vec::new();
                let neighbors = outgoing.get(cur).unwrap_or(&empty);
                if idx < neighbors.len() {
                    stack.last_mut().expect("just pushed").1 = idx + 1;
                    let next = neighbors[idx];
                    match color.get(next).copied().unwrap_or(0) {
                        0 => {
                            color.insert(next, 1);
                            path.push(next);
                            stack.push((next, 0));
                        }
                        1 => {
                            // Back edge — cycle. Extract the slice of `path`
                            // from `next` onward plus `next` itself to close.
                            let start = path.iter().position(|p| *p == next).unwrap_or(0);
                            let mut cycle: Vec<String> =
                                path[start..].iter().map(|s| (*s).to_string()).collect();
                            cycle.push(next.to_string());
                            return Some(cycle);
                        }
                        _ => {}
                    }
                } else {
                    color.insert(cur, 2);
                    path.pop();
                    stack.pop();
                }
            }
        }
    }
    None
}

fn find_cycle_indices(n_nodes: usize, outgoing: &HashMap<usize, Vec<usize>>) -> Option<Vec<usize>> {
    let mut color = vec![0_u8; n_nodes];
    for start in 0..n_nodes {
        if color[start] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        let mut path: Vec<usize> = vec![start];
        color[start] = 1;
        while let Some(&(cur, idx)) = stack.last() {
            let empty: Vec<usize> = Vec::new();
            let neighbors = outgoing.get(&cur).unwrap_or(&empty);
            if idx < neighbors.len() {
                stack.last_mut().expect("just pushed").1 = idx + 1;
                let next = neighbors[idx];
                match color.get(next).copied().unwrap_or(0) {
                    0 => {
                        color[next] = 1;
                        path.push(next);
                        stack.push((next, 0));
                    }
                    1 => {
                        let pos = path.iter().position(|p| *p == next).unwrap_or(0);
                        let mut cycle: Vec<usize> = path[pos..].to_vec();
                        cycle.push(next);
                        return Some(cycle);
                    }
                    _ => {}
                }
            } else {
                color[cur] = 2;
                path.pop();
                stack.pop();
            }
        }
    }
    None
}

fn longest_path(
    nodes: &[OperationNode],
    outgoing: &HashMap<&str, Vec<&str>>,
) -> Option<Vec<String>> {
    // Iterative DFS memoizing longest path starting from each node. Since
    // the graph is a DAG (Invariant 1 already ran), memoization is safe.
    let mut best_from: HashMap<&str, Vec<&str>> = HashMap::new();
    // Process in reverse-topological order via iterative post-order DFS.
    let mut visited: HashSet<&str> = HashSet::new();
    let mut order: Vec<&str> = Vec::new();
    for n in nodes {
        if visited.contains(n.id.as_str()) {
            continue;
        }
        let mut stack: Vec<(&str, usize)> = vec![(n.id.as_str(), 0)];
        while let Some(&(cur, idx)) = stack.last() {
            if idx == 0 && !visited.insert(cur) {
                stack.pop();
                continue;
            }
            let empty: Vec<&str> = Vec::new();
            let neighbors = outgoing.get(cur).unwrap_or(&empty);
            if idx < neighbors.len() {
                stack.last_mut().expect("just pushed").1 = idx + 1;
                let next = neighbors[idx];
                if !visited.contains(next) {
                    stack.push((next, 0));
                }
            } else {
                order.push(cur);
                stack.pop();
            }
        }
    }

    // Post-order -> process children before parents, so traversing `order`
    // gives us children first. Compute best_from[cur] = cur + best child.
    for &cur in &order {
        let empty: Vec<&str> = Vec::new();
        let neighbors = outgoing.get(cur).unwrap_or(&empty);
        let best_child: Option<&Vec<&str>> = neighbors
            .iter()
            .filter_map(|c| best_from.get(c))
            .max_by_key(|p| p.len());
        let mut path = vec![cur];
        if let Some(child) = best_child {
            path.extend_from_slice(child);
        }
        best_from.insert(cur, path);
    }

    best_from
        .values()
        .max_by_key(|p| p.len())
        .map(|p| p.iter().map(|s| (*s).to_string()).collect())
}

fn longest_path_indices(
    n_nodes: usize,
    outgoing: &HashMap<usize, Vec<usize>>,
) -> Option<Vec<usize>> {
    let mut best_from: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut visited = vec![false; n_nodes];
    let mut order: Vec<usize> = Vec::new();
    for start in 0..n_nodes {
        if visited[start] {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        visited[start] = true;
        while let Some(&(cur, idx)) = stack.last() {
            let empty: Vec<usize> = Vec::new();
            let neighbors = outgoing.get(&cur).unwrap_or(&empty);
            if idx < neighbors.len() {
                stack.last_mut().expect("just pushed").1 = idx + 1;
                let next = neighbors[idx];
                if let Some(v) = visited.get(next).copied()
                    && !v
                {
                    visited[next] = true;
                    stack.push((next, 0));
                }
            } else {
                order.push(cur);
                stack.pop();
            }
        }
    }
    for &cur in &order {
        let empty: Vec<usize> = Vec::new();
        let neighbors = outgoing.get(&cur).unwrap_or(&empty);
        let best_child: Option<&Vec<usize>> = neighbors
            .iter()
            .filter_map(|c| best_from.get(c))
            .max_by_key(|p| p.len());
        let mut path = vec![cur];
        if let Some(child) = best_child {
            path.extend_from_slice(child);
        }
        best_from.insert(cur, path);
    }
    best_from.values().max_by_key(|p| p.len()).cloned()
}

fn node_parallel_fanout(_n: &OperationNode) -> usize {
    // Finalized Subgraph has no parallel-fanout annotation; callers that
    // need the check take the builder path. Return 1 to fall back to the
    // explicit-outgoing edge count.
    1
}

// ---------------------------------------------------------------------------
// Invariant 10 — canonical byte encoding (order-independent).
// ---------------------------------------------------------------------------

/// Produce the canonical DAG-CBOR byte encoding of a subgraph.
///
/// Nodes and edges are sorted before encoding so the resulting bytes depend
/// only on the final set of nodes and edges, not on construction order. This
/// is the key property Invariant 10 guarantees — two builders that produce
/// the same final structure via different paths hash to the same CID.
///
/// # Errors
///
/// Returns [`CoreError::Serialize`] on DAG-CBOR failure.
pub(crate) fn canonical_subgraph_bytes(sg: &Subgraph) -> Result<Vec<u8>, CoreError> {
    // Project to serializable form. Nodes sorted by id + kind; edges by
    // (from, to, label).
    let mut nodes: Vec<CanonNode<'_>> = sg
        .nodes
        .iter()
        .map(|n| CanonNode {
            id: &n.id,
            kind: kind_tag(n.kind),
            properties: &n.properties,
        })
        .collect();
    nodes.sort_by(|a, b| (a.id, a.kind).cmp(&(b.id, b.kind)));

    let mut edges: Vec<CanonEdge<'_>> = sg
        .edges
        .iter()
        .map(|(f, t, l)| CanonEdge {
            from: f,
            to: t,
            label: l,
        })
        .collect();
    edges.sort_by(|a, b| (a.from, a.to, a.label).cmp(&(b.from, b.to, b.label)));

    let view = CanonView {
        handler_id: &sg.handler_id,
        nodes: &nodes,
        edges: &edges,
    };

    // `serde_ipld_dagcbor` canonicalizes CBOR map keys (length-first sort)
    // at encode time, so the on-wire bytes depend only on the post-sort
    // node/edge order we enforced above.
    serde_ipld_dagcbor::to_vec(&view).map_err(|e| CoreError::Serialize(format!("{e}")))
}

fn kind_tag(k: PrimitiveKind) -> &'static str {
    match k {
        PrimitiveKind::Read => "READ",
        PrimitiveKind::Write => "WRITE",
        PrimitiveKind::Transform => "TRANSFORM",
        PrimitiveKind::Branch => "BRANCH",
        PrimitiveKind::Iterate => "ITERATE",
        PrimitiveKind::Wait => "WAIT",
        PrimitiveKind::Call => "CALL",
        PrimitiveKind::Respond => "RESPOND",
        PrimitiveKind::Emit => "EMIT",
        PrimitiveKind::Sandbox => "SANDBOX",
        PrimitiveKind::Subscribe => "SUBSCRIBE",
        PrimitiveKind::Stream => "STREAM",
    }
}

#[derive(serde::Serialize)]
struct CanonNode<'a> {
    id: &'a str,
    kind: &'static str,
    properties: &'a BTreeMap<String, Value>,
}

#[derive(serde::Serialize)]
struct CanonEdge<'a> {
    from: &'a str,
    to: &'a str,
    label: &'a str,
}

#[derive(serde::Serialize)]
struct CanonView<'a> {
    handler_id: &'a str,
    nodes: &'a [CanonNode<'a>],
    edges: &'a [CanonEdge<'a>],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invariant_number_lookup_is_consistent() {
        assert_eq!(invariant_number(&InvariantViolation::Cycle), 1);
        assert_eq!(invariant_number(&InvariantViolation::FanoutExceeded), 3);
        assert_eq!(invariant_number(&InvariantViolation::TooManyNodes), 5);
        assert_eq!(invariant_number(&InvariantViolation::IterateNestDepth), 8);
    }

    /// Invariant 10 — canonical bytes are order-independent.
    #[test]
    fn cid_order_independent_over_edges() {
        let n1 = OperationNode::new("a", PrimitiveKind::Read);
        let n2 = OperationNode::new("b", PrimitiveKind::Transform);
        let sg1 = Subgraph {
            handler_id: "h".into(),
            nodes: vec![n1.clone(), n2.clone()],
            edges: vec![("a".into(), "b".into(), "next".into())],
            deterministic: false,
        };
        let sg2 = Subgraph {
            handler_id: "h".into(),
            // Same edges + nodes but the nodes vec is reversed.
            nodes: vec![n2, n1],
            edges: vec![("a".into(), "b".into(), "next".into())],
            deterministic: false,
        };
        assert_eq!(
            canonical_subgraph_bytes(&sg1).expect("encode"),
            canonical_subgraph_bytes(&sg2).expect("encode")
        );
    }

    #[test]
    fn handle_constructor_unused_in_checker() {
        // Smoke: NodeHandle is small and Copy — invariants never consume it.
        let h = NodeHandle(0);
        let _ = h;
    }
}
