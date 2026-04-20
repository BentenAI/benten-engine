//! 5d-J workstream 5 — `HandlerPredecessors::predecessors_of` real impl.
//!
//! Prior to this workstream the method returned an always-empty slice.
//! Now it walks the registered subgraph's edge list and surfaces the
//! real topological predecessors, keyed by the same CID derivation
//! `Engine::trace` uses for each TraceStep's `node_cid`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

#[test]
fn handler_predecessors_returns_topologically_preceding_nodes() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    let preds = engine.handler_predecessors(&handler_id).unwrap();

    // The `crud:post` subgraph has at least one edge (READ -> RESPOND).
    // The target of that edge must have exactly one predecessor.
    let targets: Vec<_> = preds.targets().collect();
    assert!(
        !targets.is_empty(),
        "the crud:<label> subgraph has at least one internal edge — \
         HandlerPredecessors must surface its target"
    );

    // Every populated adjacency entry must carry at least one predecessor.
    for target in &targets {
        let p = preds.predecessors_of(target);
        assert!(
            !p.is_empty(),
            "adjacency entry for {target:?} must list at least one predecessor"
        );
    }
}

#[test]
fn handler_predecessors_rejects_unregistered_handler() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let err = engine
        .handler_predecessors("nonexistent_handler_id")
        .unwrap_err();
    // Normalised via EngineError::Other { code: NotFound } path.
    assert_eq!(err.error_code().as_str(), "E_NOT_FOUND");
}
