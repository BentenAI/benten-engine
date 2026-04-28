//! Phase 2b R3-B — SANDBOX error-code catalog presence test (G7-A).
//!
//! Pin source: plan §3 G7-A error catalog list.
//!
//! 12 SANDBOX error codes added by G7 (per plan §3 G7-A Files-owned
//! cell). Plus E_MODULE_MANIFEST_CID_MISMATCH (D16 minimal CID-pin) —
//! owned by G10-B but listed in plan §3 G7-A error-catalog narrative.
//!
//! **cr-g7a-mr-1 fix-pass:** test FLIPPED from `#[ignore]` `todo!()`
//! to live assertion. The 12 codes ARE landed in
//! `crates/benten-errors/src/lib.rs` via PR #30 (G7-A) +
//! cross-coordinated with G7-B PR #32 (which also adds
//! `InvSandboxDepth`/`InvSandboxOutput`/`SandboxNestedDispatchDepthExceeded`).
//! G7-A's static defs satisfy this test in isolation; merge-time
//! conflict resolution handled by whichever PR merges second.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;

#[test]
fn e_sandbox_codes_present_in_catalog() {
    let codes = [
        "E_INV_SANDBOX_DEPTH",
        "E_INV_SANDBOX_OUTPUT",
        "E_SANDBOX_FUEL_EXHAUSTED",
        "E_SANDBOX_MEMORY_EXHAUSTED",
        "E_SANDBOX_WALLCLOCK_EXCEEDED",
        "E_SANDBOX_HOST_FN_DENIED",
        "E_SANDBOX_HOST_FN_NOT_FOUND",
        "E_SANDBOX_MANIFEST_UNKNOWN",
        "E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED",
        "E_SANDBOX_MODULE_INVALID",
        "E_SANDBOX_NESTED_DISPATCH_DENIED",
        "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED",
    ];
    for code_str in &codes {
        let parsed = ErrorCode::from_str(code_str);
        assert!(
            !matches!(parsed, ErrorCode::Unknown(_)),
            "{code_str} not registered as a stable variant"
        );
        assert_eq!(parsed.as_static_str(), *code_str);
    }

    // Anti-rename guard — `E_SANDBOX_REENTRANCY_DENIED` MUST round-trip
    // to `ErrorCode::Unknown(...)` (D19 + wsa-7 + r1-security
    // convergence renamed it; no deprecated alias per CLAUDE.md
    // non-negotiable rule #5).
    let stale = ErrorCode::from_str("E_SANDBOX_REENTRANCY_DENIED");
    assert!(
        matches!(stale, ErrorCode::Unknown(_)),
        "E_SANDBOX_REENTRANCY_DENIED MUST be Unknown — D19 + wsa-7 \
         renamed it to E_SANDBOX_NESTED_DISPATCH_DENIED with no alias"
    );

    // Same for the prior TIMEOUT/MEMORY/OUTPUT short forms (cr-g7a-mr-2
    // catalog cleanup target).
    for stale_code in [
        "E_SANDBOX_TIMEOUT",
        "E_SANDBOX_OUTPUT_LIMIT",
        "E_INV_SANDBOX_NESTED",
        "E_SANDBOX_MEMORY", // pre-rename short form
    ] {
        let parsed = ErrorCode::from_str(stale_code);
        assert!(
            matches!(parsed, ErrorCode::Unknown(_)),
            "stale code {stale_code} MUST round-trip to Unknown (cleanup pre-PR drift)"
        );
    }
}
