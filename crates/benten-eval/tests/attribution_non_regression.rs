//! Phase 2b R3-B — AttributionFrame non-regression carry (G7-B).
//!
//! Pin source: sec-pre-r1-13 — Phase-2a sec-r6r1-01 closure pinned the
//! AttributionFrame canonical-bytes shape via `invariant_14_fixture_cid`.
//! D20's addition of `sandbox_depth: u8` (default 0) is an EXTENSION
//! that MUST NOT change the Phase-2a fixture CID for frames where
//! sandbox_depth == 0.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_eval::AttributionFrame;

fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

#[test]
fn attribution_frame_extension_preserves_phase_2a_sec_r6r1_01_inv_14_wiring() {
    // **G20-A1 wave-8a body** (Phase 3): sec-pre-r1-13 — D20's
    // `sandbox_depth: u8` field added to AttributionFrame MUST NOT
    // break Phase-2a's invariant_14 wiring.
    //
    // The Phase-2a-pinned schema fixture CID for a default
    // (`sandbox_depth = 0`) frame:
    const PHASE_2A_FIXTURE: &str = "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a";

    // 1. Default frame canonicalises to the Phase-2a CID (the slot is
    //    omitted from canonical bytes when zero per
    //    exec_state.rs::AttributionFrame::cid).
    let frame_zero = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
        ..Default::default()
    };
    let cid_zero = frame_zero.cid().expect("default frame encodes");
    assert_eq!(
        cid_zero.to_base32(),
        PHASE_2A_FIXTURE,
        "Phase-2a Inv-14 wiring preserved: depth-0 frame CID matches \
         the Phase-2a-pinned schema fixture"
    );

    // 2. Non-zero depth produces a DIFFERENT CID (the slot becomes
    //    load-bearing in canonical bytes — extension semantics
    //    correct).
    let frame_one = AttributionFrame {
        sandbox_depth: 1,
        ..frame_zero.clone()
    };
    let cid_one = frame_one.cid().expect("depth-1 frame encodes");
    assert_ne!(
        cid_one, cid_zero,
        "non-zero sandbox_depth MUST produce distinct CID (Phase-2b \
         D20 extension load-bearing in canonical bytes)"
    );

    // 3. Round-trip via DAG-CBOR encode/decode preserves canonical
    //    bytes for default-zero frame (Phase-2a non-regression).
    let encoded =
        serde_ipld_dagcbor::to_vec(&frame_zero).expect("DAG-CBOR encode of default frame");
    let decoded: AttributionFrame =
        serde_ipld_dagcbor::from_slice(&encoded).expect("DAG-CBOR decode");
    assert_eq!(
        decoded.sandbox_depth, 0,
        "DAG-CBOR round-trip preserves default sandbox_depth = 0"
    );
    let cid_decoded = decoded.cid().expect("decoded frame encodes");
    assert_eq!(
        cid_decoded, cid_zero,
        "round-tripped depth-0 frame CID matches the Phase-2a fixture"
    );

    // 4. Companion regression: non-zero round-trip.
    let encoded_one =
        serde_ipld_dagcbor::to_vec(&frame_one).expect("DAG-CBOR encode of depth-1 frame");
    let decoded_one: AttributionFrame =
        serde_ipld_dagcbor::from_slice(&encoded_one).expect("DAG-CBOR decode");
    assert_eq!(
        decoded_one.sandbox_depth, 1,
        "DAG-CBOR round-trip preserves sandbox_depth = 1"
    );
    assert_eq!(
        decoded_one.cid().unwrap(),
        cid_one,
        "round-tripped depth-1 frame CID stable"
    );
}
