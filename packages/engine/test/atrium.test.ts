// R3-C RED-PHASE TS Vitest pin for the Atrium DSL surface (G16-D
// wave-6b; per r2-test-landscape §2.4 G16-D + plan §3 G16-D row +
// D-PHASE-3-15 + Ben's D1 decision 2026-05-04 — Pattern B-prime).
//
// ## Pin sources
//
// - r2-test-landscape §2.4 G16-D row `atrium.test.ts (TS DSL surface)`.
// - plan §3 G16-D row line "TS DSL — `engine.atrium({config}).join()`
//   factory shape per D-PHASE-3-15 + Ben's D1 ratification".
// - `D-PHASE-3-15` (subsystem method namespacing — RECONCILED at
//   R4-FP/R3-C with Ben's D1 decision: factory pattern,
//   handle-returning).
// - r1-napi-10 (namespacing surface).
// - r4-r1-napi-2 MAJOR — namespace-vs-session ambiguity resolved at
//   R4-FP/R3-C via Ben's D1: `engine.atrium({config}).join()`
//   factory.
//
// ## D1 (Ben's decision 2026-05-04): Pattern B-prime
//
// `engine.atrium` is a FACTORY function that takes a config object +
// returns an `Atrium` handle. Methods (join, leave, listPeers,
// trustPeer, revokePeer, onPeerJoin, onPeerLeave, subscribe,
// declareDeviceAttestation, listDeclaredDeviceAttestations,
// publishViewResult) live on the returned handle.
//
//   const family = engine.atrium({groupId: "family"});
//   await family.join();
//   family.listPeers();
//   await family.subscribe(...);
//   await family.leave();
//
// TS interface contract:
//   engine.atrium: AtriumFactory
//   AtriumFactory: (config: AtriumConfig) => Atrium
//   Atrium: { join, leave, listPeers, trustPeer, revokePeer,
//             onPeerJoin, onPeerLeave, subscribe,
//             declareDeviceAttestation, listDeclaredDeviceAttestations,
//             publishViewResult }
//
// Namespace methods may attach to the function-object later if needed
// (e.g. `engine.atrium.list()` to list known atriums) without breaking
// the factory call shape — function objects support both.
//
// ## RED-PHASE discipline
//
// Every test calls `it.skip(...)` until G16-D wave-6b lands the
// surface. Per pim-2-ts-canary §3.6b amendment 2026-05-05 (Ben's
// ratification: `throw new Error` shape over `expect.fail` for
// canonical TS-canary uniformity), body uses
// `throw new Error("RED-PHASE: ...")` so a forgotten un-skip
// surfaces as a clear failure rather than silently passing.
// Updated per r4-r2-napi-4 sweep 2026-05-05.

import { describe, it, expect } from "vitest";

describe("engine.atrium B-prime factory DSL (R3-C / R4-FP RED-PHASE)", () => {
  it.skip("RED-PHASE: G16-D wave-6b — engine.atrium({config}).join() factory + handle methods round-trip", async () => {
    // G16-D implementer wires this against the B-prime factory:
    //
    //   import { Engine } from "@benten/engine";
    //   const engine = await Engine.open(":memory:");
    //   const family = engine.atrium({
    //     atriumId: "family",
    //     invite: testInvite(),
    //   });
    //   await family.join();
    //   const peers = family.listPeers();
    //   expect(peers.length).toBeGreaterThan(0);
    //   await family.trustPeer(otherPeerDid);
    //   await family.revokePeer(otherPeerDid);
    //   const onJoinCalls: string[] = [];
    //   family.onPeerJoin((did) => { onJoinCalls.push(did); });
    //   const onLeaveCalls: string[] = [];
    //   family.onPeerLeave((did) => { onLeaveCalls.push(did); });
    //   await family.leave();
    //
    // OBSERVABLE consequence: the factory shape returns a per-call
    // Atrium handle whose methods carry per-session state. Defends
    // against the failure shape where ambiguous flat-namespace
    // (`engine.atrium.join`) and factory shapes coexist.
    throw new Error(
      "RED-PHASE: G16-D wave-6b wires atrium B-prime factory DSL round-trip + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G16-D wave-6b — D1 — engine.atrium is a factory function returning Atrium handles", () => {
    // D1 (Ben's decision 2026-05-04) architectural pin. The Engine
    // class MUST expose `engine.atrium` as a callable factory
    // returning an Atrium handle, NOT as a flat-namespace object with
    // top-level methods.
    //
    //   import { Engine } from "@benten/engine";
    //   const engine = await Engine.open(":memory:");
    //   // `engine.atrium` is callable (factory):
    //   expect(typeof engine.atrium).toBe("function");
    //   // Calling it with a config returns an Atrium handle:
    //   const a = engine.atrium({atriumId: "x"});
    //   expect(typeof a.join).toBe("function");
    //   expect(typeof a.leave).toBe("function");
    //   expect(typeof a.listPeers).toBe("function");
    //   // No flattened top-level engine.atrium* methods:
    //   const proto = Object.getPrototypeOf(engine);
    //   const methods = Object.getOwnPropertyNames(proto);
    //   const flattened = methods.filter(m =>
    //     m.startsWith("atrium") && m !== "atrium"
    //   );
    //   expect(flattened).toEqual([]);
    //   const instanceKeys = Object.keys(engine);
    //   const flatInstance = instanceKeys.filter(k =>
    //     k.startsWith("atrium") && k !== "atrium"
    //   );
    //   expect(flatInstance).toEqual([]);
    //
    // OBSERVABLE consequence: a future refactor that adds, e.g.,
    // engine.atriumJoin() top-level OR converts engine.atrium to a
    // namespace-only object fails this test.
    throw new Error(
      "RED-PHASE: G16-D wave-6b wires B-prime factory architectural assertion + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G16-D + G14-A2 wave-6b — atrium.declareDeviceAttestation TS round-trip per CLAUDE.md baked-in #17", async () => {
    // CLAUDE.md baked-in #17 + r1-napi-2 + r4-r1-napi-4 pin. Browser
    // tabs use `atrium.declareDeviceAttestation(...)` on a constructed
    // Atrium handle to declare their device-DID capability envelope.
    //
    // r4-r1-napi-4 raised the question "does declaration happen
    // before or after Atrium join?" — Ben's D1 ratification places
    // device-attestation declaration ON the Atrium handle (constructed
    // via factory; can be invoked before join() to seed handshake).
    //
    //   const engine = await Engine.open(":memory:");
    //   const family = engine.atrium({atriumId: "family"});
    //   // Declared BEFORE join() so handshake can present the envelope:
    //   await family.declareDeviceAttestation({
    //     deviceDid: "did:key:test-device",
    //     capabilities: [{ path: "/zone/notifications/*", ability: "read" }],
    //     freshnessWindow: 3600,
    //   });
    //   await family.join();
    //   const declared = await family.listDeclaredDeviceAttestations();
    //   expect(declared.find(a => a.deviceDid === "did:key:test-device")).toBeDefined();
    //
    // OBSERVABLE consequence: TS-side declaration on the constructed
    // handle round-trips into the engine's internal device-attestation
    // table; the declaration is observable both pre- and post-join.
    throw new Error(
      "RED-PHASE: G16-D + G14-A2 wave-6b wires declareDeviceAttestation TS round-trip + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G16-D wave-6b — atrium.subscribe round-trip on constructed handle", async () => {
    // B-prime composition pin. The subscribe surface lives on the
    // Atrium handle (not on engine top-level), receiving the
    // per-subscriber filter callback that composes with G14-D F6
    // delivery-time cap recheck.
    //
    //   const engine = await Engine.open(":memory:");
    //   const family = engine.atrium({atriumId: "family"});
    //   await family.join();
    //   const events: any[] = [];
    //   const sub = await family.subscribe("/zone/posts", (event) => {
    //     events.push(event);
    //   });
    //   // ... write happens elsewhere, sync drains, callback fires ...
    //   expect(events.length).toBeGreaterThan(0);
    //   await sub.unsubscribe();
    //   await family.leave();
    //
    // OBSERVABLE consequence: subscribe returns a handle whose
    // unsubscribe() teardown is observable; composes with the G14-D
    // per-subscriber cap-recheck pin.
    throw new Error(
      "RED-PHASE: G16-D wave-6b wires atrium.subscribe round-trip on constructed handle + drops .skip + un-comments assertions",
    );
  });
});
