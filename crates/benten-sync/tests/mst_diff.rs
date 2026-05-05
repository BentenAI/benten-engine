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
    // OBSERVABLE consequence: MST diff converges in O(log n)
    // rounds across diverged MSTs. Defends against the failure
    // shape where MST diff degrades to O(n) under specific
    // divergence patterns.
    unimplemented!("G16-C wires MST diff O(log n) convergence assertion");
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
