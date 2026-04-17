//! `ChangeSubscriber` trait shape (R1 architect major #1 — R2 landscape
//! §2.2 row 10).
//!
//! Decouples `benten-graph` from any async runtime. The trait accepts both
//! sync-callback and async-broadcast impls. Here we only assert the trait
//! object-safety + the shape.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind, ChangeSubscriber};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingSubscriber {
    count: Arc<AtomicUsize>,
}

impl ChangeSubscriber for CountingSubscriber {
    fn on_change(&self, _event: &ChangeEvent) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn change_subscriber_is_object_safe() {
    let count = Arc::new(AtomicUsize::new(0));
    let sub: Box<dyn ChangeSubscriber> = Box::new(CountingSubscriber {
        count: count.clone(),
    });
    let cid = canonical_test_node().cid().unwrap();
    let ev = ChangeEvent {
        cid,
        labels: vec!["Post".to_string()],
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
    };
    sub.on_change(&ev);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn change_subscriber_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn ChangeSubscriber>();
}
