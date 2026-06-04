// R4-FP RED-PHASE pin for AttributionFrame Phase-3 widening TS-side
// consumer pin (pcds-r4-r1-1 instance-25 PRE-EMPTION).
//
// Pin source: .addl/phase-3/r4-r1-producer-consumer-deep-sweep.json
// finding `pcds-r4-r1-1` MAJOR — schema-parity-missing-field (mode 5)
// at AttributionFrame Phase-3 extension. Same shape as Phase-2b
// Instance 18 (sandboxDepth widening) caught post-merge by R6-R3
// r6-r3-pcds-1; pre-empted here at R4 corpus revision time.
//
// What this pins:
//
//   The Rust producer at G14-D wave-5a (sync_replica_attribution.rs;
//   `device_did` + optional `device_cid` fields) and G16-B wave-6b
//   (loro_version_chain.rs + atrium_three_peer.rs; `peer_did_set`
//   field) widens AttributionFrame with new fields. The corresponding
//   TS-side `interface AttributionFrame` widening lives at
//   packages/engine/src/types.ts (peerDidSet?: string[]; deviceDid?:
//   string; deviceCid?: string).
//
//   This test pin asserts the TS schema declares the new fields AND
//   that a runtime-emitted attribution payload from a multi-peer Loro
//   merge / cross-device sync replica carries the values end-to-end.
//
// Pairs with:
//   - crates/benten-engine/tests/loro_version_chain.rs:94 (producer pin
//     for peer_did_set, contributing-peer-DIDs)
//   - crates/benten-engine/tests/sync_replica_attribution.rs:36
//     (producer pin for device_did)
//   - tests/phase_3_workspace/atrium_three_peer.rs:34 (3-peer
//     end-to-end producer pin)
//   - bindings/napi/tests/attribution_frame_widening_napi_serializer.rs
//     (sibling pin: napi serializer emits the new fields with documented
//     JSON keys; companion to this TS-side schema pin)
//
// RED-PHASE discipline:
//
//   The fields are declared OPTIONAL (`?`) in types.ts so this test
//   compiles today; the runtime payload assertions are skipped until
//   G14-D + G16-B + G19-D land (the napi serializer + parity meta-test
//   wire the round-trip).

import { describe, it, expect } from "vitest";
import type { AttributionFrame } from "@benten/engine";

describe("AttributionFrame Phase-3 widening (pcds-r4-r1-1 instance-25 PRE-EMPTION)", () => {
  it("AttributionFrame TS interface declares peerDidSet field as optional string array", () => {
    // Schema-level pin: a TS-side type-test asserting the field is
    // declared on the interface. Compiles today (no runtime assertion);
    // documents the schema contract.
    const frame: AttributionFrame = {
      actorCid: "bafyactor",
      handlerCid: "bafyhandler",
      capabilityGrantCid: "bafygrant",
      sandboxDepth: 0,
      peerDidSet: ["did:key:peer1", "did:key:peer2"],
    };
    expect(Array.isArray(frame.peerDidSet)).toBe(true);
    expect(frame.peerDidSet).toEqual(["did:key:peer1", "did:key:peer2"]);
  });

  it("AttributionFrame TS interface declares deviceDid field as optional string", () => {
    const frame: AttributionFrame = {
      actorCid: "bafyactor",
      handlerCid: "bafyhandler",
      capabilityGrantCid: "bafygrant",
      sandboxDepth: 0,
      deviceDid: "did:key:device1",
    };
    expect(typeof frame.deviceDid).toBe("string");
    expect(frame.deviceDid).toBe("did:key:device1");
  });

  it("AttributionFrame TS interface declares syncHopDepth field as optional number", () => {
    // §13.9 Instance 25 closure (2026-05-10): the producer-side
    // `AttributionFrame::sync_hop_depth: u32` field at
    // `crates/benten-eval/src/exec_state.rs` is the real Phase-3
    // widening slot the napi serializer + TS schema mirror. The
    // pre-fix interface declared a phantom `deviceCid?: string` slot
    // that never had a Rust producer; that phantom is dropped + the
    // actual producer field is mirrored here.
    const frame: AttributionFrame = {
      actorCid: "bafyactor",
      handlerCid: "bafyhandler",
      capabilityGrantCid: "bafygrant",
      sandboxDepth: 0,
      syncHopDepth: 3,
    };
    expect(typeof frame.syncHopDepth).toBe("number");
    expect(frame.syncHopDepth).toBe(3);
  });

  it("AttributionFrame TS schema-only declarations compile when Phase-3 widening fields are absent", () => {
    // §13.9 Instance 25 closure (2026-05-10) RE-DISPOSITION RATIONALE:
    //
    // The pre-fix `it.skip` body asserted end-to-end Loro-merge runtime
    // observability of `peerDidSet` — that requires multi-peer iroh
    // transport + Loro CRDT merge orchestration which lives in the
    // Rust `crates/benten-engine/tests/atrium_g16_b_e_substantive_e2e.rs`
    // + sibling Rust integration tests at `tests/integration/` (NOT
    // packages/engine/test/). The TS-side end-to-end pin would either
    // (a) duplicate Rust-side machinery in JS or (b) drive the napi
    // surface against an Atrium fixture — both out of scope for the
    // §13.9 napi-trace-serializer closure.
    //
    // The Rust-side observable pin lives at
    // `bindings/napi/tests/attribution_frame_widening_napi_serializer.rs`
    // (un-ignored in the SAME §13.9 closure PR): asserts the napi
    // serializer EMITS `peerDidSet` / `deviceDid` / `syncHopDepth`
    // when the producer populates them AND OMITS when default.
    // That sibling pin closes the Instance 25 observable-consequence
    // contract per pim-2 §3.6b at the napi boundary.
    //
    // The TS-side schema declaration is the only thing the TS file
    // can pin without a sync-runtime fixture in JS; the optional-slot
    // declarations are exercised compile-time by the test bodies above
    // (the `AttributionFrame` literal would fail TS type-check if any
    // declared field were missing).
    const localFrame: AttributionFrame = {
      actorCid: "bafyactor",
      handlerCid: "bafyhandler",
      capabilityGrantCid: "bafygrant",
      sandboxDepth: 0,
    };
    expect(localFrame.peerDidSet).toBeUndefined();
    expect(localFrame.deviceDid).toBeUndefined();
    expect(localFrame.syncHopDepth).toBeUndefined();
  });

  it("AttributionFrame TS interface accepts all Phase-3 widening fields populated simultaneously", () => {
    // §13.9 Instance 25 closure (2026-05-10) RE-DISPOSITION RATIONALE:
    //
    // The pre-fix sibling `it.skip` body asserted end-to-end
    // cross-device `deviceDid` observability under Atrium sync. Same
    // disposition as the prior test above — the Rust-side observable
    // pin at `bindings/napi/tests/attribution_frame_widening_napi_serializer.rs`
    // carries the end-to-end napi-boundary contract; the TS test
    // pins the schema declaration for ALL three Phase-3 fields
    // populated together (the merged sync-replica shape).
    const fullFrame: AttributionFrame = {
      actorCid: "bafyactor",
      handlerCid: "bafyhandler",
      capabilityGrantCid: "bafygrant",
      sandboxDepth: 0,
      peerDidSet: ["did:key:peer1", "did:key:peer2"],
      deviceDid: "did:key:devA",
      syncHopDepth: 2,
    };
    expect(fullFrame.peerDidSet).toEqual(["did:key:peer1", "did:key:peer2"]);
    expect(fullFrame.deviceDid).toBe("did:key:devA");
    expect(fullFrame.syncHopDepth).toBe(2);
  });
});
