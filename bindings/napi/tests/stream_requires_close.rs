//! R3-E RED-PHASE pins for G19-C2 napi requiresExplicitClose accessor
//! (wave 7 parallel; §7.1.2).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C2 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C2 must-pass column):
//!
//! - `tests/stream_requires_explicit_close_napi_accessor_present` — §7.1.2
//!
//! ## What G19-C2 establishes (§7.1.2)
//!
//! `bindings/napi/src/stream.rs::requiresExplicitClose` — NEW accessor
//! returning a boolean indicating whether the stream handle requires an
//! explicit close()/cancel() call. The accessor is the napi-side anchor
//! for the TS-side FinalizationRegistry leak detector.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 wires bindings/napi/src/stream.rs::requires_explicit_close accessor"]
fn stream_requires_explicit_close_napi_accessor_present() {
    // §7.1.2 pin. G19-C2 implementer wires this:
    //
    //   let engine = benten_napi::testing::open_in_memory_engine().unwrap();
    //   let sg = benten_napi::testing::register_stream_handler_for_test(&engine).unwrap();
    //   let handle = engine.open_stream(sg, "main", json!({})).unwrap();
    //
    //   // Sentinel-presence: the accessor exists + returns a boolean:
    //   let requires_close: bool = handle.requires_explicit_close();
    //   assert!(requires_close,
    //       "stream handle from openStream must require explicit close \
    //        per §7.1.2 leak-detector contract");
    //
    //   // Cleanup — exercise the close path so the leak detector doesn't
    //   // fire on the test handle itself:
    //   handle.close().unwrap();
    //
    // OBSERVABLE consequence: the napi binding exposes the
    // requiresExplicitClose accessor that the TS-side leak detector
    // consumes. Defends against a TS-side leak detector that's wired
    // to a missing napi anchor (would fail silently).
    unimplemented!("G19-C2 wires stream handle requires_explicit_close napi accessor");
}
