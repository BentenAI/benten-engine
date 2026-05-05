//! R3-B RED-PHASE proptest pin: arbitrary malformed VC input must not
//! panic the verifier (G14-A2 wave-4a'; crypto-minor-1).
//!
//! Pin source: r2-test-landscape §2.2 G14-A2 row
//! `prop_vc_verification_arbitrary_malformed_input_no_panic`; crypto-minor-1.
//!
//! ## RED-PHASE discipline
//!
//! Stays `#[ignore]`'d at R3-B landing because `benten_id::vc::*` types
//! do not exist yet. G14-A2 wave-4a' implementer un-ignores AND replaces
//! the stub body with the real proptest body driving
//! `benten_id::vc::verify` under `arbitrary` byte-input. Per
//! §3.6b pim-2, the un-ignored proptest drives the production verify
//! entry point and asserts NO panic regardless of input shape.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 wave-4a' fills benten-id::vc — proptest no-panic"]
fn prop_vc_verification_arbitrary_malformed_input_no_panic() {
    // crypto-minor-1 pin. G14-A2 implementer wires this:
    //
    //   use proptest::prelude::*;
    //   proptest! {
    //       #![proptest_config(ProptestConfig::with_cases(10_000))]
    //       #[test]
    //       fn prop_vc_no_panic_arbitrary(input in proptest::collection::vec(any::<u8>(), 0..2048)) {
    //           // The verifier must reject (typed Err) but NEVER panic
    //           // on arbitrary byte input. Defends against parser DoS
    //           // + memory-corruption via untrusted VC bytes.
    //           let trust_domain = benten_id::vc::TrustDomain::empty();
    //           let _ = benten_id::vc::verify_bytes_in_trust_domain(&input, &trust_domain);
    //           // No assertion — survival of the proptest is the assertion.
    //       }
    //   }
    //
    // OBSERVABLE consequence: 10 000 random byte sequences (including
    // truncated DAG-CBOR, malformed JSON-LD, oversized fields) all
    // result in typed Err return — never a panic. This is the
    // load-bearing fuzz-style coverage for the VC parser surface.
    unimplemented!("G14-A2 wires proptest 10k arbitrary-byte no-panic against vc::verify_bytes");
}
