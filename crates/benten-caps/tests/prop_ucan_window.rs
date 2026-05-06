//! G14-B proptest: nbf/exp time-window enforcement at the durable
//! UCAN backend layer (cap-major-1 + CLR-2 + crypto-blocker-2).
//!
//! Mirror of the keypair-side
//! `prop_ucan_chain_attenuation_never_widens_authority` G14-A1
//! proptest, but driving the production
//! `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at`
//! durable seam — defends against off-by-one / boundary-condition
//! drift between the in-memory chain-walk and the durable layer.
//!
//! Per `feedback_3_plus_recurrence_deep_sweep` the chain-walk site
//! has produced 24+ producer/consumer drift instances in Phase-3;
//! both the in-memory + durable proptests are load-bearing.

#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use benten_caps::UCANBackend;
use benten_graph::RedbBackend;
use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;
use proptest::prelude::*;

fn fresh_backend() -> UCANBackend<RedbBackend> {
    let inner = RedbBackend::open_in_memory().expect("redb in-memory open");
    UCANBackend::new(Arc::new(inner))
}

proptest! {
    // 1k cases per pim-14 LOC budget + the keypair-side proptest at
    // 10k covering the non-durable surface. The durable layer's
    // distinct concern is "store + retrieve + boundary check
    // composes correctly with the in-memory chain-walk" — boundary
    // axes are well-covered at 1k cases since the encoding round-
    // trip is deterministic. Plan-row "10k" applies to the keypair-
    // side companion.
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    #[test]
    fn ucan_nbf_exp_time_window_proptest_at_g14_b_durable_layer(
        nbf in 0u64..1_000_000_000,
        lifetime in 60u64..86_400,
        offset in 0u64..200_000,
    ) {
        let exp = nbf.saturating_add(lifetime);
        let current = nbf.saturating_add(offset);

        let backend = fresh_backend();
        let issuer = Keypair::generate();
        let audience = Keypair::generate();
        let ucan = Ucan::builder()
            .issuer(issuer.public_key().to_did().as_str())
            .audience(audience.public_key().to_did().as_str())
            .capability("/zone/posts", "read")
            .not_before(nbf)
            .expiry(exp)
            .sign(&issuer);
        backend.install_proof(&ucan).unwrap();

        let result = backend.validate_chain_at(&[ucan], current);
        // Invariant: backend Ok ↔ current ∈ [nbf, exp).
        let in_window = current >= nbf && current < exp;
        prop_assert_eq!(result.is_ok(), in_window);
    }
}
