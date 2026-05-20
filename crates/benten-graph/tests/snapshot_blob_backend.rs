//! G13-D wave-3 (GREEN-PHASE): `SnapshotBlobBackend` direct-wire as a
//! first-class [`benten_graph::GraphBackend`].
//!
//! Pin sources (per r2-test-landscape §2.1 G13-D + plan §3 G13-D
//! must-pass column):
//!
//! - `tests/snapshot_blob_backend_impls_graph_backend_read_path` — plan §3 G13-D
//! - `tests/snapshot_blob_backend_write_path_returns_read_only_error` — plan §3 G13-D
//!
//! ## What G13-D delivers
//!
//! Phase-2b shipped a read-only `SnapshotBlobBackend` impl wired only at
//! the [`benten_graph::KVBackend`] surface. G13-D promotes the
//! snapshot-blob to satisfy the umbrella [`benten_graph::GraphBackend`]
//! trait (via [`benten_graph::NodeStore`] + [`benten_graph::EdgeStore`]
//! + the umbrella's snapshot/transaction/subscriber/put-with-context
//! surface) so the engine consumes it directly without spilling to a
//! tempdir-backed `RedbBackend` (see companion pin
//! `crates/benten-engine/tests/snapshot_no_tempdir.rs`).
//!
//! - Read-path methods delegate to the existing in-memory blob lookup.
//! - Write-path methods return [`benten_errors::ErrorCode::BackendReadOnly`]
//!   with a typed error; the snapshot-blob is content-addressed and
//!   writes would break the canonical-bytes invariant.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use benten_core::testing::canonical_test_node;
use benten_core::{Cid, Node, WriteAuthority};
use benten_errors::ErrorCode;
use benten_graph::{
    ChangeSubscriber, EdgeStore, GraphBackend, KVBackend, NodeStore, SnapshotBlob,
    SnapshotBlobBackend, WriteContext,
};

/// Build a minimal one-node snapshot-blob backend for the read-path /
/// write-path pins. Same shape the in-tree unit tests in
/// `crates/benten-graph/src/backends/snapshot_blob.rs::tests` use, but
/// reachable from the integration-test target.
fn one_node_backend() -> (Cid, SnapshotBlobBackend) {
    let node = canonical_test_node();
    let cid = node.cid().unwrap();
    let body = serde_ipld_dagcbor::to_vec(&node).unwrap();
    let mut nodes: BTreeMap<Cid, Vec<u8>> = BTreeMap::new();
    nodes.insert(cid, body);
    let blob = SnapshotBlob {
        schema_version: benten_graph::backends::snapshot_blob::SNAPSHOT_BLOB_SCHEMA_VERSION,
        anchor_cid: None,
        nodes,
        system_zone_index: BTreeMap::new(),
    };
    (cid, SnapshotBlobBackend::new(blob))
}

/// Plan §3 G13-D / r2-test-landscape §2.1 G13-D — `SnapshotBlobBackend`
/// satisfies [`GraphBackend`] and the read-path returns the expected
/// node bytes.
///
/// Defends against G13-D landing the read-path delegate but forgetting
/// to add the umbrella `impl GraphBackend for SnapshotBlobBackend`
/// adapter (which would force consumers back through the tempdir
/// `RedbBackend` hop the wave was meant to retire).
#[test]
fn snapshot_blob_backend_impls_graph_backend_read_path() {
    let (cid, backend) = one_node_backend();

    // Compile-time witness — the umbrella + each sub-trait are
    // satisfied. A refactor that drops any of them fails this test's
    // compile, mirroring the redb-side pin in
    // `crates/benten-graph/tests/graph_backend_trait.rs::redb_backend_impls_graph_backend`.
    fn assert_graph_backend<B: GraphBackend>(_: &B) {}
    fn assert_kv<B: KVBackend>(_: &B) {}
    fn assert_node<B: NodeStore>(_: &B) {}
    fn assert_edge<B: EdgeStore>(_: &B) {}
    assert_graph_backend(&backend);
    assert_kv(&backend);
    assert_node(&backend);
    assert_edge(&backend);

    // Read-path runtime witness — `NodeStore::get_node` resolves the
    // populated CID by decoding the canonical-bytes body the snapshot
    // carries.
    let node = NodeStore::get_node(&backend, &cid)
        .expect("snapshot-blob NodeStore::get_node must not error on populated CID")
        .expect("snapshot-blob get_node must return Some(node) for present CID");
    assert_eq!(
        node.cid().unwrap(),
        cid,
        "decoded node CID must match the snapshot-blob key"
    );

    // Read-path miss — an absent CID returns `Ok(None)` rather than
    // erroring.
    let mut other_bytes = *cid.as_bytes();
    other_bytes[5] ^= 0xff;
    let other_cid = Cid::from_bytes(&other_bytes).unwrap();
    assert!(NodeStore::get_node(&backend, &other_cid).unwrap().is_none());

    // The umbrella's snapshot / transaction / subscriber surface is
    // present and infallibly callable (per the trait shape — failures
    // would surface through Self::Error which the trait pins as `Send +
    // Sync + 'static`).
    let _snap: <SnapshotBlobBackend as GraphBackend>::Snapshot = backend.snapshot();
    let _txn: <SnapshotBlobBackend as GraphBackend>::Transaction = backend.transaction();

    // No-op subscriber registration: the snapshot-blob is immutable so
    // it never fans events out, but the trait surface accepts the
    // registration silently so the engine can wire IVM uniformly.
    let sub: Arc<dyn ChangeSubscriber> = Arc::new(NoOpSubscriber);
    backend.register_subscriber(sub);

    // Edges aren't part of the D10 schema-v1 handoff shape — the
    // accessors return empty rather than error.
    assert!(backend.edges_from(&cid).unwrap().is_empty());
    assert!(backend.edges_to(&cid).unwrap().is_empty());
}

/// Plan §3 G13-D / r2-test-landscape §2.1 G13-D — write-path methods
/// surface `ErrorCode::BackendReadOnly` rather than corrupting the
/// content-addressed snapshot blob.
///
/// Mirrors the put / delete / put_batch coverage already pinned at the
/// [`KVBackend`] surface (`snapshot_blob_kvbackend_rejects_writes` in
/// the in-tree unit tests) but exercises the upgraded
/// [`NodeStore`] / [`EdgeStore`] / [`GraphBackend`] surfaces G13-D
/// adds.
#[test]
fn snapshot_blob_backend_write_path_returns_read_only_error() {
    let (cid, backend) = one_node_backend();

    // KVBackend writes — already pinned in unit tests; re-pinned here
    // for cross-target visibility (this test ships in the integration
    // test target).
    let put_err = backend.put(b"n:k", b"v").unwrap_err();
    assert_eq!(put_err.code(), ErrorCode::BackendReadOnly);
    let del_err = backend.delete(b"n:k").unwrap_err();
    assert_eq!(del_err.code(), ErrorCode::BackendReadOnly);
    let batch_err = backend
        .put_batch(&[(b"n:k".to_vec(), b"v".to_vec())])
        .unwrap_err();
    assert_eq!(batch_err.code(), ErrorCode::BackendReadOnly);

    // NodeStore writes — new at G13-D.
    let put_node_err = NodeStore::put_node(&backend, &canonical_test_node()).unwrap_err();
    assert_eq!(put_node_err.code(), ErrorCode::BackendReadOnly);
    let delete_node_err = NodeStore::delete_node(&backend, &cid).unwrap_err();
    assert_eq!(delete_node_err.code(), ErrorCode::BackendReadOnly);

    // EdgeStore writes — new at G13-D.
    let edge = sample_edge(&cid);
    let put_edge_err = EdgeStore::put_edge(&backend, &edge).unwrap_err();
    assert_eq!(put_edge_err.code(), ErrorCode::BackendReadOnly);
    let edge_cid = edge.cid().unwrap();
    let delete_edge_err = EdgeStore::delete_edge(&backend, &edge_cid).unwrap_err();
    assert_eq!(delete_edge_err.code(), ErrorCode::BackendReadOnly);

    // GraphBackend privileged put — even an EnginePrivileged authority
    // does NOT bypass the read-only contract. The snapshot blob is
    // content-addressed; the Inv-13 5-row matrix has no live dispatch
    // surface to thread through.
    let ctx = WriteContext {
        label: String::new(),
        is_privileged: true,
        authority: WriteAuthority::EnginePrivileged,
        // G-CORE-1 #989: snapshot-blob backend exercises the legacy
        // un-namespaced path; the per-DID storage-partition seam is the
        // redb path's concern.
        namespace_did: None,
    };
    let priv_err = <SnapshotBlobBackend as GraphBackend>::put_node_with_context(
        &backend,
        &canonical_test_node(),
        &ctx,
    )
    .unwrap_err();
    assert_eq!(priv_err.code(), ErrorCode::BackendReadOnly);
}

fn sample_edge(target: &Cid) -> benten_core::Edge {
    use benten_core::Edge;
    Edge::new(*target, *target, "ref", Default::default())
}

/// Throwaway subscriber so the read-path test can exercise the no-op
/// `register_subscriber` surface without depending on the engine's
/// production `ChangeBroadcast` plumbing.
struct NoOpSubscriber;

impl ChangeSubscriber for NoOpSubscriber {
    fn on_change(&self, _event: &benten_graph::ChangeEvent) {}
}
