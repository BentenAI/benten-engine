//! R3-D RED-PHASE pin for `BrowserManifestStore::is_persistent`
//! (G18-A wave-5a; br-r1-8 MINOR).
//!
//! Pin source: r2-test-landscape §2.6 G18-A row
//! `browser_manifest_store_is_persistent_returns_true`; br-r1-8.
//!
//! ## Persistence flag flip shape
//!
//! Phase-2b shipped `BrowserManifestStore::is_persistent` returning
//! `false` (the in-memory implementation accurately said "I'm not
//! persistent"). Phase-3 G18-A wires the IndexedDB backing per
//! D-PHASE-3-27 + br-r1-2; the flag flips to `true` reflecting the
//! durable backing.
//!
//! Source-cite is exact per br-r1-8: edit
//! `bindings/napi/src/wasm_browser.rs:114-227`
//! `BrowserManifestStore::is_persistent` returns `true`.
//!
//! Pairs with `bindings/napi/tests/indexeddb_schema.rs` (the IndexedDB
//! schema-versioning + onupgradeneeded handler that makes the flag
//! true honestly).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G18-A wave-5a flips BrowserManifestStore::is_persistent → true per br-r1-8"]
fn browser_manifest_store_is_persistent_returns_true() {
    // br-r1-8 MINOR pin. G18-A implementer wires this:
    //
    //   // Source-cite assertion against the file:
    //   let src = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("src").join("wasm_browser.rs")
    //   ).unwrap();
    //
    //   // The is_persistent fn returns `true` (not `false`):
    //   //   (implementer pins the exact form — pattern is the function
    //   //    body that immediately precedes `}` should be `true`)
    //
    //   // Heuristic source-cite: look for is_persistent fn + assert
    //   // its return value:
    //   let fn_idx = src.find("fn is_persistent").expect("is_persistent fn present");
    //   let after = &src[fn_idx..fn_idx.saturating_add(400)];
    //   assert!(after.contains("true"),
    //       "BrowserManifestStore::is_persistent must return true at G18-A per br-r1-8");
    //   assert!(!after.contains("-> bool {\n        false")
    //         && !after.contains("false\n    }"),
    //       "BrowserManifestStore::is_persistent must NOT return false at G18-A");
    //
    //   // Also under wasm32-unknown-unknown the runtime call:
    //   //
    //   //   #[cfg(target_arch = "wasm32")]
    //   //   {
    //   //       let store = BrowserManifestStore::new_indexeddb_backed(/* ... */);
    //   //       assert!(store.is_persistent());
    //   //   }
    //
    // OBSERVABLE consequence: the flag honestly reflects durable
    // backing. Defends br-r1-8 directly + pairs with the IndexedDB
    // schema-versioning pins for end-to-end persistence shape.
    unimplemented!("G18-A wires BrowserManifestStore::is_persistent → true source-cite assertion");
}
