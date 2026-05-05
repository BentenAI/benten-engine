//! R3-A RED-PHASE proptest pin: UCAN chain attenuation never widens
//! authority OR time-window (G14-A1 + G14-B canary smoke;
//! r2-test-landscape §3.A cluster + plan §4 seed).
//!
//! Pin sources:
//! - r2-test-landscape §2.2 G14-A1 row +
//!   `prop_ucan_chain_attenuation_never_widens_authority`; §3.A CLR-2
//!   cluster; cap-major-1.
//! - r2-test-landscape §11 row +
//!   `prop_ucan_chain_time_window_never_widens` (R4-FP-added pin
//!   closing `.addl/phase-3/r4-r1-test-coverage-completeness.json`
//!   finding `tcc-r1-5`).
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

    #[test]
    #[ignore = "RED-PHASE: G14-A1 wave-4a + G14-B wave-4b — fills UCAN chain time-window (nbf/exp) attenuation"]
    fn prop_ucan_chain_time_window_never_widens(
        chain_length in 2usize..=5usize,
        time_window_seed in any::<u64>(),
    ) {
        // R4-FP-added pin closing tcc-r1-5 (R2 §11 + §3.A CLR-2 row
        // enumerated `prop_ucan_chain_time_window_never_widens` as a
        // distinct component pin from
        // `prop_ucan_chain_attenuation_never_widens_authority`; the
        // 10k-cases proptest cohort for the time-window-specific
        // variant of the attenuation property).
        //
        // G14-A1 + G14-B implementer wires this:
        //
        //   let chain = build_chain_with_time_windows_from_seed(
        //       chain_length, time_window_seed,
        //   );
        //   let result = benten_id::ucan::validate_chain(&chain);
        //   match result {
        //       Err(benten_id::ucan::ChainError::TimeWindowWidened { .. }) => {
        //           // Property: chain WAS widening the (nbf, exp)
        //           // window relative to an ancestor; this is the safe
        //           // outcome.
        //       }
        //       Err(benten_id::ucan::ChainError::AttenuationViolated { .. }) => {
        //           // Authority widening (sister property) is also a
        //           // safe rejection.
        //       }
        //       Ok(()) => {
        //           // Property: leaf (nbf, exp) MUST be a SUBSET of
        //           // every ancestor's (nbf, exp) — i.e., leaf.nbf >=
        //           // ancestor.nbf AND leaf.exp <= ancestor.exp for
        //           // every ancestor in the chain.
        //           let leaf = &chain[0];
        //           for ancestor in &chain[1..] {
        //               prop_assert!(
        //                   leaf.nbf() >= ancestor.nbf(),
        //                   "leaf nbf {} earlier than ancestor nbf {} — \
        //                    chain validated but DOES widen time-window \
        //                    (nbf side)",
        //                   leaf.nbf(), ancestor.nbf()
        //               );
        //               prop_assert!(
        //                   leaf.exp() <= ancestor.exp(),
        //                   "leaf exp {} later than ancestor exp {} — \
        //                    chain validated but DOES widen time-window \
        //                    (exp side)",
        //                   leaf.exp(), ancestor.exp()
        //               );
        //           }
        //       }
        //       Err(other) => {
        //           // Other typed errors (signature failure, malformed
        //           // token, etc.) are acceptable outcomes for arbitrary
        //           // chains; only TimeWindowWidened (or
        //           // AttenuationViolated covering both axes) is the
        //           // property assertion.
        //       }
        //   }
        //
        // OBSERVABLE consequence across 10 000 cases × 2-5 link chains
        // = up to 50 000 (link, ancestor) pairs: ZERO chains validate
        // while widening their (nbf, exp) time-window relative to any
        // ancestor. Defends against the cap-major-1 attack class
        // restricted to the time-window axis: a malformed/malicious
        // chain that preserves authority-set inclusion but extends its
        // own validity window past an ancestor's expiry (or starts
        // earlier than an ancestor's not-before) would pass the
        // sister `prop_ucan_chain_attenuation_never_widens_authority`
        // proptest but fail this one. Sibling at §3.A CLR-2 cluster.
        let _ = (chain_length, time_window_seed);
        unimplemented!(
            "G14-A1 + G14-B implementer wires chain-time-window-attenuation invariant proptest"
        );
    }
}
