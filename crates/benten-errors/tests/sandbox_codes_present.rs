//! Phase 2b R3-B — SANDBOX error-code catalog presence test (G7-A).
//!
//! Pin source: plan §3 G7-A error catalog list.
//!
//! 12 new error codes added by G7 (per plan §3 G7-A Files-owned cell):
//!   - E_INV_SANDBOX_DEPTH (Inv-4)
//!   - E_INV_SANDBOX_OUTPUT (Inv-7)
//!   - E_SANDBOX_FUEL_EXHAUSTED
//!   - E_SANDBOX_MEMORY_EXHAUSTED
//!   - E_SANDBOX_WALLCLOCK_EXCEEDED (renamed from TIMEOUT per wsa-9/10)
//!   - E_SANDBOX_HOST_FN_DENIED
//!   - E_SANDBOX_HOST_FN_NOT_FOUND
//!   - E_SANDBOX_MANIFEST_UNKNOWN
//!   - E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED (D2 hybrid)
//!   - E_SANDBOX_MODULE_INVALID
//!   - E_SANDBOX_NESTED_DISPATCH_DENIED (renamed from REENTRANCY per
//!     D19 + wsa-7 + r1-security convergence)
//!   - E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED (D20 saturation overflow)
//!
//! Plus E_MODULE_MANIFEST_CID_MISMATCH (D16 minimal CID-pin) — owned by
//! G10-B not G7, but listed in plan §3 G7-A error-catalog narrative.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — sandbox error catalog additions"]
fn e_sandbox_codes_present_in_catalog() {
    // G7-A must-pass — the 12 sandbox error codes above ALL exist in
    // `ErrorCode` enum (benten-errors/src/lib.rs); ALL round-trip via
    // `from_str` / `as_str`; ALL appear in `docs/ERROR-CATALOG.md`.
    //
    // Test pattern (mirrors phase_2a_error_codes_present.rs):
    //   for code_str in &[
    //       "E_INV_SANDBOX_DEPTH",
    //       "E_INV_SANDBOX_OUTPUT",
    //       "E_SANDBOX_FUEL_EXHAUSTED",
    //       "E_SANDBOX_MEMORY_EXHAUSTED",
    //       "E_SANDBOX_WALLCLOCK_EXCEEDED",
    //       "E_SANDBOX_HOST_FN_DENIED",
    //       "E_SANDBOX_HOST_FN_NOT_FOUND",
    //       "E_SANDBOX_MANIFEST_UNKNOWN",
    //       "E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED",
    //       "E_SANDBOX_MODULE_INVALID",
    //       "E_SANDBOX_NESTED_DISPATCH_DENIED",
    //       "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED",
    //   ] {
    //       let parsed = ErrorCode::from_str(code_str);
    //       assert!(!matches!(parsed, ErrorCode::Unknown(_)),
    //               "{code_str} not registered as a stable variant");
    //       assert_eq!(parsed.as_str(), *code_str);
    //   }
    //
    // Anti-rename guard: `E_SANDBOX_REENTRANCY_DENIED` MUST round-trip
    // to `ErrorCode::Unknown(...)` — proves the rename per D19 didn't
    // leave a deprecated alias (CLAUDE.md non-negotiable rule #5).
    todo!("R5 G7-A — assert all 12 codes present + rename clean");
}
