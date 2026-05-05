//! R3-C RED-PHASE end-to-end pin: two-peer atrium bidirectional sync
//! (G16-B + G16-D wave-6b; per r2-test-landscape §2.4 G16-B + plan §3
//! G16-B row + exit-criterion 1).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row
//!   `integration/atrium_two_peer_bidirectional_sync` (file
//!   `tests/integration/atrium_two_peer.rs`).
//! - plan §3 G16-B + G16-D rows.
//! - exit-criterion 1 LOAD-BEARING (atrium two-peer bidirectional
//!   sync end-to-end — Phase-3 exit criterion).
//!
//! ## What this is at R3-C landing (relocated R4-FP)
//!
//! Hosts the end-to-end Atrium two-peer test driver. At R3-C landing
//! time the test is `#[ignore]`'d; G16-B + G16-D implementers wire the
//! production end-to-end driver.
//!
//! Originally placed in `tests/phase_3_workspace/`; relocated to
//! `tests/integration/` at R4-FP/R3-C per R3-CPC-1 + R2 §2.4 G16-B row
//! (R4 large-council Round 1 cross-partition-consistency lens).
//!
//! ## Atrium DSL shape (B-prime per Ben's D1 decision 2026-05-04)
//!
//! The TS-DSL surface is `engine.atrium({config}).join()` (factory
//! pattern returning an `Atrium` handle on which methods live), NOT
//! `engine.atrium.join({config})` (flat-namespace). Rust-side test
//! drivers consume the analogous handle-returning shape via the
//! Atrium-handle return type. See `packages/engine/test/atrium.test.ts`
//! for the canonical TS shape.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B + G16-D wave-6b — exit-criterion 1 LOAD-BEARING"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B + G16-D wave-6b — exit-criterion 1 LOAD-BEARING — atrium two-peer bidirectional"]
fn atrium_two_peer_bidirectional_sync() {
    // exit-criterion 1 LOAD-BEARING pin (Phase-3 sync-correctness
    // exit criterion sentence 1 from FULL-ROADMAP.md). G16-B +
    // G16-D implementers wire this end-to-end driver:
    //
    //   1. Spin up two engines under different peer-DIDs.
    //   2. Both peers join the same Atrium via shared invite using the
    //      B-prime factory shape:
    //        let atrium_a = engine_a.atrium(AtriumConfig { atrium_id, invite })
    //                                .join().await.unwrap();
    //   3. Handshake establishes mutual-auth + per-peer cap-set; the
    //      returned `Atrium` handle carries per-session state.
    //   4. peer_a writes to /zone/posts; peer_b sees the write.
    //   5. peer_b writes to /zone/posts; peer_a sees the write.
    //   6. atrium_a.leave().await + atrium_b.leave().await close cleanly.
    //
    // OBSERVABLE consequence: writes from each peer are visible to
    // the other within the same Atrium membership; the trust
    // boundary correctly applies cap filtering at delivery; the
    // atrium handle survives roundtrip cleanly.
    unimplemented!("G16-B + G16-D wire two-peer bidirectional Atrium sync end-to-end");
}
