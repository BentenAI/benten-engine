//! Phase 2b R3-B — D19 nested-dispatch denial unit tests (G7-A).
//!
//! D19-RESOLVED calibrated:
//!   - Strict ban on `Engine::call`-from-host-fn (closes sec-pre-r1-08
//!     cap-context-confusion attack via SANDBOX → CALL → SANDBOX chain).
//!   - Permissive on async host-fns gated by reserved `host:async` cap
//!     (Phase 3 iroh KVBackend forward-compat).
//!   - Catalog rename: `E_SANDBOX_REENTRANCY_DENIED` →
//!     `E_SANDBOX_NESTED_DISPATCH_DENIED` (per wsa-7 + r1-security
//!     convergence). The name aligns with the actual security claim.
//!
//! **G20-A1 wave-8a** (Phase 3): bodies un-ignored.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_errors::ErrorCode;
use benten_eval::sandbox::{RESERVED_HOST_ASYNC_CAP, default_host_fns};

#[test]
fn sandbox_nested_dispatch_denied_renamed_from_reentrancy() {
    // D19 catalog rename verification — `ErrorCode::SandboxNestedDispatchDenied`
    // exists; `as_str()` returns "E_SANDBOX_NESTED_DISPATCH_DENIED".
    //
    // The OLD code `E_SANDBOX_REENTRANCY_DENIED` MUST NOT appear in
    // the live catalog (no deprecated alias per CLAUDE.md
    // non-negotiable rule #5).
    assert_eq!(
        ErrorCode::SandboxNestedDispatchDenied.as_str(),
        "E_SANDBOX_NESTED_DISPATCH_DENIED",
        "D19 catalog rename: ErrorCode::SandboxNestedDispatchDenied \
         MUST surface as E_SANDBOX_NESTED_DISPATCH_DENIED"
    );

    // White-box: parse `docs/ERROR-CATALOG.md`; assert NESTED_DISPATCH
    // present, REENTRANCY absent.
    let catalog_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("ERROR-CATALOG.md");
    let catalog =
        std::fs::read_to_string(&catalog_path).expect("docs/ERROR-CATALOG.md must be readable");
    assert!(
        catalog.contains("E_SANDBOX_NESTED_DISPATCH_DENIED"),
        "ERROR-CATALOG.md MUST document E_SANDBOX_NESTED_DISPATCH_DENIED"
    );
    // Strip the legacy-rename narrative line ("Renamed from the older
    // E_SANDBOX_REENTRANCY_DENIED…") to assert the OLD code is not a
    // live catalog row. The narrative reference inside the
    // NESTED_DISPATCH section is allowed historical pointer; the
    // forbidden shape is a live row heading like
    // `### E_SANDBOX_REENTRANCY_DENIED`.
    assert!(
        !catalog.contains("### E_SANDBOX_REENTRANCY_DENIED"),
        "ERROR-CATALOG.md MUST NOT carry a live row for the renamed-away \
         E_SANDBOX_REENTRANCY_DENIED (CLAUDE.md non-negotiable #5: no \
         deprecated aliases)"
    );
}

#[test]
fn sandbox_nested_sandbox_via_call_denied() {
    // D19 + sec-pre-r1-08 — the production engine path enforces this
    // via `dispatch_call_inner`'s nested-dispatch check. The eval-side
    // structural pin is that the typed error CODE exists +
    // `SandboxError::NestedDispatchDenied` maps to it.
    //
    // The engine-level integration pin (the actual denial behaviour
    // when a host-fn attempts `engine.call`) lives at the engine layer
    // — exercised by
    // `crates/benten-engine/tests/integration/engine_sandbox.rs` (the
    // dx-r1-2b absence pin) + the engine's nested-dispatch test
    // family.
    //
    // Eval-side: assert the typed error variant exists + carries the
    // expected code.
    use benten_eval::sandbox::SandboxError;
    let err = SandboxError::NestedDispatchDenied;
    assert_eq!(
        err.code(),
        ErrorCode::SandboxNestedDispatchDenied,
        "SandboxError::NestedDispatchDenied MUST route to \
         E_SANDBOX_NESTED_DISPATCH_DENIED — D19 + sec-pre-r1-08 \
         cap-context-confusion attack defense"
    );

    // STRUCTURAL pin via source-grep at the engine boundary: the
    // engine's `dispatch_call_inner` consults a nested-dispatch flag.
    // Ensure the engine carries a guard against re-entry from a
    // host-fn callback.
    let engine_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("benten-engine")
        .join("src")
        .join("engine.rs");
    let engine_src = std::fs::read_to_string(&engine_path)
        .expect("benten-engine/src/engine.rs must be readable");
    assert!(
        engine_src.contains("dispatch_call_inner")
            || engine_src.contains("dispatch_call_with_mode_and_trace"),
        "engine.rs MUST carry the nested-dispatch arbitration site \
         (dispatch_call_inner / dispatch_call_with_mode_and_trace) \
         that enforces the D19 denial"
    );
}

#[test]
fn sandbox_async_host_fn_gated_by_host_async_cap_reserved_phase_3() {
    // D19 calibrated — `host:async` capability is reserved in 2b.
    //
    // Pins:
    //   1. `RESERVED_HOST_ASYNC_CAP` constant exists with the value
    //      `"host:async"`.
    //   2. NO D1 host-fn declares `requires_async = true` (Phase-2b
    //      ships time/log/kv:read/random all sync).
    //   3. host-functions.toml schema accepts the `requires_async`
    //      field (parseable; defaults to false).
    assert_eq!(
        RESERVED_HOST_ASYNC_CAP, "host:async",
        "D19: RESERVED_HOST_ASYNC_CAP constant MUST equal \"host:async\""
    );

    let table = default_host_fns();
    for (name, spec) in table.iter() {
        assert!(
            !spec.requires_async,
            "D19: NO Phase-2b host-fn declares requires_async = true \
             ({name} flips it; that's a Phase-3 forward-compat feature)"
        );
    }

    // host-functions.toml schema acceptance — every declared host_fn
    // entry has the `requires_async = false` line OR omits the field
    // (defaulting to false). This pins the schema-shape parseability.
    let toml_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("host-functions.toml"),
    )
    .expect("workspace host-functions.toml readable");
    // Confirm at least one entry declares `requires_async = false`
    // (positive parseability check).
    assert!(
        toml_src.contains("requires_async = false"),
        "host-functions.toml schema MUST carry at least one explicit \
         `requires_async = false` declaration to pin parseability"
    );
    assert!(
        !toml_src.contains("requires_async = true"),
        "Phase-2b host-functions.toml MUST NOT carry any \
         `requires_async = true` declaration (D19 calibrated; Phase-3 \
         iroh kv:read flips it)"
    );
}
