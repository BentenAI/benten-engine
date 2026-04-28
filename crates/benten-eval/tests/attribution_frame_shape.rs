//! R3 unit tests for G3-A (§9.1) + Phase-2b G7-B (D20-RESOLVED):
//! `AttributionFrame` struct shape pin.
//!
//! Phase-2a froze the 3-field shape `(actor_cid, handler_cid,
//! capability_grant_cid)`. Phase-2b G7-B (D20-RESOLVED) extends this with
//! a single additive field `sandbox_depth: u8` for Inv-4 nest-depth
//! tracking. The extension is provably additive per D20:
//!   - Default `sandbox_depth = 0` keeps the Phase-2a schema-fixture CID
//!     (`bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`)
//!     stable for non-SANDBOX flows. Pinned by
//!     `invariant_14_fixture_cid::attribution_schema_fixture_cid_matches_checked_in_const`.
//!   - Non-zero `sandbox_depth` produces a content-distinguishable CID
//!     (asserted by `invariant_4_overflow.rs::attribution_frame_sandbox_depth_field_present_default_zero`).
//!
//! ucca-1's optional `delegation_parent` slot remains Phase-6 additive
//! and MUST NOT appear in the Phase-2b shape; the negative shape pin
//! below catches a premature add.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.2) + G7-B extension.

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

/// SHAPE-PIN: validates the Phase-2b struct shape (Phase-2a 3-field +
/// G7-B `sandbox_depth: u8` D20 additive extension).
#[test]
fn attribution_frame_shape_is_actor_handler_grant_depth() {
    // Constructing with the four named fields must compile.
    let frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
    };
    assert_eq!(frame.actor_cid, zero_cid());
    assert_eq!(frame.handler_cid, zero_cid());
    assert_eq!(frame.capability_grant_cid, zero_cid());
    assert_eq!(frame.sandbox_depth, 0);
}

/// SHAPE-PIN: ucca-1's optional `delegation_parent` slot remains a
/// Phase-6 additive extension and MUST NOT appear in the Phase-2b shape.
/// If a future change adds it prematurely, this construction becomes
/// incomplete and the file stops compiling.
#[test]
fn attribution_frame_no_delegation_parent_in_2b() {
    let _frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
        // intentionally NO delegation_parent field here.
    };
}
