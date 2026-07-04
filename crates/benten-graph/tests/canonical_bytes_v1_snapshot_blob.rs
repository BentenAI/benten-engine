//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pin for
//! SnapshotBlob v2 canonical DAG-CBOR (Rows D-9 / D-79).
//!
//! Per Row D-9, the SnapshotBlob roundtrip + schema-version-position pins
//! already landed (`backends/snapshot_blob.rs` unit tests). This file
//! adds the MISSING ABSOLUTE hex byte-pin: the full canonical DAG-CBOR
//! encoding of a fixed v2 SnapshotBlob is frozen to literal bytes so no
//! field reorder / schema-version drift / CBOR-encoder change can slip
//! past silently.
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! The fixture is fully deterministic: a v2 blob carrying exactly ONE
//! node (`benten_core::testing::canonical_test_node`, a fixed CID/body)
//! with `anchor_cid = None`, empty `system_zone_index`, `merkle_root =
//! None`. `serde_ipld_dagcbor` writes struct fields in declaration order
//! + sorts BTreeMap keys, so two runs produce byte-identical output. The
//! hex was captured via throwaway compute and pasted below.
//!
//! would-FAIL-on-drift: a field reorder, a `schema_version` bump, a new
//! appended field, or a CBOR-encoder change all change these bytes.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::testing::canonical_test_node;
use benten_graph::backends::snapshot_blob::{SNAPSHOT_BLOB_SCHEMA_VERSION, SnapshotBlob};

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Build the canonical fixture: one-node v2 blob, everything else empty.
fn fixture_blob() -> SnapshotBlob {
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
        merkle_root: None,
    }
}

#[test]
fn snapshot_blob_v2_canonical_bytes_golden_hex() {
    let blob = fixture_blob();
    let bytes = blob.to_canonical_bytes().unwrap();
    let got = to_hex(&bytes);

    // ABSOLUTE golden hex — captured via throwaway compute over the fixed
    // one-node v2 fixture (schema_version=2, anchor_cid=None,
    // one canonical_test_node, empty index, merkle_root=None).
    let expected = "a5656e6f646573a1582401711e20abcac66ca633534959203ae12c881a69f75ecd227475b78b95f0efda56587418985218a21866186c186118621865186c1873188118641850186f18731874186a18701872186f187018651872187418691865187318a4186418741861186718731882186418721875187318741865186718721861187018681865187418691874186c1865186d18481865186c186c186f182c182018421865186e18741865186e1865187618691865187718731818182a1869187018751862186c1869187318681865186418f56a616e63686f725f636964f66b6d65726b6c655f726f6f74f66e736368656d615f76657273696f6e027173797374656d5f7a6f6e655f696e646578a0";
    assert_eq!(
        got, expected,
        "SnapshotBlob v2 canonical DAG-CBOR drifted from the frozen v1-beta bytes"
    );
}
