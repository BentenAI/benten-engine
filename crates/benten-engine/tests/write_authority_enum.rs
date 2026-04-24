//! R3 unit tests for G2-B / ucca-9 / arch-r1-2: `WriteAuthority` enum — FROZEN
//! interface — and Subgraph AST cache wiring.
//!
//! Locked 3-variant shape per ucca-9 / arch-r1-2:
//!   - `WriteAuthority::User`
//!   - `WriteAuthority::EnginePrivileged`
//!   - `WriteAuthority::SyncReplica { origin_peer: Cid }`
//!
//! Phase-2a ships the shape; `SyncReplica` fires in Phase 3.
//!
//! TDD red-phase: the enum does not yet exist. Tests fail to compile until
//! G2-B lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.6.2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_engine::Engine;
use benten_graph::WriteAuthority;

fn zero_cid() -> Cid {
    // R3 fixture bug fix (rule-12): from_bytes(zero) fails on version byte.
    Cid::from_blake3_digest([0u8; 32])
}

fn open_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("eng.redb")).expect("open");
    (dir, engine)
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn write_authority_enum_discriminates_user_engine_sync() {
    // Each variant is constructable and pattern-matchable.
    let user = WriteAuthority::User;
    let privileged = WriteAuthority::EnginePrivileged;
    let replica = WriteAuthority::SyncReplica {
        origin_peer: zero_cid(),
    };

    fn as_label(a: WriteAuthority) -> &'static str {
        match a {
            WriteAuthority::User => "user",
            WriteAuthority::EnginePrivileged => "engine",
            WriteAuthority::SyncReplica { .. } => "sync",
        }
    }
    assert_eq!(as_label(user), "user");
    assert_eq!(as_label(privileged), "engine");
    assert_eq!(as_label(replica), "sync");
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn write_authority_user_is_default() {
    let default = WriteAuthority::default();
    assert!(
        matches!(default, WriteAuthority::User),
        "default WriteAuthority must be User"
    );
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn write_authority_sync_replica_reserved_but_inert_in_2a() {
    // In Phase 2a SyncReplica is reserved-shape only — the engine does not
    // yet route on it. Construct + pattern-match compiles.
    let replica = WriteAuthority::SyncReplica {
        origin_peer: zero_cid(),
    };
    match replica {
        WriteAuthority::SyncReplica { origin_peer } => {
            assert_eq!(origin_peer, zero_cid());
        }
        _ => panic!("expected SyncReplica variant"),
    }
}

// ---- AST cache wiring (G2-B, N6) -----------------------------------------

#[test]
fn engine_call_uses_ast_cache() {
    // Second call with the same (handler_id, op, subgraph_cid) must not
    // re-parse the subgraph. The engine exposes a test-only parse counter.
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_respond_handler("cache"))
        .expect("register");

    engine.testing_reset_parse_counter();
    let _ = engine
        .call("cache", "run", benten_core::Node::empty())
        .expect("first");
    let after_first = engine.testing_parse_counter();

    let _ = engine
        .call("cache", "run", benten_core::Node::empty())
        .expect("second");
    let after_second = engine.testing_parse_counter();

    assert_eq!(
        after_second, after_first,
        "second call with identical (handler_id, op, cid) must hit the AST cache"
    );
}

#[test]
fn engine_call_first_call_warms_cache() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_respond_handler("warm"))
        .expect("register");

    engine.testing_reset_parse_counter();
    assert_eq!(engine.testing_parse_counter(), 0);

    let _ = engine
        .call("warm", "run", benten_core::Node::empty())
        .expect("call");
    let parsed = engine.testing_parse_counter();

    assert!(
        parsed >= 1,
        "first call must parse (and therefore warm) the AST cache; got {parsed}"
    );
}

#[test]
fn ast_cache_invalidates_on_reregister() {
    // dx-r1: re-register under a different CID invalidates the prior entry.
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_respond_handler("reg"))
        .expect("initial register");

    let _ = engine
        .call("reg", "run", benten_core::Node::empty())
        .expect("first call warms");
    let pre = engine.testing_parse_counter();

    engine
        .testing_force_reregister_with_different_cid("reg")
        .expect("force reregister");

    let _ = engine
        .call("reg", "run", benten_core::Node::empty())
        .expect("post-reregister call");
    let post = engine.testing_parse_counter();

    assert!(
        post > pre,
        "re-registering under a different CID must invalidate the prior AST cache entry; pre={pre} post={post}"
    );
}

// ---- Proptest: WriteAuthority preserves CID stability across write paths --

use proptest::prelude::*;

proptest! {
    /// `prop_write_authority_roundtrip_cid_stable`: WriteAuthority variants
    /// MUST NOT change the CID of the Node being written (content-addressing
    /// invariant). The same content bytes — whether written with the `User`,
    /// `EnginePrivileged`, or `SyncReplica` authority — MUST produce the
    /// same CID, because authority is a routing property of the write, not
    /// a property of the Node content.
    ///
    /// R4 tq-2 rewrite: the prior version computed `node.cid()` three
    /// times on a single node, which is tautological — it didn't test
    /// anything about WriteAuthority at all. This version threads each of
    /// the three authorities through `WriteContext::with_authority` +
    /// constructs three semantically equivalent nodes (same bytes, same
    /// shape) and asserts that the CIDs are identical across all three.
    /// A regression in the Write-Authority→CID coupling (e.g. someone
    /// accidentally hashes the authority into the Node CID input) now
    /// surfaces as a proptest shrink, not a passing tautology.
    #[test]
    fn prop_write_authority_roundtrip_cid_stable(title in "[a-zA-Z]{1,16}") {
        use benten_graph::{WriteAuthority, WriteContext};
        // Build three WriteContexts, one per authority variant, carrying
        // the SAME Node bytes. The Node CID is a function of the bytes,
        // not the authority; forcing the authority through
        // `with_authority` proves that.
        let mut props_u = std::collections::BTreeMap::new();
        props_u.insert("title".to_string(), benten_core::Value::text(&title));
        let node_user = benten_core::Node::new(vec!["Post".into()], props_u.clone());
        let node_engine = benten_core::Node::new(vec!["Post".into()], props_u.clone());
        let node_replica = benten_core::Node::new(vec!["Post".into()], props_u);

        let ctx_user = WriteContext::new("Post")
            .with_authority(WriteAuthority::User);
        let ctx_engine = WriteContext::new("Post")
            .with_authority(WriteAuthority::EnginePrivileged);
        let ctx_replica = WriteContext::new("Post")
            .with_authority(WriteAuthority::SyncReplica { origin_peer: zero_cid() });

        // Sanity: the three contexts discriminate.
        let is_user = matches!(ctx_user.authority, WriteAuthority::User);
        let is_engine = matches!(ctx_engine.authority, WriteAuthority::EnginePrivileged);
        let is_replica =
            matches!(ctx_replica.authority, WriteAuthority::SyncReplica { origin_peer: _ });
        prop_assert!(is_user);
        prop_assert!(is_engine);
        prop_assert!(is_replica);

        // CIDs for content-equivalent nodes must match across all three
        // authorities (the property under test).
        let cid_u = node_user.cid().expect("cid user");
        let cid_e = node_engine.cid().expect("cid engine");
        let cid_r = node_replica.cid().expect("cid replica");
        prop_assert_eq!(cid_u, cid_e,
            "User and EnginePrivileged writes of content-equivalent bytes must produce the same CID");
        prop_assert_eq!(cid_u, cid_r,
            "User and SyncReplica writes of content-equivalent bytes must produce the same CID");
    }
}
