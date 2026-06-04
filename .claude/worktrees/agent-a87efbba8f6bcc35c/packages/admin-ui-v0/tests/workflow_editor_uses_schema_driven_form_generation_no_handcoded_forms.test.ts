// G24-B wave-6b LANDED — G23-A consumer pin (substantive).
//
// Asserts the workflow editor's form-generation logic is driven by the
// G23-A schema-driven-rendering subgraph — NOT by handcoded form
// templates per primitive.
//
// Defense composition:
// - Amending the schema (adding a new field) MUST surface that field
//   in the regenerated form WITHOUT any source-code change to the
//   editor.
// - The editor's source file MUST NOT contain hand-coded
//   `<input name="..."` per-primitive form template strings (the
//   substantive grep-assert pin).
//
// Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.7 row 2.
//
// ## Would-FAIL-if-no-op'd
//
// - If form-gen used a hand-coded template: the amended-schema field
//   would not appear → the assertion fires.
// - If the editor's source contained a hand-coded `<input>` per
//   primitive: the grep-assert fires.

import { describe, test, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import {
  WORKFLOW_EDITOR_FORM_GEN_SOURCE_SENTINEL,
  WorkflowEditor,
  deriveFormFromSchema,
  type ManifestEnvelopeShape,
  type SchemaSubgraphSpecShape,
  type WorkflowPrimitiveKind,
} from "../src/index.js";
import type { AdminUiV0Bridge } from "../src/index.js";

function nullBridge(): AdminUiV0Bridge {
  return {
    async readNodeAs() {
      return null;
    },
    async onChangeAsWithCursor() {
      return { subscriptionId: "no-op" };
    },
    async callAs() {
      return { cid: "" };
    },
  };
}

const SCHEMA_V1: SchemaSubgraphSpecShape = {
  schemaName: "Note",
  primitives: [
    {
      id: "r_body",
      kind: "Read" as WorkflowPrimitiveKind,
      capScope: "read:Note.body",
      fieldPath: "Note.body",
    },
  ],
};

const SCHEMA_V2_ADDED_FIELD: SchemaSubgraphSpecShape = {
  schemaName: "Note",
  primitives: [
    {
      id: "r_body",
      kind: "Read" as WorkflowPrimitiveKind,
      capScope: "read:Note.body",
      fieldPath: "Note.body",
    },
    {
      id: "r_filter_label",
      kind: "Read" as WorkflowPrimitiveKind,
      capScope: "read:Note.filter_label",
      fieldPath: "Note.filter_label",
    },
  ],
};

describe("workflow_editor_uses_schema_driven_form_generation_no_handcoded_forms (G23-A consumer)", () => {
  test("amending the schema surfaces the new field automatically (no hand-coded template)", () => {
    const formV1 = deriveFormFromSchema(SCHEMA_V1);
    const formV2 = deriveFormFromSchema(SCHEMA_V2_ADDED_FIELD);

    expect(formV1.fields.map((f) => f.id)).toEqual(["r_body"]);
    expect(formV2.fields.map((f) => f.id)).toEqual([
      "r_body",
      "r_filter_label",
    ]);

    // The schema-amendment field carries the schema-derived cap-scope
    // — proves form-gen pulled from the spec, not a hand-coded
    // template.
    const newField = formV2.fields.find((f) => f.id === "r_filter_label");
    expect(newField?.capScope).toBe("read:Note.filter_label");
  });

  test("editor.openPrimitiveForm consumes the schema spec (not a template)", () => {
    const editor = WorkflowEditor.mount({
      bridge: nullBridge(),
      manifest: { requires: [{ scope: "read:Note.*" }] } as ManifestEnvelopeShape,
      principal: "did:key:zAdminUiV0",
      schemaSpecs: new Map<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>([
        ["Read", SCHEMA_V2_ADDED_FIELD],
      ]),
    });
    const form = editor.openPrimitiveForm("Read");
    expect(form.fields.length).toBe(SCHEMA_V2_ADDED_FIELD.primitives.length);
    expect(form.fields.some((f) => f.id === "r_filter_label")).toBe(true);
  });

  test("editor source carries form-gen sentinel + NO hand-coded <input> per-primitive templates", () => {
    // Grep-assert (substantive): the workflow-editor source file
    // carries the canonical form-gen sentinel AND does NOT contain
    // hand-coded `<input name=` per-primitive HTML template strings.
    const here = dirname(fileURLToPath(import.meta.url));
    const editorPath = resolve(here, "../src/workflow-editor/index.ts");
    const source = readFileSync(editorPath, "utf-8");

    expect(source).toContain(WORKFLOW_EDITOR_FORM_GEN_SOURCE_SENTINEL);

    // Allow `name=` in DOC-ONLY comment lines (the file's prose
    // mentions the failure-mode pattern). Filter to non-comment lines
    // for the substantive grep.
    const nonCommentLines = source
      .split("\n")
      .filter((line) => !line.trim().startsWith("//"))
      .filter((line) => !line.trim().startsWith("*"))
      .join("\n");
    expect(nonCommentLines).not.toMatch(/<input\s+name="/);
    expect(nonCommentLines).not.toMatch(/<input\s+type="/);
  });
});
