//! R3-B RED-PHASE pin for napi device-attestation surface
//! (G14-A2 wave-4a'; r1-napi-2 MAJOR).
//!
//! Pin source: r2-test-landscape §2.2 G14-A2 row
//! `engine_declare_device_attestation_round_trip_via_napi`; r1-napi-2 MAJOR.
//!
//! ## Ambiguous-ownership pre-emption (per r2 §13)
//!
//! `engine_declare_device_attestation_round_trip_via_napi` is shared
//! between R3-B (G14-A2 device-attestation surface — this file) and
//! R3-C (G16-D atrium-namespaced TS DSL extension). R3-B owns the
//! Rust-side napi round-trip pin; R3-C extends with the TS DSL
//! `engine.atrium.declareDeviceAttestation(...)` call-site round-trip
//! at G16-D.
//!
//! ## Architectural intent
//!
//! Per CLAUDE.md baked-in #17, browser tabs (thin-client) need to
//! declare their capability envelope to the full peer at handshake.
//! The TS DSL surface `engine.declareDeviceAttestation({...})` reaches
//! into napi; the napi shim builds + signs a `DeviceAttestation` via
//! `benten-id`. This test pins the napi-side glue.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-A2
//! implementer un-ignores. Per §3.6b pim-2, the test must drive the
//! production napi entry point and assert structural round-trip of
//! the envelope from TS-shape to Rust-shape and back.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — napi device-attestation surface. G14-A2 wave-4a' shipped (PR #108) — bindings/napi/src/identity.rs device-attestation surface lives at HEAD; test body pins specific napi-side round-trip contract that needs driver authoring; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn engine_declare_device_attestation_round_trip_via_napi() {
    // r1-napi-2 MAJOR pin. G14-A2 implementer wires this:
    //
    //   // Native cdylib path:
    //   let parent_kp = benten_napi::identity::generate_keypair();
    //   let device_kp = benten_napi::identity::generate_keypair();
    //
    //   let envelope_napi = benten_napi::identity::CapabilityEnvelopeNapi {
    //       runs_sandbox: false,
    //       holds_zones: "cache_only".to_string(),
    //       online_uptime: "session_bounded".to_string(),
    //       runs_atrium_peer: false,
    //   };
    //
    //   let attestation = benten_napi::identity::declare_device_attestation(
    //       &parent_kp,
    //       benten_napi::identity::keypair_to_did(&device_kp),
    //       envelope_napi.clone(),
    //   ).unwrap();
    //
    //   let env_round_tripped = benten_napi::identity::attestation_envelope(&attestation);
    //   assert_eq!(env_round_tripped.runs_sandbox, false);
    //   assert_eq!(env_round_tripped.holds_zones, "cache_only");
    //   assert_eq!(env_round_tripped.online_uptime, "session_bounded");
    //   assert_eq!(env_round_tripped.runs_atrium_peer, false);
    //
    //   // Signature verifies:
    //   assert!(benten_napi::identity::attestation_verify(
    //       &attestation,
    //       &benten_napi::identity::keypair_public_key(&parent_kp),
    //   ));
    //
    // OBSERVABLE consequence: the TS DSL
    // `engine.declareDeviceAttestation({runs_sandbox: false, ...})`
    // produces a structurally identical attestation when round-tripped
    // through Rust. Defends against napi v3 ABI footgun where a
    // boolean / enum-string boundary corruption silently mangles the
    // envelope.
    //
    // r1-napi-2 named this MAJOR because the napi boundary for new
    // crypto-adjacent types historically introduces silent corruption
    // (Phase-1 R7 + Phase-2b R6).
    unimplemented!(
        "G14-A2 wires napi declareDeviceAttestation round-trip + signature verify at napi shim"
    );
}
