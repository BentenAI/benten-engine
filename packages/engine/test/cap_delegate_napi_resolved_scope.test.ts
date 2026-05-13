// Phase-4-Foundation G24-D-FP-3 — TS-side end-to-end pin for the
// napi `delegateCapability` binding (the Node-side TS surface that
// `bindings/napi/src/lib.rs::Engine::delegate_capability` is
// expressed through, via `packages/engine/src/engine.ts`'s
// `Engine.delegateCapability(...)`).
//
// Production-arm + observable-consequence + would-FAIL-if-no-op'd
// per pim-2 §3.6b. The Rust-side regression guard at
// `bindings/napi/tests/cap_delegate_napi_resolved_scope_regression_guard.rs`
// drives the engine seam directly; this TS-side companion drives
// the JS class shape to ensure the napi binding is callable + the
// observable consequence holds end-to-end via the TS DSL.
//
// Pin sources:
//   - `docs/future/phase-4-backlog.md` §4.8 acceptance criteria
//     ("New TS-side test under `packages/engine/test/` exercising
//     the binding end-to-end").
//   - G24-D-FP-3 brief (TS-side test exercising the binding via napi).
//
// SKIP-on-no-native: graceful-degradation per the rest of the suite.

import { describe, it, expect } from "vitest";
import { Engine, PolicyKind, crud } from "@benten/engine";

async function openOrSkip(
  path: string,
  policy: PolicyKind,
): Promise<Engine | null> {
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

describe("G24-D-FP-3 — Engine.delegateCapability resolved-scope class-of-bug regression guard", () => {
  it("delegateCapability over napi persists the RESOLVED scope, admits the cross-plugin write, and is revocable-by-scope", async () => {
    const engine = await openOrSkip(":memory:", PolicyKind.GrantBacked);
    if (!engine) return;
    try {
      const sourcePluginDid =
        "did:key:z6MkSourcePluginTsSide1234567890abcdefghi";
      const pluginBDid = "did:key:z6MkPluginBAudienceTsSide1234567890abcdef";

      // Step 1 — mint source grant; admits writes at `store:post:write`.
      const sourceGrantCid = await engine.grantCapability({
        actor: sourcePluginDid,
        scope: "store:post:write",
      });
      expect(typeof sourceGrantCid).toBe("string");
      expect(sourceGrantCid.length).toBeGreaterThan(0);

      const handler = await engine.registerSubgraph(
        crud("post", { capability: "store:post:write" }),
      );

      // Step 2 — delegate via napi. The engine seam resolves the
      // source CID → scope text → persists new grant Node with the
      // resolved scope. The pre-FP-3 class-of-bug shape would have
      // persisted `sourceGrantCid` as the new grant's scope string,
      // which `GrantBackedPolicy::check_write` could never match
      // against the handler's derived `store:post:write` scope.
      const delegationCid = await engine.delegateCapability(
        sourceGrantCid,
        pluginBDid,
        [], // empty attenuation → inherit resolved source scope
      );
      expect(typeof delegationCid).toBe("string");
      expect(delegationCid.length).toBeGreaterThan(0);
      // The delegation CID MUST be distinct from the source CID
      // (a fresh `system:CapabilityGrant` Node was minted).
      expect(delegationCid).not.toBe(sourceGrantCid);

      // Step 3 — write under the delegated cap as plugin-B admits.
      // The `GrantBackedPolicy::check_write` walker resolves
      // `store:post:write` against the persisted delegation Node and
      // admits — only possible if the delegation Node carries the
      // resolved scope (NOT the source CID as a string).
      //
      // Would-FAIL-if-no-op'd: with a CID-keyed delegation grant
      // shape (pre-FP-3 class-of-bug), this `callAs` would route to
      // ON_DENIED → throw E_CAP_DENIED here.
      const delegatedWrite = await engine.callAs(
        handler.id,
        "post:create",
        { title: "ts-side-delegated-write", body: "post via delegation" },
        pluginBDid,
      );
      expect(typeof delegatedWrite.cid).toBe("string");

      // Step 4 — revoke the delegation by CID + verify the per-row
      // cap-recheck-at-delivery machinery (same `GrantReader`
      // walker G16-B-F's `apply_atrium_merge` per-row cap-recheck
      // consults) observes the revocation. The revocation walker
      // matches by scope STRING, so this can only fire if the
      // delegation was persisted with the resolved scope (a
      // CID-keyed delegation Node would be un-revocable-by-scope).
      await engine.revokeCapability(delegationCid, pluginBDid);

      // Post-revoke write as plugin-B MUST surface E_CAP_DENIED.
      let captured: unknown = null;
      try {
        await engine.callAs(
          handler.id,
          "post:create",
          { title: "ts-side-post-revoke", body: "should deny" },
          pluginBDid,
        );
      } catch (err) {
        captured = err;
      }
      expect(captured).not.toBeNull();
      const e = captured as Error;
      // The walker's observation of the revocation is the load-bearing
      // assertion — the message MUST surface the cap-denied error
      // (E_CAP_DENIED, "denied", or "revoked"). The pre-FP-3 shape
      // would have left the cap effectively un-revoked-by-scope, so
      // the call would have succeeded instead of throwing.
      expect(e.message).toMatch(/E_CAP_DENIED|denied|revoked/i);
    } finally {
      await engine.close();
    }
  });

  it("delegateCapability rejects private-namespace caps cross-plugin (CLAUDE.md #18 sovereignty contract)", async () => {
    const engine = await openOrSkip(":memory:", PolicyKind.GrantBacked);
    if (!engine) return;
    try {
      const ownerPluginDid =
        "did:key:z6MkPrivateOwnerTsSide1234567890abcdefghij";
      const otherPluginDid =
        "did:key:z6MkOtherAudienceTsSide12345678901234abcdef";
      const privateScope = `private:${ownerPluginDid}:notes`;

      const sourceGrantCid = await engine.grantCapability({
        actor: ownerPluginDid,
        scope: privateScope,
      });

      // Cross-plugin delegation of a `private:*` cap MUST fail
      // regardless of any manifest `shares` policy — the engine seam
      // hardcodes this denial per CLAUDE.md baked-in #18
      // private-namespace clause.
      let captured: unknown = null;
      try {
        await engine.delegateCapability(sourceGrantCid, otherPluginDid, []);
      } catch (err) {
        captured = err;
      }
      expect(captured).not.toBeNull();
      const e = captured as Error;
      expect(e.message).toMatch(
        /private[-_ ]?namespace|PluginPrivateNamespace|E_PLUGIN_PRIVATE_NAMESPACE/i,
      );
    } finally {
      await engine.close();
    }
  });
});
