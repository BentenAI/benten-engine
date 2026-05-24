//! G-CORE-9 V1-FROZEN-INTERFACE row 4 / §1.A.FROZEN item 15(h) — end-to-end
//! pin for `Engine::walk_share_scope`.
//!
//! The engine wrapper delegates to
//! `benten_core::subgraph_spec::walker::walk` (the canonical BFS
//! enumerator); this test exercises the public engine consumer surface
//! end-to-end + asserts the BFS-canonical order matches the underlying
//! walker's output (the engine wrapper does NOT reimplement the walk —
//! that would risk producer/recipient enumeration drift per
//! RATIFIED-S&C §R4).
//!
//! ## What this would-FAIL on
//!
//! - If `Engine::walk_share_scope` shadows the walker's BFS order with
//!   a different ordering (e.g. DFS, lexical-by-CID), the assertion
//!   on `enumerated` would fire — would-FAIL signal for ordering
//!   regression.
//! - If the engine wrapper swallows `SubgraphSpecError` into `Ok`
//!   (instead of routing through `EngineError::Other` /
//!   `ErrorCode::SubgraphSpecWalkFailed`), the error-path arm would
//!   fail.
//! - The compile-time signature pin (`fn(&Engine, &Spec) ->
//!   Result<WalkResult, EngineError>`) catches v1-beta signature
//!   regression at the type level.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_core::subgraph_spec::{Spec, walker};
use benten_engine::Engine;
use benten_errors::ErrorCode;
use tempfile::tempdir;

fn cid_for(label: &str) -> Cid {
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}

#[test]
fn walk_share_scope_returns_walker_bfs_order() {
    let dir = tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let r = cid_for("g-core-9-r");
    let a = cid_for("g-core-9-a");
    let b = cid_for("g-core-9-b");

    let spec = Spec::builder()
        .with_root(r)
        .with_edge(r, "edge_A".to_string(), a)
        .with_edge(r, "edge_B".to_string(), b)
        .build()
        .expect("valid spec");

    let result = engine
        .walk_share_scope(&spec)
        .expect("walk_share_scope produces enumerated paths");

    // Engine wrapper output MUST equal the underlying walker's output
    // bytewise — the wrapper does NOT reimplement the BFS.
    let underlying = walker::walk(&spec).expect("underlying walker");
    let engine_cids: Vec<Cid> = result.enumerated.iter().map(|(c, _)| *c).collect();
    let underlying_cids: Vec<Cid> =
        underlying.enumerated.iter().map(|(c, _)| *c).collect();
    assert_eq!(
        engine_cids, underlying_cids,
        "Engine::walk_share_scope MUST equal benten_core walker output \
         bytewise — wrapper drift = HALT per V1-FROZEN-INTERFACE §15.h"
    );
}

#[test]
fn walk_share_scope_typed_reject_routes_through_error_code() {
    // A spec with a root CID not resolvable in the graph store is the
    // walker's documented failure mode (`SubgraphSpecError::CidMissing`
    // per `tf3w_walker_fail_closed_on_missing_cid.rs`). The engine
    // wrapper MUST route this through
    // `EngineError::Other(code = SubgraphSpecWalkFailed)`.
    let dir = tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let phantom_root = cid_for("g-core-9-phantom-root-never-stored");
    let spec = Spec::builder()
        .with_root(phantom_root)
        .build()
        .expect("Spec with phantom root constructs (validation deferred to walk)");

    let err = engine
        .walk_share_scope(&spec)
        .expect_err("walk_share_scope MUST reject phantom-root spec");

    match err {
        benten_engine::EngineError::Other { code, .. } => {
            assert_eq!(
                code,
                ErrorCode::SubgraphSpecWalkFailed,
                "typed reject MUST surface `SubgraphSpecWalkFailed` per §3.5g \
                 cross-language mirror discipline; collapsed `Unknown` = \
                 §3.5g item 6 violation"
            );
        }
        other => panic!(
            "expected EngineError::Other(SubgraphSpecWalkFailed), got: {other:?}"
        ),
    }
}
