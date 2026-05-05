//! R3-A RED-PHASE pins for `EngineGeneric<B>` cascade (G13-B wave 2).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + plan §3 G13-B
//! must-pass column):
//!
//! - `tests/engine_generic_compiles_with_redb_default` — plan §3 G13-B
//! - `tests/engine_generic_cascade_no_inherent_redb_references_outside_default_alias` — D-PHASE-3-1
//!
//! ## What G13-B introduces
//!
//! `crates/benten-engine/src/engine.rs` introduces:
//!
//! ```text
//! pub struct EngineGeneric<B: GraphBackend> { ... }
//! pub type Engine = EngineGeneric<RedbBackend>;
//! ```
//!
//! ~16 impl blocks cascade through the new generic parameter. The default
//! alias `Engine = EngineGeneric<RedbBackend>` preserves API stability.
//! The browser-target binding ships `Engine = EngineGeneric<BrowserBackend>`
//! per cargo feature `browser-backend`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-B wave 2 introduces EngineGeneric<B>"]
fn engine_generic_compiles_with_redb_default() {
    // G13-B implementer wires this:
    //   use benten_engine::{Engine, EngineGeneric};
    //   use benten_graph::RedbBackend;
    //
    //   // Default alias resolves to EngineGeneric<RedbBackend>:
    //   fn assert_alias_is_redb_specialization()
    //   where
    //       Engine: From<EngineGeneric<RedbBackend>>,  // approximate compile pin
    //   {}
    //   assert_alias_is_redb_specialization();
    //
    //   // Open via the default alias compiles + runs:
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine = Engine::open(dir.path()).unwrap();
    //   drop(engine);
    //
    // OBSERVABLE consequence: the default `Engine::open(path)` call
    // path continues to work after the generic-cascade lands.
    // Defends against G13-B accidentally breaking API stability for
    // existing callers.
    unimplemented!("G13-B wires Engine default-alias compile + open assertion");
}

#[test]
#[ignore = "RED-PHASE: G13-B — D-PHASE-3-1 — no inherent RedbBackend refs outside default alias"]
fn engine_generic_cascade_no_inherent_redb_references_outside_default_alias() {
    // D-PHASE-3-1 RESOLVED pin. After G13-B cascades the generic
    // parameter, NO impl block on `EngineGeneric<B>` should reference
    // `RedbBackend` directly — every method that needs a backend
    // operation goes through the `B: GraphBackend` bound.
    //
    // Two sites are EXEMPT (the only allowed `RedbBackend` references):
    //
    // 1. `pub type Engine = EngineGeneric<RedbBackend>;` — the default
    //    alias.
    // 2. `impl Engine { pub fn open(path: P) -> ... { ... } }` —
    //    convenience constructor specialized for the redb path; the
    //    generic version takes a pre-constructed `B` value.
    //
    // G13-B implementer wires this:
    //   let src = std::fs::read_to_string("crates/benten-engine/src/engine.rs").unwrap();
    //   for (lineno, line) in src.lines().enumerate() {
    //       if line.contains("RedbBackend") {
    //           // Allow only on the default-alias line + Engine::open
    //           // specialized impl.
    //           let allowed = line.contains("pub type Engine =")
    //                      || line.contains("impl Engine");
    //           assert!(allowed, "engine.rs:{} cites RedbBackend outside default-alias \
    //                            scope (D-PHASE-3-1 violation): {}", lineno + 1, line);
    //       }
    //   }
    //
    // OBSERVABLE consequence: a future PR that hard-codes RedbBackend
    // in some new method body fails this test, forcing the author to
    // either add a `B: GraphBackend` bound or move to the specialized
    // `impl Engine { ... }` block.
    unimplemented!(
        "G13-B wires source-grep assertion that RedbBackend appears only in default alias"
    );
}
