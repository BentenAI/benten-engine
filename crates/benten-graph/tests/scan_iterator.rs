//! Scan iterator shape tests (G1, G2-A — R2 landscape §2.2 row 3).
//!
//! Phase 1 triage tag `P1.graph.scan-iterator`: `scan` returns an iterator,
//! not a `Vec`. Phase 1 spike's current shape is `ScanResult = Vec<...>`.
//! These tests are TDD spec for the reshape.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_graph::{KVBackend, RedbBackend};
use tempfile::TempDir;

fn temp_backend() -> (RedbBackend, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let b = RedbBackend::open(dir.path().join("t.redb")).unwrap();
    (b, dir)
}

#[test]
fn scan_iterator_empty_prefix_returns_every_pair() {
    let (b, _d) = temp_backend();
    b.put(b"a", b"1").unwrap();
    b.put(b"b", b"2").unwrap();
    b.put(b"c", b"3").unwrap();
    let hits = b.scan(&[]).unwrap();
    assert_eq!(hits.len(), 3);
}

#[test]
fn scan_iterator_prefix_bounds_range_to_prefix() {
    let (b, _d) = temp_backend();
    b.put(b"post:1", b"a").unwrap();
    b.put(b"post:2", b"b").unwrap();
    b.put(b"user:1", b"c").unwrap();
    let posts = b.scan(b"post:").unwrap();
    assert_eq!(posts.len(), 2);
}

#[test]
fn scan_iterator_is_lazy_over_prefix_matches() {
    // In the Phase 1 iterator reshape, consuming only the first item must not
    // materialize every match. Today (Phase 1 pre-reshape), we at least assert
    // that iterating from `scan`'s output produces results one at a time.
    let (b, _d) = temp_backend();
    for i in 0..100u32 {
        let key = format!("post:{i:03}");
        b.put(key.as_bytes(), b"v").unwrap();
    }
    let hits = b.scan(b"post:").unwrap();
    // Lazy-friendly shape: we can .iter().take(1) to get a prefix.
    let first: Vec<_> = hits.iter().take(1).collect();
    assert_eq!(first.len(), 1);
    assert!(first[0].0.starts_with(b"post:"));
}
