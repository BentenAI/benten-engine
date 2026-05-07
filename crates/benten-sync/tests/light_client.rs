//! G16-C wave-6b LANDED pin: MST light-client verification per
//! r2-test-landscape §2.4 G16-C + plan §3 G16-C row.
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
//! a published root. Foundational for browser thin-clients per
//! CLAUDE.md baked-in #17.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `LightClient` + `MerkleProof` API. The
//! tampered-proof case asserts the verification fails LOUDLY (not
//! silently passes) — would FAIL if the rehash check were bypassed.

#![allow(clippy::unwrap_used)]

use benten_sync::light_client::{LightClient, LightClientError};
use benten_sync::mst::{Mst, MstEntry};

fn build_canonical_mst() -> Mst {
    let mut mst = Mst::new();
    for i in 0..32 {
        let key = format!("/zone/posts/p{i:04}");
        let payload = format!("post-content-{i}").into_bytes();
        mst.insert(MstEntry::from_payload(key, payload));
    }
    mst
}

#[test]
fn mst_light_client_verification_against_content_addressed_root() {
    let full_peer_mst = build_canonical_mst();
    let published_root = full_peer_mst.root_cid();
    let subgraph_path = "/zone/posts/p0008";
    let proof = full_peer_mst.merkle_proof_for(subgraph_path).unwrap();

    // Light-client verifies WITHOUT full download:
    let lc = LightClient::new();
    let result = lc.verify(&published_root, subgraph_path, &proof);
    assert!(
        result.is_ok(),
        "light-client must verify a valid Merkle proof against the published root; got {result:?}"
    );
    let r = result.unwrap();
    assert!(r.verified);
    assert_eq!(r.verified_key, subgraph_path);

    // Tampered proof rejects with a typed error:
    let bad_proof = proof.with_tampered_node();
    let bad_result = lc.verify(&published_root, subgraph_path, &bad_proof);
    assert!(
        matches!(bad_result, Err(LightClientError::Mst(_))),
        "tampered proof must reject with a typed Mst error; got {bad_result:?}"
    );
}
