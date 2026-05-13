// G24-B wave-6b RED-PHASE pin (LOAD-BEARING substantive; T4 defense).
//
// Asserts that an attempt to mint or attach a capability beyond what
// the admin UI v0 manifest grants — e.g., crafting a SubgraphSpec with
// an embedded UCAN whose attenuation exceeds the admin UI's cap-scope
// envelope — is denied at the write boundary (T4 in
// `admin-ui-v0-threat-model.md`).
//
// This is one of the cornerstone defense pins per §3.X 6 in
// `r2-test-landscape.md` §2.7 row 3 — production write path with
// chain-validation in the loop; would-FAIL if the validation step is
// no-op'd.
//
// ## RED-PHASE status
//
// `test.skip` until G24-B wave-6b lands the workflow editor + the
// underlying `Engine::call_as` chain-validation at the synchronous
// write entry point (T4 row in `r2-test-landscape.md` §2.7 row 7).
//
// ## Closes
//
// §3.X 6 + T4 (`r2-test-landscape.md` §2.7 row 3)

// RED-PHASE production-surface canary (closes at R5 G24-A / G24-B).
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

describe("workflow_editor_cap_elevation_attempt_denied (T4 defense)", () => {
  test.skip("forged cap-elevation in workflow save is denied at write boundary (RED-PHASE: closes at R5 G24-B wave-6b)", async () => {
    // Production arm (G24-B wave-6b):
    //
    //   // Construct a "hostile" workflow whose SubgraphSpec embeds a
    //   // UCAN cap claim wider than the admin UI manifest grants.
    //   const hostileSpec = forgeWorkflowWithCapElevation({
    //     attemptedCap: "graph:write:everywhere",
    //     attestedBy: "synthetic-admin-ui-internal-did",
    //   });
    //
    //   // Attempt to save through the workflow editor's normal write
    //   // path — should call Engine.call_as which chain-validates.
    //   await expect(
    //     editor.saveSpec(hostileSpec)
    //   ).rejects.toMatchObject({
    //     code: "E_CAP_ELEVATION_DENIED",
    //   });
    //
    // Would-FAIL-if-no-op'd: removing the chain-validation step would
    // permit the save; the .rejects matcher would not match and the
    // assertion would fail.
    expect(placeholder().stage).toBe("r3-red-phase");
    throw new Error("RED-PHASE: production surface lands at G24-B wave-6b");
  });
});
