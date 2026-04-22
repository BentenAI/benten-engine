//! R3 unit test for G2-A: `CapabilityGrant` Node writes always use
//! `DurabilityMode::Immediate` regardless of the engine-configured durability
//! setting.
//!
//! Rationale: capability-grant content-hashing + privileged-path safety require
//! an immediate fsync so revocation ordering is not reordered by a grouped-
//! commit window. Survives the enum-preservation refactor above.
//!
//! TDD red-phase: the inspection API
//! (`RedbBackend::last_put_node_durability_for_label`) does not yet exist.
//! Tests will fail to compile until G2-A lands the branch-on-label logic.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.3, plan §3 G2-A).

#![allow(clippy::unwrap_used)]

use benten_caps::{CapabilityGrant, GrantScope};
use benten_core::Cid;
use benten_graph::{DurabilityMode, RedbBackend, WriteContext};

fn zero_cid() -> Cid {
    Cid::from_bytes(&[0u8; benten_core::CID_LEN]).expect("zero cid")
}

#[test]
fn capability_grant_writes_immediate() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Open backend with Async durability — the engine's configured default
    // would flow through for ordinary writes, but grant writes must override.
    let backend = RedbBackend::open_or_create_with_durability(
        dir.path().join("grant.redb"),
        DurabilityMode::Async,
    )
    .expect("open");

    let scope = GrantScope::parse("store:post:write").expect("scope");
    let grant = CapabilityGrant::new(zero_cid(), zero_cid(), scope);
    let node = grant.as_node();

    let ctx = WriteContext::privileged_for_engine_api();
    backend
        .put_node_with_context(&node, &ctx)
        .expect("grant write");

    // Inspection API: every CapabilityGrant-labelled put must have been
    // committed with Immediate durability, regardless of the configured mode.
    let observed = backend
        .last_put_node_durability_for_label("system:CapabilityGrant")
        .expect("durability inspection surface");
    assert_eq!(
        observed,
        DurabilityMode::Immediate,
        "CapabilityGrant Nodes must always commit with Immediate durability \
         (override the configured Async default)"
    );
}
