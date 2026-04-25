//! Edge-case tests: `Subgraph::load_verified` migration boundary.
//!
//! Covers the core-crate half of the subgraph-load-verification migration
//! (R2 landscape §8.2, partner of `benten-graph/tests/subgraph_load_verified_migration.rs`).
//!
//! Concerns pinned here:
//! - Corrupted DAG-CBOR bytes must produce a typed decode error (`ErrorCode::Serialize`),
//!   not a panic.
//! - Truncated bytes (zero-length, single byte, prefix-only) must refuse
//!   decoding cleanly.
//! - A Subgraph round-tripped through DAG-CBOR preserves the `deterministic`
//!   classification field (R2 §10.4 frozen interface: `subgraph_dagcbor_roundtrip_preserves_deterministic_field`).
//! - The post-migration API surface rejects legacy-shape bytes that pre-date
//!   the verification step, rather than loading them as valid.
//!
//! R3 red-phase contract: R5 (G1-A / G5-A) lands `Subgraph::load_verified`.
//! These tests compile; they fail because the function does not exist yet
//! (compile error treated as red-phase per §9 R3 gates).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Subgraph};
use benten_errors::ErrorCode;

/// Helper: produce bytes that are definitively not valid DAG-CBOR for a Subgraph.
fn garbage_bytes() -> Vec<u8> {
    // Leading byte 0xff — CBOR reserved; no major type decodes to it cleanly.
    vec![0xff, 0x00, 0x01, 0x02]
}

#[test]
fn load_verified_rejects_corrupted_bytes_with_typed_decode_error() {
    // Goal: decode failure is a typed error, never a panic.
    let bytes = garbage_bytes();
    let result = Subgraph::load_verified(&bytes);
    let err = result.expect_err("corrupted bytes must not load");
    assert_eq!(
        err.code(),
        ErrorCode::Serialize,
        "decode failure must map to E_SERIALIZE, got {:?}",
        err.code()
    );
}

#[test]
fn load_verified_rejects_empty_bytes_with_typed_decode_error() {
    // Boundary: zero-length input is the degenerate case.
    let result = Subgraph::load_verified(&[]);
    let err = result.expect_err("empty bytes must not load");
    assert_eq!(err.code(), ErrorCode::Serialize);
}

#[test]
fn load_verified_rejects_single_byte_truncation() {
    // Boundary: one byte of valid CBOR major type prefix that cannot form a
    // complete Subgraph payload.
    let result = Subgraph::load_verified(&[0xa0]); // empty CBOR map
    let err = result.expect_err("truncated (empty-map) bytes must not load as Subgraph");
    // Empty map decodes to *something* but is not a valid Subgraph shape.
    // Either Serialize (decode-layer) or InvRegistration (shape-layer) is
    // acceptable, both are typed errors, neither is a panic.
    let code = err.code();
    assert!(
        matches!(code, ErrorCode::Serialize | ErrorCode::InvRegistration),
        "truncated bytes must produce typed error, got {:?}",
        code
    );
}

#[test]
fn load_verified_rejects_cid_mismatch_with_claimed_cid() {
    // Integrity: load_verified takes the bytes AND the expected CID; if the
    // bytes hash to a different CID the function rejects.
    //
    // R3 shape: `Subgraph::load_verified_with_cid(expected_cid, &bytes)` is
    // the integrity-enforcing variant. A bare `load_verified(&bytes)` that
    // returns Ok does not pin integrity; the migration adds a CID-check path.
    // Argument order mirrors `Node::load_verified(cid, bytes)` so the two
    // verified-load paths read identically at call sites.
    let real_bytes = {
        // Build a minimal-valid Subgraph, encode it.
        let sg = Subgraph::empty_for_test("verify_migration");
        sg.to_dag_cbor().expect("encode must succeed")
    };
    // Claim a different CID than what the bytes hash to.
    let wrong_cid = Cid::from_blake3_digest([0u8; 32]);

    let result = Subgraph::load_verified_with_cid(&wrong_cid, &real_bytes);
    let err = result.expect_err("CID mismatch must fail load_verified");
    assert_eq!(
        err.code(),
        ErrorCode::InvContentHash,
        "CID mismatch must fire E_INV_CONTENT_HASH, got {:?}",
        err.code()
    );
}

#[test]
fn load_verified_roundtrip_preserves_deterministic_field() {
    // Frozen-interface pin (R2 §10.4): the `deterministic` classification field
    // survives encode → load_verified → decode round-trip. DAG-CBOR is
    // canonical-by-default so the round-trip is bit-stable.
    let mut sg = Subgraph::empty_for_test("deterministic_field_pin");
    sg.set_deterministic(true);

    let bytes = sg.to_dag_cbor().expect("encode must succeed");
    let loaded = Subgraph::load_verified(&bytes).expect("re-decode must succeed");

    assert!(
        loaded.is_deterministic(),
        "deterministic=true must survive DAG-CBOR round-trip through load_verified"
    );

    // Negative pin: false also survives.
    let mut sg2 = Subgraph::empty_for_test("deterministic_field_pin_false");
    sg2.set_deterministic(false);
    let bytes2 = sg2.to_dag_cbor().expect("encode must succeed");
    let loaded2 = Subgraph::load_verified(&bytes2).expect("re-decode must succeed");
    assert!(
        !loaded2.is_deterministic(),
        "deterministic=false must survive DAG-CBOR round-trip"
    );
}
