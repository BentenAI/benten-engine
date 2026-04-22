//! Phase 2a R3 integration — `WriteContext::privileged` bool → `WriteAuthority`
//! enum migration.
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G2-B
//! (`write_authority_enum_discriminates_user_engine_sync`) + §9.11 row 4
//! (SyncReplica) + §8 cross-2a/2b frozen interface item 8 + ucca-9 /
//! arch-r1-2 (enum lift).
//!
//! Phase-1 exposed `WriteContext::is_privileged: bool`. Phase 2a G2-B lifts
//! the two-valued field to a three-valued enum:
//!
//!     WriteAuthority::{ User, EnginePrivileged, SyncReplica { origin_peer } }
//!
//! Owned by `qa-expert` per R2 landscape §8.5. TDD red-phase.

#![cfg(feature = "phase_2a_pending_apis")]
// R4 fix-pass: gated under `phase_2a_pending_apis` until G2-B lands the
// `WriteContext::synthetic_for_test` + closure-style SubgraphBuilder
// methods (`.write(|w| ...)`, `.respond(|r| ...)`). The file still
// compiles standalone under the feature once R5 lands the APIs.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{WriteAuthority, WriteContext};
use benten_core::{Cid, Node, Value};
use benten_engine::{Engine, SubgraphSpec};
use std::collections::BTreeMap;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// User-authority writes behave as they did in Phase 1: a system-zone
/// write by a User is rejected; a non-system write by a User commits.
#[test]
fn write_authority_user_preserves_phase1_semantics() {
    let (_dir, engine) = fresh_engine();
    let sg = SubgraphSpec::builder()
        .handler_id("wa:user_benign")
        .write(|w| w.label("post").property("title", Value::Text("u".into())))
        .respond(|r| r.body("$result"))
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();
    let outcome = engine
        .call(&handler_id, "wa:run", Node::empty())
        .expect("user-label write under User authority commits");
    assert!(outcome.is_ok_edge());

    let forbidden = SubgraphSpec::builder()
        .handler_id("wa:user_system_zone")
        .write(|w| w.label("system:CapabilityGrant"))
        .respond(|r| r.body("$result"))
        .build();
    let forbidden_id = engine.register_subgraph(forbidden).unwrap();
    let denied = engine.call(&forbidden_id, "wa:run", Node::empty()).unwrap();
    assert_eq!(
        denied.error_code(),
        Some("E_SYSTEM_ZONE_WRITE"),
        "User-authority system-zone write must be rejected (old is_privileged = false semantics)"
    );
}

/// EnginePrivileged writes behave as the old `is_privileged = true` path.
#[test]
fn write_authority_engine_privileged_preserves_phase1_semantics() {
    let (_dir, engine) = fresh_engine();
    let alice = engine.create_principal("alice").unwrap();
    engine
        .grant_capability(&alice, "store:post:write")
        .expect("engine-privileged system-zone write must succeed");
}

/// `WriteAuthority::SyncReplica { origin_peer }` — Phase-3 reserved shape.
///
/// SHAPE-PIN: validates the struct shape for Phase-3 forward-compat.
/// Does NOT validate firing semantics (those land in Phase 3 sync).
#[test]
fn write_authority_sync_replica_shape_round_trips() {
    let origin = Cid::from_blake3_digest(blake3::hash(b"phase-3-remote-peer-public-key").into());
    let authority = WriteAuthority::SyncReplica {
        origin_peer: origin.clone(),
    };
    // DAG-CBOR round-trip via `serde_ipld_dagcbor` (the same encoder
    // Phase-3 sync will use). G2-B landing the enum shape includes
    // Serialize + Deserialize derives so this round-trip works.
    let encoded = serde_ipld_dagcbor::to_vec(&authority).expect("encode");
    let decoded: WriteAuthority = serde_ipld_dagcbor::from_slice(&encoded).expect("decode");
    match decoded {
        WriteAuthority::SyncReplica {
            origin_peer: round_tripped,
        } => assert_eq!(round_tripped, origin),
        other => panic!("SyncReplica payload mismatch: got {other:?}"),
    }
}

/// `WriteContext::authority` enum field presence check.
#[test]
fn write_context_authority_field_discriminates_three_variants() {
    let mut ctx = WriteContext::synthetic_for_test();
    ctx.authority = WriteAuthority::User;
    assert!(matches!(ctx.authority, WriteAuthority::User));
    ctx.authority = WriteAuthority::EnginePrivileged;
    assert!(matches!(ctx.authority, WriteAuthority::EnginePrivileged));
    ctx.authority = WriteAuthority::SyncReplica {
        origin_peer: Cid::from_blake3_digest(blake3::hash(b"peer").into()),
    };
    assert!(matches!(ctx.authority, WriteAuthority::SyncReplica { .. }));
}

/// Enumerated round-trip over the three variants + two SyncReplica seeds.
#[test]
fn proptest_write_authority_all_variants_round_trip() {
    let shapes: [WriteAuthority; 4] = [
        WriteAuthority::User,
        WriteAuthority::EnginePrivileged,
        WriteAuthority::SyncReplica {
            origin_peer: Cid::from_blake3_digest(blake3::hash(b"peer-alpha").into()),
        },
        WriteAuthority::SyncReplica {
            origin_peer: Cid::from_blake3_digest(blake3::hash(b"peer-beta-different").into()),
        },
    ];
    for shape in shapes {
        let bytes = serde_ipld_dagcbor::to_vec(&shape).expect("encode");
        let back: WriteAuthority = serde_ipld_dagcbor::from_slice(&bytes).expect("decode");
        assert_eq!(back, shape);
    }

    let _ = (Node::empty(), BTreeMap::<String, Value>::new());
}
