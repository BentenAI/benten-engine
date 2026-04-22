//! Edge-case test: `NoAuthBackend::check_write` carries forward through the
//! Phase 2a `WriteAuthority` lift (ucca-9 / arch-r1-2).
//!
//! R2 landscape §2.4 row "`NoAuthBackend::check_write` unchanged by Phase 2a".
//!
//! Concern: G2-B introduces the `WriteAuthority` enum (`User`,
//! `EnginePrivileged`, `SyncReplica { .. }`) on `WriteContext`. NoAuth must
//! ignore the new field and permit every write unconditionally — that's its
//! whole purpose as the zero-cost default.
//!
//! Edge-case flavour: we exercise every authority variant (including the
//! Phase-3-reserved `SyncReplica`) and an empty context. If NoAuth starts
//! returning Err for any of them, the thinness contract is broken.
//!
//! R3 red-phase contract: R5 (G2-B) adds the `authority` field to
//! `WriteContext`. This test compiles; it fails because the field does not
//! exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapabilityPolicy, NoAuthBackend, WriteContext};
use benten_core::Cid;
use benten_graph::WriteAuthority;

#[test]
fn noauth_still_permits_everything_after_phase2a_changes() {
    let policy = NoAuthBackend::new();

    // User authority — the Phase-1 default path.
    let ctx = WriteContext {
        label: "Post".into(),
        authority: WriteAuthority::User,
        ..WriteContext::default()
    };
    assert!(
        policy.check_write(&ctx).is_ok(),
        "NoAuth must permit User authority"
    );

    // EnginePrivileged — version-chain append path.
    let ctx = WriteContext {
        label: "Version".into(),
        authority: WriteAuthority::EnginePrivileged,
        ..WriteContext::default()
    };
    assert!(
        policy.check_write(&ctx).is_ok(),
        "NoAuth must permit EnginePrivileged authority"
    );

    // SyncReplica — Phase-3 reserved but the enum variant must not trip the
    // policy today.
    let origin = Cid::from_blake3_digest([0x7e; 32]);
    let ctx = WriteContext {
        label: "Doc".into(),
        authority: WriteAuthority::SyncReplica {
            origin_peer: origin,
        },
        ..WriteContext::default()
    };
    assert!(
        policy.check_write(&ctx).is_ok(),
        "NoAuth must permit SyncReplica authority even though the path is Phase-3"
    );
}

#[test]
fn noauth_permits_empty_context_with_default_authority() {
    // A fully-default `WriteContext` (post-Phase-2a) has authority = User.
    // NoAuth permits; this pins that default authority does not accidentally
    // become EnginePrivileged or anything else downstream code relies on.
    let policy = NoAuthBackend::new();
    let ctx = WriteContext::default();
    assert_eq!(
        ctx.authority,
        WriteAuthority::User,
        "WriteContext::default().authority must be User (Phase-2a pin)"
    );
    assert!(policy.check_write(&ctx).is_ok());
}
