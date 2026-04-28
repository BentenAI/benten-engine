//! Phase 2b R3-B — Inv-7 sandbox-output runtime unit tests (G7-B).
//!
//! D15 + D17 PRIMARY:
//!   - D17 PRIMARY: streaming `CountedSink` accumulator wraps every
//!     host-fn byte-emission; traps via Inv-7 BEFORE accepting bytes.
//!   - D15 trap-loudly default: NO silent truncation. Output overflow
//!     fires the typed error every time.
//!
//! Pin sources: D15 + D17 PRIMARY, sec-pre-r1-07.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "pending G7-A executor wiring; tracks G7-A's phase-2b/g7/a-sandbox-core PR (PR #30)"]
fn invariant_7_output_traps_loudly_via_counted_sink() {
    // D15 + D17 PRIMARY — Inv-7 fires on output overflow. The
    // CountedSink primary path is what trips the invariant; the
    // return-value backstop (D17 BACKSTOP) is the defense-in-depth
    // catcher (separate test in sandbox_output.rs).
    //
    // Test:
    //   1. SubgraphSpec declares output_max_bytes = 1024.
    //   2. SANDBOX module calls `log` with 2048-byte payload.
    //   3. CountedSink.write checks consumed (0) + 2048 > 1024;
    //      traps E_INV_SANDBOX_OUTPUT BEFORE accepting bytes.
    //   4. Trap metadata: consumed=0, limit=1024, attempted=2048,
    //      emitter_kind=HostFn("compute:log"), path="primary_streaming".
    //
    // The primary-streaming path's signature: trap fires WITHIN the
    // host-fn dispatch boundary (NOT at primitive boundary like the
    // backstop).
    todo!("R5 G7-B — assert trap fires from CountedSink primary path");
}

#[test]
#[ignore = "pending G7-A executor wiring; tracks G7-A's phase-2b/g7/a-sandbox-core PR (PR #30)"]
fn invariant_7_output_no_silent_truncation_default() {
    // D15 + sec-pre-r1-07 — default behavior is trap-loudly. NO opt-in
    // flag in 2b for silent truncation (deferred to Phase 3+ if a
    // legitimate use case arrives).
    //
    // Test:
    //   1. SubgraphSpec declares output_max_bytes = 1024 (no
    //      `truncate: true` flag — the field MUST NOT exist in 2b).
    //   2. SANDBOX module calls `log` with 2048-byte payload.
    //   3. Assertion: SANDBOX call fails with E_INV_SANDBOX_OUTPUT;
    //      log output sink received ZERO bytes (nothing partially
    //      written).
    //
    // The "received zero bytes" pin closes the silent-truncation
    // attack vector — a partially-written payload could be a
    // covert channel.
    todo!("R5 G7-B — assert log sink received zero bytes on trap");
}
