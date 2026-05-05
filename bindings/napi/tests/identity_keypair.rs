//! R3-A RED-PHASE pin for napi identity keypair surface
//! (G14-A1 wave-4a; r1-napi-1 MAJOR).
//!
//! Pin source: r2-test-landscape §2.2 G14-A1 row
//! `engine_napi_identity_keypair_round_trip`; r1-napi-1 MAJOR.
//!
//! ## What this pins
//!
//! G14-A1 lands `bindings/napi/src/identity.rs` exposing the new
//! `benten-id` Keypair / Did / Ucan types over the napi-v3 ABI. The
//! TS DSL (`packages/engine/src/identity.ts`) consumes this.
//!
//! This Rust-side pin asserts the napi bridge compiles + the Keypair
//! generate → sign → verify round-trip works END-TO-END through the
//! napi shim. The Vitest sibling at
//! `packages/engine/test/identity.test.ts` (R3-A's TS surface pin) is
//! the consumer-side pin.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A1 wave-4a introduces bindings/napi/src/identity.rs"]
fn engine_napi_identity_keypair_round_trip() {
    // r1-napi-1 MAJOR pin. G14-A1 implementer wires this:
    //
    //   // Native cdylib path (Node.js consumer):
    //   let kp = benten_napi::identity::generate_keypair();
    //   let pk = benten_napi::identity::keypair_public_key(&kp);
    //   let msg = b"napi round trip";
    //   let sig = benten_napi::identity::keypair_sign(&kp, msg);
    //   let verified = benten_napi::identity::public_key_verify(&pk, msg, &sig);
    //   assert!(verified);
    //
    //   // Did derivation crosses the napi boundary:
    //   let did = benten_napi::identity::keypair_to_did(&kp);
    //   assert!(did.as_str().starts_with("did:key:z"));
    //
    // OBSERVABLE consequence: TypeScript-side
    // `engine.identity.generate()` returns objects whose `sign()` /
    // `verify()` path round-trips byte-for-byte with the underlying
    // Rust impl. Defends against the napi-v3 ABI footgun where a
    // `Buffer` / `String` boundary corruption silently breaks
    // signatures on round-trip.
    //
    // r1-napi-1 named this MAJOR because the napi boundary for new
    // crypto types has historically been the highest-bug-density seam
    // in the codebase (Phase-1 R7 + Phase-2b R6 both caught issues
    // here).
    unimplemented!(
        "G14-A1 wires napi keypair generate → sign → verify round-trip + did:key derivation"
    );
}
