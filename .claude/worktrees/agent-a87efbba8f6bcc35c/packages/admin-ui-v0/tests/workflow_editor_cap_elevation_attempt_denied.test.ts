// G24-B wave-6b LANDED — T4 defense substantive pin.
//
// Asserts that an attempt to mint or attach a capability beyond what
// the admin UI v0 manifest grants — e.g., crafting a SubgraphSpec with
// a cap-scope outside the manifest's `requires` envelope — is denied
// at the write boundary BEFORE the spec reaches `bridge.callAs`.
//
// Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.7 row 3
// (§3.X 6 + T4).
//
// ## Would-FAIL-if-no-op'd
//
// - If the editor save path skips the envelope validator: the hostile
//   spec lands at bridge.callAs unchecked — assertion fires (the
//   bridge invocation log records a write that should never have
//   happened).
// - If the validator returns the wrong error code: the .rejects matcher
//   fails (expected E_CAP_DENIED).

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

/**
 * Restricted manifest: the admin UI is only granted `read:Note.*` and
 * `write:Note.*`. The hostile spec attempts a write outside this
 * envelope.
 */
const RESTRICTED_MANIFEST: ManifestEnvelopeShape = {
  requires: [{ scope: "read:Note.*" }, { scope: "write:Note.*" }],
};

interface BridgeProbe {
  bridge: AdminUiV0Bridge;
  callAsInvocations: { handlerId: string; actor: string }[];
}

function probeBridge(): BridgeProbe {
  const callAsInvocations: { handlerId: string; actor: string }[] = [];
  const bridge: AdminUiV0Bridge = {
    async readNodeAs() {
      return null;
    },
    async onChangeAsWithCursor() {
      return { subscriptionId: "probe" };
    },
    async callAs(handlerId: string, _op, _input, actor) {
      callAsInvocations.push({ handlerId, actor });
      return { cid: "should-not-reach" };
    },
  };
  return { bridge, callAsInvocations };
}

describe("workflow_editor_cap_elevation_attempt_denied (T4 defense)", () => {
  test("draft cap-elevation attempt is denied BEFORE bridge.callAs", async () => {
    const { bridge, callAsInvocations } = probeBridge();
    const editor = WorkflowEditor.mount({
      bridge,
      manifest: RESTRICTED_MANIFEST,
      principal: ADMIN_UI_PLUGIN_DID,
      schemaSpecs: new Map<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>(),
    });
    // User drags a primitive with a cap outside the envelope.
    editor.dragPrimitive({
      id: "w_anywhere",
      kind: "Write",
      capScope: "graph:write:everywhere",
    });

    // The save path MUST throw a typed WorkflowEditorError with the
    // `E_CAP_DENIED` code.
    let captured: unknown;
    try {
      await editor.save({ name: "hostile" });
    } catch (err) {
      captured = err;
    }
    expect(captured).toBeInstanceOf(WorkflowEditorError);
    const typed = captured as WorkflowEditorError;
    expect(typed.code).toBe(WORKFLOW_EDITOR_ERROR_CODES.capDenied);
    // CRITICAL: bridge.callAs was NEVER invoked — the elevation was
    // caught at the editor boundary, not by the engine. This is the
    // substantive defense pin.
    expect(callAsInvocations).toHaveLength(0);
  });

  test("saveSpec with forged cap-elevation rejected at write boundary", async () => {
    const { bridge, callAsInvocations } = probeBridge();
    const editor = WorkflowEditor.mount({
      bridge,
      manifest: RESTRICTED_MANIFEST,
      principal: ADMIN_UI_PLUGIN_DID,
      schemaSpecs: new Map<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>(),
    });
    // Construct a "hostile" spec whose embedded primitive has a cap
    // wider than the manifest grants. The spec bypasses the draft
    // surface entirely — testing the saveSpec defense.
    const hostileSpec: SubgraphSpecWire = {
      handlerId: "admin-ui-v0::workflow::hostile",
      primitives: [
        {
          id: "w_anywhere",
          kind: "Write",
          capScope: "graph:write:everywhere",
        },
      ],
      edges: [],
    };
    let captured: unknown;
    try {
      await editor.saveSpec(hostileSpec);
    } catch (err) {
      captured = err;
    }
    expect(captured).toBeInstanceOf(WorkflowEditorError);
    expect((captured as WorkflowEditorError).code).toBe(
      WORKFLOW_EDITOR_ERROR_CODES.capDenied,
    );
    expect(callAsInvocations).toHaveLength(0);
  });

  test("legitimate within-envelope draft IS NOT over-rejected (regression-guard)", async () => {
    // Regression-guard: defense isn't over-strict. User-authored draft
    // within the manifest envelope MUST save successfully.
    const { bridge, callAsInvocations } = probeBridge();
    const editor = WorkflowEditor.mount({
      bridge,
      manifest: RESTRICTED_MANIFEST,
      principal: ADMIN_UI_PLUGIN_DID,
      schemaSpecs: new Map<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>(),
    });
    editor.dragPrimitive({
      id: "r_body",
      kind: "Read",
      capScope: "read:Note.body",
    });
    const outcome = await editor.save({ name: "legitimate" });
    expect(outcome.cid).toBe("should-not-reach");
    expect(callAsInvocations).toHaveLength(1);
    expect(callAsInvocations[0].actor).toBe(ADMIN_UI_PLUGIN_DID);
  });
});
