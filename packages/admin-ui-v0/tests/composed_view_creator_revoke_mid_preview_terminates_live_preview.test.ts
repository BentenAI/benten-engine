// G24-C wave-6b RED-PHASE pin (substantive; T12 + ratification #7
// admin-UI-side companion).
//
// Asserts that when the user's cap to the underlying anchor pattern is
// revoked mid-live-preview, the composed-view creator's preview surface
// terminates the subscription + surfaces a typed-error notice to the
// user (NOT silently continues showing stale data, NOT crashes).
//
// Counterpart to the Rust-side pin
// `crates/benten-engine/tests/admin_ui_v0_subscribe_with_revoked_cap_terminates_session.rs`
// — that test pins the engine-side subscription-termination; THIS test
// pins the admin-ui-v0 client-side handling of the termination signal
// (graceful UX, no stale-data display).
//
// ## RED-PHASE status
//
// `test.skip` until G24-C wave-6b lands + the engine-side cap-revoke
// pathway (T12) ships at G22-D wave.
//
// ## Closes
//
// G24-C + T12 admin-UI-side handling (`r2-test-landscape.md` §2.8 +
// ratification #7 admin-UI consumer)

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

describe("composed_view_creator_revoke_mid_preview_terminates_live_preview (T12 admin-UI handling)", () => {
  test.skip("cap-revoke mid-live-preview surfaces typed-error; preview stops (RED-PHASE: closes at R5 G24-C wave-6b)", async () => {
    // Production arm (G24-C wave-6b):
    //
    //   const creator = ComposedViewCreator.mount({ engine });
    //   await creator.selectAnchorPattern("notes-by-tag");
    //   creator.beginLivePreview();
    //
    //   // Revoke the cap that underlies the anchor pattern.
    //   await engine.revokeCapability({ scope: "notes:work" });
    //
    //   await waitFor(() => {
    //     expect(creator.previewState()).toEqual({
    //       kind: "terminated",
    //       reason: "E_SYNC_REVOKED_DURING_SESSION",
    //     });
    //   });
    //
    //   // Defense: NO further deltas applied after termination
    //   const deltasAtTermination = creator.deltaCount();
    //   await engine.callAs(otherPrincipal, /* writeNote("post-revoke") */);
    //   await sleep(50);
    //   expect(creator.deltaCount()).toEqual(deltasAtTermination);
    //
    // Would-FAIL-if-no-op'd: if the client-side handler silently
    // ignored the termination signal, post-revoke writes would still
    // flow into deltaCount.
    expect(placeholder().stage).toBe("r3-red-phase");
    throw new Error("RED-PHASE: production surface lands at G24-C wave-6b");
  });
});
