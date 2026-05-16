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
#[ignore = "Hyg-4 #478 trigger-retense: destination phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation STILL OPEN, but the prior 'Phase 3 G16 re-evaluation point' trigger has PASSED (Phase 3 + Phase-4-Foundation both SHIPPED; G16-D wave-6b landed PR #163). Current trigger = the v1-assessment-window in Phase-4-Meta (per phase-3-backlog line ~270: the `ssi`/external-spec-interop + did-method decision composes with the v1-assessment-window items). Un-ignore at that v1-assessment-window re-evaluation. — production prerequisite NOT YET shipped + test body NOT YET authored at HEAD. The `benten_caps::backends::ucan::UCANBackend<B>` durable backend shipped at G14-B PR #109; the graph-Node persistence shape (Node typed-properties via `benten_graph` write path with non-opaque-blob discipline per `cag-r4-2`) requires the `validate_chain_with_revocations` chain-walker seam (which does NOT exist at HEAD — only mentioned in `crates/benten-id/src/ucan.rs:32-36` doc comments). Body remains `unreachable!()` placeholder; un-ignore composes with §2.1-followup re-evaluation outcome since `ssi` integration would re-shape the persisted Node schema (W3C VC v1.1 JSON-LD vs hand-rolled DAG-CBOR envelope)."]
fn benten_id_durable_nodes_are_graph_encoded_no_opaque_blobs() {
    unreachable!("G14-B + G14-C wires this pin at the benten-caps / benten-graph seam");
}

#[test]
#[ignore = "Hyg-4 #478 trigger-retense: destination phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation STILL OPEN, but the prior 'Phase 3 G16 re-evaluation point' trigger has PASSED (Phase 3 + Phase-4-Foundation both SHIPPED; G16-D wave-6b landed PR #163). Current trigger = the v1-assessment-window in Phase-4-Meta (per phase-3-backlog line ~270: the `ssi`/external-spec-interop + did-method decision composes with the v1-assessment-window items). Un-ignore at that v1-assessment-window re-evaluation. — production prerequisite NOT YET shipped + test body NOT YET authored at HEAD. UCAN-grant Node persistence with `:granted_to` Edge requires the durable backend chain-walker to consume revocations + the cross-DID-rotation propagation seam — neither at HEAD (per sibling tests in `did_rotation.rs::did_rotation_propagates_revocation_to_ucan_backend` + `ucan.rs::ucan_chain_revocation_propagates`). Body remains `unreachable!()` placeholder; un-ignore composes with §2.1-followup re-evaluation outcome."]
fn benten_id_ucan_claim_envelope_persisted_as_graph_node_with_granted_to_edge() {
    unreachable!("G14-B wires this pin at benten-caps::UCANBackend");
}

#[test]
#[ignore = "Hyg-4 #478 trigger-retense: destination phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation STILL OPEN, but the prior 'Phase 3 G16 re-evaluation point' trigger has PASSED (Phase 3 + Phase-4-Foundation both SHIPPED; G16-D wave-6b landed PR #163). Current trigger = the v1-assessment-window in Phase-4-Meta (per phase-3-backlog line ~270: the `ssi`/external-spec-interop + did-method decision composes with the v1-assessment-window items). Un-ignore at that v1-assessment-window re-evaluation. — production prerequisite NOT YET shipped + test body NOT YET authored at HEAD. VC-receipt Node persistence shape composes with §2.1-followup re-evaluation outcome (W3C VC v1.1 JSON-LD vs hand-rolled DAG-CBOR envelope shape decides the persisted Node schema). Body remains `unreachable!()` placeholder."]
fn benten_id_vc_issuance_receipt_persisted_as_graph_node() {
    unreachable!("G14-B wires this pin at benten-caps + benten-graph");
}

#[test]
#[ignore = "Hyg-4 #478 trigger-retense: destination phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation STILL OPEN, but the prior 'Phase 3 G16 re-evaluation point' trigger has PASSED (Phase 3 + Phase-4-Foundation both SHIPPED; G16-D wave-6b landed PR #163). Current trigger = the v1-assessment-window in Phase-4-Meta (per phase-3-backlog line ~270: the `ssi`/external-spec-interop + did-method decision composes with the v1-assessment-window items). Un-ignore at that v1-assessment-window re-evaluation. — production prerequisite NOT YET shipped + test body NOT YET authored at HEAD. Device-DID `:attests` Edge persistence (vs string-property anti-pattern) requires the durable graph-Node persistence seam at `benten-caps + benten-graph` that hasn't lit up — the cryptographic on-the-wire envelope shipped at G16-D PR #163 but the durable graph-Edge persistence layer is engine-side and ungated. Body remains `unreachable!()` placeholder; un-ignore composes with §2.1-followup re-evaluation outcome."]
fn device_did_attestation_attests_identity_via_graph_edge_not_string_property() {
    unreachable!("G14-B wires this pin at benten-caps + benten-graph");
}
