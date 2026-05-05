//! R3-C RED-PHASE pin: light-client verifies subgraph root WITHOUT
//! full replication (G16-C wave-6b; ROADMAP-2 distinct deliverable).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-C row
//!   `light_client_verifies_subgraph_root_without_full_replication`.
//! - plan §3 G16-C row line "light-client verification API + Merkle
//!   proof construction + verification against published roots;
//!   works WITHOUT full subgraph download as distinct deliverable
//!   from MST diff".
//! - `ROADMAP-2` (light-client deliverable distinct from full-peer
//!   sync — separate exit criterion).
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
#[ignore = "RED-PHASE: G16-C wave-6b — ROADMAP-2 — light-client distinct from full replication"]
fn light_client_verifies_subgraph_root_without_full_replication() {
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
