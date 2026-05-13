// G24-C wave-6b RED-PHASE pin (substantive; G23-B + T12 consumer).
//
// Asserts the composed-view creator's live-preview surface receives
// updates via the SUBSCRIBE seam (specifically
// `on_change_as_with_cursor` per the T12 defense pathway), NOT via a
// parallel admin-ui-internal polling or push-channel. This pins admin
// UI v0 as a consumer of the engine's actor-aware change-stream surface
// per T12 + ratification #7.
//
// ## RED-PHASE status
//
// `test.skip` until G23-B materializer lands + G24-C wave-6b composed-
// view-creator lands.
//
// ## Closes
//
// G24-C + G23-B consumer + T12 (`r2-test-landscape.md` §2.8 row 2)

// RED-PHASE production-surface canary (closes at R5 G24-A / G24-C).
// When un-ignored, these production-surface imports MUST resolve BEFORE
// vitest + placeholder imports below so that an absent
// @benten/engine export surfaces as a module-load failure rather than
// a deep-in-test runtime undefined-reference. Guard ordering matters:
// production imports first, test infrastructure imports second.
//
// import { Engine } from "@benten/engine"; // production-surface canary
// import { readNodeAs } from "@benten/engine/policy"; // cap-scoped read

import { describe, test, expect } from "vitest";
import { placeholder } from "../src/index.js";

describe("composed_view_creator_live_preview_propagates_through_subscribe_seam (T12 + G23-B)", () => {
  test.skip("live-preview deltas arrive via on_change_as_with_cursor (RED-PHASE: closes at R5 G24-C wave-6b)", async () => {
    // Production arm (G24-C wave-6b):
    //
    //   const creator = ComposedViewCreator.mount({ engine });
    //   await creator.selectAnchorPattern("notes-by-tag");
    //   await creator.selectProjection(["title", "body"]);
    //   creator.beginLivePreview();
    //
    //   // Write a note in another tab; live-preview must update.
    //   await engine.callAs(userPrincipal, /* writeNote("hello") */);
    //
    //   await waitFor(() => {
    //     expect(creator.previewState()).toContainEqual(
    //       expect.objectContaining({ title: "hello" })
    //     );
    //   });
    //
    //   // Grep-assert (defense-in-depth, T12 defense surface):
    //   // ComposedViewCreator subscribes ONLY via on_change_as_with_cursor
    //   // (NOT via on_change / poll / direct ChangeStream).
    //   expect(creator.subscriptionMode()).toBe("on_change_as_with_cursor");
    //
    // Would-FAIL-if-no-op'd: a parallel polling path would not produce
    // the actor-aware filtering that on_change_as_with_cursor enforces;
    // the subscription-mode assertion would fail.
    expect(placeholder().stage).toBe("r3-red-phase");
    throw new Error("RED-PHASE: production surface lands at G24-C wave-6b");
  });
});
