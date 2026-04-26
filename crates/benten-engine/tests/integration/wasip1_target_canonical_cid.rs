//! Phase 2b R3 (R3-E) — wasm32-wasip1 canonical-fixture-CID match.
//!
//! TDD red-phase. Pin source: plan §3 G10-A-wasip1 must-pass tests +
//! wasm-r1-1 (cross-target canonical-CID gate). The Phase-1 fixture CID
//! `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` MUST
//! reproduce on wasm32-wasip1 with byte-for-byte agreement against the
//! native build.
//!
//! Cross-target determinism is the load-bearing exit-gate-3 invariant:
//! if Wasi32 + native disagree on the canonical fixture CID, the
//! "code-as-graph + content-addressed everything" promise breaks at the
//! distribution boundary. This test fires the same canonical-test-node
//! through the engine on the wasm32-wasip1 target and asserts the
//! resulting Cid matches the native fixture.
//!
//! Driver design: this Rust-side test invokes the wasm32-wasip1 build of
//! the engine via the same harness `wasm-runtime.yml` will use in CI;
//! locally it asserts only that the harness shape is intact and skips
//! actual wasm execution unless `BENTEN_WASIP1_HARNESS=1` is set
//! (mirrors Phase-2a determinism.yml gating).
//!
//! **Status:** RED-PHASE (Phase 2b G10-A-wasip1 pending). The wasm32-wasip1
//! runtime path lives in `bindings/napi/src/wasm_target.rs` and does not
//! yet exist.
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;

/// The Phase-1 frozen canonical fixture CID — every target MUST hit this.
const CANONICAL_FIXTURE_CID: &str = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";

/// `wasm32_wasip1_canonical_cid_matches_native` — plan §3 G10-A-wasip1
/// must-pass + R2 §2.3.
///
/// The same `canonical_test_node()` fed through the engine on wasm32-wasip1
/// must produce the Phase-1 frozen fixture CID. Any drift = the wasm
/// build broke the canonical-bytes contract.
#[test]
#[ignore = "Phase 2b G10-A-wasip1 pending — wasm-runtime harness + wasm_target.rs unimplemented"]
fn wasm32_wasip1_canonical_cid_matches_native() {
    // Native-side anchor: the Cid we expect the wasm-side run to also produce.
    let native = canonical_test_node().cid().unwrap();
    assert_eq!(
        native.to_base32(),
        CANONICAL_FIXTURE_CID,
        "native side must hit the frozen fixture CID before we can compare \
         wasm32-wasip1 against it"
    );

    // Wasm-side: invoke the wasm32-wasip1 build via the test harness
    // (G10-A-wasip1 owns the harness shim). Locally we exercise the
    // shim only when explicitly opted in via env var; CI sets this in
    // the wasm-runtime.yml job.
    let harness_enabled = std::env::var("BENTEN_WASIP1_HARNESS")
        .map(|v| v == "1")
        .unwrap_or(false);
    if !harness_enabled {
        // Pin the canonical CID + leave a hard pointer for the implementer.
        panic!(
            "BENTEN_WASIP1_HARNESS=1 not set — wasm32-wasip1 harness shim \
             needs G10-A-wasip1 implementation in bindings/napi/src/wasm_target.rs \
             + a CI runner that builds the wasm32-wasip1 target via wasmtime"
        );
    }

    // The harness shape (filled in by G10-A-wasip1):
    //   let wasm_cid = run_canonical_node_under_wasmtime_wasip1();
    //   assert_eq!(wasm_cid.to_base32(), CANONICAL_FIXTURE_CID);
    //   assert_eq!(wasm_cid, native);
    unreachable!("G10-A-wasip1 implementer wires the wasm32-wasip1 harness here");
}
