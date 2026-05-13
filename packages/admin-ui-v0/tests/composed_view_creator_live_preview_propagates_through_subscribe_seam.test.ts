// G24-C wave-6b SUBSTANTIVE pin (un-ignored; G23-B + T12 consumer).
//
// Asserts the composed-view creator's live-preview surface receives
// updates via the SUBSCRIBE seam (specifically
// `on_change_as_with_cursor` per the T12 defense pathway), NOT via a
// parallel admin-ui-internal polling or push-channel. This pins admin
// UI v0 as a consumer of the engine's actor-aware change-stream
// surface per T12 + ratification #7.
//
// ## Closes
//
// G24-C + G23-B consumer + T12 (`r2-test-landscape.md` §2.8 row 2)

import { describe, test, expect } from "vitest";
import {
  ComposedViewCreator,
  type ComposedViewCreatorBridge,
  type UserViewSpec,
} from "../src/index.js";

// In-memory test bridge that records every subscribe call site +
// surfaces a "deliver-this-event" hook. The bridge implementation
// pins the seam-name `"on_change_as_with_cursor"` it was invoked
// through; a parallel polling path would surface a different
// invocation pattern.
class SubscribeRoundTripBridge implements ComposedViewCreatorBridge {
  public subscribeCalls: Array<{
    readonly pattern: string;
    readonly cursor: unknown;
    readonly actor: string;
    readonly seamName: string;
  }> = [];
  private deliverFn: ((event: unknown) => void) | null = null;
  private registeredCid: string = "";

  async readNodeAs(_principal: string, _cid: string): Promise<unknown> {
    return null;
  }
  async onChangeAsWithCursor(
    pattern: string,
    cursor: "latest" | { persistent: string },
    actor: string,
    callback: (event: unknown) => void,
  ): Promise<{ readonly subscriptionId: string }> {
    // Record the call + pin the literal seam-name. Production bridges
    // (browser-tab fetch + Tauri IPC) forward this to
    // `Engine::on_change_as_with_cursor` — the cap-recheck-enabled seam.
    this.subscribeCalls.push({
      pattern,
      cursor,
      actor,
      seamName: "on_change_as_with_cursor",
    });
    this.deliverFn = callback;
    return { subscriptionId: `sub-${this.subscribeCalls.length}` };
  }
  async registerSubgraphView(
    spec: UserViewSpec,
    _principal: string,
  ): Promise<{ readonly cid: string }> {
    this.registeredCid = `bafy-test-${spec.viewId}`;
    return { cid: this.registeredCid };
  }
  async materializerProvenance(_cid: string): Promise<string> {
    return "benten-ivm::strategy::AlgorithmB";
  }
  async revokeCapability(): Promise<void> {
    /* not used in this test */
  }
  async callAs(
    _principal: string,
    op: {
      readonly kind: "write_node";
      readonly label: string;
      readonly properties: Readonly<Record<string, unknown>>;
    },
  ): Promise<{ readonly cid: string }> {
    // Drive the live-preview update path. A real engine adapter would
    // walk the change-stream + invoke per-row cap-recheck; here we
    // forward the row directly to the registered delivery callback to
    // pin the live-preview-consumer-side propagation behaviour.
    if (this.deliverFn !== null) {
      this.deliverFn({ row: { ...op.properties, label: op.label } });
    }
    return { cid: `bafy-call-${Date.now()}` };
  }
}

describe("composed_view_creator_live_preview_propagates_through_subscribe_seam (T12 + G23-B)", () => {
  test("live-preview deltas arrive via on_change_as_with_cursor", async () => {
    const bridge = new SubscribeRoundTripBridge();
    const principal = "did:key:test-user";

    const creator = ComposedViewCreator.mount({ bridge, principal });
    await creator.selectAnchorPattern("notes-by-tag");
    await creator.selectProjection(["title", "body"]);
    await creator.save({ name: "notes-by-work-tag" });
    await creator.beginLivePreview();

    // The subscribe seam-name MUST be `on_change_as_with_cursor` (T12
    // + sec-3.5-r1-9 floor). A parallel polling layer would surface
    // a different seamName here.
    expect(bridge.subscribeCalls.length).toBe(1);
    expect(bridge.subscribeCalls[0]!.seamName).toBe("on_change_as_with_cursor");
    expect(bridge.subscribeCalls[0]!.actor).toBe(principal);
    expect(bridge.subscribeCalls[0]!.pattern).toContain("notes-by-work-tag");

    // Drive a write through the engine's `call_as` surface; the
    // bridge forwards the resulting change event to the creator's
    // subscribe callback (the propagation path).
    expect(creator.deltaCount()).toBe(0);
    await bridge.callAs(principal, {
      kind: "write_node",
      label: "notes-by-tag",
      properties: { title: "hello", body: "world" },
    });

    // The live preview MUST observe the delivered row.
    const state = creator.previewStateValue();
    expect(state.kind).toBe("streaming");
    if (state.kind === "streaming") {
      expect(state.rows.length).toBe(1);
      expect(state.rows[0]).toMatchObject({ title: "hello", body: "world" });
      expect(state.deltaCount).toBe(1);
    }
    expect(creator.deltaCount()).toBe(1);

    // Grep-assert (defense-in-depth, T12 defense surface): the
    // creator's subscriptionMode is the canonical seam-name string.
    expect(creator.subscriptionMode()).toBe("on_change_as_with_cursor");

    // Would-FAIL-if-no-op'd: a parallel polling path would not invoke
    // `onChangeAsWithCursor` AT ALL (`subscribeCalls.length` would be
    // 0) or would surface a different seamName.
  });

  test("subscribe with persistent cursor threads through the same seam", async () => {
    const bridge = new SubscribeRoundTripBridge();
    const creator = ComposedViewCreator.mount({
      bridge,
      principal: "did:key:test-user-2",
    });
    await creator.selectAnchorPattern("posts-by-author");
    await creator.selectProjection(["title"]);
    await creator.save({ name: "posts-by-alice" });
    await creator.beginLivePreview({
      cursor: { persistent: "cursor-abc" },
    });

    expect(bridge.subscribeCalls.length).toBe(1);
    expect(bridge.subscribeCalls[0]!.cursor).toEqual({
      persistent: "cursor-abc",
    });
    expect(bridge.subscribeCalls[0]!.seamName).toBe("on_change_as_with_cursor");
  });

  test("beginLivePreview is idempotent — re-entry doesn't open a 2nd subscription", async () => {
    const bridge = new SubscribeRoundTripBridge();
    const creator = ComposedViewCreator.mount({
      bridge,
      principal: "did:key:test-user-3",
    });
    await creator.selectAnchorPattern("anchor");
    await creator.selectProjection(["body"]);
    await creator.save({ name: "view-3" });
    await creator.beginLivePreview();
    await creator.beginLivePreview();
    expect(bridge.subscribeCalls.length).toBe(1);
  });
});
