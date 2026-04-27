//! Edge-case tests: WAIT DSL signal-keyed naming (dx-r1-8).
//!
//! R2 landscape §2.5.3 row "WAIT DSL signal naming standardized".
//!
//! The WAIT primitive's signal-keyed form is spelled `signal` in the TS DSL;
//! the Rust-level subgraph builder pins the property-name invariant. The
//! legacy `for` keyword is rejected (its deprecation path is TS-side; the
//! Rust builder simply does not expose it). The duration variant continues
//! to co-exist.
//!
//! Concerns pinned at the Rust level:
//! - `SubgraphBuilder::wait_signal(prev, name)` produces a WAIT node whose
//!   properties include `signal = Value::text(name)` and NO `for` property.
//! - `SubgraphBuilder::wait_duration(prev, d)` produces a WAIT node whose
//!   properties include `duration_ms = Value::Int(..)` and NO `signal`
//!   property.
//! - Both variants satisfy structural validation (Inv-1..6, 9, 10, 12).
//! - A signal name that is empty rejects at registration with
//!   `E_INV_REGISTRATION` (empty signal name has no routing meaning).
//!
//! R3 red-phase contract: R5 (G3-B) lands both WAIT builders + structural
//! validation. Tests compile; they fail because the builders don't exist.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes per R1 triage."
)]

use benten_core::Value;
use benten_errors::ErrorCode;
use benten_eval::SubgraphBuilder;
use benten_eval::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};
use std::time::Duration;

#[test]
fn wait_signal_builder_sets_signal_property_not_for() {
    let mut sb = SubgraphBuilder::new("wait_sig_naming");
    let start = sb.read("x");
    let w = sb.wait_signal(start, "user_resumes");
    sb.respond(w);
    let sg = sb.build_validated().expect("validation");

    let wait_node = sg
        .node_by_handle(w)
        .expect("wait node must exist in subgraph");
    assert_eq!(
        wait_node.property("signal"),
        Some(&Value::text("user_resumes")),
        "wait_signal builder must set `signal` property"
    );
    assert!(
        wait_node.property("for").is_none(),
        "wait_signal builder must NOT set `for` property (dx-r1-8 renamed)"
    );
    assert!(
        wait_node.property("duration_ms").is_none(),
        "wait_signal builder must NOT set duration_ms"
    );
}

#[test]
fn wait_duration_builder_sets_duration_ms_property_not_signal() {
    let mut sb = SubgraphBuilder::new("wait_dur_naming");
    let start = sb.read("x");
    let w = sb.wait_duration(start, Duration::from_millis(500));
    sb.respond(w);
    let sg = sb.build_validated().expect("validation");

    let wait_node = sg.node_by_handle(w).expect("wait node exists");
    assert_eq!(
        wait_node.property("duration_ms"),
        Some(&Value::Int(500)),
        "wait_duration must set duration_ms"
    );
    assert!(
        wait_node.property("signal").is_none(),
        "wait_duration must NOT set `signal`"
    );
}

#[test]
fn wait_signal_empty_name_fails_registration_with_invariant_code() {
    // Boundary: empty signal name. An empty-string signal is semantically
    // nonsensical (nothing can route to it); the builder must reject at
    // build_validated time.
    let mut sb = SubgraphBuilder::new("wait_sig_empty");
    let start = sb.read("x");
    let w = sb.wait_signal(start, "");
    sb.respond(w);

    let err = sb
        .build_validated()
        .expect_err("empty signal name must fail validation");
    assert_eq!(
        err.code(),
        ErrorCode::InvRegistration,
        "empty signal name must fire E_INV_REGISTRATION, got {:?}",
        err.code()
    );
}

#[test]
fn wait_duration_zero_allowed_but_pinned_as_immediate_timeout() {
    // Boundary: zero-duration WAIT is legal at registration (it surfaces as
    // immediate-timeout at execution; tested in wait_timeout.rs). Here we
    // only pin that build_validated accepts it.
    let mut sb = SubgraphBuilder::new("wait_dur_zero");
    let start = sb.read("x");
    let w = sb.wait_duration(start, Duration::ZERO);
    sb.respond(w);
    let sg = sb.build_validated().expect("zero-duration must validate");
    let wait_node = sg.node_by_handle(w).unwrap();
    assert_eq!(wait_node.property("duration_ms"), Some(&Value::Int(0)));
}

#[test]
fn wait_signal_and_wait_duration_are_the_same_primitive_kind() {
    // Integrity pin: both flavours use PrimitiveKind::Wait, not distinct
    // kinds — the shape branch is via properties, not the primitive enum.
    let mut sb1 = SubgraphBuilder::new("sig_kind");
    let start1 = sb1.read("x");
    let w1 = sb1.wait_signal(start1, "s");
    sb1.respond(w1);
    let sg1 = sb1.build_validated().unwrap();

    let mut sb2 = SubgraphBuilder::new("dur_kind");
    let start2 = sb2.read("x");
    let w2 = sb2.wait_duration(start2, Duration::from_millis(10));
    sb2.respond(w2);
    let sg2 = sb2.build_validated().unwrap();

    assert_eq!(
        sg1.node_by_handle(w1).unwrap().primitive_kind(),
        sg2.node_by_handle(w2).unwrap().primitive_kind(),
        "signal and duration variants must share the same PrimitiveKind::Wait"
    );
}
