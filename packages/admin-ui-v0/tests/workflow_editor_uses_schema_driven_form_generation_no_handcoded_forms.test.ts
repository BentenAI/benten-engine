// G24-B wave-6b RED-PHASE pin (substantive; G23-A consumer pin).
//
// Asserts the workflow editor's form-generation logic is driven by the
// G23-A schema-driven-rendering subgraph — NOT by handcoded form
// templates per primitive. Defense against the failure mode "form
// generation drifts from schema authority over time" + an architectural
// pin that admin UI v0 IS a consumer of D-4F-2's schema-driven
// rendering (per CLAUDE.md baked-in #15 v1-platform-gate framing).
//
// ## RED-PHASE status
//
// `test.skip` until G23-A wave-5 ships the schema-driven-rendering
// subgraph + G24-B wave-6b ships the workflow editor consumer.
//
// ## Closes
//
// G24-B + G23-A consumer (`r2-test-landscape.md` §2.7 row 2)

import { describe, test, expect } from "vitest";
import { placeholder } from "../src/index.js";

describe("workflow_editor_uses_schema_driven_form_generation_no_handcoded_forms (G23-A consumer)", () => {
  test.skip("workflow editor form-gen pulls from G23-A schema subgraph (RED-PHASE: closes at R5 G24-B wave-6b)", async () => {
    // Production arm (G24-B wave-6b):
    //
    //   // Modify the G23-A schema for "READ" primitive — add a new
    //   // typed-field Node "filter_label".
    //   await schemaSubgraph.amend({
    //     primitive: "READ",
    //     addField: { name: "filter_label", type: "Label" },
    //   });
    //
    //   // Re-mount the workflow editor; the new field MUST appear
    //   // automatically (form-gen consults schema at mount-time).
    //   const editor = WorkflowEditor.mount({ engine, manifest });
    //   const readForm = await editor.openPrimitiveForm("READ");
    //
    //   expect(readForm.fields).toContainEqual(
    //     expect.objectContaining({ name: "filter_label" })
    //   );
    //
    //   // Grep-assert: NO handcoded form templates in workflow editor
    //   // bundle source. The admin UI bundle must not contain a
    //   // hardcoded `<input name="filter_label">` string for any
    //   // primitive form (drift detector).
    //
    // Would-FAIL-if-no-op'd: handcoded forms would not pick up the new
    // schema field; the field-presence assertion would fail.
    expect(placeholder().stage).toBe("r3-red-phase");
    throw new Error("RED-PHASE: production surface lands at G24-B wave-6b");
  });
});
