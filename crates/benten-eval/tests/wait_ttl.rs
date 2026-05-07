// Phase-3 G20-A2 (D12 wave-8a) — D12 WAIT TTL registration + resume unit tests.
//
// `phase_2b_landed` cfg gate retired at G20-A2 wave-8a per scope-real-03:
// the test bodies now drive real engine surfaces (the helpers + GC
// machinery + ErrorCode variants all landed in this group), so the
// gate that suppressed compilation under default features is no longer
// needed.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::duration_suboptimal_units)]

use std::time::Duration;

use benten_engine::Engine;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// `wait_ttl_default_24h_applies_when_omitted` — D12 + R2 §1.10 row 514.
///
/// A WAIT primitive whose `args` does NOT carry a `ttl_hours` key MUST
/// receive the 24h default at suspend time (the deadline metadata
/// records `ttl_hours = 24` even when the spec omits the property).
#[test]
fn wait_ttl_default_24h_applies_when_omitted() {
    let (_dir, mut engine) = fresh_engine();
    let spec = benten_engine::testing::testing_make_wait_spec_default_ttl();
    let handler_id = engine.register_subgraph(spec).unwrap();
    let envelope =
        benten_engine::testing::testing_call_to_suspend(&mut engine, &handler_id).unwrap();
    let metadata = benten_engine::testing::testing_inspect_wait_metadata(&engine, &envelope)
        .expect("metadata in SuspensionStore");
    assert_eq!(
        metadata.ttl_hours,
        Some(24),
        "omitted ttl_hours MUST default to 24 (D12 default)"
    );
}

/// `wait_ttl_explicit_overrides_default` — D12 + R2 §1.10 row 515.
#[test]
fn wait_ttl_explicit_overrides_default() {
    let (_dir, mut engine) = fresh_engine();
    let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(48);
    let handler_id = engine.register_subgraph(spec).unwrap();
    let envelope =
        benten_engine::testing::testing_call_to_suspend(&mut engine, &handler_id).unwrap();
    let metadata = benten_engine::testing::testing_inspect_wait_metadata(&engine, &envelope)
        .expect("metadata present");
    assert_eq!(
        metadata.ttl_hours,
        Some(48),
        "explicit ttl_hours: 48 MUST override the 24h default"
    );
}

/// `wait_ttl_zero_rejected_at_registration` — D12 + R2 §1.10 row 516.
#[test]
fn wait_ttl_zero_rejected_at_registration() {
    let (_dir, engine) = fresh_engine();
    let bad_spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours_unchecked(0);
    let err = engine
        .register_subgraph(bad_spec)
        .expect_err("ttl_hours: 0 MUST be rejected at registration");
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_WAIT_TTL_INVALID"),
        "expected E_WAIT_TTL_INVALID, got: {rendered}"
    );
}

/// `wait_ttl_exceeds_max_rejected` — D12 + R2 §1.10 row 517.
#[test]
fn wait_ttl_exceeds_max_rejected() {
    let (_dir, engine) = fresh_engine();
    // Boundary inclusive: 720 accepted.
    let ok_spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(720);
    engine
        .register_subgraph(ok_spec)
        .expect("ttl_hours: 720 MUST be accepted (max inclusive)");

    // Just over: 721 rejected.
    let bad_spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours_unchecked(721);
    let err = engine
        .register_subgraph(bad_spec)
        .expect_err("ttl_hours: 721 MUST be rejected");
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_WAIT_TTL_INVALID"),
        "expected E_WAIT_TTL_INVALID for ttl_hours > 720, got: {rendered}"
    );
}

/// `wait_resume_after_expiry_fires_typed_error` — D12 + R2 §1.10 row 518.
#[test]
fn wait_resume_after_expiry_fires_typed_error() {
    let (_dir, mut engine) = fresh_engine();
    let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    let handler_id = engine.register_subgraph(spec).unwrap();
    let envelope =
        benten_engine::testing::testing_call_to_suspend(&mut engine, &handler_id).unwrap();

    // Advance well past the deadline.
    benten_engine::testing::testing_advance_wait_clock(&engine, Duration::from_secs(2 * 3600));

    let err = engine
        .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
        .expect_err("expired resume MUST fail closed");
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_WAIT_TTL_EXPIRED"),
        "expected E_WAIT_TTL_EXPIRED in error rendering, got: {rendered}"
    );
}
