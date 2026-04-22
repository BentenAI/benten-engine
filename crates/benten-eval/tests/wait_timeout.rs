//! Edge-case tests: `E_WAIT_TIMEOUT` firing semantics (G3-B, dx-r1 addendum).
//!
//! R2 landscape §2.5.3 row "`WaitTimeout` error".
//!
//! WAIT supports two variants: `wait({ duration })` and `wait({ signal })`.
//! Timeout applies to both: if the deadline elapses before resume, the
//! evaluator surfaces `E_WAIT_TIMEOUT` and routes through `ON_ERROR`.
//!
//! Concerns pinned:
//! - Duration variant with deadline already past at suspend-construction
//!   fires `E_WAIT_TIMEOUT` immediately at resume (boundary: zero-duration).
//! - Signal variant with an explicit `timeout_ms`: if the signal arrives
//!   AFTER the timeout, timeout wins (deterministic ordering pin).
//! - Signal variant with signal-arrives-before-timeout resumes normally
//!   (negative case so the two paths don't collapse).
//! - The typed error code matches `ErrorCode::WaitTimeout`, and the Outcome
//!   routes through `ON_ERROR` (NOT `ON_DENIED` — a timeout is not a capability
//!   denial).
//!
//! R3 red-phase contract: R5 (G3-B) lands `SubgraphBuilder::wait_duration`,
//! `SubgraphBuilder::wait_signal`, `wait_signal_with_timeout`, and the
//! WAIT primitive executor. These tests compile; they fail because the WAIT
//! executor is not wired.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes of diagnostic context per R1 triage."
)]

use benten_core::Value;
use benten_errors::ErrorCode;
use benten_eval::{MockTimeSource, Outcome, SubgraphBuilder, WaitOutcome, WaitResumeSignal};
use std::time::Duration;

fn wait_duration_subgraph(ms: u64) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new("wait_duration_edge");
    let start = sb.read("ignored");
    let w = sb.wait_duration(start, Duration::from_millis(ms));
    sb.respond(w);
    sb.build_validated().expect("builder validation")
}

fn wait_signal_subgraph_with_timeout_ms(ms: u64) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new("wait_signal_timeout_edge");
    let start = sb.read("ignored");
    let w = sb.wait_signal_with_timeout(start, "user_resumes", Duration::from_millis(ms));
    sb.respond(w);
    sb.build_validated().expect("builder validation")
}

#[test]
fn wait_duration_past_deadline_fires_e_wait_timeout() {
    // Zero-duration WAIT resumed after the (already-expired) deadline fires
    // E_WAIT_TIMEOUT. This is the degenerate timeout case.
    let sg = wait_duration_subgraph(0);
    let clock = MockTimeSource::at(Duration::from_secs(0));
    let mut ctx = benten_eval::EvalContext::with_clock(clock.clone());

    let step1 = benten_eval::evaluate(&sg, &mut ctx, Value::unit());
    let handle = match step1 {
        Outcome::Suspended(h) => h,
        other => panic!("WAIT must suspend first, got {other:?}"),
    };

    // Advance clock past the (zero) deadline before resume.
    clock.advance(Duration::from_millis(1));
    let result = benten_eval::resume(&sg, &mut ctx, handle, WaitResumeSignal::DurationElapsed);

    let err = match result {
        Outcome::Err(e) => e,
        other => panic!("expected E_WAIT_TIMEOUT, got {other:?}"),
    };
    assert_eq!(err.code(), ErrorCode::WaitTimeout);

    // Error-edge routing pin: WAIT-timeout routes through ON_ERROR, not
    // ON_DENIED.
    assert_eq!(
        err.routed_edge_label(),
        Some("ON_ERROR"),
        "E_WAIT_TIMEOUT must route via ON_ERROR"
    );
}

#[test]
fn wait_signal_arrives_after_timeout_fires_e_wait_timeout() {
    // Signal variant with a 100ms timeout. Clock advances 200ms before
    // resume; the signal DID arrive but the timeout expired first. The
    // timeout must win.
    let sg = wait_signal_subgraph_with_timeout_ms(100);
    let clock = MockTimeSource::at(Duration::from_secs(0));
    let mut ctx = benten_eval::EvalContext::with_clock(clock.clone());

    let outcome = benten_eval::evaluate(&sg, &mut ctx, Value::unit());
    let handle = outcome.expect_suspended();

    // Advance past the timeout boundary.
    clock.advance(Duration::from_millis(200));

    let result = benten_eval::resume(
        &sg,
        &mut ctx,
        handle,
        WaitResumeSignal::signal("user_resumes", Value::unit()),
    );
    let err = match result {
        Outcome::Err(e) => e,
        other => panic!("expected timeout, got {other:?}"),
    };
    assert_eq!(
        err.code(),
        ErrorCode::WaitTimeout,
        "signal arriving after timeout must fire E_WAIT_TIMEOUT, not resume"
    );
}

#[test]
fn wait_signal_arrives_before_timeout_resumes_normally() {
    // Negative case to ensure the timeout path is not over-firing. Signal
    // arrives at 50ms; timeout is 1000ms. Resume must produce a normal
    // Outcome::Complete.
    let sg = wait_signal_subgraph_with_timeout_ms(1000);
    let clock = MockTimeSource::at(Duration::from_secs(0));
    let mut ctx = benten_eval::EvalContext::with_clock(clock.clone());

    let handle = benten_eval::evaluate(&sg, &mut ctx, Value::unit()).expect_suspended();

    clock.advance(Duration::from_millis(50));

    let result = benten_eval::resume(
        &sg,
        &mut ctx,
        handle,
        WaitResumeSignal::signal("user_resumes", Value::text("payload")),
    );
    match result {
        Outcome::Complete(_) => {}
        other => panic!("signal-before-timeout must complete, got {other:?}"),
    }
}

#[test]
fn wait_timeout_error_is_distinct_from_denial() {
    // Contract pin: E_WAIT_TIMEOUT != E_CAP_DENIED. Developers relying on
    // the error-code string must not see a denial when a timeout fires.
    let sg = wait_duration_subgraph(0);
    let clock = MockTimeSource::at(Duration::from_secs(0));
    let mut ctx = benten_eval::EvalContext::with_clock(clock.clone());

    let handle = benten_eval::evaluate(&sg, &mut ctx, Value::unit()).expect_suspended();
    clock.advance(Duration::from_millis(1));
    let err = match benten_eval::resume(&sg, &mut ctx, handle, WaitResumeSignal::DurationElapsed) {
        Outcome::Err(e) => e,
        other => panic!("expected Err, got {other:?}"),
    };

    assert_ne!(err.code(), ErrorCode::CapDenied);
    assert_ne!(err.code(), ErrorCode::CapDeniedRead);
    assert_eq!(err.code().as_str(), "E_WAIT_TIMEOUT");
}

// Small adaptor to keep assertions concise where we assume the suspend path.
trait OutcomeExt {
    fn expect_suspended(self) -> WaitOutcome;
}

impl OutcomeExt for Outcome {
    fn expect_suspended(self) -> WaitOutcome {
        match self {
            Outcome::Suspended(h) => h,
            other => panic!("expected suspend, got {other:?}"),
        }
    }
}
