//! TF-3w (R3-W2) — `SubgraphSpec` walker BFS-canonical-path enumeration.
//!
//! Family: TF-3w — G-CORE-3w `benten_core::subgraph_spec` (the walker IS
//! a Subgraph; BFS-canonical-path; path-tagged keys per Spike E
//! Interpretation B).
//!
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3w def
//! (line 334) + §1.A.FROZEN item 15(h) (line 139) + §1.A.FROZEN item
//! 15(f) (line 137; path-tagged keys, multiple paths give multiple keys)
//! + §5 D-list row D-4M-R4 (line 400) + D-4M-R5 (line 401).
//!
//! Ratified inputs (cannot be re-debated):
//!   `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//!   §R4 — "BFS-order, carry canonical path in AuthorizationGrant" +
//!   Spike E (path-tagged keys with one canonical-owner-path; multiple
//!   paths to same Node give multiple distinct keys = feature for
//!   selective-share) + Spike F (walker-as-a-Subgraph-shipped-once).
//!
//! R2-seed: R2-test-landscape.md §2 G-CORE-3w (P-1) — "BFS-canonical-path
//! enumeration: for a fixed graph topology, producer-walker output
//! matches the canonical BFS-path ordering documented at §1.A.FROZEN
//! item 15(h). Different produced-paths for the same Node (multi-path-
//! reachable) → different keys (path-tagged property of Spike E
//! Interpretation B)."
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3w
//! production surface = `benten_core::subgraph_spec::{Spec, walker::walk,
//! StructuralPath, WalkResult}`. The walker output is `WalkResult {
//! enumerated: Vec<(Cid, StructuralPath)>, ... }` where the order matches
//! BFS-first-reachable and the `StructuralPath` is data carried into
//! `AuthorizationGrant.key_material` per D-4M-R4. R5 implementer mints
//! `crates/benten-core/src/subgraph_spec/{mod.rs, spec.rs, walker.rs,
//! builder.rs, macros.rs, combinators.rs}` + un-ignores these pins.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NON-NEGOTIABLE R3 BRIEF-TEMPLATE CHECKLIST (literal lines):
//!  1. §3.5b HARDENED (pim-1) — public-shape change ⇒ adjacent-doc sweep
//!     by G-CORE-3w implementer (ENGINE-SPEC §7 + GLOSSARY +
//!     SECURITY-POSTURE walker-coupling).
//!  2. §3.6b sub-rule 4 (pim-2) — every pin = production-arm against
//!     the (future) `walk_subgraph_spec`, observable consequence (key
//!     bytes / ordered Vec) WOULD-FAIL-IF-NO-OP'd; per-arm specific
//!     (BFS-order / path-tagged / fail-closed / max_depth).
//!  3. §3.6e (pim-12) — each arm `#[ignore]`d with "un-ignore at G-CORE-3w".
//!  4. §3.6f (pim-18) — SHAPE-not-SUBSTANCE; production call-site enumerated
//!     (`benten_core::subgraph_spec::walker::walk`).
//!  5. §3.5g — N/A (Rust-only walker; the napi cross-language
//!     observers pin lives in W5 cross-wave integration tests per R2 §7).
//!  6-19. See `tf3b_restricted_spec_contains_decidable.rs` head comment
//!     for the full reproduced checklist; all apply.
//!
//! SHAPE-flag-don't-fake: `benten_core::subgraph_spec` does NOT exist at
//! HEAD. RED-PHASE — do NOT stub the surface to make this compile.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use benten_core::{Cid, Node, Value};
// RED: `benten_core::subgraph_spec` does NOT exist at HEAD. G-CORE-3w
// creates it. The walker is itself a `Subgraph` composed of the existing
// 12 operation primitives + shipped once in `benten_core` (Spike F's
// fractal-property finding).
use benten_core::subgraph_spec::{Spec, StructuralPath, WalkResult, walker};

fn cid_for(label: &str) -> Cid {
    // Deterministic per-label CID (cross-test-stable; mirrors the
    // benten-caps test pattern).
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}

fn node_with_label(label: &str) -> Node {
    use alloc::collections::BTreeMap;
    let mut p = BTreeMap::new();
    p.insert("name".to_string(), Value::text(label));
    Node::new(vec![label.to_string()], p)
}

// ---------------------------------------------------------------------------
// Arm P-1.1 — BFS order enumeration matches documented canonical order.
//
// Topology:        R
//                 / \
//                A   B
//                |   |
//                C   D
// Roots = [R].  BFS first-reachable order from R = [R, A, B, C, D].
// The walker produces paths in that order.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: for the simple A/B/C/D diamond above, the
/// walker enumerates Nodes in BFS-first-reachable order (R, then A
/// before B if `EDGE_AB` lex-precedes `EDGE_AC`, then breadth-second).
/// WOULD-FAIL on DFS, or alpha-ordered, or anchor-ordered walk.
#[test]
#[ignore = "un-ignore at G-CORE-3w: walker BFS-canonical-path ordering (P-1.1)"]
fn walker_enumerates_in_bfs_first_reachable_order() {
    let r = cid_for("R");
    let a = cid_for("A");
    let b = cid_for("B");
    let c = cid_for("C");
    let d = cid_for("D");

    // Build a Spec with R as the sole root + edges
    //   R --edge_A--> A,  R --edge_B--> B,  A --child--> C,  B --child--> D.
    let spec = Spec::builder()
        .with_root(r)
        .with_edge(r, "edge_A".to_string(), a)
        .with_edge(r, "edge_B".to_string(), b)
        .with_edge(a, "child".to_string(), c)
        .with_edge(b, "child".to_string(), d)
        .build()
        .expect("valid spec");

    let result: WalkResult = walker::walk(&spec).expect("walk produces enumerated paths");

    // BFS-first-reachable from [R]: depth-0 = [R]; depth-1 = [A, B] in
    // edge-label-lex order (edge_A < edge_B); depth-2 = [C (from A), D
    // (from B)]. The walker's canonical ordering is documented at
    // §1.A.FROZEN item 15(h) — BFS-order = canonical path.
    let cids: Vec<Cid> = result.enumerated.iter().map(|(c, _path)| *c).collect();
    assert_eq!(
        cids,
        vec![r, a, b, c, d],
        "BFS-canonical-path order (P-1.1)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.2 — Path-tagged keys: multi-path-reachable Node gets distinct
// StructuralPaths per arrival. (Spike E Interpretation B: different
// paths ⇒ different keys is a feature for selective-share.)
//
// Topology:    R
//             / \
//            A   B
//             \ /
//              X   (multi-path-reachable Node)
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: a Node `X` reachable from `R` via TWO distinct
/// edge-label paths (R→A→X via edge_AX vs R→B→X via edge_BX) appears in
/// the walker's enumerated output TWICE — once per canonical-arrival-
/// path. The StructuralPaths differ in edge-label sequence; this is
/// the load-bearing property R4 + Spike E rest on (different paths ⇒
/// different K(N) ⇒ selective-share). WOULD-FAIL if the walker
/// dedup-merges multi-path arrivals (a subtle bug class that would
/// SILENTLY collapse the path-tagged-keys feature).
#[test]
#[ignore = "un-ignore at G-CORE-3w: walker path-tagged multi-arrival enumeration (P-1.2)"]
fn walker_emits_distinct_paths_for_multi_path_reachable_node() {
    let r = cid_for("R");
    let a = cid_for("A");
    let b = cid_for("B");
    let x = cid_for("X");

    let spec = Spec::builder()
        .with_root(r)
        .with_edge(r, "edge_to_A".to_string(), a)
        .with_edge(r, "edge_to_B".to_string(), b)
        .with_edge(a, "from_A_to_X".to_string(), x)
        .with_edge(b, "from_B_to_X".to_string(), x)
        .build()
        .expect("valid spec");

    let result = walker::walk(&spec).expect("walk multi-path graph");

    // X must appear twice — once per arrival path. (NOT dedup-collapsed
    // into a single entry.)
    let x_arrivals: Vec<&StructuralPath> = result
        .enumerated
        .iter()
        .filter(|(c, _)| *c == x)
        .map(|(_, p)| p)
        .collect();
    assert_eq!(
        x_arrivals.len(),
        2,
        "multi-path-reachable Node has two arrivals (P-1.2)"
    );

    // The two paths must be DISTINCT (path-tagged).
    assert_ne!(
        x_arrivals[0], x_arrivals[1],
        "the two arrivals carry distinct StructuralPaths (P-1.2 / Spike E feature)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.3 — Producer-walker output IS canonical BFS — recipient walks
// the same BFS order from the SubgraphSpec + carries no separate
// algorithm contract. (D-4M-R4: "data-not-contract decouples the spec
// from the walker's internal algorithm.")
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: two independent walker invocations on the same
/// Spec produce byte-equal `(Cid, StructuralPath)` enumerations.
/// Deterministic + reproducible BFS — the recipient does NOT need to
/// re-derive the order; the path is data carried in the grant.
#[test]
#[ignore = "un-ignore at G-CORE-3w: walker determinism + path-IS-data (P-1.3)"]
fn walker_output_is_deterministic_path_is_data() {
    let r = cid_for("det-R");
    let a = cid_for("det-A");
    let b = cid_for("det-B");

    let spec = Spec::builder()
        .with_root(r)
        .with_edge(r, "alpha".to_string(), a)
        .with_edge(r, "beta".to_string(), b)
        .build()
        .expect("valid spec");

    let result_1 = walker::walk(&spec).expect("first walk");
    let result_2 = walker::walk(&spec).expect("second walk");

    assert_eq!(
        result_1.enumerated, result_2.enumerated,
        "walker output deterministic across invocations (P-1.3)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.4 — Canonical-path encoding round-trips: each StructuralPath
// in the walker's output is the same path the recipient would walk to
// derive K(N) per item 15(f) — the path encoding is verifiable.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: each StructuralPath in the walker's enumeration
/// carries an ordered sequence of edge-labels matching the BFS arrival
/// path. The recipient consumes this directly (no re-walk needed).
/// WOULD-FAIL if the implementer encoded paths as opaque indices that
/// the recipient cannot independently verify.
#[test]
#[ignore = "un-ignore at G-CORE-3w: StructuralPath verifiable encoding (P-1.4)"]
fn structural_path_carries_ordered_edge_labels() {
    let r = cid_for("verif-R");
    let a = cid_for("verif-A");
    let c = cid_for("verif-C");

    let spec = Spec::builder()
        .with_root(r)
        .with_edge(r, "edge_root_to_a".to_string(), a)
        .with_edge(a, "edge_a_to_c".to_string(), c)
        .build()
        .expect("valid spec");

    let result = walker::walk(&spec).expect("walk");

    // The path for C must be [edge_root_to_a, edge_a_to_c].
    let c_path: &StructuralPath = result
        .enumerated
        .iter()
        .find(|(cid, _)| *cid == c)
        .map(|(_, p)| p)
        .expect("C must be enumerated");

    let edge_labels: Vec<&str> = c_path.edge_labels();
    assert_eq!(
        edge_labels,
        vec!["edge_root_to_a", "edge_a_to_c"],
        "StructuralPath carries ordered edge-labels (P-1.4)"
    );
}
