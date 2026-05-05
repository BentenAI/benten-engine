//! R3-A RED-PHASE pins for `SnapshotBlobBackend` direct-wire as
//! `GraphBackend` (G13-D wave-3; plan §3 G13-D).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-D + plan §3 G13-D
//! must-pass column):
//!
//! - `tests/snapshot_blob_backend_impls_graph_backend_read_path` — plan §3 G13-D
//! - `tests/snapshot_blob_backend_write_path_returns_read_only_error` — plan §3 G13-D
//!
//! ## What G13-D does
//!
//! Phase-2b shipped a read-only `SnapshotBlobBackend` impl. G13-D wires
//! it directly into the `GraphBackend` umbrella trait so
//! `EngineGeneric<SnapshotBlobBackend>` works end-to-end (replacing
//! the Phase-2b tempdir-hydration path).
//!
//! - `Engine::from_snapshot_blob(bytes)` returns
//!   `EngineGeneric<SnapshotBlobBackend>` directly.
//! - Read-path methods delegate to the existing read-only impl.
//! - Write-path methods return `BackendError::ReadOnly` with typed
//!   error; the snapshot-blob is content-addressed and writes would
//!   break the canonical-bytes invariant.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-D wave-3 — SnapshotBlobBackend impls GraphBackend"]
fn snapshot_blob_backend_impls_graph_backend_read_path() {
    // G13-D implementer wires this:
    //   // Build a snapshot blob from a populated engine:
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine_a = benten_engine::Engine::open(dir.path()).unwrap();
    //   // ... populate engine_a ...
    //   let blob_bytes = engine_a.snapshot_to_bytes().unwrap();
    //
    //   // Open a snapshot-backed engine via SnapshotBlobBackend:
    //   let snapshot_backend = benten_graph::SnapshotBlobBackend::from_bytes(&blob_bytes).unwrap();
    //   fn assert_impl<B: benten_graph::GraphBackend>(_: &B) {}
    //   assert_impl(&snapshot_backend);
    //
    //   // Read-path: scan(b"n:") returns the populated nodes:
    //   let nodes = snapshot_backend.scan(b"n:").unwrap().collect::<Vec<_>>();
    //   assert!(!nodes.is_empty());
    //
    // OBSERVABLE consequence: SnapshotBlobBackend is a first-class
    // GraphBackend; the engine can run read-only over it without
    // tempdir hydration.
    unimplemented!("G13-D wires SnapshotBlobBackend GraphBackend impl + read-path assertion");
}

#[test]
#[ignore = "RED-PHASE: G13-D wave-3 — SnapshotBlobBackend write-path read-only error"]
fn snapshot_blob_backend_write_path_returns_read_only_error() {
    // G13-D implementer wires this:
    //   let blob_bytes = ... ;
    //   let backend = benten_graph::SnapshotBlobBackend::from_bytes(&blob_bytes).unwrap();
    //   let result = backend.put(b"n:newkey", b"newdata");
    //   let err = result.unwrap_err();
    //   // Typed error carrying the read-only marker:
    //   assert_eq!(err.code(), benten_errors::ErrorCode::BackendReadOnly);
    //
    // OBSERVABLE consequence: writes against a content-addressed
    // snapshot blob fail LOUDLY with E_BACKEND_READ_ONLY rather than
    // silently dropping or corrupting the canonical-bytes invariant.
    unimplemented!("G13-D wires SnapshotBlobBackend write-path read-only error assertion");
}
