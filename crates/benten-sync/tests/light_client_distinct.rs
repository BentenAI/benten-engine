//! R3-C RED-PHASE pin: light-client mode-(a) verifies Node-CID inclusion
//! in subgraph root via Merkle proof (G16-C wave-6b; ROADMAP-2 distinct
//! deliverable).
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
//! ds-r4-6 surfaced 3 candidate light-client verification modes:
//!
//! - **(a) Single-CID inclusion proof** — verify a single Node CID is
//!   included in the published subgraph root via a Merkle inclusion
//!   proof. This is what Phase-3 commits to.
//! - **(b) Range-query proof** — verify a range of Node CIDs are
//!   included via a range-Merkle proof. **OUT OF SCOPE for Phase-3**
//!   per ds-r4r2-3; pinned at
//!   `light_client_mode_b_range_query_proof_oos_phase_3_deferred_to_phase_4`
//!   below. Destination: phase-4-backlog.md once that doc opens, or
//!   FULL-ROADMAP.md Phase-4 light-client extensions section.
//! - **(c) Signed checkpoint** — verify a signed checkpoint of the
//!   MST root state at HLC time. **OUT OF SCOPE for Phase-3** per
//!   ds-r4r2-3; pinned at
//!   `light_client_mode_c_signed_checkpoint_oos_phase_3_deferred_to_phase_4`
//!   below. Same destination as mode-(b).
//!
//! Phase-3 commits to mode-(a) only. The architectural-absence pins
//! for modes (b) + (c) clarify intent for fresh agents per the
//! HARD-RULE rule-12 BELONGS-NAMED-NOW disposition (named destination:
//! FULL-ROADMAP.md Phase 4+ light-client extensions).
//!
//! ## What this pins (distinct from the MST diff pin)
//!
//! `mst_light_client_verification_against_content_addressed_root`
//! pins the verification API + Merkle-proof construction. THIS pin
//! asserts that the light-client deliverable as a whole runs
//! WITHOUT full subgraph download — the distinguishing characteristic
//! per ROADMAP-2.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-C wave-6b — ROADMAP-2 distinct light-client deliverable"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-C wave-6b — ROADMAP-2 — mode-(a) Node-CID inclusion proof distinct from full replication"]
fn light_client_verifies_node_cid_inclusion_in_subgraph_root_via_merkle_proof() {
    // ROADMAP-2 pin. G16-C implementer wires this:
    //
    //   use benten_sync::light_client::{LightClient, BandwidthBudget};
    //   let full_peer_total_bytes = 100 * 1024 * 1024;  // 100MB
    //   let light_client_budget = BandwidthBudget::limit_bytes(64 * 1024);  // 64KB
    //
    //   let lc = LightClient::with_budget(light_client_budget);
    //   let published_root = full_peer_published_root_cid();
    //
    //   // Verify a specific subgraph path exists under the published root:
    //   let result = lc.verify_subgraph_path("/zone/posts/p1", &published_root).await.unwrap();
    //   assert!(result.verified);
    //
    //   // Bandwidth used MUST stay within budget — i.e. NO full download:
    //   assert!(lc.bytes_consumed() <= 64 * 1024,
    //       "light-client must verify within budget; consumed {} of 64KB cap",
    //       lc.bytes_consumed());
    //   assert!(lc.bytes_consumed() < full_peer_total_bytes / 100,
    //       "light-client bandwidth must be <1% of full-peer total");
    //
    // OBSERVABLE consequence: the light-client verifies subgraph
    // membership using <<full subgraph bytes; this is the
    // distinguishing characteristic per ROADMAP-2.
    unimplemented!(
        "G16-C wires light-client bandwidth-budget assertion (distinct from full replication)"
    );
}

// =====================================================================
// R4-R2-FP-C architectural-absence pins: light-client modes (b) + (c)
// OOS for Phase-3 per ds-r4r2-3 (closes ds-r4-6 mode-ambiguity).
//
// Pin sources (per .addl/phase-3/r4-r2-distributed-systems.json
// ds-r4r2-3):
//
// - light_client_mode_b_range_query_proof_oos_phase_3_deferred_to_phase_4
// - light_client_mode_c_signed_checkpoint_oos_phase_3_deferred_to_phase_4
//
// Per HARD RULE rule-12 BELONGS-NAMED-NOW disposition: deferral
// destination = FULL-ROADMAP.md Phase 4 light-client extensions
// section (or phase-4-backlog.md once that doc opens — the doc is
// not yet created at HEAD 98280fe; FULL-ROADMAP.md Phase 4 is the
// committed-scope landing surface that exists NOW).
//
// These pins use #[ignore] with explicit OOS rationale (NOT
// unimplemented!()) — they are architectural-absence pins asserting
// that modes (b) + (c) are deliberately NOT in Phase-3 scope, NOT
// RED-PHASE pins waiting for an implementation. They are inert at
// every wave and will REMAIN inert post-Phase-3 close until Phase-4+
// chooses to land them.
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
    //     traversal of a sorted MST sub-range) — additional protocol
    //     surface beyond mode-(a)'s single-CID inclusion proof.
    //   - Bandwidth bounds for range-proofs that scale with the
    //     range size, not the full corpus — distinct measurement
    //     than the mode-(a) bandwidth budget.
    //   - Wire format for range-proof messages (separate from the
    //     mode-(a) inclusion-proof wire format).
    //
    // Architectural decision per ds-r4r2-3: defer to Phase-4+
    // light-client extensions. Phase-3's thin-client protocol
    // (D-PHASE-3-30) supports range queries via fetch-then-verify
    // each Node CID in the range against mode-(a) inclusion proofs;
    // dedicated range-Merkle-proof construction is a Phase-4
    // optimization, not a Phase-3 deliverable.
    //
    // OBSERVABLE-INTENT consequence: the absence of a mode-(b)
    // implementation in the Phase-3 corpus is INTENTIONAL — fresh
    // agents reading this pin learn that range-query proofs are NOT
    // an oversight but a deliberate scope boundary.
    //
    // This test body is INERT (no unimplemented!()) — the pin is the
    // #[ignore] rationale itself, asserting architectural-absence at
    // doc layer. Will remain #[ignore]'d post-Phase-3-close.
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
    //     defenses — additional security model beyond the
    //     content-addressing-only model of mode-(a).
    //   - Checkpoint-publication infrastructure (which peer publishes
    //     checkpoints; how often; how light-clients discover them).
    //
    // Architectural decision per ds-r4r2-3: defer to Phase-4+
    // light-client extensions. Phase-3's thin-client + light-client
    // model relies on direct content-addressed verification against
    // a full peer's published root, not on intermediate signed
    // checkpoints.
    //
    // OBSERVABLE-INTENT consequence: the absence of a mode-(c)
    // implementation in the Phase-3 corpus is INTENTIONAL — fresh
    // agents reading this pin learn that signed-checkpoint
    // verification is NOT an oversight but a deliberate scope
    // boundary.
    //
    // This test body is INERT (no unimplemented!()) — the pin is the
    // #[ignore] rationale itself, asserting architectural-absence at
    // doc layer. Will remain #[ignore]'d post-Phase-3-close.
}
