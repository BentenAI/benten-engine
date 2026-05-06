//! G14-A1 wave-4a — `Keypair::generate` distinctness proptest (un-ignored).

#![allow(clippy::unwrap_used)]

use benten_id::keypair::Keypair;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    #[test]
    fn prop_keypair_generate_distinct_across_1k_calls(_seed in any::<u64>()) {
        // Each proptest case calls generate() twice and asserts the
        // two public keys differ. Across 1 000 cases × 2 generate()
        // calls = 2 000 distinct public keys. A deterministic-seed
        // regression would emit duplicates within the first ~100 cases.
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();
        prop_assert_ne!(
            kp1.public_key().to_bytes(),
            kp2.public_key().to_bytes(),
            "Keypair::generate() returned a duplicate public key — CSPRNG seeding may be broken (crypto-major-2 BLOCKER)"
        );
    }
}

/// Aggregate-set distinctness check: catches a regression where
/// `generate()` cycles through a small set of keys (e.g., a 16-key
/// PRNG-mod cycle) which the per-case pair check above would miss.
/// Per the G14-A1 mini-review MINOR finding: "1k-distinctness proptest
/// doesn't aggregate-set-detect collision."
#[test]
fn keypair_generate_aggregate_distinct_across_2k_calls() {
    use std::collections::HashSet;
    const CALLS: usize = 2_000;
    let mut seen: HashSet<[u8; 32]> = HashSet::with_capacity(CALLS);
    for _ in 0..CALLS {
        let kp = Keypair::generate();
        let pk = kp.public_key().to_bytes();
        assert!(
            seen.insert(pk),
            "Keypair::generate() emitted a duplicate public key within {CALLS} calls — \
             CSPRNG cycle detected (crypto-major-2 aggregate-set distinctness pin)"
        );
    }
    assert_eq!(seen.len(), CALLS);
}
