// R3-C RED-PHASE TS Vitest pin for the namespaced atrium DSL surface
// (G16-D wave-6b; per r2-test-landscape §2.4 G16-D + plan §3 G16-D row +
// D-PHASE-3-15).
//
// ## Pin sources
//
// - r2-test-landscape §2.4 G16-D row `atrium.test.ts (TS DSL surface)`.
// - plan §3 G16-D row line "TS DSL — namespaced form
//   `engine.atrium.join(...)` / `engine.atrium.listPeers()` /
//   `engine.atrium.trustPeer(did)` / `engine.atrium.revokePeer(did)` /
//   `engine.atrium.onPeerJoin` / `engine.atrium.onPeerLeave` per
//   D-PHASE-3-15 lean".
// - `D-PHASE-3-15` (subsystem method namespacing matches existing
//   patterns; no top-level `engine.atrium*` methods).
// - r1-napi-10 (namespacing surface).
//
// ## RED-PHASE discipline
//
// Every test calls `it.skip(...)` until the underlying surface lands at
// G16-D wave-6b. The skipped-block rationale is the RED-phase pin source.

import { describe, it } from "vitest";

describe("engine.atrium namespaced DSL (R3-C RED-PHASE)", () => {
  it.skip("RED-PHASE: G16-D wave-6b — engine.atrium.join + listPeers + trustPeer + revokePeer round-trip", async () => {
    // G16-D implementer wires this against the namespaced TS DSL:
    //
    //   import { Engine } from "@benten/engine";
    //   const engine = await Engine.open(":memory:");
    //   const atrium = await engine.atrium.join({
    //     atriumId: "test-atrium",
    //     invite: testInvite(),
    //   });
    //   const peers = atrium.listPeers();
    //   expect(peers.length).toBeGreaterThan(0);
    //   await atrium.trustPeer(otherPeerDid);
    //   await atrium.revokePeer(otherPeerDid);
    //   const onJoinCalls: string[] = [];
    //   atrium.onPeerJoin((did) => { onJoinCalls.push(did); });
    //   const onLeaveCalls: string[] = [];
    //   atrium.onPeerLeave((did) => { onLeaveCalls.push(did); });
    //
    // OBSERVABLE consequence: the namespaced surface (engine.atrium.*)
    // exposes join/list/trust/revoke/onPeerJoin/onPeerLeave as a
    // single coherent subsystem. Defends against fragmented
    // top-level `engine.atrium*` method placement.
    throw new Error("G16-D fills atrium namespaced DSL round-trip");
  });

  it.skip("RED-PHASE: G16-D wave-6b — D-PHASE-3-15 + r1-napi-10 — no top-level engine.atrium* methods", () => {
    // D-PHASE-3-15 + r1-napi-10 architectural pin. The Engine class
    // MUST NOT expose top-level methods like engine.atriumJoin(),
    // engine.atriumListPeers(), etc. All atrium operations route
    // through the engine.atrium namespace.
    //
    // Concrete shape:
    //   import { Engine } from "@benten/engine";
    //   const engine = await Engine.open(":memory:");
    //   const proto = Object.getPrototypeOf(engine);
    //   const methods = Object.getOwnPropertyNames(proto);
    //   const atriumTopLevel = methods.filter(m =>
    //     m.startsWith("atrium") && m !== "atrium"
    //   );
    //   expect(atriumTopLevel).toEqual([]);
    //
    // OBSERVABLE consequence: a future refactor that adds, e.g.,
    // engine.atriumJoin() top-level fails this test.
    throw new Error("G16-D fills no-top-level-atrium* assertion");
  });

  it.skip("RED-PHASE: G16-D wave-6b — engine.atrium.declareDeviceAttestation TS round-trip per CLAUDE.md baked-in #17", async () => {
    // CLAUDE.md baked-in #17 + r1-napi-2 pin. Browser tabs use
    // engine.atrium.declareDeviceAttestation(...) to declare their
    // device-DID capability envelope to a full peer.
    //
    //   const engine = await Engine.open(":memory:");
    //   await engine.atrium.declareDeviceAttestation({
    //     deviceDid: "did:key:test-device",
    //     capabilities: [{ path: "/zone/notifications/*", ability: "read" }],
    //     freshnessWindow: 3600,
    //   });
    //   // Round-trip through napi → engine internal:
    //   const declared = await engine.atrium.listDeclaredDeviceAttestations();
    //   expect(declared.find(a => a.deviceDid === "did:key:test-device")).toBeDefined();
    //
    // OBSERVABLE consequence: TS-side declaration round-trips into
    // the engine's internal device-attestation table.
    throw new Error("G16-D fills declareDeviceAttestation TS round-trip");
  });
});
