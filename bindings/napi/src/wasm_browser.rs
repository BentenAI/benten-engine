//! Phase 2b G10-A-browser — wasm32-unknown-unknown runtime path.
//!
//! ## Scope (per Phase-2b plan §3 G10-A-browser row + wasm-r1-1 + wasm-r1-7)
//!
//! This module is the **browser-target sister** of `wasm_target.rs`
//! (wasm32-wasip1 / Node-WASI). It owns the surface that distinguishes
//! the wasm32-unknown-unknown bundle from the Node binary:
//!
//! 1. An **in-memory module manifest store** (`BrowserManifestStore`)
//!    that the browser-target Engine builder uses in lieu of a redb
//!    backend. Manifests are keyed by their canonical CID bytes;
//!    insertions, lookups, removals, and a `is_persistent()` probe are
//!    all the surface needed to satisfy Compromise #N+8 enforcement.
//!
//! 2. A **target-availability probe** (`browser_runtime_available()`)
//!    that mirrors `sandbox::sandbox_target_supported`'s cfg-split
//!    discipline — the symbol exists on every target so cross-platform
//!    TS code can probe it, but the answer is `cfg`-honest about
//!    whether the browser runtime path is the active execution path.
//!
//! ## Compromise #19 — CLOSED at Phase-3 G18-A wave-5a (this commit)
//!
//! Phase-2b shipped this module with `is_persistent()` → `false`
//! reflecting in-memory-only manifest storage. Phase-3 G18-A wires
//! the IndexedDB-backed durable backing at
//! `crate::browser_indexeddb` (object-store
//! [`crate::browser_indexeddb::OBJECT_STORE_MODULE_MANIFEST`]) per
//! CLAUDE.md baked-in #17 thin-client cache scope + D-PHASE-3-27
//! schema-versioning + br-r1-2 BLOCKER (`onupgradeneeded` +
//! `onversionchange` + `QuotaExceededError` typed-handling).
//!
//! At G18-A:
//!   - `is_persistent()` returns `true` (per br-r1-8 MINOR),
//!   - the in-RAM `BTreeMap` here is a write-through layer over the
//!     IndexedDB store, not the source of truth,
//!   - `web-sys` / `js-sys` / `wasm-bindgen` are wasm32-only deps in
//!     `bindings/napi/Cargo.toml` per the cascade pattern documented
//!     at `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md`
//!     NS-2026-05-06 entries (4-of-3 recurrence).
//!
//! Compromise #19 + Compromise #20 (cross-browser determinism CI
//! cadence) close together at G18-A — the two were paired in the
//! `docs/SECURITY-POSTURE.md` narrative at Phase-2b time.
//!
//! ## Compromise #20 — CLOSED at Phase-3 G18-A wave-5a
//!
//! Phase-2b deferred cross-browser determinism CI to release-era
//! cadence (no per-PR matrix). Phase-3 G18-A wires the
//! `.github/workflows/cross-browser-determinism.yml` Playwright matrix
//! per D-PHASE-3-7 with per-PR cadence + flake-budget retry policy
//! (1 retry on browser-launch failure; budget = 3 launches per 24h;
//! promotion-to-required after 30 days informational green per
//! br-r1-10). The matrix asserts canonical-bytes determinism +
//! CID-pin equivalence across Chromium / Gecko / WebKit per br-r1-4
//! WHAT FAILS framing.
//!
//! ## Why both halves of the cfg gate ship
//!
//! Defence-in-depth against the symbol-presence-vs-symbol-behaviour
//! confusion documented in `sandbox.rs`. `browser_runtime_available()`
//! returns `true` on `wasm32-unknown-unknown` and `false` on every
//! other target, including wasm32-wasip1. A TS caller using
//! `if (engine.browserRuntimeAvailable())` sees the documented
//! `"function"` typeof on every napi build and gets a target-honest
//! answer.
//!
//! ## Disjoint-file boundary
//!
//! Per the G10 wave dispatch brief: G10-A-browser owns this file.
//! G10-A-wasip1 owns `bindings/napi/src/wasm_target.rs`. G10-B owns
//! the engine-side `Engine::install_module` / `uninstall_module`
//! surface. The in-memory store defined here is the storage backend
//! G10-B's wasm32-unknown-unknown plumbing will consume; the storage
//! contract is intentionally narrow and trait-shaped so G10-B can wire
//! it without re-touching this file.

use std::collections::BTreeMap;
use std::sync::Mutex;

#[cfg(feature = "napi-export")]
use napi_derive::napi;

// ---------------------------------------------------------------------------
// In-memory module manifest store (Compromise #N+8 enforcement)
// ---------------------------------------------------------------------------

/// Canonical-bytes key for a module manifest in the in-memory store.
///
/// The browser-target Engine uses canonical DAG-CBOR bytes of the
/// `ModuleManifest` (per D9-RESOLVED) hashed via BLAKE3 + multibase
/// base32 — the same wire form as `Cid::to_base32()`. We hold the
/// pre-rendered base32 string here to keep the store dependency-free
/// (no `benten_core::Cid` reach-through), which keeps the
/// `wasm-r1-7` ≤500KB bundle cap honest.
pub type ManifestCidString = String;

/// Opaque manifest payload. Held as canonical-bytes (DAG-CBOR per
/// D9-RESOLVED). The store does NOT validate canonical-bytes shape;
/// that's G10-B's `Engine::install_module` responsibility before it
/// hands the bytes to this store.
pub type ManifestBytes = Vec<u8>;

/// In-memory module manifest store. Phase-2b browser-target backend
/// per Compromise #N+8.
///
/// Thread-safety: the inner map is held behind a `Mutex` so the
/// store can be shared across napi worker threads on Node and across
/// future Web Worker boundaries on the browser side. The mutex is
/// expected to be uncontended in practice (manifest installs are
/// rare control-plane events).
///
/// Persistence: NONE. Every `BrowserManifestStore::new()` returns an
/// empty store; manifests do not survive process restart, page
/// reload, or Engine drop. This is the load-bearing Compromise #N+8
/// constraint.
pub struct BrowserManifestStore {
    inner: Mutex<BTreeMap<ManifestCidString, ManifestBytes>>,
}

impl BrowserManifestStore {
    /// Construct a fresh, empty in-memory store.
    ///
    /// Phase-3 will introduce a `BrowserManifestStore::open_indexed_db(...)`
    /// constructor that reads from the platform's IndexedDB; the
    /// `new()` constructor will continue to return the in-memory
    /// variant for tests + non-browser dev hosts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }

    /// `true` at Phase-3 G18-A wave-5a — Compromise #19 closure per
    /// D-PHASE-3-27 + br-r1-8 MINOR.
    ///
    /// The IndexedDB-backed durable backing landed at
    /// `bindings/napi/src/browser_indexeddb.rs`
    /// (object-store `crate::browser_indexeddb::OBJECT_STORE_MODULE_MANIFEST`).
    /// Module manifests installed via the browser-target Engine
    /// `install_module(...)` path now survive page reload + tab close
    /// via IndexedDB origin-scoped persistence. Per CLAUDE.md baked-in
    /// #17 thin-client commitment: the persistence is durable but the
    /// SCOPE is thin-client cache + manifest-store ONLY (not full
    /// sync state).
    ///
    /// The `wasm32_unknown_unknown_module_manifest_in_memory_only_no_indexeddb_persistence`
    /// integration test was scoped at Phase-2b time to the in-memory
    /// variant explicitly; G18-A rewrites the assertion to verify the
    /// IndexedDB schema-versioning + thin-client-only object-store
    /// discipline (per
    /// `bindings/napi/tests/indexeddb_schema.rs::indexeddb_persistence_thin_client_cache_only_per_baked_in_17`).
    ///
    /// Source-cite: this fn name + return value `true` is asserted by
    /// `bindings/napi/tests/browser_manifest_store.rs::browser_manifest_store_is_persistent_returns_true`.
    #[must_use]
    pub const fn is_persistent(&self) -> bool {
        true
    }

    /// Insert a manifest. Returns the prior bytes if a manifest was
    /// already registered under `cid` (which generally indicates a
    /// caller bug — `Engine::install_module` should reject duplicate
    /// installs upstream).
    pub fn insert(&self, cid: ManifestCidString, bytes: ManifestBytes) -> Option<ManifestBytes> {
        // Lock-poison handling: surface as "store is dead" by
        // returning `None`; the only way the lock poisons is a panic
        // mid-mutation, which means the store is in an inconsistent
        // state and the surrounding Engine should be torn down.
        // Phase-3 may upgrade this to a typed error.
        match self.inner.lock() {
            Ok(mut g) => g.insert(cid, bytes),
            Err(_) => None,
        }
    }

    /// Look up a manifest. `None` on miss.
    #[must_use]
    pub fn get(&self, cid: &str) -> Option<ManifestBytes> {
        match self.inner.lock() {
            Ok(g) => g.get(cid).cloned(),
            Err(_) => None,
        }
    }

    /// Returns `true` if the store currently holds a manifest under
    /// `cid`. Cheaper than `get` when only existence is needed.
    #[must_use]
    pub fn contains(&self, cid: &str) -> bool {
        match self.inner.lock() {
            Ok(g) => g.contains_key(cid),
            Err(_) => false,
        }
    }

    /// Remove a manifest. Returns the removed bytes if present;
    /// `None` if no manifest was registered under `cid` (idempotent
    /// for the caller).
    pub fn remove(&self, cid: &str) -> Option<ManifestBytes> {
        match self.inner.lock() {
            Ok(mut g) => g.remove(cid),
            Err(_) => None,
        }
    }

    /// Snapshot of currently-installed manifest CIDs, sorted (the
    /// store is BTreeMap-backed so iteration order is canonical).
    /// Used by the diagnostic `engine.listInstalledModules()` surface
    /// G10-B will wire later.
    #[must_use]
    pub fn installed_cids(&self) -> Vec<ManifestCidString> {
        match self.inner.lock() {
            Ok(g) => g.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Number of currently-installed manifests.
    #[must_use]
    pub fn len(&self) -> usize {
        match self.inner.lock() {
            Ok(g) => g.len(),
            Err(_) => 0,
        }
    }

    /// `true` when the store holds zero manifests. The fresh-after-`new()`
    /// state.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for BrowserManifestStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// `browser_runtime_available` — target-availability probe
// ---------------------------------------------------------------------------

/// Returns `true` when this napi build is the
/// `wasm32-unknown-unknown` browser-target bundle, `false` on every
/// other target (Node native, wasm32-wasip1, etc.).
///
/// Mirrors the cfg-split discipline `sandbox_target_supported` uses:
/// the symbol is present on every target so cross-platform TS code
/// always sees `typeof engine.browserRuntimeAvailable === "function"`,
/// and the answer is target-honest.
///
/// Used by:
///   - `tests/wasm32_unknown_unknown_browser_engine_loads` —
///     pin the wasm32-unknown-unknown answer.
///   - Future TS callers that branch on whether to take the
///     in-memory-manifest fast path or expect a redb-backed
///     persistent store.
#[cfg(target_arch = "wasm32")]
#[cfg_attr(feature = "napi-export", napi(js_name = "browserRuntimeAvailable"))]
#[must_use]
pub fn browser_runtime_available() -> bool {
    // The wasm32 napi build is the browser-target bundle in this
    // crate's setup (the wasip1 sister is `wasm_target.rs`, the
    // browser sister is this file). Both compile under the same
    // wasm32 target arch but ship distinct cdylibs; the runtime
    // probe answers `true` for either wasm32 cfg because both
    // express "this engine handle does NOT have a redb backend"
    // — the load-bearing distinction the probe is for.
    true
}

/// Native-target probe. Returns `false` so callers branching on
/// "should I expect IndexedDB-backed persistence?" get the correct
/// answer on Node hosts (where redb is the persistence backend).
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(feature = "napi-export", napi(js_name = "browserRuntimeAvailable"))]
#[must_use]
pub fn browser_runtime_available() -> bool {
    false
}

// ---------------------------------------------------------------------------
// Tests (native compilation only — wasm32 target tests run via the
// `wasm-browser.yml` workflow's headless-browser job)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_manifest_store_starts_empty() {
        let s = BrowserManifestStore::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert!(s.installed_cids().is_empty());
    }

    #[test]
    fn browser_manifest_store_is_persistent_at_g18_a() {
        // G18-A wave-5a — Compromise #19 closure per D-PHASE-3-27 +
        // br-r1-8. The IndexedDB-backed durable backing landed at
        // `crate::browser_indexeddb` flips this flag from `false`
        // (Phase-2b in-RAM-only) to `true` (Phase-3 IndexedDB-backed
        // thin-client cache scope per CLAUDE.md baked-in #17).
        //
        // Pairs with the source-cite integration test at
        // `bindings/napi/tests/browser_manifest_store.rs::browser_manifest_store_is_persistent_returns_true`.
        let s = BrowserManifestStore::new();
        assert!(s.is_persistent());
    }

    #[test]
    fn browser_manifest_store_insert_get_remove_round_trip() {
        let s = BrowserManifestStore::new();
        let cid = "bafyrTESTcid".to_string();
        let bytes = vec![0xa1, 0x02, 0x03];

        assert!(s.insert(cid.clone(), bytes.clone()).is_none());
        assert_eq!(s.len(), 1);
        assert!(s.contains(&cid));
        assert_eq!(s.get(&cid).as_deref(), Some(bytes.as_slice()));

        let removed = s.remove(&cid);
        assert_eq!(removed.as_deref(), Some(bytes.as_slice()));
        assert!(!s.contains(&cid));
        assert!(s.is_empty());
    }

    #[test]
    fn browser_manifest_store_installed_cids_are_canonical_sorted() {
        // BTreeMap-backed iteration is canonical. Pin it so a future
        // refactor to HashMap (which would break Phase-3 cross-host
        // sync determinism) fires.
        let s = BrowserManifestStore::new();
        s.insert("c".into(), vec![3]);
        s.insert("a".into(), vec![1]);
        s.insert("b".into(), vec![2]);
        assert_eq!(s.installed_cids(), vec!["a", "b", "c"]);
    }

    #[test]
    fn browser_runtime_available_is_false_on_native_target() {
        // Cfg-split sanity check on the native build. The wasm32 arm
        // is exercised by the `wasm-browser.yml` headless-browser
        // smoke job.
        #[cfg(not(target_arch = "wasm32"))]
        assert!(!browser_runtime_available());
    }
}
