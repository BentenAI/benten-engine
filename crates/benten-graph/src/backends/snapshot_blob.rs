//! Phase-2b G10-A-wasip1 read-only snapshot-blob [`KVBackend`].
//!
//! ## What this is
//!
//! D10-RESOLVED handoff shape (`.addl/phase-2b/00-implementation-plan.md` §5
//! D10): a content-addressed blob carrying a snapshot of an engine's `n:CID`
//! body keys, suitable for Phase-3 sync (peer A exports the blob, peer B
//! imports via [`SnapshotBlobBackend::from_bytes`]). The blob is a canonical
//! DAG-CBOR encoding of:
//!
//! ```text
//! SnapshotBlob {
//!     schema_version: u32,                              // currently 1
//!     anchor_cid: Option<Cid>,                          // version-anchor head if present
//!     nodes: BTreeMap<Cid, Vec<u8>>,                    // n:CID body bytes
//!     system_zone_index: BTreeMap<String, Vec<Cid>>,    // label -> CIDs
//! }
//! ```
//!
//! Both the outer struct and the two `BTreeMap`s sort by key, so two
//! engines built from identical state produce byte-identical blobs (D10 +
//! sec-pre-r1-09 Inv-13 collision-safety).
//!
//! ## Read / write posture
//!
//! - `get(b"n:" ++ cid)` returns the contained Node body bytes if present.
//! - `scan(b"n:")` enumerates every contained Node body in sorted-key
//!   order.
//! - `put` / `delete` / `put_batch` surface
//!   [`benten_errors::ErrorCode::BackendReadOnly`] (`E_BACKEND_READ_ONLY`)
//!   via [`SnapshotBlobError::ReadOnly`]. Writes against a snapshot-blob
//!   backend would break the canonical-bytes invariant the blob's CID is
//!   computed over; the contract is read-only by design.
//!
//! ## Blob CID
//!
//! [`SnapshotBlobBackend::compute_blob_cid`] computes `BLAKE3(blob_bytes)`
//! and packages it via [`benten_core::Cid::from_blake3_digest`] so the blob
//! has its own CID for Phase-3 sync. The CID of the bytes — not of the
//! decoded struct — is what makes round-trip stability load-bearing: a
//! Phase-3 peer asking "do I have blob X?" by CID gets a yes/no answer
//! that's stable across re-exports.
//!
//! Implementation notes:
//!
//! - The decoded `SnapshotBlob` is wrapped in an `Arc` so cloning the
//!   backend handle is cheap; the struct is read-only after construction
//!   so no interior mutability is needed.
//! - Reads that don't start with the `n:` Node prefix return `Ok(None)` /
//!   empty scans rather than an error: the snapshot only carries Node
//!   bodies in 2b (edges + indexes are not part of the D10 handoff shape;
//!   Phase-3 work extends the schema additively under a new
//!   `schema_version`).

use std::sync::Arc;

use benten_core::{Cid, CoreError};
use benten_errors::ErrorCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::backend::{KVBackend, ScanResult};
use crate::store::node_key;

/// Current snapshot-blob schema version. Bumped if the on-disk shape
/// changes; readers reject blobs whose `schema_version` they don't
/// understand rather than silently mis-decoding.
pub const SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 1;

/// Canonical D10 snapshot-blob payload. Encoded as DAG-CBOR; field order
/// matters for byte-stability — `serde_ipld_dagcbor` writes struct fields
/// in declaration order and sorts `BTreeMap` keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotBlob {
    /// Schema version. Currently always [`SNAPSHOT_BLOB_SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Version-anchor head CID, if the source engine had one. `None` for
    /// engines that don't use the version-chain pattern.
    pub anchor_cid: Option<Cid>,
    /// `CID -> raw DAG-CBOR Node body bytes`. BTreeMap-sorted-by-key for
    /// canonical bytes.
    pub nodes: BTreeMap<Cid, Vec<u8>>,
    /// `system_zone_label -> [CID]`. BTreeMap-sorted-by-key for canonical
    /// bytes; the inner `Vec<Cid>` preserves insertion order from the
    /// source engine's index walk (ordered by CID at construction time so
    /// the encoding is stable).
    pub system_zone_index: BTreeMap<String, Vec<Cid>>,
}

impl SnapshotBlob {
    /// Encode self as canonical DAG-CBOR bytes.
    ///
    /// # Errors
    /// [`CoreError::Serialize`] if `serde_ipld_dagcbor` cannot encode the
    /// struct.
    pub fn to_dag_cbor(&self) -> Result<Vec<u8>, CoreError> {
        serde_ipld_dagcbor::to_vec(self)
            .map_err(|e| CoreError::Serialize(format!("snapshot-blob encode: {e}")))
    }

    /// Decode canonical DAG-CBOR bytes into a [`SnapshotBlob`].
    ///
    /// # Errors
    /// [`CoreError::Serialize`] on decode failure (malformed CBOR, schema
    /// drift, etc.).
    pub fn from_dag_cbor(bytes: &[u8]) -> Result<Self, CoreError> {
        serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|e| CoreError::Serialize(format!("snapshot-blob decode: {e}")))
    }

    /// Compute the CID of an already-encoded snapshot-blob bytes blob.
    /// Same algorithm `Node::cid` uses (BLAKE3 over canonical bytes,
    /// wrapped via [`Cid::from_blake3_digest`]).
    #[must_use]
    pub fn compute_cid(bytes: &[u8]) -> Cid {
        let digest = blake3::hash(bytes);
        Cid::from_blake3_digest(*digest.as_bytes())
    }
}

/// Errors surfaced by [`SnapshotBlobBackend`].
///
/// `ReadOnly` is the dominant variant — every mutation method surfaces it
/// because the snapshot-blob is a content-addressed handoff, not a
/// mutable store. `Decode` flows from `SnapshotBlob::from_dag_cbor`
/// failures during construction; `SchemaVersion` flows from a successfully-
/// decoded blob whose `schema_version` field doesn't match the version
/// this build understands.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SnapshotBlobError {
    /// A write was attempted against the read-only snapshot-blob backend.
    #[error("backend is read-only: {operation} rejected (snapshot-blob)")]
    ReadOnly {
        /// Which mutation method was called (`"put"`, `"delete"`,
        /// `"put_batch"`). Surfaced verbatim into the catalog message so
        /// operators can pinpoint the offending call site.
        operation: &'static str,
    },
    /// Snapshot-blob bytes failed to decode as canonical DAG-CBOR.
    #[error("snapshot-blob decode: {0}")]
    Decode(#[from] CoreError),
    /// The decoded blob carries a `schema_version` this build does not
    /// understand. Refuse rather than silently mis-decode.
    #[error(
        "snapshot-blob schema mismatch: this build expects version {expected}, blob declared version {actual}"
    )]
    SchemaVersion {
        /// Schema version the running build understands
        /// ([`SNAPSHOT_BLOB_SCHEMA_VERSION`]).
        expected: u32,
        /// Schema version observed on the blob.
        actual: u32,
    },
}

impl SnapshotBlobError {
    /// Stable [`ErrorCode`] for the variant.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            SnapshotBlobError::ReadOnly { .. } => ErrorCode::BackendReadOnly,
            SnapshotBlobError::Decode(e) => e.code(),
            SnapshotBlobError::SchemaVersion { .. } => ErrorCode::Serialize,
        }
    }
}

/// Read-only [`KVBackend`] over a [`SnapshotBlob`].
///
/// Cheap to clone (the inner blob is `Arc`-shared); two clones see the
/// same underlying snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotBlobBackend {
    blob: Arc<SnapshotBlob>,
}

impl SnapshotBlobBackend {
    /// Construct a backend from a fully-formed [`SnapshotBlob`].
    #[must_use]
    pub fn new(blob: SnapshotBlob) -> Self {
        Self {
            blob: Arc::new(blob),
        }
    }

    /// Decode a snapshot-blob-bytes payload (DAG-CBOR) and wrap it as a
    /// read-only backend.
    ///
    /// # Errors
    /// - [`SnapshotBlobError::Decode`] on DAG-CBOR decode failure.
    /// - [`SnapshotBlobError::SchemaVersion`] if the blob's
    ///   `schema_version` doesn't match this build.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SnapshotBlobError> {
        let blob = SnapshotBlob::from_dag_cbor(bytes)?;
        if blob.schema_version != SNAPSHOT_BLOB_SCHEMA_VERSION {
            return Err(SnapshotBlobError::SchemaVersion {
                expected: SNAPSHOT_BLOB_SCHEMA_VERSION,
                actual: blob.schema_version,
            });
        }
        Ok(Self::new(blob))
    }

    /// Borrow the inner snapshot-blob struct.
    #[must_use]
    pub fn blob(&self) -> &SnapshotBlob {
        &self.blob
    }

    /// Re-encode the contained snapshot-blob to DAG-CBOR bytes.
    ///
    /// The result is byte-identical to the original input bytes
    /// `from_bytes` accepted, provided that input was itself canonical
    /// (`BTreeMap`-sorted, no duplicate keys). This is the load-bearing
    /// guarantee D10 round-trip stability rests on.
    ///
    /// # Errors
    /// [`CoreError::Serialize`] if `serde_ipld_dagcbor` cannot encode the
    /// struct.
    pub fn export_blob(&self) -> Result<Vec<u8>, CoreError> {
        self.blob.to_dag_cbor()
    }

    /// Compute the CID of the encoded snapshot-blob bytes (BLAKE3 wrapped
    /// via [`Cid::from_blake3_digest`]). Equivalent to
    /// `SnapshotBlob::compute_cid(&self.export_blob()?)`.
    ///
    /// # Errors
    /// [`CoreError::Serialize`] if encoding fails.
    pub fn compute_blob_cid(&self) -> Result<Cid, CoreError> {
        let bytes = self.export_blob()?;
        Ok(SnapshotBlob::compute_cid(&bytes))
    }
}

impl KVBackend for SnapshotBlobBackend {
    type Error = SnapshotBlobError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        // Snapshot-blob carries Node bodies only in the 2b D10 shape.
        // Non-`n:` reads return a clean miss rather than erroring so a
        // generic consumer (NodeStore::get_node, etc.) sees the same
        // shape it sees against the redb backend.
        let Some(rest) = key.strip_prefix(b"n:") else {
            return Ok(None);
        };
        let cid = match Cid::from_bytes(rest) {
            Ok(cid) => cid,
            // A non-CID-suffixed `n:` key is not addressable against the
            // snapshot-blob shape; treat as a clean miss.
            Err(_) => return Ok(None),
        };
        Ok(self.blob.nodes.get(&cid).cloned())
    }

    fn put(&self, _key: &[u8], _value: &[u8]) -> Result<(), Self::Error> {
        Err(SnapshotBlobError::ReadOnly { operation: "put" })
    }

    fn delete(&self, _key: &[u8]) -> Result<(), Self::Error> {
        Err(SnapshotBlobError::ReadOnly {
            operation: "delete",
        })
    }

    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, Self::Error> {
        // The snapshot-blob exposes one logical key family: `n:CID`. We
        // honor any prefix that begins with `n:` (or the empty prefix,
        // which matches everything).
        let want_nodes = prefix.is_empty() || prefix == b"n:" || prefix.starts_with(b"n:");
        if !want_nodes {
            return Ok(ScanResult::new());
        }
        let mut hits: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(self.blob.nodes.len());
        for (cid, body) in &self.blob.nodes {
            let k = node_key(cid);
            if k.starts_with(prefix) {
                hits.push((k, body.clone()));
            }
        }
        // BTreeMap iteration is already in key-sort order by `Cid`'s
        // `Ord` impl; that order matches the lexical order of the
        // assembled `n:` ++ cid_bytes keys (the prefix is constant).
        Ok(hits.into_iter().collect())
    }

    fn put_batch(&self, _pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error> {
        Err(SnapshotBlobError::ReadOnly {
            operation: "put_batch",
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests + benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    fn one_node_blob() -> SnapshotBlob {
        let node = canonical_test_node();
        let cid = node.cid().unwrap();
        let body = serde_ipld_dagcbor::to_vec(&node).unwrap();
        let mut nodes = BTreeMap::new();
        nodes.insert(cid, body);
        SnapshotBlob {
            schema_version: SNAPSHOT_BLOB_SCHEMA_VERSION,
            anchor_cid: None,
            nodes,
            system_zone_index: BTreeMap::new(),
        }
    }

    /// `snapshot_blob_kvbackend_read_path` — brief must-pass test #3.
    /// A `get(node_key(cid))` against a snapshot-blob containing that CID
    /// returns the stored body bytes; an absent CID returns `Ok(None)`.
    #[test]
    fn snapshot_blob_kvbackend_read_path() {
        let blob = one_node_blob();
        let cid = *blob.nodes.keys().next().unwrap();
        let backend = SnapshotBlobBackend::new(blob);

        let key = crate::store::node_key(&cid);
        let v = backend
            .get(&key)
            .unwrap()
            .expect("present CID must read back");
        let decoded: benten_core::Node = serde_ipld_dagcbor::from_slice(&v).unwrap();
        assert_eq!(decoded.cid().unwrap(), cid);

        // Absent CID returns clean miss.
        let other = canonical_test_node().cid().unwrap();
        let mut other_bytes = *other.as_bytes();
        other_bytes[5] ^= 0xff;
        let other_cid = Cid::from_bytes(&other_bytes).unwrap();
        let other_key = crate::store::node_key(&other_cid);
        assert!(backend.get(&other_key).unwrap().is_none());
    }

    /// `snapshot_blob_kvbackend_rejects_writes` — brief must-pass test #4.
    /// Every mutation method surfaces `ErrorCode::BackendReadOnly`.
    #[test]
    fn snapshot_blob_kvbackend_rejects_writes() {
        let backend = SnapshotBlobBackend::new(one_node_blob());

        let put_err = backend.put(b"n:k", b"v").unwrap_err();
        assert_eq!(put_err.code(), ErrorCode::BackendReadOnly);

        let del_err = backend.delete(b"n:k").unwrap_err();
        assert_eq!(del_err.code(), ErrorCode::BackendReadOnly);

        let batch_err = backend
            .put_batch(&[(b"n:k".to_vec(), b"v".to_vec())])
            .unwrap_err();
        assert_eq!(batch_err.code(), ErrorCode::BackendReadOnly);
    }

    /// Round-trip stability — encode → decode → re-encode yields the same
    /// bytes (D10 canonical-bytes discipline).
    #[test]
    fn snapshot_blob_round_trip_byte_identical_at_backend_layer() {
        let blob = one_node_blob();
        let bytes_a = blob.to_dag_cbor().unwrap();
        let backend = SnapshotBlobBackend::from_bytes(&bytes_a).unwrap();
        let bytes_b = backend.export_blob().unwrap();
        assert_eq!(
            bytes_a, bytes_b,
            "snapshot-blob encode -> decode -> re-encode must be byte-identical"
        );
    }

    /// Two blobs built from identical inputs encode to the same bytes
    /// (BTreeMap-sorted-by-key discipline; sec-pre-r1-09 Inv-13).
    #[test]
    fn snapshot_blob_btreemap_canonical_bytes_stable_at_backend_layer() {
        let bytes1 = one_node_blob().to_dag_cbor().unwrap();
        let bytes2 = one_node_blob().to_dag_cbor().unwrap();
        assert_eq!(bytes1, bytes2);
    }

    /// `compute_blob_cid` is stable across encode → decode → re-encode.
    #[test]
    fn snapshot_blob_cid_stable_across_round_trip() {
        let blob = one_node_blob();
        let bytes_a = blob.to_dag_cbor().unwrap();
        let cid_a = SnapshotBlob::compute_cid(&bytes_a);

        let backend = SnapshotBlobBackend::from_bytes(&bytes_a).unwrap();
        let cid_b = backend.compute_blob_cid().unwrap();
        assert_eq!(cid_a, cid_b);
    }

    /// Schema-version mismatch is rejected (forward-compat: a v2 blob
    /// reaching a v1-only build refuses rather than silently mis-decodes).
    #[test]
    fn snapshot_blob_schema_version_mismatch_rejected() {
        let mut blob = one_node_blob();
        blob.schema_version = SNAPSHOT_BLOB_SCHEMA_VERSION + 1;
        let bytes = blob.to_dag_cbor().unwrap();
        let err = SnapshotBlobBackend::from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, SnapshotBlobError::SchemaVersion { .. }));
    }

    /// `scan` honors the `n:` prefix and returns sorted-by-key order.
    #[test]
    fn snapshot_blob_scan_n_prefix_sorted() {
        let blob = one_node_blob();
        let backend = SnapshotBlobBackend::new(blob);
        let hits = backend.scan(b"n:").unwrap();
        assert_eq!(hits.len(), 1);
        // Non-`n:` prefix returns empty (the snapshot only carries Node bodies).
        assert!(backend.scan(b"e:").unwrap().is_empty());
    }
}
