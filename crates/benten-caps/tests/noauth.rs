//! `NoAuthBackend::check_write` — the zero-cost default (P3, G4-A — R2
//! landscape §2.4 row 2).
//!
//! Permits every write unconditionally. Zero allocations on hot path; used
//! by the out-of-the-box Engine builder (Phase 1 DX requirement).
//!
//! Canonicalized at R4 triage (M14) across `noauth.rs`, `noauth_proptest.rs`,
//! and `production_refuses_noauth.rs`. All three use `NoAuthBackend::new()`
//! constructor and a `WriteContext` with `label` / `is_privileged` /
//! `actor_hint` fields. The redundant `target_label` field was dropped at
//! the G4 mini-review (g4-cr-10) — `label` is the single label axis now.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_caps::{CapabilityPolicy, NoAuthBackend, WriteContext};

#[test]
fn noauth_permits_empty_context() {
    let policy = NoAuthBackend::new();
    let ctx = WriteContext::default();
    assert!(policy.check_write(&ctx).is_ok());
}

#[test]
fn noauth_permits_populated_context() {
    let policy = NoAuthBackend::new();
    let ctx = WriteContext {
        label: "Post".to_string(),
        is_privileged: false,
        actor_hint: Some("alice".to_string()),
        ..WriteContext::default()
    };
    assert!(policy.check_write(&ctx).is_ok());
}

#[test]
fn noauth_permits_system_zone_context() {
    // NoAuthBackend does NOT enforce system-zone labels — that's the graph
    // layer's WriteContext::enforce_system_zone check. NoAuth is meant to be
    // zero-cost at the capability layer.
    let policy = NoAuthBackend::new();
    let ctx = WriteContext {
        label: "system:IVMView".to_string(),
        is_privileged: true,
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
    let _: Box<dyn CapabilityPolicy> = Box::new(NoAuthBackend::new());
}
