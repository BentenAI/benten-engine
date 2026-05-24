//! BFS-canonical-path walker over a [`Spec`].
//!
//! Per RATIFIED §"8 substantive design refinements" #3: the walker is
//! itself a [`crate::Subgraph`] shipped **once** in `benten-core`, composed of
//! the existing 12 operation primitives only (Spike F decomposition:
//! `READ + TRANSFORM + BRANCH + ITERATE`). **No new `PrimitiveKind`
//! variant is minted** (CLAUDE.md baked-in #1 12-primitive-irreducibility).
//!
//! # Path-as-data
//!
//! Per RATIFIED §R4 + Spike H+1.1: BFS-order is the canonical arrival
//! order; the path is **data carried into the AuthorizationGrant**, not
//! a contract the recipient re-derives. Per RATIFIED §R5 + Spike E
//! Interpretation B: a multi-path-reachable Node emits **once per arrival
//! path** with distinct [`super::StructuralPath`]s — the path-tagged-keys
//! feature for selective-share.
//!
//! # Fail-closed discipline
//!
//! Per Spike F's never-silent-fallback finding (P2P-mainstream choice;
//! Veilid/MLS/Nostr-NIP-44): unresolvable roots ⇒ typed
//! [`SubgraphSpecError::CidMissing`]; self-referential specs ⇒ typed
//! [`SubgraphSpecError::SelfReferentialCycle`]; depth bounds ⇒ silent
//! truncation at the bound (callers that prefer the typed error invoke
//! [`Spec::max_depth`] reflection + reject).

extern crate alloc;

use alloc::collections::{BTreeSet, VecDeque};
use alloc::string::ToString;
use alloc::vec::Vec;

use super::errors::SubgraphSpecError;
use super::spec::{Spec, StructuralPath};
use crate::{Cid, OperationNode, PrimitiveKind, Subgraph};

// ---------------------------------------------------------------------------
// WalkResult
// ---------------------------------------------------------------------------

/// The result of [`walk`]: an ordered enumeration of `(Cid,
/// StructuralPath)` pairs.
///
/// Per RATIFIED §R4: ordering is BFS-first-reachable; tie-breaker is
/// edge-label byte-lex within a single source. Per §R5 + Spike E
/// Interpretation B: multi-path-reachable Nodes appear **multiple times**
/// — once per canonical arrival path. The recipient consumes
/// `enumerated` directly; no re-walk needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkResult {
    /// Ordered `(cid, path)` pairs in BFS-first-reachable order.
    pub enumerated: Vec<(Cid, StructuralPath)>,
}

// ---------------------------------------------------------------------------
// walk
// ---------------------------------------------------------------------------

/// Walk a [`Spec`] producing a [`WalkResult`].
///
/// The walker is the single shipped substrate — callers never re-implement
/// BFS. See [`walker_as_subgraph`] for the corresponding fractal-property
/// `Subgraph` decomposition.
///
/// # Fail-closed behavior
///
/// - A root that has neither outgoing edges declared in the Spec nor
///   appears as an edge target → [`SubgraphSpecError::CidMissing`]
///   (never-silent-fallback).
/// - A Spec whose roots include the Spec's own CID →
///   [`SubgraphSpecError::SelfReferentialCycle`] (recursive-self defense).
/// - `max_depth` is observed as silent BFS truncation at the bound (no
///   error fires; callers that need a typed-error mode pre-validate
///   against [`Spec::max_depth`]).
///
/// # Errors
/// See variant docs.
pub fn walk(spec: &Spec) -> Result<WalkResult, SubgraphSpecError> {
    // Self-referential cycle defense: the Spec's own CID must not appear
    // as one of its roots. Check before enumeration so an attacker can't
    // smuggle infinite-recursion via crafting the root-set.
    let spec_cid = spec.cid()?;
    if spec.roots().contains(&spec_cid) {
        return Err(SubgraphSpecError::SelfReferentialCycle);
    }

    // Pre-validate roots: a root must be resolvable (have at least one
    // outgoing edge declared OR appear as an edge target). Surfaces the
    // never-silent-fallback discipline.
    let edge_targets: BTreeSet<Cid> = spec
        .edges()
        .values()
        .flat_map(|m| m.values().copied())
        .collect();
    for root in spec.roots() {
        let has_out_edges = spec.edges().get(root).is_some_and(|m| !m.is_empty());
        let is_target = edge_targets.contains(root);
        if !has_out_edges && !is_target {
            return Err(SubgraphSpecError::CidMissing(*root));
        }
    }

    // BFS with per-arrival-path tagging. Multi-path-reachable Nodes emit
    // multiple times — once per canonical arrival path (Spike E
    // Interpretation B). We do NOT dedupe by CID; we dedupe by
    // `(cid, path)` pair to defend against true cycles.
    let mut enumerated: Vec<(Cid, StructuralPath)> = Vec::new();
    let mut visited: BTreeSet<(Cid, StructuralPath)> = BTreeSet::new();
    let mut frontier: VecDeque<(Cid, StructuralPath, u32)> = VecDeque::new();

    // Seed the frontier with each root at depth 0, ordered by the Spec's
    // sorted-root sequence (lex byte order). Each root has the empty
    // StructuralPath (the "I'm a root" path).
    for root in spec.roots() {
        frontier.push_back((*root, StructuralPath::root(), 0));
    }

    while let Some((cid, path, depth)) = frontier.pop_front() {
        if depth > spec.max_depth() {
            // Bound exceeded — silent BFS truncation per the doc; do not
            // enumerate.
            continue;
        }

        let key = (cid, path.clone());
        if visited.contains(&key) {
            // True structural cycle: same (cid, path) reached twice means
            // an edge cycle on the same edge-label chain — drop the
            // duplicate.
            continue;
        }
        visited.insert(key);

        // Apply Inclusion-dim predicate (Path (a) restricted-spec language).
        // For v1: SubgraphSpecRestriction::Unrestricted always passes;
        // SubgraphSpecRestriction::ByLabel narrows but is evaluated against
        // labels carried by the Spec's structural definition — at the
        // walker substrate level we emit every BFS-reached CID and leave
        // label-based filtering to downstream resolvers that have access
        // to the actual Node bodies. The walker-substrate emits CIDs;
        // downstream Subgraph-shape evaluation prunes by label.
        enumerated.push((cid, path.clone()));

        // Expand: enumerate outgoing edges of this CID in edge-label
        // byte-lex order (R4 BFS tie-breaker). depth == max_depth: don't
        // expand outgoing edges past the bound.
        if depth < spec.max_depth()
            && let Some(out_edges) = spec.edges().get(&cid)
        {
            for (label, target) in out_edges {
                let next_path = path.extended(label.clone());
                frontier.push_back((*target, next_path, depth + 1));
            }
        }
    }

    Ok(WalkResult { enumerated })
}

// ---------------------------------------------------------------------------
// walker_as_subgraph — the fractal-property `Subgraph` decomposition
// ---------------------------------------------------------------------------

/// Return the walker's own composition as a [`crate::Subgraph`] — the fractal
/// property per Spike F.
///
/// The decomposition uses **only** the existing 12 operation primitives;
/// specifically `READ + TRANSFORM + BRANCH + ITERATE`. No `WRITE` (the
/// walker is pure-read); no `SANDBOX` (no WASM host needed); no `EMIT /
/// SUBSCRIBE / STREAM / WAIT / CALL / RESPOND` (no side effects, no
/// suspension, no handler dispatch).
///
/// The composition is deterministic + byte-stable across invocations
/// (two calls produce equal canonical-bytes — load-bearing for the
/// "walker shipped ONCE" property per §1.A.FROZEN item 15(h)).
///
/// # Handler id
///
/// The walker subgraph's handler_id is the canonical
/// `"benten:subgraph_spec:walker"` — pinned by the
/// `tf3w_walker_is_a_subgraph_no_new_primitive_kind` test (P-2.2).
#[must_use]
pub fn walker_as_subgraph() -> Subgraph {
    // The BFS decomposition:
    //   r0 (READ)      — deref next-frontier CID
    //   t1 (TRANSFORM) — extract outgoing edge labels + targets
    //   b2 (BRANCH)    — visited-set membership predicate
    //   i3 (ITERATE)   — drain queue until empty
    //
    // Each OperationNode has a stable id; edges thread BFS one-pass.
    // The id strings and edge labels are constant — two calls produce
    // canonical-bytes-equal Subgraphs (byte-stability per P-2.3).
    Subgraph::new("benten:subgraph_spec:walker")
        .push_node_raw(OperationNode::new("r0_deref", PrimitiveKind::Read))
        .push_node_raw(OperationNode::new("t1_expand", PrimitiveKind::Transform))
        .push_node_raw(OperationNode::new("b2_visited", PrimitiveKind::Branch))
        .push_node_raw(OperationNode::new("i3_drain", PrimitiveKind::Iterate))
        .push_edge_raw("r0_deref", "t1_expand", "next")
        .push_edge_raw("t1_expand", "b2_visited", "next")
        .push_edge_raw("b2_visited", "i3_drain", "next")
        .push_edge_raw("i3_drain", "r0_deref", "loop")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn cid_for(label: &str) -> Cid {
        let digest = blake3::hash(label.as_bytes());
        Cid::from_blake3_digest(*digest.as_bytes())
    }

    #[test]
    fn walker_as_subgraph_handler_id_pin() {
        let sg = walker_as_subgraph();
        assert_eq!(sg.handler_id(), "benten:subgraph_spec:walker");
    }

    #[test]
    fn walker_as_subgraph_uses_only_four_primitives() {
        let sg = walker_as_subgraph();
        let allowed = [
            PrimitiveKind::Read,
            PrimitiveKind::Transform,
            PrimitiveKind::Branch,
            PrimitiveKind::Iterate,
        ];
        for op in sg.nodes() {
            assert!(allowed.contains(&op.kind), "kind {:?}", op.kind);
        }
        assert!(sg.nodes().len() >= 4);
    }

    #[test]
    fn walker_as_subgraph_byte_stable() {
        let a = walker_as_subgraph();
        let b = walker_as_subgraph();
        assert_eq!(
            a.to_canonical_bytes().unwrap(),
            b.to_canonical_bytes().unwrap()
        );
    }

    #[test]
    fn walk_phantom_root_cid_missing() {
        let phantom = cid_for("phantom-test");
        let spec = Spec::builder().with_root(phantom).build().expect("spec");
        let err = walk(&spec).expect_err("phantom");
        assert!(matches!(err, SubgraphSpecError::CidMissing(c) if c == phantom));
    }

    #[test]
    fn walk_bfs_simple_diamond() {
        let root = cid_for("R");
        let node_a = cid_for("A");
        let node_b = cid_for("B");
        let node_c = cid_for("C");
        let node_d = cid_for("D");
        let spec = Spec::builder()
            .with_root(root)
            .with_edge(root, "edge_A".to_string(), node_a)
            .with_edge(root, "edge_B".to_string(), node_b)
            .with_edge(node_a, "child".to_string(), node_c)
            .with_edge(node_b, "child".to_string(), node_d)
            .build()
            .expect("spec");
        let result = walk(&spec).expect("walk");
        let cids: Vec<Cid> = result.enumerated.iter().map(|(cid, _)| *cid).collect();
        assert_eq!(cids, alloc::vec![root, node_a, node_b, node_c, node_d]);
    }

    #[test]
    fn walk_multi_path_emits_distinct_paths() {
        let r = cid_for("MP-R");
        let a = cid_for("MP-A");
        let b = cid_for("MP-B");
        let x = cid_for("MP-X");
        let spec = Spec::builder()
            .with_root(r)
            .with_edge(r, "edge_to_A".to_string(), a)
            .with_edge(r, "edge_to_B".to_string(), b)
            .with_edge(a, "from_A_to_X".to_string(), x)
            .with_edge(b, "from_B_to_X".to_string(), x)
            .build()
            .expect("spec");
        let result = walk(&spec).expect("walk");
        let x_arrivals: Vec<&StructuralPath> = result
            .enumerated
            .iter()
            .filter(|(c, _)| *c == x)
            .map(|(_, p)| p)
            .collect();
        assert_eq!(x_arrivals.len(), 2);
        assert_ne!(x_arrivals[0], x_arrivals[1]);
    }

    #[test]
    fn walk_deterministic() {
        let r = cid_for("det-R");
        let a = cid_for("det-A");
        let spec = Spec::builder()
            .with_root(r)
            .with_edge(r, "edge".to_string(), a)
            .build()
            .expect("spec");
        let r1 = walk(&spec).unwrap();
        let r2 = walk(&spec).unwrap();
        assert_eq!(r1.enumerated, r2.enumerated);
    }

    #[test]
    fn walk_max_depth_bound() {
        let r = cid_for("d-R");
        let a = cid_for("d-A");
        let b = cid_for("d-B");
        let c = cid_for("d-C");
        let spec = Spec::builder()
            .with_root(r)
            .with_edge(r, "next".to_string(), a)
            .with_edge(a, "next".to_string(), b)
            .with_edge(b, "next".to_string(), c)
            .with_max_depth(2)
            .build()
            .expect("spec");
        let result = walk(&spec).expect("walk");
        // Depth 0 = R, depth 1 = A, depth 2 = B. C is at depth 3 ⇒ excluded.
        let cids: Vec<Cid> = result.enumerated.iter().map(|(c, _)| *c).collect();
        assert!(!cids.contains(&c));
        assert!(cids.contains(&r));
        assert!(cids.contains(&a));
        assert!(cids.contains(&b));
    }

    #[test]
    fn walk_path_carries_ordered_labels() {
        let r = cid_for("verif-R");
        let a = cid_for("verif-A");
        let c = cid_for("verif-C");
        let spec = Spec::builder()
            .with_root(r)
            .with_edge(r, "edge_root_to_a".to_string(), a)
            .with_edge(a, "edge_a_to_c".to_string(), c)
            .build()
            .expect("spec");
        let result = walk(&spec).expect("walk");
        let c_path = result
            .enumerated
            .iter()
            .find(|(cid, _)| *cid == c)
            .map(|(_, p)| p)
            .expect("c arrives");
        let labels: Vec<String> = c_path.edge_labels().iter().map(|s| s.to_string()).collect();
        assert_eq!(labels, alloc::vec!["edge_root_to_a", "edge_a_to_c"]);
    }
}
