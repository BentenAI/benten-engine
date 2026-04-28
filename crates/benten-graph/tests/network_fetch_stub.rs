//! Phase-2b G10-A-wasip1 — `NetworkFetchStubBackend` integration test
//! (`network_fetch_stub_returns_typed_error` from the brief must-pass
//! list).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_graph::{KVBackend, NetworkFetchStubBackend};

#[test]
fn network_fetch_stub_returns_typed_error() {
    let backend = NetworkFetchStubBackend::new();

    // Reads / scans surface the Phase-3-deferred-fetch typed error,
    // mapping to E_NOT_IMPLEMENTED.
    let get_err = backend.get(b"n:any").unwrap_err();
    assert_eq!(get_err.code(), ErrorCode::NotImplemented);

    let scan_err = backend.scan(b"").unwrap_err();
    assert_eq!(scan_err.code(), ErrorCode::NotImplemented);

    // Writes surface BackendReadOnly even when Phase-3 lands the
    // network-fetch read implementation; the consume side stays read-only.
    let put_err = backend.put(b"n:k", b"v").unwrap_err();
    assert_eq!(put_err.code(), ErrorCode::BackendReadOnly);

    let del_err = backend.delete(b"n:k").unwrap_err();
    assert_eq!(del_err.code(), ErrorCode::BackendReadOnly);

    let batch_err = backend
        .put_batch(&[(b"n:k".to_vec(), b"v".to_vec())])
        .unwrap_err();
    assert_eq!(batch_err.code(), ErrorCode::BackendReadOnly);
}

#[test]
fn network_fetch_stub_label_surfaces_in_diagnostics() {
    let backend = NetworkFetchStubBackend::with_label("peer-bench-A");
    let err = backend.get(b"n:k").unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("peer-bench-A"),
        "label must surface in error message: {msg}"
    );
}
