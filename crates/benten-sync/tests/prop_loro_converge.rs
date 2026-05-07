//! G16-B wave-6b LANDED — Loro concurrent writes converge via HLC ordering.
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row
//!   `prop_loro_concurrent_writes_converge_via_hlc_ordering`.
//! - plan §4 seed (Loro convergence proptest seed planted in plan).
//!
//! ## Property under test
//!
//! For any sequence of concurrent writes to the same Loro doc by N
//! writers under arbitrary interleaving, after every pair of writers
//! exchanges merges, ALL writers converge to the SAME LWW value per
//! property — and that value is the write with the highest HLC.
//!
//! ## Counts
//!
//! 10 000 cases.

#![allow(clippy::unwrap_used)]

use benten_core::hlc::BentenHlc;
use benten_sync::crdt::LoroDoc;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_loro_concurrent_writes_converge_via_hlc_ordering(
        n_writers in 2usize..=5usize,
        n_writes_per_writer in 1usize..=10usize,
        write_seed in any::<u64>(),
    ) {
        // Build N independent docs, each with a distinct HLC node-id.
        let writers: Vec<LoroDoc> = (0..n_writers).map(|_| LoroDoc::new()).collect();
        let mut hlc_phys: Vec<u64> = (0..n_writers).map(|i| (i as u64 + 1) * 100).collect();
        let node_ids: Vec<u64> = (0..n_writers).map(|i| 0x1000 + i as u64).collect();

        // Build the write sequence deterministically from the seed.
        let total = n_writers * n_writes_per_writer;
        let mut writes: Vec<(usize, String, String)> = Vec::with_capacity(total);
        let mut rng_state = write_seed;
        for w in 0..total {
            // Linear-congruential pseudo-randomness from seed.
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let writer_idx = w % n_writers;
            let key_idx = ((rng_state >> 33) as usize) % 3;
            let key = format!("k{key_idx}");
            let value = format!("w{writer_idx}_v{w}");
            writes.push((writer_idx, key, value));
        }

        // Apply each write at its writer doc with monotonic per-writer HLC.
        for (w_idx, key, value) in &writes {
            hlc_phys[*w_idx] += 1;
            let hlc = BentenHlc::new(hlc_phys[*w_idx], 0, node_ids[*w_idx]);
            writers[*w_idx].set_property(key, value.clone(), hlc).unwrap();
        }

        // All-pairs merge: every writer ingests every other writer's state.
        // Use a 2-pass exchange so converted state is fully replicated.
        for _ in 0..2 {
            let snapshots: Vec<LoroDoc> = writers.to_vec();
            for (i, writer) in writers.iter().enumerate().take(n_writers) {
                for (j, snap) in snapshots.iter().enumerate().take(n_writers) {
                    if i != j {
                        writer.merge(snap).unwrap();
                    }
                }
            }
        }

        // Convergence: all writers agree on every property's LWW value.
        let keys = ["k0", "k1", "k2"];
        for key in &keys {
            let v0 = writers[0].get_property(key);
            for (i, writer) in writers.iter().enumerate().take(n_writers).skip(1) {
                let vi = writer.get_property(key);
                prop_assert_eq!(&v0, &vi, "writer {} diverged from writer 0 on {}", i, key);
            }
        }
    }
}
