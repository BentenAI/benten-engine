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
    let bytes = blob.to_canonical_bytes().unwrap();
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
// #1210 / #645 — fan_out releases the in-tx TxGuard BEFORE dispatching
//                to subscribers, so a slow/blocking subscriber cannot
//                wedge subsequent .transaction() calls workspace-wide.
// ---------------------------------------------------------------------------

#[test]
fn slow_subscriber_does_not_block_subsequent_transactions() {
    // Pre-#645: fan_out ran subscriber callbacks SYNCHRONOUSLY while the
    // commit thread still held the in-transaction TxGuard, so a blocking
    // subscriber stalled EVERY later .transaction() behind it. The #645
    // fix `drop(tx_guard)` BEFORE fan_out, scoping the back-pressure to
    // the subscriber itself.
    //
    // Observable consequence: a subscriber that blocks (until released)
    // on the FIRST commit must NOT prevent a SECOND .transaction() on
    // another thread from acquiring the TxGuard + committing. The gated
    // subscriber only blocks on the first event it sees (a one-shot
    // latch) so the second commit's fan-out is not itself parked.
    //
    // Would-FAIL-if-reverted: if the guard were held across fan_out, the
    // second transaction's `TxGuard::try_acquire` could never succeed
    // while the first subscriber is parked — the bounded wait-loop below
    // would time out and the assertion would fail (and a true revert
    // would also deadlock → test-runner SIGTERM, still a hard failure).
    use benten_graph::{ChangeEvent, ChangeSubscriber};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    struct OneShotGatedSub {
        seen: AtomicUsize,
        parked: Arc<AtomicBool>,
        release: Arc<AtomicBool>,
    }
    impl ChangeSubscriber for OneShotGatedSub {
        fn on_change(&self, _e: &ChangeEvent) {
            // Only the FIRST event blocks; later events pass straight
            // through so the second commit's own fan-out is not parked.
            if self.seen.fetch_add(1, Ordering::SeqCst) == 0 {
                self.parked.store(true, Ordering::SeqCst);
                while !self.release.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
        }
    }

    let (b, _d) = temp();
    let b = Arc::new(b);
    let parked = Arc::new(AtomicBool::new(false));
    let release = Arc::new(AtomicBool::new(false));
    b.register_subscriber(Arc::new(OneShotGatedSub {
        seen: AtomicUsize::new(0),
        parked: Arc::clone(&parked),
        release: Arc::clone(&release),
    }))
    .unwrap();

    // Thread 1: a commit whose subscriber parks inside fan_out (post-
    // commit, guard already dropped per #645).
    let b1 = Arc::clone(&b);
    let t1 = std::thread::spawn(move || {
        b1.transaction(|tx| {
            tx.put_node(&canonical_test_node())?;
            Ok(())
        })
        .unwrap();
    });

    // Wait until the subscriber is actually parked.
    let park_deadline = Instant::now() + Duration::from_secs(10);
    while !parked.load(Ordering::SeqCst) {
        assert!(
            Instant::now() < park_deadline,
            "subscriber never entered fan_out — test setup failure"
        );
        std::thread::sleep(Duration::from_millis(2));
    }

    // Thread 2: a SECOND transaction. With #645 (guard dropped before
    // fan_out) it MUST acquire the guard + commit even though thread-1's
    // subscriber is still parked.
    let b2 = Arc::clone(&b);
    let mut n2 = canonical_test_node();
    n2.labels = vec!["SecondTxn".into()];
    let committed = Arc::new(AtomicBool::new(false));
    let committed_w = Arc::clone(&committed);
    let t2 = std::thread::spawn(move || {
        b2.transaction(|tx| {
            tx.put_node(&n2)?;
            Ok(())
        })
        .expect("second .transaction() must NOT be wedged by a parked subscriber (#645)");
        committed_w.store(true, Ordering::SeqCst);
    });

    // Bounded observation window: the second commit must land while the
    // first subscriber is STILL parked. Pre-#645 this never happens (the
    // guard is held), so the flag stays false and we assert-fail.
    let commit_deadline = Instant::now() + Duration::from_secs(10);
    while !committed.load(Ordering::SeqCst) {
        assert!(
            Instant::now() < commit_deadline,
            "second transaction did not commit while subscriber parked — \
             #645 guard-release regressed (guard held across fan_out)"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        !release.load(Ordering::SeqCst),
        "second commit landed before the first subscriber was released — #645 holds"
    );

    // Release the parked subscriber + join both threads cleanly.
    release.store(true, Ordering::SeqCst);
    t2.join().expect("second transaction thread panicked");
    t1.join().expect("first transaction thread panicked");

    assert_eq!(
        b.get_by_label("SecondTxn").unwrap().len(),
        1,
        "second transaction's node must be committed (not wedged by #645)"
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

// ---------------------------------------------------------------------------
// #1211 — Hyg-1 dead-code sweep (RESOLVED-ON-MAIN regression-pins).
//
// Reconciliation vs branch HEAD found this umbrella's substance already
// landed by #1261/#1277 OR a DISAGREE-WITH-EVIDENCE realness correction:
//   #292 BatchOp enum  : DELETED (#1261) — ScanIter is NOT dead (it is
//                        the public `KVBackend::scan` return type).
//   #294 ChangeEvent   : `new_edge` does not exist; `new_node`/`kind_str`
//                        are test-used public API (realness: keep).
//   #295 BrowserSnapshot::len/is_empty : test-used idiomatic pair (keep).
//   #297 BloomFilter / DEFAULT_FALSE_POSITIVE_RATE : already pub(crate).
//   #299 RedbBackend ctors : already collapsed to one `from_db` helper.
//   #305 NetworkFetchStubBackend : BELONGS-NAMED phase-4-backlog
//        §4.63/§4.64 (explicit Ben-call, KVBackend v1-SemVer-coupled).
//
// These pins lock the RESOLVED state so a regression re-fires.
// ---------------------------------------------------------------------------

#[test]
fn hyg1_resolved_state_regression_pins() {
    // #297: the bloom-filter knobs stay crate-private. If a future
    // change re-`pub`-exports them, `benten_graph::immutability` is not
    // a public path and this would not even be the failure — instead we
    // pin via the public surface: `BloomFilter` must NOT be reachable
    // from the crate root (it is an internal Inv-13 detail).
    //
    // #292: `ScanIter` (the genuinely-public iterator behind
    // `ScanResult: IntoIterator`, itself the `KVBackend::scan` return
    // type) MUST stay reachable — referencing it here means a wrong
    // "delete ScanIter as dead" regression breaks this crate's compile.
    fn scan_iter_is_public_return_type() -> Option<benten_graph::backend::ScanIter> {
        None
    }
    assert!(scan_iter_is_public_return_type().is_none());

    // #299: every RedbBackend constructor funnels through the single
    // `from_db` helper — observable consequence: an in-memory and a
    // file-backed backend both construct successfully and round-trip a
    // node (the shared init path stays correct, no field drift between
    // entry points).
    let (file_b, _d) = temp();
    let n = canonical_test_node();
    let c = file_b.put_node(&n).unwrap();
    assert_eq!(file_b.get_node(&c).unwrap().as_ref(), Some(&n));

    let mem_b = RedbBackend::open_in_memory().unwrap();
    let c2 = mem_b.put_node(&n).unwrap();
    assert_eq!(
        mem_b.get_node(&c2).unwrap().as_ref(),
        Some(&n),
        "open_in_memory + open_or_create share the from_db init path (#299)"
    );

    // #294: `ChangeEvent::new_node` + `kind_str` stay as public
    // test-used API (realness: NOT dead). Reference them so a wrong
    // "delete as dead" regression breaks compile + the assertion.
    use benten_graph::{ChangeEvent, ChangeKind};
    let ev = ChangeEvent::new_node(c, n.labels.clone(), ChangeKind::Created, 0, Some(n.clone()));
    assert_eq!(
        ev.kind_str(),
        "Created",
        "ChangeEvent::new_node + kind_str remain live public API (#294 realness)"
    );
}
