//! R4-FP-R3-B RED-PHASE pins: per-device CapabilityPolicy dispatch
//! (G14-A2 wave-4a' + G14-B wave-4b; cap-r4-4 MAJOR closure of
//! cap-minor-5 fix-now-action).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-4):
//!
//! - `tests/capability_policy_can_dispatch_per_device_cid_when_provided` — cap-r4-4 (a)
//! - `tests/capability_policy_treats_missing_device_cid_as_legacy_actor_only_path` — cap-r4-4 (b)
//! - `tests/capability_policy_per_device_cid_dispatch_observable_in_runtime_arm` — cap-r4-4 (c)
//!
//! ## Architectural intent (cap-r4-4 MAJOR closure)
//!
//! D-PHASE-3-25 promises per-device cap policy via the existing
//! `CapabilityPolicy` trait without modification. R1 cap-minor-5
//! noted that `WriteContext` + `ReadContext` do NOT carry device-DID
//! context — heterogeneous policies cannot distinguish "desktop X
//! writes" from "phone X writes". Without this dimension, per-device
//! rate-limit / size-budget / SANDBOX-eligibility claims in
//! D-PHASE-3-25 are paper-only.
//!
//! The fix is an additive optional field:
//!
//!   pub struct WriteContext { ... pub device_cid: Option<Cid> }
//!   pub struct ReadContext  { ... pub device_cid: Option<Cid> }
//!
//! Backward-compat: existing policies that ignore the field continue
//! to work; new heterogeneous policies dispatch on it.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-A2
//! wave-4a' AND G14-B wave-4b BOTH land. Per §3.6b pim-2 these tests
//! must drive the production CapabilityPolicy::pre_write /
//! pre_read path + assert observable consequence (different policy
//! decisions for different device CIDs under same actor).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 + G14-B — cap-r4-4 (a) — policy dispatches per device CID"]
fn capability_policy_can_dispatch_per_device_cid_when_provided() {
    // cap-r4-4 (a) pin. A custom CapabilityPolicy impl can dispatch
    // different decisions for the SAME actor based on the
    // `device_cid` field on the context.
    //
    // Concrete shape:
    //   use benten_caps::policy::{CapabilityPolicy, WriteContext};
    //
    //   let actor_did = ...;
    //   let device_a_cid = ...; // desktop
    //   let device_b_cid = ...; // phone
    //
    //   struct PerDevicePolicy {
    //       desktop: Cid,
    //       phone: Cid,
    //   }
    //   impl CapabilityPolicy for PerDevicePolicy {
    //       fn pre_write(&self, ctx: &WriteContext, ...) -> Result<(), CapError> {
    //           match ctx.device_cid.as_ref() {
    //               Some(d) if d == &self.desktop => Ok(()),
    //               Some(d) if d == &self.phone =>
    //                   Err(CapError::SizeLimitExceeded("phone device limit".into())),
    //               _ => Ok(()),
    //           }
    //       }
    //       ...
    //   }
    //
    //   let policy = PerDevicePolicy { desktop: device_a_cid.clone(), phone: device_b_cid.clone() };
    //
    //   // Same actor, different device → different decisions:
    //   let ctx_a = WriteContext { actor_cid: actor_did.clone(),
    //       device_cid: Some(device_a_cid.clone()), ..Default::default() };
    //   policy.pre_write(&ctx_a, &node).unwrap();
    //
    //   let ctx_b = WriteContext { actor_cid: actor_did.clone(),
    //       device_cid: Some(device_b_cid.clone()), ..Default::default() };
    //   let err = policy.pre_write(&ctx_b, &node).unwrap_err();
    //   assert!(matches!(err, benten_caps::CapError::SizeLimitExceeded(_)),
    //       "policy must dispatch differently per device_cid per cap-r4-4 (a)");
    //
    // OBSERVABLE consequence: heterogeneous per-device policies are
    // expressible in the existing CapabilityPolicy trait via an
    // additive `device_cid` field on the context. Defends D-PHASE-3-25
    // from being paper-only.
    unimplemented!(
        "G14-B wires WriteContext.device_cid + ReadContext.device_cid additive field per cap-r4-4 (a)"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-B — cap-r4-4 (b) — missing device_cid treated as legacy actor-only path"]
fn capability_policy_treats_missing_device_cid_as_legacy_actor_only_path() {
    // cap-r4-4 (b) pin. Backward-compat: existing policies that ignore
    // the device_cid field continue to work. A context with
    // `device_cid: None` MUST behave identically to a context built
    // by pre-cap-r4-4 callers.
    //
    // Concrete shape:
    //   use benten_caps::policy::{CapabilityPolicy, WriteContext, NoAuthBackend};
    //
    //   let actor_did = ...;
    //   let policy = NoAuthBackend::default();
    //
    //   // Legacy actor-only context (device_cid not provided):
    //   let ctx_legacy = WriteContext::new(actor_did.clone(), zone, label);
    //   assert!(ctx_legacy.device_cid.is_none(),
    //       "default WriteContext::new must leave device_cid None per cap-r4-4 (b) backward-compat");
    //
    //   // Policy decision identical:
    //   policy.pre_write(&ctx_legacy, &node).unwrap();
    //
    //   // Existing rate-limit policy ignores device_cid:
    //   let rate_policy = benten_caps::rate_limit::RateLimitPolicy::builder()
    //       .actor_writes_per_second(actor_did.clone(), "/zone/posts", 10)
    //       .build();
    //   for _ in 0..10 { rate_policy.pre_write(&ctx_legacy, &node).unwrap(); }
    //   let err = rate_policy.pre_write(&ctx_legacy, &node).unwrap_err();
    //   assert!(matches!(err, benten_caps::CapError::RateLimitExceeded { .. }),
    //       "legacy ignore-device-cid policy must function unchanged per cap-r4-4 (b)");
    //
    // OBSERVABLE consequence: pre-cap-r4-4 policies continue working
    // unchanged; the `device_cid` field is purely additive. Defends
    // against breaking-change semantics.
    unimplemented!("G14-B wires backward-compat for legacy actor-only callers per cap-r4-4 (b)");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 + G14-B — cap-r4-4 (c) — per-device dispatch observable in runtime arm"]
fn capability_policy_per_device_cid_dispatch_observable_in_runtime_arm() {
    // cap-r4-4 (c) pin (pim-2 production-runtime-arm assertion). The
    // device_cid field MUST be threaded from the production write-
    // path call site to the policy invocation — not just a struct
    // field that no production caller populates.
    //
    // Concrete shape:
    //   use benten_engine::Engine;
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(),
    //       benten_id::device_attestation::CapabilityEnvelope::default()).unwrap();
    //
    //   // Custom policy that records device_cid into a side channel:
    //   let observed_device_cids = Arc::new(Mutex::new(Vec::new()));
    //   let policy = RecordingPolicy { observed: observed_device_cids.clone() };
    //
    //   let engine = Engine::builder()
    //       .with_policy(policy)
    //       .open_for_device(store_dir.path(),
    //           parent_kp, device_kp.clone(), attestation).unwrap();
    //
    //   // Drive a production write:
    //   engine.write_node(&node).unwrap();
    //
    //   // Policy observed the device_cid (production runtime threaded it through):
    //   let observed = observed_device_cids.lock().unwrap();
    //   assert!(!observed.is_empty(),
    //       "policy MUST be invoked with device_cid populated per cap-r4-4 (c)");
    //   assert_eq!(observed[0].as_ref().unwrap(),
    //       &device_kp.public_key().to_cid(),
    //       "device_cid in WriteContext must match the engine's device per cap-r4-4 (c)");
    //
    //   // Source-cite check that the production codepath threads the field:
    //   let src = std::fs::read_to_string("crates/benten-engine/src/runtime/write_path.rs").unwrap();
    //   assert!(src.contains("device_cid:") || src.contains("device_cid ="),
    //       "production write_path.rs must populate device_cid at WriteContext-construction site");
    //
    // OBSERVABLE consequence: the production runtime arm threads the
    // device_cid into the policy invocation. Defends against the pim-2
    // sentinel-presence-vs-end-to-end shape (struct field exists; no
    // production caller populates it).
    unimplemented!(
        "G14-A2 + G14-B wire production runtime threading of device_cid per cap-r4-4 (c)"
    );
}
