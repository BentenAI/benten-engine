//! R3 unit tests for G3-A (§9.1): `AttributionFrame` struct shape — FROZEN.
//!
//! Locked 3-field shape per plan §9.1:
//! ```rust
//! pub struct AttributionFrame {
//!     pub actor_cid: Cid,
//!     pub handler_cid: Cid,
//!     pub capability_grant_cid: Cid,
//! }
//! ```
//!
//! Phase-2a ships a single-frame chain for Phase-1 compatibility; ucca-1's
//! optional `delegation_parent` is Phase-6 additive and MUST NOT appear in
//! the 2a shape.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_eval::AttributionFrame;

fn zero_cid() -> Cid {
    // R5 G3-A note: the test-fixture helper uses `from_blake3_digest` (valid
    // CIDv1 envelope over a zero digest) rather than `from_bytes` on an
    // all-zero buffer — the latter fails CID-header validation. This is the
    // zero-digest CID all G3-A shape-pin tests intended.
    Cid::from_blake3_digest([0u8; 32])
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn attribution_frame_shape_is_actor_handler_grant() {
    // Constructing with the three named fields must compile.
    let frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
    };
    assert_eq!(frame.actor_cid, zero_cid());
    assert_eq!(frame.handler_cid, zero_cid());
    assert_eq!(frame.capability_grant_cid, zero_cid());
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn attribution_frame_no_delegation_parent_in_2a() {
    // Phase-2a single-frame chain: the `delegation_parent` optional slot is
    // a Phase-6 additive extension. If the field is added prematurely this
    // block below fails to compile (the 3-field constructor becomes
    // incomplete). ucca-1 documented this as Phase-6 additive.
    let _frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        // intentionally NO delegation_parent field here.
    };
}
