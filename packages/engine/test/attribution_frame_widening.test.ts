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

  it("AttributionFrame TS interface declares deviceCid field as optional string", () => {
    const frame: AttributionFrame = {
      actorCid: "bafyactor",
      handlerCid: "bafyhandler",
      capabilityGrantCid: "bafygrant",
      sandboxDepth: 0,
      deviceCid: "bafydevice",
    };
    expect(typeof frame.deviceCid).toBe("string");
    expect(frame.deviceCid).toBe("bafydevice");
  });

  it.skip("RED-PHASE: G16-B wave-6b — runtime AttributionFrame from Loro merge carries peerDidSet end-to-end", async () => {
    // pcds-r4-r1-1 LOAD-BEARING end-to-end pin per pim-2 §3.6b.
    // Implementer wires this:
    //
    //   const { Engine } = await import("@benten/engine");
    //   const peerA = await Engine.open(":memory:");
    //   const peerB = await Engine.open(":memory:");
    //
    //   // Both peers join the same Atrium + write the same anchor
    //   // concurrently, triggering a Loro merge:
    //   const atriumA = await peerA.atrium.join({ atriumId: "test", invite });
    //   const atriumB = await peerB.atrium.join({ atriumId: "test", invite });
    //   await peerA.call(handler, "write", { x: 1 });
    //   await peerB.call(handler, "write", { y: 2 });
    //   // ... wait for sync ...
    //
    //   // Inspect the merged version's attribution chain:
    //   const trace = await peerA.trace(handler, "read", {});
    //   const attribution = trace.steps[0].attribution;
    //
    //   // OBSERVABLE consequence: peerDidSet is populated with both
    //   // contributing peer DIDs (G16-B widening end-to-end):
    //   expect(attribution?.peerDidSet).toBeDefined();
    //   expect(attribution?.peerDidSet?.length).toBeGreaterThanOrEqual(2);
    //   expect(attribution?.peerDidSet).toContain(peerA.deviceDid);
    //   expect(attribution?.peerDidSet).toContain(peerB.deviceDid);
    //
    // Defends against the failure shape where the Rust producer widens
    // peer_did_set but the napi serializer / TS schema drop it silently
    // (Phase-2b Instance 18 sandboxDepth shape — caught post-merge).
    throw new Error(
      "G16-B wave-6b wires Loro-merge AttributionFrame.peerDidSet end-to-end TS round-trip",
    );
  });

  it.skip("RED-PHASE: G14-D wave-5a — runtime AttributionFrame from cross-device sync replica carries deviceDid end-to-end", async () => {
    // pcds-r4-r1-1 LOAD-BEARING end-to-end pin per pim-2 §3.6b.
    // Implementer wires this:
    //
    //   // Two engines on the same identity but different device DIDs
    //   // (the device-mesh shape per CLAUDE.md baked-in #17):
    //   const engineDeviceA = await Engine.openWithDevice(":memory:", "did:key:devA");
    //   const engineDeviceB = await Engine.openWithDevice(":memory:", "did:key:devB");
    //
    //   await engineDeviceA.call(handler, "write", { from: "A" });
    //   // ... sync to device B ...
    //
    //   const trace = await engineDeviceB.trace(handler, "read", {});
    //   const attribution = trace.steps[0].attribution;
    //
    //   // OBSERVABLE consequence: deviceDid carries the originating
    //   // device's DID; cross-device write provenance is observable:
    //   expect(attribution?.deviceDid).toBe("did:key:devA");
    //
    // Defends against the failure shape where cross-device writes
    // appear authored by the local device (provenance drift — direct
    // mirror of the Phase-2b sandboxDepth post-merge instance).
    throw new Error(
      "G14-D wave-5a wires cross-device AttributionFrame.deviceDid end-to-end TS round-trip",
    );
  });
});
