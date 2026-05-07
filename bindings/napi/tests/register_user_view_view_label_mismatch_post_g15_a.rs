//! R4-R2 BLOCKER closure pin (r4-r2-ivm-1) — napi-boundary
//! `engine.registerUserView` view-label-mismatch fail-loud preserved
//! post-G15-A generalization (G15-A wave-5a; phase-3-backlog §6.6).
//!
//! ## Lineage
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
//!
//! ## §3.6b end-to-end pin shape (pim-2)
//!
//! Drives the production-grade entry point — `engine.register_user_view`
//! at the napi bridge (`bindings/napi/src/lib.rs::register_user_view`).
//! Asserts an OBSERVABLE behavioral consequence — the typed
//! `E_VIEW_LABEL_MISMATCH` error fires through the napi-rs error-context
//! surface (per G19-B JSON envelope carrier; supersedes the pre-G19-B
//! Phase-2b `$$benten-context$$` sentinel suffix pattern; Instance 8
//! mapNativeError round-trip preserved).
//!
//! Would FAIL if the arm were silently no-op'd (e.g., a refactor that
//! widens acceptance for canonical view ids + mismatched labels post
//! G15-A generalization).
//!
//! Pairs with:
//!   - `crates/benten-engine/tests/register_user_view.rs` (Rust-side
//!     `Engine::register_user_view` engine-boundary pin under the same
//!     naming root).
//!   - `packages/engine/test/views.test.ts::validateUserViewSpec`
//!     fail-loud (TS-side pure-validator pin) — distinct surface; this
//!     pin closes the napi-boundary gap that the TS-DSL pure-validator
//!     does not exercise.
//!   - `crates/benten-ivm/tests/algorithm_b_general.rs::algorithm_b_view_label_mismatch_fail_loud_remains_present`
//!     (kernel-side pin; this napi pin is the cross-language-boundary
//!     companion).
//!
//! ## Recurrence-shape note
//!
//! This is the closure of the 3rd cross-partition-handoff
//! phantom-destination instance (per memory
//! `feedback_3_plus_recurrence_deep_sweep`):
//!   1. Phase-2b R6-R3 `r6-r3-ivm-1` (sentinel-presence pin).
//!   2. Phase-3 R4-FP `ivm-r4-3` (PR #92 → PR #95 routing failure).
//!   3. Phase-3 R3-CPC-3 sibling-package thin-client placement.
//!
//! Reaches the 3+-recurrence threshold; surfaced for provisional
//! pim-12 codification at R6-Phase-3 close (per `r4-r2-ivm-1`
//! recommendation — orchestrator-direct followup).

#![allow(clippy::unwrap_used, dead_code)]

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — ivm-r4-3 / r4-r2-ivm-1 — napi-boundary register_user_view view-label-mismatch fail-loud preserved post-generalization"]
fn napi_register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a()
 {
    // r4-r2-ivm-1 BLOCKER closure (napi-boundary regression pin).
    //
    // G15-A implementer (post-generalization) wires this:
    //
    //   // (i) Construct a napi Engine via the standard test fixture:
    //   let engine = benten_napi::testing::open_in_memory_engine();
    //
    //   // (ii) Call `engine.register_user_view` with a CANONICAL view
    //   //      id + a mismatched label_pattern. Per ivm-major-5 +
    //   //      D-PHASE-3-28: even after G15-A generalizes the kernel,
    //   //      the engine still REJECTS this combination loudly:
    //   let spec = benten_napi::UserViewSpec {
    //       id: "crud:post".to_string(),                  // canonical id
    //       input_pattern: benten_napi::LabelPatternJson::Exact {
    //           label: "user".to_string(),                // mismatch
    //       },
    //       strategy: None,                                // defaults to B
    //       projection: None,
    //   };
    //   let result = engine.register_user_view(spec);
    //
    //   // (iii) Assert the typed E_VIEW_LABEL_MISMATCH error fires
    //   //       through the napi-rs error-context surface. Instance 8
    //   //       (R6 Round-2 r6-r2-napi-3) wired the structured JSON
    //   //       envelope that engine_err emits for structured
    //   //       EngineError variants per G19-B (supersedes the
    //   //       pre-G19-B `$$benten-context$$` sentinel suffix);
    //   //       mapNativeError parses the JSON-shape carrier.
    //   //       The structured-context fields surface here:
    //   match result {
    //       Err(napi_err) => {
    //           let code = benten_napi::error::code_of(&napi_err);
    //           assert_eq!(
    //               code, "E_VIEW_LABEL_MISMATCH",
    //               "napi register_user_view MUST surface typed error \
    //                E_VIEW_LABEL_MISMATCH for canonical id + mismatched \
    //                label post G15-A generalization (ivm-r4-3 BLOCKER)"
    //           );
    //           // Structured context round-trip per Instance 8:
    //           let ctx = benten_napi::error::context_of(&napi_err);
    //           assert_eq!(
    //               ctx.get("view_id").and_then(|v| v.as_str()),
    //               Some("crud:post"),
    //           );
    //           assert_eq!(
    //               ctx.get("expected_label").and_then(|v| v.as_str()),
    //               Some("post"),  // canonical id "crud:post" maps to label "post"
    //           );
    //           assert_eq!(
    //               ctx.get("supplied_label").and_then(|v| v.as_str()),
    //               Some("user"),
    //           );
    //       }
    //       Ok(_) => panic!(
    //           "expected E_VIEW_LABEL_MISMATCH, got Ok — \
    //            napi register_user_view silently widened acceptance \
    //            for canonical id + mismatched label post G15-A; \
    //            ivm-r4-3 BLOCKER regression"
    //       ),
    //   }
    //
    // OBSERVABLE consequence: a refactor that silently widens
    // `register_user_view` acceptance at the napi boundary (e.g., post
    // G15-A generalization that drops the canonical-id label-equality
    // check by mistake) FAILS this test. The napi-rs error-context
    // surface MUST round-trip the typed error code AND the structured
    // context fields; sentinel-presence (constructor name only) does
    // NOT satisfy this pin per pim-2 §3.6b.
    //
    // Companion pins:
    //   - Engine-side: crates/benten-engine/tests/register_user_view.rs
    //   - Kernel-side: crates/benten-ivm/tests/algorithm_b_general.rs
    //                  ::algorithm_b_view_label_mismatch_fail_loud_remains_present
    //   - TS-DSL-side: packages/engine/test/views.test.ts
    //                  ::validateUserViewSpec fail-loud (pure-validator)
    //
    // The TS-DSL pure-validator covers DSL-layer rejection BEFORE the
    // napi boundary; THIS pin covers the napi boundary itself (which
    // a hand-built napi caller — Rust integration test, alternate FFI
    // consumer — would exercise without going through the TS-DSL
    // pre-validation). Both surfaces matter; neither substitutes for
    // the other.
    unimplemented!(
        "G15-A wires napi-boundary register_user_view view-label-mismatch fail-loud \
         (canonical id + mismatched label rejected with typed E_VIEW_LABEL_MISMATCH \
         + structured context round-trip per Instance 8; ivm-r4-3 / r4-r2-ivm-1 closure)"
    );
}
