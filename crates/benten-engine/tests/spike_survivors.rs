//! Spike-survivor tests (R2 landscape §2.6 row 1 — `Engine::open /
//! create_node / get_node`).
//!
//! These are the spike's two tests lifted out to an integration file so they
//! survive the N4/N5/N6 reshape. They test the public API, not implementation
//! details.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;

#[test]
fn create_then_get_roundtrip_survives_spike_reshape() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let node = canonical_test_node();
    let cid = engine.create_node(&node).unwrap();
    let fetched = engine.get_node(&cid).unwrap().expect("node exists");
    assert_eq!(fetched, node);
    assert_eq!(fetched.cid().unwrap(), cid);
}

#[test]
fn missing_cid_returns_none_survives_spike_reshape() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let cid = canonical_test_node().cid().unwrap();
    assert!(engine.get_node(&cid).unwrap().is_none());
}

#[test]
fn create_node_is_idempotent_for_identical_content() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let node = canonical_test_node();
    let c1 = engine.create_node(&node).unwrap();
    let c2 = engine.create_node(&node).unwrap();
    // Content-addressed: same content → same CID. Idempotent upsert.
    assert_eq!(c1, c2);
}
