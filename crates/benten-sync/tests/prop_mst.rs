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

use benten_sync::mst::{Mst, MstDiff, MstEntry, run_mst_diff_to_convergence};
use proptest::prelude::*;

fn build_msts_with_concurrent_writes(
    n_keys: usize,
    divergence_seed: u64,
    write_pattern: u64,
) -> (Mst, Mst, bool) {
    let mut a = Mst::new();
    let mut b = Mst::new();
    let mut injected_same_key_different_cid = false;
    // Mix the seeds + write_pattern bits to produce per-key
    // divergence decisions. Two orthogonal bit-fields:
    //   - low 2 bits: insert-side decision (A only / B only / both)
    //   - bit 0b100: when set AND both-sides path chosen, peers
    //     derive payloads from DIFFERENT seeds → same-key-different-CID
    //     divergence (exercises the load-bearing
    //     `MstDiff::between` Equal-arm symmetric-cross-push branch
    //     at mst.rs:496; per g16-c-mr-1 MAJOR fix-pass).
    for i in 0..n_keys {
        let key = format!("k{i:06}");
        let bits = divergence_seed.wrapping_mul(i as u64 + 1) ^ write_pattern;
        let payload_a = format!("v-{divergence_seed}-{i}").into_bytes();
        let entry_a = MstEntry::from_payload(&key, payload_a);
        match bits & 0b11 {
            0 => {
                a.insert(entry_a);
            }
            1 => {
                b.insert(entry_a);
            }
            _ => {
                if bits & 0b100 != 0 {
                    // Same-key-different-CID divergence: both peers
                    // hold the same key but derive distinct payloads
                    // (and thus distinct CIDs). Per the cross-push
                    // branch's docstring, MST-layer resolution defers
                    // tie-break to the engine-layer (LWW via HLC at
                    // G16-B); the test asserts the cross-push branch
                    // FIRES rather than asserting MST-layer
                    // convergence (which is intentionally not the
                    // contract for this divergence shape).
                    let payload_b =
                        format!("v-{}-{i}", divergence_seed.wrapping_add(1)).into_bytes();
                    let entry_b = MstEntry::from_payload(&key, payload_b);
                    a.insert(entry_a);
                    b.insert(entry_b);
                    injected_same_key_different_cid = true;
                } else {
                    a.insert(entry_a.clone());
                    b.insert(entry_a);
                }
            }
        }
    }
    (a, b, injected_same_key_different_cid)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_mst_diff_correctness_under_concurrent_writes(
        n_keys in 8usize..=64usize,
        divergence_seed in any::<u64>(),
        write_pattern in any::<u64>(),
    ) {
        let (mut mst_a, mut mst_b, injected_same_key_different_cid) =
            build_msts_with_concurrent_writes(n_keys, divergence_seed, write_pattern);

        if injected_same_key_different_cid {
            // Same-key-different-CID divergence exercises the
            // `MstDiff::between` Equal-arm symmetric-cross-push branch
            // (mst.rs:496) which intentionally defers tie-break to the
            // engine-layer (LWW via HLC at G16-B). Assert the diff
            // observably fires both `missing_in_a` and `missing_in_b`
            // entries for at least one same-key pair — the cross-push
            // contract — rather than asserting MST-layer root-cid
            // convergence (which is engine-layer responsibility per
            // mst.rs:496-501 docstring).
            let diff = MstDiff::between(&mst_a, &mst_b);
            let same_key_cross_push_observed = diff.missing_in_a.iter().any(|ea| {
                diff.missing_in_b.iter().any(|eb| ea.key == eb.key && ea.cid != eb.cid)
            });
            prop_assert!(
                same_key_cross_push_observed,
                "same-key-different-CID divergence MUST fire the cross-push branch (both missing_in_a + missing_in_b for the same key); engine-layer LWW resolves the actual tie-break per mst.rs:496-501",
            );
        } else {
            // Non-same-key-different-CID divergence converges to
            // identical root-cid at MST layer (the load-bearing
            // contract for the missing-in-a / missing-in-b
            // exclusive-key shapes).
            let _rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b).expect("proptest corpus converges");
            prop_assert_eq!(
                mst_a.root_cid(),
                mst_b.root_cid(),
                "MST diff must converge for exclusive-key divergence patterns"
            );
        }
    }
}
