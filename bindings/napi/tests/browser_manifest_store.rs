//! G18-A wave-5a un-ignored source-cite pin for
//! `BrowserManifestStore::is_persistent` HONEST disclosure (br-r1-8 MINOR).
//!
//! Pin source: r2-test-landscape §2.6 G18-A row
//! `browser_manifest_store_is_persistent_returns_false_until_idb_wired`;
//! br-r1-8.
//!
//! ## Persistence flag honest-disclosure shape (G18-A wave-5a)
//!
//! Phase-2b shipped `BrowserManifestStore::is_persistent` returning
//! `false` (in-memory only). G18-A wave-5a lands the IndexedDB schema
//! + handler SCAFFOLDING but DEFERS the wasm32 `web-sys` / `js-sys` /
//! `wasm-bindgen-futures` plumbing to G18-A-followup wave per
//! `docs/future/phase-3-backlog.md` §4.3. Until that plumbing wires,
//! the in-RAM BTreeMap is the source of truth and `is_persistent`
//! HONESTLY returns `false`. The flag flips to `true` at
//! G18-A-followup wave when the wasm32 IDB plumbing actually persists
//! manifests across page reload.
//!
//! Pairs with `bindings/napi/tests/indexeddb_schema.rs` (the IndexedDB
//! schema-versioning architectural pin — schema landed at G18-A;
//! durable backing wires at G18-A-followup).

#![allow(clippy::unwrap_used)]

#[test]
fn browser_manifest_store_is_persistent_returns_false_until_idb_wired() {
    // br-r1-8 MINOR pin. Source-cite assertion against the file.
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("wasm_browser.rs"),
    )
    .unwrap();

    // Heuristic source-cite: locate the is_persistent fn body and
    // assert it returns `false` honestly until G18-A-followup wires
    // the wasm32 IDB plumbing.
    let fn_idx = src
        .find("fn is_persistent")
        .expect("is_persistent fn present in wasm_browser.rs");
    let after = &src[fn_idx..fn_idx.saturating_add(1200)];
    assert!(
        after.contains("false"),
        "BrowserManifestStore::is_persistent must return false at G18-A \
         until G18-A-followup wires the wasm32 IDB plumbing per br-r1-8 \
         honest-disclosure principle"
    );
    // The body must NOT lie about durability — there must be NO
    // bare `true` literal returned from the fn body. The honest
    // disclosure narrative requires `false` until G18-A-followup.
    assert!(
        !after.contains("    pub const fn is_persistent(&self) -> bool {\n        true\n    }"),
        "BrowserManifestStore::is_persistent must NOT return true at G18-A — \
         the wasm32 IDB plumbing is deferred to G18-A-followup wave per \
         docs/future/phase-3-backlog.md §4.3"
    );

    // Runtime check: the flag matches honestly. (host build exercises
    // this; wasm32 build will exercise through the Playwright matrix
    // at `.github/workflows/cross-browser-determinism.yml` once the
    // G18-A-followup wave authors the fixture bodies.)
    use benten_napi::wasm_browser::BrowserManifestStore;
    let store = BrowserManifestStore::new();
    assert!(
        !store.is_persistent(),
        "BrowserManifestStore::is_persistent must return false at G18-A \
         until G18-A-followup wires the wasm32 IDB plumbing"
    );
}
