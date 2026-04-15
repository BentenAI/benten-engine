//! Edge-case test: scan that matches zero keys must return an empty iterator,
//! NOT error and NOT panic.
//!
//! This is the pure "the API honestly said no" surface: the caller supplied
//! a well-formed prefix, the backend searched, and found nothing. That must
//! not be an error — it's information.
//!
//! Complements the happy-path scan iterator tests owned by rust-test-writer-unit.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_graph::{KVBackend, RedbBackend};
use tempfile::tempdir;

fn fresh_backend() -> (RedbBackend, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");
    let b = RedbBackend::open_or_create(&db_path).unwrap();
    (b, dir)
}

#[test]
fn scan_on_empty_store_returns_empty_iterator() {
    let (backend, _dir) = fresh_backend();

    // Even an empty prefix on an empty store must yield an empty iterator.
    let hits = backend.scan(&[]).unwrap();
    assert!(
        hits.is_empty(),
        "empty store + empty prefix must yield nothing"
    );

    // Same for a non-empty prefix.
    let hits = backend.scan(b"anything").unwrap();
    assert!(
        hits.is_empty(),
        "empty store + any prefix must yield nothing"
    );
}

#[test]
fn scan_with_no_matches_on_populated_store_returns_empty() {
    let (backend, _dir) = fresh_backend();

    backend.put(b"alpha", b"1").unwrap();
    backend.put(b"beta", b"2").unwrap();
    backend.put(b"gamma", b"3").unwrap();

    // Prefix sorts strictly after every stored key.
    let hits = backend.scan(b"zzz").unwrap();
    assert!(hits.is_empty(), "prefix after every key must yield nothing");

    // Prefix that cannot be a prefix of any stored key even though it sorts
    // within range.
    let hits = backend.scan(b"delta").unwrap();
    assert!(hits.is_empty(), "prefix matching no key must yield nothing");
}

#[test]
fn scan_returns_consistent_empty_shape() {
    let (backend, _dir) = fresh_backend();

    // Inserting after the miss does not retroactively populate the
    // previous result. The "no hit" is a point-in-time answer.
    let empty = backend.scan(b"prefix").unwrap();
    assert!(empty.is_empty());

    backend.put(b"prefix:1", b"value").unwrap();
    let after = backend.scan(b"prefix").unwrap();
    assert_eq!(after.len(), 1, "post-insert scan sees the new key");

    // The original `empty` result is still the original empty — iterators
    // are snapshots or lazy-but-consistent, not mutable views into the store.
    // (If this ever fails it indicates scan returned a live iterator that
    // would race with concurrent writers — a P2+ concern, but we pin the
    // shape here so R5 cannot accidentally regress to a live-view iterator.)
    assert!(empty.is_empty());
}

#[test]
fn scan_single_byte_prefix_matching_zero_keys() {
    let (backend, _dir) = fresh_backend();

    // Populate only keys starting with 'a'.
    backend.put(b"alpha", b"1").unwrap();
    backend.put(b"apple", b"2").unwrap();

    // Prefix 'z' matches nothing. Must return empty, not error.
    let hits = backend.scan(&[b'z']).unwrap();
    assert!(hits.is_empty());

    // Prefix 0x00 matches nothing (no stored key starts with NUL).
    let hits = backend.scan(&[0x00]).unwrap();
    assert!(hits.is_empty());

    // Prefix 0xff matches nothing (no stored key starts with 0xff).
    let hits = backend.scan(&[0xff]).unwrap();
    assert!(hits.is_empty());
}
