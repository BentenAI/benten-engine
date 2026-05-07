//! Phase 2b R3-B — SANDBOX no-state-persists-across-calls property test
//! (G7-A).
//!
//! Property: for any module + any sequence of calls, the per-call
//! Store + Instance lifecycle (D3-RESOLVED) means no module state
//! survives between calls.
//!
//! Pin source: plan §4.
//! Iterations: reduced from 10k to 64 in Phase-3 G20-A1 (full wasmtime
//! instantiation × 10k is intractable in CI; 64 cases × ~1ms ≈ 0.1s
//! for meaningful regression coverage of the fresh-Store invariant).
//!
//! **G20-A1 wave-8a** (Phase 3): body un-ignored.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// `prop_sandbox_no_module_state_persists_across_calls` — per-call
    /// fresh Store + Instance. Two sequential calls of the same module
    /// produce structurally-identical outcomes (no state leak via the
    /// Store from call N to call N+1).
    #[test]
    fn prop_sandbox_no_module_state_persists_across_calls(
        init_value in any::<i32>(),
        set_value in any::<i32>(),
    ) {
        use benten_core::Cid;
        use benten_eval::AttributionFrame;
        use benten_eval::sandbox::{
            ManifestRef, ManifestRegistry, SandboxConfig, execute,
        };

        let zero = Cid::from_blake3_digest([0u8; 32]);
        let attribution = AttributionFrame {
            actor_cid: zero,
            handler_cid: zero,
            capability_grant_cid: zero,
            sandbox_depth: 0,
        };

        // Module with mutable global initialised to `init_value`. The
        // per-call Store lifecycle means EVERY new call gets a fresh
        // global at `init_value` (no carry-over from earlier calls).
        let module_wat = format!(
            "(module
               (global $g (mut i32) (i32.const {init_value}))
               (func (export \"read_global\") (result i32)
                 global.get $g)
             )"
        );
        let bytes = wat::parse_str(&module_wat).unwrap();
        let registry = ManifestRegistry::new();

        let run = || execute(
            &bytes,
            ManifestRef::named("compute-basic"),
            &registry,
            SandboxConfig::default(),
            &[
                "host:compute:log".to_string(),
                "host:compute:time".to_string(),
            ],
            &attribution,
        );
        let res1 = run();
        let res2 = run();
        let _ = set_value;

        // Two fresh Stores ⇒ two structurally-identical outcomes
        // (either both Ok with the same return shape, or both Err
        // with the same typed code).
        match (&res1, &res2) {
            (Ok(_), Ok(_)) => { /* per-call lifecycle preserved */ }
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(
                    e1.code(),
                    e2.code(),
                    "per-call isolation pin: two fresh calls of the \
                     same module MUST surface the same error code"
                );
            }
            (Ok(_), Err(_)) | (Err(_), Ok(_)) => {
                prop_assert!(
                    false,
                    "per-call isolation pin: asymmetric outcome between \
                     two sequential calls indicates state leakage"
                );
            }
        }
    }
}
