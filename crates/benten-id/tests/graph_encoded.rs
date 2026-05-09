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
#[ignore = "phase-3-backlog §7.3.D — durable graph-Node persistence at benten-caps + benten-graph seam. G14-B + G14-C wave-4b shipped (PRs #109 + #110); test body pins specific graph-Node persistence shape contract; un-ignore at §2.1-followup ssi external UCAN/VC spec compatibility re-evaluation outcome per Wave-E rationale-only sweep."]
fn benten_id_durable_nodes_are_graph_encoded_no_opaque_blobs() {
    unreachable!("G14-B + G14-C wires this pin at the benten-caps / benten-graph seam");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — UCAN-grant graph-Node persistence at benten-caps::UCANBackend. G14-B PR #109 + G15-A PR #113 (Compromise #11 closure) both shipped; test body pins specific UCAN-grant persistence contract; un-ignore at §2.1-followup re-evaluation outcome per Wave-E rationale-only sweep."]
fn benten_id_ucan_claim_envelope_persisted_as_graph_node_with_granted_to_edge() {
    unreachable!("G14-B wires this pin at benten-caps::UCANBackend");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — VC-receipt graph-Node persistence at benten-caps + benten-graph. G14-B + G14-A2 shipped; test body pins specific VC-receipt persistence contract; un-ignore at §2.1-followup re-evaluation outcome per Wave-E rationale-only sweep."]
fn benten_id_vc_issuance_receipt_persisted_as_graph_node() {
    unreachable!("G14-B wires this pin at benten-caps + benten-graph");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — device-DID attestation graph-Edge persistence. G14-B + G16-D wave-6b shipped (PRs #109 + #163; on-the-wire device-DID-attestation envelope); test body pins specific graph-Edge persistence contract; un-ignore at §2.1-followup re-evaluation outcome per Wave-E rationale-only sweep."]
fn device_did_attestation_attests_identity_via_graph_edge_not_string_property() {
    unreachable!("G14-B wires this pin at benten-caps + benten-graph");
}
