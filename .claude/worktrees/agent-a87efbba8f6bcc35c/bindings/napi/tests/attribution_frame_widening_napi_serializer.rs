//! §13.9 Instance 25 grep-witness sentinel: napi serializer for
//! AttributionFrame Phase-3 widening fields.
//!
//! Pin source: `docs/future/phase-3-backlog.md` §13.9 (BLOCKER —
//! producer/consumer drift Instance 25; Phase-3 sibling of the
//! Phase-2b Instance 18 `sandboxDepth` drift caught at R6 R3
//! r6-r3-pcds-1).
//!
//! ## SOURCE-CITE DIAGNOSTIC (not load-bearing per pim-2 §3.6b)
//!
//! This file is a SOURCE-CITE WITNESS PIN — the LOAD-BEARING
//! behavioral assertions live as inline `#[cfg(test)] mod tests` in
//! `bindings/napi/src/trace.rs::tests` (companion module). The
//! source-cite witness pattern mirrors
//! `bindings/napi/tests/describe_sandbox.rs`: integration tests in
//! `bindings/napi/tests/` cannot reach the private
//! `trace_step_to_json` walker without promoting the gate of
//! `mod trace` + `mod node` (both carry `napi-rs` dep imports that
//! fail to link under the rlib-only `in-process-test` build mode the
//! integration test crate uses). The inline tests get straightforward
//! access to the private walker without disturbing the production
//! cfg gates.
//!
//! ## What this defends
//!
//! The integration test below greps the production source text at
//! `bindings/napi/src/trace.rs` for the three Phase-3 widening fields:
//!   - `peerDidSet` (Rust: `peer_did_set: Option<BTreeSet<Did>>`)
//!   - `deviceDid` (Rust: `device_did: Option<Did>`)
//!   - `syncHopDepth` (Rust: `sync_hop_depth: u32`)
//!
//! A regression that drops any of the three emission sites from
//! `trace_step_to_json` would surface here as a hard test fail. The
//! actual behavioral assertion (would-FAIL-if-no-op'd at the
//! serializer-output level) lives in the companion inline tests.
//!
//! ## DISAGREE-WITH-EXPLANATION (HARD RULE clause-c)
//!
//! The §13.9 backlog brief + the pre-fix TS interface
//! (`packages/engine/src/types.ts::AttributionFrame.deviceCid`)
//! reference a `device_cid: Option<Cid>` slot as the third Phase-3
//! widening field. The Rust producer `AttributionFrame` carries
//! `sync_hop_depth: u32` — the `device_cid` slot lives on `Engine` /
//! `WriteContext` / `ReadContext` but is never woven into the
//! AttributionFrame producer. The phantom `deviceCid` TS field is
//! dropped in the same fix-pass + the real producer field is mirrored
//! as `syncHopDepth`. The §13.9 backlog entry is left intact for
//! retrospective traceability but the on-the-wire reality is the
//! three fields above.
//!
//! ## Pairs with
//!
//!   - `bindings/napi/src/trace.rs::tests` — companion inline
//!     load-bearing pin module.
//!   - `packages/engine/src/types.ts::AttributionFrame` — TS-side
//!     declaration mirror (`syncHopDepth?: number` post-fix).
//!   - `packages/engine/test/attribution_frame_widening.test.ts` —
//!     TS-side schema declaration tests.
//!   - `crates/benten-eval/src/exec_state.rs::AttributionFrame::cid()`
//!     — producer-side skip-on-default canonical encoding.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn trace_rs_source() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(manifest_dir).join("src/trace.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// §13.9 grep-witness for `peerDidSet` emission site.
#[test]
fn napi_trace_serializer_emits_peer_did_set_camel_case_key() {
    let src = trace_rs_source();
    assert!(
        src.contains("\"peerDidSet\""),
        "bindings/napi/src/trace.rs::trace_step_to_json MUST emit the \
         camelCase JSON key \"peerDidSet\" mirroring the Rust producer's \
         `AttributionFrame::peer_did_set` field (§13.9 Instance 25 \
         closure). Companion inline pin at trace.rs::tests asserts the \
         emission with would-FAIL-if-no-op'd discipline."
    );
}

/// §13.9 grep-witness for `deviceDid` emission site.
#[test]
fn napi_trace_serializer_emits_device_did_camel_case_key() {
    let src = trace_rs_source();
    assert!(
        src.contains("\"deviceDid\""),
        "bindings/napi/src/trace.rs::trace_step_to_json MUST emit the \
         camelCase JSON key \"deviceDid\" mirroring the Rust producer's \
         `AttributionFrame::device_did` field (§13.9 Instance 25 \
         closure). Companion inline pin at trace.rs::tests asserts the \
         emission with would-FAIL-if-no-op'd discipline."
    );
}

/// §13.9 grep-witness for `syncHopDepth` emission site.
#[test]
fn napi_trace_serializer_emits_sync_hop_depth_camel_case_key() {
    let src = trace_rs_source();
    assert!(
        src.contains("\"syncHopDepth\""),
        "bindings/napi/src/trace.rs::trace_step_to_json MUST emit the \
         camelCase JSON key \"syncHopDepth\" mirroring the Rust \
         producer's `AttributionFrame::sync_hop_depth` field (§13.9 \
         Instance 25 closure). The §13.9 brief + the pre-fix TS \
         interface referenced a phantom `deviceCid` slot that was \
         never on the Rust producer; the actual mirrored field is \
         `sync_hop_depth`. Companion inline pin at trace.rs::tests \
         asserts the emission with would-FAIL-if-no-op'd discipline."
    );
}

/// §13.9 negative-pin grep-witness: the phantom `deviceCid` field that
/// the pre-fix TS interface declared (but the Rust producer never
/// carried) MUST NOT have a napi serializer emission site. Regression
/// guard against a future change that re-introduces the phantom field.
#[test]
fn napi_trace_serializer_does_not_emit_phantom_device_cid_key() {
    let src = trace_rs_source();
    assert!(
        !src.contains("\"deviceCid\""),
        "bindings/napi/src/trace.rs::trace_step_to_json MUST NOT emit a \
         `deviceCid` camelCase JSON key — the `device_cid` slot is on \
         `Engine` / `WriteContext` / `ReadContext` but is NOT a field \
         of `AttributionFrame`. The pre-fix TS interface declared a \
         phantom `deviceCid?: string` slot; the §13.9 Instance 25 \
         closure dropped that phantom in favor of the real producer \
         field (`syncHopDepth`)."
    );
}
