//! Phase 2b R3-C — SANDBOX adversarial escape-vector batch (G7-B).
//!
//! Red-phase TDD per `.addl/phase-2b/r2-test-landscape.md` §5.1 +
//! `.addl/phase-2b/pre-r1-security-deliverables.md` §1. One test per ESC-N
//! vector; together they form the must-pass adversarial batch named in
//! plan §3 G7-A as `tests/sandbox_escape_attempts_denied (adversarial
//! fixture batch — 16 enumerated vectors per
//! pre-r1-security-deliverables.md Section 1)`.
//!
//! Each test loads its `.wat` fixture (or its pre-built `.wasm` per D26),
//! invokes the SANDBOX primitive via the future `engine.sandbox_call`
//! surface, and asserts the expected `E_SANDBOX_*` / `E_INV_SANDBOX_*`
//! variant fires. Bodies remain `todo!`-stubbed until R5 G7-A/G7-B lands
//! the SANDBOX surface (see `tests/sandbox_basic.rs` for the same pattern
//! R3-B established).
//!
//! Pin sources: pre-r1-security-deliverables.md Section 1 (ESC-1..16),
//! plan §3 G7-A + G7-B, D7 / D18 / D19 / D20 / D21 / D26 RESOLVED.
//! Cross-territory ownership: per R2 §10, R3-C owns the security drivers;
//! R3-B owns per-axis enforcement (fuel/memory/wallclock/output) tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed by these tests:
//   benten_eval::sandbox::{Sandbox, SandboxConfig, SandboxResult, ManifestRef}
//   benten_eval::sandbox::host_fns
//   benten_errors::ErrorCode::{
//       SandboxModuleInvalid,
//       SandboxFuelExhausted,
//       SandboxMemoryExhausted,
//       SandboxHostFnDenied,
//       SandboxHostFnNotFound,
//       SandboxNestedDispatchDenied,        // D19 RESOLVED rename
//       SandboxNestedDispatchDepthExceeded, // D20 saturation
//       SandboxManifestUnknown,
//       InvSandboxOutput,
//   }
//
// All ESC tests are gated #[cfg(not(target_arch = "wasm32"))] per
// sec-pre-r1-05; SANDBOX symbol is absent on wasm32.

const FIXTURE_DIR: &str = "tests/fixtures/sandbox/escape";

// =====================================================================
// Category: Memory (ESC-1..3)
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — SANDBOX executor not yet landed"]
fn sandbox_escape_oob_linmem_read_traps() {
    // ESC-1 — Out-of-bounds linear-memory read.
    //
    // Fixture: oob_linmem_read.wat / .wasm — load i32 at offset 0xFFFFFFF0
    // against a single-page memory.
    //
    // R5 wires:
    //   1. Compile fixture (D26 pre-built bytes already committed).
    //   2. `engine.sandbox_call(fixture_cid, ManifestRef::Named("compute-basic"),
    //                            input_bytes)` returns Err.
    //   3. Asserts the err's ErrorCode == SandboxModuleInvalid (or
    //      reserved SandboxStackExhausted if R1 grants one — current
    //      12-variant set folds into MODULE_INVALID).
    todo!("R5 G7-B — wire fixture load + assert ErrorCode::SandboxModuleInvalid");
}

#[test]
#[ignore = "Phase 2b G7-B pending — memory-budget enforcement"]
fn sandbox_escape_linmem_grow_to_limit_kills() {
    // ESC-2 — Linear-memory grow beyond per-call cap.
    //
    // Fixture: linmem_grow_to_limit.wat — memory.grow loop until OOM-cap
    // (default candidate: 64 MiB).
    //
    // R5 wires: assert ErrorCode::SandboxMemoryExhausted fires
    // deterministically before host OOM (i.e. before runner crashes).
    todo!("R5 G7-B — assert ErrorCode::SandboxMemoryExhausted");
}

#[test]
#[ignore = "Phase 2b G7-B pending — host-fn buf-len validation"]
fn sandbox_escape_host_buf_overrun_rejected() {
    // ESC-3 — Host-buffer overrun via host-fn output write.
    //
    // Fixture: host_buf_overrun.wat — passes pathological out_len to
    // kv:read; host-fn MUST validate against module's declared memory.
    //
    // R5 wires: ErrorCode::SandboxModuleInvalid (or HostFnDenied if the
    // host-fn validates ownership rather than module shape).
    todo!("R5 G7-B — assert ErrorCode::SandboxModuleInvalid on overrun");
}

// =====================================================================
// Category: Control-flow (ESC-4..5)
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — fuel-meter wiring"]
fn sandbox_escape_infinite_loop_fuel_bound() {
    // ESC-4 — Infinite loop without fuel.
    //
    // Fixture: infinite_loop.wat — `loop ... br 0 ... end`.
    //
    // R5 wires: ErrorCode::SandboxFuelExhausted fires within the per-call
    // fuel budget (D21 priority chain: WALLCLOCK > FUEL — in this fixture
    // fuel exhausts before the 30s D24 wallclock).
    todo!("R5 G7-B — assert ErrorCode::SandboxFuelExhausted");
}

#[test]
#[ignore = "Phase 2b G7-B pending — wasmtime stack-depth pin"]
fn sandbox_escape_recursive_call_overflow_traps() {
    // ESC-5 — Recursion-depth overflow via deep WASM call stack.
    //
    // Fixture: recursive_call_overflow.wat — unbounded direct recursion.
    //
    // R5 wires: wasmtime configures a max stack-depth; trap surfaces as
    // ErrorCode::SandboxModuleInvalid (or new SandboxStackExhausted if
    // R1 reserves one — current 12-variant set folds into MODULE_INVALID).
    // Distinct from Inv-4 (E_INV_SANDBOX_DEPTH counts SANDBOX-primitive
    // nest depth) — this is intra-module call depth.
    todo!("R5 G7-B — assert ErrorCode::SandboxModuleInvalid on recursion");
}

// =====================================================================
// Category: Fuel (ESC-6..7)
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — fuel u64 overflow regression pin"]
fn sandbox_escape_fuel_overflow_regression_held() {
    // ESC-6 — Fuel-counter overflow regression.
    //
    // Fixture: fuel_overflow_regression.wat — long-running arith loop.
    //
    // R5 wires: ErrorCode::SandboxFuelExhausted fires at the configured
    // budget regardless of how long the computation has run (i.e. the
    // u64 fuel counter doesn't silently restart). Pin against future
    // wasmtime upgrades.
    todo!("R5 G7-B — assert no fuel-counter overflow under long-running loop");
}

#[test]
#[ignore = "Phase 2b G7-B pending — D19 nested-dispatch denial"]
fn sandbox_escape_fuel_refill_via_host_fn_denied() {
    // ESC-7 — Fuel-refill bypass via host-fn re-entry.
    //
    // Fixture: fuel_refill_via_host_fn.wat — burns fuel while calling
    // log() repeatedly; driver-supplied log body attempts to re-enter
    // engine.call() to refresh fuel.
    //
    // R5 wires: ErrorCode::SandboxNestedDispatchDenied fires (D19
    // RESOLVED rename from REENTRANCY_DENIED). Per-call instance
    // lifecycle (D3-RESOLVED) ensures there's no persistent fuel state
    // to refill across primitives anyway; this test pins the in-call
    // re-entry denial.
    todo!("R5 G7-B — assert ErrorCode::SandboxNestedDispatchDenied via log-reentry");
}

// =====================================================================
// Category: Host-fn (ESC-8..10)
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — manifest allowlist enforcement"]
fn sandbox_escape_host_fn_not_on_manifest() {
    // ESC-8 — Call host-fn not in manifest.
    //
    // Fixture: host_fn_not_on_manifest.wat — module imports kv_read but
    // the test invocation passes ManifestRef::Named("compute-basic")
    // (which covers `time` + `log` only).
    //
    // R5 wires: link-time refusal yielding ErrorCode::SandboxHostFnNotFound
    // (preferred per ESC-8 inventory) OR call-time
    // ErrorCode::SandboxHostFnDenied (acceptable fallback).
    todo!("R5 G7-B — assert HostFnNotFound (preferred) or HostFnDenied");
}

#[test]
#[ignore = "Phase 2b G7-B + G7-A pending — D7 hybrid + D18 per_call recheck"]
fn sandbox_escape_host_fn_after_cap_revoke() {
    // ESC-9 — Call host-fn after cap revoked mid-primitive.
    //
    // Fixture: host_fn_after_cap_revoke.wat — invokes kv:read twice with
    // a yield-to-driver hook in the middle.
    //
    // R5 wires (D7 hybrid + D18 per_call):
    //   1. Set up engine with host:compute:kv:read cap granted.
    //   2. Driver-side `testing_yield_for_revoke` host-fn body calls
    //      `testing_revoke_cap_mid_call(engine, &CapScope::host_compute_kv_read())`
    //      between calls.
    //   3. Assert: first kv_read returns Ok; second kv_read returns Err
    //      with ErrorCode::SandboxHostFnDenied.
    //   4. Pinned property: D18 default `cap_recheck = "per_call"` for
    //      kv:read closes the TOCTOU window per `host-fn invocation`.
    //
    // TOCTOU bound documented in SECURITY-POSTURE: ~1 µs revocation
    // visibility (per host-fn call) — TIGHTER than Phase-1 Compromise #1
    // ITERATE batch boundary.
    todo!("R5 G7-B — assert second kv_read fires ErrorCode::SandboxHostFnDenied");
}

#[test]
#[ignore = "Phase 2b G7-B pending — D19 nested dispatch denial"]
fn sandbox_escape_reentrancy_via_host_fn_denied() {
    // ESC-10 — Host-fn re-entrancy denial.
    //
    // Fixture: reentrancy_via_host_fn.wat — calls driver-supplied
    // `testing_call_engine_dispatch` which attempts engine.call() back
    // through the dispatcher.
    //
    // R5 wires: ErrorCode::SandboxNestedDispatchDenied fires at the
    // inner SANDBOX dispatch attempt (D19 RESOLVED — name aligns with
    // the actual security claim of denying nested Engine::call). Closes
    // sec-pre-r1-08 cap-context confusion via SANDBOX → CALL → SANDBOX.
    todo!("R5 G7-B — assert ErrorCode::SandboxNestedDispatchDenied");
}

// =====================================================================
// Category: Component-Model (ESC-11..12) — gated; current 2b state has
// `component-model` feature removed per wsa-3.
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — Component-Model gated (wsa-3 removed feature)"]
#[cfg(feature = "component-model")]
fn sandbox_escape_component_type_mismatch_rejected() {
    // ESC-11 — Component-Model type mismatch with declared interface.
    //
    // Fixture: component_type_mismatch.wat — exports (i32) -> i64; host
    // imports as (i64) -> i32.
    //
    // R5 wires: wasmtime Component-Model type-checker refuses link;
    // surfaces as ErrorCode::SandboxModuleInvalid.
    //
    // Skip-gated per R2 §11.2 microgap 4 — wsa-3 removed `component-model`
    // from wasmtime Cargo features. Re-enable when feature is restored.
    todo!("R5 G7-B (when component-model re-enabled) — assert ModuleInvalid");
}

#[test]
#[ignore = "Phase 2b G7-B pending — Component-Model gated (wsa-3 removed feature)"]
#[cfg(feature = "component-model")]
fn sandbox_escape_resource_handle_forgery_rejected() {
    // ESC-12 — Resource handle forgery.
    //
    // Fixture: resource_handle_forgery.wat — passes 0xCAFEBABE as a
    // forged resource handle.
    //
    // R5 wires: Component-Model resource-handle table validates;
    // mismatch fires ErrorCode::SandboxModuleInvalid (or
    // SandboxHostFnDenied if the host-fn validates ownership).
    //
    // Skip-gated same as ESC-11.
    todo!("R5 G7-B (when component-model re-enabled) — assert ModuleInvalid");
}

// =====================================================================
// Category: wasmtime-internal (ESC-13)
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — Store-poison + nested-dispatch defense-in-depth"]
fn sandbox_escape_trap_in_fuel_callback_denied() {
    // ESC-13 — Trap during fuel-meter callback / Store-state corruption
    // attempt.
    //
    // No `.wat` for this vector — it's exercised by a Rust-side custom
    // host-fn driver that:
    //   1. Registers a host-fn whose body is invoked from inside a
    //      wasmtime fuel-trap unwinding path.
    //   2. The body attempts to mutate the Store while the trap is in
    //      flight (via engine.call() or direct Store mutation).
    //
    // R5 wires:
    //   - The host-fn dispatch path refuses re-entry while a trap is
    //     unwinding: ErrorCode::SandboxNestedDispatchDenied fires.
    //   - wasmtime's own Store-poison check fires deterministically on
    //     next access (defense-in-depth).
    //
    // Defense-in-depth even if current wasmtime forbids this — the test
    // pins the guarantee against wasmtime upgrades.
    todo!("R5 G7-B — assert ErrorCode::SandboxNestedDispatchDenied + Store-poison");
}

// =====================================================================
// Category: Cap-system (ESC-14..15)
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — embedded section ignored, manifest authoritative"]
fn sandbox_escape_forged_cap_claim_section_ignored() {
    // ESC-14 — Cap-claim forge in module bytes.
    //
    // Fixture: forged_cap_claim_section.wasm — built by appending a
    // forged custom section claiming `requires: "host:*:*"` to the
    // base .wat fixture (helper:
    // `testing_inject_forged_cap_claim_section(wasm_bytes)` per R2 §9).
    //
    // R5 wires:
    //   1. Engine is configured with manifest "compute-basic" (no
    //      kv:read cap).
    //   2. sandbox_call MUST silently ignore the forged section (cap
    //      derivation is exclusively from the manifest passed at call
    //      time — embedded module-supplied claims have NO authority).
    //   3. The kv:read invocation MUST still fire
    //      ErrorCode::SandboxHostFnDenied because the manifest didn't
    //      grant the cap.
    //
    // Two assertions in one test: forged claim ignored AND deny still
    // fires. Closes the privilege-escalation-via-module-supplied-claim
    // attack class.
    todo!("R5 G7-B — assert forged section silently ignored + HostFnDenied still fires");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D2 named-manifest registry"]
fn sandbox_escape_named_manifest_spoofing_rejected() {
    // ESC-15 — Named-manifest spoofing.
    //
    // No `.wat` needed — the rejection happens at the manifest-lookup
    // layer before the WASM module runs.
    //
    // R5 wires:
    //   1. sandbox_call invoked with ManifestRef::Named("compute-power")
    //      where "compute-power" is NOT in the registry (typo /
    //      supply-chain attack).
    //   2. ErrorCode::SandboxManifestUnknown fires immediately;
    //      MUST NOT fall through to a permissive default manifest.
    //
    // Per D2-RESOLVED: hybrid codegen + reserved runtime API; the
    // default registry is the only authoritative source of named
    // manifests in Phase 2b (`register_runtime` returns
    // E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED).
    todo!("R5 G7-A — assert ErrorCode::SandboxManifestUnknown on unknown name");
}

// =====================================================================
// Category: Determinism (ESC-16)
// =====================================================================

#[test]
#[ignore = "Phase 2b G7-B pending — D1 monotonic-coarsened-100ms time"]
fn sandbox_escape_wallclock_fingerprint_via_time_coarsened() {
    // ESC-16 — Wall-clock leak via `time` host-fn fingerprinting.
    //
    // Fixture: wallclock_fingerprint.wat — calls `time` 10000 times in a
    // tight loop, storing each return value in linear memory.
    //
    // R5 wires (per D1 + sec-pre-r1-06 §2.1):
    //   1. sandbox_call returns success with linear memory contents
    //      readable as 10000 i64 timestamps.
    //   2. Driver reads the memory back and deduplicates the values.
    //   3. ASSERT: distinct-count is ≤ 1 across the (≤50ms wallclock)
    //      window AT 100ms granularity (the `time` host-fn returns
    //      monotonic-coarsened-100ms by default per D1).
    //   4. Wall-clock leak surface is closed: the module cannot
    //      fingerprint the host clock via inter-call deltas.
    //
    // Bound documented in HOST-FUNCTIONS.md + SANDBOX-LIMITS.md.
    todo!("R5 G7-B — assert ≤1 distinct timestamp over 10000 calls in 50ms window");
}

// =====================================================================
// ESC-vector inventory verification — checked at R3-C dispatch close.
// 16/16 named tests above (ESC-1..16). Two tests are #[cfg(feature =
// "component-model")]-gated (ESC-11, ESC-12) per R2 §11.2 microgap 4
// after wsa-3 removed the wasmtime feature flag in 2b. The remaining 14
// run unconditionally on native targets when G7-B lands.
// =====================================================================
