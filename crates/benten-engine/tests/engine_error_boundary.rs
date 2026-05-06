//! G13-B GREEN-PHASE pins for engine error-boundary erasure
//! (Phase-3 R5 wave-2; D-PHASE-3-1a / D-B / arch-r1-1 BLOCKER closure).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + plan §3 G13-B
//! must-pass column + plan §6 D-B):
//!
//! - `crates/benten-engine/tests/engine_error_boundary.rs::engine_error_boundary_erases_backend_specific_error_to_dyn_std_error` — D-PHASE-3-1a / arch-r1-1 BLOCKER
//! - `crates/benten-engine/tests/engine_error_boundary.rs::engine_error_carries_typed_backend_error_per_d_phase_3_1a` — D-B
//!
//! ## What G13-B / D-PHASE-3-1a establishes
//!
//! Per arch-r1-1 BLOCKER + D-B resolution: the engine PUBLIC boundary
//! erases backend-specific errors to `Box<dyn std::error::Error + Send +
//! Sync>` (preserves API stability + napi error-mapping). INSIDE the
//! `EngineGeneric<B>` impl, errors stay typed via `B::Error`.
//!
//! The two test pins assert BOTH directions of that contract:
//!
//! 1. **Public boundary erases:** the public `EngineError` enum carries
//!    a `Backend(Box<dyn std::error::Error + Send + Sync>)` variant
//!    that wraps `B::Error` losslessly.
//! 2. **Source chain preserves typed:** downcasting via the
//!    `Box<dyn std::error::Error>::downcast_ref::<GraphError>()` path
//!    recovers the typed error for diagnostics — the typed error is
//!    NOT irrevocably lost at the boundary.

#![allow(clippy::unwrap_used)]

use benten_engine::{Engine, EngineError};
use benten_graph::GraphError;

/// Helper: drive `Engine::open` to a backend-construction failure. We
/// use a path that cannot exist as a redb file (a path under `/dev/null`
/// rejects `open` with an ENOTDIR-shape error on macOS / Linux). This
/// is the load-bearing entry-point for both pins below.
fn open_failing_engine() -> EngineError {
    let result = Engine::open("/dev/null/benten-g13-b-no-such-dir");
    result.expect_err(
        "Engine::open against an unconstructable redb path must produce a backend-construction error",
    )
}

#[test]
fn engine_error_boundary_erases_backend_specific_error_to_dyn_std_error() {
    // D-PHASE-3-1a / arch-r1-1 BLOCKER pin.
    //
    // Trigger a backend error and assert the public-boundary variant is
    // `EngineError::Backend(Box<dyn std::error::Error + Send + Sync>)`
    // — the dyn-erased shape that preserves API stability across
    // alternative backends (G13-C BrowserBackend / G13-D
    // SnapshotBlobBackend / future).
    let err = open_failing_engine();

    match &err {
        EngineError::Backend(boxed) => {
            // Compile-time pin: the Box is the dyn-erased shape with
            // the Send + Sync bounds the public boundary requires.
            let _: &(dyn std::error::Error + Send + Sync) = &**boxed;

            // Smoke: the boxed error has a Display rendering (every
            // `std::error::Error` does). Defends against a future
            // refactor that accidentally erases to a non-Display
            // shape.
            let rendered = format!("{boxed}");
            assert!(
                !rendered.is_empty(),
                "boxed Backend error must have a non-empty Display rendering; got empty string"
            );
        }
        other => panic!(
            "expected EngineError::Backend (D-PHASE-3-1a / arch-r1-1 BLOCKER closure), \
             got: {other:?}"
        ),
    }
}

#[test]
fn engine_error_carries_typed_backend_error_per_d_phase_3_1a() {
    // D-B resolution pin. The dyn-erased boundary MUST NOT lose the
    // typed error: downcasting through the boxed dyn Error recovers
    // the original `benten_graph::GraphError` for diagnostics +
    // structured logging.
    let err = open_failing_engine();

    match &err {
        EngineError::Backend(boxed) => {
            // Direct downcast on the boxed dyn error.
            let direct = boxed.downcast_ref::<GraphError>();

            // Source-chain walk fallback (in case a future refactor
            // wraps the typed error one level deeper inside another
            // adapter). Either path MUST recover the typed error.
            let chained = boxed.source().and_then(|s| s.downcast_ref::<GraphError>());

            assert!(
                direct.is_some() || chained.is_some(),
                "typed `benten_graph::GraphError` MUST be recoverable from the dyn-erased \
                 EngineError::Backend variant via direct downcast OR source-chain downcast — \
                 D-B resolution requires the typed error path stays open for diagnostics. \
                 boxed Display: {boxed}"
            );
        }
        other => panic!("expected EngineError::Backend, got {other:?}"),
    }
}
