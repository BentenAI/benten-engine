//! R3-C RED-PHASE proptest pin: MST diff correctness under concurrent
//! writes (G16-C wave-6b; plan §4 seed).
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
//! 10 000 cases.

#![allow(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    unreachable_code,
    reason = "RED-PHASE proptest stubs; G16-C implementer wires real bodies + drops these allows"
)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    #[ignore = "RED-PHASE: G16-C wave-6b — plan §4 seed — MST diff correctness"]
    fn prop_mst_diff_correctness_under_concurrent_writes(
        n_keys in 8usize..=64usize,
        divergence_seed in any::<u64>(),
        write_pattern in any::<u64>(),
    ) {
        // G16-C implementer wires this:
        //
        //   let (mst_a, mst_b) = build_msts_with_concurrent_writes(
        //       n_keys, divergence_seed, write_pattern);
        //   let _rounds = run_mst_diff_to_convergence(&mut mst_a, &mut mst_b);
        //   prop_assert_eq!(mst_a.root_cid(), mst_b.root_cid(),
        //       "MST diff must converge for any divergence pattern; \
        //        n_keys={n_keys}, divergence={divergence_seed}, writes={write_pattern}");
        //
        // OBSERVABLE consequence across 10 000 cases × varied divergence
        // patterns: ZERO non-converging cases.
        let _ = (n_keys, divergence_seed, write_pattern);
        unimplemented!("G16-C wires MST diff convergence proptest");
    }
}
