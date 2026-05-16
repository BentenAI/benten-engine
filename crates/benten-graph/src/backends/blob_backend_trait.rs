//! Phase-3 G13-pre-B — `BlobBackend` trait surface scaffold.
//!
//! ## What this is
//!
//! Phase-3 wave-1pre orchestrator-direct scaffold that locks the public
//! shape of the durable blob storage abstraction. Two concrete impls land
//! in later Phase-3 waves and consume this trait surface from day one:
//!
//! 1. **G14-C native blob backend** (`crates/benten-graph/src/backends/
//!    blob_backend.rs`) — redb side-table, full peer / persistent.
//! 2. **G18-A browser blob backend** (`crates/benten-graph/src/backends/
//!    browser_blob_backend.rs` — wasm32-unknown-unknown only) —
//!    IndexedDB-backed thin-client cache (per CLAUDE.md baked-in #17).
//!
//! Locking the trait surface in wave-1pre mirrors the Phase-2b chunk-sink-
//! scaffold precedent (seq-major-6): downstream consumers (G14-C engine
//! `register_module_bytes` CID validation per D-PHASE-3-12, G18-A
//! cross-browser determinism CI per D-PHASE-3-7) can be sequenced
//! independently because the API boundary is fixed early.
//!
//! ## Decisions baked in
//!
//! - **Generic-cascade per D-PHASE-3-1 RESOLVED (R1-revision 2026-05-04).**
//!   The trait is intentionally **NOT object-safe**: it carries an
//!   associated `type Error` (so each backend surfaces its own typed-error
//!   enum without `Box<dyn Error>` erasure inside the generic impl per
//!   D-PHASE-3-1a) AND its methods return `impl Future + Send` (RPITIT,
//!   stable since 1.75 — also incompatible with `dyn` erasure). Engine
//!   composition is therefore generic-cascade — `Engine<B: GraphBackend>`
//!   plus storage-side `BlobBackend` consumed via concrete or generic
//!   type parameters, never `Arc<dyn BlobBackend>`. The `tests/
//!   blob_backend_trait_object_safety_per_d_resolution.rs` integration
//!   test pins the generic-cascade direction as a positive smoke; the
//!   non-object-safety property is guaranteed *by construction* (the type
//!   system rejects `dyn BlobBackend` because of the associated type +
//!   RPITIT, no separate runtime assertion required).
//!
//! - **Async-shape baked in (browser-target compatibility).** Methods
//!   return `impl Future<Output = ...> + Send` so the IndexedDB-browser
//!   variant — inherently async because of the JS `IDBRequest` / Promise
//!   wire — can implement the trait without an async-runtime adapter.
//!   The native redb impl wraps the synchronous redb API in
//!   `core::future::ready(...)` (sync-blocking shim — redb itself has no
//!   async surface). The `tests/blob_backend_trait_browser_target_
//!   compatible_signatures.rs` integration test pins that the trait
//!   compiles + can be implemented without dragging tokio / std::thread
//!   into the dep graph; the wasm32-unknown-unknown target compile is
//!   verified in CI (`wasm-checks.yml` extends to `-p benten-graph`).
//!
//! - **CID-validating `put` per D-PHASE-3-12 RESOLVED.** The end-to-end
//!   CID-validation contract lives at the `Engine::register_module_bytes`
//!   call site (G14-C). The trait-level `put` accepts a caller-supplied
//!   `Cid`; the implementation MAY recompute BLAKE3 over the bytes and
//!   reject mismatches, OR delegate that check to the caller. G14-C's
//!   redb impl recomputes locally for defense in depth; G18-A's IndexedDB
//!   variant inherits the same defense (an attacker writing into IndexedDB
//!   directly cannot poison the cache because the next read recomputes
//!   the BLAKE3 of the stored bytes against the queried CID).
//!
//! - **Thin-client commitment per CLAUDE.md baked-in #17.** The native
//!   variant is a full peer (persistent across engine restarts); the
//!   browser variant is a thin-client cache (in-RAM `BTreeMap` or
//!   IndexedDB-backed but cache-only — never the source of truth, never
//!   the sync state). `is_persistent()` distinguishes the two so callers
//!   that need durability guarantees can refuse to wire a non-persistent
//!   backend.
//!
//! ## Method shape
//!
//! Three methods cover the minimum-viable consumer surface for G14-C +
//! G18-A:
//!
//! - `get(cid)` → `Option<Vec<u8>>` — fetch the bytes for a CID; `None`
//!   on a clean miss (information, not failure).
//! - `put(cid, bytes)` → `()` — store bytes under a caller-supplied CID.
//!   Implementations MAY validate `BLAKE3(bytes) == cid` and surface a
//!   typed mismatch error; the contract permits but does not require it
//!   (D-PHASE-3-12 makes the engine-side validator authoritative).
//! - `is_persistent()` → `bool` — true on full-peer durable backends
//!   (redb-native), false on thin-client cache backends (browser).
//!
//! Future Phase-3 waves may extend this surface (delete, scan, batch);
//! the `#[non_exhaustive]` posture is reserved on the trait via additive
//! default-method extensions rather than enum-style `#[non_exhaustive]`.
//!
//! ## Additive evolution exercised (Fwd-2 #1012)
//!
//! The documented additive-default-method evolution posture is now
//! *exercised* (not just paper) ahead of the v1-SemVer-lock so the
//! evolution-cost is known:
//!
//! - `delete(cid)` — evict module-bytes from the durable side-table
//!   (Phase-4-Meta plugin uninstall + rotation-log revocation paths
//!   need this). Default: idempotent `Ok(())` no-op (mirrors
//!   `KVBackend::delete` idempotent semantics) so non-mutating /
//!   thin-client backends satisfy the trait unchanged.
//! - `list_cids()` — enumerate every blob CID in the backend
//!   (rehydrate-on-engine-open + plugin-library walk). Default:
//!   `Ok(Vec::new())` so backends without an enumeration index satisfy
//!   the trait unchanged; `RedbBlobBackend` overrides with its existing
//!   inherent `list_blob_cids` full-zone walk.
//!
//! Both land as **additive default methods** (not sub-traits) per the
//! documented posture — every existing impl compiles unchanged. The
//! v1-stabilization fork (additive-default vs. `MutableBlobBackend` /
//! `EnumerableBlobBackend` sub-trait split) is named at
//! `docs/future/phase-4-backlog.md §4.61`.

use core::future::Future;

use benten_core::Cid;

/// Durable blob storage abstraction. See the module-level docstring for
/// the full design rationale (D-PHASE-3-1 / D-PHASE-3-1a / D-PHASE-3-12 /
/// D-PHASE-3-7 + CLAUDE.md baked-in #17 thin-client commitment).
///
/// **Not object-safe** (associated `type Error` + RPITIT method returns).
/// Consume via generic-cascade — `fn install<B: BlobBackend>(...)` — never
/// via `Arc<dyn BlobBackend>` / `Box<dyn BlobBackend>`.
pub trait BlobBackend: Send + Sync + 'static {
    /// Backend-specific typed-error enum. Constrained to the standard
    /// error-object shape so consumers can `.source()`-chain across
    /// heterogeneous backends without `Box<dyn Error>` erasure inside the
    /// generic impl.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Fetch the bytes stored under `cid`. Returns `Ok(None)` on a clean
    /// miss — a missing CID is information, not a failure.
    ///
    /// # Errors
    /// Returns `Self::Error` on a backend I/O failure (e.g. the underlying
    /// `KVBackend` read fails, the durable side-table is unreadable, or a
    /// stored payload fails to decode). A missing CID is `Ok(None)`, not an
    /// error.
    fn get(&self, cid: &Cid) -> impl Future<Output = Result<Option<Vec<u8>>, Self::Error>> + Send;

    /// Store `bytes` under `cid`. Implementations MAY recompute
    /// `BLAKE3(bytes) == cid` and surface a typed mismatch error;
    /// D-PHASE-3-12's authoritative validator lives at the engine call
    /// site (`Engine::register_module_bytes`), so the trait-level put
    /// permits but does not require local recomputation.
    ///
    /// # Errors
    /// Returns `Self::Error` on a backend write failure (e.g. the
    /// underlying `KVBackend` put fails) or, for impls that opt into
    /// local recomputation, a typed `BLAKE3(bytes) != cid` mismatch.
    fn put(&self, cid: &Cid, bytes: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// True iff this backend persists across engine restarts. Native
    /// redb-backed impls return `true`; browser thin-client cache impls
    /// return `false` per CLAUDE.md baked-in #17.
    fn is_persistent(&self) -> bool;

    /// Evict the blob stored under `cid` from the durable side-table.
    ///
    /// Idempotent: deleting an absent CID is `Ok(())`, never an error
    /// (mirrors [`crate::KVBackend::delete`] semantics). Phase-4-Meta
    /// plugin-uninstall + rotation-log revocation paths consume this.
    ///
    /// **Additive default (Fwd-2 #1012):** the default is an idempotent
    /// no-op so non-mutating / thin-client backends satisfy the trait
    /// unchanged. `RedbBlobBackend` overrides with a real eviction.
    fn delete(&self, cid: &Cid) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let _ = cid;
        core::future::ready(Ok(()))
    }

    /// Enumerate the CIDs of every blob currently persisted in this
    /// backend. Used by rehydrate-on-engine-open + the plugin-library
    /// walk.
    ///
    /// **Additive default (Fwd-2 #1012):** the default is
    /// `Ok(Vec::new())` so backends without an enumeration index satisfy
    /// the trait unchanged. `RedbBlobBackend` overrides with its
    /// existing inherent full-zone-walk (`list_blob_cids`).
    fn list_cids(&self) -> impl Future<Output = Result<Vec<Cid>, Self::Error>> + Send {
        core::future::ready(Ok(Vec::new()))
    }
}
