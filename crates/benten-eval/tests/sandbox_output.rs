//! Phase 2b R3-B — SANDBOX output-axis unit tests (G7-A).
//!
//! D17-RESOLVED defense-in-depth:
//!   - PRIMARY: streaming `CountedSink` accumulator wraps every host-fn
//!     byte-emission; traps via Inv-7 BEFORE accepting bytes.
//!   - BACKSTOP: return-value path runs same check at primitive boundary;
//!     catches host-fn paths that forgot to thread the sink.
//!
//! Both must be live (defense-in-depth — ON).
//!
//! Pin sources: D17-RESOLVED, wsa-1 (11×100KB log calls), wsa D17 boundary.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — D17 PRIMARY path"]
fn sandbox_output_limit_routes_inv_7_via_counted_sink_primary() {
    // D17 PRIMARY — single host-fn emits > limit. Streaming CountedSink
    // checks `consumed + bytes.len() > limit` BEFORE accepting bytes;
    // traps with `E_INV_SANDBOX_OUTPUT { consumed, limit, emitter_kind:
    // HostFn("compute:log") }`.
    //
    // Fixture: module calls `log` once with a 2 MiB payload; budget = 1 MiB.
    // Assertion: trap fires from CountedSink path (white-box: trap
    // metadata's `path = "primary_streaming"`, NOT `"return_backstop"`).
    todo!("R5 G7-A — fixture log_2MiB.wat + budget=1MiB + path assertion");
}

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — D17 BACKSTOP path"]
fn sandbox_output_limit_return_value_backstop_catches_misbehaving_host_fn() {
    // D17 BACKSTOP — host-fn that intentionally bypasses the streaming
    // sink (test-only fixture using `testing_register_uncounted_host_fn`
    // helper). Module emits 2 MiB through the bypass path; the
    // streaming CountedSink's `consumed` stays at 0; the wasm-export
    // return-value carries 2 MiB.
    //
    // Assertion: the return-value backstop check fires
    // `E_INV_SANDBOX_OUTPUT` at primitive boundary; trap metadata's
    // `path = "return_backstop"` (NOT primary).
    //
    // This test EXISTS to prove the backstop is live as defense-in-depth.
    // If the primary path is correctly threaded (post-G7-A), no
    // production host-fn ever reaches this code path — the backstop
    // exists for misbehaving / forgotten threading.
    todo!("R5 G7-A — testing helper to register uncounted host-fn for backstop test");
}

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — wsa D17 boundary condition"]
fn sandbox_output_at_exact_limit_succeeds() {
    // wsa D17 boundary — `consumed == limit` succeeds; `consumed == limit + 1`
    // traps. Off-by-one regression guard.
    //
    // Test 1: emit exactly N bytes with limit=N → succeeds.
    // Test 2: emit N+1 bytes with limit=N → traps with
    //   E_INV_SANDBOX_OUTPUT.
    todo!("R5 G7-A — boundary fixture + dual-assertion");
}

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — wsa-1 aggregate enforcement"]
fn sandbox_output_aggregate_across_host_fns_enforces_inv_7() {
    // wsa-1 suggested fix — 11 successive `log` calls @ 100 KiB each
    // under a 1 MiB ceiling. Tenth call succeeds (consumed=1.0 MiB);
    // eleventh call traps with E_INV_SANDBOX_OUTPUT.
    //
    // Confirms the AtomicU64 budget counter is shared across all host-fn
    // invocations within a single primitive call (NOT per-host-fn-reset).
    todo!("R5 G7-A — fixture log_loop_100KB.wat + budget=1MiB + 11-iter loop");
}
