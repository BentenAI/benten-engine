//! R3 Family D RED-PHASE pin for G23-A grep-assert: `register_subgraph`
//! signature unchanged (arch-r1-15).
//!
//! Pin source: r2-test-landscape §2.4 row 7.
//!
//! ## Defense
//!
//! Grep-asserts the public-API surface of `Engine::register_subgraph` is the
//! same shape post-G23-A as pre-G23-A: signature
//! `pub fn register_subgraph(&mut self, spec: impl IntoSubgraphSpec) -> Result<...>`
//! (or equivalent). Catches the failure mode where G23-A adds a schema-only
//! parallel registration surface that would fork the registration path.

#![allow(clippy::unwrap_used)]

// Un-ignored at G23-A wave-4 (2026-05-12 canary).
//
// SHAPE (grep the engine source for the canonical signature) +
// SUBSTANCE (assert no parallel `register_subgraph_from_schema` /
// `register_schema_subgraph` / `register_typed_schema` /
// `register_compiled_schema` surface) per §3.6f.
#[test]
fn schema_compiler_does_not_widen_register_subgraph_signature() {
    let engine_src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("benten-engine")
        .join("src")
        .join("engine.rs");
    let body = std::fs::read_to_string(&engine_src)
        .unwrap_or_else(|e| panic!("read {}: {}", engine_src.display(), e));

    // SHAPE: the canonical `pub fn register_subgraph<S>` signature must
    // exist (at least once; exactly-one is enforced by cargo-public-api
    // drift, not here).
    assert!(
        body.contains("pub fn register_subgraph<S>(&self, spec: S) -> Result<String, EngineError>"),
        "canonical `pub fn register_subgraph<S>(&self, spec: S) -> Result<String, EngineError>` \
         must exist in crates/benten-engine/src/engine.rs (arch-r1-15)"
    );

    // SUBSTANCE: no parallel schema-registration surfaces. Companion
    // surfaces `register_subgraph_replace` + `register_subgraph_aggregate`
    // are legitimate (Phase-2b + Phase-3 work) and are NOT flagged here.
    for forbidden in [
        "register_subgraph_from_schema",
        "register_schema_subgraph",
        "register_typed_schema",
        "register_compiled_schema",
    ] {
        assert!(
            !body.contains(forbidden),
            "schema compiler must not introduce parallel registration surface; \
             engine.rs contains forbidden symbol `{forbidden}`"
        );
    }
}
