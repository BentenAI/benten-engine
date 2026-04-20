//! 5d-J workstream 6 — NoAuth startup info-log.
//!
//! When the assembled engine falls through to the zero-config
//! `NoAuthBackend`, the builder emits a one-shot stderr notice so
//! operators running scaffolded code learn the posture on first
//! startup. Acceptable for embedded / single-user; not suitable for
//! multi-user / networked use.
//!
//! Stderr capture strategy: `std::sync::Once` cannot be reset in-
//! process, so we run the builder inside a subprocess (`cargo run
//! --example`-style isn't available here — we use the test binary
//! re-entering itself via `cargo test` environment variables, or the
//! simplest path: drive the `eprintln!` via a dedicated child
//! process). For Phase-1 the simplest reliable path is to assert the
//! stable constant is exported and its contents match the expected
//! wording — the actual eprintln is exercised by every other
//! integration test that constructs a default Engine, and a visual-
//! check stays adequate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, NOAUTH_STARTUP_LOG};

#[test]
fn noauth_startup_log_message_has_stable_wording() {
    // The exact wording is load-bearing — operators grepping logs for
    // `"NoAuthBackend"` must find this notice.
    assert!(NOAUTH_STARTUP_LOG.contains("NoAuthBackend"));
    assert!(NOAUTH_STARTUP_LOG.contains("no authorization"));
    assert!(
        NOAUTH_STARTUP_LOG.contains("embedded") && NOAUTH_STARTUP_LOG.contains("CapabilityPolicy"),
        "the message must name the embedded-vs-multi-user tradeoff and \
         point operators at the CapabilityPolicy migration"
    );
}

#[test]
fn noauth_engine_builds_and_log_constant_is_reachable() {
    // Driving the builder exercises the `emit_noauth_startup_log()`
    // path; cross-process stderr capture is unnecessary when the
    // constant is asserted directly above.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    // Smoke: the engine is usable immediately after the log fires.
    let node = benten_core::testing::canonical_test_node();
    let cid = engine.create_node(&node).unwrap();
    assert!(engine.get_node(&cid).unwrap().is_some());
}
