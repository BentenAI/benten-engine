//! G16-C wave-6b LANDED pins for Merkle Search Tree diff per
//! r2-test-landscape §2.4 G16-C + plan §3 G16-C row.
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
//! ## pim-2 §3.6b end-to-end discipline
//!
//! These tests drive the production `Mst` + `run_mst_diff_to_convergence`
//! entry points from the public `benten_sync::mst` API. The
//! convergence assertion is OBSERVABLE in the returned round count
//! and would FAIL if the diff protocol degraded to O(n).

#![allow(clippy::unwrap_used)]

use benten_sync::mst::{Mst, MstEntry, run_mst_diff_to_convergence};

fn build_diverged_msts(n: usize) -> (Mst, Mst) {
    let mut a = Mst::new();
    let mut b = Mst::new();
    // Roughly half the keys overlap; the rest split between A-only +
    // B-only by index parity. Producing a divergence pattern that
    // exercises both `missing_in_a` and `missing_in_b` paths.
    for i in 0..n {
        let key = format!("k{i:06}");
        let payload = format!("v{i}").into_bytes();
        let entry = MstEntry::from_payload(&key, payload);
        match i % 4 {
            0 => {
                a.insert(entry);
            }
            1 => {
                b.insert(entry);
            }
            _ => {
                a.insert(entry.clone());
                b.insert(entry);
            }
        }
    }
    (a, b)
}

fn build_depth4_branch8_corpus() -> (Mst, Mst) {
    // Canonical net-major-2 fixture: depth 4 / branch 8 ≈ 4096 nodes.
    // Drive divergence on every 7th and 11th entry so the diff is
    // non-trivial across both directions.
    let total = 4096;
    let mut a = Mst::new();
    let mut b = Mst::new();
    for i in 0..total {
        let key = format!("zone-{i:04}");
        let payload = format!("payload-{i}").into_bytes();
        let entry = MstEntry::from_payload(&key, payload);
        if i % 7 == 0 {
            a.insert(entry);
        } else if i % 11 == 0 {
            b.insert(entry);
        } else {
            a.insert(entry.clone());
            b.insert(entry);
        }
    }
    (a, b)
}

fn build_diverged_msts_with_divergent_count(n_total: usize, n_divergent: usize) -> (Mst, Mst) {
    let mut a = Mst::new();
    let mut b = Mst::new();
    for i in 0..n_total {
        let key = format!("k{i:06}");
        let payload = format!("v{i}").into_bytes();
        let entry = MstEntry::from_payload(&key, payload);
        if i < n_divergent {
            // Diverge: half in A only, half in B only.
            if i % 2 == 0 {
                a.insert(entry);
            } else {
                b.insert(entry);
            }
        } else {
            // Shared.
            a.insert(entry.clone());
            b.insert(entry);
        }
    }
    (a, b)
}

#[test]
fn mst_diff_two_peer_converges_in_log_n_rounds() {
    // plan §3 G16-C pin (closes ds-r4r2-4 n-shape ambiguity by
    // documenting `n = total Node count`).
    let n = 1024usize;
    let (mut mst_a, mut mst_b) = build_diverged_msts(n);
    let rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
    // Integer log2 (ceiling): for n that's a power of two, ilog2 is
    // exact; otherwise ilog2 + 1. Avoids `as f64` precision-loss
    // clippy warning.
    let log_n = if n.is_power_of_two() {
        n.ilog2() as usize
    } else {
        n.ilog2() as usize + 1
    };
    assert!(
        rounds <= log_n * 4,
        "MST diff convergence took {rounds} rounds for n={n}; expected O(log n) ~{log_n}"
    );
    assert_eq!(
        mst_a.root_cid(),
        mst_b.root_cid(),
        "post-convergence roots must match"
    );
}

#[test]
fn mst_diff_convergence_under_divergent_node_count_n_definition() {
    // ds-r4r2-4 sibling pin (closes ds-r4-7).
    let n_total = 4096;
    let n_divergent = 16;
    let (mut mst_a, mut mst_b) = build_diverged_msts_with_divergent_count(n_total, n_divergent);
    let rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
    let log_n_div = if n_divergent.is_power_of_two() {
        n_divergent.ilog2() as usize
    } else {
        n_divergent.ilog2() as usize + 1
    };
    assert!(
        rounds <= log_n_div * 4 + 2,
        "MST diff with n_total={n_total} n_div={n_divergent} took {rounds} rounds; expected O(log n_div) ~{log_n_div}"
    );
    assert_eq!(mst_a.root_cid(), mst_b.root_cid());
}

#[test]
fn mst_diff_convergence_o_log_n_for_corpus_with_depth_4_branch_8() {
    // net-major-2 canonical fixture pin.
    let (mut mst_a, mut mst_b) = build_depth4_branch8_corpus();
    let rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
    // For depth 4 / branch 8 → log2(4096) = 12; 4× constant gives
    // 48-round headroom. The MST diff implementation typically
    // converges in 1-2 rounds for the BTreeMap-backed shape, well
    // below the bound.
    assert!(
        rounds <= 48,
        "canonical depth-4/branch-8 MST diff converged in {rounds} rounds; expected <= 48"
    );
    assert_eq!(
        mst_a.root_cid(),
        mst_b.root_cid(),
        "post-convergence roots must match"
    );
}
