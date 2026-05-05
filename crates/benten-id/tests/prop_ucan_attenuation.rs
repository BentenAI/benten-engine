//! R3-A RED-PHASE proptest pin: UCAN chain attenuation never widens
//! authority (G14-A1 + G14-B canary smoke; r2-test-landscape §3.A
//! cluster + plan §4 seed).
//!
//! Pin source: r2-test-landscape §2.2 G14-A1 row
//! `prop_ucan_chain_attenuation_never_widens_authority`; §3.A CLR-2
//! cluster; cap-major-1.
//!
//! ## Property under test
//!
//! For any 2+ -link UCAN delegation chain, the EFFECTIVE granted
//! authority of the leaf token MUST be a SUBSET (proper or improper)
//! of every ancestor's granted authority. Concretely: if parent grants
//! `{(/zone/posts, read)}`, child cannot grant `{(/zone/posts, write)}`
//! and cannot grant `{(/zone/admin, read)}`.
//!
//! ## Counts
//!
//! 10 000 cases. Each case generates an arbitrary chain of 2-5 links
//! with arbitrary capabilities at each link, and asserts that
//! `validate_chain` either:
//!
//! 1. Rejects with `AttenuationViolated` (chain has at least one link
//!    that widens authority), OR
//! 2. Accepts AND the leaf's authority is a subset of every ancestor's.
//!
//! Never: accepts with authority that exceeds an ancestor.

#![allow(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    unreachable_code,
    reason = "RED-PHASE proptest stubs; G14-A1 + G14-B implementer wire real bodies + drop these allows"
)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    #[ignore = "RED-PHASE: G14-A1 wave-4a + G14-B wave-4b — fills UCAN chain validation"]
    fn prop_ucan_chain_attenuation_never_widens_authority(
        chain_length in 2usize..=5usize,
        capability_seed in any::<u64>(),
    ) {
        // G14-A1 + G14-B implementer wires this:
        //
        //   let chain = build_chain_from_seed(chain_length, capability_seed);
        //   let result = benten_id::ucan::validate_chain(&chain);
        //   match result {
        //       Err(benten_id::ucan::ChainError::AttenuationViolated { .. }) => {
        //           // Property: chain WAS widening; this is the safe outcome.
        //       }
        //       Ok(()) => {
        //           // Property: leaf authority MUST be subset of every ancestor.
        //           let leaf_authority = chain[0].capabilities();
        //           for ancestor in &chain[1..] {
        //               for cap in leaf_authority {
        //                   prop_assert!(
        //                       ancestor.capabilities_imply(cap),
        //                       "leaf authority {cap:?} not implied by ancestor {ancestor:?} \
        //                        — chain validated but DOES widen authority"
        //                   );
        //               }
        //           }
        //       }
        //       Err(other) => {
        //           // Other typed errors (signature failure, expired, etc.)
        //           // are acceptable outcomes for arbitrary chains; only
        //           // AttenuationViolated is the property assertion.
        //       }
        //   }
        //
        // OBSERVABLE consequence across 10 000 cases × 2-5 link chains
        // = up to 50 000 link checks: ZERO chains validate while
        // widening authority. Defends against the cap-major-1 attack
        // class where a malformed/malicious chain slips through with
        // wider authority than its root grant.
        let _ = (chain_length, capability_seed);
        unimplemented!("G14-A1 + G14-B implementer wires chain-attenuation invariant proptest");
    }
}
