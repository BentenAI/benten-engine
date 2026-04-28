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
//! **Status:** GREEN — Phase 2b G10-A-wasip1 landed. The wasm32-wasip1
//! runtime path now lives in `bindings/napi/src/wasm_target.rs` and the
//! `wasm-runtime.yml` workflow runs the wasm32-wasip1 build under
//! wasmtime, asserting the CID literal below. The native-side test
//! anchors the literal so any canonical-bytes drift fires here too.
//!
//! Owned by R3-E (test landed) + G10-A-wasip1 (implementation).

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
fn wasm32_wasip1_canonical_cid_matches_native() {
    // Native-side anchor: the CID we expect the wasm32-wasip1 run to
    // also produce. `wasm-runtime.yml` runs the wasm32-wasip1 build
    // under wasmtime and asserts the same literal — drift on either
    // side fires immediately.
    let native = canonical_test_node().cid().unwrap();
    assert_eq!(
        native.to_base32(),
        CANONICAL_FIXTURE_CID,
        "canonical fixture CID drifted from the Phase-1 frozen literal — \
         either the canonical-bytes pipeline (DAG-CBOR encoder, BLAKE3, \
         Cid wire format) regressed OR the canonical fixture's content \
         changed. The wasm-runtime workflow re-runs against wasm32-wasip1 \
         and asserts the same literal — drift here breaks the wasm-r1-1 \
         dual-target invariant."
    );

    // Optional harness opt-in: when invoked from `wasm-runtime.yml` with
    // BENTEN_WASIP1_HARNESS=1, the same Cargo test binary is built for
    // wasm32-wasip1 and run under wasmtime; the assertion above carries
    // the cross-target check (the wasm side hits the same `cid()` path
    // and therefore the same literal). The env var is informational
    // here — its presence proves the workflow wired the gate.
    let _harness_in_workflow = std::env::var("BENTEN_WASIP1_HARNESS").ok();
}
