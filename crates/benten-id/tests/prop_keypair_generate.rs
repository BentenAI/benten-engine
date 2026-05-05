//! R3-A RED-PHASE proptest pin: `Keypair::generate()` produces distinct
//! keypairs across 1 000 invocations (`crypto-major-2`).
//!
//! Pin source: `.addl/phase-3/r2-test-landscape.md` §2.2 G14-A1 row
//! `prop_keypair_generate_distinct_across_1k_calls`; plan §3 G14-A1
//! must-pass column.
//!
//! ## RED-PHASE discipline
//!
//! Test stays `#[ignore]`'d until G14-A1 wave-4a fills `benten_id::keypair::Keypair`
//! with the OS-CSPRNG-pinned `generate()` impl. Implementer un-ignores +
//! confirms the proptest run actually executes 1 000 cases.
//!
//! Counts as ~1 of the 10-12 proptests in the Phase-3 test budget per
//! r2-test-landscape §1 category table; budgeted at 1 000 cases (not 10 000)
//! because each `generate()` calls `getrandom` / `OsRng` which is
//! comparatively slow vs in-memory state-machine proptests, and because
//! the assertion is collision-style (uniqueness across N samples) which
//! converges at lower iteration counts than property-style invariant
//! checks.

#![allow(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    unreachable_code,
    reason = "RED-PHASE proptest stubs; G14-A1 implementer wires real bodies + drops these allows"
)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    #[test]
    #[ignore = "RED-PHASE: G14-A1 wave-4a fills benten-id::keypair::Keypair::generate"]
    fn prop_keypair_generate_distinct_across_1k_calls(_seed in any::<u64>()) {
        // G14-A1 implementer wires this:
        //
        //   let mut seen = std::collections::HashSet::new();
        //   for _ in 0..2 {
        //       let kp = benten_id::keypair::Keypair::generate();
        //       let pk_bytes: [u8; 32] = kp.public_key().to_bytes();
        //       prop_assert!(seen.insert(pk_bytes),
        //           "Keypair::generate() returned a duplicate public key — \
        //            CSPRNG seeding may be broken (crypto-major-2 BLOCKER)");
        //   }
        //
        // OBSERVABLE consequence: across 1 000 proptest cases × 2
        // generate() calls per case = 2 000 distinct public keys.
        // A deterministic-seed regression would emit duplicates within
        // the first ~100 cases.
        //
        // The `_seed` parameter is unused but pinned via `any::<u64>()`
        // so proptest has a strategy (proptest requires at least one
        // parameter; the assertion does not depend on it).
        let _ = _seed;
        unimplemented!(
            "G14-A1 wires HashSet uniqueness assertion across 2 generate() calls per proptest case"
        );
    }
}
