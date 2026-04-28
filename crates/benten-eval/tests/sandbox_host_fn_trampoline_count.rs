//! Phase 2b R3-B — D25 trampoline-counts unit tests (G7-A).
//!
//! D25-RESOLVED — host-fn output bytes counted at the codegen-emitted
//! TRAMPOLINE (centralized accounting; one place to audit), NOT in the
//! host-fn body. Body never touches the counter directly.
//!
//! This is the implementation default; host-fns that need to bypass the
//! output budget (NONE in 2b's D1 surface) declare
//! `bypass_output_budget = true` in host-functions.toml.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — D25 trampoline accounting"]
fn sandbox_host_fn_output_bytes_counted_at_trampoline_not_body() {
    // D25-RESOLVED — emitter is the trampoline. White-box test:
    //
    //   1. Register a test host-fn whose body returns N bytes via the
    //      standard return-path; body does NOT call sink.write() directly.
    //   2. Invoke the host-fn from a SANDBOX module.
    //   3. Assert: SandboxOutputBudget.consumed += N (incremented by
    //      trampoline AFTER body returns, BEFORE handing bytes back to
    //      the guest).
    //   4. Assert: body's source code (statically scanned via
    //      `testing_assert_no_direct_sink_writes_in_body(host_fn_id)`)
    //      contains zero direct `CountedSink::write()` invocations —
    //      proving the centralized accounting claim.
    //
    // Centralized accounting = one audit point per host-fn output path.
    todo!("R5 G7-A — assert trampoline counts + body has no direct writes");
}

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — D25 bypass field default false"]
fn sandbox_host_fn_bypass_output_budget_field_default_false() {
    // D25 — host-functions.toml field `bypass_output_budget: bool`
    // defaults to `false`. NONE of the D1 initial surface
    // (time/log/kv:read) sets it to `true`.
    //
    // Test:
    //   1. Parse host-functions.toml.
    //   2. For each entry, if `bypass_output_budget` is unset, codegen
    //      MUST emit `bypass_output_budget: false` (regression guard).
    //   3. For the D1 initial surface, ALL three entries have
    //      `bypass_output_budget == false` (positive assertion that no
    //      production host-fn ships with bypass=true).
    //
    // Defense-in-depth: a future PR that sets bypass=true on a host-fn
    // MUST be reviewed for security implications (the default helps
    // ensure that requires explicit opt-in).
    todo!("R5 G7-A — assert default false + D1 surface has no bypass=true");
}
