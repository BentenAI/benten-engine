# Benten Engine

**The engine for the decentralized web.**

Benten is a Rust graph execution engine where every platform capability composes on top rather than being built in. On it, we're building personal AI assistants that replace the paid software stack — organizing your knowledge and generating the tools you need on demand, all running on hardware you trust. The platform is funded by treasury interest on its USD-pegged stable currency.

**Status:** Phase 1 complete (2026-04-21). Content-addressed graph engine with 8-primitive evaluator, 5-view IVM, pluggable capability policy, TypeScript bindings, scaffolder, and the 10-minute DX path shipped. Phase 2 (evaluator completion + WASM SANDBOX + remaining invariants) is the next milestone. See [`docs/FULL-ROADMAP.md`](docs/FULL-ROADMAP.md) for the committed 8-phase plan.

## Three Pillars

1. **The engine** — a Rust graph execution engine where data and code are one content-addressed structure. Every capability (auth, sync, storage, compute, governance, economics) composes from 12 operation primitives.
2. **The adoption driver** — personal AI assistants that organize your life via PARA methodology, generate tools on demand, and replace the paid software stack with one you own.
3. **The economic engine** — treasury bond interest on USD-pegged credit reserves (primary revenue), with a peer-to-peer compute network providing secondary revenue and driving credit utilization.

See [`docs/VISION.md`](docs/VISION.md) for the full articulation.

## Hello Benten

```sh
npx create-benten-app my-app
cd my-app && npm install && npm test
```

```typescript
// Your first handler — no schema, no auth, no config required:
import { crud } from '@benten/engine';
export const postHandlers = crud('post');
```

```typescript
// Use it:
import { Engine } from '@benten/engine';

const engine = await Engine.open('.benten/my-app.redb');
const handler = await engine.registerSubgraph(postHandlers);

await engine.call(handler.id, 'post:create', { title: 'Hello', body: 'Works.' });
const { items } = await engine.call(handler.id, 'post:list', {});
```

Every call produces a deterministic, content-addressed audit trail you can inspect:
```typescript
console.log(handler.toMermaid());                             // Visual diagram of the subgraph
console.log(await engine.trace(handler.id, 'post:create', {...}));  // Per-step evaluation trace
```

(Dev server with hot reload is Phase 2; Phase 1 ships the `npm test` path — the scaffolder smoke test exercises six exit-criterion gates, each mechanically verifiable.)

That's the entire onboarding surface. Complexity (capabilities, IVM views, version chains, P2P sync, AI integration) is opt-in as you need it.

See [`docs/QUICKSTART.md`](docs/QUICKSTART.md) for the full 10-minute path.

## What's Different

- **Code IS graph.** Handlers are subgraphs of operation Nodes, not source code strings. Inspectable, auditable, statically analyzable, versionable, forkable — and directly composable by AI agents.
- **Answers exist before questions.** Incremental View Maintenance pre-computes query results. Reads are O(1).
- **Not Turing complete by design.** Bounded DAGs. Guaranteed termination. WASM sandbox is the controlled escape hatch.
- **Capabilities as data, policy as plugin.** UCAN-compatible grants stored as Nodes. Pluggable policy backends.
- **History IS the graph.** Version chains with content-addressed hashing (CIDv1). Undo, audit, time-travel are graph traversals.

## Committed Scope (Phase 1-8)

| Phase | Deliverable |
|-------|-------------|
| **1** | ✅ Core engine: 7 crates, 12 primitive types (8 executable), 8 of 14 structural invariants, 5 hand-written IVM views, pluggable capability policy, napi-rs TypeScript bindings, scaffolder + debug tooling |
| **2** | Evaluator completion, WASM build target, wasmtime SANDBOX with fuel metering |
| **3** | P2P sync (iroh, CRDT, Merkle Search Trees, DID). **Atriums ship here.** |
| **4** | Thrum CMS migration to the engine — 3,200+ tests pass |
| **5** | Platform features — schema-driven rendering, self-composing admin, declarative plugin manifests |
| **6** | **Personal AI Assistant MVP** — MCP, PARA knowledge organization, on-demand tool generation |
| **7** | **Digital Gardens MVP** — community spaces with admin governance, invitation flows, moderation |
| **8** | **Benten Credits MVP** — USD-pegged currency, treasury-backed, FedNow on/off ramp, tab settlement |

Everything beyond Phase 8 (full Groves, federation, general compute marketplace, DAO transition) is exploratory. See [`docs/future/`](docs/future/).

## Start Here

| If you want... | Read |
|---------------|------|
| The 10-minute quickstart | [`docs/QUICKSTART.md`](docs/QUICKSTART.md) |
| The vision and three pillars | [`docs/VISION.md`](docs/VISION.md) |
| How the layers compose | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) |
| The 8-phase plan (committed + exploratory) | [`docs/FULL-ROADMAP.md`](docs/FULL-ROADMAP.md) |
| The Rust engine blueprint | [`docs/ENGINE-SPEC.md`](docs/ENGINE-SPEC.md) |
| The TypeScript developer API | [`docs/DSL-SPECIFICATION.md`](docs/DSL-SPECIFICATION.md) |
| Error codes and fixes | [`docs/ERROR-CATALOG.md`](docs/ERROR-CATALOG.md) |
| Terms with specific meaning | [`docs/GLOSSARY.md`](docs/GLOSSARY.md) |
| How to contribute | [`CONTRIBUTING.md`](CONTRIBUTING.md) |

## Repository Structure

```
benten-engine/
├── crates/          # Rust workspace crates (7 at Phase 1 close: benten-errors, benten-core, benten-graph, benten-ivm, benten-caps, benten-eval, benten-engine)
├── bindings/        # napi-rs, wasm, python bindings
├── tests/           # Cross-crate integration tests and benchmarks
├── docs/
│   ├── VISION, ARCHITECTURE, ENGINE-SPEC, PLATFORM-DESIGN, BUSINESS-PLAN,
│   │   DSL-SPECIFICATION, FULL-ROADMAP, QUICKSTART, ERROR-CATALOG,
│   │   DEVELOPMENT-METHODOLOGY, PROJECT-HISTORY, GLOSSARY
│   ├── future/      # Exploratory proposals not in committed scope
│   ├── research/    # Active explorations
│   ├── validation/  # Empirical artifacts (paper prototype, IVM benchmark)
│   └── archive/     # Historical critiques, reviews, superseded specs
├── Cargo.toml       # Workspace root
├── CLAUDE.md        # AI dev instructions
├── CONTRIBUTING.md
└── README.md        # This file
```

## The 12 Operation Primitives

```
READ     WRITE     TRANSFORM    BRANCH     ITERATE    WAIT
CALL     RESPOND   EMIT         SANDBOX    SUBSCRIBE  STREAM
```

Empirically validated against 5 real handlers with 2.5% SANDBOX rate. See [`docs/validation/paper-prototype-handlers.md`](docs/validation/paper-prototype-handlers.md). (Note: paper prototype used the original set — VALIDATE and GATE were dropped, SUBSCRIBE and STREAM added during the 2026-04-14 critic review. Phase 1 ships all 12 primitive *types* with structural validation; 8 have live executors today — READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT — and the other 4 — WAIT, STREAM, SUBSCRIBE, SANDBOX — return `E_PRIMITIVE_NOT_IMPLEMENTED` until their Phase 2 executors land.)

## Tech Stack (Validated April 2026)

Rust 1.94+ (2024 edition) · blake3 · serde_ipld_dagcbor · multihash (CIDv1) · redb 4 · papaya · mimalloc · thiserror 2 · tracing · criterion 0.8 · proptest · wasmtime · napi-rs 3 · cargo-nextest

Phase 3+: iroh · Loro · ed25519-dalek · ssi · uhlc

## License

MIT OR Apache-2.0
