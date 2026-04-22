//! Phase 2a R3 security — ExecutionState DAG-CBOR round-trip + CID
//! stability (G3-A seed per ucca-1 / sec-r1-1).
//!
//! **Attack class covered.** ExecutionState is a new durable security
//! boundary. If its DAG-CBOR round-trip is not bit-stable, an attacker
//! can exploit the non-determinism to manufacture payloads that canonicalise
//! differently on suspend vs resume, breaking the envelope's `payload_cid`
//! integrity guard (sec-r1-1 / §9.1 resume-step-1).
//!
//! **Proptest contract (from plan §4.2 + ucca R2 seed).**
//! - Random-depth `attribution_chain` (1..=N AttributionFrames; N picked
//!   to exercise Phase-6 AI-assistant delegation chains).
//! - Random `pinned_subgraph_cids` (sorted, deduped — the canonical form).
//! - Random `context_binding_snapshots`.
//! - Random `resumption_principal_cid`.
//! - Random `frame_stack` shape.
//!
//! Assertions:
//!   1. Encode → decode round-trip equals the original (structural
//!      equality).
//!   2. Re-encoding after decode hashes to the same CID (bit-stability
//!      under canonical DAG-CBOR).
//!   3. `payload_cid` recomputed from re-encoded payload bytes equals the
//!      envelope's claimed `payload_cid` — the §9.1 step-1 integrity
//!      guard's happy path.
//!
//! Case count: 10_000 (dialed via `PROPTEST_CASES` env var; CI default;
//! nightly fuzz bumps higher).
//!
//! **Red-phase contract.** G3-A has not yet landed `ExecutionStateEnvelope`
//! / `ExecutionStatePayload` / `AttributionFrame`. The test file compiles
//! but the entire proptest is `#[ignore]`d behind a pending marker. Once
//! G3-A lands, drop the `#[ignore]`; proptest runs 10k cases.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::default())]

    /// G3-A seed: ExecutionStatePayload DAG-CBOR round-trip preserves
    /// structural equality AND CID (bit-stable canonicalisation).
    ///
    /// Marked `#[ignore]` until the struct shapes land in G3-A.
    #[test]
    #[ignore = "phase-2a-pending: ExecutionStateEnvelope / ExecutionStatePayload / AttributionFrame land in G3-A per plan §9.1. Drop #[ignore] once the types are serde-derived."]
    fn exec_state_round_trip_preserves_cid(
        // Chain depth 1..=8 covers Phase-1 single-frame through Phase-6
        // AI-delegation chains. Deeper chains are bounded by ucca-6's
        // max_chain_depth to avoid CPU-DoS on resume.
        chain_depth in 1u8..=8,
        // pinned_subgraph_cids count.
        pin_count in 0u8..=8,
        // context_binding count.
        ctx_count in 0u8..=4,
    ) {
        // Target shape (G3-A):
        //
        //     use benten_eval::exec_state::{ExecutionStateEnvelope,
        //         ExecutionStatePayload, AttributionFrame};
        //     use benten_core::Cid;
        //
        //     let frames: Vec<AttributionFrame> = (0..chain_depth).map(|i| {
        //         AttributionFrame {
        //             actor_cid: deterministic_cid_for(format!("actor-{i}")),
        //             handler_cid: deterministic_cid_for(format!("handler-{i}")),
        //             capability_grant_cid: deterministic_cid_for(format!("grant-{i}")),
        //         }
        //     }).collect();
        //
        //     let pins: Vec<Cid> = (0..pin_count)
        //         .map(|i| deterministic_cid_for(format!("pin-{i}")))
        //         .collect::<std::collections::BTreeSet<_>>()
        //         .into_iter()
        //         .collect();  // sorted+deduped
        //
        //     let ctx: Vec<(String, Cid, Vec<u8>)> = (0..ctx_count)
        //         .map(|i| (format!("k{i}"), deterministic_cid_for(format!("c{i}")), vec![i; 8]))
        //         .collect();
        //
        //     let payload = ExecutionStatePayload {
        //         attribution_chain: frames,
        //         pinned_subgraph_cids: pins,
        //         context_binding_snapshots: ctx,
        //         resumption_principal_cid: deterministic_cid_for("resumer"),
        //         frame_stack: Vec::new(),  // exercise the empty case
        //         frame_index: 0,
        //     };
        //
        //     // 1. Round-trip via DAG-CBOR.
        //     let bytes_1 = serde_ipld_dagcbor::to_vec(&payload).unwrap();
        //     let decoded: ExecutionStatePayload =
        //         serde_ipld_dagcbor::from_slice(&bytes_1).unwrap();
        //     prop_assert_eq!(decoded, payload.clone());
        //
        //     // 2. Re-encode is bit-identical.
        //     let bytes_2 = serde_ipld_dagcbor::to_vec(&decoded).unwrap();
        //     prop_assert_eq!(bytes_1, bytes_2,
        //         "DAG-CBOR canonicalisation must be bit-stable");
        //
        //     // 3. Envelope CID integrity.
        //     let envelope = ExecutionStateEnvelope::from_payload(payload.clone())
        //         .unwrap();
        //     let recomputed = envelope.recompute_payload_cid();
        //     prop_assert_eq!(envelope.payload_cid, recomputed,
        //         "payload_cid must recompute stably from re-encoded bytes");
        //
        // Until G3-A lands, the assertions below keep the proptest valid.
        // They don't exercise any real assertion — the `#[ignore]` keeps
        // the test from running, but the proptest! macro requires SOME
        // body. Keep it minimal and typed.
        prop_assert!(chain_depth as u32 + pin_count as u32 + ctx_count as u32 <= 255);
    }
}
