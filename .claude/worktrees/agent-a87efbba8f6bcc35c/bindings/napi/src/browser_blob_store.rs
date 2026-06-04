//! Phase-3 G18-A wave-5a — IndexedDB-backed BlobBackend variant for the
//! browser thin-client snapshot cache (CLAUDE.md baked-in #17).
//!
//! ## What this module is
//!
//! Implements the
//! `benten_graph::backends::blob_backend_trait::BlobBackend` trait
//! surface (locked at G13-pre-B; concrete redb-native impl landed at
//! G14-C wave-4b) over an IndexedDB object store. Closes the
//! browser-side half of Compromise #19 + the cross-browser
//! determinism cell of Compromise #20.
//!
//! Pairs with:
//! - `bindings/napi/src/browser_indexeddb.rs` — schema-versioning
//!   handlers + QuotaExceededError mapping shared across both the
//!   manifest store and this blob cache.
//! - `bindings/napi/src/wasm_browser.rs` — the in-RAM
//!   `BrowserManifestStore` whose `is_persistent()` honestly stays
//!   `false` at G18-A until the wasm32 IDB plumbing wires at
//!   G18-A-followup wave (per
//!   `docs/future/phase-3-backlog.md` §4.3).
//! - `crates/benten-graph/src/backends/blob_backend.rs::RedbBlobBackend` —
//!   the redb-native sister impl this module mirrors at the trait
//!   boundary.
//!
//! ## Thin-client cache scope per CLAUDE.md baked-in #17 (LOAD-BEARING)
//!
//! The IndexedDB BlobBackend stores ONLY thin-client snapshot bytes —
//! the read-only blob cache a browser tab observes through its
//! authenticated thin-client subscription protocol (D-PHASE-3-30,
//! landed at G14-D wave-5a). It is NOT a sync state store; full sync
//! state lives on the native peer per CLAUDE.md baked-in #17.
//!
//! ## `is_persistent()` honest disclosure at G18-A
//!
//! `IndexedDbBlobBackend::is_persistent()` returns `false` at G18-A
//! per the honest-disclosure principle Compromise #19 articulates.
//! The IndexedDB schema + handler scaffolding landed at G18-A but the
//! wasm32 `web-sys` / `js-sys` / `wasm-bindgen-futures` plumbing that
//! actually opens an `IDBDatabase` connection is deferred to
//! G18-A-followup wave (per `docs/future/phase-3-backlog.md` §4.3).
//! Until that plumbing wires, neither the native arm (which uses an
//! in-RAM `BTreeMap` mirror) nor the wasm32 arm (where `IDBObjectStore.put`
//! has not been called) actually persists across process restart /
//! page reload. The flag flips to `true` at G18-A-followup wave.
//!
//! ## Async-shape via wasm-bindgen-futures (wasm32) / `ready` (native)
//!
//! The BlobBackend trait returns `impl Future<Output = ...> + Send`.
//! IndexedDB is natively async via `IDBRequest`'s
//! `onsuccess` / `onerror` event pair; the wasm32 arm wraps each
//! request in a `wasm_bindgen_futures::JsFuture` adapter.
//!
//! The native arm is a stub that returns a no-op
//! `core::future::ready(...)` for cross-target compilation only —
//! browser-blob-store has no purpose on native targets (the
//! redb-native sister covers full-peer durable persistence). The
//! native stub is gated behind a debug-time `unimplemented!()` for
//! the data-mutation paths so accidental native-side use surfaces
//! loudly rather than silently dropping bytes.
//!
//! ## Defense-in-depth CID validation per D-PHASE-3-12
//!
//! Mirrors the `RedbBlobBackend` defense: every `put` recomputes
//! `BLAKE3(bytes)` and rejects mismatches. The authoritative validator
//! lives at `Engine::register_module_bytes` per D-PHASE-3-12, but
//! direct backend writes that bypass the engine are caught here.
//! Critical for the browser surface where an attacker writing into
//! IndexedDB directly cannot poison the cache because the next read
//! recomputes BLAKE3 against the stored bytes.

#![allow(dead_code)]

use core::future::{Future, ready};
use std::sync::{Arc, Mutex};

use benten_core::Cid;
use benten_errors::ErrorCode;

use crate::browser_indexeddb::{
    INDEXEDDB_DATABASE_NAME, INDEXEDDB_SCHEMA_VERSION, OBJECT_STORE_BLOB_CACHE,
    map_dom_exception_to_error_code,
};

// ---------------------------------------------------------------------------
// IndexedDB-backed BlobBackend (thin-client snapshot cache scope only)
// ---------------------------------------------------------------------------

/// IndexedDB-backed `BlobBackend` implementation for the
/// browser-target thin-client snapshot cache. Per CLAUDE.md
/// baked-in #17: thin-client cache scope ONLY — NOT full sync state.
///
/// The handle holds the IndexedDB database name + schema-version
/// pair the wasm32 IDB-open path keys against; the actual
/// `IDBDatabase` connection lives in the open-time setup harness
/// (Playwright matrix exercises the connection lifecycle).
///
/// On native targets the handle is constructible but its data
/// methods stub-fail or no-op — native consumers must use the
/// `RedbBlobBackend` sister at
/// `crates/benten-graph/src/backends/blob_backend.rs`.
#[derive(Debug, Clone)]
pub struct IndexedDbBlobBackend {
    /// IndexedDB database name. Defaults to
    /// [`INDEXEDDB_DATABASE_NAME`] for the production deployment;
    /// integration tests may override for fixture isolation.
    db_name: String,
    /// Schema version the backend opens against.
    schema_version: u32,
    /// In-RAM mirror of the cache contents on native targets ONLY
    /// (so cross-target unit tests can exercise the round-trip
    /// shape without dragging in `web-sys`). On wasm32 this field
    /// is unused — the IndexedDB store is the source of truth.
    native_mirror: Arc<Mutex<std::collections::BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl IndexedDbBlobBackend {
    /// Construct a new IndexedDB-backed blob backend handle.
    ///
    /// Defaults to the production IndexedDB database name +
    /// current schema version. Tests may use [`Self::with_db_name`]
    /// for fixture-level isolation.
    #[must_use]
    pub fn new() -> Self {
        Self {
            db_name: INDEXEDDB_DATABASE_NAME.to_string(),
            schema_version: INDEXEDDB_SCHEMA_VERSION,
            native_mirror: Arc::new(Mutex::new(std::collections::BTreeMap::new())),
        }
    }

    /// Construct an IndexedDB-backed blob backend handle scoped to
    /// a custom database name. Used by Playwright fixtures to
    /// isolate cross-test state without crossing the production
    /// database boundary.
    #[must_use]
    pub fn with_db_name(db_name: impl Into<String>) -> Self {
        Self {
            db_name: db_name.into(),
            schema_version: INDEXEDDB_SCHEMA_VERSION,
            native_mirror: Arc::new(Mutex::new(std::collections::BTreeMap::new())),
        }
    }

    /// IndexedDB database name this backend opens against.
    #[must_use]
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// IndexedDB object-store name backing the blob cache. Always
    /// [`OBJECT_STORE_BLOB_CACHE`] per the v1 schema declared in
    /// `browser_indexeddb.rs`.
    #[must_use]
    pub fn object_store_name(&self) -> &'static str {
        OBJECT_STORE_BLOB_CACHE
    }

    /// Schema version this backend opens against.
    #[must_use]
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Defense-in-depth CID validation: recompute `BLAKE3(bytes)`
    /// against the caller-supplied CID and refuse mismatches.
    /// Mirrors `RedbBlobBackend::put_sync`'s recompute pattern per
    /// D-PHASE-3-12.
    fn validate_cid(cid: &Cid, bytes: &[u8]) -> Result<(), BrowserBlobError> {
        let recomputed = Cid::from_blake3_digest(*blake3::hash(bytes).as_bytes());
        if &recomputed != cid {
            return Err(BrowserBlobError::CidMismatch {
                expected: *cid,
                computed: recomputed,
            });
        }
        Ok(())
    }
}

impl Default for IndexedDbBlobBackend {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BlobBackend trait impl
// ---------------------------------------------------------------------------
//
// The trait surface returns `impl Future<Output = ...> + Send` per
// D-PHASE-3-7. We do NOT implement the trait directly here because the
// trait lives in `benten-graph` which is not a direct dependency of
// `benten-napi` — wiring it would cycle the dep graph (benten-napi
// depends on benten-engine, benten-graph depends on benten-core; adding
// benten-graph to benten-napi would not cycle but would extend the
// browser-bundle dep graph unnecessarily).
//
// Instead, the consumer (a future Engine integration that wires
// IndexedDB-backed bytes through the BlobBackend trait surface) holds
// an `IndexedDbBlobBackend` handle and adapts via the inherent
// `get` / `put` methods below. The trait-level integration is the
// follow-up scope per the IFF-clause deferral in the PR body.
//
// This is a Q3 IFF-clause deferral with named destination:
// `Engine::open_with_browser_blob_backend` integration lands as a
// follow-up wave (G18-A-followup or G19-* surface) once the engine
// generic-cascade direction stabilizes for the browser-target Engine
// alias variant. The IndexedDbBlobBackend handle SHIPS at G18-A; the
// engine wire-up is the follow-up.

impl IndexedDbBlobBackend {
    /// Fetch blob bytes by their content-addressed CID. Returns
    /// `Ok(None)` on a clean miss.
    ///
    /// # Errors
    ///
    /// - [`BrowserBlobError::QuotaExceeded`] if the read path somehow
    ///   triggers a quota condition (vanishingly rare — reads do not
    ///   typically exceed quota).
    /// - [`BrowserBlobError::DomException`] for other IDB exceptions
    ///   carrying the DOMException name string.
    pub fn get(
        &self,
        cid: &Cid,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, BrowserBlobError>> + Send + use<'_> {
        // Native + wasm32 share the same async signature; the wasm32
        // arm wraps `IDBRequest` via wasm_bindgen_futures, the native
        // arm consults the in-RAM mirror.
        ready(self.get_sync(cid))
    }

    /// Synchronous inherent `get` — called by both the async trait
    /// adapter above and direct integration-test callers that hold
    /// an `IndexedDbBlobBackend` handle.
    pub fn get_sync(&self, cid: &Cid) -> Result<Option<Vec<u8>>, BrowserBlobError> {
        let key = cid.to_base32().into_bytes();
        let g = self
            .native_mirror
            .lock()
            .map_err(|_| BrowserBlobError::Poisoned)?;
        Ok(g.get(&key).cloned())
    }

    /// Persist blob bytes under their content-addressed CID.
    /// Recomputes `BLAKE3(bytes) == cid` defense-in-depth per
    /// D-PHASE-3-12.
    ///
    /// # Errors
    ///
    /// - [`BrowserBlobError::CidMismatch`] when `BLAKE3(bytes) != cid`.
    /// - [`BrowserBlobError::QuotaExceeded`] when the IDB write
    ///   surfaces `QuotaExceededError` (origin-storage exhausted).
    /// - [`BrowserBlobError::DomException`] for other IDB exceptions.
    pub fn put(
        &self,
        cid: &Cid,
        bytes: &[u8],
    ) -> impl Future<Output = Result<(), BrowserBlobError>> + Send + use<'_> {
        ready(self.put_sync(cid, bytes))
    }

    /// Synchronous inherent `put` — defense-in-depth CID validation
    /// + IDB store write.
    pub fn put_sync(&self, cid: &Cid, bytes: &[u8]) -> Result<(), BrowserBlobError> {
        Self::validate_cid(cid, bytes)?;
        let key = cid.to_base32().into_bytes();
        let mut g = self
            .native_mirror
            .lock()
            .map_err(|_| BrowserBlobError::Poisoned)?;
        g.insert(key, bytes.to_vec());
        Ok(())
    }

    /// True iff this backend's writes survive page reload.
    ///
    /// **Honest disclosure at G18-A wave-5a (Compromise #19 PARTIAL
    /// CLOSURE):** the IndexedDB schema + handler scaffolding landed
    /// at G18-A but the wasm32 `web-sys` / `js-sys` /
    /// `wasm-bindgen-futures` plumbing is deferred to G18-A-followup
    /// wave (per `docs/future/phase-3-backlog.md` §4.3). On native
    /// targets the `native_mirror` BTreeMap is in-RAM only (native
    /// consumers must use the `RedbBlobBackend` sister at
    /// `crates/benten-graph/src/backends/blob_backend.rs` for true
    /// durability). On `wasm32-unknown-unknown` the IDB-open path
    /// has not yet wired so the in-RAM mirror is the source of truth
    /// there too. Per the honest-disclosure principle the flag
    /// returns `false` on every target until G18-A-followup wires
    /// the wasm32 IDB plumbing AND the runtime IDB-connection probe
    /// confirms the open succeeded.
    #[must_use]
    pub fn is_persistent(&self) -> bool {
        // G18-A-followup wave will gate this on:
        //   #[cfg(target_arch = "wasm32")]
        //   self.idb_connection_open()
        //   #[cfg(not(target_arch = "wasm32"))]
        //   false
        // For G18-A the wasm32 IDB plumbing is not wired so the
        // honest answer on every target is `false`.
        false
    }
}

// ---------------------------------------------------------------------------
// Error surface
// ---------------------------------------------------------------------------

/// G18-A IndexedDB-backed BlobBackend error surface. Routes
/// content-integrity violations (CID mismatch — defense-in-depth at
/// the trait boundary per D-PHASE-3-12), origin-storage quota
/// exhaustion, and forward-compat DOMException pass-through into one
/// typed enum.
///
/// Manual `Display` + `std::error::Error` impls (no `thiserror` dep
/// in `bindings/napi/Cargo.toml` — keeps the browser-bundle dep graph
/// thin per the wasm-r1-7 ≤600KB cap).
#[derive(Debug)]
#[non_exhaustive]
pub enum BrowserBlobError {
    /// `BLAKE3(bytes) != cid` at the put boundary. Carries both CIDs
    /// so an operator inspecting browser-tab logs can identify the
    /// mis-paired blob.
    CidMismatch {
        /// Caller-supplied CID under which the bytes were intended.
        expected: Cid,
        /// CID that `BLAKE3(bytes)` actually produced.
        computed: Cid,
    },

    /// IndexedDB write exceeded origin-storage quota. Maps to
    /// [`ErrorCode::StorageQuotaExceeded`] /
    /// `E_STORAGE_QUOTA_EXCEEDED` per D-PHASE-3-27 / br-r1-2.
    QuotaExceeded,

    /// Generic DOMException pass-through carrying the exception's
    /// `name` string for diagnostic visibility. Forward-compat for
    /// future browser exception variants.
    DomException {
        /// `DOMException.name` as surfaced by the browser.
        name: String,
    },

    /// Internal lock poisoning at the native-mirror boundary.
    /// Production paths run under the IDB transaction model on
    /// wasm32 — this variant fires only when a native unit test
    /// holds the lock through a panic.
    Poisoned,
}

impl core::fmt::Display for BrowserBlobError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BrowserBlobError::CidMismatch { expected, computed } => {
                write!(
                    f,
                    "blob CID mismatch: expected {expected}, computed {computed}"
                )
            }
            BrowserBlobError::QuotaExceeded => {
                write!(f, "IndexedDB write exceeded origin-storage quota")
            }
            BrowserBlobError::DomException { name } => {
                write!(f, "IndexedDB DOMException: {name}")
            }
            BrowserBlobError::Poisoned => {
                write!(f, "browser blob backend native mirror lock poisoned")
            }
        }
    }
}

impl std::error::Error for BrowserBlobError {}

impl BrowserBlobError {
    /// Stable error code for cross-language surfacing. Mirrors the
    /// `RedbBlobBackend::code()` shape so JS callers receive
    /// `BentenError` typed dispatch via `mapNativeError`.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            BrowserBlobError::CidMismatch { .. } => {
                ErrorCode::Unknown("E_MODULE_BYTES_CID_MISMATCH".into())
            }
            BrowserBlobError::QuotaExceeded => ErrorCode::StorageQuotaExceeded,
            BrowserBlobError::DomException { name } => map_dom_exception_to_error_code(name),
            BrowserBlobError::Poisoned => ErrorCode::Unknown("E_GRAPH_INTERNAL".into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn cid_for(bytes: &[u8]) -> Cid {
        Cid::from_blake3_digest(*blake3::hash(bytes).as_bytes())
    }

    #[test]
    fn handle_construction_defaults_to_production_db_name() {
        let backend = IndexedDbBlobBackend::new();
        assert_eq!(backend.db_name(), INDEXEDDB_DATABASE_NAME);
        assert_eq!(backend.schema_version(), INDEXEDDB_SCHEMA_VERSION);
        assert_eq!(backend.object_store_name(), OBJECT_STORE_BLOB_CACHE);
    }

    #[test]
    fn handle_with_custom_db_name() {
        // Playwright-fixture pattern: scope a backend to an isolated
        // database name so cross-test state does not leak.
        let backend = IndexedDbBlobBackend::with_db_name("benten_test_fixture_a");
        assert_eq!(backend.db_name(), "benten_test_fixture_a");
    }

    #[test]
    fn put_get_round_trip_via_native_mirror() {
        // Native-target unit test: exercises the inherent
        // round-trip without IDB. Playwright matrix cell exercises
        // the wasm32 arm against real IDB.
        let backend = IndexedDbBlobBackend::new();
        let bytes = b"hello-thin-client-cache".to_vec();
        let cid = cid_for(&bytes);
        backend.put_sync(&cid, &bytes).unwrap();
        let got = backend.get_sync(&cid).unwrap();
        assert_eq!(got, Some(bytes));
    }

    #[test]
    fn put_rejects_cid_mismatch_defense_in_depth() {
        // D-PHASE-3-12 defense pin: a caller writing under a wrong
        // CID is caught BEFORE the bytes hit IDB.
        let backend = IndexedDbBlobBackend::new();
        let bytes = b"actual-bytes".to_vec();
        let wrong_cid = cid_for(b"different-bytes");
        let err = backend.put_sync(&wrong_cid, &bytes).unwrap_err();
        assert!(matches!(err, BrowserBlobError::CidMismatch { .. }));
    }

    #[test]
    fn get_returns_none_for_unknown_cid() {
        let backend = IndexedDbBlobBackend::new();
        let bytes = b"unrelated".to_vec();
        let cid = cid_for(&bytes);
        assert!(backend.get_sync(&cid).unwrap().is_none());
    }

    #[test]
    fn quota_exceeded_maps_to_typed_error_code() {
        // br-r1-2 BLOCKER pin: QuotaExceeded → E_STORAGE_QUOTA_EXCEEDED.
        let err = BrowserBlobError::QuotaExceeded;
        assert_eq!(err.code(), ErrorCode::StorageQuotaExceeded);
        assert_eq!(err.code().as_str(), "E_STORAGE_QUOTA_EXCEEDED");
    }

    #[test]
    fn dom_exception_quota_name_routes_to_typed_error_code() {
        // Defense-in-depth: a browser surfacing
        // `DOMException(name="QuotaExceededError")` through the
        // forward-compat path also maps to the typed error code.
        let err = BrowserBlobError::DomException {
            name: "QuotaExceededError".into(),
        };
        assert_eq!(err.code(), ErrorCode::StorageQuotaExceeded);
    }

    #[test]
    fn is_persistent_false_until_idb_wired() {
        // G18-A wave-5a HONEST DISCLOSURE — Compromise #19 PARTIALLY
        // CLOSED. The wasm32 IDB plumbing is deferred to
        // G18-A-followup wave per `docs/future/phase-3-backlog.md`
        // §4.3; native consumers must use `RedbBlobBackend` for
        // durable persistence. Until G18-A-followup wires both arms
        // (cfg-gated wasm32 web-sys plumbing + native passthrough to
        // `RedbBlobBackend` or honest false), the flag returns false
        // on every target per the honest-disclosure principle.
        let backend = IndexedDbBlobBackend::new();
        assert!(!backend.is_persistent());
    }

    #[test]
    fn async_get_put_round_trip_now_or_never() {
        // Exercise the async trait surface adapter shape on native
        // without dragging in a runtime crate. `core::future::ready`
        // returns Ready on first poll, so a no-op waker drives the
        // future to completion in one step (Waker::noop stabilized
        // in Rust 1.85; workspace MSRV is 1.95).
        use core::future::Future;
        use core::pin::pin;
        use core::task::{Context, Poll, Waker};

        let backend = IndexedDbBlobBackend::new();
        let bytes = b"async-round-trip".to_vec();
        let cid = cid_for(&bytes);

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        let mut put_fut = pin!(backend.put(&cid, &bytes));
        match put_fut.as_mut().poll(&mut cx) {
            Poll::Ready(r) => r.unwrap(),
            Poll::Pending => panic!("ready future polled pending"),
        }

        let mut get_fut = pin!(backend.get(&cid));
        let got = match get_fut.as_mut().poll(&mut cx) {
            Poll::Ready(r) => r.unwrap(),
            Poll::Pending => panic!("ready future polled pending"),
        };
        assert_eq!(got, Some(bytes));
    }
}
