//! R3-B RED-PHASE pins for `benten-id` MultiSigSurface trait
//! (G14-A2 wave-4a'; crypto-minor-2 + cag-5 + D-PHASE-3-24).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-A2):
//!
//! - `tests/multi_sig_surface_trait_signature_pinned` — crypto-minor-2 (architectural-pin)
//! - `tests/multi_sig_surface_ed25519_single_key_default_impl_round_trip` — crypto-minor-2 (unit)
//! - `tests/multi_sig_surface_threshold_extension_point_present` — crypto-minor-2 (architectural-pin)
//! - `tests/multi_sig_surface_no_recovery_protocol_specific_behavior_in_phase_3` — cag-5 + D-PHASE-3-24 (architectural-pin)
//!
//! ## Architectural intent
//!
//! Per plan §3 G14-A2 row + D-PHASE-3-24 (identity-recovery deferral
//! to post-Phase-3 v1-assessment-window), the `MultiSigSurface` trait
//! lands at G14-A2 with the `Ed25519SingleKey` default impl ONLY.
//! Threshold + recovery protocol-specific impls are deferred. The
//! trait surface is load-bearing because future identity-recovery
//! protocols compose on top of it; the API contract must be stable
//! across G14-A2 → post-Phase-3 expansion.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Tests stay `#[ignore]`'d until G14-A2
//! wave-4a' implementer un-ignores AND replaces stub bodies. Per
//! §3.6b pim-2, the trait-pin tests must drive the live trait
//! definition + the default impl's sign/verify path; sentinel-
//! presence (`std::any::TypeId::of::<...>()`) does not suffice.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-minor-2 — MultiSigSurface trait signature pinned"]
fn multi_sig_surface_trait_signature_pinned() {
    // crypto-minor-2 architectural pin. The trait surface must be
    // EXACTLY:
    //
    //   pub trait MultiSigSurface {
    //       type Signature;
    //       type Error;
    //       fn sign(&self, msg: &[u8]) -> Result<Self::Signature, Self::Error>;
    //       fn verify(&self, msg: &[u8], sig: &Self::Signature) -> Result<(), Self::Error>;
    //       fn threshold(&self) -> u32;       // 1 for SingleKey
    //       fn participants(&self) -> u32;    // 1 for SingleKey
    //   }
    //
    // G14-A2 implementer wires this as a TYPE-LEVEL pin. Compile-time
    // verification via static_assertions or trybuild:
    //
    //   const _: fn() = || {
    //       fn assert_signature<S: benten_id::multi_sig::MultiSigSurface>() {
    //           let _: fn(&S, &[u8]) -> Result<S::Signature, S::Error> = S::sign;
    //           let _: fn(&S, &[u8], &S::Signature) -> Result<(), S::Error> = S::verify;
    //           let _: fn(&S) -> u32 = S::threshold;
    //           let _: fn(&S) -> u32 = S::participants;
    //       }
    //       assert_signature::<benten_id::multi_sig::Ed25519SingleKey>();
    //   };
    //   assert!(true, "compile-time check passed if this file compiles");
    //
    // OBSERVABLE consequence: any future trait-signature drift
    // (renaming `threshold` → `t`, removing `Self::Error`, etc.)
    // produces a compile error, which fails this test loudly.
    unimplemented!("G14-A2 wires compile-time trait-signature pin for MultiSigSurface");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-minor-2 — Ed25519SingleKey round-trip"]
fn multi_sig_surface_ed25519_single_key_default_impl_round_trip() {
    // crypto-minor-2 unit pin. `Ed25519SingleKey` is the load-bearing
    // default impl that carries Phase-3 identity work. The trait's
    // sign + verify path must round-trip.
    //
    // G14-A2 implementer wires:
    //
    //   use benten_id::multi_sig::{Ed25519SingleKey, MultiSigSurface};
    //   let kp = benten_id::keypair::Keypair::generate();
    //   let surface = Ed25519SingleKey::new(kp);
    //   let msg = b"multi-sig round trip";
    //   let sig = surface.sign(msg).unwrap();
    //   surface.verify(msg, &sig).unwrap();
    //   assert_eq!(surface.threshold(), 1);
    //   assert_eq!(surface.participants(), 1);
    //
    //   // Tampered message rejects:
    //   let bad = b"tampered round trip";
    //   assert!(surface.verify(bad, &sig).is_err());
    //
    // OBSERVABLE consequence: sign/verify round-trips byte-for-byte;
    // tampering invalidates. The default impl honors the trait
    // contract end-to-end.
    unimplemented!("G14-A2 wires Ed25519SingleKey sign/verify round-trip + tamper rejection");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-minor-2 — threshold extension point compile-only"]
fn multi_sig_surface_threshold_extension_point_present() {
    // crypto-minor-2 architectural pin. The trait surface must be
    // EXTENSIBLE for future threshold (k-of-n) impls without breaking
    // the SingleKey signature. Per plan §3 G14-A2 row: "threshold
    // extension point present (compile-only); load-bearing for
    // future identity-recovery protocols."
    //
    // G14-A2 implementer demonstrates the extension point lands by
    // wiring a placeholder type that compiles against the trait but
    // has no real impl yet:
    //
    //   pub struct ThresholdEd25519 { /* phase-9+ */ }
    //   // Compile-only stub:
    //   impl benten_id::multi_sig::MultiSigSurface for ThresholdEd25519 {
    //       type Signature = ...;
    //       type Error = benten_id::multi_sig::NotImplemented;
    //       fn sign(&self, _: &[u8]) -> Result<Self::Signature, Self::Error> {
    //           Err(benten_id::multi_sig::NotImplemented::PostPhase3)
    //       }
    //       fn verify(...) -> ... { Err(benten_id::multi_sig::NotImplemented::PostPhase3) }
    //       fn threshold(&self) -> u32 { 0 }
    //       fn participants(&self) -> u32 { 0 }
    //   }
    //
    // Then the test asserts the extension point exists by checking
    // the public type is at least DECLARED in the module hierarchy:
    //
    //   const _: fn() = || {
    //       fn assert_extensible<T: benten_id::multi_sig::MultiSigSurface>() {}
    //       // If the trait restricts to a closed set (e.g. via sealed
    //       // trait), this assert_extensible would be impossible to
    //       // call from outside the crate. The test passing means the
    //       // trait IS open for downstream extension.
    //   };
    //
    // OBSERVABLE consequence: a downstream crate (or post-Phase-3
    // wave) can implement MultiSigSurface for a new type without
    // forking benten-id. This is the load-bearing extensibility
    // contract.
    unimplemented!("G14-A2 wires extensibility check that MultiSigSurface is not sealed");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — cag-5 + D-PHASE-3-24 — no recovery-specific behavior in Phase 3"]
fn multi_sig_surface_no_recovery_protocol_specific_behavior_in_phase_3() {
    // cag-5 + D-PHASE-3-24 architectural pin. The MultiSigSurface
    // trait + Ed25519SingleKey default impl land at G14-A2; what is
    // EXPLICITLY OUT OF SCOPE for Phase 3:
    //
    // - Shamir secret sharing
    // - Social-recovery-via-UCAN
    // - MLS group key agreement
    // - Hardware-escrow / TPM-backed signing
    //
    // Per D-PHASE-3-24, identity-recovery protocol-choice is deferred
    // to post-Phase-3 v1-assessment-window. The Phase-3 trait surface
    // must NOT bake in protocol-specific assumptions (e.g., "Shamir
    // share count" as a method on the trait would break MLS impls).
    //
    // G14-A2 implementer wires this as a SOURCE-CITE assertion:
    //
    //   let src = std::fs::read_to_string("crates/benten-id/src/multi_sig.rs").unwrap();
    //   const FORBIDDEN: &[&str] = &[
    //       "shamir", "Shamir", "SHAMIR",
    //       "mls::", "MLS",
    //       "social_recovery", "SocialRecovery",
    //       "tpm", "TPM", "hardware_escrow",
    //   ];
    //   for needle in FORBIDDEN {
    //       assert!(!src.contains(needle),
    //           "multi_sig.rs MUST NOT name protocol {} per D-PHASE-3-24 deferral", needle);
    //   }
    //
    // OBSERVABLE consequence: the trait surface stays neutral on
    // recovery protocol choice; future wave-after-v1-assessment can
    // pick the right protocol without a breaking trait change.
    unimplemented!(
        "G14-A2 wires source-grep that multi_sig.rs is free of recovery-protocol-specific names"
    );
}
