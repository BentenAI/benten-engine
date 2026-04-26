#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: STREAM error catalog drift (G6-A).
//!
//! Pin source: plan §3 G6-A error catalog adds —
//! `E_STREAM_BACKPRESSURE_DROPPED`, `E_STREAM_CLOSED_BY_PEER`,
//! `E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED`.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;

/// `E_STREAM_BACKPRESSURE_DROPPED` round-trips through `as_str` / `from_str`.
#[test]
#[ignore = "Phase 2b G6-A pending — depends on ErrorCode::StreamBackpressureDropped variant"]
fn e_stream_backpressure_dropped_routed_through_typed_error_catalog() {
    let code = ErrorCode::StreamBackpressureDropped;
    assert_eq!(code.as_str(), "E_STREAM_BACKPRESSURE_DROPPED");
    assert_eq!(
        ErrorCode::from_str("E_STREAM_BACKPRESSURE_DROPPED"),
        ErrorCode::StreamBackpressureDropped
    );
}

/// `E_STREAM_CLOSED_BY_PEER` round-trips.
#[test]
#[ignore = "Phase 2b G6-A pending — depends on ErrorCode::StreamClosedByPeer variant"]
fn e_stream_closed_by_peer_routed_through_typed_error_catalog() {
    let code = ErrorCode::StreamClosedByPeer;
    assert_eq!(code.as_str(), "E_STREAM_CLOSED_BY_PEER");
    assert_eq!(
        ErrorCode::from_str("E_STREAM_CLOSED_BY_PEER"),
        ErrorCode::StreamClosedByPeer
    );
}

/// `E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED` round-trips. Companion to the
/// `stream_producer_wallclock_kills_blocked_send` runtime test.
#[test]
#[ignore = "Phase 2b G6-A pending — depends on ErrorCode::StreamProducerWallclockExceeded variant"]
fn e_stream_producer_wallclock_exceeded_routed_through_typed_error_catalog() {
    let code = ErrorCode::StreamProducerWallclockExceeded;
    assert_eq!(code.as_str(), "E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED");
    assert_eq!(
        ErrorCode::from_str("E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED"),
        ErrorCode::StreamProducerWallclockExceeded
    );
}
