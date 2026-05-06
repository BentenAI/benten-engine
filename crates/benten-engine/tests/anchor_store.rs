//! G14-C wave-4b: anchor-store consolidation closed (cov-f3 +
//! phase-2-backlog §6.3).
//!
//! Pin source: r2-test-landscape §2.2 G14-C row
//! `anchor_store_consolidation_cov_f3_no_residual`.
//!
//! ## Architectural intent
//!
//! Phase-2 left a tracked residual where multiple ad-hoc anchor /
//! version-chain accessors lived across benten-engine + benten-graph.
//! G14-C consolidates: the engine exposes a single
//! [`Engine::anchor_store`] handle backed by the single canonical
//! `core::version::Anchor` shape.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_eval::SubgraphBuilder;
use benten_eval::{SubgraphBuilderExt, SubgraphExt};

fn build_handler(handler_id: &str, label: &str) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new(handler_id);
    let r = sb.read(label);
    sb.respond(r);
    sb.build_validated().expect("must build")
}

#[test]
fn anchor_store_consolidation_cov_f3_no_residual() {
    // (1) The consolidated API exists at exactly one site:
    //     `crates/benten-engine/src/anchor_store.rs`. The previous
    //     residual (cov-f3) named the absence of a single accessor.
    let engine_src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let anchor_store_files: Vec<_> = std::fs::read_dir(&engine_src_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("anchor_store"))
        .collect();
    assert_eq!(
        anchor_store_files.len(),
        1,
        "cov-f3: anchor-store implementation MUST live at exactly one site; \
         got {} files: {:?}",
        anchor_store_files.len(),
        anchor_store_files
            .iter()
            .map(|e| e.file_name())
            .collect::<Vec<_>>()
    );

    // (2) The consolidated API is consumed by Engine + handler-
    //     version chain queries via a uniform handle.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("anchor.redb")).unwrap();

    // Empty store before any registrations.
    let chains = engine.anchor_store().list_handler_chains().unwrap();
    assert!(chains.is_empty(), "fresh engine has no handler chains");

    // Register a handler; the consolidated API surfaces its chain.
    let sg = build_handler("demo:create_post", "post");
    let v1_cid = sg.cid().unwrap();
    engine.register_subgraph(sg).unwrap();

    let store = engine.anchor_store();
    let chain = store
        .fetch_handler_chain("demo:create_post")
        .expect("registered handler MUST surface a chain through anchor_store");
    assert_eq!(chain.versions(), &[v1_cid]);
    let anchor = store
        .fetch_handler_anchor("demo:create_post")
        .expect("non-empty chain has an anchor");
    assert_eq!(
        anchor.head, v1_cid,
        "anchor head equals the chain root (oldest version)"
    );

    // (3) `list_handler_chains` enumerates every registered handler.
    let chains = store.list_handler_chains().unwrap();
    assert_eq!(chains.len(), 1);
    assert_eq!(chains.get("demo:create_post").unwrap(), &vec![v1_cid]);
}
