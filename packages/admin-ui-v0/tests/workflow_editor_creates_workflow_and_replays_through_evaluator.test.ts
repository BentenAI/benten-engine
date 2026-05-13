// G24-B wave-6b RED-PHASE pin (LOAD-BEARING §3.6f substantive).
//
// Asserts: user creates a workflow via the admin UI workflow editor →
// admin UI persists the resulting subgraph to redb via Engine.call_as →
// reloading the workflow + replaying through the evaluator yields the
// same content-addressed CID.
//
// This is the end-to-end pin for G24-B exit criterion (per
// `r2-test-landscape.md` §2.7 row 1). Defends against the failure mode
// "workflow editor emits a subgraph that round-trips through the wire
// format but the replay produces a different CID" — which would break
// content-addressing of workflows + Phase-6 AI-workflow forking.
//
// ## RED-PHASE status
//
// `test.skip` until G24-B wave-6b lands the workflow editor UX +
// schema-driven form generation + Engine.call_as plumbing through the
// browser/Tauri renderer surface. TS-side analog of Rust `#[ignore]`
// per dispatch-conventions §3.6e.
//
// ## Compliance
//
// - §3.6b LOAD-BEARING substantive: end-to-end through production write
//   path; would-FAIL if persistence layer or replay diverged.
// - §3.6e RED-PHASE: `test.skip` rationale names the un-ignore wave.
// - §3.6f SHAPE-not-SUBSTANCE pre-flight: assertion targets CID
//   equality post-replay, not mere "workflow saved" callback.

import { describe, test, expect } from "vitest";
import { placeholder } from "../src/index.js";

describe("workflow_editor_creates_workflow_and_replays_through_evaluator (G24-B end-to-end)", () => {
  test.skip("user-created workflow round-trips through evaluator with stable CID (RED-PHASE: closes at R5 G24-B wave-6b)", async () => {
    // Production arm (G24-B wave-6b):
    //
    //   const editor = WorkflowEditor.mount({ engine, manifest });
    //   await editor.dragPrimitive("READ");
    //   await editor.dragPrimitive("TRANSFORM");
    //   await editor.dragPrimitive("WRITE");
    //   await editor.connectEdges([[0, 1], [1, 2]]);
    //   const result = await editor.save({ name: "my-workflow" });
    //   const cid1 = result.cid;
    //
    //   // Reload + replay through evaluator
    //   const reloaded = await engine.readNodeAs(userPrincipal, cid1);
    //   const replayCid = await engine.replayWorkflow(reloaded);
    //
    //   expect(replayCid).toEqual(cid1);
    //
    // Would-FAIL-if-no-op'd: any divergence in canonical-bytes encoding
    // between save-time and replay-time would surface here.
    expect(placeholder().stage).toBe("r3-red-phase");
    throw new Error("RED-PHASE: production surface lands at G24-B wave-6b");
  });
});
