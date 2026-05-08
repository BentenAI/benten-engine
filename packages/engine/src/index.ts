// `@benten/engine` — TypeScript DSL wrapper over the Benten graph engine.
//
// Three things live here:
//   1. The `Engine` class — open / registerSubgraph / call / trace / close.
//   2. The DSL — `subgraph()`, the 12 primitive helpers, `crud()` shorthand.
//   3. `toMermaid()` — pure diagram renderer for any built Subgraph.
//
// Typed errors live in `@benten/engine/errors` (sibling subpath export).

export { Engine, PolicyKind } from "./engine.js";

// Phase-3 G16-D wave-6b — Atrium B-prime factory-handle TS surface
// per Ben's D1 ratification 2026-05-05.
export type {
  Atrium,
  AtriumConfig,
  AtriumFactory,
  AtriumSubscription,
  PeerLifecycleCallback,
  SubscribeCallback,
} from "./atrium.js";

export {
  branch,
  call,
  crud,
  emit,
  isCrudHandler,
  isSubgraph,
  iterate,
  read,
  respond,
  sandbox,
  stream,
  subgraph,
  SubgraphBuilder,
  BranchBuilder,
  CaseBuilder,
  subscribe,
  transform,
  typedCall,
  wait,
  write,
  type BranchArgs,
  type CallArgs,
  type CrudHandler,
  type CrudOptions,
  type EmitArgs,
  type IterateArgs,
  type ReadArgs,
  type RespondArgs,
  type SandboxArgs,
  type StreamArgs,
  type SubscribeArgs,
  type TransformArgs,
  type TypedCallNodeArgs,
  type WaitArgs,
  type WriteArgs,
} from "./dsl.js";

export { toMermaid } from "./mermaid.js";

export {
  assertSandboxComposed,
  isSandboxBearing,
} from "./sandbox.js";

// Phase 2b G10-B — module manifest helpers (TS-side mirror of Rust
// `module_manifest::ManifestSummary` rendering). The top-level engine
// install/uninstall surface lives on `Engine` itself in `./engine.ts`.
export {
  manifestSummary,
  renderManifestSummary,
  type ManifestSummary,
} from "./manifest.js";

// Phase 2b G6-B: STREAM + SUBSCRIBE consumer-side wrappers.
export {
  wrapStreamHandle,
  validateStreamCallArgs,
  type NativeStreamHandle,
} from "./stream.js";
export {
  wrapEmitSubscriptionHandle,
  wrapSubscriptionHandle,
  serializeCursor,
  validateOnChangeArgs,
  validateOnEmitArgs,
  type NativeEmitSubscriptionJs,
  type NativeSubscriptionJs,
  type OnChangeCallback,
  type OnEmitCallback,
} from "./subscribe.js";

export type {
  AttributionFrame,
  CapabilityClaim,
  CapabilityGrant,
  Chunk,
  DeviceAttestation,
  Edge,
  EmitSubscription,
  HandlerAdjacencies,
  JsonValue,
  ManifestSignature,
  MigrationStep,
  ModuleManifest,
  ModuleManifestEntry,
  Outcome,
  Primitive,
  RegisteredHandler,
  SandboxArgsByCaps,
  SandboxArgsByName,
  SandboxNodeDescription,
  SandboxOptions,
  SandboxResult,
  Strategy,
  StreamCursor,
  StreamHandle,
  Subgraph,
  SubgraphNode,
  SubscribeCursor,
  Subscription,
  SuspensionResult,
  Trace,
  TraceStep,
  TraceStepUnknown,
  TypedCallInput,
  TypedCallInputShapes,
  TypedCallOp,
  TypedCallOutput,
  TypedCallOutputShapes,
  UserView,
  UserViewInputPattern,
  UserViewSpec,
  Value,
  ViewDef,
  ViewDelta,
} from "./types.js";

// Phase-3 G21-T2 — typed-CALL value re-exports (constants).
export { TYPED_CALL_PREFIX, TYPED_CALL_REQUIRED_CAP } from "./types.js";

export {
  buildUserViewHandle,
  resolveUserViewStrategy,
  userViewSpecToNativeJson,
  validateUserViewSpec,
} from "./views.js";
