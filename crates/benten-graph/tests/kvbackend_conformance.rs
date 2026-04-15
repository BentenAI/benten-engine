//! `KVBackend::get / put / delete / scan / put_batch` conformance suite
//! (R2 landscape §2.2 row 1).
//!
//! Runs against the `RedbBackend` in Phase 1. When an in-memory mock impl
//! lands (G2-A), the identical assertions re-run against it to prove the
//! trait contract holds uniformly.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_graph::{KVBackend, RedbBackend};
use tempfile::TempDir;

fn temp_backend() -> (RedbBackend, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("t.redb")).unwrap();
    (backend, dir)
}

#[test]
fn get_on_missing_key_returns_none() {
    let (b, _d) = temp_backend();
    assert_eq!(b.get(b"missing").unwrap(), None);
}

#[test]
fn put_then_get_returns_same_bytes() {
    let (b, _d) = temp_backend();
    b.put(b"k", b"v").unwrap();
    assert_eq!(b.get(b"k").unwrap(), Some(b"v".to_vec()));
}

#[test]
fn delete_removes_key() {
    let (b, _d) = temp_backend();
    b.put(b"k", b"v").unwrap();
    b.delete(b"k").unwrap();
    assert_eq!(b.get(b"k").unwrap(), None);
}

#[test]
fn scan_returns_matching_prefix_only() {
    let (b, _d) = temp_backend();
    b.put(b"post:1", b"p1").unwrap();
    b.put(b"post:2", b"p2").unwrap();
    b.put(b"user:1", b"u1").unwrap();

    let posts = b.scan(b"post:").unwrap();
    assert_eq!(posts.len(), 2);
    assert!(posts.iter().all(|(k, _)| k.starts_with(b"post:")));
}

#[test]
fn put_batch_commits_every_pair() {
    let (b, _d) = temp_backend();
    let pairs = vec![
        (b"a".to_vec(), b"1".to_vec()),
        (b"b".to_vec(), b"2".to_vec()),
        (b"c".to_vec(), b"3".to_vec()),
    ];
    b.put_batch(&pairs).unwrap();
    assert_eq!(b.get(b"a").unwrap(), Some(b"1".to_vec()));
    assert_eq!(b.get(b"b").unwrap(), Some(b"2".to_vec()));
    assert_eq!(b.get(b"c").unwrap(), Some(b"3".to_vec()));
}
