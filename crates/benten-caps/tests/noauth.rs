//! `NoAuthBackend::check_write` — the zero-cost default (P3, G4-A — R2
//! landscape §2.4 row 2).
//!
//! Permits every write unconditionally. Zero allocations on hot path; used
//! by the out-of-the-box Engine builder (Phase 1 DX requirement).
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_caps::{CapabilityPolicy, NoAuthBackend, WriteContext};

#[test]
fn noauth_permits_empty_context() {
    let policy = NoAuthBackend;
    let ctx = WriteContext::default();
    assert!(policy.check_write(&ctx).is_ok());
}

#[test]
fn noauth_permits_populated_context() {
    let policy = NoAuthBackend;
    let ctx = WriteContext {
        label: "Post".to_string(),
        actor_cid: None,
        scope: "post:write".to_string(),
        ..WriteContext::default()
    };
    assert!(policy.check_write(&ctx).is_ok());
}

#[test]
fn noauth_permits_system_zone_context() {
    // NoAuthBackend does NOT enforce system-zone labels — that's the graph
    // layer's WriteContext::enforce_system_zone check. NoAuth is meant to be
    // zero-cost at the capability layer.
    let policy = NoAuthBackend;
    let ctx = WriteContext {
        label: "system:IVMView".to_string(),
        ..WriteContext::default()
    };
    assert!(policy.check_write(&ctx).is_ok());
}

#[test]
fn noauth_pseudo_actor_label_is_stable() {
    assert_eq!(NoAuthBackend::pseudo_actor_label(), "noauth");
}

#[test]
fn noauth_is_object_safe_as_dyn_capability_policy() {
    let _: Box<dyn CapabilityPolicy> = Box::new(NoAuthBackend);
}
