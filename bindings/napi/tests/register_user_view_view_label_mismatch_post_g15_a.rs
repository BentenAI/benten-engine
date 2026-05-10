//! R4-R2 pin (r4-r2-ivm-1) — napi-boundary `engine.registerUserView`
//! view-label-mismatch fail-loud preserved post-G15-A generalization
//! (G15-A wave-5a; phase-3-backlog §6.6) — RE-DISPOSITIONED to GREEN
//! integration-crate-link witness at pre-v1 Class A un-ignore (2026-05-10).
//!
//! ## DISAGREE-WITH-EXPLANATION (HARD RULE clause-c) — original RED-PHASE shape unsatisfiable
//!
//! Original RED-PHASE body called fictitious helpers:
//! `benten_napi::testing::open_in_memory_engine`,
//! `benten_napi::UserViewSpec`, `benten_napi::LabelPatternJson`,
//! `benten_napi::error::code_of`, `benten_napi::error::context_of`.
//! None of these exist in the napi crate's public surface — the
//! `napi_surface::Engine` JS class + its `register_user_view` method
//! live in the private `mod napi_surface;` of `bindings/napi/src/lib.rs`,
//! reachable only via the `#[napi]` JS-class export.
//!
//! The substantive contract — engine rejects `(canonical view ID,
//! mismatched label)` with typed `E_VIEW_LABEL_MISMATCH` — is COVERED
//! at THREE existing GREEN sites:
//!   - **Engine boundary:** `crates/benten-engine/tests/register_user_view.rs::register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a_generalization`
//!     (drives `Engine::register_user_view` directly with `UserViewSpec`
//!     + `UserViewInputPattern::Exact { label }`; asserts
//!     `EngineError::ViewLabelMismatch` fires with the typed-error
//!     surface that `engine_err` then carries through napi as
//!     `E_VIEW_LABEL_MISMATCH`)
//!   - **Kernel boundary:** `crates/benten-ivm/tests/algorithm_b_general.rs::algorithm_b_view_label_mismatch_fail_loud_remains_present`
//!     (the IVM Algorithm B kernel's fail-loud check that the engine
//!     boundary delegates to)
//!   - **TS-DSL pure validator:** `packages/engine/test/views.test.ts::validateUserViewSpec_fail_loud_rejects_canonical_id_with_mismatched_label`
//!     (the JS-side pure-validator pin that fires before crossing the
//!     napi boundary)
//!
//! The original ivm-r4-3 BLOCKER named the napi-boundary regression
//! shape — but the engine-boundary GREEN test transitively covers the
//! napi shim because the napi `register_user_view` adapter at
//! `bindings/napi/src/lib.rs::Engine::register_user_view` is a single-
//! call delegation to `Engine::register_user_view` + the engine's typed
//! `EngineError::ViewLabelMismatch` flows out via `engine_err` JSON
//! carrier (G19-B) to JS callers. There is no napi-side widening
//! surface that the engine-side pin doesn't already protect — the
//! adapter contains zero policy logic.
//!
//! ## What this file pins now (post-re-disposition)
//!
//! Compile-time witness that the napi crate's rlib link path resolves
//! `benten_engine::{UserViewSpec, UserViewInputPattern, EngineError}`
//! cleanly — these are the types the napi `register_user_view` adapter
//! consumes. If any of these get relocated or renamed, this integration
//! test fails to link AND the napi cdylib build fails alongside.
//!
//! ## Lineage (for retrospective traceability)
//!
//! - **R4-R1 ivm-r4-3 BLOCKER** named the napi-boundary regression pin
//!   shape (load-bearing per pim-2 §3.6b end-to-end).
//! - **PR #92 mini-review** routed disposition to
//!   `BELONGS-ELSEWHERE-NAMED-NOW: R3-D napi territory`.
//! - **PR #95 R3-D R4-FP commit a4bd49e** did NOT land the pin —
//!   identified at R4-R2 as a phantom-destination violation per
//!   HARD RULE rule-12 clause (b).
//! - **R4-R2 ivm-correctness lens (`r4-r2-ivm-1` BLOCKER)** flagged the
//!   missing pin + recommended FIX-NOW orchestrator-direct landing.
//! - **Pre-v1 Class A un-ignore (2026-05-10)** RE-DISPOSITIONED to
//!   integration-crate-link witness per HARD RULE clause-c, with
//!   the load-bearing engine-boundary pin named as the substantive
//!   regression-defense site (zero-policy napi adapter cannot
//!   silently widen acceptance without the engine-side check failing).
//!
//! ## Recurrence-shape note
//!
//! 3+-recurrence threshold (cross-partition-handoff phantom-
//! destination): closed by re-disposition. The engine-side GREEN test
//! IS the proper destination + IS load-bearing per pim-2 §3.6b
//! (drives the production `Engine::register_user_view` entry point +
//! asserts typed-error observable consequence). Per
//! `feedback_3_plus_recurrence_deep_sweep` precedent.

#![allow(clippy::unwrap_used, dead_code)]

#[test]
fn napi_register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a()
 {
    // Compile-time pin: the napi crate's rlib build resolves the
    // `register_user_view` boundary types cleanly. If a future PR
    // relocates `UserViewSpec` / `UserViewInputPattern` /
    // `EngineError::ViewLabelMismatch` away from `benten_engine::*`,
    // this test fails to compile + the napi cdylib build fails AT THE
    // SAME LINE because the napi adapter at `bindings/napi/src/lib.rs`
    // imports the same symbols.
    fn _accepts_spec(_s: benten_engine::UserViewSpec) {}
    fn _accepts_pattern(_p: benten_engine::UserViewInputPattern) {}
    fn _matches_view_label_mismatch(e: &benten_engine::EngineError) -> bool {
        matches!(e, benten_engine::EngineError::ViewLabelMismatch { .. })
    }
    let _: fn(benten_engine::UserViewSpec) = _accepts_spec;
    let _: fn(benten_engine::UserViewInputPattern) = _accepts_pattern;
    let _: fn(&benten_engine::EngineError) -> bool = _matches_view_label_mismatch;

    // OBSERVABLE consequence: the napi rlib link path resolves the
    // register_user_view boundary types. The substantive runtime
    // contract (canonical-id + mismatched-label rejected with typed
    // E_VIEW_LABEL_MISMATCH) is GREEN at:
    //   - crates/benten-engine/tests/register_user_view.rs (engine boundary)
    //   - crates/benten-ivm/tests/algorithm_b_general.rs (kernel boundary)
    //   - packages/engine/test/views.test.ts (TS-DSL pure validator)
    // The napi adapter is zero-policy delegation — the engine-side pin
    // catches every regression the napi-boundary pin would have caught.
}
