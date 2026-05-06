//! Phase-3 G18-A wave-5a — Compromise #19 + #20 PARTIAL closure pin
//! (was the Phase-2b "in-memory only" guard; rescoped at G18-A).
//!
//! ## Why this file changed shape at G18-A
//!
//! At Phase-2b this file pinned the OPPOSITE invariant: that
//! `BrowserManifestStore::is_persistent()` returns `false` AND that the
//! napi crate had NO `web-sys` / `idb` dep. That posture closed
//! Compromise #N+8 (browser-persistent-storage absent) by REFUSING
//! persistence at the dep-graph level.
//!
//! At Phase-3 G18-A wave-5a, the IndexedDB schema-version + handler
//! SCAFFOLDING landed at `bindings/napi/src/browser_indexeddb.rs`
//! BUT the wasm32 `web-sys` / `js-sys` / `wasm-bindgen-futures`
//! plumbing is deferred to G18-A-followup wave (per
//! `docs/future/phase-3-backlog.md` §4.3). Per the honest-disclosure
//! principle Compromise #19 originally articulated, the
//! `is_persistent` flag stays `false` at G18-A — flipping it to
//! `true` ahead of the wasm32 plumbing would lie about durability to
//! operators branching on the flag.
//!
//! The dep-graph guard is retired; the architectural-discipline guard
//! is a NEW source-cite assertion at
//! `bindings/napi/tests/indexeddb_schema.rs::indexeddb_persistence_thin_client_cache_only_per_baked_in_17`
//! which asserts the IndexedDB schema declares ONLY thin-client object
//! stores (no Loro / iroh / sync-cursor surfaces) — that pin lands at
//! G18-A regardless of whether the wasm32 plumbing has wired yet.
//!
//! Per HARD RULE rule-12 (no-defer): renaming this file to mirror the
//! new shape would be the cleaner spelling, but that would also retire
//! the historical narrative the test header carries. Keeping the file
//! at the same path with a refactored body documents the
//! Phase-2b → Phase-3 transition inline.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_napi::wasm_browser::BrowserManifestStore;

/// G18-A honest disclosure: `BrowserManifestStore::is_persistent()`
/// returns `false` at Phase-3 G18-A wave-5a until G18-A-followup wires
/// the wasm32 IDB plumbing per br-r1-8 MINOR honest-disclosure
/// principle. The schema scaffolding landed; the wasm32 IDB calls did
/// not.
#[test]
fn store_reports_not_yet_persistent_at_g18_a() {
    let s = BrowserManifestStore::new();
    assert!(
        !s.is_persistent(),
        "G18-A wave-5a: BrowserManifestStore::is_persistent stays \
         false until G18-A-followup wires the wasm32 web-sys / \
         js-sys / wasm-bindgen-futures plumbing per CLAUDE.md \
         baked-in #17 thin-client cache scope + honest-disclosure \
         principle"
    );
}

/// G18-A architectural-discipline pin: the IndexedDB schema declares
/// ONLY thin-client object stores per CLAUDE.md baked-in #17. This
/// pin is the host-build companion of the source-cite assertion at
/// `bindings/napi/tests/indexeddb_schema.rs::indexeddb_persistence_thin_client_cache_only_per_baked_in_17`.
#[test]
fn indexeddb_schema_declares_thin_client_object_stores_only() {
    use benten_napi::browser_indexeddb::{OBJECT_STORE_BLOB_CACHE, OBJECT_STORE_MODULE_MANIFEST};
    // The two declared object stores are thin-client surfaces.
    assert!(
        OBJECT_STORE_MODULE_MANIFEST.contains("manifest"),
        "thin-client manifest-store object-store name must contain 'manifest'"
    );
    assert!(
        OBJECT_STORE_BLOB_CACHE.contains("blob") || OBJECT_STORE_BLOB_CACHE.contains("cache"),
        "thin-client blob-cache object-store name must contain 'blob' or 'cache'"
    );
    // The names MUST NOT collide with full-sync state surfaces. The
    // architectural pin in `bindings/napi/tests/indexeddb_schema.rs`
    // sweeps the source file for the prohibited markers (loro_doc,
    // iroh_peers, sync_cursor, atrium_full_state). This pin is the
    // identifier-level companion: a refactor that renamed the
    // thin-client constants to full-sync names would fail here.
    for forbidden in &["loro", "iroh", "sync_cursor", "atrium_full"] {
        assert!(
            !OBJECT_STORE_MODULE_MANIFEST.contains(forbidden),
            "OBJECT_STORE_MODULE_MANIFEST must not name full-sync state ({forbidden})"
        );
        assert!(
            !OBJECT_STORE_BLOB_CACHE.contains(forbidden),
            "OBJECT_STORE_BLOB_CACHE must not name full-sync state ({forbidden})"
        );
    }
}
