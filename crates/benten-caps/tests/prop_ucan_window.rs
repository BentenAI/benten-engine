//! R3-B RED-PHASE proptest pin for `benten-caps` UCANBackend
//! nbf/exp time-window enforcement (G14-B wave-4b; cap-major-1 +
//! CLR-2).
//!
//! Pin source: r2-test-landscape §2.2 G14-B row
//! `ucan_nbf_exp_time_window_proptest_at_g14_b_durable_layer`;
//! cap-major-1 + CLR-2.
//!
//! ## CLR-2 cross-lens cluster
//!
//! The keypair-side proptest at `prop_ucan_chain_attenuation_never_widens_authority`
//! (G14-A1, R3-A) covers attenuation + time-window propagation at the
//! chain-walk site. This sibling proptest covers the same surface AT
//! THE DURABLE LAYER — feeding through `UCANBackend::validate_chain`
//! against a real durable store. Both tests are load-bearing because
//! the chain-walk site is where Phase-3 produces 24+ producer/consumer
//! drift instances per `feedback_3_plus_recurrence_deep_sweep`.
//!
//! ## RED-PHASE discipline
//!
//! Stays `#[ignore]`'d until G14-B implementer un-ignores. Per
//! §3.6b pim-2, the proptest body must drive the production durable
//! validate_chain entry point + assert that arbitrary nbf/exp +
//! current-time triplets produce the expected Ok/Err outcome.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-B — cap-major-1 + CLR-2 — UCAN nbf/exp proptest at durable layer"]
fn ucan_nbf_exp_time_window_proptest_at_g14_b_durable_layer() {
    // cap-major-1 + CLR-2 pin. G14-B implementer wires this:
    //
    //   use proptest::prelude::*;
    //   proptest! {
    //       #![proptest_config(ProptestConfig::with_cases(10_000))]
    //       #[test]
    //       fn prop_ucan_window_durable(
    //           nbf in 0u64..1_000_000_000,
    //           lifetime in 60u64..86_400,
    //           offset in 0u64..200_000,
    //       ) {
    //           let exp = nbf.saturating_add(lifetime);
    //           let current = nbf.saturating_add(offset);
    //
    //           let store_dir = tempfile::tempdir().unwrap();
    //           let backend = benten_caps::UCANBackend::open(store_dir.path()).unwrap();
    //
    //           let issuer = benten_id::keypair::Keypair::generate();
    //           let ucan = benten_id::ucan::Ucan::builder()
    //               .issuer(issuer.public_key().to_did())
    //               .audience(...)
    //               .nbf(nbf)
    //               .exp(exp)
    //               .sign(&issuer).unwrap();
    //           backend.install_proof(&ucan).unwrap();
    //
    //           let result = backend.validate_chain_at(&[ucan], current);
    //           // Invariant: backend Ok ↔ current is within [nbf, exp]:
    //           let in_window = current >= nbf && current < exp;
    //           prop_assert_eq!(result.is_ok(), in_window);
    //       }
    //   }
    //
    // OBSERVABLE consequence: across 10 000 random (nbf, lifetime,
    // current-time) triplets, the durable-layer validate result
    // matches the abstract in-window predicate. Defends against
    // off-by-one / boundary-condition bugs at the time-window enforce
    // site. CLR-2 cluster pin per crypto-blocker-2 closure.
    unimplemented!(
        "G14-B wires proptest 10k random nbf/exp/current triplets against UCANBackend::validate_chain_at"
    );
}
