//! R3-C RED-PHASE proptest pin: Loro concurrent writes converge via
//! HLC ordering (G16-B wave-6b; plan §4 seed).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row
//!   `prop_loro_concurrent_writes_converge_via_hlc_ordering`.
//! - plan §4 seed (Loro convergence proptest seed planted in plan).
//!
//! ## Property under test
//!
//! For any sequence of concurrent writes to the same Loro doc +
//! property by N writers under arbitrary interleaving, after every
//! pair of writers exchanges merges, ALL writers converge to the
//! SAME value — and that value is the write with the highest HLC.
//!
//! ## Counts
//!
//! 10 000 cases.

#![allow(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    unreachable_code,
    reason = "RED-PHASE proptest stubs; G16-B implementer wires real bodies + drops these allows"
)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    #[ignore = "RED-PHASE: G16-B wave-6b — plan §4 seed — concurrent-write HLC convergence"]
    fn prop_loro_concurrent_writes_converge_via_hlc_ordering(
        n_writers in 2usize..=5usize,
        n_writes_per_writer in 1usize..=10usize,
        write_seed in any::<u64>(),
    ) {
        // G16-B implementer wires this:
        //
        //   let mut writers: Vec<(LoroDoc, Hlc)> = (0..n_writers)
        //       .map(|i| (LoroDoc::new(), Hlc::new(i as u64 + 0x1000)))
        //       .collect();
        //   let writes = build_write_sequence(write_seed, n_writers, n_writes_per_writer);
        //   for (writer_idx, key, value) in &writes {
        //       let (ref doc, ref mut hlc) = writers[*writer_idx];
        //       doc.set_property(key, value, hlc.now()).unwrap();
        //   }
        //   // All-pairs merge:
        //   for i in 0..n_writers {
        //       for j in 0..n_writers {
        //           if i != j {
        //               let donor = writers[j].0.clone();
        //               writers[i].0.merge(&donor).unwrap();
        //           }
        //       }
        //   }
        //   // Property: all writers converge on the SAME canonical bytes
        //   let canonical_bytes_set: std::collections::BTreeSet<_> =
        //       writers.iter().map(|(d, _)| d.to_canonical_bytes()).collect();
        //   prop_assert_eq!(canonical_bytes_set.len(), 1, "all writers must converge");
        //
        // OBSERVABLE consequence across 10 000 cases × up to 50
        // writes per case: ZERO divergent end-states. Defends
        // against the failure shape where Loro merges are
        // non-deterministic under concurrent writes.
        let _ = (n_writers, n_writes_per_writer, write_seed);
        unimplemented!("G16-B wires concurrent-writes-converge proptest");
    }
}
