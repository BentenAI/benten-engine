//! Phase 2b R3 (R3-E) — D10-RESOLVED snapshot-blob round-trip integration.
//!
//! TDD red-phase. Pin source: plan §3 G10-A-wasip1 + §5 D10-RESOLVED
//! (`Engine::from_snapshot_blob(bytes: &[u8])` paired with
//! `Engine::export_snapshot_blob() -> Vec<u8>`; snapshot-blob is DAG-CBOR
//! `{ schema_version, anchor_cid, nodes: BTreeMap<Cid, NodeBytes>,
//! system_zone_index: BTreeMap<String, Vec<Cid>> }` rolled through
//! `Node::cid` so the blob has its own CID; BTreeMap-sorted-by-key for
//! canonical bytes — non-negotiable per Inv-13 collision-safety per
//! sec-pre-r1-09).
//!
//! These tests assert:
//! 1. `export_snapshot_blob → from_snapshot_blob → export_snapshot_blob`
//!    yields BYTE-IDENTICAL bytes (canonical-CID round-trip stability).
//! 2. The exported blob's bytes are stable when the underlying engine
//!    state is fixed (BTreeMap canonical-bytes discipline pin).
//! 3. The blob computes a stable CID under `Node::cid` so Phase-3 sync
//!    can treat blob handoff as a content-addressed transfer.
//!
//! **Status:** RED-PHASE (Phase 2b G10-A-wasip1 pending). The
//! `from_snapshot_blob` / `export_snapshot_blob` APIs do not yet exist
//! on `Engine`; G10-A-wasip1 implementation lands them.
//!
//! Owned by R3-E (R3 territory: WASM target + cross-crate integration +
//! CI workflow tests + SuspensionStore + WAIT TTL).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;

/// Build a fresh engine with one canonical node so every call produces
/// the SAME observable graph state (the canonical fixture CID).
fn engine_with_canonical_state() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let _cid = engine.create_node(&canonical_test_node()).unwrap();
    (dir, engine)
}

/// `snapshot_blob_round_trips_export_import` — R2 §2.3 (D10-RESOLVED).
///
/// The cycle export → import → export MUST yield byte-identical output.
/// Any drift means BTreeMap canonical-bytes discipline broke OR a hidden
/// non-deterministic field crept into the snapshot envelope.
#[test]
#[ignore = "Phase 2b G10-A-wasip1 pending — Engine::{export,from}_snapshot_blob unimplemented"]
fn snapshot_blob_round_trips_export_import() {
    let (_dir, engine) = engine_with_canonical_state();

    // First export from the natively-built engine.
    let blob_a = engine.export_snapshot_blob().unwrap();

    // Reconstruct via the snapshot-blob constructor.
    let reopened = Engine::from_snapshot_blob(&blob_a).unwrap();

    // Re-export from the snapshot-imported engine.
    let blob_b = reopened.export_snapshot_blob().unwrap();

    assert_eq!(
        blob_a, blob_b,
        "snapshot-blob export → import → re-export must be BYTE-IDENTICAL \
         (D10 canonical-bytes discipline; Phase-3 sync depends on this)"
    );
}

/// `snapshot_blob_btreemap_canonical_bytes_stable` — R2 §2.3 (D10 +
/// sec-pre-r1-09 Inv-13 collision-safety).
///
/// Two engines built from identical inputs MUST produce byte-identical
/// snapshot blobs. Any drift indicates a hidden non-deterministic field
/// (HashMap insertion order, timestamp leak, etc.).
#[test]
#[ignore = "Phase 2b G10-A-wasip1 pending"]
fn snapshot_blob_btreemap_canonical_bytes_stable() {
    let (_dir1, engine1) = engine_with_canonical_state();
    let (_dir2, engine2) = engine_with_canonical_state();

    let blob1 = engine1.export_snapshot_blob().unwrap();
    let blob2 = engine2.export_snapshot_blob().unwrap();

    assert_eq!(
        blob1, blob2,
        "two engines built from identical canonical state must produce \
         byte-identical snapshot blobs (BTreeMap-sorted-by-key discipline)"
    );
}

/// `snapshot_blob_round_trips_under_canonical_cid` — R2 §2.3 (wasm-r1-2).
///
/// The exported snapshot blob has its OWN CID (rolled through `Node::cid`
/// per D10). That CID MUST be stable across export → import → re-export.
#[test]
#[ignore = "Phase 2b G10-A-wasip1 pending"]
fn snapshot_blob_round_trips_under_canonical_cid() {
    let (_dir, engine) = engine_with_canonical_state();

    let blob_a = engine.export_snapshot_blob().unwrap();
    let cid_a = Engine::compute_snapshot_blob_cid(&blob_a).unwrap();

    let reopened = Engine::from_snapshot_blob(&blob_a).unwrap();
    let blob_b = reopened.export_snapshot_blob().unwrap();
    let cid_b = Engine::compute_snapshot_blob_cid(&blob_b).unwrap();

    assert_eq!(
        cid_a, cid_b,
        "snapshot-blob CID must be stable across export → import → re-export \
         (Phase-3 sync uses this CID as the content-addressed handoff key)"
    );
}
