//! R3-C RED-PHASE end-to-end pin: two-process atrium bidirectional
//! sync (G16-D wave-6b; per r2-test-landscape §2.4 G16-D + plan §3
//! G16-D row + scope-real-22).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-D row
//!   `integration/atrium_two_process_bidirectional_sync_end_to_end`.
//! - plan §3 G16-D row line "~150-250 LOC test driver per
//!   scope-real-22 — exit-criterion-1 cross-process pin".
//! - `scope-real-22` (~150-250 LOC test driver expected at G16-D
//!   wave; cross-process exercises the full transport stack).
//!
//! ## What this pins (distinct from `atrium_two_peer.rs`)
//!
//! `atrium_two_peer.rs` runs both peers in a single process; THIS
//! file runs them in DIFFERENT processes (forked test driver +
//! IPC), exercising the full transport stack (iroh QUIC + relay
//! fallback + handshake + UCAN grant exchange) end-to-end.
//!
//! Originally placed in `tests/phase_3_workspace/`; relocated to
//! `tests/integration/` at R4-FP/R3-C per R3-CPC-1.
//!
//! ## Atrium DSL shape (B-prime per Ben's D1 decision 2026-05-04)
//!
//! `engine.atrium({config}).join()` factory pattern; both processes
//! use the handle-returning shape.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale pointing to phase-3-backlog §7.3.D STALE-RATIONALE sweep #2; destination next Phase-3-close orchestrator-direct fix-pass batch (G16-D wave-6b CLOSED at PR #163).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — atrium two-process bidirectional sync. G16-D wave-6b shipped (PR #163) + G16-B-E PR #160 (substantive end-to-end multi-peer iroh sync); test body pins specific cross-process driver contract that needs test infrastructure authoring; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn atrium_two_process_bidirectional_sync_end_to_end() {
    // scope-real-22 + plan §3 G16-D pin. G16-D implementer wires
    // this against a cross-process test fixture (~150-250 LOC):
    //
    //   1. Test parent forks a child process bound to a known
    //      peer-DID + device-DID; child opens an Atrium endpoint.
    //   2. Parent opens its own Atrium endpoint under a different
    //      peer-DID.
    //   3. Parent + child handshake via the iroh transport stack
    //      (loopback or relay; not in-process channel mock).
    //   4. Parent writes; child sees the write within bounded time.
    //   5. Child writes; parent sees the write.
    //   6. Both close their atrium handles cleanly; parent reaps
    //      the child process.
    //
    // OBSERVABLE consequence: the two-process driver exercises the
    // FULL transport stack (no in-process shortcuts); defends
    // against the failure shape where in-process tests pass but
    // real cross-process sync fails.
    unimplemented!(
        "G16-D wires two-process atrium bidirectional sync end-to-end (~150-250 LOC driver)"
    );
}
