//! R3-D RED-PHASE pin for IndexedDB-backed BlobBackend (G18-A wave 5a;
//! plan §3 G18-A).
//!
//! Pin source: r2-test-landscape §2.6 G18-A row
//! `browser_blob_backend_round_trip`.
//!
//! ## IndexedDB BlobBackend shape
//!
//! G18-A authors `bindings/napi/src/browser_blob_store.rs` (NEW) — an
//! IndexedDB-backed implementation of the `BlobBackend` trait surface
//! (locked at G13-pre-B per `crates/benten-graph/src/backends/blob_backend_trait.rs`).
//!
//! Per CLAUDE.md baked-in #17 thin-client commitment: the
//! IndexedDB-backed BlobBackend is THIN-CLIENT SNAPSHOT CACHE SCOPE
//! ONLY. It is NOT a full sync state store; full sync remains
//! native-only per G14-D + G16-* boundaries.
//!
//! ## Why round-trip pin
//!
//! The basic put/get round-trip ensures the IndexedDB store correctly
//! serializes to + deserializes from IndexedDB without dropping bytes.
//! Pairs with `indexeddb_schema.rs` for the schema-versioning +
//! quota-handling shape, and with the cross-browser determinism CI
//! cell for canonical-bytes equivalence across Chromium/Gecko/WebKit.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G18-A wave 5a authors browser_blob_store.rs (IndexedDB BlobBackend variant)"]
fn browser_blob_backend_round_trip() {
    // plan §3 G18-A pin. G18-A implementer wires this:
    //
    //   // Pure source-cite assertion (Rust integration test, host
    //   // build target, asserts the file exists + names BlobBackend):
    //   let src = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("src").join("browser_blob_store.rs")
    //   ).unwrap();
    //
    //   assert!(src.contains("BlobBackend"),
    //       "browser_blob_store.rs must implement BlobBackend trait per G14-C surface lock");
    //   assert!(src.contains("indexeddb") || src.contains("IndexedDB") || src.contains("idb"),
    //       "browser_blob_store.rs must use IndexedDB backing per G18-A + br-r1-2");
    //   assert!(src.contains("thin-client") || src.contains("thin client") || src.contains("baked-in #17"),
    //       "browser_blob_store.rs must document thin-client cache scope per CLAUDE.md baked-in #17");
    //
    //   // Round-trip wasm32 runtime test (under wasm-bindgen-test or
    //   // Playwright cell) — implementer wires this OR routes through
    //   // the cross-browser-determinism Playwright matrix cell:
    //   //
    //   //   #[wasm_bindgen_test::wasm_bindgen_test]
    //   //   async fn round_trip_in_browser() {
    //   //       let backend = browser_blob_store::IndexedDbBlobBackend::open("test").await.unwrap();
    //   //       backend.put(b"key", b"data").await.unwrap();
    //   //       let got = backend.get(b"key").await.unwrap();
    //   //       assert_eq!(got, Some(vec![100, 97, 116, 97]));
    //   //   }
    //
    // OBSERVABLE consequence: the IndexedDB BlobBackend honors the
    // BlobBackend trait surface AND survives the put/get round-trip
    // through actual IndexedDB. Defends plan §3 G18-A surface.
    unimplemented!(
        "G18-A wires browser_blob_store.rs source-cite + (optional wasm-bindgen-test) round-trip"
    );
}
