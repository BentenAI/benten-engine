//! Phase 1 R3 integration — engine-level cross-process CID determinism.
//!
//! Partner to `crates/benten-graph/tests/d2_cross_process_graph.rs` (lower
//! layer). This file tests through the Engine public API to confirm the
//! full orchestrator path (benten-engine -> benten-graph -> redb -> reverse)
//! preserves the canonical CID.
//!
//! **Status:** PASSING against the spike shape today; protects post-G7 reshape.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;
use benten_engine::Engine;

const CANONICAL_CID: &str = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";

#[test]
fn engine_level_persisted_cid_deterministic_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let written = {
        let engine = Engine::builder().path(&db_path).build().unwrap();
        engine.create_node(&canonical_test_node()).unwrap()
    };
    assert_eq!(written.to_base32(), CANONICAL_CID);

    let engine = Engine::builder().path(&db_path).build().unwrap();
    let n = engine.get_node(&written).unwrap().expect("persisted");
    assert_eq!(n.cid().unwrap().to_base32(), CANONICAL_CID);
}
