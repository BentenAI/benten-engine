//! R3 unit tests for G2-A + E11: Inv-13 immutability — second put_node with
//! the same CID + User authority fires `E_INV_IMMUTABILITY`. Bloom-filter CID-
//! existence cache covers the fast-path; exact-check fallback covers bloom
//! false positives; cache warms on first put.
//!
//! TDD red-phase: `WriteAuthority`, the bloom cache, and the immutability
//! branch do not yet exist on `RedbBackend`. Tests will fail to compile until
//! G2-A + G2-B land.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.3, plan §9.11 rows 1-2).

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use benten_errors::ErrorCode;
use benten_graph::{RedbBackend, WriteAuthority, WriteContext};
use std::collections::BTreeMap;

fn sample_node(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text(title));
    Node::new(vec!["Post".into()], props)
}

fn open_backend() -> (tempfile::TempDir, RedbBackend) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("imm.redb");
    let backend = RedbBackend::open_or_create(path).expect("open");
    (dir, backend)
}

#[test]
fn immutability_rejects_reput() {
    let (_dir, backend) = open_backend();
    let node = sample_node("first-write");
    let ctx = WriteContext::new("Post").with_authority(WriteAuthority::User);

    // First put succeeds.
    let cid1 = backend
        .put_node_with_context(&node, &ctx)
        .expect("first put");

    // Second put of byte-identical content under User authority must fire
    // E_INV_IMMUTABILITY (row 1 of the 5-row matrix).
    let err = backend
        .put_node_with_context(&node, &ctx)
        .expect_err("re-put must deny");
    assert_eq!(
        err.code(),
        ErrorCode::InvImmutability,
        "User re-put of same CID must fire E_INV_IMMUTABILITY"
    );
    // The caller's CID handle on the first put is preserved (sanity).
    let _ = cid1;
}

#[test]
fn immutability_bloom_false_positive_falls_back_to_exact_check() {
    let (_dir, backend) = open_backend();

    // Prime the bloom cache with a distinct Node, then issue a put for a
    // different Node whose bloom hash collides. Backend under test exposes a
    // test-only `force_bloom_collision_for_next_put` shim so the path is
    // reproducible. Exact redb lookup must still correctly return "not found"
    // and the put must succeed.
    let primed = sample_node("primed");
    let ctx = WriteContext::new("Post").with_authority(WriteAuthority::User);
    backend
        .put_node_with_context(&primed, &ctx)
        .expect("primed write");

    let distinct = sample_node("distinct");
    backend.force_bloom_collision_for_next_put();

    let cid = backend
        .put_node_with_context(&distinct, &ctx)
        .expect("bloom false positive must fall through to exact check");
    // CIDs must differ; correct exact-check branch was taken.
    let cid_primed = primed.cid().expect("cid primed");
    assert_ne!(
        cid, cid_primed,
        "bloom false-positive must not silently dedupe the distinct Node"
    );
}

#[test]
fn immutability_cache_warms_on_first_put() {
    let (_dir, backend) = open_backend();
    let node = sample_node("warm");
    let cid = node.cid().expect("cid");

    // Before any write, the cache must not report the CID as known.
    assert!(
        !backend.cache_contains_cid(&cid),
        "bloom/cache should be cold before first put"
    );

    let ctx = WriteContext::new("Post").with_authority(WriteAuthority::User);
    backend.put_node_with_context(&node, &ctx).expect("put");

    assert!(
        backend.cache_contains_cid(&cid),
        "bloom/cache must be warm after first put — otherwise Inv-13 fast-path never fires"
    );
}
