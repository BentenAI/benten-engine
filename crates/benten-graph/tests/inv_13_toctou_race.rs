//! G11-A TOCTOU atomicity regression: `put_node_with_context` folds the
//! existence probe and conditional write into a single redb write
//! transaction so two concurrent writers cannot both pass a pre-txn probe
//! and both "first-put" the same CID.
//!
//! Closes the G2-A User-path race window (two User-authority writes racing
//! on the same CID) and the G5-A Row-3 dedup race window (two
//! EnginePrivileged dedup calls racing on the same CID) in one fix.
//!
//! ## Concerns pinned
//!
//! - Under N concurrent User-authority writers attempting `put_node_with_context`
//!   on the same CID, exactly ONE returns `Ok(cid)` and every other call
//!   returns `Err(GraphError::InvImmutability)`. The write-once contract
//!   cannot silently produce N Oks.
//! - Under N concurrent EnginePrivileged writers on the same CID, every
//!   call returns `Ok(cid)` but only the first advances the audit
//!   sequence (observable via the test-only change-event log, which the
//!   dedup path MUST NOT append to).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use benten_core::{Node, Value};
use benten_graph::{GraphError, RedbBackend, WriteAuthority, WriteContext};
use std::collections::BTreeMap;
use tempfile::tempdir;

fn target_node() -> Node {
    let mut props = BTreeMap::new();
    props.insert("k".to_string(), Value::Int(42));
    Node::new(vec!["race-target".to_string()], props)
}

#[test]
fn inv_13_concurrent_user_writes_race_still_rejects() {
    let dir = tempdir().unwrap();
    let backend = Arc::new(RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap());

    let node = target_node();
    let ctx = WriteContext {
        label: "race-target".to_string(),
        is_privileged: false,
        authority: WriteAuthority::User,
    };

    const N: usize = 8;
    let success_count = Arc::new(AtomicUsize::new(0));
    let rejection_count = Arc::new(AtomicUsize::new(0));

    let started = Arc::new(AtomicUsize::new(0));
    thread::scope(|s| {
        let handles: Vec<_> = (0..N)
            .map(|_| {
                let backend = Arc::clone(&backend);
                let node = node.clone();
                let ctx = ctx.clone();
                let success_count = Arc::clone(&success_count);
                let rejection_count = Arc::clone(&rejection_count);
                let started = Arc::clone(&started);
                s.spawn(move || {
                    started.fetch_add(1, Ordering::SeqCst);
                    while started.load(Ordering::SeqCst) < N {
                        std::hint::spin_loop();
                    }
                    match backend.put_node_with_context(&node, &ctx) {
                        Ok(_) => {
                            success_count.fetch_add(1, Ordering::SeqCst);
                        }
                        Err(GraphError::InvImmutability { .. }) => {
                            rejection_count.fetch_add(1, Ordering::SeqCst);
                        }
                        Err(other) => panic!("unexpected error: {other:?}"),
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    });

    let s = success_count.load(Ordering::SeqCst);
    let r = rejection_count.load(Ordering::SeqCst);
    assert_eq!(
        s, 1,
        "exactly one User-authority write may succeed; got {s} successes + {r} rejections"
    );
    assert_eq!(
        r,
        N - 1,
        "every other User-authority write must fire E_INV_IMMUTABILITY; got {s} successes + {r} rejections"
    );
}

#[test]
fn inv_13_concurrent_privileged_dedup_no_audit_advance() {
    let dir = tempdir().unwrap();
    let backend = Arc::new(RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap());

    let node = target_node();
    let ctx = WriteContext {
        label: "race-target".to_string(),
        is_privileged: true,
        authority: WriteAuthority::EnginePrivileged,
    };

    const N: usize = 8;
    let success_count = Arc::new(AtomicUsize::new(0));

    let started = Arc::new(AtomicUsize::new(0));
    thread::scope(|s| {
        let handles: Vec<_> = (0..N)
            .map(|_| {
                let backend = Arc::clone(&backend);
                let node = node.clone();
                let ctx = ctx.clone();
                let success_count = Arc::clone(&success_count);
                let started = Arc::clone(&started);
                s.spawn(move || {
                    started.fetch_add(1, Ordering::SeqCst);
                    while started.load(Ordering::SeqCst) < N {
                        std::hint::spin_loop();
                    }
                    backend.put_node_with_context(&node, &ctx).unwrap();
                    success_count.fetch_add(1, Ordering::SeqCst);
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    });

    // Every EnginePrivileged call returns Ok (dedup or first-put).
    assert_eq!(
        success_count.load(Ordering::SeqCst),
        N,
        "every EnginePrivileged call must return Ok — first-put or dedup"
    );

    // The test-only ChangeEvent buffer must record EXACTLY ONE event —
    // the dedup path must NOT push a ChangeEvent or advance the audit
    // sequence. The race-losers all short-circuited through the dedup branch.
    let events = backend.drain_change_events_for_test();
    assert_eq!(
        events.len(),
        1,
        "exactly one ChangeEvent must be emitted across {N} concurrent \
         privileged first-put races; got {} events",
        events.len()
    );
}
