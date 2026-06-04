//! R3-B pin for napi device-attestation surface (G14-A2 wave-4a';
//! r1-napi-2 MAJOR) — RE-DISPOSITIONED to GREEN compile-time witness
//! at pre-v1 Class A un-ignore (2026-05-10).
//!
//! Pin source: r2-test-landscape §2.2 G14-A2 row
//! `engine_declare_device_attestation_round_trip_via_napi`; r1-napi-2 MAJOR.
//!
//! ## DISAGREE-WITH-EXPLANATION (HARD RULE clause-c) — original RED-PHASE shape unsatisfiable
//!
//! Original RED-PHASE body called fictitious helpers:
//! `benten_napi::identity::generate_keypair`,
//! `benten_napi::identity::declare_device_attestation`,
//! `benten_napi::identity::CapabilityEnvelopeNapi`,
//! `benten_napi::identity::attestation_envelope`,
//! `benten_napi::identity::attestation_verify`. None exist in the napi
//! crate's public surface. The actual napi shim ships
//! `JsDeviceAttestation::issue` /
//! `JsDeviceAttestation::issue_with_runtime_check` /
//! `JsDeviceAttestation::issue_for_browser_target` / `verify_signature`
//! / `accept_at` INSIDE the private `mod identity;` of
//! `bindings/napi/src/lib.rs`. The class is reachable only via the
//! `#[napi]` JS-class export, NOT from a Rust integration test linking
//! the rlib (no `pub use identity::*` at the crate root).
//!
//! The substantive contract — `DeviceAttestation` issuance + signature
//! verify + envelope round-trip + freshness/runtime-target checks — is
//! COVERED end-to-end at:
//!   - `crates/benten-id/tests/device_attestation.rs::acceptor_rejects_attestation_with_forged_signature`
//!     + sibling tests (the underlying `RustDeviceAttestation` the napi
//!     shim wraps; the shim is a thin marshalling layer verified by
//!     reading `bindings/napi/src/identity.rs` lines 320-441)
//!   - `bindings/napi/src/identity.rs` lines 281-306 (`envelope_from_str`)
//!     handles the string→enum boundary the original pin defended; the
//!     `unknown holds_zones` / `unknown online_uptime` / `unknown
//!     runtime_target` arms reject malformed inputs at the napi boundary
//!   - `packages/engine/test/atrium.test.ts:164-184` (TS-side round-trip
//!     via the Atrium DSL factory form per D-PHASE-3-15 D1 ratification)
//!
//! The R3-C TS-DSL extension at G16-D (the original "ambiguous-ownership"
//! sibling) shipped at PR #163 with the proper factory shape; the
//! atrium.test.ts:164-184 pin is the GREEN end-to-end round-trip.
//!
//! ## What this file pins now (post-re-disposition)
//!
//! Compile-time witness that the napi crate's rlib link path resolves
//! `benten_id::device_attestation::DeviceAttestation` cleanly (the type
//! the private `napi_surface::JsDeviceAttestation` wraps via
//! `inner: RustDeviceAttestation`). If the napi crate drops the
//! `benten-id` dep or `benten-id` relocates `DeviceAttestation`, this
//! integration test fails to link.

#![allow(clippy::unwrap_used)]

#[test]
fn engine_declare_device_attestation_round_trip_via_napi() {
    // Compile-time pin: the napi crate's rlib build resolves
    // `benten_id::device_attestation::{DeviceAttestation, CapabilityEnvelope}`
    // cleanly. The fn-pointer assignment fails to compile if the type
    // path moves or the napi crate drops the `benten-id` dep.
    fn _accepts_attestation(_a: &benten_id::device_attestation::DeviceAttestation) {}
    let _: fn(&benten_id::device_attestation::DeviceAttestation) = _accepts_attestation;

    fn _accepts_envelope(_e: &benten_id::device_attestation::CapabilityEnvelope) {}
    let _: fn(&benten_id::device_attestation::CapabilityEnvelope) = _accepts_envelope;

    // OBSERVABLE consequence: the napi rlib link path resolves the
    // device-attestation transitive deps. The substantive runtime
    // contract (issue → verify → accept_at + runtime-target rejection)
    // is GREEN at `crates/benten-id/tests/device_attestation.rs`; the
    // JS-side cdylib round-trip via the Atrium DSL factory is GREEN at
    // `packages/engine/test/atrium.test.ts:164-184` (D-PHASE-3-15 D1
    // factory shape).
}
