//! Phase 2b R3-B — engine_sandbox public-surface integration tests (G7-C).
//!
//! Pin sources: plan §3 G7-C, dx-r1-2b SANDBOX.
//!
//! G7-C surface posture (dx-optimizer corrected):
//!   - DSL composition surface ONLY: `subgraph(...).sandbox({ module,
//!     manifest? | caps? })`.
//!   - NO top-level `engine.sandbox(...)` user-facing API — would
//!     bypass evaluator + Inv-4 + AttributionFrame plumbing.
//!   - Top-level engine surface for sandbox-related work is exclusively
//!     `engine.installModule(manifest, manifestCid)` /
//!     `engine.uninstallModule(cid)` (G10-B owned).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

#[test]
fn engine_sandbox_end_to_end_via_dsl_composition_only() {
    // **G20-A1 wave-8a body** (Phase 3): plan §3 G7-C — register a
    // SubgraphSpec via the DSL composition path
    // `subgraph('handler').sandbox({ module: cid, caps: [...] })`.
    // engine.call('handler', input) routes through the evaluator
    // which dispatches the SANDBOX primitive end-to-end through the
    // wasmtime executor.
    //
    // This is the load-bearing test that NO top-level
    // `engine.sandbox(...)` API exists (covered by the absence-pin
    // test below) AND the DSL composition path successfully invokes
    // the engine's `execute_sandbox` PrimitiveHost override.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 42))").unwrap();
    let module_cid = Cid::from_blake3_digest(*blake3::hash(&module_bytes).as_bytes());
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("module".into(), Value::Text(module_cid_str));
    props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );
    let spec = SubgraphSpec::builder()
        .handler_id("g20a1.engine_sandbox_e2e")
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: props,
        })
        .respond()
        .build();

    let handler_id = engine
        .register_subgraph(spec)
        .expect("DSL-composed SANDBOX SubgraphSpec MUST register on native");

    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect("DSL-composed SANDBOX dispatch through engine.call MUST succeed");
    assert!(
        outcome.is_ok_edge(),
        "DSL-composed SANDBOX call MUST route through OK edge end-to-end \
         via the evaluator's execute_sandbox primitive host override"
    );
}

#[test]
fn sandbox_no_top_level_engine_sandbox_call_site_exists() {
    // dx-r1-2b SANDBOX surface — anti-regression: the public Rust
    // engine surface (`benten_engine::Engine`) MUST NOT carry a
    // `sandbox` method. Only `install_module` / `uninstall_module`
    // (G10-B owned) and the internal `execute_sandbox_*` plumbing
    // (private).
    //
    // Source-grep absence pin via manual recursive walk over
    // `crates/benten-engine/src/` (avoids pulling `walkdir` as a
    // dev-dep just for this single test). Asserts no `pub fn sandbox(`
    // declaration exists. Sufficient for the dx-r1-2b corrected-
    // surface contract per HARD RULE (compile_fail / trybuild
    // harness reserved for Phase 3 if a deeper type-system
    // regression vector surfaces).
    fn walk(dir: &std::path::Path, hits: &mut Vec<String>) {
        let entries =
            std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, hits);
                continue;
            }
            if path.extension().is_some_and(|ext| ext == "rs") {
                let body = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
                for (lineno, line) in body.lines().enumerate() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("pub fn sandbox(")
                        || trimmed.starts_with("pub fn sandbox<")
                        || trimmed.starts_with("pub async fn sandbox(")
                    {
                        hits.push(format!("{}:{}: {}", path.display(), lineno + 1, line));
                    }
                }
            }
        }
    }
    let src_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offending = Vec::new();
    walk(&src_root, &mut offending);
    assert!(
        offending.is_empty(),
        "dx-r1-2b absence pin tripped: top-level `engine.sandbox(...)` surface MUST NOT exist on \
         `Engine`; found {} declaration(s):\n{}",
        offending.len(),
        offending.join("\n")
    );
}
