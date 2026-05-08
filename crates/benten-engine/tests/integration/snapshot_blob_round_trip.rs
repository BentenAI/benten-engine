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
//! **Status:** SHIPPED at tag `phase-2b-close` (`3d0f018`). The
//! `Engine::from_snapshot_blob` / `Engine::export_snapshot_blob` APIs
//! landed in `crates/benten-engine/src/engine_snapshot.rs`; all 4
//! tests in this file run live against the shipped surface (0
//! ignored).
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

/// R6-R3 r6-r3-arch-1 — load-bearing end-to-end Rust pin
/// (per `dispatch-conventions.md` §3.6b). Companion to the TS-side
/// pin at `packages/engine/test/snapshot_blob_round_trip.test.ts`
/// "rejects deletes (read-only contract — delete-path symmetry)".
///
/// PR #68 wired `is_read_only_snapshot` enforcement at
/// `PrimitiveHost::put_node` only; `PrimitiveHost::delete_node` had no
/// matching check. A handler dispatched via
/// `engine.call("crud:<label>", ":delete", {cid: <base32>})` against
/// an engine constructed via `Engine::from_snapshot_blob` SILENTLY
/// DELETED the targeted Node, bypassing D10's read-only contract.
///
/// This test drives the production `engine.call` entry point through
/// the CRUD `delete` action (which routes to
/// `PrimitiveHost::delete_node` via the WRITE primitive's `op="delete"`
/// + `target_cid` path) and asserts the call surfaces
/// `EngineError::Other { code: BackendReadOnly, .. }`.
///
/// Would FAIL if the new `check_not_read_only_snapshot` arm in
/// `delete_node` were silently no-op'd back to its pre-fix permissive
/// behavior.
#[test]
fn snapshot_blob_rejects_delete_via_dispatch_handler() {
    use benten_core::{Node, Value};
    use benten_engine::EngineError;
    use std::collections::BTreeMap;

    // Set up the source engine with one persisted "post" Node, so the
    // snapshot blob has something for the dst engine to attempt to
    // delete via dispatch.
    let src_dir = tempfile::tempdir().unwrap();
    let src = Engine::builder()
        .path(src_dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("title".into(), Value::text("deletable"));
    let post_node = Node::new(vec!["post".into()], props);
    let cid = src.create_node(&post_node).unwrap();

    let blob = src.export_snapshot_blob().unwrap();

    // Reconstruct via from_snapshot_blob — D10 read-only contract
    // applies.
    let dst = Engine::from_snapshot_blob(&blob).unwrap();

    // Register the crud handler on the dst engine so `dst.call(...)` can
    // route through to the WRITE primitive's delete arm. Snapshot-blob
    // rehydration restores GRAPH STATE only (nodes + system-zone index);
    // handler registry is per-engine runtime state and does NOT survive
    // the snapshot envelope by design (D10 ships graph bytes, not handler
    // bytecode). Without this registration the dispatch surfaces
    // `NotFound { handler not registered: crud:post }` BEFORE reaching
    // the read-only WRITE check, masking the load-bearing assertion.
    dst.register_crud("post")
        .expect("register crud:post on snapshot-blob dst engine");

    // Dispatch the CRUD delete action against the snapshot-blob engine.
    // The "crud:post" handler id route triggers `subgraph_for_crud` →
    // delete branch → WRITE primitive with op="delete" + target_cid →
    // `PrimitiveHost::delete_node` invocation.
    let mut input_props: BTreeMap<String, Value> = BTreeMap::new();
    input_props.insert("cid".into(), Value::text(cid.to_base32()));
    let input = Node::new(vec![], input_props);

    let outcome = dst.call("crud:post", ":delete", input);

    // Load-bearing assertion: the dispatch must surface
    // E_BACKEND_READ_ONLY (typed via EngineError::Other variant carrying
    // the BackendReadOnly catalog code per host_error_to_engine_error
    // routing).
    match outcome {
        Err(EngineError::Other { ref code, .. })
            if matches!(code, benten_engine::ErrorCode::BackendReadOnly) =>
        {
            // expected
        }
        Err(other) => panic!(
            "expected EngineError::Other {{ code: BackendReadOnly, .. }} \
             for delete-via-dispatch against snapshot-blob engine; got {other:?}"
        ),
        Ok(o) => panic!(
            "expected delete-via-dispatch against snapshot-blob engine to \
             FAIL with E_BACKEND_READ_ONLY; pre-r6-r3-arch-1 this would \
             succeed silently. Got Ok({o:?})"
        ),
    }
}
