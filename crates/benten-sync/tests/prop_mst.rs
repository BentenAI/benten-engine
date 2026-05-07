//! G16-C wave-6b LANDED proptest pin: MST diff correctness under
//! concurrent writes per plan §4 seed.
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-C row
//!   `prop_mst_diff_correctness_under_concurrent_writes`.
//! - plan §4 seed.
//!
//! ## Property under test
//!
//! For any sequence of concurrent writes to two MSTs by N writers
//! under arbitrary divergence patterns, MST diff converges to the
//! same root CID across both peers.
//!
//! ## Counts
//!
//! 10 000 cases (proptest config below).
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `Mst::insert` + `run_mst_diff_to_convergence`
//! API. Each generated case performs a real divergent-write +
//! convergence round-trip + asserts root equivalence.

#![allow(clippy::unwrap_used)]

use benten_sync::mst::{Mst, MstEntry, run_mst_diff_to_convergence};
use proptest::prelude::*;

fn build_msts_with_concurrent_writes(
    n_keys: usize,
    divergence_seed: u64,
    write_pattern: u64,
) -> (Mst, Mst) {
    let mut a = Mst::new();
    let mut b = Mst::new();
    // Mix the seeds + write_pattern bits to produce per-key
    // divergence decisions.
    for i in 0..n_keys {
        let key = format!("k{i:06}");
        let payload = format!("v-{divergence_seed}-{i}").into_bytes();
        let entry = MstEntry::from_payload(&key, payload);
        let bits = divergence_seed.wrapping_mul(i as u64 + 1) ^ write_pattern;
        match bits & 0b11 {
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_mst_diff_correctness_under_concurrent_writes(
        n_keys in 8usize..=64usize,
        divergence_seed in any::<u64>(),
        write_pattern in any::<u64>(),
    ) {
        let (mut mst_a, mut mst_b) =
            build_msts_with_concurrent_writes(n_keys, divergence_seed, write_pattern);
        let _rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
        prop_assert_eq!(
            mst_a.root_cid(),
            mst_b.root_cid(),
            "MST diff must converge for any divergence pattern"
        );
    }
}
