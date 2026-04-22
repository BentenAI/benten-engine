//! R3 unit tests for phil-r1-1 (FROZEN): `AttributionFrame` schema fixture CID.
//!
//! A pinned empty-extensions `AttributionFrame` encodes to a CID that equals a
//! checked-in constant. Guarantees Phase-6 additions are provably additive:
//! if the Phase-2a shape changes, this CID changes and the test fires.
//!
//! TDD red-phase: on first run, the `todo!()` guard captures the actual CID
//! so the R3 writer can paste it into `FIXTURE_CID` and re-run.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.5 phil-r1-1).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_eval::AttributionFrame;

fn zero_cid() -> Cid {
    Cid::from_bytes(&[0u8; benten_core::CID_LEN]).expect("zero cid")
}

/// Pinned CID for the canonical empty-extensions AttributionFrame.
/// TBD — first run captures via the todo!() guard; write the string back in.
const FIXTURE_CID: &str = "TBD";

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn attribution_schema_fixture_cid_matches_checked_in_const() {
    let frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
    };
    let cid = frame.cid().expect("attribution frame cid");
    let actual = cid.to_string();

    if FIXTURE_CID == "TBD" {
        todo!("capture fixture CID and paste into FIXTURE_CID: {actual}");
    }
    assert_eq!(
        actual, FIXTURE_CID,
        "AttributionFrame schema fixture CID must stay pinned"
    );
}
