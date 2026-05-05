//! R3-A RED-PHASE pin: `benten-id` dependency-edge architectural
//! constraint (G14-A1 wave-4a; arch-r1-10).
//!
//! Pin source: r2-test-landscape §2.2 G14-A1 row
//! `benten_id_no_unauthorized_dependency_edges`; arch-r1-10.
//!
//! ## Architectural constraint
//!
//! `benten-id` is the new 9th workspace crate landing at G14-A1. To keep
//! the dependency graph layered (per `docs/ARCHITECTURE.md` thinness
//! contract), `benten-id` MUST NOT depend on:
//!
//! - `benten-graph` (storage layer — `benten-id` is identity, not storage).
//! - `benten-engine` (orchestrator — `benten-id` is consumed BY the engine).
//! - `benten-eval` (evaluator — `benten-id` is consumed BY the evaluator).
//!
//! The expected dependency manifest (per plan §3 G14-A1 row):
//!
//! ```text
//! [dependencies]
//! ed25519-dalek = "2"
//! ssi = "0.7"
//! blake3 = ...
//! serde_ipld_dagcbor = ...
//! zeroize = ...
//! secrecy = ...
//! subtle = ...
//! getrandom = ...
//! ```
//!
//! Plus optionally `benten-core` (for `Cid` reuse) and `benten-errors`
//! (for typed-error mapping). NO other workspace crate names.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A1 — arch-r1-10 — dependency edges constrained"]
fn benten_id_no_unauthorized_dependency_edges() {
    // G14-A1 implementer wires this against the post-implementation
    // crates/benten-id/Cargo.toml. The test reads the manifest and
    // asserts the dependency table contains ONLY the allow-list.
    //
    // Concrete shape:
    //   let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("Cargo.toml");
    //   let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    //   let toml: toml::Value = manifest.parse().unwrap();
    //   let deps = toml.get("dependencies")
    //       .and_then(|d| d.as_table())
    //       .map(|t| t.keys().cloned().collect::<Vec<_>>())
    //       .unwrap_or_default();
    //   const FORBIDDEN: &[&str] = &["benten-graph", "benten-engine", "benten-eval"];
    //   for dep in &deps {
    //       for forbidden in FORBIDDEN {
    //           assert!(dep != forbidden,
    //               "benten-id MUST NOT depend on {} per arch-r1-10", forbidden);
    //       }
    //   }
    //
    // OBSERVABLE consequence: a future refactor that adds
    // `benten-engine` to benten-id's dep manifest fails this test
    // loudly, preventing the layering inversion before it lands.
    unimplemented!(
        "G14-A1 wires Cargo.toml manifest grep against {{benten-graph, benten-engine, benten-eval}} forbidden list"
    );
}
