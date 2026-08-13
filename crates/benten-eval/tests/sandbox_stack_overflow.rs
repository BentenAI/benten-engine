//! `E_SANDBOX_STACK_OVERFLOW` typed-variant cascade pins
//! (G17-A1 wave-5b; r1-wsa-7 MAJOR + phase-3-backlog §6.4).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1):
//!
//! - `tests/sandbox_stack_overflow_routes_to_e_sandbox_stack_overflow_typed_variant`
//!   — r1-wsa-7
//! - `tests/sandbox_recursive_call_overflow_traps_via_dedicated_variant`
//!   — r1-wsa-7
//!
//! ## Cascade shape (r1-wsa-7)
//!
//! Phase-2b routed `wasmtime::Trap::StackOverflow` through a generic
//! `SandboxError::ModuleInvalid` variant, conflating "guest used too
//! much stack" with "guest hit some other trap." r1-wsa-7 pinned the
//! cascade fix:
//!
//! 1. Mint dedicated [`benten_errors::ErrorCode::SandboxStackOverflow`]
//!    + `E_SANDBOX_STACK_OVERFLOW` catalog code per phase-3-backlog
//!    §6.4. **Landed at G17-A1 wave-5b.**
//! 2. Mint dedicated `SandboxError::StackOverflow { max_wasm_stack }`
//!    typed variant in `crates/benten-eval/src/primitives/sandbox.rs`.
//!    **Landed at G17-A1 wave-5b.**
//! 3. Route `Trap::StackOverflow` to the new variant in
//!    `crates/benten-eval/src/sandbox/trap_to_typed.rs`. **Landed at
//!    G17-A1 wave-5b.**
//! 4. Cascade through napi error-mapping at
//!    `bindings/napi/src/error.rs::engine_err`. **Landed via the
//!    existing generic `engine_err` mapping** — the typed variant's
//!    `code()` dispatch surfaces the stable `E_SANDBOX_STACK_OVERFLOW`
//!    string through `format!("{code}: {err}")` automatically; no
//!    per-variant special-case needed.
//!
//! ## Why two distinct pin functions
//!
//! - `..._routes_to_e_sandbox_stack_overflow_typed_variant` is the
//!   "type variant exists + Trap::StackOverflow routes here" pin
//!   (drives the trap_to_typed::map_call_error path with a
//!   synthesized `Trap::StackOverflow` and asserts the dedicated
//!   variant comes out).
//! - `..._traps_via_dedicated_variant` is the cascade-completeness
//!   pin (asserts the napi error mapping + outer-engine error catalog
//!   reach the same dedicated variant). Distinct end-to-end
//!   observable per pim-2 §3.6b.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::SandboxError;
use benten_eval::sandbox::{
    ENGINE_MAX_WASM_STACK_BYTES, MAX_WASM_STACK_DEFAULT, ManifestRef, ManifestRegistry,
    SandboxConfig, engine_max_wasm_stack_bytes, execute,
    trap_to_typed::{MapCallErrorContext, map_call_error},
};

// =====================================================================
// R6 phase-close — the reported ceiling MUST be the enforced ceiling
// =====================================================================
//
// The defect these two tests exist to prevent (found at R6 phase-close,
// ORCH-verified by execution, not by reading):
//
//   `SandboxConfig::max_wasm_stack` is INERT. The guest stack ceiling is
//   set once per process on the `OnceLock<wasmtime::Engine>` singleton
//   (`sandbox/instance.rs::shared_engine`), so a per-call value cannot
//   take effect even in principle. But the executor threaded that
//   per-call value into `MapCallErrorContext`, so
//   `SandboxError::StackOverflow` REPORTED IT. Setting
//   `max_wasm_stack: 4_000_000` and overflowing at the real 512 KiB
//   produced, verbatim:
//
//     SANDBOX stack overflow: guest exceeded max_wasm_stack (4000000 bytes)
//
//   The diagnostic named a limit that was never enforced, and sent an
//   operator hunting a bug that does not exist.
//
// A test that merely asserts `StackOverflow` fires does NOT catch this —
// the pre-existing pins in this file all passed throughout. Catching it
// requires driving a REAL overflow with a config value deliberately
// DIFFERENT from the enforced one, and asserting on the number.

const FIXTURE_DIR: &str = "tests/fixtures/sandbox/escape";

fn recursive_overflow_fixture() -> Vec<u8> {
    let path = format!("{FIXTURE_DIR}/recursive_call_overflow.wat");
    let wat_bytes = std::fs::read(&path).unwrap_or_else(|_| panic!("fixture {path} missing"));
    wat::parse_bytes(&wat_bytes)
        .map_or_else(|e| panic!("fixture {path} parse: {e}"), |c| c.into_owned())
}

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
        ..Default::default()
    }
}

#[test]
fn sandbox_stack_overflow_reports_enforced_ceiling_not_inert_config_request() {
    // END-TO-END falsification guard. Runs a genuinely stack-overflowing
    // guest through the production executor with an `SandboxConfig::
    // max_wasm_stack` that is deliberately WRONG (4 MB vs the enforced
    // 512 KiB), and asserts the operator-visible number is the ENFORCED
    // ceiling.
    //
    // FALSIFYING MUTATION (verified to fail):
    //   in `crates/benten-eval/src/primitives/sandbox.rs`, revert the
    //   `MapCallErrorContext` construction to
    //       max_wasm_stack: config.max_wasm_stack,
    //   => reported value becomes 4_000_000 and BOTH assertions below
    //      fail. This is exactly the pre-R6 shipped behaviour.
    const DELIBERATELY_WRONG_REQUEST: u64 = 4_000_000;
    assert_ne!(
        DELIBERATELY_WRONG_REQUEST,
        engine_max_wasm_stack_bytes(),
        "the guard is only meaningful while the requested value DIFFERS \
         from the enforced one; pick another value if 512 KiB ever moves"
    );

    let bytes = recursive_overflow_fixture();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let mut cfg = SandboxConfig::default();
    // Generous fuel so the STACK path is observed, not the fuel path.
    cfg.fuel = 100_000_000;
    cfg.max_wasm_stack = DELIBERATELY_WRONG_REQUEST;
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .unwrap_err();

    assert_eq!(err.code(), ErrorCode::SandboxStackOverflow);
    let SandboxError::StackOverflow { max_wasm_stack } = err else {
        panic!("expected StackOverflow, got {err:?}");
    };
    assert_eq!(
        max_wasm_stack,
        engine_max_wasm_stack_bytes(),
        "E_SANDBOX_STACK_OVERFLOW MUST report the ENFORCED process-wide \
         ceiling. Reporting the caller's inert `SandboxConfig::max_wasm_stack` \
         request instead names a limit that was never applied and sends \
         operators after a bug that does not exist."
    );
    assert!(
        !err_display_contains(&SandboxError::StackOverflow { max_wasm_stack }, "4000000"),
        "the operator-facing message must not quote the inert request"
    );
}

fn err_display_contains(err: &SandboxError, needle: &str) -> bool {
    err.to_string().contains(needle)
}

#[test]
fn enforced_stack_ceiling_is_a_single_constant_shared_with_the_engine() {
    // Companion structural guard. `MAX_WASM_STACK_DEFAULT` (the value
    // advertised on the public surface + used by every other test in the
    // tree) and `ENGINE_MAX_WASM_STACK_BYTES` (the value actually handed
    // to `Config::max_wasm_stack` on the singleton) were INDEPENDENT
    // `512 * 1024` literals before R6 phase-close and could drift
    // silently. They now derive from one constant.
    //
    // FALSIFYING MUTATION (verified to fail):
    //   in `crates/benten-eval/src/primitives/sandbox.rs`, replace
    //       pub const MAX_WASM_STACK_DEFAULT: u64 =
    //           crate::sandbox::instance::ENGINE_MAX_WASM_STACK_BYTES as u64;
    //   with a bare literal of a different size (e.g. `1024 * 1024`)
    //   => this test fails, re-establishing the drift it forbids.
    assert_eq!(
        MAX_WASM_STACK_DEFAULT, ENGINE_MAX_WASM_STACK_BYTES as u64,
        "the advertised stack ceiling MUST BE the enforced stack ceiling"
    );
    assert_eq!(engine_max_wasm_stack_bytes(), MAX_WASM_STACK_DEFAULT);
}

#[test]
fn sandbox_stack_overflow_routes_to_e_sandbox_stack_overflow_typed_variant() {
    // r1-wsa-7 pin — drive a synthesized `Trap::StackOverflow` through
    // the production trap-routing arm + assert the dedicated typed
    // variant comes out (NOT the legacy generic ModuleInvalid).
    let err = wasmtime::Error::from(wasmtime::Trap::StackOverflow);
    let mapped = map_call_error(
        err,
        MapCallErrorContext {
            consumed_fuel: 0,
            wallclock_limit_ms: 30_000,
            memory_limit_bytes: 64 * 1024 * 1024,
            fuel_limit: 1_000_000,
            max_wasm_stack: MAX_WASM_STACK_DEFAULT,
        },
    );

    assert!(
        matches!(
            mapped,
            SandboxError::StackOverflow {
                max_wasm_stack: MAX_WASM_STACK_DEFAULT
            }
        ),
        "Trap::StackOverflow MUST route to SandboxError::StackOverflow per \
         phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure (got: {mapped:?})"
    );

    // The catalog code is the dedicated `E_SANDBOX_STACK_OVERFLOW`,
    // not `E_SANDBOX_MODULE_INVALID`:
    assert_eq!(mapped.code().as_static_str(), "E_SANDBOX_STACK_OVERFLOW");

    // ERROR-CATALOG.md lists the new code per cascade step 4:
    let catalog = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("ERROR-CATALOG.md"),
    )
    .expect("docs/ERROR-CATALOG.md must exist");
    assert!(
        catalog.contains("E_SANDBOX_STACK_OVERFLOW"),
        "ERROR-CATALOG.md must list E_SANDBOX_STACK_OVERFLOW per phase-3-backlog §6.4 + r1-wsa-7"
    );
}

#[test]
fn sandbox_recursive_call_overflow_traps_via_dedicated_variant() {
    // r1-wsa-7 cascade-completeness pin. The dedicated typed variant
    // is preserved end-to-end through:
    //
    // - `crates/benten-eval/src/sandbox/trap_to_typed.rs` — the
    //   trap-routing arm (cascade step 3).
    // - `crates/benten-eval/src/primitives/sandbox.rs` — the typed
    //   variant lives here (cascade step 2).
    // - `bindings/napi/src/error.rs` — the generic `engine_err`
    //   mapping surfaces the typed variant's catalog code through
    //   the `format!("{code}: {err}")` shape (cascade step 4).
    let trap_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("sandbox")
            .join("trap_to_typed.rs"),
    )
    .expect("trap_to_typed.rs must exist");
    assert!(
        trap_src.contains("Trap::StackOverflow") && trap_src.contains("StackOverflow"),
        "trap_to_typed.rs must route Trap::StackOverflow to the dedicated variant per phase-3-backlog §6.4"
    );
    // The trap arm uses the dedicated SandboxError::StackOverflow
    // variant (not ModuleInvalid):
    assert!(
        trap_src.contains("SandboxError::StackOverflow"),
        "trap_to_typed.rs MUST route Trap::StackOverflow → SandboxError::StackOverflow per r1-wsa-7"
    );

    // The variant is declared in primitives/sandbox.rs:
    let primitives_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("primitives")
            .join("sandbox.rs"),
    )
    .expect("primitives/sandbox.rs must exist");
    assert!(
        primitives_src.contains("StackOverflow"),
        "primitives/sandbox.rs MUST declare the SandboxError::StackOverflow typed variant per phase-3-backlog §6.4"
    );

    // The napi engine_err mapping passes the catalog code through
    // generically (no per-variant special-case needed; post Phase-3
    // G19-B (PR #127) the JSON envelope formatter lives in
    // `bindings/napi/src/error_envelope.rs` — `err.code()` is invoked
    // there to populate the structured `code` field. The cascade is
    // honored via the JSON envelope shape (the production
    // `engine_err` carrier consumes `error_envelope::engine_err_envelope_json`).
    let napi_envelope_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("bindings")
            .join("napi")
            .join("src")
            .join("error_envelope.rs"),
    )
    .expect("bindings/napi/src/error_envelope.rs must exist");
    assert!(
        napi_envelope_src.contains("err.code()"),
        "bindings/napi/src/error_envelope.rs::engine_err_envelope_json MUST surface the typed code via err.code() per the cascade (post G19-B refactor)"
    );
}
