//! Phase-2b G10-A-wasip1 — `SnapshotBlobBackend` `KVBackend` integration
//! tests (D10-RESOLVED).
//!
//! Brief must-pass tests:
//!
//! - `snapshot_blob_kvbackend_read_path` — present CID is readable;
//!   absent CID returns clean miss; non-`n:` keys return `Ok(None)`.
//! - `snapshot_blob_kvbackend_rejects_writes` — every mutation method
//!   surfaces `ErrorCode::BackendReadOnly`.
//! - `snapshot_blob_round_trips_export_import` — export -> from_bytes ->
//!   re-export yields byte-identical output (D10 canonical-bytes).
//! - `snapshot_blob_btreemap_canonical_bytes_stable` — two blobs built
//!   from identical state encode to the same bytes (Inv-13 collision
//!   safety; sec-pre-r1-09).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, testing::canonical_test_node};
use benten_errors::ErrorCode;
use benten_graph::{
    KVBackend, SnapshotBlob, SnapshotBlobBackend,
    backends::snapshot_blob::SNAPSHOT_BLOB_SCHEMA_VERSION,
};

fn one_node_blob() -> (Cid, SnapshotBlob) {
    let node = canonical_test_node();
    let cid = node.cid().unwrap();
    let body = serde_ipld_dagcbor::to_vec(&node).unwrap();
    let mut nodes = BTreeMap::new();
    nodes.insert(cid, body);
    (
        cid,
        SnapshotBlob {
            schema_version: SNAPSHOT_BLOB_SCHEMA_VERSION,
            anchor_cid: None,
            nodes,
            system_zone_index: BTreeMap::new(),
        },
    )
}

fn node_key(cid: &Cid) -> Vec<u8> {
    let mut k = b"n:".to_vec();
    k.extend_from_slice(cid.as_bytes());
    k
}

#[test]
fn snapshot_blob_kvbackend_read_path() {
    let (cid, blob) = one_node_blob();
    let backend = SnapshotBlobBackend::new(blob);

    // Present CID returns its body bytes.
    let present = backend.get(&node_key(&cid)).unwrap();
    assert!(present.is_some(), "present CID must read back");

    // Absent CID returns clean miss.
    let mut absent_bytes = *cid.as_bytes();
    absent_bytes[5] ^= 0xff;
    let absent_cid = Cid::from_bytes(&absent_bytes).unwrap();
    let absent = backend.get(&node_key(&absent_cid)).unwrap();
    assert!(absent.is_none(), "absent CID must clean-miss");

    // Non-`n:` key returns clean miss (snapshot-blob carries Node bodies only).
    let other = backend.get(b"e:foo").unwrap();
    assert!(other.is_none(), "non-Node key must clean-miss");
}

#[test]
fn snapshot_blob_kvbackend_rejects_writes() {
    let (_cid, blob) = one_node_blob();
    let backend = SnapshotBlobBackend::new(blob);

    let put_err = backend.put(b"n:k", b"v").unwrap_err();
    assert_eq!(put_err.code(), ErrorCode::BackendReadOnly);

    let del_err = backend.delete(b"n:k").unwrap_err();
    assert_eq!(del_err.code(), ErrorCode::BackendReadOnly);

    let batch_err = backend
        .put_batch(&[(b"n:k".to_vec(), b"v".to_vec())])
        .unwrap_err();
    assert_eq!(batch_err.code(), ErrorCode::BackendReadOnly);
}

#[test]
fn snapshot_blob_round_trips_export_import() {
    let (_cid, blob) = one_node_blob();
    let bytes_a = blob.to_dag_cbor().unwrap();

    let backend = SnapshotBlobBackend::from_bytes(&bytes_a).unwrap();
    let bytes_b = backend.export_blob().unwrap();

    assert_eq!(
        bytes_a, bytes_b,
        "snapshot-blob export -> import -> re-export must be byte-identical \
         (D10 canonical-bytes discipline)"
    );
}

#[test]
fn snapshot_blob_btreemap_canonical_bytes_stable() {
    let (_, blob1) = one_node_blob();
    let (_, blob2) = one_node_blob();
    let bytes1 = blob1.to_dag_cbor().unwrap();
    let bytes2 = blob2.to_dag_cbor().unwrap();
    assert_eq!(
        bytes1, bytes2,
        "two blobs built from identical state must encode to identical bytes \
         (BTreeMap-sorted-by-key discipline; Inv-13 collision-safety)"
    );
}

#[test]
fn snapshot_blob_scan_n_prefix_returns_node_bodies() {
    let (cid, blob) = one_node_blob();
    let backend = SnapshotBlobBackend::new(blob);
    let hits = backend.scan(b"n:").unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits.iter().next().unwrap().0, node_key(&cid));
}

#[test]
fn snapshot_blob_scan_e_prefix_is_empty() {
    let (_cid, blob) = one_node_blob();
    let backend = SnapshotBlobBackend::new(blob);
    let hits = backend.scan(b"e:").unwrap();
    assert!(hits.is_empty(), "snapshot-blob carries no edges in 2b");
}
