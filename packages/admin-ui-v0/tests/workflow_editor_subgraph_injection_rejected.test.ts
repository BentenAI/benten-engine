// G24-B wave-6b RED-PHASE pin (LOAD-BEARING substantive; T1 defense).
//
// Asserts that a workflow editor form submission whose emitted
// SubgraphSpec includes nodes WITHOUT a corresponding derived
// cap-scope (i.e., the form crafts a "Trojan" subgraph by injecting
// edges to high-privilege nodes that the schema would not have surfaced
// in the form) is rejected before persistence (T1 in
// `admin-ui-v0-threat-model.md`).
//
// Defense composition:
//
// 1. Form-gen consults schema → emits only schema-derivable edges.
// 2. The save path re-derives cap-scope from the emitted spec.
// 3. Mismatch (spec contains an edge whose derived cap is wider than
//    the form would have surfaced) → reject.
//
// ## RED-PHASE status
//
// `test.skip` until G24-B wave-6b. The cap-scope-derivation lift +
// re-derivation check are Track B cleanups (per CLAUDE.md
// Phase-4-Foundation status header).
//
// ## Closes
//
// §3.X 6 + T1 (`r2-test-landscape.md` §2.7 row 4)

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

describe("workflow_editor_subgraph_injection_rejected (T1 defense)", () => {
  test.skip("form-emitted SubgraphSpec with injected edge fails cap-scope re-derivation check (RED-PHASE: closes at R5 G24-B wave-6b)", async () => {
    // Production arm (G24-B wave-6b):
    //
    //   const injectedSpec = forgeSubgraphWithInjectedEdge({
    //     baseShape: legitimateFormSubmission(),
    //     injectedEdge: {
    //       from: "user-node",
    //       to: "host-fn:fs:write",  // not in admin UI manifest
    //     },
    //   });
    //
    //   await expect(editor.saveSpec(injectedSpec)).rejects.toMatchObject({
    //     code: "E_SUBGRAPH_INJECTION_REJECTED",
    //     details: expect.stringContaining("cap-scope derivation"),
    //   });
    //
    // Would-FAIL-if-no-op'd: skipping the re-derivation step would
    // permit the injection; the .rejects would not match.
    expect(placeholder().stage).toBe("r3-red-phase");
    throw new Error("RED-PHASE: production surface lands at G24-B wave-6b");
  });
});
