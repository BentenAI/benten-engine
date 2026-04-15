//! Phase 1 R3 integration — regression coverage for the six named
//! compromises from the R1 Triage Addendum.
//!
//! Each sub-test has a comment `// Phase 1 compromise; remove when Phase X
//! implements Y` so removing the compromise is a grep-and-delete exercise.
//!
//! Named compromises (from R2 §2.8):
//! 1. TOCTOU window bound at 100-iter batch
//! 2. E_CAP_DENIED_READ leaks existence (option A)
//! 3. ErrorCode enum lives in benten-core
//! 4. WASM runtime is compile-check only for bindings/napi
//! 5. No write rate-limits but metric recorded
//! 6. BLAKE3 128-bit collision-resistance note in SECURITY-POSTURE.md
//!
//! **Status:** FAILING until relevant groups land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

// Phase 1 compromise; remove when Phase 2 implements per-iteration capability re-check.
#[test]
fn compromise_1_toctou_window_bound_at_100_iter_batch() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // 300-iter handler, revoke at iter 150.
    let sg = benten_engine::SubgraphSpec::builder()
        .handler_id("toctou")
        .iterate(300, |b| {
            b.write(|w| {
                w.label("post")
                    .requires("store:post:write")
                    .property("n", Value::Int(0))
            })
        })
        .respond()
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();
    let actor = engine.create_principal("alice").unwrap();
    engine.grant_capability(&actor, "store:post:write").unwrap();

    let outcome = engine
        .call_with_revocation_at(
            &handler_id,
            "toctou",
            Node::empty(),
            &actor,
            "store:post:write",
            150,
        )
        .unwrap();
    let completed = outcome.completed_iterations().unwrap();
    assert!(
        completed >= 149,
        "iter 149 must complete before revoke is observed (bounds window below)"
    );
    assert!(
        completed < 250,
        "iter 250 must NOT complete (bounds window above); got {completed}"
    );
}

// Phase 1 compromise; remove when Phase 2 introduces option-B (existence-hiding) read semantics.
#[test]
fn compromise_2_ecapdenied_read_leaks_existence() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Create a secret post via engine-privileged path.
    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("secret".into()));
    let secret_cid = engine
        .create_node(&Node::new(vec!["post".into()], p))
        .unwrap();

    // Register a read handler; do NOT grant read capability.
    let handler_id = engine.register_crud_with_grants("post").unwrap();
    let actor = engine.create_principal("bob").unwrap();

    let mut input = BTreeMap::new();
    input.insert("cid".into(), Value::Text(secret_cid.to_base32()));
    let outcome = engine
        .call_as(
            &handler_id,
            "post:get",
            Node::new(vec!["input".into()], input),
            &actor,
        )
        .unwrap();

    assert!(outcome.routed_through_edge("ON_DENIED"));
    assert_eq!(
        outcome.error_code(),
        Some("E_CAP_DENIED_READ"),
        "option A: denial code identifies capability, not existence — documented in SECURITY-POSTURE.md"
    );
    assert!(
        !outcome
            .error_message()
            .unwrap_or_default()
            .contains("not found"),
        "error message must not reveal existence/non-existence"
    );
}

// Phase 1 compromise; remove when Phase 2 extracts ErrorCode into its own crate.
#[test]
fn compromise_3_error_code_enum_in_benten_core() {
    // Regression marker: the canonical ErrorCode enum lives in benten_core,
    // not in a dedicated crate. Verify by importing from benten_core directly.
    let _code = benten_core::ErrorCode::from_str("E_CAP_DENIED");
    let type_name = std::any::type_name::<benten_core::ErrorCode>();
    assert!(
        type_name.starts_with("benten_core::"),
        "ErrorCode must live in benten_core crate (Phase 2 may extract); got {type_name}"
    );
}

// Phase 1 compromise; remove when Phase 2 ships network-fetch KVBackend + WASM runtime tests.
#[test]
fn compromise_4_wasm_runtime_only_compile_check() {
    // Verify CI workflow documents the compromise — scan for the canary comment.
    let ci = std::fs::read_to_string(".github/workflows/ci.yml").expect("CI file present");
    assert!(
        ci.contains("napi-wasm32-compile-check") || ci.contains("wasm32-unknown-unknown"),
        "CI must have a compile-check job for napi; got no match"
    );
    // No runtime test for napi under wasmtime yet; protect against accidental addition.
    assert!(
        !ci.contains("napi-wasm-runtime"),
        "napi WASM runtime test is Phase 2; see PLATFORM-DESIGN"
    );
}

// Phase 1 compromise; remove when Phase 2 adds write rate-limiting.
#[test]
fn compromise_5_no_write_rate_limits_but_metric_recorded() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    // 1000 fast writes — no rate-limit errors expected.
    for i in 0..1000u32 {
        let mut p = BTreeMap::new();
        p.insert("title".into(), Value::Text(format!("t{i}")));
        let outcome = engine
            .call(
                &handler_id,
                "post:create",
                Node::new(vec!["post".into()], p),
            )
            .unwrap();
        assert!(outcome.is_ok_edge(), "no rate-limit in Phase 1");
    }

    // Metric must be present and observable (even if value is 0 or low).
    let metrics = engine.metrics_snapshot();
    assert!(
        metrics.contains_key("benten.ivm.view_stale_count")
            || metrics.contains_key("benten.writes.total"),
        "metric plumbing must be in place even if rate-limit policy is not"
    );
}

// Phase 1 compromise; remove when a 256-bit collision-resistance posture is adopted.
#[test]
fn compromise_6_blake3_collision_resistance_note_in_security_posture() {
    let posture =
        std::fs::read_to_string("docs/SECURITY-POSTURE.md").expect("SECURITY-POSTURE.md present");
    assert!(
        posture.contains("BLAKE3") && posture.contains("128"),
        "SECURITY-POSTURE.md must document 128-bit collision-resistance of BLAKE3 as a Phase 1 compromise"
    );
}
