// R4-FP RED-PHASE pin for DeviceAttestation TS schema-level interface
// presence (pcds-r4-r1-2 instance-26 PRE-EMPTION).
//
// Pin source: .addl/phase-3/r4-r1-producer-consumer-deep-sweep.json
// finding `pcds-r4-r1-2` MAJOR — schema-parity-missing-field (mode 5)
// at DeviceAttestation TS interface. Same shape as Phase-2b §7.9
// Edge.cid phantom — runtime contract pinned at test layer but TS
// schema-level declaration was missing.
//
// What this pins:
//
//   The Rust producer at G14-A2 + G16-D ships a typed
//   `#[napi(object)] DeviceAttestationDeclaration` struct accepted by
//   `engine.atrium.declareDeviceAttestation(envelope)`. The TS-side
//   `interface DeviceAttestation` declaration lives at
//   packages/engine/src/types.ts (deviceDid + capabilities +
//   freshnessWindow fields).
//
//   This test pin asserts the TS schema declares the interface AND
//   that callers writing the typed shape get compile-time type-checking
//   rather than implicit `any`/`unknown`.
//
// Pairs with:
//   - bindings/napi/tests/device_attestation_napi.rs (R3-C napi
//     producer pin for round-trip)
//   - packages/engine/test/atrium.test.ts:71 (R3-C TS-DSL round-trip
//     pin via inline-literal envelope; this file pins the *schema-level*
//     interface declaration)
//
// RED-PHASE discipline:
//
//   The interface is declared in types.ts so this test compiles today;
//   the runtime round-trip assertion is skipped until G14-A2 + G16-D
//   land the typed napi struct + napi serializer.

import { describe, it, expect } from "vitest";
import type {
  DeviceAttestation,
  CapabilityClaim,
} from "@benten/engine";

describe("DeviceAttestation TS interface present (pcds-r4-r1-2 instance-26 PRE-EMPTION)", () => {
  it("DeviceAttestation TS interface declares deviceDid + capabilities + freshnessWindow fields", () => {
    // Schema-level pin: a TS-side type-test asserting the interface is
    // declared with the documented field shape. Compiles today; documents
    // the contract for callers + defends against the Edge.cid phantom
    // shape (interface declared but missing fields).
    const claim: CapabilityClaim = {
      path: "/zone/notifications/*",
      ability: "read",
    };
    const envelope: DeviceAttestation = {
      deviceDid: "did:key:test-device",
      capabilities: [claim],
      freshnessWindow: 3600,
    };
    expect(envelope.deviceDid).toBe("did:key:test-device");
    expect(envelope.capabilities).toHaveLength(1);
    expect(envelope.capabilities[0]?.path).toBe("/zone/notifications/*");
    expect(envelope.capabilities[0]?.ability).toBe("read");
    expect(envelope.freshnessWindow).toBe(3600);
  });

  it("CapabilityClaim TS interface declares path + ability fields", () => {
    const claim: CapabilityClaim = {
      path: "/posts/*",
      ability: "write",
    };
    expect(typeof claim.path).toBe("string");
    expect(typeof claim.ability).toBe("string");
  });

  it.skip("RED-PHASE: G14-A2 + G16-D — engine.atrium.declareDeviceAttestation accepts typed envelope and round-trips via napi typed struct", async () => {
    // pcds-r4-r1-2 LOAD-BEARING end-to-end pin per pim-2 §3.6b.
    // G14-A2 + G16-D implementer wires this:
    //
    //   const { Engine } = await import("@benten/engine");
    //   const engine = await Engine.open(":memory:");
    //
    //   const envelope: DeviceAttestation = {
    //     deviceDid: "did:key:test-device",
    //     capabilities: [{ path: "/zone/notifications/*", ability: "read" }],
    //     freshnessWindow: 3600,
    //   };
    //   await engine.atrium.declareDeviceAttestation(envelope);
    //
    //   // Round-trip through napi typed struct (NOT serde_json::Value):
    //   const declared = await engine.atrium.listDeclaredDeviceAttestations();
    //   const found = declared.find((a) => a.deviceDid === envelope.deviceDid);
    //   expect(found).toBeDefined();
    //   expect(found?.capabilities).toEqual(envelope.capabilities);
    //   expect(found?.freshnessWindow).toBe(envelope.freshnessWindow);
    //
    // OBSERVABLE consequence: TS-typed envelope → napi typed struct →
    // engine internal table → list-back round-trip preserves every
    // field with type-checking at every layer. Defends against the
    // Edge.cid phantom failure shape (silent field drops via untyped
    // `serde_json::Value` parameter).
    throw new Error(
      "G14-A2 + G16-D wires engine.atrium.declareDeviceAttestation typed-struct round-trip",
    );
  });
});
