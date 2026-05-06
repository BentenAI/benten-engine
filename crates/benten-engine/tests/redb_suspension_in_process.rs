//! G17-A2 GREEN-PHASE pins for `RedbSuspensionStore` retention-window
//! override (phase-3-backlog §6.5 + r1-wsa-10).
//!
//! - `redb_suspension_store_retention_window_in_process_round_trip` —
//!   §6.5 single-process correctness pin: a cursor whose
//!   `registered_at` predates `set_retention_window(window)` by more
//!   than `window` is reported retention-exhausted by the production
//!   `RedbSuspensionStore::is_retention_exhausted` override.
//! - `redb_suspension_store_retention_window_override_persists_across_engine_open`
//!   — r1-wsa-10 durability pin: the override survives store close +
//!   re-open against the same redb file.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use benten_core::{Cid, SubscriberId};
use benten_engine::suspension_store::RedbSuspensionStore;
use benten_eval::suspension_store::SuspensionStore;

fn cursor_subscriber(seed: u8) -> SubscriberId {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    SubscriberId::from_cid(Cid::from_blake3_digest(bytes))
}

#[test]
fn redb_suspension_store_retention_window_in_process_round_trip() {
    // §6.5 — single-process correctness pin. The
    // `is_retention_exhausted` check fires within one
    // `RedbSuspensionStore::open` lifecycle once the configured window
    // elapses against the cursor's stamped `registered_at`.
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("susp.redb");
    let store = RedbSuspensionStore::open(&path).unwrap();

    // 1ms window — the test must drive past 1s in wall time because
    // `is_retention_exhausted` resolves at second-granularity per the
    // PersistedCursorMeta `registered_at_unix_secs` shape (the redb
    // side-table avoids sub-second drift between close + re-open).
    // Use 0-second window which is the operator force-exhaust escape
    // hatch + a deterministic test-time signal.
    store.set_retention_window(Duration::ZERO).unwrap();

    // Insert a cursor — `put_cursor` lazy-stamps registered_at on
    // first put.
    let sub = cursor_subscriber(7);
    store.put_cursor(&sub, 1).unwrap();

    // Every cursor is exhausted under a ZERO window — the
    // is_retention_exhausted check fires per §6.5 + serves as the
    // operator-visible canary that the override is wired.
    assert!(
        store.is_retention_exhausted(&sub),
        "RedbSuspensionStore::is_retention_exhausted MUST fire under \
         ZERO retention window per §6.5"
    );
}

#[test]
fn redb_suspension_store_retention_window_override_persists_across_engine_open() {
    // r1-wsa-10 — the override is stored durably in the redb side-
    // table; closing the store and re-opening the same file MUST yield
    // the same `retention_window()` reading.
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("susp.redb");

    // Handle A: set the override + drop the store.
    let target = Duration::from_hours(1);
    {
        let store_a = RedbSuspensionStore::open(&path).unwrap();
        store_a.set_retention_window(target).unwrap();
        let read_back = store_a.retention_window().unwrap();
        assert_eq!(
            read_back,
            Some(target),
            "set_retention_window MUST round-trip in handle A"
        );
        drop(store_a);
    }

    // Handle B: re-open the same file. The override must still be in
    // effect — without persistence, this returns None (or a different
    // value) and the test fails.
    let store_b = RedbSuspensionStore::open(&path).unwrap();
    let observed = store_b.retention_window().unwrap();
    assert_eq!(
        observed,
        Some(target),
        "retention-window override MUST persist across engine re-open per r1-wsa-10 \
         (persistent-state shape transition pin); observed: {observed:?}"
    );
}
