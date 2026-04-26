//! Phase 2b R3-B — `kv:read` host-fn unit tests (G7-A).
//!
//! D1 + sec-pre-r1-06 §2.4 — per-call cap-check via D18 + 1000-read
//! default budget per primitive call.
//!
//! Test surface:
//!   1. Per-grant budget enforcement (sandbox_host_fn_kv_read_respects_per_grant_budget_1000).
//!   2. Per-call cap-recheck after revoke (sandbox_host_fn_kv_read_per_call_cap_check_after_revoke).
//!      — pin source: D18 + ESC-9.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — D1 kv:read per-grant 1000 budget"]
fn sandbox_host_fn_kv_read_respects_per_grant_budget_1000() {
    // D1 + sec-pre-r1-06 §2.4 — kv:read declared with
    //   behavior = { kind = "kv_read", per_call_read_cap = 1000 }
    //
    // Test:
    //   1. Module invokes kv:read 999 times → all succeed.
    //   2. 1000th invocation succeeds (== cap is allowed).
    //   3. 1001st invocation returns E_SANDBOX_HOST_FN_DENIED (or
    //      E_SANDBOX_KV_READ_BUDGET_EXCEEDED if R5 picks a dedicated
    //      code; pin to whichever R5 lands).
    //
    // Read-amplification DOS defense — bounds host backend load per
    // primitive call.
    todo!("R5 G7-A — fixture kv_read_loop.wat + 1001 invocations + budget assertion");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D18 + ESC-9 cap revoke during kv:read"]
fn sandbox_host_fn_kv_read_per_call_cap_check_after_revoke() {
    // D18 + ESC-9 — `kv:read` is `cap_recheck = "per_call"` (sensitive).
    //
    // Test:
    //   1. Grant module `host:compute:kv:read` cap.
    //   2. Module invokes kv:read once → SUCCESS.
    //   3. `testing_revoke_cap_mid_call(engine, &kv_read_scope)`.
    //   4. Module invokes kv:read again → FAILS with
    //      E_SANDBOX_HOST_FN_DENIED.
    //
    // Mirrors ESC-9 escape vector. The umbrella ESC-9 driver (R3-C
    // territory) batches this into the security-class suite; this is
    // the surface-level unit-test for D18's per_call enforcement on
    // kv:read specifically.
    todo!("R5 G7-A — testing_revoke_cap_mid_call + per_call kv:read denial");
}
