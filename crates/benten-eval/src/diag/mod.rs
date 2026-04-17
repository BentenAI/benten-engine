//! Developer-experience diagnostics (G6-C, E7 + E8).
//!
//! Phase-1 deliverables per `docs/QUICKSTART.md`:
//!
//! - `subgraph.to_mermaid()` — render any operation subgraph as a Mermaid
//!   flowchart so the dev can `paste into github and see their handler`.
//!   Pure function, no evaluation required. See [`mermaid`].
//! - `engine.trace(handler, input)` — step-by-step evaluation trace with
//!   per-node microsecond timings. Backed by
//!   [`Evaluator::run_with_trace`](crate::Evaluator::run_with_trace) and
//!   rendered by [`trace`].
//!
//! Both are behind the `diag` Cargo feature so the thin-engine path
//! (benten-core + benten-graph + benten-engine with `NoAuthBackend`, no IVM
//! subscribers, and `default-features = false`) compiles without the
//! diagnostic surface. `benten-engine` enables the feature via its own
//! `default` feature list — every real user gets diagnostics, but a minimal
//! embedder can opt out.
//!
//! Per the R5 LOC budget, `src/diag/**` stays under 500 lines; both modules
//! deliberately keep rendering logic simple (no external template engine).

#[cfg(feature = "diag")]
pub mod mermaid;
#[cfg(feature = "diag")]
pub mod trace;

#[cfg(not(feature = "diag"))]
pub mod mermaid {
    //! Stub `mermaid` module for slim builds — empty rendering.
    use crate::Subgraph;

    /// Rendering entry point. With the `diag` feature disabled this returns
    /// an empty string so callers in slim builds produce no-op output.
    #[must_use]
    pub fn render(_sg: &Subgraph) -> String {
        String::new()
    }
}

#[cfg(not(feature = "diag"))]
pub mod trace {
    //! Stub `trace` module for slim builds.
}
