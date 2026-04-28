//! Regression test for g5-p2-ivm-3: panic isolation in
//! `Subscriber::apply_event`.
//!
//! A panicking view MUST NOT take down the whole subscriber. Other views
//! continue receiving events; the panicking view is marked stale via
//! `View::mark_stale`. The load-bearing mechanism is the
//! `catch_unwind(AssertUnwindSafe(...))` wrap in `apply_event`; a refactor
//! that removes the `AssertUnwindSafe` (or inlines the `view.update` call
//! out of the `catch_unwind`) silently regresses the isolation guarantee,
//! and this test is the backstop.
//!
//! G5 pass-2 mini-review `.addl/phase-1/r5-g5-pass2-ivm-algorithm-b-reviewer.json`.

#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind, ChangeSubscriber};
use benten_ivm::{Subscriber, View, ViewError, ViewQuery, ViewResult};

/// A view whose `update` always panics. We only expect one update call —
/// the subscriber marks the view stale on panic, and stale views
/// short-circuit further `update` dispatches, so subsequent events do not
/// re-panic.
#[derive(Debug)]
struct PanickingView {
    stale: bool,
}

impl PanickingView {
    fn new() -> Self {
        Self { stale: false }
    }
}

impl View for PanickingView {
    fn update(&mut self, _event: &ChangeEvent) -> Result<(), ViewError> {
        panic!("intentional panic for isolation test");
    }

    fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
        Ok(ViewResult::Cids(Vec::new()))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        self.stale = false;
        Ok(())
    }

    fn id(&self) -> &str {
        "panicking"
    }

    fn is_stale(&self) -> bool {
        self.stale
    }

    fn mark_stale(&mut self) {
        self.stale = true;
    }
}

/// A healthy view that counts `update` calls it has observed. Uses an
/// `Arc<AtomicUsize>` so the test body can read the counter after giving
/// the view away as `Box<dyn View>`.
#[derive(Debug)]
struct CountingView {
    updates_seen: Arc<AtomicUsize>,
}

impl CountingView {
    fn new() -> (Self, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        (
            Self {
                updates_seen: Arc::clone(&counter),
            },
            counter,
        )
    }
}

impl View for CountingView {
    fn update(&mut self, _event: &ChangeEvent) -> Result<(), ViewError> {
        self.updates_seen.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn read(&self, _query: &ViewQuery) -> Result<ViewResult, ViewError> {
        Ok(ViewResult::Cids(Vec::new()))
    }

    fn rebuild(&mut self) -> Result<(), ViewError> {
        Ok(())
    }

    fn id(&self) -> &str {
        "counting"
    }

    fn is_stale(&self) -> bool {
        false
    }
}

fn sample_event() -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        labels: vec!["Post".to_string()],
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: None,
        edge_endpoints: None,
    }
}

#[test]
fn panicking_view_marks_stale_healthy_view_continues() {
    let (counting, counter) = CountingView::new();
    let sub = Subscriber::new()
        .with_view(Box::new(PanickingView::new()))
        .with_view(Box::new(counting));

    assert_eq!(sub.view_count(), 2);

    // `on_change` takes `&self` and must absorb the panic — a bubble-up
    // would abort the test thread here.
    sub.on_change(&sample_event());

    // The healthy view saw the event exactly once.
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "healthy CountingView must observe the event despite co-registered panic"
    );

    // Subscriber did not drop or poison the view list.
    assert_eq!(
        sub.view_count(),
        2,
        "subscriber must retain both views after a view panic"
    );

    // Second event: PanickingView is already stale so the subscriber
    // short-circuits it (no second panic), and CountingView sees the next
    // update. This exercises the "stale stays stale" path alongside the
    // isolation guarantee.
    sub.on_change(&sample_event());
    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "healthy CountingView must continue receiving events on subsequent dispatches"
    );
}

#[test]
fn panicking_view_does_not_prevent_later_registrations_from_receiving_events() {
    // Order-variant: register the panicking view LAST, so the healthy view
    // processes the event before `apply_event` reaches the panic path. The
    // isolation must still hold for any registration order.
    let (counting, counter) = CountingView::new();
    let sub = Subscriber::new()
        .with_view(Box::new(counting))
        .with_view(Box::new(PanickingView::new()));

    sub.on_change(&sample_event());
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(sub.view_count(), 2);
}
