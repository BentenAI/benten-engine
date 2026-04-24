//! Edge-case / integration test: devserver hot-reload dispatched during an
//! in-flight evaluation — the existing evaluation must complete against the
//! pre-reload handler version; the next call uses the new registration.
//!
//! R2 landscape §2.9 "Devserver in-flight evaluations complete before reload"
//! (dx-r1 devserver). G11-A DEVSERVER wave (2026-04-24) lands the real
//! coordinator + `call_for_test` so this file goes green.
//!
//! Concerns pinned:
//! - An evaluation started before a reload completes using the old subgraph.
//! - A call made *after* the reload lands uses the new subgraph.
//! - Hot-reload issued during a suspended WAIT honours the suspend
//!   (serialized bytes remain resumable because suspension pins the
//!   subgraph CID, not a pointer to a handler-id slot).
//! - A panic-safe path: if the in-flight eval panics mid-call, reload
//!   still proceeds without deadlocking.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Value;
use benten_dev::{DevServer, ReloadCoordinator};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

/// Helper: pick the coordinator the dev-server uses internally so the
/// test can release the slow-transform gate from the harness side.
/// Phase-2a exposes a `slow_transform_release` test affordance via a
/// freshly-issued reload-coordinator handle that shares state through
/// `Arc::ptr_eq`. Today the simplest path is to construct a
/// coordinator directly per dev-server; the dev-server's
/// `call_for_test` parks on its OWN coordinator, so the harness uses
/// the dev-server's `slow_transform_release_for_test` accessor.
fn release_slow_transform(dev: &DevServer) {
    dev.slow_transform_release_for_test();
}

#[test]
fn devserver_in_flight_evaluations_complete_before_reload() {
    // 1. Register a slow v1 handler.
    // 2. Thread A calls the handler (blocks on slow_transform).
    // 3. Main thread re-registers handler with different source → v2.
    // 4. Main releases the slow gate; thread A completes against v1.
    // 5. Next call from main thread uses v2.
    let dir = tempdir().unwrap();
    let dev = Arc::new(DevServer::builder().workspace(dir.path()).build().unwrap());

    dev.register_handler_from_str("slow_h", "run", "read('x') >> slow_transform >> respond")
        .unwrap();

    let dev_a = Arc::clone(&dev);
    let thread_a = thread::spawn(move || {
        dev_a
            .call_for_test("slow_h", "run", Value::unit())
            .expect("in-flight eval must complete on v1")
    });

    // Give thread_a a moment to enter slow_transform_wait.
    thread::sleep(Duration::from_millis(20));

    // Reload to v2 while thread A is parked at slow_transform.
    dev.register_handler_from_str(
        "slow_h",
        "run",
        "read('x') >> transform('identity') >> respond",
    )
    .expect("reload must succeed with in-flight call");

    // Release thread A's slow_transform gate — it now resolves against
    // its captured v1 snapshot.
    release_slow_transform(&dev);

    let v1_result = thread_a.join().expect("thread A must not panic");

    assert_eq!(
        v1_result.handler_version_tag_for_test(),
        "v1",
        "in-flight eval must complete on pre-reload version"
    );

    // Next call from main thread uses v2. The fresh handler does NOT
    // contain slow_transform, so this returns immediately.
    let v2_result = dev.call_for_test("slow_h", "run", Value::unit()).unwrap();
    assert_eq!(
        v2_result.handler_version_tag_for_test(),
        "v2",
        "post-reload calls must use new version"
    );
}

#[test]
fn devserver_reload_during_suspended_wait_leaves_suspension_intact() {
    // WAIT-suspend captures a subgraph CID; reloading the handler does not
    // invalidate already-issued suspension handles. Resume must still work.
    let dir = tempdir().unwrap();
    let dev = DevServer::builder()
        .workspace(dir.path())
        .build()
        .expect("devserver start");

    dev.register_handler_from_str("wait_h", "run", "read('x') >> wait_signal('go') >> respond")
        .unwrap();

    let handle_bytes = dev
        .call_with_suspension_for_test("wait_h", "run", Value::unit())
        .expect("initial call must suspend");

    // Reload with a different handler shape.
    dev.register_handler_from_str(
        "wait_h",
        "run",
        "read('x') >> transform('identity') >> respond",
    )
    .expect("reload must succeed");

    // The previously-issued handle must still resume cleanly because it
    // was pinned to the v1 subgraph CID.
    let result = dev
        .resume_for_test(&handle_bytes, Value::text("go_payload"))
        .expect("suspension from v1 must still resume under v1 subgraph CID");
    assert_eq!(
        result.handler_version_tag_for_test(),
        "v1",
        "resume must execute on v1 subgraph CID, ignoring the reload"
    );
}

#[test]
fn devserver_reload_after_panicking_in_flight_call_does_not_deadlock() {
    // Panic-safety pin: if an in-flight eval panics, the reload coordinator
    // must NOT deadlock. Subsequent calls succeed.
    let dir = tempdir().unwrap();
    let dev = Arc::new(DevServer::builder().workspace(dir.path()).build().unwrap());

    dev.register_handler_from_str(
        "panicky_h",
        "run",
        "read('x') >> explode_transform >> respond",
    )
    .unwrap();

    let dev_panic = Arc::clone(&dev);
    let panic_thread = thread::spawn(move || {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dev_panic.call_for_test("panicky_h", "run", Value::unit());
        }));
    });
    panic_thread.join().expect("panic-thread must terminate");

    // Reload must succeed even though a panicking call happened.
    dev.register_handler_from_str("panicky_h", "run", "read('x') >> respond")
        .expect("reload after panic must not deadlock");

    // And a subsequent call must succeed on the new handler.
    let ok = dev.call_for_test("panicky_h", "run", Value::unit());
    assert!(
        ok.is_ok(),
        "call after reload-post-panic must succeed, got {:?}",
        ok
    );
}

#[test]
fn reload_coordinator_in_flight_count_drops_on_panic_unwind() {
    // Sanity pin on the coordinator's panic-safety contract — independent
    // of the DevServer surface so the regression surface is narrow.
    let coord = Arc::new(ReloadCoordinator::new());
    let c2 = Arc::clone(&coord);
    let h = thread::spawn(move || {
        let _g = c2.begin_call();
        panic!("intentional");
    });
    let _ = h.join();
    assert_eq!(coord.in_flight_count(), 0);
}
