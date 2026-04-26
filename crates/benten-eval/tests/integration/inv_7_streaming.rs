//! Phase 2b R3-B — Inv-7 streaming-sandbox-output integration test (G7-A).
//!
//! Pin sources: D17 + wsa-18.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — Inv-7 streaming end-to-end"]
fn invariant_7_end_to_end_with_streaming_sandbox_output() {
    // D17 + wsa-18 — full integration: a SANDBOX module emits chunks
    // into a STREAM (host-fn `chunk_emit`). The streaming sink wraps
    // the SANDBOX output budget via D17 CountedSink. As cumulative
    // chunk bytes approach the budget, the inv-7 trap fires
    // BEFORE the budget is exceeded.
    //
    // Test:
    //   1. SubgraphSpec: STREAM consumer → SANDBOX producer (host-fn
    //      `chunk_emit` writes chunks).
    //   2. output_max_bytes = 1024.
    //   3. Module emits a sequence of 100-byte chunks.
    //   4. After ~10 chunks, the next chunk attempt traps
    //      E_INV_SANDBOX_OUTPUT through the CountedSink primary path.
    //   5. Downstream STREAM consumer observes the trap as a
    //      `e_stream_closed_by_peer` (or equivalent — the boundary
    //      surface decision is in R3-A's STREAM territory).
    todo!("R5 G7-A — STREAM + SANDBOX + chunk_emit host-fn + inv-7 trap");
}
