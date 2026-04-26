//! arch-N pin: `benten-dsl-compiler` dep direction.
//!
//! Per `r1-architect-reviewer.json` D-point G12-B-scope + plan §3.2 G12-B +
//! `00-implementation-plan.md` line 582: `benten-dsl-compiler` sits as a
//! sibling of `benten-engine`, depends on `benten-core` for `Subgraph` /
//! `SubgraphSpec` types, and **MUST NOT depend on `benten-eval` or
//! `benten-graph`** — preserves arch-1.
//!
//! TDD red-phase: file scans `Cargo.toml` for forbidden dep entries and
//! asserts the allowed set. Lifts to green when G12-B R5 implementer wires
//! the real parser keeping the dep set narrow.
//!
//! Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-B red-phase: dep-direction scan against Cargo.toml not yet implemented"]
fn benten_dsl_compiler_does_not_depend_on_benten_eval() {
    todo!(
        "R5 G12-B: parse Cargo.toml [dependencies] + [dev-dependencies] + [build-dependencies]; assert no `benten-eval` entry"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: dep-direction scan not yet implemented"]
fn benten_dsl_compiler_does_not_depend_on_benten_graph() {
    todo!("R5 G12-B: parse Cargo.toml; assert no `benten-graph` entry")
}

#[test]
#[ignore = "R5 G12-B red-phase: dep-direction scan not yet implemented"]
fn benten_dsl_compiler_does_not_depend_on_benten_engine() {
    todo!(
        "R5 G12-B: parse Cargo.toml; assert no `benten-engine` entry — compiler is sibling, not child"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: dep-direction scan not yet implemented"]
fn benten_dsl_compiler_depends_on_benten_core_for_subgraph_types() {
    todo!("R5 G12-B: parse Cargo.toml; assert `benten-core` IS present in [dependencies]")
}
