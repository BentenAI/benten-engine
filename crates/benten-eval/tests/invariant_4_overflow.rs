//! Phase 2b R3-B — Inv-4 overflow-saturation unit tests (G7-B).
//!
//! D20 overflow:
//!   - max_nest_depth default = 4, max = 8.
//!   - Saturates to typed error at the configured max.
//!   - u8 boundary regression guard (no wraparound).
//!
//! Pin sources: D20-RESOLVED, wsa-D20 u8-boundary, Phase-2a
//! sec-r6r1-01 forward-compat (frame extension).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-B pending — D20 saturation at max"]
fn invariant_4_depth_saturates_to_typed_error_at_max_4() {
    // D20 — at depth 5 (one over default max 4), the SANDBOX entry
    // saturates to E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED (or
    // E_INV_SANDBOX_DEPTH per the catalog list). The check fires
    // BEFORE wasmtime instantiation.
    //
    // Test:
    //   - Default config (max_nest_depth=4); compose 5 nested SANDBOX.
    //   - Assert: depth-5 SANDBOX entry returns the saturation error;
    //     no wasmtime::Module compilation is observed for the
    //     depth-5 module CID (would fire D22 cold-start cost otherwise).
    todo!("R5 G7-B — depth-5 chain + saturation typed error");
}

#[test]
#[ignore = "Phase 2b G7-B pending — wsa-D20 u8 boundary"]
fn invariant_4_depth_overflow_saturates_no_wraparound_u8_boundary() {
    // wsa-D20 — `sandbox_depth: u8` saturates cleanly at u8::MAX (255).
    // A depth-256+ attempt does NOT wrap to 0; the
    // checked_add(1).ok_or(E_INV_SANDBOX_DEPTH) pattern fires
    // deterministically.
    //
    // White-box: directly synthesize an AttributionFrame with
    // sandbox_depth = u8::MAX; attempt to push another SANDBOX child;
    // assert the typed error fires and the frame is NOT mutated.
    //
    // Configuration boundary: even with max_nest_depth = u8::MAX
    // (engine config maximum), the type-level u8 boundary is the hard
    // ceiling — NOT a configurable knob.
    todo!("R5 G7-B — synth frame at u8::MAX + assert checked_add saturation");
}

#[test]
#[ignore = "Phase 2b G7-B pending — D20 + Phase-2a sec-r6r1-01 forward-compat"]
fn attribution_frame_sandbox_depth_field_present_default_zero() {
    // D20 + Phase-2a sec-r6r1-01 — `sandbox_depth: u8` field exists on
    // `AttributionFrame`; default value at frame construction is 0.
    //
    // sec-r6r1-01 carry: AttributionFrame is extension-shaped; D20's
    // u8 field addition does NOT break Phase-2a CIDs (the field
    // defaults to 0 and serializes to 0 — old fixtures load with the
    // field at default).
    //
    // Test:
    //   1. `AttributionFrame::new(...)` → frame.sandbox_depth == 0.
    //   2. Round-trip a Phase-2a-shape AttributionFrame through DAG-CBOR
    //      decode/encode → frame.sandbox_depth == 0 + frame.cid()
    //      MATCHES the Phase-2a-pinned fixture CID.
    //   3. Increment frame.sandbox_depth to 1; re-encode; CID DIFFERS
    //      (proves the field IS in canonical bytes when non-zero).
    todo!("R5 G7-B — assert field default + Phase-2a CID stability + non-zero CID delta");
}
