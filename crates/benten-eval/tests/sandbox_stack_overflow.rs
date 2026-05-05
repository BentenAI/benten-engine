//! R3-D RED-PHASE pins for `E_SANDBOX_STACK_OVERFLOW` typed-variant
//! cascade (G17-A1 wave 5b; r1-wsa-7 MAJOR + phase-3-backlog §6.4).
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
//! `SandboxError::Trap` variant, conflating "guest used too much
//! stack" with "guest hit some other trap." r1-wsa-7 pinned the
//! cascade fix:
//!
//! 1. Mint dedicated `E_SANDBOX_STACK_OVERFLOW` typed variant in
//!    `benten-errors` per phase-3-backlog §6.4.
//! 2. Route `Trap::StackOverflow` to the new variant in
//!    `crates/benten-eval/src/sandbox/trap_to_typed.rs`.
//! 3. Cascade ~20 sites enumerated per r1-wsa-7 (napi error mapping
//!    in `bindings/napi/src/lib.rs` engine_err arm; pinned-CID rebake
//!    hazard for ~3-5 fixture CIDs).
//!
//! ## Why two distinct pin functions
//!
//! - `..._routes_to_e_sandbox_stack_overflow_typed_variant` is the
//!   "type variant exists + is reached" pin — drives a fixture that
//!   recurses past the guest stack ceiling and asserts the typed
//!   variant comes out.
//! - `..._traps_via_dedicated_variant` is the cascade-completeness
//!   pin — asserts the napi mapping + outer-engine error catalog
//!   reach the same dedicated variant (not the legacy
//!   generic-Trap variant). Distinct end-to-end observable per
//!   pim-2 §3.6b.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b mints E_SANDBOX_STACK_OVERFLOW + routes Trap::StackOverflow per phase-3-backlog §6.4"]
fn sandbox_stack_overflow_routes_to_e_sandbox_stack_overflow_typed_variant() {
    // r1-wsa-7 pin. G17-A1 implementer wires this:
    //
    // PRECONDITION — fixture committed:
    //   crates/benten-eval/tests/fixtures/sandbox/recursive_overflow.wat
    //   (a guest fn that recurses indefinitely until stack ceiling)
    //
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig};
    //
    //   let module = load_fixture_wat_or_wasm("recursive_overflow");
    //   let sandbox = Sandbox::new(/* config with low stack ceiling */);
    //   let result = sandbox.execute(module);
    //
    //   // The dedicated typed variant fires:
    //   assert!(matches!(
    //       result.unwrap_err(),
    //       benten_eval::SandboxError::StackOverflow { .. }
    //   ));
    //
    //   // And the typed-error catalog has the dedicated variant:
    //   let catalog = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("docs").join("ERROR-CATALOG.md")
    //   ).unwrap();
    //   assert!(catalog.contains("E_SANDBOX_STACK_OVERFLOW"),
    //       "ERROR-CATALOG.md must list E_SANDBOX_STACK_OVERFLOW per phase-3-backlog §6.4 + r1-wsa-7");
    //
    // OBSERVABLE consequence: stack-overflow attribution distinguishes
    // a benign-but-buggy recursive guest from a malicious escape vector
    // (which would surface as `EscapeAttempt`). Defends pim-2 — typed
    // routing is end-to-end pin-able.
    unimplemented!(
        "G17-A1 wires Trap::StackOverflow → E_SANDBOX_STACK_OVERFLOW + ERROR-CATALOG sweep"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b cascades typed variant through napi + engine-err mapping"]
fn sandbox_recursive_call_overflow_traps_via_dedicated_variant() {
    // r1-wsa-7 cascade-completeness pin. G17-A1 implementer wires:
    //
    //   // Drive the recursive-overflow fixture through the OUTER
    //   // engine API + napi mapping (not just the inner Sandbox API).
    //   // The dedicated variant is preserved end-to-end.
    //
    //   // Outer engine surface assertion:
    //   let engine_err_src = std::fs::read_to_string(
    //       "bindings/napi/src/lib.rs"
    //   ).unwrap();
    //   assert!(engine_err_src.contains("E_SANDBOX_STACK_OVERFLOW")
    //         || engine_err_src.contains("StackOverflow"),
    //       "bindings/napi engine_err mapping must surface the dedicated variant per r1-wsa-7");
    //
    //   // Trap routing assertion:
    //   let trap_src = std::fs::read_to_string(
    //       "crates/benten-eval/src/sandbox/trap_to_typed.rs"
    //   ).unwrap();
    //   assert!(trap_src.contains("StackOverflow"),
    //       "trap_to_typed.rs must route Trap::StackOverflow to the dedicated variant per phase-3-backlog §6.4");
    //
    //   // Fixture CID rebake hazard tracked:
    //   //   ~3-5 fixture CIDs change because the typed variant cascades
    //   //   through ExecutionState envelope. The G17-A1 implementer
    //   //   surfaces the rebake hazard at PR description time.
    //
    // OBSERVABLE consequence: distinct from the previous pin because
    // it asserts cascade completeness through the napi + outer-engine
    // surfaces, not just the Sandbox boundary. A regression that
    // updates the inner variant but leaves napi unmapped fails here.
    unimplemented!("G17-A1 wires napi + trap_to_typed cascade source-cite assertions");
}
