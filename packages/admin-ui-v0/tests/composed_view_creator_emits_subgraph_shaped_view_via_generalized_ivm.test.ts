// G24-C wave-6b RED-PHASE pin (substantive; D-4F-2 consumer pin).
//
// Asserts the composed-view creator emits a view whose representation
// is a subgraph shape consumed by the generalized IVM Algorithm B
// kernel (D-4F-2 + G23-0). Pins admin UI v0 as a consumer of D-4F-2's
// IVM-subgraph generalization — the failure mode being defended
// against is admin UI v0 introducing a parallel view-materialization
// path that diverges from the engine's IVM kernel.
//
// ## RED-PHASE status
//
// `test.skip` until G23-0 lands generalized IVM + G24-C wave-6b lands
// the composed-view creator UI.
//
// ## Closes
//
// G24-C + D-4F-2 consumer (`r2-test-landscape.md` §2.8 row 1)

import { describe, test, expect } from "vitest";
import { placeholder } from "../src/index.js";

describe("composed_view_creator_emits_subgraph_shaped_view_via_generalized_ivm (D-4F-2 consumer)", () => {
  test.skip("composed view representation is a subgraph; materializes via IVM kernel (RED-PHASE: closes at R5 G24-C wave-6b)", async () => {
    // Production arm (G24-C wave-6b):
    //
    //   const creator = ComposedViewCreator.mount({ engine });
    //   await creator.selectAnchorPattern("notes-by-tag");
    //   await creator.selectProjection(["title", "body"]);
    //   const result = await creator.save({ name: "notes-by-work-tag" });
    //
    //   // The persisted view is a subgraph whose nodes are the
    //   // generalized IVM Algorithm B's expected input shape.
    //   const viewSubgraph = await engine.readNodeAs(userPrincipal, result.cid);
    //   expect(viewSubgraph.kind).toBe("ComposedView");
    //   expect(viewSubgraph.ivmStrategy).toBe("AlgorithmB");
    //
    //   // Materialization path: changes to anchor nodes propagate
    //   // through IVM kernel — NOT through a parallel admin-ui-v0-only
    //   // view-recomputation path.
    //   await engine.callAs(userPrincipal, /* write to notes-by-tag */);
    //   expect(await engine.materializerProvenance(result.cid))
    //     .toBe("benten-ivm::strategy::AlgorithmB");
    //
    // Would-FAIL-if-no-op'd: a parallel materialization path would
    // surface in the provenance string.
    expect(placeholder().stage).toBe("r3-red-phase");
    throw new Error("RED-PHASE: production surface lands at G24-C wave-6b");
  });
});
