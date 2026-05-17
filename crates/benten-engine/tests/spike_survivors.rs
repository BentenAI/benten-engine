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
use benten_engine::{Engine, EngineError};

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
fn create_node_identical_content_second_put_is_inv13_refused() {
    // refinement-audit-2026-05 #615/#617 (ST-GRAPH lane, umbrella #1208,
    // META #660 Inv-13 5-row matrix) — cross-lane contract propagation.
    //
    // **Original contract (now corrected):** this test previously asserted
    // that a second `Engine::create_node` of identical content was a silent
    // idempotent upsert returning the same CID (`c1 == c2`). That relied on
    // the bare `put_node` REPLACE-on-collision bypass.
    //
    // **Inv-13 bypass #615/#617 closed:** `Engine::create_node` routes
    // `backend.transaction(|tx| tx.put_node(node))` → `Transaction::put_node`,
    // which now runs the in-txn NODES_TABLE existence probe + 5-row authority
    // dispatch. For `WriteAuthority::User` + an already-present CID this is an
    // Inv-13 Row-1 immutability violation (`GraphError::InvImmutability`),
    // surfaced at the `Engine` boundary as the transaction-wrapped
    // `EngineError::Graph(GraphError::TxAborted { reason })`. This mirrors the
    // graph-lane's own retarget of `indexes_idempotent_put_node_twice`
    // (`put_node_twice_same_cid_rejects_second_...`) at the integration-test
    // layer the disjoint single-crate review could not reach (§3.5l
    // cross-crate-consumer class). NOT a coverage weakening: the load-bearing
    // content-addressing property (the violation names the colliding CID;
    // first write succeeded) is still pinned.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let node = canonical_test_node();
    let c1 = engine.create_node(&node).unwrap();

    let err = engine.create_node(&node).expect_err(
        "second User-authority create_node of an already-present CID must be \
         refused by Inv-13 (Row 1), not silently REPLACE",
    );
    match err {
        EngineError::Graph(g) => {
            let reason = g.to_string();
            assert!(
                reason.contains("immutability violation")
                    && reason.contains(&c1.to_string())
                    && reason.contains("attempted_authority: User"),
                "expected Inv-13 Row-1 immutability violation naming the \
                 colliding CID {c1} under User authority, got: {reason}",
            );
        }
        other => panic!("expected EngineError::Graph (Inv-13 Row-1), got {other:?}"),
    }

    // Content-addressing property survives the contract correction: the
    // first write persisted under the canonical CID and is still readable.
    let fetched = engine.get_node(&c1).unwrap().expect("first write persisted");
    assert_eq!(fetched, node);
}
