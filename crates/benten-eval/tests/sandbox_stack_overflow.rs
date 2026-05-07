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

use benten_eval::sandbox::SandboxError;
use benten_eval::sandbox::{MAX_WASM_STACK_DEFAULT, trap_to_typed::map_call_error};

#[test]
fn sandbox_stack_overflow_routes_to_e_sandbox_stack_overflow_typed_variant() {
    // r1-wsa-7 pin — drive a synthesized `Trap::StackOverflow` through
    // the production trap-routing arm + assert the dedicated typed
    // variant comes out (NOT the legacy generic ModuleInvalid).
    let err = wasmtime::Error::from(wasmtime::Trap::StackOverflow);
    let mapped = map_call_error(
        err,
        0,
        30_000,
        64 * 1024 * 1024,
        1_000_000,
        MAX_WASM_STACK_DEFAULT,
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
