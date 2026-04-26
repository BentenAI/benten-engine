//! G12-C red-phase: assert `benten-eval` has NO `pub struct Subgraph`
//! definition after the type migration to `benten-core` — only a re-export.
//!
//! Per plan §3.2 G12-C: "delete eval-side `Subgraph` + the two `todo!()` stubs
//! at `:859,867`; re-export from `benten-core`."
//!
//! The test scans the `benten-eval` source tree for forbidden patterns:
//!   - `pub struct Subgraph {`  (definition; should be 0 occurrences)
//!   - `pub struct SubgraphBuilder {`  (definition; should be 0 occurrences)
//! And asserts a re-export pattern is present:
//!   - `pub use benten_core::{... Subgraph ...}` (or equivalent).
//!
//! This is a static-source scan, NOT a type-system check — the type-system
//! check rides on `crates/benten-engine/src/subgraph_spec.rs` already pointing
//! at the canonical type post-migration.
//!
//! TDD red-phase. Owner: R5 G12-C (qa-r4-02 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-C red-phase: source-tree scan not yet implemented"]
fn benten_eval_does_not_define_subgraph_struct() {
    // Walk `crates/benten-eval/src/**/*.rs`; assert no line matches the
    // pattern `^\s*pub\s+struct\s+Subgraph\s*[\{<]` (definition, not re-export).
    todo!(
        "R5 G12-C: walk benten-eval src tree; \
         assert zero `pub struct Subgraph` definitions remain post-migration"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: source-tree scan not yet implemented"]
fn benten_eval_does_not_define_subgraph_builder_struct() {
    todo!(
        "R5 G12-C: walk benten-eval src tree; \
         assert zero `pub struct SubgraphBuilder` definitions remain"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: re-export presence not yet asserted"]
fn benten_eval_re_exports_subgraph_from_benten_core() {
    // Pin: the migration replaces local definitions with `pub use benten_core::Subgraph;`
    // (or equivalent), so existing `benten_eval::Subgraph` callsites keep working.
    todo!(
        "R5 G12-C: scan benten-eval/src/lib.rs for `pub use benten_core::` containing Subgraph; \
         assert at least one re-export line is present"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: todo!() stub residual scan not yet wired"]
fn benten_eval_subgraph_todo_stubs_at_old_lines_859_867_removed() {
    // Per plan §3.2 G12-C explicit cleanup: "delete eval-side Subgraph + the
    // two `todo!()` stubs at :859,867". This test pins their absence.
    todo!(
        "R5 G12-C: scan benten-eval/src/lib.rs for `todo!()` calls inside any \
         `impl Subgraph` block; assert zero remain"
    )
}
