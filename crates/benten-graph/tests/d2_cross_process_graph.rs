//! Phase 1 R3 integration — cross-process determinism for the graph layer.
//!
//! Extension of `crates/benten-core/tests/d2_cross_process.rs` from bare Node
//! hashing to persisted-CID round-trip: write the canonical node in process A
//! (simulated via a child redb file), then open the same file in process B
//! (a fresh Engine handle) and assert the retrieved Node re-hashes to the
//! spike fixture CID.
//!
//! Because Rust integration tests each compile to their own binary, the
//! "two processes" property is modeled by: create engine handle 1, write,
//! drop it (flushing redb + closing the file), then open engine handle 2.
//! `cargo nextest run --no-fail-fast` additionally isolates each test in a
//! separate process by default, so this file exercises both the in-process
//! drop-and-reopen and the nextest-forked-process path.
//!
//! **Status:** FAILING only if the G3 reshape breaks persisted-Node CID
//! round-trip; PASSING against the spike shape today.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;

const CANONICAL_CID: &str = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";

#[test]
fn persisted_cid_survives_drop_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // "Process A": write the canonical Node.
    let written_cid = {
        let engine = Engine::builder().path(&db_path).build().unwrap();
        engine.create_node(&canonical_test_node()).unwrap()
    }; // engine drops, file flushes

    assert_eq!(
        written_cid.to_base32(),
        CANONICAL_CID,
        "CID matches spike fixture on write"
    );

    // "Process B": reopen, retrieve, re-hash.
    let engine = Engine::builder().path(&db_path).build().unwrap();
    let fetched = engine
        .get_node(&written_cid)
        .unwrap()
        .expect("persisted node retrievable after reopen");
    let rehashed = fetched.cid().unwrap();

    assert_eq!(
        rehashed.to_base32(),
        CANONICAL_CID,
        "persisted Node re-hashes to the same CID across reopen"
    );
    assert_eq!(
        rehashed, written_cid,
        "round-trip CID equality across reopen"
    );
}

#[test]
fn cross_process_graph_cid_matches_fixture_on_write() {
    // Protects the D2 fixture reach through the storage layer (not just bare
    // `Node::cid()`). If anyone regresses the canonicalization on the graph
    // write-path (e.g., by re-encoding through a different serializer), this
    // test will catch it before the D2 hashing test would.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let cid = engine.create_node(&canonical_test_node()).unwrap();
    assert_eq!(cid.to_base32(), CANONICAL_CID);
}
