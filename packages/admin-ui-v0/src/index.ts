// G24-A canary — admin UI v0 browser-wasm32 + Tauri embedded-webview entry.
//
// The admin UI v0 ships AS the first app-level plugin per CLAUDE.md
// baked-in #18. The handler-side Rust module at
// `crates/benten-platform-foundation/src/admin_ui_v0/` carries the
// canonical sources-of-truth (4-category nav, route subgraph
// composition, subscribe pattern shape, IndexedDB allowed/forbidden
// store names, WinterTC forbidden APIs, private-namespace prefix).
//
// THIS TS-side surface is the browser shell that loads the
// wasm32-unknown-unknown bundle, presents the 4-category nav, and
// routes reads via the napi/wasm bridge to `Engine.read_node_as` (the
// CLASS B β seam per CLAUDE.md baked-in #18). The TS shell NEVER
// references engine-internal seams — `subscribe_change_events`,
// `read_node` (without `_as`) — both are pinned absent by Rust-side
// grep-assert tests at `crates/benten-engine/tests/admin_ui_v0_*`.
//
// ## Subscribe seam — `on_change_as_with_cursor` ONLY (sec-3.5-r1-9)
//
// All TS-side subscribe paths route through the bridge's
// `on_change_as_with_cursor` method. The grep-assert pin
// `admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`
// verifies this constant string appears here (and that
// `subscribe_change_events` never appears anywhere in this directory).
//
// ## Cap-scoped reads — `read_node_as` ONLY (cag-r1-9)
//
// All TS-side reads route through the bridge's `read_node_as` method.
// The pub(crate) engine-internal `read_node` (no `_as` suffix) is NEVER
// referenced. The grep-assert pin
// `admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`
// verifies this.
//
// ## IndexedDB scope — snapshot_cache + manifest_store ONLY (br-r1-7)
//
// The browser bundle's IndexedDB persistence is for snapshot cache +
// manifest store ONLY. No `caps` / `ucan` / `secrets` / `sync_state`
// / `loro_state` / `iroh_state` object stores. The grep-assert pin
// `admin_ui_v0_indexeddb_writes_only_snapshot_cache_and_manifest_store.rs`
// enforces.
//
// ## WinterTC future-compat (br-r1-8)
//
// No DOM-only / FormData / fetch-relative URLs. CI guard at G26-B.

/**
 * The 4 admin UI v0 categories (canonical order per ratification #4).
 * Mirrors `benten_platform_foundation::NAV_CATEGORIES` (Rust side
 * source-of-truth).
 */
export const ADMIN_UI_V0_CATEGORIES = [
  "Plugins",
  "Workflows",
  "Content Types",
  "Views",
] as const;

export type AdminUiV0Category = (typeof ADMIN_UI_V0_CATEGORIES)[number];

/**
 * Canonical URL-route slugs (kebab-case so multi-word labels survive
 * routing). Mirrors `benten_platform_foundation::Category::route_slug`.
 */
export const ADMIN_UI_V0_ROUTE_SLUGS: Readonly<Record<AdminUiV0Category, string>> = {
  Plugins: "plugins",
  Workflows: "workflows",
  "Content Types": "content-types",
  Views: "views",
};

/**
 * IndexedDB object stores the admin UI MAY write to. Mirrors
 * `INDEXEDDB_SNAPSHOT_CACHE_STORE` + `INDEXEDDB_MANIFEST_STORE_STORE`.
 */
export const ADMIN_UI_V0_INDEXEDDB_ALLOWED_STORES = [
  "snapshot_cache",
  "manifest_store",
] as const;

/**
 * Canonical seam name the TS-side bridge uses for cap-scoped reads —
 * the Class B β surface per CLAUDE.md baked-in #18. The TS bridge
 * invokes `bridge.readNodeAs(principal, cid)` which delegates over
 * napi/wasm to `Engine::read_node_as`. Pinned to be referenced here
 * by the grep-assert at
 * `admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`.
 */
export const ADMIN_UI_V0_CLASS_B_BETA_READ_SEAM = "read_node_as";

/**
 * Canonical seam name the TS-side bridge uses for change
 * subscriptions per sec-3.5-r1-9. Pinned by the grep-assert at
 * `admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`.
 */
export const ADMIN_UI_V0_SUBSCRIBE_SEAM = "on_change_as_with_cursor";

/**
 * Bridge surface (transport-agnostic). Browser-tab shape (b) wires a
 * fetch-backed implementation against the thin-client session
 * protocol (G24-F `DidKeyedSession`). Tauri embedded-webview shape
 * (c) wires the same surface to in-process IPC against the
 * `benten-renderer-tauri` crate's `IpcAllowlist` (G24-E).
 *
 * Production admin UI components compose against THIS surface only —
 * they never reach for global `fetch` directly (so the WinterTC
 * forbidden-API list is respected by construction).
 */
export interface AdminUiV0Bridge {
  /**
   * Cap-scoped read via the engine's CLASS B β seam (`read_node_as`).
   * The principal CID is threaded; NEVER bypasses to the
   * pub(crate) `read_node` engine-internal seam.
   */
  readNodeAs(principal: string, cid: string): Promise<unknown>;

  /**
   * Subscribe with cap-recheck via `on_change_as_with_cursor`. Bare
   * `subscribe_change_events` is NEVER called — that surface has no
   * cap-recheck on event delivery.
   */
  onChangeAsWithCursor(
    pattern: string,
    cursor: "latest" | { persistent: string },
    actor: string,
    callback: (event: unknown) => void,
  ): Promise<{ readonly subscriptionId: string }>;
}

/**
 * Bootstrap entrypoint. The browser-tab `index.html` invokes this
 * after loading the wasm bundle + getting a configured `bridge`.
 *
 * G24-A canary deliverable: the 4-category nav surface state + bridge
 * wiring. The browser-side DOM mount + workflow editor + view creator
 * land at G24-B + G24-C wave-6b (see `docs/ADMIN-UI.md §9.1`).
 */
export function bootstrapAdminUiV0(_bridge: AdminUiV0Bridge): {
  readonly categories: typeof ADMIN_UI_V0_CATEGORIES;
  readonly routeSlugs: typeof ADMIN_UI_V0_ROUTE_SLUGS;
  readonly subscribeSeam: typeof ADMIN_UI_V0_SUBSCRIBE_SEAM;
  readonly readSeam: typeof ADMIN_UI_V0_CLASS_B_BETA_READ_SEAM;
} {
  return {
    categories: ADMIN_UI_V0_CATEGORIES,
    routeSlugs: ADMIN_UI_V0_ROUTE_SLUGS,
    subscribeSeam: ADMIN_UI_V0_SUBSCRIBE_SEAM,
    readSeam: ADMIN_UI_V0_CLASS_B_BETA_READ_SEAM,
  };
}

/**
 * @deprecated R3 RED-PHASE placeholder; retained for legacy test pins
 * that haven't migrated to `bootstrapAdminUiV0`. New code uses
 * [`bootstrapAdminUiV0`].
 */
export function placeholder(): { readonly stage: "r3-red-phase" } {
  return { stage: "r3-red-phase" };
}

// G24-B wave-6b — workflow editor surface (admin UI v0 first plugin's
// workflow authoring path). See `./workflow-editor/index.ts` for full
// docs.
export {
  CANONICAL_12_PRIMITIVE_KINDS,
  WORKFLOW_EDITOR_ERROR_CODES,
  WORKFLOW_EDITOR_FORM_GEN_SOURCE_SENTINEL,
  WorkflowEditor,
  WorkflowEditorError,
  deriveFormFromSchema,
  manifestEnvelopeAdmits,
  validateDraftWithinEnvelope,
  validateSubgraphSpecWithinEnvelope,
} from "./workflow-editor/index.js";
export type {
  ManifestCapRequirement,
  ManifestEnvelopeShape,
  SchemaPrimitiveDescriptor,
  SchemaSubgraphSpecShape,
  SubgraphSpecWire,
  WorkflowDraft,
  WorkflowEdge,
  WorkflowEditorErrorCode,
  WorkflowEditorMountInput,
  WorkflowForm,
  WorkflowFormField,
  WorkflowPrimitiveKind,
  WorkflowPrimitiveSelection,
  WorkflowSaveOutcome,
} from "./workflow-editor/index.js";

// G24-C wave-6b — composed-view creator re-exports.
export {
  ComposedViewCreator,
  userViewSpec,
} from "./view-composer/index.js";
export type {
  ComposedViewCreatorBridge,
  LabelPattern,
  PreviewState,
  SaveOutcome,
  SubscribeCursor,
  TypedOutputProjection,
  UserViewSpec,
} from "./view-composer/index.js";
