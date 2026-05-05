//! R3-C RED-PHASE pin: `engine.atrium.declareDeviceAttestation`
//! round-trip via napi (G14-A2 + G16-D wave-6b; per r2-test-landscape
//! §2.4 G16-D + r1-napi-2).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-D row
//!   `engine_declare_device_attestation_round_trip_via_napi`.
//! - plan §3 G14-A2 row + G16-D row.
//! - `r1-napi-2` (device-attestation napi bridge surface).
//! - r2-test-landscape §13 ambiguous-ownership pre-emption ("R3-B
//!   writes the napi-side test; R3-C extends with TS DSL
//!   `engine.atrium.declareDeviceAttestation(...)` round-trip").
//!
//! ## R3-C extension (per ambiguous-ownership pre-emption)
//!
//! R3-B (G14-A2 device-attestation surface) authors a napi-side
//! test for the underlying `bindings/napi::declare_device_attestation`
//! shape. R3-C extends with the `engine.atrium.declareDeviceAttestation(...)`
//! TS-DSL round-trip pin (the namespaced TS surface that browser
//! tabs use to declare their device attestation envelope to a full
//! peer per CLAUDE.md baked-in #17).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G14-A2 + G16-D wave-6b — declare-device-attestation napi + TS DSL"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 + G16-D — r1-napi-2 — engine.atrium.declareDeviceAttestation round-trip via napi"]
fn engine_declare_device_attestation_round_trip_via_napi() {
    // r1-napi-2 + plan §3 G14-A2 + G16-D pin. G14-A2 + G16-D
    // implementers wire this:
    //
    //   1. Construct a device-DID + capability envelope.
    //   2. Call `engine.atrium.declareDeviceAttestation(envelope)`
    //      via the napi binding.
    //   3. Round-trip through the napi serializer (TS object →
    //      napi-rs serde → Rust struct → AttributionFrame).
    //   4. Assert the round-trip preserves: device-DID,
    //      capability list, signature bytes, freshness window.
    //
    // OBSERVABLE consequence: the napi bridge accepts a TS-side
    // device-attestation declaration and produces an
    // engine-internal AttributionFrame that round-trips byte-equal
    // through canonical-bytes encoding. Defends against the failure
    // shape where a TS-side type drift loses fields silently.
    unimplemented!("G14-A2 + G16-D wire device-attestation napi round-trip + TS DSL surface");
}
