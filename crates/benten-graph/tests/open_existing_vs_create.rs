//! Edge-case test: `open_existing` must error on a missing file; `open_or_create`
//! must succeed regardless. Same tag as §2.2 G2 (`P1.graph.open-vs-create`).
//!
//! Covers error code:
//! - `E_BACKEND_NOT_FOUND` — `RedbBackend::open_existing(path)` where `path`
//!   does not exist must fail with this code, not with a generic I/O error.
//!
//! Spike rationale (SPIKE Next Actions #1): a single `open` that silently
//! creates a database file masks typos. The Phase 1 split is "explicit create
//! on first run, explicit reuse on restart." R5 (G2-B) lands the split.
//!
//! R3 contract: `RedbBackend::open_existing` and `RedbBackend::open_or_create`
//! do not exist today; only `RedbBackend::open` exists. These tests compile-fail
//! until G2-B lands — deliberate. The spike's single `open` ambiguity is the
//! exact issue under test.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_graph::{GraphError, RedbBackend};
use tempfile::tempdir;

#[test]
fn open_existing_missing_errors() {
    // Boundary: the file simply does not exist. The "honest no" is
    // E_BACKEND_NOT_FOUND — not "unable to allocate page" or similar
    // redb-internal nonsense bubbling through GraphError::Redb.
    let dir = tempdir().unwrap();
    let nonexistent = dir.path().join("never-created.redb");

    let err = RedbBackend::open_existing(&nonexistent)
        .expect_err("opening a missing file for existing-only use must fail");

    match err {
        GraphError::BackendNotFound { .. } => {}
        other => {
            panic!("expected GraphError::BackendNotFound (E_BACKEND_NOT_FOUND), got {other:?}")
        }
    }
}

#[test]
fn open_or_create_missing_succeeds() {
    // Positive-boundary pair: the create-allowed variant does create the file.
    let dir = tempdir().unwrap();
    let fresh = dir.path().join("fresh.redb");
    assert!(!fresh.exists(), "fixture must start without the file");

    let _backend =
        RedbBackend::open_or_create(&fresh).expect("open_or_create on missing file must succeed");
    assert!(fresh.exists(), "open_or_create must create the file");
}

#[test]
fn open_existing_existing_file_succeeds() {
    // Positive-boundary: after `open_or_create` once, subsequent
    // `open_existing` calls must succeed. Prevents the inverse bug where
    // open_existing refuses to reopen files it legitimately can.
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db.redb");

    {
        let _first = RedbBackend::open_or_create(&db_path).unwrap();
    }

    let _reopened = RedbBackend::open_existing(&db_path)
        .expect("open_existing must succeed once the file exists");
}

#[test]
fn open_or_create_idempotent_on_existing_file() {
    // Boundary: `open_or_create` on an already-present file does not clobber it.
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("idempotent.redb");

    let _first = RedbBackend::open_or_create(&db_path).unwrap();
    // File now exists. Second call must not error and must not truncate.
    let _second = RedbBackend::open_or_create(&db_path)
        .expect("open_or_create must be idempotent on an existing file");
}
