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
    // R5 G3-A note: `from_bytes` on an all-zero buffer fails CID-header
    // validation; the zero-digest CID is the intended fixture.
    Cid::from_blake3_digest([0u8; 32])
}

/// Pinned CID for the canonical empty-extensions AttributionFrame.
/// Captured by the R5 G3-A first run once `AttributionFrame::cid()` fired:
/// three zero-digest CIDs routed through `Node::cid()` with label
/// `"AttributionFrame"` produce this stable CIDv1/dag-cbor/blake3. If this
/// string changes, the Phase-2a shape changed non-additively — review R4b.
const FIXTURE_CID: &str = "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a";

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn attribution_schema_fixture_cid_matches_checked_in_const() {
    let frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        // Phase-2b G7-B (D20 additive): default sandbox_depth = 0 keeps the
        // Phase-2a fixture CID stable. `AttributionFrame::cid()` only
        // includes the `sandbox_depth` slot in the canonical Node when
        // the value is non-zero — see exec_state.rs `cid()` discipline.
        sandbox_depth: 0,
    };
    let cid = frame.cid().expect("attribution frame cid");
    let actual = cid.to_string();

    if FIXTURE_CID == "TBD" {
        todo!("capture fixture CID and paste into FIXTURE_CID: {actual}");
    }
    assert_eq!(
        actual, FIXTURE_CID,
        "AttributionFrame schema fixture CID must stay pinned (D20 additive: \
         sandbox_depth=0 must NOT alter the canonical bytes)"
    );
}
