//! R3-D RED-PHASE pins for `RedbSuspensionStore` retention-window
//! override (G17-A2 wave 5b; phase-3-backlog §6.5 + r1-wsa-10 MINOR).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A2):
//!
//! - `tests/redb_suspension_store_retention_window_in_process_round_trip`
//!   — phase-3-backlog §6.5 (RECALIBRATED at R4-FP per r4-r1-wsa-10
//!   MINOR — function/file name was `cross_process_round_trip` but the
//!   body is in-process; renamed for accuracy. The cross-process
//!   semantics — durability across engine open lifecycles — are
//!   covered by the second pin `..._override_persists_across_engine_open`,
//!   so the cross-process narrative is not lost; it's placed at the
//!   symbol where the body actually drives it).
//! - `tests/redb_suspension_store_retention_window_override_persists_across_engine_open`
//!   — r1-wsa-10 (persistent-state shape transition; this is the
//!   actual cross-engine-handle / cross-process-equivalent shape —
//!   redb durability is process-agnostic at the file level)
//!
//! ## Retention-window override shape
//!
//! Phase-2b shipped `SuspensionStore::is_retention_exhausted` as
//! always-`false` for `RedbSuspensionStore` (no expiration). Phase-3
//! wires the override per phase-3-backlog §6.5: a configurable
//! retention window + a typed expiration check that fires when a
//! suspension exceeds the window.
//!
//! Because the override is stored DURABLY in redb (per r1-wsa-10),
//! the persistent-state shape must transition correctly across engine
//! re-opens — a process that sets the override, closes the engine,
//! and reopens MUST observe the same retention window.
//!
//! ## Why two distinct pin functions (R4-FP rename per r4-r1-wsa-10)
//!
//! - `..._in_process_round_trip` — single-process round-trip with a
//!   suspension that exceeds the retention window: the
//!   `is_retention_exhausted` check fires within one
//!   `RedbSuspensionStore::open` lifecycle. Verifies CORRECTNESS of
//!   the expiration check.
//! - `..._override_persists_across_engine_open` — the actual cross-
//!   engine-handle / cross-process-equivalent shape: the override is
//!   set in handle A, the store is dropped, the same redb file is
//!   re-opened in handle B (which is process-agnostic at the file
//!   level — redb durability is governed by file commits, not handle
//!   identity), and handle B observes the same retention window.
//!   Verifies DURABILITY of the override per r1-wsa-10.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A2 wave 5b wires RedbSuspensionStore retention-window override per phase-3-backlog §6.5 + r4-r1-wsa-10 rename to in_process (body shape is single-lifecycle correctness; durability is the sibling pin's concern)"]
fn redb_suspension_store_retention_window_in_process_round_trip() {
    // phase-3-backlog §6.5 pin. G17-A2 implementer wires this:
    //
    //   use benten_engine::suspension_store::RedbSuspensionStore;
    //
    //   let tmpdir = tempfile::tempdir().unwrap();
    //   let store = RedbSuspensionStore::open(tmpdir.path().join("susp.redb")).unwrap();
    //
    //   // Set retention window to 1 ms (so test can drive it):
    //   store.set_retention_window(std::time::Duration::from_millis(1)).unwrap();
    //
    //   // Insert a suspension at t0:
    //   let token = store.insert_suspension(/* test fixture */).unwrap();
    //
    //   // Sleep past retention:
    //   std::thread::sleep(std::time::Duration::from_millis(5));
    //
    //   // is_retention_exhausted fires:
    //   assert!(store.is_retention_exhausted(&token).unwrap(),
    //       "RedbSuspensionStore.is_retention_exhausted must fire after retention window per §6.5");
    //
    // OBSERVABLE consequence: suspensions older than the retention
    // window are reaped (or refused) per the override. Defends §6.5
    // surface.
    unimplemented!(
        "G17-A2 wires RedbSuspensionStore retention-window override + same-process expiration assertion"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-A2 wave 5b — retention-window override persists across Engine::open per r1-wsa-10"]
fn redb_suspension_store_retention_window_override_persists_across_engine_open() {
    // r1-wsa-10 persistent-state pin. G17-A2 implementer:
    //
    //   let tmpdir = tempfile::tempdir().unwrap();
    //   let path = tmpdir.path().join("susp.redb");
    //
    //   // Process A: set retention window, write a suspension, close.
    //   {
    //       let store = RedbSuspensionStore::open(&path).unwrap();
    //       store.set_retention_window(std::time::Duration::from_secs(60 * 60)).unwrap();
    //       let _ = store.insert_suspension(/* fixture */).unwrap();
    //       drop(store); // closes redb file handle
    //   }
    //
    //   // Process B: re-open the same file. The retention-window
    //   // setting persisted to disk (durable redb table per r1-wsa-10).
    //   {
    //       let store = RedbSuspensionStore::open(&path).unwrap();
    //       let observed = store.retention_window().unwrap();
    //       assert_eq!(observed, std::time::Duration::from_secs(60 * 60),
    //           "retention window override MUST persist across engine re-open per r1-wsa-10 \
    //            (persistent-state shape transition pin)");
    //   }
    //
    // OBSERVABLE consequence: the override survives engine close +
    // re-open. A regression that stores the override only in-memory
    // (e.g. as a struct field on `RedbSuspensionStore` but not persisted
    // to a redb table) would fail this pin. Defends r1-wsa-10 directly.
    //
    // Pairs with the same-process round-trip pin: that one verifies
    // CORRECTNESS; this one verifies DURABILITY.
    unimplemented!("G17-A2 wires retention-window-override persistence across engine re-open");
}
