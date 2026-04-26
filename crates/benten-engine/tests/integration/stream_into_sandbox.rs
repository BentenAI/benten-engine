//! Phase 2b R3-B — STREAM-into-SANDBOX integration test (G7-A).
//!
//! Pin sources: wsa-18, arch-pre-r1-9. R2 §10 owner: R3-B (SANDBOX-side
//! composition is the point; ChunkSink is the consumed contract).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — STREAM-into-SANDBOX back-pressure"]
fn stream_into_sandbox_via_chunk_sink_back_pressure() {
    // wsa-18 + arch-pre-r1-9 — SANDBOX consumes a STREAM via the
    // ChunkSink trait shape (G6-A's `benten_eval::chunk_sink::ChunkSink`).
    // SANDBOX-side composition: a host-fn shim writes byte chunks
    // into the sink as wasm guest produces them.
    //
    // Back-pressure: when SANDBOX cannot drain chunks fast enough,
    // upstream STREAM producer's `.try_send` hits the bounded mpsc
    // (D4-RESOLVED PULL-based) and applies back-pressure (lossless
    // default, opt-in lossy).
    //
    // Test:
    //   1. Producer side: STREAM yields chunks at rate X.
    //   2. Consumer side: SANDBOX module reads chunks at rate Y < X.
    //   3. Assertion: producer observes back-pressure (channel-full
    //      → blocks under default lossless mode); no chunks dropped;
    //      total bytes consumed by SANDBOX matches total bytes
    //      produced by STREAM.
    //   4. SANDBOX output budget (D17 CountedSink) is independent of
    //      this composition — STREAM-into-SANDBOX is the INPUT path,
    //      not the output path.
    todo!("R5 G7-A — STREAM producer + SANDBOX consumer + back-pressure assertion");
}
