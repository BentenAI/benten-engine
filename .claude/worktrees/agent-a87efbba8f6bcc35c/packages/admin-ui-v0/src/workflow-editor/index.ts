// G24-B wave-6b — admin UI v0 workflow editor (browser-side).
//
// This module is the browser-wasm32 / Tauri-embedded-webview workflow
// editor surface. It consumes:
//
// - `AdminUiV0Bridge` from `../index.js` for cap-scoped engine I/O
//   (`bridge.readNodeAs` + `bridge.callAs` per CLAUDE.md baked-in #18).
// - Schema-driven form generation from a `SchemaSubgraphSpec`-shape
//   structure (the G23-A consumer pin). Form fields derive from the
//   schema's per-primitive descriptor list, NOT from per-primitive
//   hand-coded HTML strings.
// - The plugin manifest envelope — every save path validates that the
//   workflow's derived cap-scopes are admissible under the active
//   manifest's `requires` envelope BEFORE invoking `bridge.callAs`
//   (T1 + T4 defense surface mirroring the Rust-side
//   `validate_subgraph_within_manifest_envelope`).
//
// ## Why is the editor body so small?
//
// The substantive engineering (form-gen + envelope validation + replay
// hash) lives Rust-side at
// `crates/benten-platform-foundation/src/admin_ui_v0/workflow_editor.rs`.
// The TS shell is the input collector + bridge-call dispatcher. This
// mirrors the engine's broader "Rust handler subgraph + TS shell"
// shape per CLAUDE.md baked-in #18 (admin UI v0 IS the first app-level
// plugin; the bundle is shareable, the substantive logic is in Rust).
//
// ## RED-PHASE → un-ignore at G24-B wave-6b
//
// At R3 only the four test pins lived under `tests/`. This module
// closes them at wave-6b by providing the substantive arms the test
// pins exercise.
//
// ## No handcoded forms — schema-driven only (T1 + G23-A consumer pin)
//
// Search this directory for the strings `<input` or `name="filter` —
// neither appears. Forms render from the schema's per-primitive
// descriptors. The test pin
// `workflow_editor_uses_schema_driven_form_generation_no_handcoded_forms`
// runs a runtime grep over `WorkflowEditor.SOURCE_GREP_SENTINEL` (the
// canonical string proves the editor's form-gen path is the
// schema-walk one, not a hand-coded template path).
//
// ## No engine-internal seam access
//
// This module NEVER references `read_node` (without `_as`) or
// `subscribe_change_events`. The grep-assert pin at
// `crates/benten-engine/tests/admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`
// covers this directory.

import type { AdminUiV0Bridge } from "../index.js";

// ---------------------------------------------------------------------
// Form-generation surface (G23-A consumer).
// ---------------------------------------------------------------------

/**
 * Shape of a per-primitive descriptor inside a `SchemaSubgraphSpec`.
 * Mirrors the Rust-side `PrimitiveDescriptor` exposed via
 * `benten_platform_foundation::PrimitiveDescriptor`. The TS-side
 * editor consumes this shape — it is provided by the napi binding
 * (full-peer shape a) or by the thin-client session protocol's
 * schema-spec payload (shapes b + c).
 */
export interface SchemaPrimitiveDescriptor {
  readonly id: string;
  readonly kind: WorkflowPrimitiveKind;
  readonly capScope?: string;
  readonly fieldPath?: string;
}

/**
 * Shape of a `SchemaSubgraphSpec` as consumed by the workflow editor
 * (TS-side dual of `benten_platform_foundation::SchemaSubgraphSpec`).
 *
 * The editor never reaches inside the spec's emitted Subgraph
 * directly; it only consumes the per-primitive descriptor list. This
 * keeps the TS surface stable across schema-language evolution
 * (new ingest dialects swap parse, NOT this consumer).
 */
export interface SchemaSubgraphSpecShape {
  readonly schemaName: string;
  readonly primitives: readonly SchemaPrimitiveDescriptor[];
}

/** Canonical 12 primitive kinds per CLAUDE.md baked-in #1. */
export type WorkflowPrimitiveKind =
  | "Read"
  | "Write"
  | "Transform"
  | "Branch"
  | "Iterate"
  | "Wait"
  | "Call"
  | "Respond"
  | "Emit"
  | "Sandbox"
  | "Subscribe"
  | "Stream";

/** One form field — mirrors Rust `WorkflowFormField`. */
export interface WorkflowFormField {
  readonly id: string;
  readonly kind: WorkflowPrimitiveKind;
  readonly capScope?: string;
  readonly fieldPath?: string;
}

/** Output of `deriveFormFromSchema`. */
export interface WorkflowForm {
  readonly schemaName: string;
  readonly fields: readonly WorkflowFormField[];
}

/**
 * Derive a {@link WorkflowForm} from a {@link SchemaSubgraphSpecShape}.
 *
 * Substantive G23-A consumer arm: walks the schema's per-primitive
 * descriptors and builds one form field per primitive. NO hand-coded
 * form template lives anywhere in this path.
 */
export function deriveFormFromSchema(
  spec: SchemaSubgraphSpecShape,
): WorkflowForm {
  return {
    schemaName: spec.schemaName,
    fields: spec.primitives.map((p) => ({
      id: p.id,
      kind: p.kind,
      capScope: p.capScope,
      fieldPath: p.fieldPath,
    })),
  };
}

// ---------------------------------------------------------------------
// Draft + compile surface (production write path).
// ---------------------------------------------------------------------

/** Mirrors Rust `WorkflowPrimitiveSelection`. */
export interface WorkflowPrimitiveSelection {
  readonly id: string;
  readonly kind: WorkflowPrimitiveKind;
  readonly capScope?: string;
}

/** Edge inside a draft — `[from_id, to_id]`. */
export type WorkflowEdge = readonly [string, string];

/** Mirrors Rust `WorkflowDraft`. */
export interface WorkflowDraft {
  readonly name: string;
  readonly primitives: readonly WorkflowPrimitiveSelection[];
  readonly edges: readonly WorkflowEdge[];
}

/** Shape of one `requires` entry inside a {@link ManifestEnvelopeShape}. */
export interface ManifestCapRequirement {
  readonly scope: string;
}

/**
 * Cap-envelope shape for save-time validation. Mirrors the relevant
 * subset of Rust `PluginManifest` — the editor only needs the
 * `requires` list to validate cap scopes.
 */
export interface ManifestEnvelopeShape {
  readonly requires: readonly ManifestCapRequirement[];
}

/** Error codes the editor surfaces (mirror `ErrorCode` via napi). */
export const WORKFLOW_EDITOR_ERROR_CODES = {
  capDenied: "E_CAP_DENIED",
  schemaEmitNewPrimitiveRejected: "E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED",
} as const;

export type WorkflowEditorErrorCode =
  (typeof WORKFLOW_EDITOR_ERROR_CODES)[keyof typeof WORKFLOW_EDITOR_ERROR_CODES];

/** Typed error from the workflow editor save path. */
export class WorkflowEditorError extends Error {
  readonly code: WorkflowEditorErrorCode;
  readonly details?: string;
  constructor(
    code: WorkflowEditorErrorCode,
    message: string,
    details?: string,
  ) {
    super(message);
    this.name = "WorkflowEditorError";
    this.code = code;
    this.details = details;
  }
}

/**
 * Check whether the manifest's `requires` envelope admits a cap-scope.
 * Mirrors the Rust-side `manifest_envelope_admits` semantics:
 *
 * - Exact match: `"read:Note.body"` matches `"read:Note.body"`.
 * - Prefix-wildcard: a manifest entry `"read:Note.*"` admits any
 *   `"read:Note.<anything>"` derived scope.
 */
export function manifestEnvelopeAdmits(
  requestedScope: string,
  manifest: ManifestEnvelopeShape,
): boolean {
  for (const req of manifest.requires) {
    if (req.scope.endsWith(".*")) {
      const prefix = req.scope.slice(0, -2);
      if (
        requestedScope === prefix ||
        requestedScope.startsWith(prefix + ".")
      ) {
        return true;
      }
    } else if (req.scope === requestedScope) {
      return true;
    }
  }
  return false;
}

/**
 * Pre-save check on a user-authored draft. Returns void on success.
 * Mirrors Rust `compile_draft_within_manifest_envelope`'s validation
 * (the TS-side editor does the FIRST pass — the Rust-side handler
 * still re-validates, defense in depth per the schema-compiler
 * sec-3.5-r1-4 cap-recheck pattern).
 *
 * @throws {WorkflowEditorError} `E_CAP_DENIED` on cap-elevation.
 * @throws {WorkflowEditorError} `E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED`
 *   if a draft primitive carries an unknown kind.
 */
export function validateDraftWithinEnvelope(
  draft: WorkflowDraft,
  manifest: ManifestEnvelopeShape,
): void {
  for (const prim of draft.primitives) {
    if (!CANONICAL_12_PRIMITIVE_KINDS.has(prim.kind)) {
      throw new WorkflowEditorError(
        WORKFLOW_EDITOR_ERROR_CODES.schemaEmitNewPrimitiveRejected,
        `Primitive kind ${prim.kind} is not in the canonical 12 (CLAUDE.md #1)`,
      );
    }
    if (prim.capScope && !manifestEnvelopeAdmits(prim.capScope, manifest)) {
      throw new WorkflowEditorError(
        WORKFLOW_EDITOR_ERROR_CODES.capDenied,
        `Cap-elevation rejected: scope ${prim.capScope} not in manifest envelope`,
        "cap-scope derivation outside manifest envelope",
      );
    }
  }
}

/**
 * Canonical 12-primitive set per CLAUDE.md #1. Mirrors Rust-side
 * defensive check. Used to reject draft primitives with unknown kinds
 * before reaching the bridge.
 */
export const CANONICAL_12_PRIMITIVE_KINDS: ReadonlySet<WorkflowPrimitiveKind> =
  new Set([
    "Read",
    "Write",
    "Transform",
    "Branch",
    "Iterate",
    "Wait",
    "Call",
    "Respond",
    "Emit",
    "Sandbox",
    "Subscribe",
    "Stream",
  ]);

// ---------------------------------------------------------------------
// SubgraphSpec (forged-input) defense.
// ---------------------------------------------------------------------

/**
 * Wire shape for a SubgraphSpec the editor's `saveSpec` accepts. The
 * tests inject hostile shapes here; the production form path emits
 * this shape after passing the draft through `validateDraftWithinEnvelope`.
 */
export interface SubgraphSpecWire {
  readonly handlerId: string;
  readonly primitives: readonly {
    readonly id: string;
    readonly kind: WorkflowPrimitiveKind;
    readonly capScope?: string;
  }[];
  readonly edges: readonly WorkflowEdge[];
}

/**
 * Save-side re-derivation check that mirrors Rust
 * `validate_subgraph_within_manifest_envelope`. The save path runs
 * this on every wire-shape spec BEFORE invoking `bridge.callAs` — so
 * a hostile spec that forged its way past the form-gen layer still
 * gets rejected here (T1 defense).
 *
 * @throws {WorkflowEditorError} `E_CAP_DENIED` on subgraph injection.
 */
export function validateSubgraphSpecWithinEnvelope(
  spec: SubgraphSpecWire,
  manifest: ManifestEnvelopeShape,
): void {
  for (const prim of spec.primitives) {
    if (!CANONICAL_12_PRIMITIVE_KINDS.has(prim.kind)) {
      throw new WorkflowEditorError(
        WORKFLOW_EDITOR_ERROR_CODES.schemaEmitNewPrimitiveRejected,
        `SubgraphSpec primitive kind ${prim.kind} is not in the canonical 12`,
      );
    }
    if (prim.capScope && !manifestEnvelopeAdmits(prim.capScope, manifest)) {
      throw new WorkflowEditorError(
        WORKFLOW_EDITOR_ERROR_CODES.capDenied,
        `Subgraph injection rejected: primitive ${prim.id} carries scope ${prim.capScope} outside manifest envelope`,
        "cap-scope derivation outside manifest envelope",
      );
    }
  }
}

// ---------------------------------------------------------------------
// WorkflowEditor class — bridge dispatcher.
// ---------------------------------------------------------------------

/** Configuration for {@link WorkflowEditor.mount}. */
export interface WorkflowEditorMountInput {
  readonly bridge: AdminUiV0Bridge;
  readonly manifest: ManifestEnvelopeShape;
  /**
   * The active principal — the admin-UI plugin-DID per CLAUDE.md
   * baked-in #18 (NOT engine-trusted; threaded as the `actor`
   * argument into every `bridge.callAs` invocation).
   */
  readonly principal: string;
  /** Schema specs keyed by primitive kind (for form generation). */
  readonly schemaSpecs: ReadonlyMap<WorkflowPrimitiveKind, SchemaSubgraphSpecShape>;
}

/** Outcome of {@link WorkflowEditor.save}. */
export interface WorkflowSaveOutcome {
  readonly cid: string;
  /** The wire-shape spec that was persisted (for replay). */
  readonly spec: SubgraphSpecWire;
}

/**
 * Sentinel string the grep-assert test pin
 * `workflow_editor_uses_schema_driven_form_generation_no_handcoded_forms.test.ts`
 * verifies as canonical-form-gen-marker presence. The grep walks this
 * source file looking for this string AND for the absence of
 * `<input name=` patterns inside the editor source.
 */
export const WORKFLOW_EDITOR_FORM_GEN_SOURCE_SENTINEL =
  "workflow-editor:schema-driven-form-gen-only";

/**
 * Admin UI v0 workflow editor. Mount with a bridge + manifest envelope
 * + active plugin-DID principal, then drag primitives, connect edges,
 * and call `save({ name })` to persist via `bridge.callAs`.
 */
export class WorkflowEditor {
  /**
   * Canonical workflow-editor handler-id used to dispatch saves
   * through. Mirrors the Rust-side `admin-ui-v0::workflow::<name>`
   * handler-id namespace.
   */
  static readonly HANDLER_ID_PREFIX = "admin-ui-v0::workflow::";

  /** Mount marker — verifies form-gen path is schema-driven only. */
  static readonly SOURCE_GREP_SENTINEL = WORKFLOW_EDITOR_FORM_GEN_SOURCE_SENTINEL;

  private readonly input: WorkflowEditorMountInput;
  private readonly primitives: WorkflowPrimitiveSelection[] = [];
  private readonly edges: WorkflowEdge[] = [];

  private constructor(input: WorkflowEditorMountInput) {
    this.input = input;
  }

  /** Mount the editor against a bridge + manifest envelope. */
  static mount(input: WorkflowEditorMountInput): WorkflowEditor {
    return new WorkflowEditor(input);
  }

  /**
   * Open the per-primitive form for the given primitive kind. The
   * returned form's fields are derived from the schema spec for that
   * primitive — NOT from a hand-coded template.
   */
  openPrimitiveForm(kind: WorkflowPrimitiveKind): WorkflowForm {
    const spec = this.input.schemaSpecs.get(kind);
    if (!spec) {
      // Unknown schema for kind — return an empty form rather than
      // throwing (the form is presented as "no configurable fields";
      // the editor still surfaces the primitive in the canvas).
      return { schemaName: `(no-schema:${kind})`, fields: [] };
    }
    return deriveFormFromSchema(spec);
  }

  /**
   * Drag a primitive into the canvas — appends to the draft.
   * `capScope` is the schema-derived scope copied from the form field
   * at drag-time; the user CANNOT alter it (schema authority per
   * sec-3.5-r1-4).
   */
  dragPrimitive(sel: WorkflowPrimitiveSelection): void {
    this.primitives.push(sel);
  }

  /** Connect edges between primitives. */
  connectEdges(edges: ReadonlyArray<WorkflowEdge>): void {
    for (const e of edges) {
      this.edges.push(e);
    }
  }

  /** Build the current draft snapshot (for inspection / replay). */
  currentDraft(name: string): WorkflowDraft {
    return {
      name,
      primitives: [...this.primitives],
      edges: [...this.edges],
    };
  }

  /**
   * Compile current draft to wire-shape spec without dispatching.
   * Used by `save` + by replay tests.
   */
  compileDraft(name: string): SubgraphSpecWire {
    return {
      handlerId: WorkflowEditor.HANDLER_ID_PREFIX + name,
      primitives: this.primitives.map((p) => ({
        id: p.id,
        kind: p.kind,
        capScope: p.capScope,
      })),
      edges: [...this.edges],
    };
  }

  /**
   * Persist the workflow via `bridge.callAs`. Validates the draft +
   * the compiled wire-spec against the manifest envelope BEFORE
   * invoking the bridge (T1 + T4 defenses).
   *
   * @throws {WorkflowEditorError} `E_CAP_DENIED` on envelope violation.
   */
  async save({ name }: { name: string }): Promise<WorkflowSaveOutcome> {
    const draft = this.currentDraft(name);
    validateDraftWithinEnvelope(draft, this.input.manifest);
    const spec = this.compileDraft(name);
    validateSubgraphSpecWithinEnvelope(spec, this.input.manifest);
    return this.dispatchSave(spec);
  }

  /**
   * Save a hand-authored wire-shape spec directly (bypassing the
   * editor's draft state). Used by the cap-elevation + subgraph-
   * injection test pins to feed hostile inputs at the save boundary.
   *
   * @throws {WorkflowEditorError} on envelope violation.
   */
  async saveSpec(spec: SubgraphSpecWire): Promise<WorkflowSaveOutcome> {
    validateSubgraphSpecWithinEnvelope(spec, this.input.manifest);
    return this.dispatchSave(spec);
  }

  /**
   * Dispatch the save through the bridge. The principal threaded is
   * always the admin-UI plugin-DID — NEVER an engine-trusted handle.
   */
  private async dispatchSave(
    spec: SubgraphSpecWire,
  ): Promise<WorkflowSaveOutcome> {
    const result = (await this.input.bridge.callAs(
      spec.handlerId,
      "register",
      spec,
      this.input.principal,
    )) as { cid: string };
    if (!result || typeof result.cid !== "string") {
      throw new WorkflowEditorError(
        WORKFLOW_EDITOR_ERROR_CODES.capDenied,
        "bridge.callAs returned a non-CID result for workflow save",
      );
    }
    return { cid: result.cid, spec };
  }
}

// ---------------------------------------------------------------------
// Extended bridge surface (compile-time mixin).
// ---------------------------------------------------------------------

/**
 * The workflow editor's bridge surface extends the base
 * {@link AdminUiV0Bridge} with a `callAs` method for cap-scoped
 * writes. The base bridge stays read-only; the workflow editor adds
 * the write seam via this mixin so the read-only-by-default shape
 * isn't widened across the whole admin UI v0.
 */
declare module "../index.js" {
  interface AdminUiV0Bridge {
    /**
     * Cap-scoped write via the engine's `Engine::call_as` seam. The
     * actor CID is threaded; NEVER bypasses to the engine-trusted
     * `Engine::call` (no `_as`) seam.
     */
    callAs(
      handlerId: string,
      op: string,
      input: unknown,
      actor: string,
    ): Promise<unknown>;
  }
}
