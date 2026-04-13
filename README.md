# Benten Engine

A self-evaluating graph — a Rust-native execution engine where data and code are both Nodes and Edges. The graph evaluates itself. There is no distinction between "database" and "application."

**This is the foundation of the decentralized web.** Every person, family, or organization runs their own instance. Data is owned by the user. Instances sync subgraphs bidirectionally. Either party can fork at any time.

## What Makes It Different

**Code IS the graph.** A route handler is not a string of source code stored in a Node. It is a subgraph of operation Nodes connected by control-flow Edges. "Executing" a handler means the engine walks the subgraph.

**Answers exist before questions.** Incremental View Maintenance pre-computes query results and updates them in real-time as data changes. Reads are O(1).

**Not Turing complete by design.** Operation subgraphs are DAGs with bounded iteration. Every handler is guaranteed to terminate. The WASM sandbox is the escape hatch for complex computation.

**Capabilities, not permissions.** UCAN-compatible capability grants enforced at the engine level. Same system for local modules, remote instances, AI agents.

**History IS the graph.** Version chains with content-addressed hashing. Undo, audit, time-travel, and sync are all graph traversals.

## Architecture

```
benten-engine/
├── crates/
│   ├── benten-core/       # Node, Edge, Value types, content hashing, version chains
│   ├── benten-graph/      # Graph storage, indexes, MVCC, persistence, IVM
│   ├── benten-eval/       # 12 operation primitives, evaluator, capabilities
│   └── benten-engine/     # Orchestrator: public API
├── bindings/
│   ├── napi/              # Node.js/TypeScript bindings
│   └── wasm/              # WASM bindings (browsers/edge)
├── prototypes/
│   └── ivm/               # IVM algorithm benchmark (Algorithm B selected)
├── docs/
│   ├── ENGINE-SPEC.md     # Technical implementation blueprint
│   ├── PLATFORM-DESIGN.md # Networking, governance, sync architecture
│   ├── BUSINESS-PLAN.md   # Economics, token model, legal
│   └── DSL-SPECIFICATION.md # TypeScript/Python developer API
└── CLAUDE.md              # AI development instructions
```

## The 12 Operation Primitives

| Primitive | Purpose |
|-----------|---------|
| READ | Retrieve from graph |
| WRITE | Mutate graph (with automatic versioning) |
| TRANSFORM | Pure data reshaping (sandboxed expressions) |
| BRANCH | Conditional routing |
| ITERATE | Bounded collection processing |
| WAIT | Suspend for signal/timeout |
| GATE | Custom logic escape hatch |
| CALL | Execute another subgraph |
| RESPOND | Produce output |
| EMIT | Fire-and-forget notification |
| SANDBOX | WASM computation (no re-entrancy) |
| VALIDATE | Schema + integrity check |

## The Three Networking Tiers

- **Atriums** — Peer-to-peer. Partners, friends, student↔school.
- **Digital Gardens** — Community spaces. Member-mesh, no central server.
- **Groves** — Governed communities. Fractal, polycentric, fork-and-compete.

## Status

Specification complete and validated. Pre-development (setting up Rust workspace).

## Documentation

- [Engine Technical Spec](docs/ENGINE-SPEC.md) — Implementation blueprint
- [Platform Design](docs/PLATFORM-DESIGN.md) — Architecture beyond the engine
- [Business Plan](docs/BUSINESS-PLAN.md) — Economics and business model
- [DSL Specification](docs/DSL-SPECIFICATION.md) — TypeScript/Python developer API
- [Paper Prototype](docs/paper-prototype-handlers.md) — 5 validated handler designs
- [IVM Benchmark](prototypes/ivm/RESULTS.md) — Algorithm selection data

## Related

- [Thrum](../thrum/) — The TypeScript platform that will run on this engine
