//! End-to-end ESC runtime-arm closure pins (Phase-3 wave-5c —
//! `docs/future/phase-3-backlog.md §6.1-followup`).
//!
//! These tests drive `benten_eval::sandbox::execute_with_live_cap_check`
//! end-to-end against a real `wasmtime::Module` + `Store` + `Instance`
//! and assert observable typed-error firing per pim-2 §3.6b: a test
//! that drives the production entry point + asserts the typed
//! `SandboxError::EscapeAttempt` (or its peers) routes through
//! `map_call_error` / `EscapeAttemptMarker`. Each test would FAIL if
//! the runtime arm were silently no-op'd (the SHAPE-not-SUBSTANCE
//! pattern that PR #117 carried; the wave-5c PR closes it).
//!
//! ## Coverage
//!
//! - `esc_7_runtime_arm_fires_via_time_host_fn_re_entry_injection`
//!   — closes r1-wsa-1 BLOCKER half-a (ESC-7) end-to-end.
//! - `esc_9_runtime_arm_fires_via_live_cap_check_revoke_mid_call`
//!   — closes r1-wsa-3 MAJOR (ESC-9) end-to-end.
//! - `esc_13_runtime_arm_fires_via_panic_in_host_fn_callback`
//!   — closes r1-wsa-1 BLOCKER half-b (ESC-13) end-to-end.
//! - `esc_16_runtime_arm_fires_after_threshold_time_host_fn_calls`
//!   — closes r1-wsa-4 MAJOR (ESC-16) end-to-end.
//!
//! The test seam at `SandboxConfig::testing_inject_attack` is
//! cfg-gated behind `feature = "test-helpers"`; the production cdylib
//! does not compile the field. The wave-5c production wiring is
//! exercised through the SAME runtime arm a real attack pattern would
//! trigger (the test seam mutates per-call `EscDefenseState` from
//! inside the `time` host-fn trampoline, then `run_all_checks` at the
//! host-fn boundary fires the typed error via
//! `EscapeAttemptMarker`).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]
#![cfg(any(test, feature = "test-helpers", feature = "testing"))]

use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapBundle, EscVector, ManifestRef, ManifestRegistry, SandboxConfig, SandboxError,
    TestEscAttackInjection, execute_with_live_cap_check,
};
use std::sync::{Arc, Mutex};

/// Test attribution frame helper.
fn test_attribution() -> AttributionFrame {
    let zero = benten_core::Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 1,
        ..Default::default()
    }
}

/// A trivial guest module that calls `host:time` once + returns the
/// result. Drives the `time` host-fn trampoline end-to-end.
fn module_calls_time_once() -> Vec<u8> {
    wat::parse_str(
        r#"(module
            (import "host" "time" (func $time (result i64)))
            (func (export "run") (result i64)
                call $time
            )
        )"#,
    )
    .expect("trivial time-calling module compiles")
}

/// Calls `host:time` three times so the 3rd call trips
/// FINGERPRINT_COLLAPSE_THRESHOLD via the engine-side `record_wallclock_write`
/// + `read_collapse_state` chain.
fn module_calls_time_thrice() -> Vec<u8> {
    wat::parse_str(
        r#"(module
            (import "host" "time" (func $time (result i64)))
            (func (export "run") (result i64)
                call $time
                drop
                call $time
                drop
                call $time
            )
        )"#,
    )
    .expect("trivial 3x-time module compiles")
}

/// Calls `host:kv_read` twice — used for the ESC-9 cap-revoke-mid-call
/// drive. The first call succeeds (cap present); the live_cap_check
/// callback then mutates its capture, so the second call observes
/// revocation and surfaces `SandboxError::HostFnDenied`.
fn module_calls_kv_read_twice() -> Vec<u8> {
    // Imports MUST precede locally-defined memory per WAT module-section
    // ordering. We declare memory via import-style (no actual host memory
    // is needed) — wasmtime accepts module-defined memory after imports.
    wat::parse_str(
        r#"(module
            (import "host" "kv_read"
                (func $kvread (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "run") (result i32)
                ;; first call
                i32.const 0    ;; key_ptr
                i32.const 0    ;; key_len
                i32.const 0    ;; out_ptr
                i32.const 0    ;; out_len
                call $kvread
                drop
                ;; second call (denied via revoked cap)
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $kvread
            )
        )"#,
    )
    .expect("kv_read-twice module compiles")
}

#[test]
fn esc_7_runtime_arm_fires_via_time_host_fn_re_entry_injection() {
    // r1-wsa-1 BLOCKER half-a end-to-end pin.
    //
    // The test seam at `SandboxConfig::testing_inject_attack` requests
    // ESC-7 attack-pattern injection. The `time` host-fn trampoline
    // observes the request + bumps `re_entry_count` while
    // `guest_active = true`; `run_all_checks` at the host-fn boundary
    // fires `EscapeAttempt(Esc7FuelRefillViaReEntry)` via
    // `EscapeAttemptMarker`; `map_call_error` unwraps to the typed
    // variant; `Sandbox::execute` returns `Err(EscapeAttempt(...))`.
    //
    // A regression that strips the boundary `run_all_checks` call (or
    // forgets to inject `EscapeAttemptMarker`) would silently return
    // `Ok` here — this pin fails such a regression.
    let bytes = module_calls_time_once();
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
    let attribution = test_attribution();
    let config = SandboxConfig {
        testing_inject_attack: TestEscAttackInjection::Esc7ReEntryAttempt,
        ..SandboxConfig::default()
    };

    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        config,
        &["host:compute:time".to_string()],
        &attribution,
        None,
    )
    .expect_err("ESC-7 attack-pattern injection MUST surface as Err end-to-end");

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
    assert_eq!(
        err.code(),
        ErrorCode::SandboxEscapeAttempt,
        "ESC-7 typed error routes to E_SANDBOX_ESCAPE_ATTEMPT through map_call_error"
    );
}

#[test]
fn esc_9_runtime_arm_fires_via_live_cap_check_revoke_mid_call() {
    // r1-wsa-3 MAJOR end-to-end pin (ESC-9 cap-revoke mid-call).
    //
    // The live_cap_check callback consults a shared mutex containing
    // the revoke-flag. Initially the flag is false — the first
    // `kv_read` call passes the cap check. Between the first and
    // second host-fn invocations the test mutates the flag to true;
    // the second call's PerCall recheck observes the revocation and
    // surfaces `SandboxError::HostFnDenied`.
    //
    // The test simulates the production semantic: an engine-backed
    // callback that observes `revoked_actors` mid-call. A regression
    // that snapshots the cap-set at SANDBOX entry would let call #2
    // silently succeed — this pin fails such a regression.
    let bytes = module_calls_kv_read_twice();
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:kv:read".to_string()], None);
    let attribution = test_attribution();
    let mut config = SandboxConfig::default();
    // Generous fuel so we don't run out before the second kv_read.
    config.fuel = 10_000_000;

    // Shared revoke flag — flipped from `Once` semantics by the
    // callback itself. The callback returns `true` for the first
    // invocation it sees, then flips the flag and returns `false`
    // thereafter (the production analogue: the engine receives a
    // `revoke_capability` call between host-fn dispatches).
    let revoked = Arc::new(Mutex::new(false));
    let revoked_clone = Arc::clone(&revoked);
    let live_cap_check: benten_eval::sandbox::LiveCapCheck = Arc::new(move |cap: &str| -> bool {
        if cap != "host:compute:kv:read" {
            return false;
        }
        let mut g = revoked_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *g {
            // Cap revoked — second + later calls deny.
            return false;
        }
        // First call observes cap present, then flips the flag so
        // the next invocation is denied.
        *g = true;
        true
    });

    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        config,
        &["host:compute:kv:read".to_string()],
        &attribution,
        Some(live_cap_check),
    )
    .expect_err("ESC-9 cap-revoke mid-call MUST surface as Err end-to-end");

    assert!(
        matches!(
            err,
            SandboxError::HostFnDenied { ref cap }
                if cap == "host:compute:kv:read"
        ),
        "ESC-9 end-to-end MUST surface HostFnDenied(kv:read) on the post-revoke call; got {err:?}"
    );
    assert_eq!(
        err.code(),
        ErrorCode::SandboxHostFnDenied,
        "ESC-9 cap-revoke mid-call routes to E_SANDBOX_HOST_FN_DENIED"
    );
    assert!(
        *revoked.lock().unwrap(),
        "callback MUST have observed at least one invocation + flipped the revoke flag"
    );
}

#[test]
fn esc_13_runtime_arm_fires_via_panic_in_host_fn_callback() {
    // r1-wsa-1 BLOCKER half-b end-to-end pin (ESC-13).
    //
    // The test seam requests ESC-13 panic-injection. The `time` host-
    // fn trampoline panics; the `std::panic::catch_unwind` wrapper
    // around `func.call` in `execute_with_live_cap_check` catches the
    // panic + surfaces typed `EscapeAttempt(Esc13StorePoison)`.
    //
    // Pre-wave-5c the panic would unwind through the wasmtime host
    // frames and EITHER abort the process OR poison the Store
    // silently — both catastrophic. The catch_unwind wrapper +
    // typed-error routing + per-call Store lifecycle (D3-RESOLVED)
    // give the panic-recovery path that closes ESC-13.
    let bytes = module_calls_time_once();
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
    let attribution = test_attribution();
    let config = SandboxConfig {
        testing_inject_attack: TestEscAttackInjection::Esc13FuelMeterCallbackTrap,
        ..SandboxConfig::default()
    };

    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        config,
        &["host:compute:time".to_string()],
        &attribution,
        None,
    )
    .expect_err("ESC-13 panic injection MUST surface as Err end-to-end");

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
    assert_eq!(
        err.code(),
        ErrorCode::SandboxEscapeAttempt,
        "ESC-13 typed error routes to E_SANDBOX_ESCAPE_ATTEMPT"
    );
}

#[test]
fn esc_13_recovery_path_next_call_fresh_store_no_poison_leak() {
    // ESC-13 recovery-path pin: per D3-RESOLVED per-call `Store`
    // lifecycle, after an ESC-13 fires, a SUBSEQUENT SANDBOX call
    // gets a FRESH Store + does NOT carry the poisoned flag.
    //
    // A regression that re-uses the poisoned Store across calls
    // would fail this pin.
    let bytes = module_calls_time_once();
    let registry = ManifestRegistry::new();
    let attribution = test_attribution();

    // Call #1: ESC-13 attack-pattern injected. Must fail with ESC-13.
    {
        let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
        let cfg = SandboxConfig {
            testing_inject_attack: TestEscAttackInjection::Esc13FuelMeterCallbackTrap,
            ..SandboxConfig::default()
        };
        let err = execute_with_live_cap_check(
            &bytes,
            ManifestRef::Inline(inline),
            &registry,
            cfg,
            &["host:compute:time".to_string()],
            &attribution,
            None,
        )
        .expect_err("call #1 fires ESC-13");
        assert!(matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc13StorePoison,
                ..
            }
        ));
    }

    // Call #2: NO attack injection. Must succeed (fresh Store; no
    // poison leakage). This proves D3-RESOLVED per-call Store
    // lifecycle survives the panic path.
    let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
    let cfg = SandboxConfig::default();
    let res = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        cfg,
        &["host:compute:time".to_string()],
        &attribution,
        None,
    );
    assert!(
        res.is_ok(),
        "ESC-13 recovery: call #2 with fresh Store MUST succeed (no poison leak); got {res:?}"
    );
}

#[test]
fn esc_16_runtime_arm_fires_after_threshold_time_host_fn_calls() {
    // r1-wsa-4 MAJOR end-to-end pin (ESC-16 fingerprint-collapse).
    //
    // The `time` host-fn trampoline calls
    // `record_wallclock_write` + `read_collapse_state` per invocation.
    // After FINGERPRINT_COLLAPSE_THRESHOLD = 3 calls within a single
    // SANDBOX dispatch, the next host-fn boundary
    // `run_all_checks` fires
    // `EscapeAttempt(Esc16FingerprintCollapse)`.
    //
    // The guest module calls `host:time` 3 times; the 3rd call's
    // boundary check trips the threshold + surfaces the typed error.
    //
    // A regression that strips the `record_wallclock_write` call
    // (or the `read_collapse_state` increment) would silently let
    // this test return Ok — the assertion fails such a regression.
    let bytes = module_calls_time_thrice();
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
    let attribution = test_attribution();
    let config = SandboxConfig::default();

    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        config,
        &["host:compute:time".to_string()],
        &attribution,
        None,
    )
    .expect_err("3x time-call MUST trip ESC-16 threshold end-to-end");

    assert!(
        matches!(
            err,
            SandboxError::EscapeAttempt {
                vector: EscVector::Esc16FingerprintCollapse,
                ..
            }
        ),
        "ESC-16 end-to-end MUST surface EscapeAttempt(Esc16FingerprintCollapse); got {err:?}"
    );
    assert_eq!(
        err.code(),
        ErrorCode::SandboxEscapeAttempt,
        "ESC-16 typed error routes to E_SANDBOX_ESCAPE_ATTEMPT"
    );
}

#[test]
fn esc_16_silent_below_threshold_two_time_calls_pass() {
    // ESC-16 below-threshold pin: 2 `time` calls within one SANDBOX
    // dispatch is BELOW the threshold (3); the call succeeds.
    //
    // A regression that fires ESC-16 on EVERY tainted read (threshold
    // = 1 silent regression) would fail this pin.
    let bytes = wat::parse_str(
        r#"(module
            (import "host" "time" (func $time (result i64)))
            (func (export "run") (result i64)
                call $time
                drop
                call $time
            )
        )"#,
    )
    .expect("2x-time module compiles");
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
    let attribution = test_attribution();
    let config = SandboxConfig::default();

    let res = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        config,
        &["host:compute:time".to_string()],
        &attribution,
        None,
    );
    assert!(
        res.is_ok(),
        "ESC-16 below-threshold (2 time calls) MUST succeed; got {res:?}"
    );
}

#[test]
fn esc_runtime_arms_no_op_when_attack_injection_none() {
    // Production-equivalent path pin: with `testing_inject_attack =
    // None`, NO attack pattern is set, the boundary `run_all_checks`
    // calls observe a clean state, and a single `time` call succeeds.
    //
    // This is the production-shape sanity check: the ESC defenses do
    // NOT fire on legitimate guest behaviour. A regression that
    // false-positives (e.g. fires ESC-7 on EVERY host-fn invocation)
    // would fail this pin.
    let bytes = module_calls_time_once();
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:time".to_string()], None);
    let attribution = test_attribution();
    let config = SandboxConfig::default();

    let res = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        config,
        &["host:compute:time".to_string()],
        &attribution,
        None,
    );
    assert!(
        res.is_ok(),
        "production-equivalent legitimate guest call MUST succeed; got {res:?}"
    );
}
