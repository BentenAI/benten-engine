//! Phase 2b R3-B — SANDBOX memory-axis unit tests (G7-A).
//!
//! Pin sources: plan §3 G7-A (memory limit), ESC-2 (linmem grow attack).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — memory exhaustion routing"]
fn sandbox_memory_limit_kills_routes_e_sandbox_memory_exhausted() {
    // Plan §3 G7-A — module repeatedly invokes `memory.grow` until
    // exceeding per-call memory cap (default 64 MiB candidate).
    //
    // Assertion: `E_SANDBOX_MEMORY_EXHAUSTED` fires deterministically
    // BEFORE host OOM (wasmtime memory-limiter must intercept).
    //
    // D21 priority: MEMORY trumps WALLCLOCK + FUEL + OUTPUT — if a
    // memory-grow chain consumes fuel along the way, MEMORY axis wins.
    todo!("R5 G7-A — fixture linmem_grow_to_limit.wat + cap=64MiB assertion");
}
