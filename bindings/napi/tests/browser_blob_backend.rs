//! G18-A wave-5a un-ignored source-cite pin for IndexedDB-backed
//! BlobBackend (plan §3 G18-A).
//!
//! Pin source: r2-test-landscape §2.6 G18-A row
//! `browser_blob_backend_round_trip`.
//!
//! ## IndexedDB BlobBackend shape (now LIVE)
//!
//! G18-A authors `bindings/napi/src/browser_blob_store.rs` — an
//! IndexedDB-backed implementation mirroring the
//! [`benten_graph::backends::blob_backend_trait::BlobBackend`] surface
//! locked at G13-pre-B.
//!
//! Per CLAUDE.md baked-in #17 thin-client commitment: the
//! IndexedDB-backed BlobBackend is THIN-CLIENT SNAPSHOT CACHE SCOPE
//! ONLY. NOT a full sync state store.
//!
//! ## Round-trip pin
//!
//! The basic put/get round-trip (host build, native unit-test arm of
//! the module's `IndexedDbBlobBackend`) ensures the backend's
//! defense-in-depth CID validation + bytes round-trip behaves correctly
//! on every target. The Playwright matrix at
//! `.github/workflows/cross-browser-determinism.yml` runs the wasm32
//! arm against real IndexedDB across Chromium / Gecko / WebKit per
//! D-PHASE-3-7 + br-r1-4.

#![allow(clippy::unwrap_used)]

#[test]
fn browser_blob_backend_round_trip() {
    // plan §3 G18-A pin. Source-cite assertion against the file.
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("browser_blob_store.rs"),
    )
    .unwrap();

    assert!(
        src.contains("BlobBackend"),
        "browser_blob_store.rs must reference BlobBackend trait per G14-C surface lock"
    );
    assert!(
        src.contains("indexeddb")
            || src.contains("IndexedDB")
            || src.contains("idb")
            || src.contains("Indexed"),
        "browser_blob_store.rs must use IndexedDB backing per G18-A + br-r1-2"
    );
    assert!(
        src.contains("thin-client") || src.contains("thin client") || src.contains("baked-in #17"),
        "browser_blob_store.rs must document thin-client cache scope per CLAUDE.md baked-in #17"
    );

    // Runtime round-trip — exercise the inherent put/get path on
    // host build (the wasm32 arm is exercised through the Playwright
    // matrix cell).
    use benten_core::Cid;
    use benten_napi::browser_blob_store::IndexedDbBlobBackend;
    let backend = IndexedDbBlobBackend::new();
    let bytes = b"thin-client-cache-round-trip".to_vec();
    let cid = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());
    backend.put_sync(&cid, &bytes).unwrap();
    let got = backend.get_sync(&cid).unwrap();
    assert_eq!(got, Some(bytes), "round-trip must yield identical bytes");
    // Defense-in-depth CID validation per D-PHASE-3-12.
    let wrong_bytes = b"different-bytes".to_vec();
    let err = backend
        .put_sync(&cid, &wrong_bytes)
        .expect_err("CID-mismatch put must be rejected");
    let _ = err; // diagnostic — concrete enum is internal to browser_blob_store.
}
