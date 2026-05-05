//! R3-C RED-PHASE pin for `benten-sync` dependency-edge architectural
//! constraint (G16-A wave-6 canary; arch-r1-11 + D-PHASE-3-14).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-A row
//!   `benten_sync_no_dependency_on_benten_engine_or_eval`.
//! - `arch-r1-11` (architectural constraint: dependency direction
//!   is engine → sync, never the reverse).
//! - `D-PHASE-3-14` (per-NEW-crate dep-edge audit).
//! - plan §3 G16-A row line "deps `iroh`, `iroh-net`, `tokio`,
//!   `benten-graph`, `benten-id`, `uhlc`; **NO benten-engine /
//!   benten-eval dep per arch-r1-11**".
//!
//! ## Architectural constraint
//!
//! `benten-sync` is the new 10th workspace crate landing at G16-A.
//! To keep the dependency graph layered, `benten-sync` MUST NOT
//! depend on:
//!
//! - `benten-engine` (orchestrator — `benten-sync` is consumed BY
//!   the engine via `engine.atrium.*` surface, not the reverse).
//! - `benten-eval` (evaluator — `benten-sync` is consumed BY the
//!   evaluator's primitive arms, not the reverse).
//!
//! The expected dependency manifest (per plan §3 G16-A row):
//!
//! ```text
//! [dependencies]
//! iroh = "..."
//! iroh-net = "..."
//! tokio = "..."
//! benten-graph = ...
//! benten-id = ...
//! uhlc = "..."
//! # G16-B adds: loro = "..."
//! # plus optionally benten-core (for Cid reuse) and benten-errors.
//! ```
//!
//! NO other workspace crate names (specifically: NO benten-engine, NO benten-eval).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A wave-6 fills Cargo.toml deps; arch-r1-11 audit at landing time"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — arch-r1-11 + D-PHASE-3-14 — dependency edges constrained"]
fn benten_sync_no_dependency_on_benten_engine_or_eval() {
    // arch-r1-11 + D-PHASE-3-14 pin. G16-A implementer wires this
    // against the post-implementation crates/benten-sync/Cargo.toml.
    // The test reads the manifest and asserts the dependency table
    // (and target-specific dep tables) contains NO forbidden
    // workspace-crate names.
    //
    // Concrete shape:
    //   let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("Cargo.toml");
    //   let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    //   const FORBIDDEN: &[&str] = &["benten-engine", "benten-eval"];
    //   for forbidden in FORBIDDEN {
    //       assert!(
    //           !manifest.contains(forbidden),
    //           "benten-sync MUST NOT depend on {} per arch-r1-11 + D-PHASE-3-14 (dependency direction is engine → sync, never reverse)",
    //           forbidden
    //       );
    //   }
    //
    // (More precise: parse Cargo.toml as TOML and walk every
    // dependency table including target-specific tables. The naive
    // string-grep above suffices for the architectural pin since
    // the forbidden names don't appear elsewhere in a valid
    // benten-sync manifest.)
    //
    // OBSERVABLE consequence: a future refactor that adds
    // `benten-engine` or `benten-eval` to benten-sync's dep
    // manifest fails this test loudly, preventing the layering
    // inversion before it lands.
    unimplemented!(
        "G16-A wires Cargo.toml manifest grep against {{benten-engine, benten-eval}} forbidden list"
    );
}
