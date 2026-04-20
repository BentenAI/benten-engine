//! R4 triage (m13) — E_CAP_ATTENUATION fired at the declared-vs-actual
//! check on chained CALL.
//!
//! Attack class: a CALL chain where the child handler's declared capability
//! set exceeds what the parent context holds. The attenuation check runs at
//! CALL entry: child.required ⊆ parent.held must hold or `E_CAP_ATTENUATION`
//! fires.
//!
//! The codepath was only exercised indirectly via `error_code_mapping.rs`;
//! this file pins the trigger surface in its own test.
//!
//! Status: FAILING until R5 lands the attenuation check in the evaluator
//! CALL dispatch.

#![allow(clippy::unwrap_used)]

use benten_caps::{CapError, CapabilityPolicy, GrantScope, NoAuthBackend, WriteContext};
use benten_errors::ErrorCode;

#[test]
fn chained_call_exceeding_parent_caps_fires_attenuation() {
    // A parent context holds `store:post:read`. The child CALL declares a
    // required capability of `store:post:write` — strictly larger. The
    // attenuation check must reject at CALL entry.
    let parent_scope = GrantScope::parse("store:post:read").unwrap();
    let child_required = GrantScope::parse("store:post:write").unwrap();

    let err = benten_caps::testing::check_attenuation(&parent_scope, &child_required)
        .expect_err("child requires strictly more than parent — must attenuate");
    assert_eq!(err.code(), ErrorCode::CapAttenuation);
    assert!(matches!(err, CapError::Attenuation));
}

#[test]
fn chained_call_within_parent_caps_passes_attenuation() {
    // Positive-boundary: child.required ⊆ parent.held means no attenuation fires.
    let parent_scope = GrantScope::parse("store:post:*").unwrap();
    let child_required = GrantScope::parse("store:post:read").unwrap();
    benten_caps::testing::check_attenuation(&parent_scope, &child_required)
        .expect("child requirements within parent scope — no attenuation");
}

#[test]
fn noauth_backend_does_not_fire_attenuation() {
    // Under NoAuth, the attenuation check is never consulted — every write
    // is permitted. This is the baseline contract.
    let backend = NoAuthBackend::new();
    let ctx = WriteContext::default();
    assert!(backend.check_write(&ctx).is_ok());
}
