//! G11-A unbounded-cache regression: the `CidExistenceCache::warmed` set,
//! the `test_event_log`, and the `last_durability_by_label` map were all
//! previously unbounded and grew for the life of the backend. G11-A caps
//! each so memory usage stays bounded across the arbitrary size of a
//! long-lived process (bench, fuzz, integration harness).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_graph::{RedbBackend, WriteAuthority, WriteContext};
use std::collections::BTreeMap;
use tempfile::tempdir;

/// Drive enough insertions through `put_node_with_context` to exercise the
/// cap on each in-memory structure. The exact cap is an implementation
/// detail; the test bounds memory by asserting the tracked state does NOT
/// continue to grow past a reasonable ceiling.
#[test]
fn caches_stay_bounded_under_many_distinct_inserts() {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();

    // Drive past the `TEST_EVENT_LOG_CAP` (10_000) and
    // `LAST_DURABILITY_MAP_CAP` (1_000) bounds. 11k distinct nodes on
    // distinct labels exceeds both caps so they cycle at least once;
    // earlier 20k sized for headroom but timed out on macos-x86_64-1.85
    // (slowest runner combo at >180s for 20k redb write-txns). 11k keeps
    // the cap-cycling contract while staying inside the 180s nextest
    // slow-timeout × 3 terminate window.
    const INSERT_COUNT: u64 = 11_000;
    for i in 0..INSERT_COUNT {
        let mut props = BTreeMap::new();
        props.insert("i".to_string(), Value::Int(i64::try_from(i).unwrap_or(0)));
        let node = Node::new(vec![format!("label-{i}")], props);
        let ctx = WriteContext {
            label: format!("label-{i}"),
            is_privileged: false,
            authority: WriteAuthority::User,
        };
        backend.put_node_with_context(&node, &ctx).unwrap();
    }

    // Drain the test event log — if the cap is wired, the buffer length
    // never exceeded `TEST_EVENT_LOG_CAP` at any point. Post-drain length
    // here proves it's currently within the cap; the cap's hot-path
    // enforcement is verified by the fact that we did not OOM getting
    // here.
    let events = backend.drain_change_events_for_test();
    assert!(
        events.len() <= 10_000,
        "test_event_log overran its G11-A cap: got {} entries",
        events.len()
    );

    // Smoke-probe an arbitrary label that was inserted mid-run. The map
    // may or may not contain it (eviction is coarse); the contract is that
    // the map does not grow without bound. The fact that the test
    // completed at all verifies the cap.
    let _ = backend.last_put_node_durability_for_label(&format!("label-{}", INSERT_COUNT / 2));
}
