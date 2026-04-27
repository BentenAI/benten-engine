//! Phase 2a R4 qa-r4-8 / dx-r1-9: WAIT renders as a stadium-shape node
//! (`([text])`) with a dashed `-.->` resume edge labelled `on resume`.
//!
//! Partners with the TS side in `packages/engine/src/mermaid.test.ts`.
//!
//! TDD red-phase: G3-B lands the WAIT-specific rendering delta. Until
//! then `subgraph.to_mermaid()` emits a generic stadium-shape node with no
//! dashed resume edge, so both assertions here fail.
//!
//! R4 fix-pass: gated under `phase_2a_pending_apis` because the test body
//! needs the `.wait()` closure-style `SubgraphSpecBuilder` method which
//! G3-B lands. See `wait_resume_determinism.rs` header for the same gate
//! rationale.

#![cfg(feature = "phase_2a_pending_apis")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, SubgraphSpec};
use benten_eval::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn wait_renders_as_stadium_with_dashed_resume_edge() {
    let (_dir, engine) = fresh_engine();

    // Minimal WAIT-composing handler: WAIT → RESPOND. The rendered output
    // must contain the stadium shape for the WAIT node and a dashed edge
    // from the WAIT node to the RESPOND node labelled `on resume`.
    let spec = SubgraphSpec::builder()
        .handler_id("mermaid_wait:simple")
        .wait(|w| w.signal("external:ping"))
        .respond(|r| r.body("$result"))
        .build();
    let handler_id = engine
        .register_subgraph(spec)
        .expect("register simple WAIT-respond handler");

    let mermaid = engine
        .handler_to_mermaid(&handler_id)
        .expect("handler_to_mermaid for registered WAIT handler");

    // Stadium shape `([WAIT: ...])` — Phase-1 Rust already renders every
    // node in stadium form, so this assertion catches a regression where
    // a future refactor drops WAIT out of stadium form.
    assert!(
        mermaid.contains("([\"WAIT") || mermaid.contains("([WAIT"),
        "WAIT node must render in stadium form `([WAIT...])`; got:\n{mermaid}"
    );

    // Dashed resume edge — G3-B lands this. `-.->` is Mermaid's dashed
    // arrow; `on resume` is the label dx-r1-9 picked. Red-phase until G3-B.
    assert!(
        mermaid.contains("-.->") && mermaid.contains("on resume"),
        "WAIT rendering must emit a dashed `-.->` resume edge labelled \
         `on resume` (dx-r1-9); got:\n{mermaid}"
    );
}
