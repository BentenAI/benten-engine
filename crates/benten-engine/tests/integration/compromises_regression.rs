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
//
// Named-compromise #1 scope (per R1 triage clarification in r4-triage.md):
// the evaluator refreshes the capability snapshot at commit / CALL entry /
// ITERATE batch boundaries. Default batch size is 100 iterations. Writes
// 1..=100 succeed under the granted cap (snapshot held); at iteration 101
// the next batch re-reads and sees the revoked cap, so write 101 fails with
// E_CAP_REVOKED_MID_EVAL.
#[test]
fn compromise_1_toctou_window_bound_at_100_iter_batch() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // 300-iter handler, revoke mid batch 1 (iter 50). Batch size 100 means
    // writes 1..=100 land before the next boundary re-reads caps.
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
            50,
        )
        .unwrap();
    let successful = outcome.successful_write_count();
    assert_eq!(
        successful, 100,
        "iterations 1..=100 must complete (first batch holds cap snapshot); got {successful}"
    );
    // Write 101 sits at the next batch boundary where cap is re-read and
    // found revoked.
    assert_eq!(
        outcome.error_code(),
        Some("E_CAP_REVOKED_MID_EVAL"),
        "write 101 must fail with E_CAP_REVOKED_MID_EVAL (batch boundary cap refresh saw revocation)"
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
//
// Rewritten at R4 triage (M15) — the v1 version grepped filenames, which was
// brittle to workflow-name drift. The semantic checks are:
//   (a) `benten-napi`'s Cargo.toml does NOT declare `wasmtime` as a runtime
//       dep (would be a Phase 2 surface),
//   (b) `.github/workflows/wasm-checks.yml` exists and contains a
//       `cargo check --target wasm32-unknown-unknown` invocation (the
//       compile-check gate is the Phase 1 contract),
//   (c) no workflow invokes `cargo test --target wasm32-wasip1` (runtime
//       testing is Phase 2).
#[test]
fn compromise_4_wasm_runtime_is_phase_2() {
    // (a) bindings/napi Cargo.toml does NOT declare `wasmtime`. We check the
    // `[dependencies]` section specifically — `wasmtime` appears in workspace
    // deps but must NOT reach the napi crate in Phase 1.
    let napi_manifest = std::fs::read_to_string("../../bindings/napi/Cargo.toml")
        .or_else(|_| std::fs::read_to_string("bindings/napi/Cargo.toml"))
        .expect("bindings/napi/Cargo.toml present");
    // A naive string-grep is brittle, but `wasmtime =` at line start or after
    // whitespace is the canonical TOML form for a dependency declaration.
    let has_wasmtime_dep = napi_manifest.lines().any(|l| {
        let t = l.trim();
        t.starts_with("wasmtime =") || t.starts_with("wasmtime=")
    });
    assert!(
        !has_wasmtime_dep,
        "bindings/napi/Cargo.toml must NOT declare `wasmtime` as a direct \
         dep in Phase 1 — the WASM runtime landing is Phase 2"
    );

    // (b) wasm-checks workflow exists and invokes the compile-check target.
    let wasm_workflow = std::fs::read_to_string("../../.github/workflows/wasm-checks.yml")
        .or_else(|_| std::fs::read_to_string(".github/workflows/wasm-checks.yml"))
        .expect(".github/workflows/wasm-checks.yml must exist in Phase 1");
    assert!(
        wasm_workflow.contains("wasm32-unknown-unknown"),
        "wasm-checks.yml must contain the `wasm32-unknown-unknown` compile-check target"
    );
    assert!(
        wasm_workflow.contains("cargo check"),
        "wasm-checks.yml must invoke `cargo check` (not `cargo test`) per the Phase 1 gate"
    );

    // (c) No workflow runs `cargo test --target wasm32-wasip1` — runtime
    // testing of WASM is Phase 2 scope.
    let workflow_dir = if std::path::Path::new("../../.github/workflows").exists() {
        std::path::PathBuf::from("../../.github/workflows")
    } else {
        std::path::PathBuf::from(".github/workflows")
    };
    for entry in std::fs::read_dir(&workflow_dir).expect("read workflow dir") {
        let entry = entry.expect("readdir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("read yml");
        // G7 test-authoring fix: the v1 check `content.contains("wasm32-wasip1")`
        // was too broad — determinism.yml legitimately references the
        // `wasm32-wasip1` target in its `cargo build --release` step that
        // feeds a cross-runtime CID-determinism canary (T6). The compromise
        // prohibits `cargo test --target wasm32-wasip1`, not `cargo build`.
        // Check for the narrower form per the comment above.
        let has_cargo_test_wasip1 = content
            .lines()
            .any(|l| l.contains("cargo test") && l.contains("wasm32-wasip1"));
        assert!(
            !has_cargo_test_wasip1,
            "no workflow may invoke `cargo test --target wasm32-wasip1` in \
             Phase 1; got reference in {}",
            path.display()
        );
    }

    // R4 triage (m16 minor): fixture-CID canary — protects against encoding
    // drift slipping through alongside a WASM-scope change.
    let expected_fixture = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";
    let canonical = benten_core::testing::canonical_test_node().cid().unwrap();
    assert_eq!(
        canonical.to_base32(),
        expected_fixture,
        "canonical fixture CID drift detected — investigate encoding path before merging"
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
//
// G7 test-authoring fix: integration tests run from the crate directory
// (`crates/benten-engine/`), not the repo root. The v1 `"docs/..."` path
// resolved to `crates/benten-engine/docs/...` (which does not exist). This
// mirrors the `or_else` fallback pattern compromise_4_wasm_runtime_is_phase_2
// already uses a few tests above.
#[test]
fn compromise_6_blake3_collision_resistance_note_in_security_posture() {
    let posture = std::fs::read_to_string("../../docs/SECURITY-POSTURE.md")
        .or_else(|_| std::fs::read_to_string("docs/SECURITY-POSTURE.md"))
        .expect("SECURITY-POSTURE.md present");
    assert!(
        posture.contains("BLAKE3") && posture.contains("128"),
        "SECURITY-POSTURE.md must document 128-bit collision-resistance of BLAKE3 as a Phase 1 compromise"
    );
}
