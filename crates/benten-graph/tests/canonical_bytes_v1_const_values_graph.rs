//! S-3 closure — frozen const VALUE pins for `benten-graph`.
//!
//! The required `cargo-public-api` baseline records const TYPES only, so a
//! value-only edit to the persisted schema version or to any decode bound
//! below is an EMPTY baseline diff.

use benten_graph::aead_wrap::{IROH_BLOCK_SIZE, WHOLE_AEAD_THRESHOLD};
use benten_graph::backends::blob_backend::{
    BLOB_BYTES_PROPERTY, BLOB_CID_PROPERTY, MAX_MODULE_BYTES_ZONE_SCAN, MODULE_BYTES_LABEL,
};
use benten_graph::backends::snapshot_blob::{
    MAX_SNAPSHOT_BLOB_BYTES, SNAPSHOT_BLOB_SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// Group 1 — snapshot blob on-disk schema.
//
// WHAT BREAKS: SNAPSHOT_BLOB_SCHEMA_VERSION is the persisted schema version;
// a change without a migration makes existing snapshots unreadable.
// MAX_SNAPSHOT_BLOB_BYTES bounds a decode of stored/received bytes.
// ---------------------------------------------------------------------------

#[test]
fn snapshot_blob_constants_are_frozen() {
    assert_eq!(
        SNAPSHOT_BLOB_SCHEMA_VERSION, 2,
        "persisted snapshot-blob schema version"
    );
    assert_eq!(
        MAX_SNAPSHOT_BLOB_BYTES,
        256 * 1024 * 1024,
        "snapshot-blob decode ceiling"
    );
    assert_eq!(MAX_SNAPSHOT_BLOB_BYTES, 268_435_456, "literal value");
}

// ---------------------------------------------------------------------------
// Group 2 — blob-backend graph vocabulary + scan bound.
//
// WHAT BREAKS: the label/property keys are hashed into node CIDs.
// MAX_MODULE_BYTES_ZONE_SCAN bounds a scan driven by stored data.
// ---------------------------------------------------------------------------

#[test]
fn blob_backend_vocabulary_and_bounds_are_frozen() {
    assert_eq!(MODULE_BYTES_LABEL, "system:ModuleBytes");
    assert_eq!(BLOB_CID_PROPERTY, "blob_cid");
    assert_eq!(BLOB_BYTES_PROPERTY, "blob_bytes");
    assert_eq!(
        MAX_MODULE_BYTES_ZONE_SCAN, 100_000,
        "zone-scan ceiling — bounds a data-driven scan"
    );
}

// ---------------------------------------------------------------------------
// Group 3 — AEAD chunking mirrors.
//
// WHAT BREAKS: these are re-exports of the crypto-suite values. Pinning them
// here catches an upstream move that a re-export-only reading would let
// through, and keeps the chunk geometry visible at the storage boundary that
// actually applies it.
// ---------------------------------------------------------------------------

#[test]
fn graph_aead_chunk_geometry_mirrors_crypto_suite() {
    assert_eq!(
        IROH_BLOCK_SIZE,
        16 * 1024,
        "chunk size mirror — must match benten_crypto_suite::aead::IROH_BLOCK_SIZE"
    );
    assert_eq!(
        WHOLE_AEAD_THRESHOLD,
        64 * 1024,
        "whole-vs-chunked threshold mirror — must match \
         benten_crypto_suite::aead::WHOLE_CONTENT_AEAD_THRESHOLD"
    );
}
