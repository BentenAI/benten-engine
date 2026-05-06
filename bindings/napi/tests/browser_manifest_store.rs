//! G18-A wave-5a un-ignored source-cite pin for
//! `BrowserManifestStore::is_persistent` flip → `true` (br-r1-8 MINOR).
//!
//! Pin source: r2-test-landscape §2.6 G18-A row
//! `browser_manifest_store_is_persistent_returns_true`; br-r1-8.
//!
//! ## Persistence flag flip shape (now LIVE)
//!
//! Phase-2b shipped `BrowserManifestStore::is_persistent` returning
//! `false`. Phase-3 G18-A wires the IndexedDB backing per
//! D-PHASE-3-27 + br-r1-2 — the flag flips to `true` reflecting the
//! durable backing.
//!
//! Pairs with `bindings/napi/tests/indexeddb_schema.rs` (the IndexedDB
//! schema-versioning + onupgradeneeded handler that makes the flag
//! true honestly).

#![allow(clippy::unwrap_used)]

#[test]
fn browser_manifest_store_is_persistent_returns_true() {
    // br-r1-8 MINOR pin. Source-cite assertion against the file.
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("wasm_browser.rs"),
    )
    .unwrap();

    // Heuristic source-cite: locate the is_persistent fn body and
    // assert it returns `true`.
    let fn_idx = src
        .find("fn is_persistent")
        .expect("is_persistent fn present in wasm_browser.rs");
    let after = &src[fn_idx..fn_idx.saturating_add(800)];
    assert!(
        after.contains("true"),
        "BrowserManifestStore::is_persistent must return true at G18-A per br-r1-8"
    );
    // The Phase-2b literal `false` arm must NOT linger in the body —
    // it would be dead code if it did.
    assert!(
        !after.contains("        false\n    }"),
        "BrowserManifestStore::is_persistent must NOT return false at G18-A"
    );

    // Runtime check: the constructor's flag matches. (host build
    // exercises this; wasm32 build exercises through the Playwright
    // matrix at `.github/workflows/cross-browser-determinism.yml`.)
    use benten_napi::wasm_browser::BrowserManifestStore;
    let store = BrowserManifestStore::new();
    assert!(
        store.is_persistent(),
        "BrowserManifestStore::is_persistent must return true at G18-A"
    );
}
