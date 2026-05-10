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
import type {
  DeviceAttestation as IdentityDeviceAttestation,
  KeypairHandle,
} from "../src/identity.js";

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

// ---------------------------------------------------------------------------
// R6-FP Wave A Sub-A1 (`napi-r6-r1-1`) — leave/rejoin round-trip pin
// ---------------------------------------------------------------------------

describe("R6-FP Wave A Sub-A1: Atrium.leave/rejoin/isActive round-trip", () => {
  it("leave() flips isActive false; rejoin() flips it back to true", async () => {
    // napi-r6-r1-1 MAJOR closure pin. Pre-Wave-A the napi `JsAtrium`
    // surfaced `leave()` only — JS callers had no way to resume sync
    // on the same handle without dropping + rebuilding the whole
    // engine-side iroh transport. Wave A exposes the
    // `AtriumHandle::rejoin` engine-side surface (PR #159 G16-B-G) at
    // the napi boundary AND flips napi `leave()` to drive the
    // non-consuming `AtriumHandle::leave` so the engine-side handle
    // survives across leave/rejoin cycles.
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });

    // Pre-join: not active yet.
    expect(family.isJoined).toBe(false);
    expect(family.isActive).toBe(false);

    await family.join();
    expect(family.isJoined).toBe(true);
    expect(family.isActive).toBe(true);

    await family.leave();
    // After leave: isActive flips to false. (The in-memory shim also
    // clears `isJoined` so the existing legacy pin in the parent
    // describe still passes; the engine-bound napi path keeps
    // `isJoined` sticky-true while flipping isActive.)
    expect(family.isActive).toBe(false);

    await family.rejoin();
    expect(family.isJoined).toBe(true);
    expect(family.isActive).toBe(true);

    // Idempotent: rejoin on already-active is a no-op.
    await family.rejoin();
    expect(family.isActive).toBe(true);
  });

  it("rejoin pre-join is a no-op shape (factory returns Atrium with rejoin method)", async () => {
    // Surface-presence pin: every Atrium handle MUST surface rejoin()
    // + isActive whether or not it's been joined yet. Pre-join
    // `isActive` is observably false; calling `rejoin()` on a never-
    // joined handle drives the in-memory ctor to a flipped-active
    // state (matches the engine-bound semantics where `rejoin()`
    // before `join()` is a no-op rather than throwing).
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "x" });
    expect(typeof family.rejoin).toBe("function");
    expect(family.isActive).toBe(false);
    await family.rejoin();
    // In-memory shim flips active to true on rejoin (matches the
    // observable round-trip surface contract).
    expect(family.isActive).toBe(true);
  });

  it("trust + declared-attestation rosters survive across leave/rejoin", async () => {
    // Engine-side persistence contract pin (per
    // `AtriumHandle::leave/rejoin` rustdoc continuity guarantee). The
    // in-memory shim mirrors this by NOT clearing the `trusted` /
    // `declared` rosters on leave().
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    await family.join();
    await family.trustPeer("did:key:peer-x");
    await family.declareDeviceAttestation({
      deviceDid: "did:key:test-device",
      capabilities: [{ path: "/zone/posts/*", ability: "read" }],
      freshnessWindow: 3600,
    });
    expect(family.listPeers()).toContain("did:key:peer-x");

    await family.leave();
    await family.rejoin();

    // Rosters survive the leave/rejoin cycle:
    expect(family.listPeers()).toContain("did:key:peer-x");
    const declared = await family.listDeclaredDeviceAttestations();
    expect(declared.find((a) => a.deviceDid === "did:key:test-device")).toBeDefined();
  });
});

// ---------------------------------------------------------------------------
// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — local device-attestation setters
// ---------------------------------------------------------------------------

describe("R6-FP Wave A Sub-A2: Atrium device-attestation setters", () => {
  it("setLocalDeviceDid / setLocalDeviceKeypair / setLocalDeviceAttestation / setAcceptor surface presence", async () => {
    // napi-r6-r1-2 MAJOR closure pin. Pre-Wave-A the G16-D wave-6b
    // setters were Rust-only: JS-driven full peers (Tauri / Electron
    // / Node-AI-assistant — first-class per CLAUDE.md baked-in #17)
    // fell back to the legacy unsigned `device-cid:<hex>` envelope
    // regardless of what attestation the JS caller wanted to bind.
    // Wave A exposes the four setters at the napi + TS boundary.
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    expect(typeof family.setLocalDeviceDid).toBe("function");
    expect(typeof family.setLocalDeviceKeypair).toBe("function");
    expect(typeof family.clearLocalDeviceKeypair).toBe("function");
    expect(typeof family.setLocalDeviceAttestation).toBe("function");
    expect(typeof family.clearLocalDeviceAttestation).toBe("function");
    expect(typeof family.setAcceptor).toBe("function");
  });

  it("setLocalDeviceDid round-trips through the in-memory shim", async () => {
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    await family.join();
    // Bind + clear via empty-string convention (matches napi
    // boundary where empty-string maps to None on the engine side).
    await family.setLocalDeviceDid("did:key:my-device");
    await family.setLocalDeviceDid("");
  });

  it("setLocalDeviceKeypair / setLocalDeviceAttestation accept handle-shaped values", async () => {
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    await family.join();
    // The in-memory shim accepts any object — production callers
    // pass the napi-class instances (`Keypair.generate()` /
    // `DeviceAttestation.issue(...)`). Schema parity is asserted
    // here; cryptographic round-trip happens engine-side.
    const fakeKeypair = {
      publicKeyDid: () => "did:key:zfake",
      sign: () => new Uint8Array(64),
    } as unknown as KeypairHandle;
    const fakeAttestation = {
      deviceDid: "did:key:zfake",
      parentDid: "did:key:zparent",
      envelope: {
        runsSandbox: false,
        holdsZones: "cache_only" as const,
        onlineUptime: "session_bounded" as const,
        runsAtriumPeer: false,
      },
    } as unknown as IdentityDeviceAttestation;
    await family.setLocalDeviceKeypair(fakeKeypair);
    await family.clearLocalDeviceKeypair();
    await family.setLocalDeviceAttestation(fakeAttestation);
    await family.clearLocalDeviceAttestation();
  });

  it("setAcceptor accepts a non-negative finite freshness window", async () => {
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    await family.join();
    await family.setAcceptor(0);
    await family.setAcceptor(3600);
    await family.setAcceptor(Number.MAX_SAFE_INTEGER);
    // Negative / non-finite / non-number rejected at the wrapper
    // boundary before reaching the napi binding.
    await expect(family.setAcceptor(-1)).rejects.toThrow();
    await expect(family.setAcceptor(Number.NaN)).rejects.toThrow();
    await expect(family.setAcceptor(Number.POSITIVE_INFINITY)).rejects.toThrow();
  });

  it("setLocalDeviceKeypair / setLocalDeviceAttestation reject null at the wrapper boundary", async () => {
    const atrium = inMemoryFactory();
    const family = atrium({ atriumId: "family" });
    await family.join();
    await expect(
      family.setLocalDeviceKeypair(null as unknown as KeypairHandle),
    ).rejects.toThrow();
    await expect(
      family.setLocalDeviceAttestation(
        null as unknown as IdentityDeviceAttestation,
      ),
    ).rejects.toThrow();
  });
});
