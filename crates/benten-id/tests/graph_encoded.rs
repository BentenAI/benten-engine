//! R3-B RED-PHASE pin: `benten-id` durable nodes are graph-encoded
//! (G14-A2 wave-4a'; cag-1).
//!
//! Pin source: r2-test-landscape §2.2 G14-A2 row
//! `benten_id_durable_nodes_are_graph_encoded_no_opaque_blobs`; cag-1.
//!
//! ## Architectural intent
//!
//! Per CLAUDE.md baked-in #3 (code-as-graph), every durable artifact
//! Phase 3 produces (Keypair-anchor Nodes, DID rotation chains, VC
//! envelopes, device-DID attestations, UCAN delegation tokens) MUST
//! be encoded as Nodes in the graph — not as opaque CBOR blobs that
//! happen to be stored alongside the graph.
//!
//! ## RED-PHASE discipline
//!
//! Stays `#[ignore]`'d at R3-B landing. G14-A2 implementer un-ignores
//! AND replaces the stub body with assertions against the live
//! durable-node types per cag-1.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 — cag-1 — durable identity nodes are graph-encoded"]
fn benten_id_durable_nodes_are_graph_encoded_no_opaque_blobs() {
    // cag-1 architectural pin. G14-A2 implementer wires this:
    //
    //   // Each durable identity surface (keypair anchor, DID rotation
    //   // attestation, device attestation, UCAN delegation envelope)
    //   // exposes a `to_node() -> benten_core::Node` method:
    //
    //   let kp = benten_id::keypair::Keypair::generate();
    //   let kp_node: benten_core::Node = kp.public_key().to_anchor_node();
    //   // Node has structured properties (label = "id:keypair-anchor",
    //   // pubkey property, etc.) — NOT a single `bytes: [u8; ...]`
    //   // property containing an opaque CBOR encoding.
    //   assert_eq!(kp_node.label(), "id:keypair-anchor");
    //   assert!(kp_node.properties().keys().any(|k| k == "pubkey"));
    //   // Forbidden shape: a single "blob" property with everything
    //   // marshaled inside.
    //   assert!(!kp_node.properties().keys().any(|k| k == "blob" || k == "bytes"));
    //
    //   // Same for DID rotation:
    //   let new_kp = benten_id::keypair::Keypair::generate();
    //   let attestation = benten_id::did_rotation::rotate_keypair(
    //       &kp.public_key().to_did(), &kp, &new_kp).unwrap();
    //   let att_node: benten_core::Node = attestation.to_node();
    //   assert_eq!(att_node.label(), "id:rotation-attestation");
    //   // ... structured properties expected here too.
    //
    //   // Same for device attestation:
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //   let dev_att = benten_id::device_attestation::DeviceAttestation::issue(
    //       &kp, device_kp.public_key().to_did(),
    //       benten_id::device_attestation::CapabilityEnvelope::default()).unwrap();
    //   let dev_node: benten_core::Node = dev_att.to_node();
    //   assert_eq!(dev_node.label(), "id:device-attestation");
    //
    // OBSERVABLE consequence: durable identity surfaces are queryable,
    // diff-able, version-able through standard graph operations.
    // Defends against the "opaque CBOR escape hatch that bypasses
    // code-as-graph" anti-pattern.
    unimplemented!(
        "G14-A2 wires assertion that identity durable surfaces expose to_node() with structured properties"
    );
}
