# Benten Engine

> Everything is a graph; materialized as anything.
> Unified graph store and runtime in Rust.

Your backend is a graph.
Your frontend is a graph.
Your data is a graph.
Your AI is a graph.
Your community is a graph.

Everything is a graph; materialized as anything.

One engine. Data and logic live in the same content-addressed structure; the engine walks that structure to execute.

## What this is

A Rust graph engine where **the application is just more graph**. Handlers aren't source-code strings — they're subgraphs of operation Nodes that the engine walks. Data, code, queries, capabilities, and audit trails share one representation and one storage layer.

- **Content-addressed.** Every Node has a CID (BLAKE3 + DAG-CBOR). Identical content has identical identity across machines.
- **Bounded by construction.** Handlers are DAGs of 12 operation primitives. Not Turing complete; guaranteed to terminate.
- **Incremental by default.** Materialized views stay current through change subscriptions, so common reads are O(1).
- **Capability-shaped, policy-pluggable.** The engine has a pre-write hook; policy backends plug in. The default is open; a revocation-aware grant backend ships with it.
- **Content-identity is the sync story.** Because everything is a CID, two machines merge by exchanging content, not by reconciling schemas.

## What it looks like

```sh
npx create-benten-app my-app
cd my-app && npm install && npm test
```

```typescript
import { crud } from '@benten/engine';

// A five-action handler: create / get / list / update / delete.
export const postHandlers = crud('post');
```

```typescript
import { Engine } from '@benten/engine';

const engine = await Engine.open('.benten/my-app.redb');
const handler = await engine.registerSubgraph(postHandlers);

await engine.call(handler.id, 'post:create', { title: 'Hello', body: 'Works.' });
const { items } = await engine.call(handler.id, 'post:list', {});
```

The handler is data. You can inspect it:

```typescript
console.log(handler.toMermaid());                                     // visual diagram
console.log(await engine.trace(handler.id, 'post:create', { ... })); // step-by-step trace
```

## Current state

Phase 1 shipped (2026-04-21). Phase 2a is closing — implementation complete; quality-council review in progress ahead of the release-era CI pass and tag. Phase 2a extended the evaluator with the WAIT primitive, the multiplicative iteration budget, system-zone runtime enforcement, structural causal attribution, immutability enforcement, capability TOCTOU hardening, and DAG-CBOR suspended-state persistence. Phase 2b adds WASM SANDBOX, STREAM, and SUBSCRIBE.

**Live today:** the eight Phase-1 primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT), the `crud()` zero-config path, content-addressed storage with MVCC, hand-written IVM views, pluggable capability policy, scaffolder, debug tooling, `handler.toMermaid()` and `engine.trace()` introspection. TypeScript bindings via napi-rs; Rust API available directly.

**Not yet live:** SANDBOX, STREAM, and SUBSCRIBE return `E_PRIMITIVE_NOT_IMPLEMENTED` until Phase 2b. P2P sync and UCAN land in Phase 3. See [`docs/HOW-IT-WORKS.md`](docs/HOW-IT-WORKS.md) for how the phases compose.

## Start here

| If you want… | Read |
|---|---|
| The 10-minute quickstart | [`docs/QUICKSTART.md`](docs/QUICKSTART.md) |
| The plain-English tour of Benten | [`docs/HOW-IT-WORKS.md`](docs/HOW-IT-WORKS.md) |
| The architecture at depth — crates, boundaries, invariants | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) |
| Error codes by discriminant, with context | [`docs/ERROR-CATALOG.md`](docs/ERROR-CATALOG.md) |
| Terms that mean something specific here | [`docs/GLOSSARY.md`](docs/GLOSSARY.md) |
| How to contribute | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| How to report a security issue | [`SECURITY.md`](SECURITY.md) |

## Repository layout

```
benten-engine/
├── crates/          # 7-crate Rust workspace (see ARCHITECTURE)
├── bindings/napi/   # Node.js bindings (native + WASM) via napi-rs v3
├── packages/engine/ # TypeScript DSL wrapper (@benten/engine)
├── tools/           # create-benten-app scaffolder + dev tooling
├── docs/            # Public documentation
└── .github/         # CI workflows
```

## Tech stack

Rust 2024 edition, MSRV 1.89, stable 1.94+.

Core: `blake3`, `serde_ipld_dagcbor`, `multihash`, `redb` 4, `papaya`, `mimalloc`, `thiserror` 2, `tracing`, `criterion` 0.8, `proptest`, `wasmtime` (Phase 2b), `napi-rs` 3, `cargo-nextest`.

Phase 3+: `iroh`, `Loro`, `ed25519-dalek`, `ssi`, `uhlc`.

## License

MIT OR Apache-2.0
