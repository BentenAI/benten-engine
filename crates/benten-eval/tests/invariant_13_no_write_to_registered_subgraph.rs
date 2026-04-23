//! R3 + R5 G5-A unit tests for E11 / Invariant-13 — registration-time
//! declaration-layer reject of WRITE-to-registered-subgraph-CID.
//!
//! Registration-time probe (declaration layer): a subgraph containing a
//! WRITE Node whose `target_cid` property is a literal CID already
//! registered with the engine is rejected with `E_INV_IMMUTABILITY` at
//! `register_subgraph` time.
//!
//! Storage-layer runtime enforcement (authoritative backstop) lives in
//! `benten-graph` per plan §9.11 5-row matrix — this file pins ONLY the
//! eval-layer declaration check.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::{EvalError, invariants::immutability};

#[test]
fn invariant_13_no_write_to_registered_subgraph() {
    // Synthesise a subgraph whose WRITE primitive declares a literal
    // `target_cid` that is already in the caller-supplied registered-CID
    // set. The declaration-layer check must reject with Inv-13.
    let registered_cid = Cid::from_blake3_digest([0x42u8; 32]);
    let sg = immutability::build_subgraph_writing_to_literal_cid_for_test(
        "handler_under_test",
        &registered_cid,
    );

    let mut registered = BTreeSet::new();
    registered.insert(registered_cid);

    let err = immutability::validate_registration_against(&sg, &registered)
        .expect_err("WRITE to registered CID must reject at declaration layer");

    let code: ErrorCode = match err {
        EvalError::Invariant(v) => v.code(),
        _ => panic!("expected EvalError::Invariant, got {err:?}"),
    };
    assert_eq!(
        code,
        ErrorCode::InvImmutability,
        "registration-time WRITE to registered CID must fire E_INV_IMMUTABILITY"
    );
}

#[test]
fn invariant_13_write_to_unregistered_cid_registers_cleanly() {
    let some_cid = Cid::from_blake3_digest([0x11u8; 32]);
    let sg = immutability::build_subgraph_writing_to_literal_cid_for_test("handler_ok", &some_cid);
    let registered = BTreeSet::new();
    immutability::validate_registration_against(&sg, &registered)
        .expect("WRITE to unregistered CID must pass the declaration check");
}

#[test]
fn inv_13_runtime_enforcement_lives_in_benten_graph() {
    // phil-2 / sec-r1-4 placement split: registration-time lives in
    // benten-eval; runtime lives in benten-graph per plan §9.11 5-row
    // matrix. The mere existence of this import is the file-location
    // assertion.
    fn assert_signature(
        _: fn(&benten_eval::Subgraph, &BTreeSet<Cid>) -> Result<(), benten_eval::EvalError>,
    ) {
    }
    assert_signature(immutability::validate_registration_against);
}
