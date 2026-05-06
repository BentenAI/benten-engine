//! R3-C RED-PHASE pins for Merkle Search Tree diff (G16-C wave-6b;
//! per r2-test-landscape §2.4 G16-C + plan §3 G16-C row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-C rows
//!   `mst_diff_two_peer_converges_in_log_n_rounds` +
//!   `mst_diff_convergence_o_log_n_for_corpus_with_depth_4_branch_8`.
//! - plan §3 G16-C row line "Merkle Search Tree diff for subgraph
//!   sync; converges in O(log n) rounds; canonical fixture corpus
//!   depth 4 / branch 8 per net-major-2".
//! - `net-major-2` (canonical fixture corpus depth 4 / branch 8;
//!   convergence claim measurable).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-C wave-6b lands MST diff"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-C wave-6b — plan §3 G16-C — MST diff converges in O(log n) rounds"]
fn mst_diff_two_peer_converges_in_log_n_rounds() {
    // plan §3 G16-C pin. G16-C implementer wires this:
    //
    //   use benten_sync::mst::Mst;
    //   let n = 1024;
    //   let (mst_a, mst_b) = build_diverged_msts(n);
    //   let rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
    //   // O(log n) bound: rounds <= ceil(log2(n)) * constant
    //   let log_n = (n as f64).log2().ceil() as usize;
    //   assert!(rounds <= log_n * 4,
    //       "MST diff convergence took {rounds} rounds for n={n}; expected O(log n) ~{log_n}");
    //   assert_eq!(mst_a.root_cid(), mst_b.root_cid(), "post-convergence roots must match");
    //
    // ## n-definition (clarified per ds-r4r2-4 — closes ds-r4-7)
    //
    // **n = total Node count in the MST** (NOT divergent-Node count).
    // This is the n-shape used in the MST literature for tree-depth
    // bounds — log2(n) is the expected tree depth, and the diff
    // protocol's per-round work is bounded by tree depth times the
    // number of divergent paths. The 4x constant is a safety factor
    // covering:
    //
    //   - Worst-case branching-factor variance from the average
    //     log2-tree shape (real MSTs are not perfectly balanced).
    //   - Round-trip overhead per diff exchange (request + response).
    //   - Re-traversal when divergent paths share prefixes (each
    //     shared prefix may add up to one round of constant overhead).
    //
    // The 4x constant is empirical, derived from the
    // depth-4-branch-8 canonical fixture corpus measured at G16-C
    // landing time (see `mst_diff_convergence_o_log_n_for_corpus_with_depth_4_branch_8`
    // below for the canonical-fixture variant). Literature cite:
    // Auvolat & Taïani 2019 "Merkle Search Trees" §4 (MST diff
    // round-bound proof; the round-bound is O(log n) where n = total
    // tree node count).
    //
    // For divergent-Node-count cases (n_div < n_total) where the
    // diff is expected to be sub-log(n_total), see the optional
    // sibling pin `mst_diff_convergence_under_divergent_node_count_n_definition`
    // below.
    //
    // OBSERVABLE consequence: MST diff converges in O(log n)
    // rounds across diverged MSTs. Defends against the failure
    // shape where MST diff degrades to O(n) under specific
    // divergence patterns.
    unimplemented!("G16-C wires MST diff O(log n) convergence assertion");
}

#[test]
#[ignore = "RED-PHASE: G16-C wave-6b — ds-r4r2-4 — n-shape sibling pin: divergent-Node-count bound"]
fn mst_diff_convergence_under_divergent_node_count_n_definition() {
    // ds-r4r2-4 sibling pin (clarifies ds-r4-7 n-shape ambiguity).
    // The companion convergence bound: when divergent-Node-count
    // n_div is much smaller than total-Node-count n_total, the diff
    // protocol converges in O(log n_div) rounds — a tighter bound
    // than O(log n_total).
    //
    // G16-C implementer wires:
    //
    //   use benten_sync::mst::Mst;
    //   let n_total = 4096;          // total Node count (depth 4 / branch 8)
    //   let n_divergent = 16;         // only 16 nodes differ between peers
    //   let (mst_a, mst_b) = build_diverged_msts_with_divergent_count(
    //       n_total, n_divergent);
    //
    //   let rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
    //
    //   // O(log n_div) tighter bound when divergence is small:
    //   let log_n_div = (n_divergent as f64).log2().ceil() as usize;
    //   assert!(rounds <= log_n_div * 4 + 2,  // +2 for handshake + final ack
    //       "MST diff with n_total={n_total} n_div={n_divergent} \
    //        took {rounds} rounds; expected O(log n_div) ~{log_n_div}");
    //   assert_eq!(mst_a.root_cid(), mst_b.root_cid(),
    //       "post-convergence roots must match");
    //
    // OBSERVABLE consequence: MST diff observably exploits low
    // divergence — when only a small subset diverges between peers,
    // round count scales with divergent-Node-count, NOT total-Node-
    // count. Defends against the regression where diff overhead
    // scales with the full corpus regardless of divergence size.
    // Pinning both n-shapes side-by-side closes the ds-r4-7 n-shape
    // ambiguity definitively per ds-r4r2-4.
    unimplemented!("G16-C wires divergent-Node-count n-shape convergence bound per ds-r4r2-4");
}

#[test]
#[ignore = "RED-PHASE: G16-C wave-6b — net-major-2 — canonical depth-4/branch-8 corpus"]
fn mst_diff_convergence_o_log_n_for_corpus_with_depth_4_branch_8() {
    // net-major-2 pin. The canonical convergence fixture: depth 4,
    // branch 8 (~4096 nodes). G16-C implementer wires this:
    //
    //   let canonical_corpus = build_depth4_branch8_corpus();
    //   let (mst_a, mst_b) = diverge_corpus(&canonical_corpus, divergence_pattern());
    //   let rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
    //   // For depth 4, log2(4096) = 12; a 4x constant gives 48 max rounds:
    //   assert!(rounds <= 48,
    //       "canonical depth-4/branch-8 MST diff converged in {rounds} rounds; expected <= 48");
    //
    // OBSERVABLE consequence: the canonical fixture converges within
    // the documented bound; defends against regression on the
    // load-bearing fixture corpus.
    unimplemented!("G16-C wires depth-4/branch-8 canonical-fixture convergence assertion");
}
