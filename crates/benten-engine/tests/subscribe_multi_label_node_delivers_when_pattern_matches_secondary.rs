//! R6FP-Group-1 (Round-2 Instance 6 BLOCKER) regression pin —
//! a multi-labeled Node delivers to a SUBSCRIBE consumer matching ANY
//! of its labels, not only the primary.
//!
//! Pre-fix, the bridge at `crates/benten-engine/src/builder.rs` collapsed
//! `event.labels: Vec<String>` to a single `primary_label: String` when
//! translating graph::ChangeEvent → eval::ChangeEvent. The matcher at
//! `subscribe.rs::publish_change_event_with_label` then consulted only
//! that one label. A multi-labeled Node `["User","Admin"]` silently
//! missed delivery to a SUBSCRIBE consumer matching `Admin:*` because
//! `primary_label = "User"` was the only label tested. This was a
//! BEHAVIORAL DEFECT, not just an observability gap — the consumer
//! never saw the change, with no error or warning.
//!
//! R6FP-G1 widens the eval-side ChangeEvent to carry `labels:
//! Vec<String>`, the bridge forwards every label, and
//! `publish_change_event_with_labels` walks each label to fire when ANY
//! one matches. This test pins the cross-layer behavioural contract:
//! write a Node with labels `["User","Admin"]`, subscribe to `Admin:*`,
//! assert the callback fires.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use benten_core::{Node, Value};
use benten_engine::{Engine, SubscribeCursor};

#[test]
fn subscribe_multi_label_node_delivers_when_pattern_matches_secondary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("engine.redb")).expect("open engine");

    // Subscribe to Admin:* — the SECONDARY label of the Node we're about
    // to write. Pre-R6FP-G1 the bridge collapsed labels to the primary
    // ("User") and this subscriber would silently miss every event.
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_for_cb = Arc::clone(&hits);
    let cb: Arc<dyn Fn(u64, &benten_engine::Chunk) + Send + Sync + 'static> =
        Arc::new(move |_seq, _chunk| {
            hits_for_cb.fetch_add(1, Ordering::SeqCst);
        });
    // Pattern "Admin*" — glob-form matching the literal label "Admin"
    // (and any longer label starting with "Admin"). Pre-R6FP-G1 the
    // matcher only consulted the primary label; the secondary "Admin"
    // would silently miss.
    let _sub = engine
        .on_change_with_cursor("Admin*", SubscribeCursor::Latest, cb)
        .expect("on_change registers");

    // Write a multi-labeled Node. The first label ("User") would have
    // been the pre-fix primary_label; "Admin" is the secondary label
    // the subscriber matches.
    let mut props = std::collections::BTreeMap::new();
    props.insert("name".to_string(), Value::text("alice"));
    let node = Node::new(vec!["User".to_string(), "Admin".to_string()], props);

    // Drive the write through the engine's transaction surface so the
    // ChangeEvent fires on commit. (Engine::put_node is not yet wired
    // beyond G2-A; transaction is the existing user-write path.)
    engine
        .transaction(|tx| {
            tx.put_node(&node)
                .map_err(|e| benten_engine::EngineError::Other {
                    code: benten_errors::ErrorCode::Unknown("E_TEST_HARNESS".into()),
                    message: format!("put_node: {e:?}"),
                })?;
            Ok(())
        })
        .expect("commit multi-labeled Node");

    // Yield briefly so any same-thread ChangeBroadcast dispatch
    // completes (publish path is synchronous on the committing thread,
    // but the ThreadsafeFunction enqueue is libuv-async; this test
    // exercises the engine-side OnChangeCallback Arc directly so the
    // callback fires synchronously on commit).
    std::thread::sleep(Duration::from_millis(50));

    assert!(
        hits.load(Ordering::SeqCst) >= 1,
        "subscribe(\"Admin:*\") MUST fire for a Node with labels \
         [\"User\",\"Admin\"] — pre-R6FP-G1 (Round-2 Instance 6 BLOCKER) \
         the graph→eval bridge collapsed labels: Vec<String> to a single \
         primary_label: String, so only the FIRST label (\"User\") was \
         matched, and a multi-labeled Node silently missed delivery to \
         consumers matching any non-primary label"
    );
}
