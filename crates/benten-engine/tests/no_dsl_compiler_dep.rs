//! arch-N pin: `benten-engine` MUST NOT depend on `benten-dsl-compiler` in
//! Phase 2b — the compiler is a sibling-of-engine crate consumed by
//! `tools/benten-dev` directly. Per `r1-architect-reviewer.json` G12-B-scope
//! item (e): "Devserver consumes `benten-dsl-compiler` directly (not via
//! `benten-engine`); `benten-engine` does NOT take a `benten-dsl-compiler`
//! dep in 2b — keeps the dep edge optional."
//!
//! TDD red-phase: scans `crates/benten-engine/Cargo.toml` for
//! `benten-dsl-compiler` entries; asserts none. Lifts to green when G12-B
//! lands the compiler crate without wiring it into benten-engine.
//!
//! Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-B red-phase: Cargo.toml scan not yet implemented"]
fn benten_engine_does_not_depend_on_benten_dsl_compiler() {
    todo!(
        "R5 G12-B: read CARGO_MANIFEST_DIR/Cargo.toml; \
           assert no entry matching `benten-dsl-compiler` in [dependencies], \
           [dev-dependencies], or [build-dependencies]"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: feature-flag scan not yet implemented"]
fn benten_engine_register_handler_from_str_not_publicly_surfaced_in_phase_2b() {
    // Per arch-reviewer: the optional `register_handler_from_str` API is
    // explicitly NOT shipped in 2b — keeps the cargo-public-api baseline narrow.
    todo!(
        "R5 G12-B: assert benten_engine::Engine has no public `register_handler_from_str` method exposed in 2b"
    )
}
