//! Phase 1 R3 integration — cross-process determinism for the graph layer.
//!
//! Rewritten at R4 triage (M3) to spawn an actual child PID via
//! `std::process::Command` invoking the `write-canonical-and-exit` bin under
//! `CARGO_BIN_EXE_*`. This replaces the previous "drop-and-reopen within one
//! test" simulation, which did not span processes.
//!
//! The parent test:
//!   1. Creates a tempdir and resolves the child's db path.
//!   2. Spawns the child with the path as argv[1].
//!   3. Asserts the child exits 0 and prints the canonical CID.
//!   4. Opens the db in the parent (a distinct process) and re-hashes the
//!      stored Node. Rehash must match the fixture CID — proves persisted-CID
//!      round-trip across real PIDs.
//!
//! **Status:** FAILING only if the G3 reshape breaks persisted-Node CID
//! round-trip; PASSING against the spike shape today.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;
use std::process::Command;

const CANONICAL_CID: &str = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";

#[test]
fn persisted_cid_survives_actual_child_process() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // "Process A": real subprocess, not just a scope in this test.
    let bin = env!("CARGO_BIN_EXE_write-canonical-and-exit");
    let output = Command::new(bin)
        .arg(&db_path)
        .output()
        .expect("spawn write-canonical-and-exit");
    assert!(
        output.status.success(),
        "child must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let child_cid = String::from_utf8(output.stdout)
        .expect("child stdout utf-8")
        .trim()
        .to_string();
    assert_eq!(
        child_cid, CANONICAL_CID,
        "child process wrote the canonical fixture CID"
    );

    // "Process B" (this test's PID): reopen, retrieve, re-hash.
    let engine = Engine::builder().path(&db_path).build().unwrap();
    let expected_cid = canonical_test_node().cid().unwrap();
    let fetched = engine
        .get_node(&expected_cid)
        .unwrap()
        .expect("persisted node retrievable after child exits");
    let rehashed = fetched.cid().unwrap();

    assert_eq!(
        rehashed.to_base32(),
        CANONICAL_CID,
        "persisted Node re-hashes to the same CID across real PIDs"
    );
    assert_eq!(rehashed, expected_cid, "round-trip CID equality");
}

#[test]
fn cross_process_graph_cid_matches_fixture_on_write() {
    // Protects the D2 fixture reach through the storage layer (not just bare
    // `Node::cid()`) within a single process. Complements the real-subprocess
    // test above.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let cid = engine.create_node(&canonical_test_node()).unwrap();
    assert_eq!(cid.to_base32(), CANONICAL_CID);
}
