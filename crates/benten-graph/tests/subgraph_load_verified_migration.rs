//! Edge-case tests: graph-layer half of the subgraph-load-verification
//! migration (R2 landscape §8.2).
//!
//! Partners `benten-core/tests/subgraph_load_verified_migration.rs` which
//! covers the pure-encode/decode edges. This file covers the persistence
//! round-trip: store → load from bytes → verify CID → return typed error on
//! mismatch or tamper.
//!
//! Concerns pinned:
//! - A Subgraph round-tripped through storage is bit-identical (CID stable).
//! - Tampered bytes at rest fail `load_verified_from_store` with a typed
//!   `E_INV_CONTENT_HASH` error.
//! - A CID-present-but-bytes-decode-fail surfaces `E_SERIALIZE`, not a panic.
//! - A CID-missing path returns `Ok(None)` (clean miss), not an error.
//!
//! R3 red-phase contract: R5 (G2-A / G5-A) introduces
//! `RedbBackend::load_subgraph_verified(cid)`. Tests compile; they fail
//! because the method does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Subgraph};
use benten_errors::ErrorCode;
use benten_graph::RedbBackend;
use tempfile::tempdir;

fn backend() -> (tempfile::TempDir, RedbBackend) {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("sgload.redb")).unwrap();
    (dir, backend)
}

#[test]
fn load_verified_from_store_returns_none_for_missing_cid() {
    let (_dir, backend) = backend();
    let missing = Cid::from_blake3_digest([0u8; 32]);
    let out = backend
        .load_subgraph_verified(&missing)
        .expect("missing CID must be Ok(None), not Err");
    assert!(out.is_none(), "missing CID must return None");
}

#[test]
fn load_verified_from_store_rejects_tampered_bytes_with_content_hash_error() {
    // Store a Subgraph, corrupt the bytes at rest, then load_verified must
    // reject with E_INV_CONTENT_HASH (NOT E_SERIALIZE — the bytes may still
    // decode; the integrity check is CID-vs-content-hash).
    let (_dir, backend) = backend();
    let sg = Subgraph::empty_for_test("tamper_check");
    let cid = backend
        .store_subgraph(&sg)
        .expect("seed store must succeed");

    // Corrupt the stored bytes in place.
    backend
        .corrupt_subgraph_bytes_for_test(&cid, |b| {
            if let Some(byte) = b.last_mut() {
                *byte = byte.wrapping_add(1);
            }
        })
        .expect("test-only corruption hook must succeed");

    let err = backend
        .load_subgraph_verified(&cid)
        .expect_err("tampered bytes must fail load_verified");
    assert_eq!(
        err.code(),
        ErrorCode::InvContentHash,
        "tamper at rest must fire E_INV_CONTENT_HASH, got {:?}",
        err.code()
    );
}

#[test]
fn load_verified_from_store_rejects_undecodeable_bytes_with_serialize_error() {
    // A stored blob that hash-matches its key (because we also replace the
    // key with the corrupted-bytes CID) but is not decodeable as a Subgraph
    // MUST fail with E_SERIALIZE.
    let (_dir, backend) = backend();
    let garbage: &[u8] = &[0xff, 0xfe, 0xfd, 0xfc];
    let cid_of_garbage = backend
        .inject_raw_subgraph_bytes_for_test(garbage)
        .expect("test-only injection hook must succeed");

    let err = backend
        .load_subgraph_verified(&cid_of_garbage)
        .expect_err("undecodable bytes must fail load_verified");
    assert_eq!(
        err.code(),
        ErrorCode::Serialize,
        "undecodable bytes must fire E_SERIALIZE, got {:?}",
        err.code()
    );
}

#[test]
fn load_verified_from_store_round_trip_returns_bit_identical_subgraph() {
    // Happy-path regression: encoding + storing + loading yields the same
    // Subgraph struct (DAG-CBOR canonicality guarantees bit-stability).
    let (_dir, backend) = backend();
    let sg = Subgraph::empty_for_test("roundtrip");
    let cid = backend.store_subgraph(&sg).unwrap();

    let loaded = backend
        .load_subgraph_verified(&cid)
        .expect("load must succeed")
        .expect("present CID must yield Some(sg)");

    assert_eq!(loaded.handler_id(), sg.handler_id());
    // Re-encode must produce the same CID.
    let cid2 = loaded.cid().expect("re-cid must succeed");
    assert_eq!(cid2, cid, "round-trip must preserve CID");
}
