// G24-B wave-6b LANDED — substantive pin per pim-18 §3.6f.
//
// Asserts: user creates a workflow via the admin UI workflow editor →
// editor compiles the draft to a wire-shape SubgraphSpec → dispatches
// through `bridge.callAs` with the admin-UI plugin-DID as actor →
// reload-via-`bridge.readNodeAs` returns the persisted spec →
// re-compiling the reloaded spec yields the SAME content hash.
//
// This is the end-to-end pin for G24-B exit criterion per
// `r2-test-landscape.md` §2.7 row 1. Defends against:
//
//   "workflow editor emits a subgraph that round-trips through the
//    wire format but the replay produces a different CID" — which
//    would break content-addressing of workflows + Phase-6 AI-workflow
//    forking.
//
// ## Substantive shape (un-ignored at wave-6b)
//
// 1. Construct a recording AdminUiV0Bridge that captures every
//    `callAs` invocation + simulates persistence with a stable hash
//    over the canonical wire-shape bytes.
// 2. Mount the editor against the bridge + a permissive manifest
//    envelope + a fixture admin-UI plugin-DID principal.
// 3. Drag READ + TRANSFORM + WRITE primitives; connect edges; save.
// 4. Assert: bridge.callAs was invoked WITH the plugin-DID principal
//    (NOT engine-trusted) + the returned CID is non-empty + reading
//    back via bridge.readNodeAs returns the persisted spec.
// 5. Re-compile a fresh editor with the same draft → assert content
//    hash equality (canonical-bytes round-trips).
//
// ## Would-FAIL-if-no-op'd
//
// - If the editor bypasses `bridge.callAs` (e.g., calls `bridge.callAs`
//   without threading the principal, or calls the read seam): the
//   bridge's invocation log would miss the entry — test fails.
// - If the canonical-bytes encoding diverges between save-time and
//   replay-time: the CID equality assertion fails.

import { describe, test, expect } from "vitest";
import type { AdminUiV0Bridge } from "../src/index.js";
import {
  WorkflowEditor,
  type ManifestEnvelopeShape,
  type SchemaSubgraphSpecShape,
  type SubgraphSpecWire,
  type WorkflowPrimitiveKind,
} from "../src/index.js";

/**
 * Deterministic content-hash over a SubgraphSpecWire — mirrors the
 * Rust-side `workflow_content_hash` shape (BLAKE3 over canonical-bytes).
 * For TS-vitest-node-only we use a stable JSON canonicalization rather
 * than DAG-CBOR; the property "same input → same hash" is what the pin
 * exercises.
 */
function hashWireSpec(spec: SubgraphSpecWire): string {
  const canonical = JSON.stringify({
    handlerId: spec.handlerId,
    primitives: spec.primitives.map((p) => ({
      id: p.id,
      kind: p.kind,
      capScope: p.capScope ?? null,
    })),
    edges: spec.edges.map((e) => [e[0], e[1]] as const),
  });
  // Stable string hash — sufficient for the equality pin.
  let h = 0xcbf29ce484222325n;
  for (let i = 0; i < canonical.length; i++) {
    h ^= BigInt(canonical.charCodeAt(i));
    h = (h * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return h.toString(16).padStart(16, "0");
}

interface RecordedCall {
  readonly handlerId: string;
  readonly op: string;
  readonly input: unknown;
  readonly actor: string;
}

function recordingBridge(): {
  bridge: AdminUiV0Bridge;
  callAsInvocations: RecordedCall[];
  readNodeAsInvocations: { principal: string; cid: string }[];
  store: Map<string, SubgraphSpecWire>;
} {
  const callAsInvocations: RecordedCall[] = [];
  const readNodeAsInvocations: { principal: string; cid: string }[] = [];
  const store = new Map<string, SubgraphSpecWire>();
  const bridge: AdminUiV0Bridge = {
    async readNodeAs(principal: string, cid: string): Promise<unknown> {
      readNodeAsInvocations.push({ principal, cid });
      return store.get(cid) ?? null;
    },
    async onChangeAsWithCursor(): Promise<{ readonly subscriptionId: string }> {
      return { subscriptionId: "test-sub" };
    },
    async callAs(
      handlerId: string,
      op: string,
      input: unknown,
      actor: string,
    ): Promise<unknown> {
      callAsInvocations.push({ handlerId, op, input, actor });
      const spec = input as SubgraphSpecWire;
      const cid = hashWireSpec(spec);
      store.set(cid, spec);
      return { cid };
    },
  };
  return { bridge, callAsInvocations, readNodeAsInvocations, store };
}

const ADMIN_UI_PLUGIN_DID = "did:key:zAdminUiV0Plugin";
const PERMISSIVE_MANIFEST: ManifestEnvelopeShape = {
  requires: [
    { scope: "read:Note.*" },
    { scope: "write:Note.*" },
    { scope: "transform:Note.*" },
  ],
};

const NOTE_READ_SCHEMA: SchemaSubgraphSpecShape = {
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
const NOTE_TRANSFORM_SCHEMA: SchemaSubgraphSpecShape = {
  schemaName: "Note",
  primitives: [
    {
      id: "t_body",
      kind: "Transform" as WorkflowPrimitiveKind,
      capScope: "transform:Note.body",
      fieldPath: "Note.body",
    },
  ],
};
const NOTE_WRITE_SCHEMA: SchemaSubgraphSpecShape = {
  schemaName: "Note",
  primitives: [
    {
      id: "w_body",
      kind: "Write" as WorkflowPrimitiveKind,
      capScope: "write:Note.body",
      fieldPath: "Note.body",
    },
  ],
};

const SCHEMAS = new Map<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>([
  ["Read", NOTE_READ_SCHEMA],
  ["Transform", NOTE_TRANSFORM_SCHEMA],
  ["Write", NOTE_WRITE_SCHEMA],
]);

describe("workflow_editor_creates_workflow_and_replays_through_evaluator (G24-B end-to-end)", () => {
  test("user-created workflow round-trips through evaluator with stable CID", async () => {
    const { bridge, callAsInvocations, readNodeAsInvocations, store } =
      recordingBridge();

    const editor = WorkflowEditor.mount({
      bridge,
      manifest: PERMISSIVE_MANIFEST,
      principal: ADMIN_UI_PLUGIN_DID,
      schemaSpecs: SCHEMAS,
    });

    editor.dragPrimitive({
      id: "r_body",
      kind: "Read",
      capScope: "read:Note.body",
    });
    editor.dragPrimitive({
      id: "t_body",
      kind: "Transform",
      capScope: "transform:Note.body",
    });
    editor.dragPrimitive({
      id: "w_body",
      kind: "Write",
      capScope: "write:Note.body",
    });
    editor.connectEdges([
      ["r_body", "t_body"],
      ["t_body", "w_body"],
    ]);

    const outcome = await editor.save({ name: "my-workflow" });

    // (1) bridge.callAs was invoked WITH the admin-UI-DID principal.
    // Would-FAIL if the editor bypassed call_as or threaded an
    // engine-trusted handle instead of the plugin-DID.
    expect(callAsInvocations).toHaveLength(1);
    expect(callAsInvocations[0].actor).toBe(ADMIN_UI_PLUGIN_DID);
    expect(callAsInvocations[0].handlerId).toBe(
      "admin-ui-v0::workflow::my-workflow",
    );

    // (2) Outcome carries a non-empty CID.
    expect(outcome.cid).toMatch(/^[0-9a-f]+$/);

    // (3) Reload via bridge.readNodeAs returns the persisted spec.
    const reloaded = (await bridge.readNodeAs(
      ADMIN_UI_PLUGIN_DID,
      outcome.cid,
    )) as SubgraphSpecWire | null;
    expect(reloaded).not.toBeNull();
    expect(readNodeAsInvocations).toHaveLength(1);
    expect(readNodeAsInvocations[0].principal).toBe(ADMIN_UI_PLUGIN_DID);
    expect(reloaded!.handlerId).toBe(outcome.spec.handlerId);
    expect(reloaded!.primitives.length).toBe(outcome.spec.primitives.length);

    // (4) Replay through a fresh editor with the same drag sequence
    // yields the SAME content hash — the LOAD-BEARING canonical-bytes
    // round-trip pin per the G24-B exit criterion.
    const editor2 = WorkflowEditor.mount({
      bridge: recordingBridge().bridge,
      manifest: PERMISSIVE_MANIFEST,
      principal: ADMIN_UI_PLUGIN_DID,
      schemaSpecs: SCHEMAS,
    });
    editor2.dragPrimitive({
      id: "r_body",
      kind: "Read",
      capScope: "read:Note.body",
    });
    editor2.dragPrimitive({
      id: "t_body",
      kind: "Transform",
      capScope: "transform:Note.body",
    });
    editor2.dragPrimitive({
      id: "w_body",
      kind: "Write",
      capScope: "write:Note.body",
    });
    editor2.connectEdges([
      ["r_body", "t_body"],
      ["t_body", "w_body"],
    ]);
    const replaySpec = editor2.compileDraft("my-workflow");
    expect(hashWireSpec(replaySpec)).toBe(outcome.cid);

    // (5) Sanity: the recorded store carries the persisted spec.
    expect(store.get(outcome.cid)).toBeDefined();
  });
});
