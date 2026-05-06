//! G14-A2 wave-4a' — proptest pin: arbitrary malformed VC input must
//! not panic the verifier (`crypto-minor-1`).

#![allow(clippy::unwrap_used)]

use benten_id::vc::{TrustDomain, verify_bytes_in_trust_domain};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]
    #[test]
    fn prop_vc_verification_arbitrary_malformed_input_no_panic(
        input in proptest::collection::vec(any::<u8>(), 0..2048)
    ) {
        // The verifier MUST reject (typed Err) but NEVER panic on
        // arbitrary byte input. Defends against parser DoS + memory-
        // corruption via untrusted VC bytes.
        let trust_domain = TrustDomain::empty();
        let _ = verify_bytes_in_trust_domain(&input, &trust_domain);
        // No assertion — survival of the proptest is the assertion.
    }
}
