//! `CapabilityGrant` Node type (P2 — R2 landscape §2.4 row 6).
//!
//! Grants are plain Nodes with label `"system:CapabilityGrant"`. HLC
//! timestamps in the properties make every grant content-distinct
//! (anti-dedupe semantic).
//!
//! Test edit (G4 mini-review g4-cr-2): `issuer` is now a required field on
//! `CapabilityGrant` — the prior two-construction-paths hazard (struct
//! literal without issuer vs. `::new` with issuer) is a principal-confusion
//! vector. Tests build grants with a real issuer CID.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_caps::{CAPABILITY_GRANT_LABEL, CapabilityGrant};
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
        // Phase 2a ucca-8: default None preserves Phase-1 CID.
        ttl_hlc_duration: None,
    };
    let node = grant.as_node();
    // Match against the shared constant so the label-namespace contract
    // is pinned at one location (r6b-ivm-2). The namespaced
    // `"system:CapabilityGrant"` form is load-bearing for View 1's filter
    // and the `BackendGrantReader` lookup.
    assert_eq!(node.labels, vec![CAPABILITY_GRANT_LABEL.to_string()]);
    assert_eq!(CAPABILITY_GRANT_LABEL, "system:CapabilityGrant");
}

#[test]
fn grant_cid_is_deterministic_for_identical_content() {
    let grantee = canonical_test_node().cid().unwrap();
    let issuer = canonical_test_node().cid().unwrap();
    let g1 = CapabilityGrant {
        grantee,
        issuer,
        scope: "post:write".to_string(),
        hlc_stamp: 7,
        ttl_hlc_duration: None,
    };
    let g2 = CapabilityGrant {
        grantee,
        issuer,
        scope: "post:write".to_string(),
        hlc_stamp: 7,
        ttl_hlc_duration: None,
    };
    assert_eq!(g1.cid().unwrap(), g2.cid().unwrap());
}
