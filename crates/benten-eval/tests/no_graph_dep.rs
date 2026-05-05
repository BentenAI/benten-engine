//! R3-A RED-PHASE pin: `benten-eval` has NO dependency on `benten-graph`
//! after G13-A umbrella-trait extraction (G13-A wave-1; arch-1).
//!
//! Pin source: r2-test-landscape §2.1 G13-A row
//! `arch_1_benten_eval_no_graph_dep_post_g13`; plan §3 G13-A.
//!
//! ## Architectural constraint
//!
//! After G13-A lands the `GraphBackend` umbrella trait in `benten-graph`,
//! the layering should be:
//!
//! ```text
//!   benten-eval                <- 12 primitives, evaluator
//!     |
//!     v depends on
//!   benten-engine              <- engine orchestrator (Engine<B: GraphBackend>)
//!     |
//!     v depends on
//!   benten-graph               <- GraphBackend trait + RedbBackend / BrowserBackend impls
//! ```
//!
//! `benten-eval` MUST NOT depend on `benten-graph` directly — it
//! receives storage indirectly via the engine's host interface
//! (`PrimitiveHost<B>`) which itself parameterizes over the graph
//! backend.
//!
//! ## Already-shipped state
//!
//! Phase-1 already ships this layering (see Phase-1 R7 audit). This
//! test is the REGRESSION GUARD against G13-B / G14-* refactors
//! accidentally pulling `benten-graph` into eval's dep tree.

#![allow(clippy::unwrap_used)]

#[test]
fn arch_1_benten_eval_no_graph_dep_post_g13() {
    // Read the workspace `Cargo.toml`s for benten-eval and assert
    // `benten-graph` does not appear in `[dependencies]` (or any
    // target-conditional dependency table).
    //
    // Note: this test is RUNNABLE NOW (not #[ignore]'d) because the
    // Phase-1 layering already holds the constraint. It serves as a
    // regression guard for G13-B / G14-* / G16-* waves that might
    // otherwise drag benten-graph into eval's deps.
    let manifest = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .expect("read benten-eval/Cargo.toml");

    // Strip out comment lines so `# benten-graph = ...` examples in
    // doc-comments don't trigger the assertion.
    let stripped: String = manifest
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    // The assertion is conservative: any `benten-graph` mention outside
    // a comment fails. Phase-1 + Phase-2b ship without it; G13-A
    // landing should preserve the absence.
    assert!(
        !stripped.contains("benten-graph"),
        "benten-eval MUST NOT depend on benten-graph (per arch-1 + plan §3 G13-A); \
         the GraphBackend trait extraction must not invert the layering. \
         Found 'benten-graph' in benten-eval/Cargo.toml — investigate the regression."
    );
}
