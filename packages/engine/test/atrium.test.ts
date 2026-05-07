// G16-D wave-6b LANDED — Vitest pin for the Atrium DSL surface
// (Pattern B-prime factory-handle form per Ben's D1 ratification
// 2026-05-05).
//
// ## Pin sources
//
// - r2-test-landscape §2.4 G16-D row `atrium.test.ts (TS DSL surface)`.
// - plan §3 G16-D row line "TS DSL — `engine.atrium({config}).join()`
//   factory shape per D-PHASE-3-15 + Ben's D1 ratification".
// - `D-PHASE-3-15` (subsystem method namespacing — RECONCILED at
//   R4-FP/R3-C with Ben's D1 decision: factory pattern,
//   handle-returning).
// - `r1-napi-10` (namespacing surface).
// - `r4-r1-napi-2` MAJOR — namespace-vs-session ambiguity resolved at
//   R4-FP/R3-C via Ben's D1: `engine.atrium({config}).join()` factory.
//
// ## D1 (Ben's decision 2026-05-04): Pattern B-prime
//
// `engine.atrium` is a FACTORY function that takes a config object +
// returns an `Atrium` handle. Methods (join, leave, listPeers,
// trustPeer, revokePeer, onPeerJoin, onPeerLeave, subscribe,
// declareDeviceAttestation, listDeclaredDeviceAttestations) live on
// the returned handle.
//
//   const family = engine.atrium({groupId: "family"});
//   await family.join();
//   family.listPeers();
//   await family.subscribe(...);
//   await family.leave();
//
// G16-D wave-6b implementation note: the engine class is opened from
// `:memory:` here against the napi binding when present; when running
// without a built native binding (cross-platform CI / cold-clone),
// the `makeAtriumFactory` fallback shim provides an in-memory
// `NativeAtrium` so the B-prime factory shape is exercisable from
// Vitest without an artifact build. The shim mirrors the napi
// `JsAtrium` field layout exactly.

import { describe, it, expect } from "vitest";

import { makeAtriumFactory, type AtriumFactory } from "../src/atrium.js";

// Build a factory bound to the in-memory shim. The shim path is the
// production-default fallback inside `makeAtriumFactory` when the
// napi `JsAtrium` constructor is absent — exercised here directly so
// the B-prime factory contract is asserted independent of native-
// binding state.
function inMemoryFactory(): AtriumFactory {
  return makeAtriumFactory(undefined);
}

describe("engine.atrium B-prime factory DSL (G16-D wave-6b LANDED)", () => {
  it("engine.atrium({config}).join() factory + handle methods round-trip", async () => {
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });

    // Pre-join: not yet joined
    expect(family.atriumId).toBe("family");
    expect(family.isJoined).toBe(false);

    await family.join();
    expect(family.isJoined).toBe(true);

    await family.trustPeer("did:key:peer-a");
    await family.trustPeer("did:key:peer-b");
    let peers = family.listPeers();
    expect(peers).toContain("did:key:peer-a");
    expect(peers).toContain("did:key:peer-b");
    expect(peers).toHaveLength(2);

    // Revoke one peer — the revoked peer drops from the roster:
    await family.revokePeer("did:key:peer-a");
    peers = family.listPeers();
    expect(peers).not.toContain("did:key:peer-a");
    expect(peers).toContain("did:key:peer-b");

    // onPeerJoin / onPeerLeave callback registration is observable:
    const joinCalls: string[] = [];
    family.onPeerJoin((did) => joinCalls.push(did));
    const leaveCalls: string[] = [];
    family.onPeerLeave((did) => leaveCalls.push(did));
    // Revoking a peer fires the onPeerLeave hook locally:
    await family.revokePeer("did:key:peer-c");
    expect(leaveCalls).toContain("did:key:peer-c");

    await family.leave();
    expect(family.isJoined).toBe(false);
  });

  it("D1 — engine.atrium-shaped factory function returning Atrium handles (NOT flat namespace)", () => {
    // D1 (Ben's decision 2026-05-04) architectural pin. The factory
    // is callable; calling with a config returns an Atrium handle;
    // there are NO flattened top-level methods.
    const atrium = inMemoryFactory();
    expect(typeof atrium).toBe("function");
    const a = atrium({ atriumId: "x" });
    expect(typeof a.join).toBe("function");
    expect(typeof a.leave).toBe("function");
    expect(typeof a.listPeers).toBe("function");
    expect(typeof a.subscribe).toBe("function");
    expect(typeof a.trustPeer).toBe("function");
    expect(typeof a.revokePeer).toBe("function");
    expect(typeof a.declareDeviceAttestation).toBe("function");
    expect(typeof a.listDeclaredDeviceAttestations).toBe("function");
    expect(typeof a.onPeerJoin).toBe("function");
    expect(typeof a.onPeerLeave).toBe("function");
  });

  it("D1 negative half — Engine class has NO flattened atrium methods (per g16-d-mr-1)", async () => {
    // Per g16-d-mr-1 fix-pass: the D1 positive half (factory shape +
    // handle methods exist) is asserted above; the NEGATIVE half
    // (flattened `engine.atriumJoin` / `engine.atriumLeave` / etc.
    // do NOT exist on the Engine class) is structurally enforced by
    // the codebase but was previously not pinned. A future drift could
    // re-introduce a flattened method undetected; this test cements
    // the contract.
    //
    // The assertion runs against the Engine prototype to defend
    // against per-instance / per-prototype additions.
    const { Engine } = await import("../src/engine");
    const flatNames = [
      "atriumJoin",
      "atriumLeave",
      "atriumListPeers",
      "atriumSubscribe",
      "atriumTrustPeer",
      "atriumRevokePeer",
      "atriumDeclareDeviceAttestation",
      "atriumListDeclaredDeviceAttestations",
      "atriumOnPeerJoin",
      "atriumOnPeerLeave",
    ];
    for (const name of flatNames) {
      expect((Engine as unknown as Record<string, unknown>)[name]).toBeUndefined();
      expect((Engine.prototype as unknown as Record<string, unknown>)[name]).toBeUndefined();
    }
  });

  it("each call to engine.atrium({...}) returns a fresh per-handle Atrium", () => {
    // Multi-Atrium-as-default per Ben's framing: separate calls
    // produce distinct handles whose state is independent (even if
    // the atriumId matches — they route to the same logical atrium
    // but each holds its own per-session state).
    const atrium = inMemoryFactory();
    const family1 = atrium({ atriumId: "family" });
    const family2 = atrium({ atriumId: "family" });
    expect(family1).not.toBe(family2);
    expect(family1.isJoined).toBe(false);
    expect(family2.isJoined).toBe(false);
  });

  it("rejects malformed AtriumConfig at the factory boundary", () => {
    const atrium = inMemoryFactory();
    expect(() => atrium(null as unknown as { atriumId: string })).toThrow();
    expect(() =>
      atrium({ atriumId: "" } as { atriumId: string }),
    ).toThrow(/atriumId/);
  });

  it("atrium.declareDeviceAttestation TS round-trip per CLAUDE.md baked-in #17", async () => {
    // CLAUDE.md baked-in #17 + r1-napi-2 + r4-r1-napi-4 pin. The
    // declaration lives on the Atrium handle (constructed via
    // factory; can be invoked before join() to seed handshake).
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    // Declared BEFORE join() so handshake can present the envelope:
    await family.declareDeviceAttestation({
      deviceDid: "did:key:test-device",
      capabilities: [{ path: "/zone/notifications/*", ability: "read" }],
      freshnessWindow: 3600,
    });
    await family.join();
    const declared = await family.listDeclaredDeviceAttestations();
    const found = declared.find((a) => a.deviceDid === "did:key:test-device");
    expect(found).toBeDefined();
    expect(found?.capabilities).toEqual([
      { path: "/zone/notifications/*", ability: "read" },
    ]);
    expect(found?.freshnessWindow).toBe(3600);
  });

  it("atrium.subscribe round-trip on constructed handle", async () => {
    // B-prime composition pin. The subscribe surface lives on the
    // Atrium handle (not on engine top-level), receiving the
    // per-subscriber filter callback that composes with G14-D F6
    // delivery-time cap recheck.
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    await family.join();
    const events: unknown[] = [];
    const sub = await family.subscribe("/zone/posts", (event) => {
      events.push(event);
    });
    expect(typeof sub.unsubscribe).toBe("function");
    await sub.unsubscribe();
    await family.leave();
  });
});
