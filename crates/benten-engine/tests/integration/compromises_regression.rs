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
#[ignore = "TODO(phase-2-iterate-subgraph): GrantBackedPolicy IS wired for the static cap check, but the TOCTOU-window 100-iter batch semantics need (a) the `SubgraphSpec::iterate(n, ...)` builder (currently fails registration with RegistrationError), (b) a per-batch capability re-read at the ITERATE boundary inside the evaluator, and (c) `call_with_revocation_at` driving an actual mid-evaluation revocation. All three land in Phase 2."]
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

// 5d-J workstream 1: migrated to Option C (symmetric-None +
// diagnose_read escape hatch). The old Option-A test has been replaced.
#[test]
fn compromise_2_option_c_symmetric_none_plus_diagnose_read() {
    use benten_engine::EngineError;

    // A deny-all-reads policy that permits the `debug:read` probe so
    // diagnose_read still surfaces the distinction.
    #[derive(Debug)]
    struct DenyReadsPermitDebug;
    impl benten_caps::CapabilityPolicy for DenyReadsPermitDebug {
        fn check_write(
            &self,
            _ctx: &benten_caps::WriteContext,
        ) -> Result<(), benten_caps::CapError> {
            Ok(())
        }
        fn check_read(&self, ctx: &benten_caps::ReadContext) -> Result<(), benten_caps::CapError> {
            if ctx.label == "debug" || ctx.label.is_empty() {
                return Ok(());
            }
            Err(benten_caps::CapError::DeniedRead {
                required: format!("store:{}:read", ctx.label),
                entity: String::new(),
            })
        }
    }

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(DenyReadsPermitDebug))
        .build()
        .unwrap();

    // Insert a secret post via the NoAuth-equivalent write path.
    let mut p = BTreeMap::new();
    p.insert("title".into(), Value::Text("secret".into()));
    let secret_cid = engine
        .create_node(&Node::new(vec!["post".into()], p))
        .unwrap();

    // Option C primary path: engine.get_node returns Ok(None) — the
    // caller cannot distinguish denial from miss.
    let opt = engine.get_node(&secret_cid).unwrap();
    assert!(
        opt.is_none(),
        "Option C: denied reads collapse to Ok(None), symmetric with a miss"
    );

    // Option C escape hatch: diagnose_read surfaces the distinction
    // under `debug:read`.
    let info = engine
        .diagnose_read(&secret_cid)
        .expect("debug:read permitted");
    assert!(info.exists_in_backend);
    assert!(info.denied_by_policy.is_some());
    assert!(!info.not_found);

    // Sanity: a completely unheld `debug:read` (via a policy that denies
    // everything including debug) fires CapDenied at diagnose_read time.
    #[derive(Debug)]
    struct DenyAllIncludingDebug;
    impl benten_caps::CapabilityPolicy for DenyAllIncludingDebug {
        fn check_write(
            &self,
            _ctx: &benten_caps::WriteContext,
        ) -> Result<(), benten_caps::CapError> {
            Ok(())
        }
        fn check_read(&self, _ctx: &benten_caps::ReadContext) -> Result<(), benten_caps::CapError> {
            Err(benten_caps::CapError::DeniedRead {
                required: "store:debug:read".into(),
                entity: String::new(),
            })
        }
    }
    let dir2 = tempfile::tempdir().unwrap();
    let eng2 = Engine::builder()
        .path(dir2.path().join("benten.redb"))
        .capability_policy(Box::new(DenyAllIncludingDebug))
        .build()
        .unwrap();
    // Insert with the *permissive* engine first (writes permitted under both).
    let cid2 = eng2
        .create_node(&Node::new(vec!["post".into()], BTreeMap::new()))
        .unwrap();
    let err = eng2.diagnose_read(&cid2).unwrap_err();
    assert!(matches!(
        err,
        EngineError::Cap(benten_caps::CapError::Denied { .. })
    ));
}

// Compromise #3 CLOSED (Phase 1): the canonical ErrorCode enum was extracted
// from benten_core into a dedicated `benten_errors` root crate. This test now
// pins the new home so any accidental re-introduction of an `ErrorCode` type in
// benten_core (or any other crate) surfaces as a CI failure.
#[test]
fn compromise_3_error_code_enum_in_benten_errors() {
    let _code = benten_errors::ErrorCode::from_str("E_CAP_DENIED");
    let type_name = std::any::type_name::<benten_errors::ErrorCode>();
    assert!(
        type_name.starts_with("benten_errors::"),
        "ErrorCode must live in benten_errors crate (compromise #3 closed); got {type_name}"
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
    // Phase 2b G10-A-wasip1 explicitly CLOSES the Phase-1 prohibition:
    // wasm-runtime.yml + wasm-conformance.yml are the new Phase-2b workflows
    // that exercise the wasi runtime (D-NS-49 night-shift catch). Other
    // workflows are still subject to the Phase-1 invariant — drifting a
    // wasi-test invocation into a non-G10 workflow is still a regression.
    let phase_2b_wasi_workflows: &[&str] = &["wasm-runtime.yml", "wasm-conformance.yml"];
    for entry in std::fs::read_dir(&workflow_dir).expect("read workflow dir") {
        let entry = entry.expect("readdir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if phase_2b_wasi_workflows.contains(&file_name) {
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
            "Compromise #4 regression: only the Phase-2b G10 wasi workflows \
             ({phase_2b_wasi_workflows:?}) may invoke `cargo test --target \
             wasm32-wasip1`; got reference in {}",
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

// Named Compromise #7 CLOSURE regression: benten-graph's
// `write-canonical-and-exit` bin is gated behind the `test-fixtures`
// feature. `cargo install benten-graph --no-default-features` no longer
// compiles the test fixture; workspace-wide `cargo test` still builds it
// via the default-enabled feature. If this test goes red, someone
// removed the `required-features` gate and re-opened the compromise.
#[test]
fn compromise_7_benten_graph_bin_is_required_features_gated() {
    let manifest = std::fs::read_to_string("../benten-graph/Cargo.toml")
        .or_else(|_| std::fs::read_to_string("crates/benten-graph/Cargo.toml"))
        .expect("benten-graph/Cargo.toml must be present");

    // Extract the `[[bin]]` section named `write-canonical-and-exit`.
    let mut in_bin = false;
    let mut bin_body = String::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && in_bin {
            // Next section starts — stop capturing.
            break;
        }
        if trimmed == "[[bin]]" {
            in_bin = true;
            continue;
        }
        if in_bin {
            bin_body.push_str(line);
            bin_body.push('\n');
        }
    }

    assert!(
        bin_body.contains("name = \"write-canonical-and-exit\""),
        "benten-graph must declare the write-canonical-and-exit bin; got \
         bin section:\n{bin_body}"
    );
    assert!(
        bin_body.contains("test = false"),
        "bin must declare test = false so cargo test target discovery \
         skips it"
    );
    assert!(
        bin_body.contains("bench = false"),
        "bin must declare bench = false so cargo bench target discovery \
         skips it"
    );
    // Compromise #7 closure: the bin is gated behind `test-fixtures`
    // so `cargo install --no-default-features` does not compile a
    // test-only fixture binary alongside the library. The feature is
    // default-enabled in `[features]` so workspace `cargo test` still
    // builds the bin via the `CARGO_BIN_EXE_*` path.
    assert!(
        bin_body.contains("required-features = [\"test-fixtures\"]"),
        "Compromise #7 closure: benten-graph `[[bin]]` MUST declare \
         `required-features = [\"test-fixtures\"]`. If this gate is \
         removed, re-open Compromise #7 in docs/SECURITY-POSTURE.md."
    );

    // Belt + suspenders: the feature must exist (and be default) in the
    // `[features]` table so the bin is buildable by default.
    assert!(
        manifest.contains("test-fixtures = []"),
        "Compromise #7 closure: benten-graph Cargo.toml must declare \
         `test-fixtures = []` under `[features]`"
    );
    assert!(
        manifest.contains("default = [\"test-fixtures\"]"),
        "Compromise #7 closure: `test-fixtures` must be default so \
         `cargo test` / `cargo nextest run --workspace` build the bin"
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
