//! Phase 2b R3-B — D19 nested-dispatch denial unit tests (G7-A).
//!
//! D19-RESOLVED calibrated:
//!   - Strict ban on `Engine::call`-from-host-fn (closes sec-pre-r1-08
//!     cap-context-confusion attack via SANDBOX → CALL → SANDBOX chain).
//!   - Permissive on async host-fns gated by reserved `host:async` cap
//!     (Phase 3 iroh KVBackend forward-compat).
//!   - Catalog rename: `E_SANDBOX_REENTRANCY_DENIED` →
//!     `E_SANDBOX_NESTED_DISPATCH_DENIED` (per wsa-7 + r1-security
//!     convergence). The name aligns with the actual security claim.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — D19 catalog rename"]
fn sandbox_nested_dispatch_denied_renamed_from_reentrancy() {
    // D19 catalog rename verification — `ErrorCode::SandboxNestedDispatchDenied`
    // exists; `as_str()` returns "E_SANDBOX_NESTED_DISPATCH_DENIED".
    //
    // The OLD code `E_SANDBOX_REENTRANCY_DENIED` MUST NOT appear in the
    // catalog (no deprecated alias per CLAUDE.md non-negotiable rule #5).
    //
    // White-box: parse `docs/ERROR-CATALOG.md`; assert NESTED_DISPATCH
    // present, REENTRANCY absent.
    todo!("R5 G7-A — catalog code presence + absence assertion");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D19 + sec-pre-r1-08 nested SANDBOX denial"]
fn sandbox_nested_sandbox_via_call_denied() {
    // D19 + sec-pre-r1-08 — host-fn callback attempts `engine.call(...)`
    // which would dispatch into another handler (potentially containing
    // a SANDBOX). The outer SANDBOX execution context's
    // `dispatch_in_flight: bool` flag is set; the inner Engine::call
    // checks the flag BEFORE acquiring the dispatch lock and returns
    // E_SANDBOX_NESTED_DISPATCH_DENIED.
    //
    // Cap-context-confusion attack defense: a SANDBOX module CANNOT
    // launder caps through CALL by piggy-backing on a host-fn's
    // (actor, grant) context.
    //
    // Test:
    //   1. Register host-fn that calls engine.call() internally
    //      (test-only fixture).
    //   2. Module invokes that host-fn during SANDBOX primitive.
    //   3. Assertion: outer SANDBOX result is
    //      E_SANDBOX_NESTED_DISPATCH_DENIED.
    todo!("R5 G7-A — fixture + test-only host-fn that triggers nested call");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D19 calibrated async forward-compat"]
fn sandbox_async_host_fn_gated_by_host_async_cap_reserved_phase_3() {
    // D19 calibrated — `host:async` capability is reserved in 2b.
    //
    // No async host-fn ships in 2b (D1 ships time/log/kv:read all sync).
    // But the calibration test pins:
    //   1. `requires_async: true` field exists in host-functions.toml
    //      schema and is parseable by build.rs codegen.
    //   2. `host:async` capability string is recognized by the cap
    //      system (capability registry has the entry).
    //   3. A test-only host-fn declared with `requires_async = true` AND
    //      `requires = "host:async"` is rejected at SANDBOX entry if the
    //      dispatching grant lacks `host:async` (E_SANDBOX_HOST_FN_DENIED).
    //   4. wasmtime `async-support` feature flag is enabled (D27): assert
    //      `wasmtime::Config` constructor with `.async_support(true)`
    //      returns Ok (would compile-error or no-op without feature).
    //
    // Phase 3 iroh kv:read flips `requires_async = true` and acquires
    // `host:async` via the host-functions.toml manifest — no breaking
    // change required.
    todo!("R5 G7-A — async-support flag wired + host:async cap reserved");
}
