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
//!
//! The redb-backed [`crate::RedbBackend`] is the production native-target
//! backend and lives at the crate root for legacy-call-site discoverability.

pub mod blob_backend_trait;
pub mod network_fetch_stub;
pub mod snapshot_blob;

pub use blob_backend_trait::BlobBackend;
pub use network_fetch_stub::{NetworkFetchStubBackend, NetworkFetchStubError};
pub use snapshot_blob::{SnapshotBlob, SnapshotBlobBackend, SnapshotBlobError};
