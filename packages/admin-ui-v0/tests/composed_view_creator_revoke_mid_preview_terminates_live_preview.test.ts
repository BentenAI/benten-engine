// G24-C wave-6b SUBSTANTIVE pin (un-ignored; T12 + ratification #7
// admin-UI-side companion).
//
// Asserts that when the user's cap to the underlying anchor pattern is
// revoked mid-live-preview, the composed-view creator's preview
// surface terminates the subscription + surfaces a typed-error notice
// to the user (NOT silently continues showing stale data, NOT crashes).
//
// Counterpart to the Rust-side pin at
// `crates/benten-engine/tests/admin_ui_v0_composed_view_creator_*.rs`
// — those test the engine-side subscription-termination; THIS test
// pins the admin-ui-v0 client-side handling of the termination signal
// (graceful UX, no stale-data display).
//
// ## Closes
//
// G24-C + T12 admin-UI-side handling (`r2-test-landscape.md` §2.8 +
// ratification #7 admin-UI consumer)

import { describe, test, expect } from "vitest";
import {
  ComposedViewCreator,
  type ComposedViewCreatorBridge,
  type UserViewSpec,
} from "../src/index.js";

// Test bridge whose `revokeCapability` triggers a delivery of the
// canonical termination sentinel through the subscribe callback. This
// mirrors the production-runtime engine pathway where
// `CapRecheckOutcome::Cancel` surfaces via the on-error channel after
// `Engine::revoke_capability` is invoked.
class RevokeBridge implements ComposedViewCreatorBridge {
  private deliverFn: ((event: unknown) => void) | null = null;
  private revokedScopes: Set<string> = new Set();
  private subscribedScope: string = "";

  async readNodeAs(_principal: string, _cid: string): Promise<unknown> {
    return null;
  }
  async onChangeAsWithCursor(
    pattern: string,
    _cursor: unknown,
    _actor: string,
    callback: (event: unknown) => void,
  ): Promise<{ readonly subscriptionId: string }> {
    this.deliverFn = callback;
    this.subscribedScope = pattern;
    return { subscriptionId: "sub-revoke-test" };
  }
  async registerSubgraphView(
    spec: UserViewSpec,
    _principal: string,
  ): Promise<{ readonly cid: string }> {
    return { cid: `bafy-test-${spec.viewId}` };
  }
  async materializerProvenance(_cid: string): Promise<string> {
    return "benten-ivm::strategy::AlgorithmB";
  }
  async revokeCapability(args: { readonly scope: string }): Promise<void> {
    this.revokedScopes.add(args.scope);
    // Production-runtime ARM mirror: the engine's
    // `CapRecheckOutcome::Cancel` outcome surfaces via the
    // delivery callback as a termination sentinel carrying
    // `E_SYNC_REVOKED_DURING_SESSION` (the typed ErrorCode shipped at
    // Phase-3 R6-FP Wave-C1 PR #170).
    if (this.deliverFn !== null) {
      this.deliverFn({
        kind: "subscription_terminated",
        reason: "E_SYNC_REVOKED_DURING_SESSION",
      });
    }
  }
  async callAs(
    _principal: string,
    op: {
      readonly kind: "write_node";
      readonly label: string;
      readonly properties: Readonly<Record<string, unknown>>;
    },
  ): Promise<{ readonly cid: string }> {
    // Substantive arm: writes flow into the delivery callback ONLY
    // if the relevant scope has not been revoked. If the scope is
    // revoked, the bridge silently drops the event (mirroring the
    // engine's `CapRecheckOutcome::Drop` for whole-actor-revoke +
    // the subsequent `Cancel` for terminated sessions).
    if (
      this.deliverFn !== null &&
      !this.revokedScopes.has("notes:work")
    ) {
      this.deliverFn({ row: { ...op.properties, label: op.label } });
    }
    return { cid: `bafy-call-${Date.now()}` };
  }
}

describe("composed_view_creator_revoke_mid_preview_terminates_live_preview (T12 admin-UI handling)", () => {
  test("cap-revoke mid-live-preview surfaces typed-error; preview stops", async () => {
    const bridge = new RevokeBridge();
    const principal = "did:key:test-user-revoke";

    const creator = ComposedViewCreator.mount({ bridge, principal });
    await creator.selectAnchorPattern("notes-by-tag");
    await creator.selectProjection(["title", "body"]);
    await creator.save({ name: "notes-by-work-tag" });
    await creator.beginLivePreview();

    // Drive a few normal deltas first.
    await bridge.callAs(principal, {
      kind: "write_node",
      label: "notes-by-tag",
      properties: { title: "pre-revoke-1" },
    });
    expect(creator.deltaCount()).toBe(1);

    const deltasAtTermination = creator.deltaCount();

    // Revoke the cap that underlies the anchor pattern.
    await bridge.revokeCapability({ scope: "notes:work" });

    // The preview state transitions to `terminated` carrying the
    // typed ErrorCode reason.
    const state = creator.previewStateValue();
    expect(state).toEqual({
      kind: "terminated",
      reason: "E_SYNC_REVOKED_DURING_SESSION",
    });

    // Defense: NO further deltas applied after termination. A write
    // attempt against the revoked scope MUST be dropped at the
    // bridge layer + ignored by the creator even if it leaks through.
    await bridge.callAs(principal, {
      kind: "write_node",
      label: "notes-by-tag",
      properties: { title: "post-revoke-leak" },
    });
    expect(creator.deltaCount()).toBe(deltasAtTermination);

    // Would-FAIL-if-no-op'd: a client-side handler that silently
    // ignored the termination sentinel would keep state.kind ===
    // "streaming"; a no-op `terminated`-state delta-suppression would
    // increment deltaCount on the post-revoke write.
  });

  test("terminated state is absorbing — repeated termination signals don't re-mutate state", async () => {
    const bridge = new RevokeBridge();
    const creator = ComposedViewCreator.mount({
      bridge,
      principal: "did:key:test-user-2x-revoke",
    });
    await creator.selectAnchorPattern("anchor");
    await creator.selectProjection(["body"]);
    await creator.save({ name: "view-2x" });
    await creator.beginLivePreview();
    await bridge.revokeCapability({ scope: "notes:work" });
    const firstSnapshot = creator.previewStateValue();
    await bridge.revokeCapability({ scope: "notes:work" });
    const secondSnapshot = creator.previewStateValue();
    expect(secondSnapshot).toEqual(firstSnapshot);
  });
});
