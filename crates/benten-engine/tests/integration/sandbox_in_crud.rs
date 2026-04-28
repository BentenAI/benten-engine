//! Phase 2b R3-B — SANDBOX-inside-CRUD-handler integration tests (G7-A).
//!
//! Pin source: plan §4 SANDBOX integration.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — SANDBOX inside CRUD handler"]
fn sandbox_inside_crud_handler_e2e() {
    // Plan §4 SANDBOX integration — a `crud('post')` generated handler
    // composes a SANDBOX primitive (e.g., for content-validation
    // module). End-to-end:
    //   1. Build crud('post') handler with embedded SANDBOX node.
    //   2. Engine.call('post', input).
    //   3. SANDBOX module runs, returns validated output.
    //   4. WRITE primitive at the end of the handler persists the
    //      validated value.
    //   5. Read it back through engine.read.
    //   6. Assert: read value matches SANDBOX-validated payload.
    todo!("R5 G7-A — wire SANDBOX node into crud handler builder");
}

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — host-boundary cap-check on WRITE"]
fn sandbox_result_fed_to_write_cap_checked_at_host_boundary() {
    // Plan §4 — when SANDBOX output feeds a WRITE primitive (e.g.,
    // SANDBOX returns the value to write), the WRITE's cap-check fires
    // at the host boundary against the dispatcher's grant.
    //
    // The SANDBOX module's caps (from its manifest) DO NOT extend to
    // the WRITE — the WRITE is evaluated by the engine in the handler's
    // context, not the module's.
    //
    // Test:
    //   1. Dispatcher grant has caps = ["host:compute:..."] but NO
    //      WRITE-authority caps.
    //   2. Handler: SANDBOX → WRITE.
    //   3. SANDBOX succeeds; WRITE fails with E_CAP_DENIED.
    //   4. The SANDBOX's manifest CANNOT include WRITE-authority caps
    //      (manifest cap-set is engine-validated against the
    //      dispatcher's grant at SANDBOX entry per D7).
    todo!("R5 G7-A — SANDBOX → WRITE cap-check at host boundary");
}
