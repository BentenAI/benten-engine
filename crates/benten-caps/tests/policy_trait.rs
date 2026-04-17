//! `CapabilityPolicy` trait shape (P1, G4-A — R2 landscape §2.4 row 1).
//!
//! `check_write(&self, ctx: &WriteContext) -> Result<(), CapError>`. Trait
//! object safe.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_caps::{CapError, CapabilityPolicy, NoAuthBackend, UcanBackend, WriteContext};

#[test]
fn trait_signature_accepts_write_context_by_ref() {
    let ctx = WriteContext::default();
    let policy: Box<dyn CapabilityPolicy> = Box::new(NoAuthBackend);
    let _result: Result<(), CapError> = policy.check_write(&ctx);
}

#[test]
fn trait_is_object_safe_for_noauth() {
    let _: Box<dyn CapabilityPolicy> = Box::new(NoAuthBackend);
}

#[test]
fn trait_is_object_safe_for_ucan() {
    let _: Box<dyn CapabilityPolicy> = Box::new(UcanBackend);
}

#[test]
fn two_backends_implement_the_same_trait() {
    // Minimal smoke: a slice of boxed policies compiles + can be iterated.
    let ctx = WriteContext::default();
    let policies: Vec<Box<dyn CapabilityPolicy>> =
        vec![Box::new(NoAuthBackend), Box::new(UcanBackend)];
    let outcomes: Vec<Result<(), CapError>> =
        policies.iter().map(|p| p.check_write(&ctx)).collect();
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes[0].is_ok());
    // g4-cr-6: NotImplemented carries `backend` + `lands_in_phase` fields.
    assert!(matches!(outcomes[1], Err(CapError::NotImplemented { .. })));
}
