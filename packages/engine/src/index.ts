// `@benten/engine` — TypeScript DSL wrapper over the Benten graph engine.
//
// Three things live here:
//   1. The `Engine` class — open / registerSubgraph / call / trace / close.
//   2. The DSL — `subgraph()`, the 12 primitive helpers, `crud()` shorthand.
//   3. `toMermaid()` — pure diagram renderer for any built Subgraph.
//
// Typed errors live in `@benten/engine/errors` (sibling subpath export).

export { Engine } from "./engine.js";

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
  type WaitArgs,
  type WriteArgs,
} from "./dsl.js";

export { toMermaid } from "./mermaid.js";

export type {
  HandlerAdjacencies,
  JsonValue,
  Primitive,
  RegisteredHandler,
  Subgraph,
  SubgraphNode,
  Trace,
  TraceStep,
  Value,
} from "./types.js";
