//! R3-C RED-PHASE pin: MST light-client verification (G16-C wave-6b;
//! per r2-test-landscape §2.4 G16-C + plan §3 G16-C row).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-C row
//!   `mst_light_client_verification_against_content_addressed_root`.
//! - plan §3 G16-C row.
//!
//! ## What this pins
//!
//! Light-client verification API: a client without full subgraph
//! download verifies a subgraph's root CID via Merkle proof against
//! a published root. This is foundational for browser thin-clients
//! that hold partial state but verify completeness against a full
//! peer's published roots.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-C wave-6b lands light-client verification"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-C wave-6b — plan §3 G16-C — light-client verification against content-addressed root"]
fn mst_light_client_verification_against_content_addressed_root() {
    // plan §3 G16-C pin. G16-C implementer wires this:
    //
    //   use benten_sync::light_client::{LightClient, MerkleProof};
    //   let full_peer_mst = build_canonical_mst();
    //   let published_root = full_peer_mst.root_cid();
    //   let subgraph_path = "/zone/posts/p1";
    //   let proof = full_peer_mst.merkle_proof_for(subgraph_path).unwrap();
    //
    //   // Light-client verifies WITHOUT full download:
    //   let lc = LightClient::new();
    //   assert!(
    //       lc.verify(&published_root, subgraph_path, &proof).is_ok(),
    //       "light-client must verify a valid Merkle proof against the published root"
    //   );
    //
    //   // Tampered proof rejects:
    //   let bad_proof = proof.with_tampered_node();
    //   assert!(lc.verify(&published_root, subgraph_path, &bad_proof).is_err());
    //
    // OBSERVABLE consequence: a light-client verifies subgraph
    // membership without downloading the full subgraph; tampered
    // proofs are rejected. Foundational for browser thin-client
    // commitment per CLAUDE.md baked-in #17.
    unimplemented!("G16-C wires light-client Merkle-proof verification");
}
