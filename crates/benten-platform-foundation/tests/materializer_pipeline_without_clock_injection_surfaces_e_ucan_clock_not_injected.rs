//! R3 Family E RED-PHASE pin: materializer wallclock fail-closed inheritance
//! per sec-3.5-r1-7 + threat-model T11 (LOAD-BEARING substantive).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 9.
//! - sec-3.5-r1-7 wallclock fail-closed inheritance (Compromise #24 floor).
//! - Threat-model §3 T11 (wallclock-injection-bypass).
//!
//! ## Why the materializer inherits the fail-closed floor
//!
//! The materializer's walk invokes UCAN cap-policy evaluation at every
//! READ fanout. UCAN time-window checks REQUIRE an injected clock per
//! `E_UCAN_CLOCK_NOT_INJECTED`. If the materializer is constructed without
//! a clock injection, the walk MUST fail-closed (NOT silently default to
//! `now()`) per Compromise #24 + Phase-3 G16-B-B closure floor.
//!
//! Negative pin: construct a materializer with no clock + drive a walk
//! whose READ fanout would consult a time-bounded UCAN; verify the error
//! returned carries `ErrorCode::UcanClockNotInjected`.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer fail-closed-no-clock pathway doesn't exist at HEAD; G23-B wave-5 wires \
    UCAN clock-injection inheritance through HtmlJsonMaterializer construction. Closes \
    r2-test-landscape §2.5 row 9 + sec-3.5-r1-7 + T11."]
fn materializer_pipeline_without_clock_injection_surfaces_e_ucan_clock_not_injected() {
    // G23-B implementer wires this:
    //
    //   use benten_engine::Engine;
    //   use benten_engine::EngineError;
    //   use benten_errors::ErrorCode;
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //
    //   // Engine WITHOUT clock injection (skip Engine::open_with_clock).
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    //
    //   // ... write content + register a UCAN-backed cap policy with time
    //   //     window so the materializer's READ fanout consults UCAN ...
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let err = mat
    //       .materialize_with_gate(&engine, /* spec */ ..)
    //       .expect_err("materializer MUST fail-closed with no clock injected per sec-3.5-r1-7");
    //
    //   // Surfaced as ErrorCode::UcanClockNotInjected (existing variant; not new).
    //   match err {
    //       EngineError::Other { code: ErrorCode::UcanClockNotInjected, .. } => (),
    //       other => panic!(
    //           "expected E_UCAN_CLOCK_NOT_INJECTED, got {other:?} — fail-closed floor breached"
    //       ),
    //   }
    //
    //   // SUBSTANCE: also verify that injecting a clock to the SAME setup
    //   // makes the walk succeed (proves the failure is clock-driven, not
    //   // unrelated).
    //   let engine2 = Engine::open_with_clock(dir.path().join("benten2.redb"), test_clock).unwrap();
    //   let _ok = mat.materialize_with_gate(&engine2, /* spec */ ..)
    //       .expect("with clock injected, walk succeeds");
    let _ = materializer_fixtures::actor_principal_alice_cid();
    unimplemented!(
        "G23-B wave-5 wires materializer fail-closed inheritance of \
         E_UCAN_CLOCK_NOT_INJECTED + the with-clock-positive sibling check per sec-3.5-r1-7"
    );
}
