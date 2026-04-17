//! `CapabilityGrant` Node type (P2 — R2 landscape §2.4 row 6).
//!
//! Grants are plain Nodes with label `"CapabilityGrant"`. HLC timestamps in
//! the properties make every grant content-distinct (anti-dedupe semantic).
//!
//! Test edit (G4 mini-review g4-cr-2): `issuer` is now a required field on
//! `CapabilityGrant` — the prior two-construction-paths hazard (struct
//! literal without issuer vs. `::new` with issuer) is a principal-confusion
//! vector. Tests build grants with a real issuer CID.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_caps::CapabilityGrant;
use benten_core::testing::canonical_test_node;

#[test]
fn grant_as_node_label_is_capability_grant() {
    let grantee = canonical_test_node().cid().unwrap();
    let issuer = canonical_test_node().cid().unwrap();
    let grant = CapabilityGrant {
        grantee,
        issuer,
        scope: "post:write".to_string(),
        hlc_stamp: 1,
    };
    let node = grant.as_node();
    assert_eq!(node.labels, vec!["CapabilityGrant".to_string()]);
}

#[test]
fn grant_cid_is_deterministic_for_identical_content() {
    let grantee = canonical_test_node().cid().unwrap();
    let issuer = canonical_test_node().cid().unwrap();
    let g1 = CapabilityGrant {
        grantee: grantee.clone(),
        issuer: issuer.clone(),
        scope: "post:write".to_string(),
        hlc_stamp: 7,
    };
    let g2 = CapabilityGrant {
        grantee,
        issuer,
        scope: "post:write".to_string(),
        hlc_stamp: 7,
    };
    assert_eq!(g1.cid().unwrap(), g2.cid().unwrap());
}
