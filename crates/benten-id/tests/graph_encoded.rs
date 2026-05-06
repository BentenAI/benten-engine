//! G14-A2 wave-4a' — durable identity graph-encoding pins.
//!
//! All four pins in this file remain `#[ignore]`'d at G14-A2 with
//! RED-PHASE rationale routing them to G14-B / G14-C (where the
//! durable backend lands the graph-side persistence layer). Per the
//! HARD RULE rule-12 disposition (b), the BELONGS-ELSEWHERE
//! destination is THIS file's `#[ignore = "..."]` rationale strings
//! pointing at the named G14-B / G14-C waves; the destinations exist
//! in the implementation plan.
//!
//! ## Why deferred at G14-A2
//!
//! These pins reference `benten_core::Node` / `benten_core::Edge`
//! types that live in the upstream `benten-core` crate. Per
//! `arch-r1-10`, `benten-id` MUST NOT depend on `benten-graph` (the
//! Node/Edge persistence layer); the durable graph-encoding seam
//! lands at `benten-caps` / `benten-graph` (G14-B). G14-A2 lands the
//! IN-MEMORY identity primitives + signature contracts; G14-B lands
//! the durable graph-Node persistence with the per-shape pins from
//! `cag-r4-2` MAJOR + `cag-r4-5` MINOR.
//!
//! Pin sources (per `cag-1` + `cag-r4-2` MAJOR + `cag-r4-5` MINOR):
//!
//! - `benten_id_durable_nodes_are_graph_encoded_no_opaque_blobs`
//! - `benten_id_ucan_claim_envelope_persisted_as_graph_node_with_granted_to_edge`
//! - `benten_id_vc_issuance_receipt_persisted_as_graph_node`
//! - `device_did_attestation_attests_identity_via_graph_edge_not_string_property`

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-B + G14-C — durable graph-Node persistence lands at the benten-caps + benten-graph seam (arch-r1-10 forbids benten-id::benten-graph dep)"]
fn benten_id_durable_nodes_are_graph_encoded_no_opaque_blobs() {
    unreachable!("G14-B + G14-C wires this pin at the benten-caps / benten-graph seam");
}

#[test]
#[ignore = "RED-PHASE: G14-B — UCAN-grant graph-Node persistence lands at benten-caps::UCANBackend per cag-r4-2 + ARCHITECTURE.md:181-191 + Compromise #11 closure floor"]
fn benten_id_ucan_claim_envelope_persisted_as_graph_node_with_granted_to_edge() {
    unreachable!("G14-B wires this pin at benten-caps::UCANBackend");
}

#[test]
#[ignore = "RED-PHASE: G14-B — VC-receipt graph-Node persistence lands at benten-caps + benten-graph per cag-r4-2 (arch-r1-10 forbids benten-id::benten-graph dep)"]
fn benten_id_vc_issuance_receipt_persisted_as_graph_node() {
    unreachable!("G14-B wires this pin at benten-caps + benten-graph");
}

#[test]
#[ignore = "RED-PHASE: G14-B — device-DID attestation graph-Edge persistence lands at benten-caps + benten-graph per cag-r4-5"]
fn device_did_attestation_attests_identity_via_graph_edge_not_string_property() {
    unreachable!("G14-B wires this pin at benten-caps + benten-graph");
}
