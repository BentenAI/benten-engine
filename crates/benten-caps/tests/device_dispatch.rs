//! GREEN-PHASE pins (partial): per-device CapabilityPolicy dispatch
//! structural surface (G16-B canary; r4b-cap-3 BLOCKER closure).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-4 + R4b r4b-cap-3):
//!
//! - `capability_policy_can_dispatch_per_device_cid_when_provided` — cap-r4-4 (a)
//! - `capability_policy_treats_missing_device_cid_as_legacy_actor_only_path` — cap-r4-4 (b)
//! - `capability_policy_per_device_cid_dispatch_observable_in_runtime_arm` — cap-r4-4 (c)
//!
//! ## Architectural intent (cap-r4-4 + r4b-cap-3 BLOCKER closure)
//!
//! D-PHASE-3-25 promises per-device cap policy via the existing
//! `CapabilityPolicy` trait without modification. Pre-G16-B-canary the
//! `WriteContext` + `ReadContext` types did NOT carry a `device_cid`
//! field — heterogeneous policies could not distinguish "desktop X
//! writes" from "phone X writes". G16-B canary lands the additive
//! `pub device_cid: Option<Cid>` field on both context types
//! (BLOCKER r4b-cap-3 structural-surface closure).
//!
//! ## Coverage
//!
//! Tests (a) + (b) below exercise the structural surface — the field
//! exists, dispatch on the field works, missing field maintains
//! backward-compat. The (c) end-to-end production-runtime test pin
//! requires engine-side write-path threading of `device_cid` through
//! the `WriteContext` construction sites; that wiring landed and (c)
//! was relocated to (and is GREEN at HEAD in)
//! `crates/benten-engine/tests/device_cid_runtime_arm.rs` (it requires
//! the `benten-engine` dependency). It is no longer `#[ignore]`'d.

#![allow(clippy::unwrap_used)]

use benten_caps::policy::{CapabilityPolicy, ReadContext, WriteContext};
use benten_caps::{CapError, NoAuthBackend};
use benten_core::Cid;

fn cid_for(seed: &[u8]) -> Cid {
    Cid::from_blake3_digest(*blake3::hash(seed).as_bytes())
}

#[test]
fn capability_policy_can_dispatch_per_device_cid_when_provided() {
    // cap-r4-4 (a) pin. A custom CapabilityPolicy impl can dispatch
    // different decisions for the SAME actor based on the
    // `device_cid` field on the context.
    let actor = cid_for(b"actor:alice");
    let device_desktop = cid_for(b"device:desktop");
    let device_phone = cid_for(b"device:phone");

    struct PerDevicePolicy {
        desktop: Cid,
    }
    impl CapabilityPolicy for PerDevicePolicy {
        fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError> {
            match ctx.device_cid.as_ref() {
                Some(d) if d == &self.desktop => Ok(()),
                Some(_) => Err(CapError::Denied {
                    required: ctx.scope.clone(),
                    entity: ctx.label.clone(),
                }),
                _ => Ok(()),
            }
        }
    }

    let policy = PerDevicePolicy {
        desktop: device_desktop,
    };

    // Same actor, different device → different decisions:
    let ctx_desktop = WriteContext {
        label: "post".into(),
        actor_cid: Some(actor),
        device_cid: Some(device_desktop),
        ..Default::default()
    };
    assert!(
        policy.check_write(&ctx_desktop).is_ok(),
        "policy MUST permit writes from the desktop device per cap-r4-4 (a)"
    );

    let ctx_phone = WriteContext {
        label: "post".into(),
        actor_cid: Some(actor),
        device_cid: Some(device_phone),
        ..Default::default()
    };
    assert!(
        matches!(policy.check_write(&ctx_phone), Err(CapError::Denied { .. })),
        "policy MUST dispatch differently per device_cid per cap-r4-4 (a)"
    );
}

#[test]
fn capability_policy_treats_missing_device_cid_as_legacy_actor_only_path() {
    // cap-r4-4 (b) pin. Backward-compat: existing policies that ignore
    // the device_cid field continue to work. A context with
    // `device_cid: None` MUST behave identically to a context built by
    // pre-cap-r4-4 callers.
    let policy = NoAuthBackend;
    let actor = cid_for(b"actor:alice");

    // Legacy actor-only context (device_cid not provided):
    let ctx_legacy = WriteContext {
        label: "post".into(),
        actor_cid: Some(actor),
        ..Default::default()
    };
    assert!(
        ctx_legacy.device_cid.is_none(),
        "default WriteContext MUST leave device_cid None per cap-r4-4 (b) backward-compat"
    );
    // Policy decision identical (NoAuth permits everything).
    assert!(
        policy.check_write(&ctx_legacy).is_ok(),
        "legacy actor-only path MUST function unchanged per cap-r4-4 (b)"
    );

    // Same for ReadContext.
    let read_legacy = ReadContext {
        label: "post".into(),
        actor_cid: Some(actor),
        ..Default::default()
    };
    assert!(
        read_legacy.device_cid.is_none(),
        "default ReadContext MUST leave device_cid None per cap-r4-4 (b) backward-compat"
    );
    assert!(
        policy.check_read(&read_legacy).is_ok(),
        "legacy ReadContext path MUST function unchanged per cap-r4-4 (b)"
    );
}

// Note: the production-runtime-arm assertion that drives
// `engine.write_node(...)` through a recording policy + the engine's
// configured device_cid lives at
// `crates/benten-engine/tests/device_cid_runtime_arm.rs` because it
// requires the `benten-engine` dependency. This file's
// `capability_policy_per_device_cid_dispatch_observable_in_runtime_arm`
// (a + b structural pins above) covers the structural-surface side
// of the contract; the engine-side pin closes pim-2 end-to-end.
