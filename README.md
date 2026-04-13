# Benten Engine

A Rust-native graph execution engine where data storage, computation, reactivity, synchronization, and capability enforcement are unified in a single system.

**This is not a database.** It is the runtime foundation for the Benten universal composable platform — a system where every person, family, or organization can own their data, sync it with others, and fork at any time.

## What It Does

- **Graph storage** — Nodes, Edges, labels, properties. The universal data model.
- **Incremental View Maintenance** — Answers are pre-computed and maintained in real-time as data changes. Reads are O(1).
- **Version chains** — Every mutation creates a version. History IS the graph. Undo, audit, time-travel built in.
- **Capability enforcement** — UCAN-compatible capability grants checked at the data layer. The engine rejects unauthorized operations before they reach storage.
- **CRDT sync** — Subgraphs sync between instances. Conflicts resolve automatically. Either party can fork.
- **Reactive notifications** — Subscribe to data changes. No polling.
- **True concurrency** — MVCC for readers, fine-grained locking for writers.
- **Embeddable everywhere** — Native (servers), WASM (browsers/edge), napi-rs (Node.js).

## Why Build This

No existing database combines all of the above. We tested PostgreSQL+AGE, PGlite, Grafeo, SurrealDB, CozoDB, and others. Each has fundamental limitations. See `docs/SPECIFICATION.md` for the full analysis.

## Architecture

```
benten-engine/
├── crates/
│   ├── benten-core/          # Node, Edge, Label, Property types
│   ├── benten-graph/         # Graph storage, indexes, traversal
│   ├── benten-ivm/           # Incremental View Maintenance
│   ├── benten-version/       # Version chains, CURRENT pointers
│   ├── benten-capability/    # Capability grants, enforcement, UCAN
│   ├── benten-sync/          # CRDT merge, sync protocol
│   ├── benten-query/         # Cypher parser + query planner
│   ├── benten-persist/       # WAL, snapshots, disk storage
│   ├── benten-reactive/      # Subscription management, notifications
│   └── benten-engine/        # Orchestrator: ties everything together
├── bindings/
│   ├── napi/                 # Node.js bindings via napi-rs
│   └── wasm/                 # WASM bindings via wasm-bindgen
├── tests/
│   ├── integration/          # Cross-crate integration tests
│   └── benchmarks/           # Performance benchmarks
└── docs/
    └── SPECIFICATION.md      # Full specification
```

## Status

Pre-development. Specification phase.

## Related

- [Thrum](../thrum/) — The TypeScript platform that runs on this engine
- [Specification](docs/SPECIFICATION.md) — Complete engine specification
