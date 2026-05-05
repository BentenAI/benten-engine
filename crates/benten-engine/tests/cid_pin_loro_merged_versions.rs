//! R3-C RED-PHASE CID-rebake cohort pins for Loro merged Version
//! Nodes (G16-B wave-6b; per r2-test-landscape §9 + §3.B + plan §3
//! G16-B row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §9 row
//!   `canonical_bytes_loro_merged_version_pinned_cid_*` (×10 sites
//!   per D28 precedent).
//! - r2-test-landscape §3.B Loro CLR-1 cluster.
//! - plan §3 G16-B row line "~10+ pinned-CID test rebake (G12-C-cont
//!   D28 precedent)".
//! - `D-PHASE-3-22` hybrid (iii) Loro→Version-chain.
//! - `D28` (Phase-2b precedent: rebake CID pins in same PR as
//!   encoding changes).
//!
//! ## What this is
//!
//! Per the D28 precedent, every encoding change to a content-addressed
//! data shape ships with a cohort of pinned-CID test sites that
//! lock in the canonical bytes. The Loro merged-Version cohort
//! comprises ~10 sites that pin specific (anchor_id, version_props,
//! contributing_peers) → expected_cid combinations.
//!
//! At R3-C landing time these test sites use placeholder CIDs that
//! the implementer must fill in once the canonical-bytes encoding
//! lands. Each pin asserts:
//!
//! 1. Building the merged Version Node from the same inputs produces
//!    the same canonical bytes byte-for-byte.
//! 2. The CID is stable across runs (deterministic encoding).
//! 3. Cross-process: spawning the build in a child process produces
//!    the same bytes.
//!
//! Per the R2 row pattern `cid_pin_loro_*.rs`, this file owns 10
//! variant pins as separately-named test functions; the implementer
//! at G16-B fills the actual placeholder CIDs.
//!
//! ## RED-PHASE discipline
//!
//! All 10 pins `#[ignore]`'d with rationale
//! `"RED-PHASE: G16-B wave-6b lands canonical-bytes encoding for merged Version Nodes; CID rebake at landing time"`.

#![allow(clippy::unwrap_used)]

const RED_PHASE_PLACEHOLDER_CID: &str = "PLACEHOLDER_CID_FILL_IN_AT_G16_B_LANDING";

/// Helper: builds the canonical fixture Anchor+Versions for cid-pin site N.
/// G16-B implementer wires the real fixture builders.
fn fixture_n(_n: usize) {
    // G16-B implementer wires:
    //   let anchor_id = AnchorId::from_seed(_n as u64);
    //   let v1 = make_version(anchor_id, props_v1_for_seed(_n), peer_a_did);
    //   let v2 = make_version(anchor_id, props_v2_for_seed(_n), peer_b_did);
    //   let merged = merge_versions(&[v1, v2]);
    //   merged.canonical_bytes()
    unimplemented!("G16-B wires canonical fixture builders for cid-pin sites 1..=10");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 1"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_1() {
    // G16-B implementer fills RED_PHASE_PLACEHOLDER_CID with the
    // actual canonical CID for fixture #1, then asserts:
    //
    //   let bytes = fixture_n(1);
    //   let cid = blake3_dag_cbor_cid(&bytes);
    //   assert_eq!(cid.to_string(), "<actual-CID-from-fixture-1>");
    fixture_n(1);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 1");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 2"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_2() {
    fixture_n(2);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 2");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 3"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_3() {
    fixture_n(3);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 3");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 4"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_4() {
    fixture_n(4);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 4");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 5"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_5() {
    fixture_n(5);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 5");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 6"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_6() {
    fixture_n(6);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 6");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 7"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_7() {
    fixture_n(7);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 7");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 8"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_8() {
    fixture_n(8);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 8");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 9"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_9() {
    fixture_n(9);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 9");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CID-rebake cohort site 10"]
fn canonical_bytes_loro_merged_version_pinned_cid_site_10() {
    fixture_n(10);
    let _ = RED_PHASE_PLACEHOLDER_CID;
    unimplemented!("G16-B fills CID for fixture site 10");
}
