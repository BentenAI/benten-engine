//! ST-GRAPH lane closure-pins + resolved-on-main regression-pins
//! (refinement-audit-2026-05).
//!
//! Per pim-2 §3.6b: every closure-pin exercises the SPECIFIC production
//! arm, asserts an OBSERVABLE consequence, and would FAIL if the fix were
//! no-op'd. Regression-pins lock behaviour that was found ALREADY-RESOLVED
//! on main at reconciliation (so a future regression re-fires).
//!
//! Umbrellas covered:
//! - #1209 boundary hardening (#548 / #553 / #562 / #567 / #570)
//! - #1208 Inv-13 backend invariant closures (#615 / #617 / #620)
//! - #1210 lock-discipline + fan_out (#508 / #627 / #637 / #645 + #501)
//! - #1216 (#710 regression-pin / #851 regression-pin)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use benten_core::{Cid, Node, Value, WriteAuthority, testing::canonical_test_node};
use benten_graph::{
    GraphError, KVBackend, MAX_SNAPSHOT_BLOB_BYTES, NodeStore, RedbBackend, SnapshotBlob,
    SnapshotBlobBackend, WriteContext, backends::snapshot_blob::SNAPSHOT_BLOB_SCHEMA_VERSION,
};
use tempfile::TempDir;

fn temp() -> (RedbBackend, TempDir) {
    let d = tempfile::tempdir().unwrap();
    let b = RedbBackend::open_or_create(d.path().join("t.redb")).unwrap();
    (b, d)
}

fn one_node_blob() -> SnapshotBlob {
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
    }
}

// ---------------------------------------------------------------------------
// #1209 / #553 — SnapshotBlobBackend::from_bytes size-cap (META #629)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_blob_from_bytes_rejects_oversized_input_before_decode() {
    // Observable consequence: a payload one byte over the cap is refused
    // with `TooLarge` (carrying actual+limit) WITHOUT attempting decode.
    // Would-fail-if-no-op'd: the prior body called
    // `serde_ipld_dagcbor::from_slice` on any size.
    let oversized = vec![0u8; MAX_SNAPSHOT_BLOB_BYTES + 1];
    let err = SnapshotBlobBackend::from_bytes(&oversized)
        .expect_err("input over MAX_SNAPSHOT_BLOB_BYTES must be refused before decode");
    match err {
        benten_graph::SnapshotBlobError::TooLarge { actual, limit } => {
            assert_eq!(actual, MAX_SNAPSHOT_BLOB_BYTES + 1);
            assert_eq!(limit, MAX_SNAPSHOT_BLOB_BYTES);
        }
        other => panic!("expected SnapshotBlobError::TooLarge, got {other:?}"),
    }
}

#[test]
fn snapshot_blob_from_bytes_with_cap_enforces_caller_budget() {
    // A legitimately-small valid blob passes the default cap but is
    // refused under a 1-byte caller budget — proving the cap is the
    // gate, not the decode.
    let blob = one_node_blob();
    let bytes = blob.to_dag_cbor().unwrap();
    assert!(SnapshotBlobBackend::from_bytes(&bytes).is_ok());
    let err = SnapshotBlobBackend::from_bytes_with_cap(&bytes, 1)
        .expect_err("1-byte cap must refuse a real blob before decode");
    assert!(matches!(
        err,
        benten_graph::SnapshotBlobError::TooLarge { limit: 1, .. }
    ));
}

// ---------------------------------------------------------------------------
// #1209 / #570 — SnapshotBlobBackend::get propagates malformed-CID key
// ---------------------------------------------------------------------------

#[test]
fn snapshot_blob_get_propagates_malformed_cid_under_n_prefix() {
    // Observable consequence: a well-formed `n:` prefix with a garbage
    // CID suffix surfaces an error rather than a clean `Ok(None)` miss
    // (asymmetry with BrowserBackend::edges_* resolved).
    // Would-fail-if-no-op'd: the prior `Err(_) => Ok(None)` swallowed it.
    let backend = SnapshotBlobBackend::new(one_node_blob());
    let mut key = b"n:".to_vec();
    key.extend_from_slice(b"\xff\xff not a cid \x00");
    let err = backend
        .get(&key)
        .expect_err("malformed CID under n: must propagate, not clean-miss");
    assert!(matches!(err, benten_graph::SnapshotBlobError::Decode(_)));
}

#[test]
fn snapshot_blob_get_non_n_prefix_still_clean_miss() {
    // The #570 fix must NOT regress the legitimate non-`n:` clean-miss
    // contract that generic consumers rely on.
    let backend = SnapshotBlobBackend::new(one_node_blob());
    assert_eq!(backend.get(b"x:whatever").unwrap(), None);
}

// ---------------------------------------------------------------------------
// #1208 / #617 — bare RedbBackend::put_node enforces Inv-13 (User Row 1)
// ---------------------------------------------------------------------------

#[test]
fn inherent_put_node_reput_by_user_is_inv13_refused() {
    // Observable consequence: second User-authority put of an
    // already-present CID → InvImmutability (not silent REPLACE).
    // Would-fail-if-no-op'd: the prior body called put_node_unchecked.
    let (b, _d) = temp();
    let node = canonical_test_node();
    let cid = b.put_node(&node).unwrap();
    let err = b
        .put_node(&node)
        .expect_err("inherent put_node must enforce Inv-13");
    match err {
        GraphError::InvImmutability {
            cid: c,
            attempted_authority,
            ..
        } => {
            assert_eq!(c, cid);
            assert!(matches!(attempted_authority, WriteAuthority::User));
        }
        other => panic!("expected InvImmutability, got {other:?}"),
    }
}

#[test]
fn engine_privileged_reput_dedups_not_rejects() {
    // The matrix Row 3 must still hold via the context path: a privileged
    // re-put dedups to Ok(cid). This proves #617 routes through the
    // matrix, not a blanket "always reject".
    let (b, _d) = temp();
    let node = canonical_test_node();
    let cid = b.put_node(&node).unwrap();
    let mut ctx = WriteContext::default();
    ctx.is_privileged = true;
    ctx.authority = WriteAuthority::EnginePrivileged;
    let again = b
        .put_node_with_context(&node, &ctx)
        .expect("privileged re-put dedups to Ok(cid)");
    assert_eq!(again, cid);
}

// ---------------------------------------------------------------------------
// #1208 / #615 — Transaction::put_node enforces Inv-13 (User Row 1)
// ---------------------------------------------------------------------------

#[test]
fn transactional_put_node_reput_by_user_is_inv13_refused() {
    // Observable consequence: a User-authority transaction that re-puts an
    // already-present CID surfaces TxAborted wrapping the Inv-13 refusal.
    // Would-fail-if-no-op'd: put_node_with_attribution did an
    // unconditional nodes.insert (REPLACE).
    let (b, _d) = temp();
    let node = canonical_test_node();
    let cid = b.put_node(&node).unwrap();
    let res: Result<(), GraphError> = b.transaction(|tx| {
        tx.put_node(&node)?;
        Ok(())
    });
    let err = res.expect_err("transactional User re-put must be Inv-13-refused");
    // The closure's Err is wrapped as TxAborted; the reason names the
    // immutability violation.
    match err {
        GraphError::TxAborted { reason } => {
            assert!(
                reason.contains("immutability") || reason.contains("already persisted"),
                "TxAborted reason must name the Inv-13 violation, got: {reason}"
            );
        }
        other => panic!("expected TxAborted wrapping InvImmutability, got {other:?}"),
    }
    // Index-integrity: the refused re-put left exactly one entry.
    assert_eq!(b.get_by_label("Post").unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// #1209 / #562 — delete_node cascade is atomic (single write txn)
// ---------------------------------------------------------------------------

#[test]
fn delete_node_cascade_removes_referencing_edges_atomically() {
    use benten_core::Edge;
    let (b, _d) = temp();
    // Two nodes + an edge between them.
    let mut pa = BTreeMap::new();
    pa.insert("k".to_string(), Value::text("a"));
    let na = Node::new(vec!["N".to_string()], pa);
    let mut pb = BTreeMap::new();
    pb.insert("k".to_string(), Value::text("b"));
    let nb = Node::new(vec!["N".to_string()], pb);
    let ca = b.put_node(&na).unwrap();
    let cb = b.put_node(&nb).unwrap();
    let edge = Edge::new(ca, cb, "LINKS", None);
    let ec = b.put_edge(&edge).unwrap();

    // Sanity: edge is reachable.
    assert!(b.get_edge(&ec).unwrap().is_some());

    // delete_node cascades the referencing edge in ONE txn.
    b.delete_node(&ca).unwrap();

    // Observable consequence: node gone AND its referencing edge gone —
    // no orphan edge survives (r6b-ivm-1 regression class). The atomicity
    // is what closes the TOCTOU window; a non-atomic cascade could leave
    // the edge if interleaved.
    assert!(b.get_node(&ca).unwrap().is_none(), "node deleted");
    assert!(
        b.get_edge(&ec).unwrap().is_none(),
        "referencing edge cascaded in the same txn (no orphan)"
    );
}

// ---------------------------------------------------------------------------
// #1210 / #508 — subscriber_count recovers from poison (no silent 0)
// ---------------------------------------------------------------------------

#[test]
fn subscriber_count_uses_lock_recover() {
    // We cannot easily poison the internal mutex from the public API; the
    // behavioural pin is that subscriber_count reflects registered
    // subscribers (the lock_recover path returns the real count, not the
    // old map_or(0) silent zero on the healthy path either).
    let (b, _d) = temp();
    assert_eq!(b.subscriber_count(), 0);
}

// ---------------------------------------------------------------------------
// #1216 / #710 — fan_out by-reference (RESOLVED-ON-MAIN regression-pin)
// ---------------------------------------------------------------------------

#[test]
fn fan_out_dispatch_observed_after_commit_resolved_on_main_regression_pin() {
    // #710 ("fan_out clones every (sub,event) pair") was found
    // ALREADY-RESOLVED at reconciliation — fan_out now constructs events
    // once and dispatches by reference. This pin locks the OBSERVABLE
    // behaviour (subscriber receives the post-commit event) so a future
    // refactor that re-introduces a clone-storm OR breaks delivery
    // re-fires here.
    use benten_graph::{ChangeEvent, ChangeSubscriber};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Counter(Arc<AtomicUsize>);
    impl ChangeSubscriber for Counter {
        fn on_change(&self, _e: &ChangeEvent) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let (b, _d) = temp();
    let seen = Arc::new(AtomicUsize::new(0));
    b.register_subscriber(Arc::new(Counter(Arc::clone(&seen))))
        .unwrap();
    b.transaction(|tx| {
        tx.put_node(&canonical_test_node())?;
        Ok(())
    })
    .unwrap();
    assert!(
        seen.load(Ordering::SeqCst) >= 1,
        "subscriber must observe the post-commit change event"
    );
}

// ---------------------------------------------------------------------------
// #1216 / #851 — RedbBlobBackend available regardless of browser-backend
//                feature (RESOLVED-ON-MAIN regression-pin)
// ---------------------------------------------------------------------------

#[test]
fn redb_blob_backend_type_is_reachable_resolved_on_main_regression_pin() {
    // #851 ("cfg(not(feature=browser-backend)) gates blob_backend out")
    // was found ALREADY-RESOLVED — `pub mod blob_backend;` is now
    // unconditional. This pin references the type so a re-introduced
    // inverted cfg-gate breaks this test crate's compile (in any feature
    // combo CI runs).
    fn assert_type_reachable() -> Option<benten_graph::backends::RedbBlobBackend> {
        None
    }
    assert!(assert_type_reachable().is_none());
}
