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

/// **G20-A1 wave-8a body** (Phase 3): ESC-7 fuel-refill defense fires
/// via the test-only attack-pattern injection seam +
/// `execute_with_live_cap_check`. The same runtime arm a real attack
/// would trigger (host-fn dispatch path attempts to re-enter the
/// SANDBOX Store) is exercised by this fixture: the `time` host-fn
/// trampoline observes the seam request, mutates EscDefenseState
/// (`re_entry_count = 1` while `guest_active = true`), and
/// `run_all_checks` at the host-fn boundary fires
/// `EscapeAttempt(Esc7FuelRefillViaReEntry)` BEFORE any inner fuel
/// refill takes effect.
///
/// Cross-ref: companion E2E test at
/// `crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs::esc_7_runtime_arm_fires_via_time_host_fn_re_entry_injection`.
#[test]
fn sandbox_escape_fuel_refill_via_host_fn_denied() {
    use benten_eval::sandbox::{
        EscVector, SandboxError, TestEscAttackInjection, execute_with_live_cap_check,
    };

    let bytes = wat::parse_str(
        r#"(module
            (import "host" "time" (func $time (result i64)))
            (func (export "run") (result i64)
                call $time
            )
        )"#,
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let cfg = SandboxConfig {
        testing_inject_attack: TestEscAttackInjection::Esc7ReEntryAttempt,
        ..SandboxConfig::default()
    };

    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &default_grant(),
        &attribution,
        None,
    )
    .expect_err("ESC-7 attack-pattern injection MUST surface as Err");

    assert!(
        matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc7FuelRefillViaReEntry,
                ..
            }
        ),
        "ESC-7 end-to-end MUST surface EscapeAttempt(Esc7FuelRefillViaReEntry); got {err:?}"
    );
    assert_eq!(err.code(), ErrorCode::SandboxEscapeAttempt);
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

/// **G20-A1 wave-8a body** (Phase 3): ESC-9 cap-revoke mid-call
/// defense. The live_cap_check callback consults a shared revoke
/// flag — first kv_read invocation observes cap present, then flips
/// the flag; second invocation observes revocation +
/// `SandboxError::HostFnDenied` fires.
///
/// Cross-ref: companion E2E test at
/// `crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs::esc_9_runtime_arm_fires_via_live_cap_check_revoke_mid_call`
/// + sibling at
/// `tests/sandbox_capability_check_per_call_after_revoke.rs`.
#[test]
fn sandbox_escape_host_fn_after_cap_revoke() {
    use benten_eval::sandbox::{LiveCapCheck, SandboxError, execute_with_live_cap_check};
    use std::sync::{Arc, Mutex};

    let bytes = wat::parse_str(
        r#"(module
            (import "host" "kv_read"
                (func $kvread (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "run") (result i32)
                ;; first call
                i32.const 0 i32.const 0 i32.const 0 i32.const 0
                call $kvread
                drop
                ;; second call (denied via revoked cap)
                i32.const 0 i32.const 0 i32.const 0 i32.const 0
                call $kvread
            )
        )"#,
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let mut config = SandboxConfig::default();
    config.fuel = 10_000_000;

    // Shared revoke flag. Callback returns true on first observation,
    // then flips the flag; second invocation returns false.
    let revoked = Arc::new(Mutex::new(false));
    let revoked_clone = Arc::clone(&revoked);
    let live_cap_check: LiveCapCheck = Arc::new(move |cap: &str| -> bool {
        if cap != "host:compute:kv:read" {
            return false;
        }
        let mut g = revoked_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *g {
            return false;
        }
        *g = true;
        true
    });

    // Grant covers all three caps the compute-with-kv manifest
    // declares; the live_cap_check callback only flips kv:read so
    // the second kv:read invocation is what surfaces the denial.
    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::named("compute-with-kv"),
        &registry,
        config,
        &[
            "host:compute:kv:read".to_string(),
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
        Some(live_cap_check),
    )
    .expect_err("ESC-9 cap-revoke mid-call MUST surface as Err");

    assert!(
        matches!(err, SandboxError::HostFnDenied { ref cap } if cap == "host:compute:kv:read"),
        "ESC-9 end-to-end MUST surface HostFnDenied(kv:read); got {err:?}"
    );
    assert_eq!(err.code(), ErrorCode::SandboxHostFnDenied);
}

/// **G21-T3 fill** (Phase-3, audit-5 ESC-10 carry): ESC-10 host-fn
/// re-entrancy denial. The structural defense: NO Phase-2b host-fn
/// ships an `Engine::call` re-entry path (D19-RESOLVED).
///
/// The runtime defense for ESC-10 (a host-fn ATTEMPTING nested
/// dispatch) maps to ESC-7 in EscDefenseState (re_entry_count
/// observed during guest execution); the typed error fires at the
/// host-fn boundary via `run_all_checks`.
///
/// **G21-T3 widening:** the eval-side `testing_call_engine_dispatch`
/// helper now drives the EscDefenseState transition the host-fn
/// trampoline would observe, so this test asserts the defense
/// fires end-to-end via the typed `EscapeAttempt` variant rather
/// than only pinning the helper-stub return shape.
#[test]
fn sandbox_escape_reentrancy_via_host_fn_denied() {
    use benten_eval::sandbox::escape_defenses::{EscDefenseState, EscVector, run_all_checks};
    use benten_eval::testing::testing_call_engine_dispatch;

    // Drive the EscDefenseState through the helper. The helper
    // simulates the host-fn trampoline observing a nested-dispatch
    // attempt: enter_guest + re_entry_count bump.
    let mut state = EscDefenseState::new();
    testing_call_engine_dispatch(&mut state);
    assert!(
        state.guest_active,
        "ESC-10 helper MUST flip guest_active=true (matches host-fn \
         trampoline state during re-entry attempt)"
    );
    assert_eq!(
        state.re_entry_count, 1,
        "ESC-10 helper MUST bump re_entry_count to 1 (matches host-\
         fn trampoline re-entry observation)"
    );

    // run_all_checks at the host-fn boundary MUST surface the typed
    // error — the production defense the simulation pins.
    let err =
        run_all_checks(&state).expect_err("ESC-10/ESC-7 simulation MUST trip the runtime defense");
    assert!(
        matches!(
            err,
            benten_eval::sandbox::SandboxError::EscapeAttempt {
                vector: EscVector::Esc7FuelRefillViaReEntry,
                ..
            }
        ),
        "ESC-10 nested-dispatch simulation MUST surface \
         EscapeAttempt(Esc7FuelRefillViaReEntry) — the EscDefenseState \
         vector ESC-10's runtime arm maps to per audit-5; got {err:?}"
    );

    // The typed-error variant exists at the eval boundary: the
    // ErrorCode catalog row is present for the runtime arm to surface
    // when an actual nested dispatch attempt fires.
    let err = benten_eval::sandbox::SandboxError::NestedDispatchDenied;
    assert_eq!(
        err.code(),
        ErrorCode::SandboxNestedDispatchDenied,
        "ESC-10 typed-error variant MUST route to \
         E_SANDBOX_NESTED_DISPATCH_DENIED"
    );
}

// =====================================================================
// Category: Component-Model (ESC-11..12) — gated; current 2b state has
// `component-model` feature removed per wsa-3.
// =====================================================================

#[test]
#[ignore = "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA — Component-Model held cut at Phase-3 R1 per D-PHASE-3-6 RESOLVED-at-R1; rationale rewritten per D-PHASE-3-16 named destination ratified 2026-05-05 (docs/future/phase-3-backlog.md §7.3.A.8 + the Component-Model held-cut decision pin at component_model_phase3_decision_lands_per_d_phase_3_6.rs). Phase-4 pre-R1 inherits this deferral when Phase 4 opens; if wasmtime Component-Model GA-stability changes materially before then, surface for re-evaluation."]
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
#[ignore = "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA — Component-Model held cut at Phase-3 R1 per D-PHASE-3-6 RESOLVED-at-R1; rationale rewritten per D-PHASE-3-16 named destination ratified 2026-05-05 (docs/future/phase-3-backlog.md §7.3.A.8 + the Component-Model held-cut decision pin at component_model_phase3_decision_lands_per_d_phase_3_6.rs). Phase-4 pre-R1 inherits this deferral when Phase 4 opens; if wasmtime Component-Model GA-stability changes materially before then, surface for re-evaluation."]
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

/// **G20-A1 wave-8a body** (Phase 3): ESC-13 trap-in-fuel-callback
/// defense — `EscapeAttempt(Esc13StorePoison)` fires when the
/// fuel-meter callback panics. Exercised via the test-only attack-
/// pattern injection at `SandboxConfig::testing_inject_attack`; the
/// `time` host-fn trampoline panics; `std::panic::catch_unwind`
/// wraps `func.call`, sets `fuel_meter_callback_trapped = true`, and
/// surfaces the typed error.
///
/// Cross-ref: companion E2E test at
/// `crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs::esc_13_runtime_arm_fires_via_panic_in_host_fn_callback`.
#[test]
fn sandbox_escape_trap_in_fuel_callback_denied() {
    use benten_eval::sandbox::{
        EscVector, SandboxError, TestEscAttackInjection, execute_with_live_cap_check,
    };

    let bytes = wat::parse_str(
        r#"(module
            (import "host" "time" (func $time (result i64)))
            (func (export "run") (result i64)
                call $time
            )
        )"#,
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let cfg = SandboxConfig {
        testing_inject_attack: TestEscAttackInjection::Esc13FuelMeterCallbackTrap,
        ..SandboxConfig::default()
    };

    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &default_grant(),
        &attribution,
        None,
    )
    .expect_err("ESC-13 panic injection MUST surface as Err");

    assert!(
        matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc13StorePoison,
                ..
            }
        ),
        "ESC-13 end-to-end MUST surface EscapeAttempt(Esc13StorePoison); got {err:?}"
    );
    assert_eq!(err.code(), ErrorCode::SandboxEscapeAttempt);
}

// =====================================================================
// Category: Cap-system (ESC-14..15)
// =====================================================================

/// **G20-A1 wave-8a body** (Phase 3): ESC-14 — forged cap-claim
/// section in module bytes is silently ignored. The cap derivation
/// path is EXCLUSIVELY the manifest passed at call time. We exercise
/// this by:
///   1. Constructing a wasm module that imports kv_read (manifest
///      `compute-basic` does NOT grant kv:read).
///   2. Asserting the SANDBOX call fails with HostFnDenied / HostFnNotFound
///      regardless of what the module bytes contain — the executor
///      consults the manifest, not the bytes, for cap derivation.
///   3. Source-grep at primitives/sandbox.rs to confirm NO custom-
///      section parsing path exists for cap derivation.
///
/// The dedicated `sandbox_esc14_forged_cap_claim_section.rs` covers
/// the explicit forged-section construction; this file's pin focuses
/// on the structural absence claim.
#[test]
fn sandbox_escape_forged_cap_claim_section_ignored() {
    let bytes = wat::parse_str(
        r#"(module
            (import "host" "kv_read"
                (func $kvread (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "run") (result i32)
                i32.const 0 i32.const 0 i32.const 0 i32.const 0
                call $kvread
            )
        )"#,
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        // compute-basic does NOT grant kv:read.
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &default_grant(),
        &attribution,
    )
    .expect_err("forged cap-claim attempt MUST be denied");
    assert!(
        matches!(
            err.code(),
            ErrorCode::SandboxHostFnDenied | ErrorCode::SandboxHostFnNotFound
        ),
        "ESC-14: cap derivation EXCLUSIVELY from manifest; module \
         bytes have no cap authority; got {:?}",
        err.code()
    );

    // Structural pin: primitives/sandbox.rs MUST NOT parse custom
    // sections for cap derivation. Source-grep at the executor.
    let exec_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("primitives")
            .join("sandbox.rs"),
    )
    .expect("benten-eval/src/primitives/sandbox.rs must be readable");
    assert!(
        !exec_src.contains("custom_section")
            && !exec_src.contains("read_custom_section")
            && !exec_src.contains("module.custom_sections"),
        "ESC-14 absence pin: the SANDBOX executor MUST NOT parse \
         custom sections for cap derivation (cap-claim forge defense)"
    );
}

/// **G21-T3 fill** (Phase-3, audit-5 ESC-14 carry): drive the
/// ESC-14 forged-cap-claim defense via the eval-side helper
/// `testing_inject_forged_cap_claim_section` (filled at G21-T3
/// per audit-5 disposition; previously paper-only marker).
///
/// The helper appends a custom section naming a `host:*:*`-style
/// forged claim onto well-formed wasm bytes. The dispatch then
/// asserts the engine STILL refuses kv:read access — proving cap
/// derivation is EXCLUSIVELY from the call-time manifest, not from
/// embedded bytes. Sibling to the inline-helper variant at
/// `sandbox_esc14_forged_cap_claim_section.rs::
/// engine_silently_ignores_forged_cap_claim_custom_section`.
#[test]
fn sandbox_escape_forged_cap_claim_section_helper_driven() {
    use benten_eval::testing::testing_inject_forged_cap_claim_section;

    // Well-formed module that imports kv_read; manifest "compute-
    // basic" does NOT grant kv:read.
    let bytes = wat::parse_str(
        r#"(module
            (import "host" "kv_read"
                (func $kvread (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "run") (result i32)
                i32.const 0 i32.const 0 i32.const 0 i32.const 0
                call $kvread
            )
        )"#,
    )
    .unwrap();

    // Inject the forged cap-claim section via the helper.
    let forged_bytes = testing_inject_forged_cap_claim_section(&bytes, "requires:host:*:*");
    assert!(
        forged_bytes.len() > bytes.len(),
        "ESC-14 helper MUST extend the bytes with the forged section"
    );
    assert!(
        forged_bytes.starts_with(&bytes),
        "ESC-14 helper MUST preserve the original module bytes verbatim \
         (forge appends a trailing custom section)"
    );

    // Drive the forged module through SANDBOX execution. The engine
    // MUST consult the call-time manifest exclusively — the embedded
    // forged claim has zero cap authority.
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let err = execute(
        &forged_bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &default_grant(),
        &attribution,
    )
    .expect_err("forged cap-claim section MUST be denied (manifest is authoritative)");
    assert!(
        matches!(
            err.code(),
            ErrorCode::SandboxHostFnDenied | ErrorCode::SandboxHostFnNotFound
        ),
        "ESC-14 helper-driven: cap derivation EXCLUSIVELY from manifest; \
         forged custom section MUST NOT widen caps; got {:?}",
        err.code()
    );
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
