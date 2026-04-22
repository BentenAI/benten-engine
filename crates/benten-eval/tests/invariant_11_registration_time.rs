//! R3 unit tests for G5-B-i / E10: Inv-11 registration-time literal-CID
//! rejection.
//!
//! Registration-time probe: a subgraph containing a literal CID whose label
//! matches a `system:*` prefix must be rejected with `E_INV_SYSTEM_ZONE`.
//! Non-system labels register cleanly.
//!
//! TDD red-phase: the registration-time system-zone probe in
//! `benten_eval::invariants::system_zone` does not yet consult
//! `SYSTEM_ZONE_PREFIXES` in the Phase-2a shape. Tests fail until G5-B-i lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.5 E10).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::invariants::system_zone;

#[test]
fn invariant_11_static_system_zone_rejected_at_registration() {
    // Subgraph READs a literal CID whose resolved label is
    // "system:CapabilityGrant" — must reject at registration time.
    let subgraph =
        system_zone::build_subgraph_reading_literal_system_cid_for_test("system:CapabilityGrant");
    let err = system_zone::validate_registration(&subgraph)
        .expect_err("system-zone label must reject at registration");
    assert_eq!(
        err.code(),
        ErrorCode::InvSystemZone,
        "literal system:* CID at registration must fire E_INV_SYSTEM_ZONE"
    );
}

#[test]
fn invariant_11_non_system_label_accepted() {
    // A subgraph reading a Post-labelled literal CID must register cleanly.
    let subgraph = system_zone::build_subgraph_reading_literal_system_cid_for_test("Post");
    system_zone::validate_registration(&subgraph).expect("non-system labels must register cleanly");
}

#[test]
fn inv_11_runtime_enforcement_lives_in_primitive_host() {
    // phil-2 / sec-r1-3 placement split: registration-time lives in
    // benten-eval; runtime lives in benten-engine. This test pins the
    // *registration*-time fn into benten-eval.
    // The mere existence of this import (from benten-eval::invariants::system_zone)
    // is the file-location check.
    fn assert_signature(_: fn(&benten_eval::Subgraph) -> Result<(), benten_eval::EvalError>) {}
    assert_signature(system_zone::validate_registration);
}
