// G24-B wave-6b LANDED — T1 defense substantive pin.
//
// Asserts that a form-emitted SubgraphSpec whose injected primitives
// carry cap-scopes the schema-driven form-gen path would not have
// surfaced is rejected at the save boundary.
//
// Defense composition:
//
// 1. Form-gen consults the schema → emits only schema-derivable edges.
// 2. The save path re-derives the cap-scope envelope from the emitted
//    spec.
// 3. Mismatch (spec contains a primitive with a cap-scope outside the
//    manifest envelope, indicating it was crafted by something other
//    than the form path) → reject.
//
// Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.7 row 4
// (§3.X 6 + T1).
//
// ## Would-FAIL-if-no-op'd
//
// - If the save path skips the re-derivation step: the injection lands
//   at bridge.callAs unchecked — assertion fires.
// - If the validator returns the wrong error code: .rejects matcher
//   fails (expected E_CAP_DENIED with derivation-related details).

import { describe, test, expect } from "vitest";
import type { AdminUiV0Bridge } from "../src/index.js";
import {
  WORKFLOW_EDITOR_ERROR_CODES,
  WorkflowEditor,
  WorkflowEditorError,
  type ManifestEnvelopeShape,
  type SchemaSubgraphSpecShape,
  type SubgraphSpecWire,
  type WorkflowPrimitiveKind,
} from "../src/index.js";

const ADMIN_UI_PLUGIN_DID = "did:key:zAdminUiV0Plugin";

const RESTRICTED_MANIFEST: ManifestEnvelopeShape = {
  requires: [{ scope: "read:Note.*" }, { scope: "write:Note.*" }],
};

function probe(): {
  bridge: AdminUiV0Bridge;
  invocations: { handlerId: string; actor: string }[];
} {
  const invocations: { handlerId: string; actor: string }[] = [];
  const bridge: AdminUiV0Bridge = {
    async readNodeAs() {
      return null;
    },
    async onChangeAsWithCursor() {
      return { subscriptionId: "probe" };
    },
    async callAs(handlerId: string, _op, _input, actor) {
      invocations.push({ handlerId, actor });
      return { cid: "should-not-reach" };
    },
  };
  return { bridge, invocations };
}

describe("workflow_editor_subgraph_injection_rejected (T1 defense)", () => {
  test("form-emitted SubgraphSpec with injected edge fails cap-scope re-derivation", async () => {
    const { bridge, invocations } = probe();
    const editor = WorkflowEditor.mount({
      bridge,
      manifest: RESTRICTED_MANIFEST,
      principal: ADMIN_UI_PLUGIN_DID,
      schemaSpecs: new Map<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>(),
    });
    // Base shape: legitimate form submission for a Note read.
    // Then INJECT an edge to a high-privilege primitive whose
    // cap-scope is outside the manifest envelope ("host-fn:fs:write").
    const injected: SubgraphSpecWire = {
      handlerId: "admin-ui-v0::workflow::injected",
      primitives: [
        {
          id: "r_body",
          kind: "Read",
          capScope: "read:Note.body",
        },
        // INJECTED — schema-driven form-gen would never have surfaced
        // this primitive because it's not in the Note schema. The
        // save-side re-derivation MUST catch it.
        {
          id: "host_fn_fs_write",
          kind: "Write",
          capScope: "host-fn:fs:write",
        },
      ],
      edges: [["r_body", "host_fn_fs_write"]],
    };

    let captured: unknown;
    try {
      await editor.saveSpec(injected);
    } catch (err) {
      captured = err;
    }
    expect(captured).toBeInstanceOf(WorkflowEditorError);
    const typed = captured as WorkflowEditorError;
    expect(typed.code).toBe(WORKFLOW_EDITOR_ERROR_CODES.capDenied);
    expect(typed.details).toContain("cap-scope derivation");
    // CRITICAL: bridge.callAs was NEVER invoked.
    expect(invocations).toHaveLength(0);
  });

  test("injection with a different primitive id outside envelope rejected", async () => {
    // Variation: even a single injected primitive without any
    // accompanying legitimate primitives is rejected. The check
    // doesn't depend on the spec having a "mostly legitimate" prefix.
    const { bridge, invocations } = probe();
    const editor = WorkflowEditor.mount({
      bridge,
      manifest: RESTRICTED_MANIFEST,
      principal: ADMIN_UI_PLUGIN_DID,
      schemaSpecs: new Map<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>(),
    });
    const lone: SubgraphSpecWire = {
      handlerId: "admin-ui-v0::workflow::lone-injection",
      primitives: [
        {
          id: "x_network",
          kind: "Call",
          capScope: "network:exfiltrate",
        },
      ],
      edges: [],
    };
    let captured: unknown;
    try {
      await editor.saveSpec(lone);
    } catch (err) {
      captured = err;
    }
    expect(captured).toBeInstanceOf(WorkflowEditorError);
    expect((captured as WorkflowEditorError).code).toBe(
      WORKFLOW_EDITOR_ERROR_CODES.capDenied,
    );
    expect(invocations).toHaveLength(0);
  });
});
