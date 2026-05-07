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
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

const FIXTURE_DIR: &str = "tests/fixtures/sandbox/escape";

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
    }
}

fn load_fixture(name: &str) -> Vec<u8> {
    let path = format!("{FIXTURE_DIR}/{name}");
    let wat_bytes = std::fs::read(&path).unwrap_or_else(|_| panic!("fixture {path} missing"));
    wat::parse_bytes(&wat_bytes)
        .map_or_else(|e| panic!("fixture {path} parse: {e}"), |c| c.into_owned())
}

fn default_grant() -> Vec<String> {
    vec![
        "host:compute:log".to_string(),
        "host:compute:time".to_string(),
    ]
}

fn run_with_default(
    bytes: &[u8],
) -> Result<benten_eval::sandbox::SandboxResult, benten_eval::sandbox::SandboxError> {
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    execute(
        bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &default_grant(),
        &attribution,
    )
}

// =====================================================================
// Category: Memory (ESC-1..3)
// =====================================================================

#[test]
fn sandbox_escape_oob_linmem_read_traps() {
    // ESC-1 — OOB load surfaces as SandboxModuleInvalid (wasmtime trap
    // mapped via trap_to_typed).
    let bytes = load_fixture("oob_linmem_read.wat");
    let err = run_with_default(&bytes).unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxModuleInvalid);
}

#[test]
fn sandbox_escape_linmem_grow_to_limit_kills() {
    // ESC-2 — memory.grow loop exceeds per-call cap; ResourceLimiter
    // raises MemoryCapExceededMarker → SandboxError::MemoryExhausted.
    //
    // Wave-8d-narrative: the committed `linmem_grow_to_limit.wat`
    // fixture was re-authored to compile under wasmtime 43 (the
    // original used `br_if 1` outside a containing block which carried
    // a value into a no-result-type loop and failed to compile). The
    // re-authored shape wraps the loop in `(block $done (result i32))`
    // so the limiter-trip branch carries the iteration count out via
    // `br $done`. The test now exercises the committed fixture
    // directly rather than the inline-built equivalent wave-8b used as
    // a workaround.
    let bytes = load_fixture("linmem_grow_to_limit.wat");
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let cfg = SandboxConfig {
        memory_bytes: 1024 * 1024, // 1 MiB cap; loop grows by 1 page per iter
        fuel: 100_000_000,
        wallclock_ms: 60_000,
        ..SandboxConfig::default()
    };
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &default_grant(),
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxMemoryExhausted);
}

#[test]
fn sandbox_escape_host_buf_overrun_rejected() {
    // ESC-3 — pathological out_len passed to kv_read; the trampoline's
    // bounds check fires SandboxModuleInvalid (via Trap::MemoryOutOfBounds).
    let bytes = load_fixture("host_buf_overrun.wat");
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        ManifestRef::named("compute-with-kv"),
        &registry,
        SandboxConfig::default(),
        &[
            "host:compute:kv:read".to_string(),
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxModuleInvalid);
}

// =====================================================================
// Category: Control-flow (ESC-4..5)
// =====================================================================

#[test]
fn sandbox_escape_infinite_loop_fuel_bound() {
    // ESC-4 — infinite loop fuel-bound.
    let bytes = load_fixture("infinite_loop.wat");
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let cfg = SandboxConfig {
        fuel: 50_000,
        wallclock_ms: 60_000,
        ..SandboxConfig::default()
    };
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &default_grant(),
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxFuelExhausted);
}

#[test]
fn sandbox_escape_recursive_call_overflow_traps() {
    // ESC-5 — recursive overflow → wasmtime StackOverflow trap →
    // dedicated SandboxStackOverflow typed variant via trap_to_typed
    // (Phase-3 G17-A1 wave-5b mint per r1-wsa-7).
    let bytes = load_fixture("recursive_call_overflow.wat");
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    // Generous fuel so the stack-overflow path is observed (not fuel
    // path).
    let cfg = SandboxConfig {
        fuel: 100_000_000,
        ..SandboxConfig::default()
    };
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &default_grant(),
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxStackOverflow);
}

// =====================================================================
// Category: Fuel (ESC-6..7)
// =====================================================================

#[test]
fn sandbox_escape_fuel_overflow_regression_held() {
    // ESC-6 — fuel-counter overflow regression: a long-running arith
    // loop trips the fuel budget regardless of how many iterations.
    // The fixture's loop terminates only when `i64.gt_s` against 0
    // returns false (which is never given the strictly-positive
    // increment). Fuel exhaustion fires within the configured budget.
    let bytes = load_fixture("fuel_overflow_regression.wat");
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let cfg = SandboxConfig {
        fuel: 100_000,
        wallclock_ms: 60_000,
        ..SandboxConfig::default()
    };
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &default_grant(),
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxFuelExhausted);
}

#[test]
#[ignore = "Phase 3 — ESC-7 fuel-refill via host-fn re-entry body deferred per docs/future/phase-3-backlog.md §7.3.A.7 (security-critical; cross-ref SECURITY-POSTURE.md ESC matrix + Compromise #4 honest disclosure)"]
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
fn sandbox_escape_host_fn_not_on_manifest() {
    // ESC-8 — module imports kv_read; manifest "compute-basic" only
    // covers time+log, so kv_read is NOT registered in the linker for
    // this primitive call. wasmtime raises an "unknown import" error
    // which the executor maps to SandboxHostFnNotFound (preferred per
    // ESC-8 inventory).
    let bytes = load_fixture("host_fn_not_on_manifest.wat");
    let err = run_with_default(&bytes).unwrap_err();
    assert!(
        matches!(
            err.code(),
            ErrorCode::SandboxHostFnNotFound | ErrorCode::SandboxHostFnDenied
        ),
        "ESC-8 MUST route to NotFound or Denied; got {:?}",
        err.code()
    );
}

#[test]
#[ignore = "Phase 3 — ESC-9 host-fn-after-cap-revoke body deferred per docs/future/phase-3-backlog.md §7.3.A.7 (testing_revoke_cap_mid_call helper; cross-ref SECURITY-POSTURE.md ESC matrix entry for ESC-9 + Compromise #4 honest disclosure)"]
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
#[ignore = "Phase 3 — ESC-10 host-fn-reentrancy-denied body deferred per docs/future/phase-3-backlog.md §7.3.A.7 (testing_call_engine_dispatch helper; cross-ref SECURITY-POSTURE.md ESC matrix entry for ESC-10 + Compromise #4 honest disclosure)"]
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
#[ignore = "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA — Component-Model held cut at Phase-3 R1 per D-PHASE-3-6 RESOLVED-at-R1; rationale rewritten per D-PHASE-3-16 named destination ratified 2026-05-05 (docs/FULL-ROADMAP.md Phase 4 entry naming wasmtime Component-Model re-evaluation). Phase-4 pre-R1 inherits this deferral when Phase 4 opens; if wasmtime Component-Model GA-stability changes materially before then, surface for re-evaluation."]
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
#[ignore = "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA — Component-Model held cut at Phase-3 R1 per D-PHASE-3-6 RESOLVED-at-R1; rationale rewritten per D-PHASE-3-16 named destination ratified 2026-05-05 (docs/FULL-ROADMAP.md Phase 4 entry naming wasmtime Component-Model re-evaluation). Phase-4 pre-R1 inherits this deferral when Phase 4 opens; if wasmtime Component-Model GA-stability changes materially before then, surface for re-evaluation."]
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
#[ignore = "Phase 3 — ESC-13 trap-in-fuel-callback Store-poison body deferred per docs/future/phase-3-backlog.md §7.3.A.7 (custom test driver lives at engine layer; cross-ref SECURITY-POSTURE.md ESC matrix entry for ESC-13 + Compromise #4)"]
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
#[ignore = "Phase 3 — ESC-14/-15 forged-cap-claim-section ignored body deferred per docs/future/phase-3-backlog.md §7.3.A.7 (testing_inject_forged_cap_claim_section helper; cross-ref SECURITY-POSTURE.md ESC matrix + Compromise #4 honest disclosure)"]
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
fn sandbox_escape_named_manifest_spoofing_rejected() {
    // ESC-15 — unknown manifest name fires SandboxManifestUnknown.
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))").unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        ManifestRef::named("compute-power"),
        &registry,
        SandboxConfig::default(),
        &default_grant(),
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxManifestUnknown);
}

#[allow(dead_code)]
fn _esc_15_unused_marker_old() {
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
fn sandbox_escape_wallclock_fingerprint_via_time_coarsened() {
    // ESC-16 end-to-end — the trampoline's `time` host-fn returns
    // module-relative monotonic ms coarsened to 100ms (the host-fn-
    // level defense; pinned at
    // `sandbox_host_fn_time.rs::sandbox_host_fn_time_returns_monotonic_coarsened_100ms`),
    // AND **Phase-3 wave-5c §6.1-followup task #2 + r1-wsa-4 MAJOR
    // closure**: repeated `time` calls within a single SANDBOX
    // dispatch trip the engine-side fingerprint-collapse defense
    // (`FINGERPRINT_COLLAPSE_THRESHOLD = 3`). The wallclock-correlated
    // memory-cell side-table is populated by `record_wallclock_write`
    // on each `time` invocation; `read_collapse_state` increments the
    // per-call counter; `run_all_checks` at the host-fn boundary
    // surfaces `SandboxError::EscapeAttempt(Esc16FingerprintCollapse)`
    // BEFORE the side-channel becomes guest-observable.
    //
    // Pre-wave-5c this test asserted the 1000-call loop succeeded
    // under default budget (no defense was wired). Post-wave-5c: the
    // 1000-call loop trips the defense at the 3rd call; the typed
    // error fires. Calling `time` >= 3 times within one SANDBOX
    // dispatch IS the fingerprint-collapse pattern (per the threshold
    // rationale in `crates/benten-eval/src/sandbox/fingerprint.rs`).
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"time\" (func $time (result i64)))
           (memory (export \"memory\") 4)
           (func (export \"run\") (result i32)
             (local $i i32)
             (loop $L
               call $time
               drop
               local.get $i
               i32.const 1
               i32.add
               local.tee $i
               i32.const 1000
               i32.lt_s
               br_if $L
             )
             local.get $i
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let res = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &default_grant(),
        &attribution,
    );
    let err = res.expect_err(
        "ESC-16 wave-5c: 1000 `time` calls in one SANDBOX dispatch \
         MUST trip the fingerprint-collapse defense end-to-end",
    );
    assert!(
        matches!(
            err,
            benten_eval::sandbox::SandboxError::EscapeAttempt {
                vector: benten_eval::sandbox::EscVector::Esc16FingerprintCollapse,
                ..
            }
        ),
        "ESC-16 end-to-end MUST surface EscapeAttempt(Esc16FingerprintCollapse); got {err:?}"
    );
}

#[allow(dead_code)]
fn _esc_16_unused_marker_old() {
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
