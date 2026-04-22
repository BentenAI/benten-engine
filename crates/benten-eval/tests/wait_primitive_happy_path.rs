//! R3 unit tests for G3-A: `primitives::wait::execute` happy path.
//!
//! Three behaviors:
//! 1. Returns a SuspendedHandle (not a terminal Outcome) when the signal is
//!    still pending.
//! 2. Emits `TraceStep::SuspendBoundary` with the suspended state CID.
//! 3. Registers a pending-signal entry in the `system:WaitPending` zone.
//!
//! TDD red-phase: the WAIT executor module does not yet exist. Tests fail to
//! compile until G3-A lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.3).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::primitives::wait;
use benten_eval::{SuspendedHandle, TraceStep};

#[test]
fn wait_suspends_frame_when_signal_pending() {
    let signal = "waitfor:resume_1";
    let outcome = wait::execute_for_test_signal(signal).expect("wait execute");

    // Must return the Suspended arm, not Complete.
    match outcome {
        wait::WaitOutcome::Suspended(handle) => {
            // Type assertion: `handle` must be `SuspendedHandle`.
            fn assert_is_suspended_handle(_: &SuspendedHandle) {}
            assert_is_suspended_handle(&handle);
        }
        wait::WaitOutcome::Complete(_) => {
            panic!("WAIT with unmatched signal must return Suspended, not Complete")
        }
    }
}

#[test]
fn wait_emits_suspend_boundary_trace_step() {
    let signal = "waitfor:trace";
    let (outcome, trace) = wait::execute_for_test_signal_with_trace(signal).expect("wait+trace");

    // Trace contains exactly one SuspendBoundary step as the terminal trace
    // entry for this WAIT invocation.
    let last = trace
        .last()
        .expect("wait must emit at least one trace step");
    match last {
        TraceStep::SuspendBoundary { state_cid } => {
            assert_eq!(
                *state_cid,
                outcome.state_cid(),
                "boundary state_cid must match"
            );
        }
        other => panic!("expected SuspendBoundary, got {other:?}"),
    }
}

#[test]
fn wait_registers_pending_signal_in_system_wait_pending_zone() {
    let signal = "waitfor:zone";
    let reg = wait::execute_and_capture_zone_writes(signal).expect("execute");
    // Exactly one pending-signal entry written under `system:WaitPending`.
    let writes = reg.zone_writes_for_label("system:WaitPending");
    assert_eq!(
        writes.len(),
        1,
        "WAIT must register exactly one system:WaitPending entry, found {}",
        writes.len()
    );
}
