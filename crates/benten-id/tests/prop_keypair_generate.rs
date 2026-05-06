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
