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
//! ## Per-shape granularity (cag-r4-2 MAJOR)
//!
//! The composite `benten_id_durable_nodes_are_graph_encoded_no_opaque_blobs`
//! pin enumerates 5 R1-named shapes (keypair anchor / DID rotation
//! attestation / VC envelope / UCAN-claim-envelope / device-DID
//! attestation) but only walks 3 of 5 in implementer pseudo-code.
//! Per cag-r4-2 (R4 large-council Round 1 + Round 2 carry), the VC
//! envelope and UCAN-claim-envelope MUST have INDIVIDUAL per-shape
//! pins — without them, a future implementer can store install_proof()
//! inputs as opaque CBOR-blob KVBackend.put() entries and pass the
//! composite pin while violating ARCHITECTURE.md:181-191 (capability
//! grants are themselves Nodes with GRANTED_TO edges).
//!
//! Per-shape sibling pins (cag-r4-2 closure):
//!
//! - `tests/benten_id_ucan_claim_envelope_persisted_as_graph_node_with_granted_to_edge` — cag-r4-2
//! - `tests/benten_id_vc_issuance_receipt_persisted_as_graph_node` — cag-r4-2
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

#[test]
#[ignore = "RED-PHASE: G14-A2 / G14-B — cag-r4-2 MAJOR — UCAN claim envelope persisted as graph Node with GRANTED_TO Edge"]
fn benten_id_ucan_claim_envelope_persisted_as_graph_node_with_granted_to_edge() {
    // cag-r4-2 MAJOR pin (Charter 3 per-shape granularity). Closes
    // the gap that ucan_backend.rs (G14-B durable backend pin) covers
    // SEMANTICS but NOT durable Node-shape — a future implementer
    // could store install_proof() inputs as opaque CBOR-blob
    // KVBackend.put() entries and pass all 7 ucan_backend.rs pins
    // while violating ARCHITECTURE.md:181-191 + Compromise #11
    // closure floor (capability grants are themselves Nodes with
    // GRANTED_TO edges).
    //
    // G14-A2 / G14-B implementer wires this:
    //
    //   let issuer_kp = benten_id::keypair::Keypair::generate();
    //   let audience_kp = benten_id::keypair::Keypair::generate();
    //
    //   let ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(issuer_kp.public_key().to_did())
    //       .audience(audience_kp.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .not_before(1_000_000_000)
    //       .expiry(1_000_003_600)
    //       .sign(&issuer_kp).unwrap();
    //
    //   // Install into the durable backend (G14-B):
    //   let backend = benten_id::ucan::DurableBackend::test_instance();
    //   backend.install_proof(&ucan).unwrap();
    //
    //   // The grant reads back as a Node with canonical label
    //   // `id:ucan-grant` and structured properties — NOT a single
    //   // `bytes` property containing opaque CBOR:
    //   let grant_node: benten_core::Node = backend.grant_to_node(&ucan.cid()).unwrap();
    //   assert_eq!(grant_node.label(), "id:ucan-grant",
    //       "UCAN grant MUST persist as a Node with label `id:ucan-grant` per cag-r4-2");
    //
    //   let prop_keys: std::collections::BTreeSet<String> =
    //       grant_node.properties().keys().cloned().collect();
    //   for required in &["issuer_did", "audience_did", "capability", "nbf", "exp"] {
    //       assert!(prop_keys.contains(*required),
    //           "UCAN grant Node MUST carry structured property `{}` per cag-r4-2", required);
    //   }
    //   assert!(!prop_keys.contains("blob") && !prop_keys.contains("bytes"),
    //       "UCAN grant Node MUST NOT have an opaque `blob`/`bytes` property per cag-r4-2");
    //
    //   // The GRANTED_TO Edge connects the grant Node to the audience-DID Node:
    //   let edges: Vec<benten_core::Edge> = backend.outgoing_edges(&grant_node.cid()).unwrap();
    //   let granted_to: Vec<&benten_core::Edge> = edges.iter()
    //       .filter(|e| e.label() == "GRANTED_TO").collect();
    //   assert_eq!(granted_to.len(), 1,
    //       "UCAN grant Node MUST emit exactly one GRANTED_TO Edge per cag-r4-2");
    //   assert_eq!(granted_to[0].dst_label(), Some("id:did"),
    //       "GRANTED_TO Edge MUST point to a Node with label `id:did` (the audience-DID Node)");
    //
    // OBSERVABLE consequence: a UCAN grant is queryable through the
    // standard graph + IVM surfaces; cap-policy can subscribe to
    // GRANTED_TO Edges to detect new grants in real time. Defends
    // against the "opaque CBOR escape hatch" anti-pattern + closes
    // the cag-r4-2 Charter-3 per-shape granularity gap that
    // ucan_backend.rs's semantic pins do not cover.
    unimplemented!(
        "G14-A2 / G14-B wires UCAN grant graph-encoding pin: Node label `id:ucan-grant` + \
         structured properties (issuer_did/audience_did/capability/nbf/exp) + GRANTED_TO Edge \
         to audience-DID Node per cag-r4-2 + ARCHITECTURE.md:181-191 + Compromise #11 closure floor"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — cag-r4-2 MAJOR — VC issuance receipt persisted as graph Node"]
fn benten_id_vc_issuance_receipt_persisted_as_graph_node() {
    // cag-r4-2 MAJOR pin (Charter 3 per-shape granularity for VC).
    // Closes the gap that vc.rs covers SEMANTICS (issuance round-trip
    // + expiry-rejection + revocation-via-credential-status + trust-
    // domain-allow-list) but NOT durable Node-shape — a future
    // implementer could store VC envelopes as opaque CBOR blobs.
    //
    // G14-A2 implementer wires this:
    //
    //   let issuer_kp = benten_id::keypair::Keypair::generate();
    //   let subject_did = benten_id::keypair::Keypair::generate().public_key().to_did();
    //
    //   let vc = benten_id::vc::Credential::issue(
    //       &issuer_kp,
    //       subject_did.clone(),
    //       benten_id::vc::Claim::new("urn:claim:friendOf", "did:key:abc..."),
    //       benten_id::vc::IssuanceParams {
    //           issuance_date: 1_000_000_000,
    //           expiration_date: Some(1_000_086_400),
    //           credential_status: None,
    //           trust_domain: "benten.ai".into(),
    //       },
    //   ).unwrap();
    //
    //   // The VC reads back as a Node with canonical label
    //   // `id:vc-receipt` and structured properties:
    //   let vc_node: benten_core::Node = vc.to_node();
    //   assert_eq!(vc_node.label(), "id:vc-receipt",
    //       "VC receipt MUST persist as a Node with label `id:vc-receipt` per cag-r4-2");
    //
    //   let prop_keys: std::collections::BTreeSet<String> =
    //       vc_node.properties().keys().cloned().collect();
    //   for required in &["issuer", "subject", "claim", "issuance_date", "trust_domain"] {
    //       assert!(prop_keys.contains(*required),
    //           "VC receipt Node MUST carry structured property `{}` per cag-r4-2", required);
    //   }
    //   assert!(!prop_keys.contains("blob") && !prop_keys.contains("bytes"),
    //       "VC receipt Node MUST NOT have an opaque `blob`/`bytes` property per cag-r4-2");
    //
    //   // Edge from VC Node to issuer-DID Node:
    //   let edges: Vec<benten_core::Edge> = vc.outgoing_edges();
    //   let issued_by: Vec<&benten_core::Edge> = edges.iter()
    //       .filter(|e| e.label() == "ISSUED_BY").collect();
    //   assert_eq!(issued_by.len(), 1,
    //       "VC receipt Node MUST emit exactly one ISSUED_BY Edge per cag-r4-2");
    //   assert_eq!(issued_by[0].dst_label(), Some("id:did"),
    //       "ISSUED_BY Edge MUST point to a Node with label `id:did` (the issuer-DID Node)");
    //
    // OBSERVABLE consequence: VC receipts are queryable through the
    // standard graph + IVM surfaces; trust-domain queries are graph
    // operations rather than CBOR-blob inspection. Defends against
    // the "opaque CBOR escape hatch" anti-pattern + closes the
    // cag-r4-2 Charter-3 per-shape granularity gap that vc.rs's
    // semantic pins do not cover.
    unimplemented!(
        "G14-A2 wires VC receipt graph-encoding pin: Node label `id:vc-receipt` + \
         structured properties (issuer/subject/claim/issuance_date/trust_domain) + \
         ISSUED_BY Edge to issuer-DID Node per cag-r4-2"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — cag-r4-5 MINOR — device-DID attestation Edge-to-parent-DID structure"]
fn device_did_attestation_attests_identity_via_graph_edge_not_string_property() {
    // cag-r4-5 MINOR pin (Charter 3 + Charter 6 — Edge-shape pin).
    // device_attestation.rs grew +187 LOC of SEMANTIC composition
    // pins in R4-FP (envelope-attenuation + widening-rejection +
    // runs_sandbox_false-cannot-widen + capability-envelope-downgrade-
    // blocked + revoked-device-cannot-sign-new-UCAN). None pin the
    // graph-Edge structure asserting device-DID-Node connects to
    // parent-DID-Node via an Edge labeled `attests_identity_of`.
    // Risk per cag-r4-5: a future implementation could store
    // parent_did as a String property (`parent_did_str =
    // 'did:key:abc...'`) defeating standard graph-traversal queries.
    //
    // G14-A2 implementer wires this:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //       benten_id::device_attestation::CapabilityEnvelope::default(),
    //   ).unwrap();
    //
    //   // The device-DID-Node connects to the parent-DID-Node via a
    //   // graph EDGE — NOT via a String property:
    //   let dev_node: benten_core::Node = attestation.to_node();
    //   assert_eq!(dev_node.label(), "id:device-attestation");
    //
    //   let edges: Vec<benten_core::Edge> = attestation.outgoing_edges();
    //   let attests: Vec<&benten_core::Edge> = edges.iter()
    //       .filter(|e| e.label() == "attests_identity_of").collect();
    //   assert_eq!(attests.len(), 1,
    //       "device-DID attestation MUST emit exactly one `attests_identity_of` \
    //        Edge to parent-DID Node per cag-r4-5");
    //
    //   // Forbidden shape: parent_did stored as a String property
    //   // (defeats graph traversal):
    //   let prop_keys: std::collections::BTreeSet<String> =
    //       dev_node.properties().keys().cloned().collect();
    //   assert!(
    //       !prop_keys.contains("parent_did_str") && !prop_keys.contains("parent_did"),
    //       "device-DID attestation Node MUST NOT store parent_did as a String \
    //        property — the parent linkage MUST be a graph Edge per cag-r4-5"
    //   );
    //
    // OBSERVABLE consequence: standard graph traversal queries (e.g.,
    // "find all devices attesting to identity X") work through the
    // graph layer without bespoke property-key parsing. Defends
    // against architectural drift toward shadow side-channels.
    unimplemented!(
        "G14-A2 wires device-DID attestation Edge-shape pin: `attests_identity_of` Edge \
         from device-DID Node to parent-DID Node + forbidden `parent_did_str`/`parent_did` \
         String property assertion per cag-r4-5"
    );
}
