//! Phase 2b R3-B — AttributionFrame non-regression carry (G7-B).
//!
//! Pin source: sec-pre-r1-13 — Phase-2a sec-r6r1-01 closure pinned the
//! AttributionFrame canonical-bytes shape via `invariant_14_fixture_cid`.
//! D20's addition of `sandbox_depth: u8` (default 0) is an EXTENSION
//! that MUST NOT change the Phase-2a fixture CID for frames where
//! sandbox_depth == 0.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-B pending — sec-pre-r1-13 non-regression carry"]
fn attribution_frame_extension_preserves_phase_2a_sec_r6r1_01_inv_14_wiring() {
    // sec-pre-r1-13 — D20's `sandbox_depth: u8` field added to
    // AttributionFrame MUST NOT break Phase-2a's invariant_14 wiring
    // (which pins the (actor, handler, capability_grant) tuple shape).
    //
    // Test:
    //   1. Construct AttributionFrame with sandbox_depth = 0 (default).
    //   2. Assert: frame.cid() == the Phase-2a-pinned fixture CID
    //      (re-use the constant from invariant_14_fixture_cid.rs).
    //   3. Construct AttributionFrame with sandbox_depth = 1.
    //   4. Assert: frame.cid() != the pinned CID (proves the field is
    //      load-bearing in canonical bytes when non-zero — extension
    //      semantics are correct).
    //   5. Round-trip a frame at sandbox_depth = 0 through DAG-CBOR
    //      decode/encode + Inv-14 attribution check; assert the
    //      Phase-2a wiring still passes.
    todo!("R5 G7-B — Inv-14 wiring preserved across D20 extension");
}
