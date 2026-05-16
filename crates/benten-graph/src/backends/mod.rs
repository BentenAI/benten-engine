//! Phase-2b non-redb [`KVBackend`](crate::KVBackend) implementations and
//! Phase-3 storage-trait scaffolds.
//!
//! Each backend in this module is a read-mostly waist that serves a Phase-2b
//! G10 deliverable:
//!
//! - [`snapshot_blob`] — D10-RESOLVED snapshot-blob `KVBackend`. Decodes a
//!   DAG-CBOR blob produced by [`SnapshotBlobBackend::export_blob`] and
//!   serves the contained `n:CID` body keys read-only. Writes surface
//!   [`benten_errors::ErrorCode::BackendReadOnly`] (`E_BACKEND_READ_ONLY`).
//! - [`network_fetch_stub`] — Phase-2a §9.8 typed-error-only stub. Reserves
//!   the `KVBackend` shape for the Phase-3 iroh-fetch implementation; in
//!   2b every operation surfaces a typed `Phase3DeferredFetch` error so
//!   any test/binary that wires the stub fails loud rather than silently
//!   serving zeros.
//! - [`blob_backend_trait`] — Phase-3 G13-pre-B trait surface scaffold for
//!   the durable blob storage abstraction. Locks the `BlobBackend` API so
//!   G14-C (redb-native) and G18-A (IndexedDB-browser) impls can be
//!   sequenced independently. Generic-cascade per D-PHASE-3-1; async-shape
//!   per D-PHASE-3-7; CID-validating `put` per D-PHASE-3-12.
//! - `blob_backend` — Phase-3 G14-C concrete redb-native `RedbBlobBackend`
//!   implementation (native target only; cfg'd out under the
//!   `browser-backend` feature) closing **Compromise #17** (in-memory
//!   module-bytes
//!   registry). Stores blobs as `system:ModuleBytes` zone Nodes;
//!   defense-in-depth CID re-check at the put boundary mirrors
//!   `Engine::register_module_bytes`'s authoritative validator.
//!
//! The redb-backed [`crate::RedbBackend`] is the production native-target
//! backend and lives at the crate root for legacy-call-site discoverability.

// `blob_backend` depends on `crate::redb_backend::RedbBackend`, so it must
// share `redb_backend`'s gate exactly: available whenever redb is (any
// non-`wasm32-unknown-unknown` target, including `wasm32-wasip1`),
// independent of the `browser-backend` feature. The prior
// `cfg(not(feature = "browser-backend"))` gate inverted the relationship —
// enabling `browser-backend` on a native target (a valid combination per
// the module docs) wrongly cfg'd out the production-critical
// `RedbBlobBackend` even though `redb_backend` itself stayed compiled
// (Surf-1 #851 feature-flag inversion).
#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
pub mod blob_backend;
pub mod blob_backend_trait;
pub mod network_fetch_stub;
pub mod snapshot_blob;

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
pub use blob_backend::{
    BLOB_BYTES_PROPERTY, BLOB_CID_PROPERTY, BlobError, MODULE_BYTES_LABEL, RedbBlobBackend,
};
pub use blob_backend_trait::BlobBackend;
pub use network_fetch_stub::{NetworkFetchStubBackend, NetworkFetchStubError};
pub use snapshot_blob::{SnapshotBlob, SnapshotBlobBackend, SnapshotBlobError};
