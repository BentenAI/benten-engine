//! G16-C wave-6b LANDED pin: light-client mode-(a) verifies Node-CID
//! inclusion in subgraph root via Merkle proof per ROADMAP-2 distinct
//! deliverable.
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-C row
//!   `light_client_verifies_node_cid_inclusion_in_subgraph_root_via_merkle_proof`
//!   (renamed from `light_client_verifies_subgraph_root_without_full_replication`
//!   per R4-R2-FP-C ds-r4r2-3 — closes ds-r4-6 mode-(a/b/c) ambiguity).
//! - plan §3 G16-C row line "light-client verification API + Merkle
//!   proof construction + verification against published roots;
//!   works WITHOUT full subgraph download as distinct deliverable
//!   from MST diff".
//! - `ROADMAP-2` (light-client deliverable distinct from full-peer
//!   sync — separate exit criterion).
//!
//! ## Mode commitment per ds-r4r2-3 (closes ds-r4-6)
//!
//! Phase-3 commits to mode-(a) only. Architectural-absence pins for
//! modes (b) + (c) at this file's tail clarify intent for fresh
//! agents per the HARD-RULE rule-12 BELONGS-NAMED-NOW disposition
//! (named destination: FULL-ROADMAP.md Phase 4+ light-client
//! extensions).
//!
//! ## What this pins (distinct from the MST diff pin)
//!
//! `mst_light_client_verification_against_content_addressed_root`
//! pins the verification API + Merkle-proof construction. THIS pin
//! asserts that the light-client deliverable as a whole runs
//! WITHOUT full subgraph download — the distinguishing characteristic
//! per ROADMAP-2.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `LightClient::with_budget` +
//! `LightClient::bytes_consumed` API + asserts bandwidth is bounded
//! well below full-subgraph payload size.

#![allow(clippy::unwrap_used)]

use benten_sync::light_client::{BandwidthBudget, LightClient};
use benten_sync::mst::{Mst, MstEntry};

#[test]
fn light_client_verifies_node_cid_inclusion_in_subgraph_root_via_merkle_proof() {
    // ROADMAP-2 distinguishing characteristic: light-client verifies
    // membership using bandwidth << full subgraph bytes.

    // Build a 1-MiB-payload MST: 16 entries × 64 KiB each. Total
    // payload bytes = 1 MiB; the light-client must verify a single
    // entry's inclusion using bytes far below that.
    let mut full_peer_mst = Mst::new();
    let n_entries: usize = 16;
    let payload_per_entry: usize = 64 * 1024;
    let full_peer_total_bytes = n_entries * payload_per_entry;

    for i in 0..n_entries {
        let key = format!("/zone/posts/p{i:04}");
        let payload = vec![i as u8; payload_per_entry];
        full_peer_mst.insert(MstEntry::from_payload(key, payload));
    }
    let published_root = full_peer_mst.root_cid();

    // Light-client with a 64-KiB budget — far below the 1-MiB total
    // payload bytes of the full subgraph.
    let light_client_budget = BandwidthBudget::limit_bytes(64 * 1024);
    let lc = LightClient::with_budget(light_client_budget);

    let proof = full_peer_mst.merkle_proof_for("/zone/posts/p0008").unwrap();

    // Verify a specific subgraph path exists under the published root:
    let result = lc
        .verify(&published_root, "/zone/posts/p0008", &proof)
        .unwrap();
    assert!(result.verified);

    // OBSERVABLE consequence: bandwidth used STAYS within budget.
    assert!(
        lc.bytes_consumed() <= 64 * 1024,
        "light-client must verify within budget; consumed {} of 64KB cap",
        lc.bytes_consumed()
    );
    // OBSERVABLE consequence: bandwidth << full-peer total. We
    // require <1% (the actual ratio in practice for proofs that
    // exclude payload bytes is ~0.1%).
    assert!(
        lc.bytes_consumed() < full_peer_total_bytes / 100,
        "light-client bandwidth must be <1% of full-peer total; got {} of {full_peer_total_bytes}",
        lc.bytes_consumed()
    );
}

// =====================================================================
// Architectural-absence pins per ds-r4r2-3 — light-client modes (b) +
// (c) OOS for Phase-3, deferred to FULL-ROADMAP.md Phase-4+
// light-client extensions.
//
// These pins use #[ignore] with explicit OOS rationale (not
// unimplemented!()) — they are architectural-absence pins asserting
// modes (b) + (c) are deliberately NOT in Phase-3 scope. Inert at
// every wave; remain inert post-Phase-3 close.
// =====================================================================

#[test]
#[ignore = "ARCHITECTURAL-ABSENCE: light-client mode-(b) range-query proof is OOS for Phase-3; deferred to Phase-4+ per FULL-ROADMAP.md light-client extensions"]
fn light_client_mode_b_range_query_proof_oos_phase_3_deferred_to_phase_4() {
    // ds-r4r2-3 architectural-absence pin (closes ds-r4-6 mode-(b)
    // ambiguity). Phase-3 commits to mode-(a) only:
    // light_client_verifies_node_cid_inclusion_in_subgraph_root_via_merkle_proof
    // above. Mode-(b) range-query proofs require:
    //
    //   - Range-Merkle-proof construction (e.g. authenticated
    //     traversal of a sorted MST sub-range).
    //   - Bandwidth bounds for range-proofs scaling with range size.
    //   - Wire format for range-proof messages.
    //
    // Architectural decision: defer to Phase-4+. Phase-3's
    // thin-client protocol (D-PHASE-3-30) supports range queries via
    // fetch-then-verify each Node CID in the range against mode-(a)
    // inclusion proofs; dedicated range-Merkle-proof construction is
    // a Phase-4 optimization.
    //
    // OBSERVABLE-INTENT consequence: the absence of a mode-(b)
    // implementation is INTENTIONAL.
    //
    // Inert body — the pin is the #[ignore] rationale itself.
}

#[test]
#[ignore = "ARCHITECTURAL-ABSENCE: light-client mode-(c) signed checkpoint is OOS for Phase-3; deferred to Phase-4+ per FULL-ROADMAP.md light-client extensions"]
fn light_client_mode_c_signed_checkpoint_oos_phase_3_deferred_to_phase_4() {
    // ds-r4r2-3 architectural-absence pin (closes ds-r4-6 mode-(c)
    // ambiguity). Mode-(c) signed-checkpoint verification requires:
    //
    //   - Periodic checkpoint commitment from full peers signing
    //     the MST root state at a particular HLC time.
    //   - Signature verification + checkpoint freshness/replay
    //     defenses.
    //   - Checkpoint-publication infrastructure.
    //
    // Architectural decision: defer to Phase-4+. Phase-3's
    // thin-client + light-client model relies on direct
    // content-addressed verification against a full peer's published
    // root, not on intermediate signed checkpoints.
    //
    // OBSERVABLE-INTENT consequence: the absence of a mode-(c)
    // implementation is INTENTIONAL.
    //
    // Inert body — the pin is the #[ignore] rationale itself.
}
