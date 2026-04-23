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

// R3 fixture bug fix (G2-A named deviation D1.1): the original fixture called
// `Cid::from_bytes(&[0u8; CID_LEN])` which fails at construction because
// byte[0] must be `CID_V1` (0x01), not zero. Every prior G1-A / G3-A / G9-A
// group hit the same bug — the workspace-standard workaround is
// `Cid::from_blake3_digest([0u8; 32])`, which threads the zero BLAKE3 digest
// through Benten's CID header synthesis and produces a valid zero-ish CID.
fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
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
