//! perf-r6-9 regression guard for Inv-8 multiplicative cumulative budget on
//! diamond DAGs.
//!
//! ## Background
//!
//! Commit `a01302c` rewrote the per-node cumulative-budget walker in
//! `benten_eval::invariants::budget` from a recursive DFS (which re-walked
//! shared subtrees on diamond shapes — `O(2^V)` on a balanced fork-join DAG)
//! to a Kahn topological-DP pass (`O(V + E)`). The walker's docstring
//! (`crates/benten-eval/src/invariants/budget.rs:240-244`) referenced
//! "perf_inv_8_diamond_*" regression tests, but no such test file existed —
//! a grep confirmed the tests were promised but never landed.
//!
//! ## What this guards
//!
//! Constructs a 16-level diamond DAG: each level has two parallel "lane"
//! nodes; every node in level `i` feeds both lane nodes in level `i+1`. The
//! number of distinct root-to-sink paths therefore doubles at each level —
//! `2^16 = 65_536` paths through 34 nodes / 64 edges. The recursive walker
//! would visit every path independently and take seconds-to-minutes; the
//! Kahn DP pass walks the 34 nodes once.
//!
//! The wall-clock budget is intentionally generous (1 second) so a CI
//! runner under load is not flaky, while still catching any reintroduction
//! of an `O(2^V)` shape — the recursive walker on this DAG would exceed the
//! budget by orders of magnitude on any modern machine.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries diagnostic context; mirrors crate-internal API."
)]

use std::time::{Duration, Instant};

use benten_eval::{
    InvariantConfig, PrimitiveKind, Subgraph, SubgraphBuilder, invariants::validate_subgraph,
};

/// Number of diamond levels. `2^DIAMOND_LEVELS` is the path count a naive
/// recursive walker would visit; 16 → 65_536 paths is well past anything an
/// `O(V + E)` walker breaks a sweat over but blows up an `O(2^V)` walker
/// instantly.
const DIAMOND_LEVELS: usize = 16;

/// Wall-clock budget for the structural validation pass over the 16-deep
/// diamond. The Kahn DP path completes in single-digit milliseconds on a
/// developer laptop; 1s is generous enough to absorb a heavily-loaded CI
/// runner while still rejecting any reintroduction of the `O(2^V)`
/// recursive walker (which would take seconds-to-minutes on this DAG).
const PERF_BUDGET: Duration = Duration::from_secs(1);

/// Build a `DIAMOND_LEVELS`-deep diamond DAG.
///
/// Shape:
/// ```text
///        root
///        /  \
///     L0_a  L0_b      ← level 0
///       \  /  \  /
///     L1_a  L1_b      ← level 1
///       ...
///     L15_a  L15_b    ← level 15
///        \  /
///        sink
/// ```
///
/// Every node in level `i` has edges to BOTH lane nodes of level `i+1`,
/// creating `2^DIAMOND_LEVELS` distinct root-to-sink paths through the DAG.
/// Per-node fan-out is 2 (under the default cap of 16); longest path is
/// `DIAMOND_LEVELS + 2` nodes (under the default depth cap of 64); total
/// node count is `2 * DIAMOND_LEVELS + 2 = 34` (under the 4096 cap).
fn build_diamond_dag() -> Subgraph {
    let mut sb = SubgraphBuilder::new("diamond_perf");
    let root = sb.push_primitive("root", PrimitiveKind::Read);

    // Per-level lane handles.
    let mut prev_lanes: [_; 2] = [root, root];
    for level in 0..DIAMOND_LEVELS {
        let a = sb.push_primitive(format!("L{level}_a"), PrimitiveKind::Transform);
        let b = sb.push_primitive(format!("L{level}_b"), PrimitiveKind::Transform);
        // Connect every prev-level lane to BOTH new-level lanes. At level 0
        // both `prev_lanes` slots point at `root`, so we dedup to avoid
        // accidentally fanning root → a twice.
        let unique_prev: Vec<_> = if level == 0 {
            vec![prev_lanes[0]]
        } else {
            vec![prev_lanes[0], prev_lanes[1]]
        };
        for p in unique_prev {
            sb.add_edge(p, a);
            sb.add_edge(p, b);
        }
        prev_lanes = [a, b];
    }

    let sink = sb.push_primitive("sink", PrimitiveKind::Respond);
    sb.add_edge(prev_lanes[0], sink);
    sb.add_edge(prev_lanes[1], sink);

    // Use the unvalidated finalize so we can run `validate_subgraph` on the
    // finalized form below — that exercises the `cumulative_by_id` path
    // (the one whose docstring promised a regression test) directly.
    sb.build_unvalidated_for_test()
}

#[test]
fn perf_inv_8_diamond_does_not_explode() {
    let sg = build_diamond_dag();

    // Sanity-check the constructed DAG matches the documented shape. If
    // someone refactors `build_diamond_dag` and accidentally collapses the
    // diamond into a chain, the perf assertion below would still pass
    // trivially — these structural pins prevent that silent regression.
    assert_eq!(
        sg.nodes().len(),
        2 * DIAMOND_LEVELS + 2,
        "expected root + 2*levels + sink nodes"
    );
    // 1 root → 2 L0 = 2 edges + (DIAMOND_LEVELS-1) inter-level pairs ×
    // 2 prev × 2 new = 4 each + 2 sink edges. Total = 2 + 4*(L-1) + 2.
    let expected_edges = 2 + 4 * (DIAMOND_LEVELS - 1) + 2;
    assert_eq!(
        sg.edges().len(),
        expected_edges,
        "diamond edge count should be {expected_edges}"
    );

    let cfg = InvariantConfig::default();

    let start = Instant::now();
    let result = validate_subgraph(&sg, &cfg, false);
    let elapsed = start.elapsed();

    // The diamond is structurally sound (no Inv-8 factors anywhere — every
    // node is a Read/Transform/Respond contributing factor 1), so it must
    // pass. A failure here indicates an unrelated invariant regression, not
    // the perf bug this test guards.
    assert!(
        result.is_ok(),
        "diamond DAG must pass structural validation; got {result:?}"
    );

    // Perf pin — the recursive O(2^V) walker on this shape would take
    // seconds-to-minutes on any modern machine. The Kahn DP rewrite
    // completes in single-digit milliseconds. 1s is intentionally generous
    // for CI-runner variance while still catching any reintroduction of the
    // exponential walker.
    assert!(
        elapsed < PERF_BUDGET,
        "validate_subgraph on a 16-deep diamond DAG (2^16 = 65,536 paths) \
         took {elapsed:?}, exceeding the {PERF_BUDGET:?} regression budget. \
         This almost certainly means the Inv-8 cumulative-budget walker \
         has reverted to the pre-a01302c recursive shape that was O(2^V) \
         on diamond DAGs. See crates/benten-eval/src/invariants/budget.rs \
         cumulative_by_id."
    );
}
