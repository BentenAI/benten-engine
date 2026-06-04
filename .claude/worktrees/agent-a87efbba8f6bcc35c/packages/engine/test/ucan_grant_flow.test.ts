// Phase-3 G21-T2 §B audit-6-1 closure — TS-side end-to-end pin for
// the napi `PolicyKind.Ucan` rewire from Phase-1 stub
// (`benten_caps::LegacyUcanStubBackend`, returns `E_CAP_NOT_IMPLEMENTED`)
// to the durable grant-backed policy (G14-B `UCANBackend` proof-chain
// validator + `GrantBackedPolicy` revocation-aware hook).
//
// Per pim-2 §3.6b end-to-end-pin requirement: this drives the
// production `engine.openWithPolicy(path, PolicyKind.Ucan)` +
// `grantCapability` + `callAs` flow + asserts observable behavioral
// consequences. A sentinel-presence test would not suffice — we
// assert the durable backend is genuinely consulted (succeed-with-grant
// + fail-without).
//
// Pin sources:
//   - phase-3-backlog §2.3 (a) + (b) + (g) — napi-UCAN-wireup
//     end-to-end test target.
//   - G21-T2 brief §B end-to-end pin requirement.
//   - audit-6-1 closure — `PolicyKind::Ucan` durable surface routing.
//
// SKIP-on-no-native: graceful-degradation per the rest of the suite.

import { describe, it, expect } from "vitest";
import { Engine, PolicyKind, crud } from "@benten/engine";

async function openOrSkip(path: string, policy: PolicyKind): Promise<Engine | null> {
  try {
    return await Engine.openWithPolicy(path, policy);
  } catch (err) {
    const e = err as Error;
    if (e.name === "BentenNativeNotLoaded" || /not loadable/.test(e.message)) {
      return null;
    }
    throw err;
  }
}

describe("G21-T2 §B audit-6-1 — PolicyKind.Ucan durable wireup", () => {
  it("widened parse_grant_json carries issuer + hlc to the durable backend", async () => {
    const engine = await openOrSkip(":memory:", PolicyKind.Ucan);
    if (!engine) return;
    try {
      const aliceDid = "did:key:z6MkrJVnaZkeFzdQyMZu1csdAuLAaKSzjGpJSjm9V1F4xzm";
      // Pre-G21-T2 honest-state: this call surfaced E_CAP_NOT_IMPLEMENTED.
      // Post-G21-T2 audit-6-1 closure: the call routes through the
      // durable grant-backed policy + parser threads issuer + hlc to the
      // engine's `grant_capability_with_proof` adapter.
      const grantCid = await engine.grantCapability({
        actor: aliceDid,
        scope: "store:post:write",
        issuer: aliceDid,
        hlc: 1_000_000,
      });
      expect(typeof grantCid).toBe("string");
      expect(grantCid.length).toBeGreaterThan(0);
    } finally {
      await engine.close();
    }
  });

  it("PolicyKind.Ucan permits a write with a valid grant (durable backend consulted)", async () => {
    const engine = await openOrSkip(":memory:", PolicyKind.Ucan);
    if (!engine) return;
    try {
      const aliceDid = "did:key:z6MkrJVnaZkeFzdQyMZu1csdAuLAaKSzjGpJSjm9V1F4xzm";

      // Grant alice the post:write scope. Phase-3 G21-T2 wired the
      // durable backend, so this grant lives in
      // `system:CapabilityGrant` and `GrantBackedPolicy::check_write`
      // consults it on the next `callAs`.
      await engine.grantCapability({
        actor: aliceDid,
        scope: "store:post:write",
      });

      const handler = await engine.registerSubgraph(
        crud("post", { capability: "store:post:write" }),
      );

      // Drive a write through the policy gate. Pre-G21-T2 this would
      // surface E_CAP_NOT_IMPLEMENTED at the stub. Post-G21-T2 the
      // durable backend says permit + the WRITE commits.
      const out = await engine.callAs(
        handler.id,
        "post:create",
        { title: "ucan-grant-flow-test", body: "write under PolicyKind.Ucan" },
        aliceDid,
      );
      expect(typeof out.cid).toBe("string");
    } finally {
      await engine.close();
    }
  });

  it("PolicyKind.Ucan denies a write WITHOUT a valid grant (E_CAP_DENIED)", async () => {
    const engine = await openOrSkip(":memory:", PolicyKind.Ucan);
    if (!engine) return;
    try {
      const malloryDid = "did:key:z6MkrMalloryNoGrantNoEntryToTheZone1234567890";

      // No grant for mallory. The durable backend MUST deny.
      const handler = await engine.registerSubgraph(
        crud("post", { capability: "store:post:write" }),
      );

      let captured: unknown = null;
      try {
        await engine.callAs(
          handler.id,
          "post:create",
          { title: "should-be-denied", body: "no grant" },
          malloryDid,
        );
      } catch (err) {
        captured = err;
      }
      expect(captured).not.toBeNull();
      const e = captured as Error;
      // Post-G21-T2 the failure mode is E_CAP_DENIED (durable backend
      // policy reject); pre-G21-T2 it was E_CAP_NOT_IMPLEMENTED (stub).
      // The flip is the GREEN-phase signal that the runtime end-to-end
      // half is real per pim-2 §3.6b.
      expect(e.message).toMatch(/E_CAP_DENIED|denied/i);
      // Negative pin: post-G21-T2 the stub error MUST NOT fire.
      expect(e.message).not.toMatch(/E_CAP_NOT_IMPLEMENTED/);
    } finally {
      await engine.close();
    }
  });

  it("PolicyKind.Ucan + revokeCapability denies a previously-permitted write", async () => {
    const engine = await openOrSkip(":memory:", PolicyKind.Ucan);
    if (!engine) return;
    try {
      const carolDid = "did:key:z6MkrCarolHasGrantThenLosesIt12345678901234567";

      const grantCid = await engine.grantCapability({
        actor: carolDid,
        scope: "store:post:write",
      });

      const handler = await engine.registerSubgraph(
        crud("post", { capability: "store:post:write" }),
      );

      // Pre-revoke: write succeeds.
      const ok = await engine.callAs(
        handler.id,
        "post:create",
        { title: "pre-revoke", body: "should succeed" },
        carolDid,
      );
      expect(typeof ok.cid).toBe("string");

      // Revoke. The durable backend writes a
      // `system:CapabilityRevocation` Node; subsequent dispatch
      // observes the revocation via the chain-walker.
      await engine.revokeCapability(grantCid, carolDid);

      // Post-revoke: write is denied.
      let captured: unknown = null;
      try {
        await engine.callAs(
          handler.id,
          "post:create",
          { title: "post-revoke", body: "should deny" },
          carolDid,
        );
      } catch (err) {
        captured = err;
      }
      expect(captured).not.toBeNull();
      const e = captured as Error;
      expect(e.message).toMatch(/E_CAP_DENIED|denied|revoked/i);
    } finally {
      await engine.close();
    }
  });
});
