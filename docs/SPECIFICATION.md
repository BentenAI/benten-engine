# Benten Engine — Specification Document

**Created:** 2026-04-13
**Purpose:** Complete specification for the Benten graph execution engine — a custom Rust-native system where data storage, computation, reactivity, synchronization, and capability enforcement are unified.
**Context:** This specification synthesizes ~15 research explorations, multiple architectural discussions, and database spike tests conducted during the Thrum V3→V4 transition.

---

## 1. Why Build This

### 1.1 The Problem

Every existing database we evaluated has fundamental limitations for our use case:

| Database | What's Good | What's Missing |
|----------|------------|----------------|
| PostgreSQL+AGE | Full SQL + Cypher, ACID, MVCC, indexes | External process, network latency, not embeddable |
| PGlite+AGE | In-process, full PostgreSQL features | Single-threaded WASM, 860MB memory, no concurrent reads/writes |
| Grafeo | Fast (<0.1ms), embedded, Cypher, ACID | No sorted indexes, no MVCC, single-writer lock |
| CozoDB | Datalog + graph + Rust + WASM | Abandoned since Nov 2024 |
| SurrealDB | Multi-model, embedded, LIVE queries | BSL license (not open source until 2030) |

No existing tool combines: graph data model + reactive incremental view maintenance + CRDT sync + capability enforcement + embeddable + WASM + true concurrent read/write.

### 1.2 The Vision

Benten is a universal composable platform for the decentralized web. Every person, family, or organization runs their own instance. Data is owned by the user. Instances sync subgraphs bidirectionally. Either party can fork at any time.

The engine is the foundation that makes this possible. It is not a database that an application queries. It IS the application's runtime — data and computation are unified.

### 1.3 What We Learned from Research

**15 exploration documents produced:**
- `explore-wasm-sandbox-2026.md` — @sebastianwessel/quickjs for code sandbox
- `explore-module-communication-2026.md` — 4 channels: events, services, store, sync
- `explore-module-communication-fresh-eyes-2026.md` — capabilities as primitives, contracts
- `explore-capability-system-design.md` — UCAN-compatible capability grants
- `explore-sync-as-channel.md` — sync is a 4th channel with unique semantics
- `explore-sync-as-store-layer.md` — sync wraps store as a decorator, zero engine changes
- `explore-sync-as-fundamental.md` — two layers, one graph (graph for definitions, runtime for execution)
- `explore-p2p-sync-precedents.md` — AT Protocol, Holochain, UCAN, NextGraph
- `explore-nextgraph-deep-dive.md` — adopt concepts (commit DAG, capabilities), not RDF
- `explore-graph-native-protocol.md` — version chains: anchor + snapshot + CURRENT pointer
- `explore-in-process-graph-db-2026.md` — Grafeo recommended but limited
- `explore-database-is-application.md` — IVM is the key insight; graph = computation substrate
- `explore-execution-in-data-2026.md` — nothing existing does it all; compose from Rust crates
- `explore-nextgraph-deep-dive.md` — commit DAG, cryptographic capabilities, broker topology

**Key insights that shaped the specification:**
1. The graph should be the single-layer runtime, not a database you query
2. Incremental View Maintenance eliminates the "query speed" problem — answers are pre-computed
3. Version chains (anchor + snapshot + CURRENT pointer) unify versioning in the graph
4. Capabilities replace fixed trust tiers — operator-configured, UCAN-compatible
5. Multi-tenancy dissolves into capability scopes
6. Subgraphs are query patterns, not containers — they emerge from the data
7. The same sync mechanism handles P2P, edge CDN, dev/prod, mobile
8. The "invokes" edge concept is subsumed by the service registry with capability mediation
9. Event dispatch should be reactive (IVM), not a lookup
10. History IS the graph — no separate operation log

---

## 2. What the Engine Does

### 2.1 Core Primitives

**Node** — The universal data unit.
- Stable identity (anchor) with version chain
- Properties: typed key-value pairs
- Labels: zero or more type tags
- Every Node is potentially versioned (CURRENT pointer to latest version)

**Edge** — The universal relationship.
- Directed, typed, with optional properties
- Connects two Nodes (by anchor identity)
- Version-aware: edges point to anchors, not specific versions

**Subgraph** — An emergent collection defined by a traversal pattern.
- Not a container — defined by a query pattern (e.g., "all Nodes reachable from X via Y edges")
- The unit of sync, sharing, and capability scoping
- Represented as a TraversalPattern Node in the graph itself

### 2.2 Incremental View Maintenance (IVM)

The engine maintains materialized views that update incrementally when data changes.

**How it works:**
1. A "view" is defined by a query pattern (e.g., "all EventHandlers listening to content:afterCreate, sorted by priority")
2. The engine pre-computes the result and caches it
3. When a write occurs (create/update/delete Node or Edge), the engine identifies which views are affected
4. Affected views are incrementally updated (not recomputed from scratch)
5. Reads of materialized views are O(1) — no query execution, just return the cached result

**What this means for Thrum:**
- Event handler lookup: O(1) read from a materialized view. Updated only when handlers are registered/unregistered.
- Capability check: O(1) read from a materialized view. Updated only when capabilities are granted/revoked.
- Content listing: O(1) read from a materialized view with sorted, paginated results. Updated incrementally when content changes.

**This is the key innovation.** It eliminates the query-speed problem entirely. The "answer exists before the question" — because it's maintained in real-time as data changes.

### 2.3 Version Chains

Every versionable entity has:
- An **anchor Node** (stable identity, never changes)
- **Version Nodes** (complete snapshots of the entity at each state)
- **NEXT_VERSION edges** (chain: v1→v2→v3)
- A **CURRENT pointer** (edge from anchor to latest version)

External edges point to anchors, not versions. Resolving "current state" is one hop: anchor→CURRENT→version.

History is the graph. Undo = move CURRENT pointer back. Audit = traverse version chain. Sync = exchange version Nodes.

### 2.4 Capability Enforcement

Capabilities are first-class Nodes in the graph:
- **CapabilityGrant Node**: `{ domain: 'store', action: 'read', scope: 'content/*' }`
- **GRANTED_TO edge**: from grant to entity (module, user, remote instance, AI agent)
- Checked at every operation boundary
- UCAN-compatible structure (for P2P, same grant serializes to a signed token)
- Attenuation: scope can only narrow, never widen
- Operator-configured via platform settings

The engine enforces capabilities at the data layer, not the application layer. A write that violates capabilities is rejected before it reaches storage.

### 2.5 CRDT Sync

Subgraphs sync between instances using CRDTs:
- Each write produces a versioned operation (captured in the version chain)
- Sync = exchange version Nodes newer than the peer's latest
- Conflict resolution per data type:
  - Node properties: per-field last-write-wins with Hybrid Logical Clocks
  - Edges: add-wins semantics
  - Graph structure: schema validation on receive
- Either party can fork (stop syncing, keep all data)

### 2.6 Reactive Notifications

When data changes, the engine notifies subscribers:
- Subscribe to a Node: get notified when it changes
- Subscribe to a query pattern: get notified when the result set changes
- Subscribe to a subgraph: get notified when any Node/Edge in scope changes

This replaces polling, webhooks, and event bus dispatch for the common case.

### 2.7 Concurrency

- True multi-threaded read/write (Rust ownership model prevents data races)
- MVCC: readers see a consistent snapshot while writers modify
- Serializable transactions for atomic multi-operation writes
- No single-writer lock (unlike Grafeo)

---

## 3. How It Relates to the Existing Codebase

### 3.1 What Carries Forward

**From @benten/engine (TypeScript):**
- Node/Edge type definitions (adapted to Rust, exposed via napi-rs bindings)
- EngineError codes and error model
- The concept of Store as an interface (becomes the engine's API)

**From @benten/cms:**
- Content type definitions, block definitions, composition model
- Schema-as-code pattern (defineContentType, defineBlock)
- Materializer pipeline pattern

**From @benten/store-postgres:**
- Query patterns (the kinds of queries Thrum needs inform the engine's query optimizer)
- Graph CRUD operations (create/get/update/delete Node/Edge)

**From V3-5A:**
- syncRegistriesToGraph concept (becomes native: definitions ARE the graph)
- DEFINITION_NODE_TYPES (Module, ContentType, FieldDef, Block — become native labels)
- IoC registerGraphSync pattern (becomes reactive: modules subscribe, engine notifies)

**From apps/web:**
- SvelteKit web app (unchanged — consumes the engine via TypeScript bindings)
- UI components, blocks, admin

### 3.2 What Gets Replaced

| Current | Replaced By |
|---------|-------------|
| In-memory registries (Maps) | Materialized views in the engine |
| Event bus (sortedSnapshot, dispatch) | Reactive subscriptions + IVM |
| RestrictedEventBus + TIER_MATRIX | Capability enforcement at engine level |
| PostgreSQL+AGE for graph | The engine's native graph storage |
| PostgreSQL for relational | The engine's indexed property queries |
| createSingleton pattern | Service registration as graph Nodes |
| compositions.previous_blocks | Version chains |
| content_revisions table | Version Nodes |
| module_settings table | Settings as graph Nodes |
| clearModules/clearRegistries | N/A — graph is persistent |

### 3.3 The TypeScript Layer After Migration

The TypeScript layer becomes:
- **API bindings** (napi-rs generated types for engine operations)
- **Domain logic** (CMS content types, blocks, compositions, materializer)
- **Web framework** (SvelteKit, routes, UI components)
- **Module system** (defineModule → writes to graph, lifecycle hooks)

The TypeScript layer does NOT contain:
- Storage logic (the engine handles persistence)
- Event dispatch (the engine handles reactive notifications)
- Capability checks (the engine enforces at write time)
- Version management (the engine handles version chains)
- Sync logic (the engine handles CRDT merge)

---

## 4. Architecture

### 4.1 Rust Crate Structure

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
└── Cargo.toml                # Workspace manifest
```

### 4.2 Key Rust Crate Dependencies

| Crate | Purpose |
|-------|---------|
| petgraph | Graph data structures (adjacency list) |
| crepe or datafrog | Datalog evaluation for IVM rules |
| yrs or automerge | CRDT sync primitives |
| redb | Embedded persistent key-value storage |
| napi-rs | Node.js native bindings |
| wasm-bindgen | WASM compilation target |
| tokio | Async runtime for concurrency |
| dashmap | Concurrent HashMap for MVCC |

### 4.3 API Surface (TypeScript)

```typescript
// Core operations
engine.createNode(labels: string[], properties: Record<string, Value>): NodeId
engine.getNode(id: NodeId): Node | null
engine.updateNode(id: NodeId, properties: Record<string, Value>): void
engine.deleteNode(id: NodeId): void

engine.createEdge(type: string, from: NodeId, to: NodeId, properties?: Record<string, Value>): EdgeId
engine.getEdges(nodeId: NodeId, options?: { direction?, type? }): Edge[]
engine.deleteEdge(id: EdgeId): void

// Queries (Cypher)
engine.query(cypher: string, params?: Record<string, Value>): QueryResult

// Materialized views (IVM)
engine.createView(name: string, query: string): ViewId
engine.readView(name: string): QueryResult  // O(1) — pre-computed
engine.dropView(name: string): void

// Reactive subscriptions
engine.subscribe(query: string, callback: (change: ChangeEvent) => void): SubscriptionId
engine.unsubscribe(id: SubscriptionId): void

// Version chains
engine.getHistory(nodeId: NodeId): VersionNode[]
engine.rollback(nodeId: NodeId, version: number): void

// Capabilities
engine.grantCapability(entity: EntityId, capability: Capability): void
engine.revokeCapability(entity: EntityId, capability: Capability): void
engine.checkCapability(entity: EntityId, capability: Capability): boolean

// Transactions
engine.transaction(fn: (tx: Transaction) => void): void

// Sync
engine.sync(peer: PeerConnection, subgraph: TraversalPattern): SyncResult
engine.fork(subgraph: TraversalPattern): void

// Persistence
engine.open(path: string): void  // disk-backed
engine.create(): void            // in-memory
engine.checkpoint(): void        // force WAL flush
engine.close(): void
```

---

## 5. What the Engine Does NOT Do

- **Rendering** — the TypeScript materializer pipeline handles this
- **HTTP serving** — SvelteKit handles this
- **Code sandboxing** — @sebastianwessel/quickjs handles this (via engine API)
- **UI** — Svelte components handle this
- **Module discovery** — npm/registry handles this
- **P2P networking** — Yggdrasil handles this (engine provides sync primitives)

The engine is the DATA + COMPUTATION foundation. Everything above it is composed in TypeScript.

---

## 6. Performance Targets

Based on our spike tests and the "single-layer runtime" vision:

| Operation | Target | Notes |
|-----------|--------|-------|
| Node lookup by ID | <0.01ms | Indexed, O(1) |
| Materialized view read | <0.01ms | Pre-computed, O(1) |
| Event handler resolution | <0.01ms | Materialized view |
| Capability check | <0.01ms | Materialized view |
| Content listing (paginated) | <0.1ms | Sorted index, O(log n) |
| Node creation | <0.01ms | Write + IVM update |
| Edge creation | <0.01ms | Write + IVM update |
| Transaction (5 ops) | <0.1ms | Atomic commit |
| Version chain traversal | <0.1ms | Linked list walk |
| Sync (100 version Nodes) | <10ms | CRDT merge |
| Concurrent readers | Unlimited | MVCC snapshots |
| Concurrent writers | Lock-free or fine-grained | Per-Node locking |

---

## 7. Development Approach

- **Rust crates** with comprehensive test suites
- **napi-rs bindings** tested from TypeScript/Vitest
- **WASM bindings** tested in browser environment
- **Benchmark suite** tracking performance across development
- **Compatibility tests** verifying the engine can serve Thrum's existing test suite
- **Agent-driven development** using the same ADDL pipeline as Thrum

---

## 8. Open Questions

1. Should the Cypher query language be the primary API, or a Rust-native API with Cypher as one frontend?
2. How much of the IVM system needs to land in the first release vs being added incrementally?
3. Should CRDT sync be part of the engine or a separate crate that uses the engine?
4. What persistent storage backend? redb (pure Rust) vs RocksDB (C++ with Rust bindings)?
5. Should the engine support SQL as well as Cypher for relational-style queries?
6. How do we handle schema evolution (adding/removing labels, properties) during sync?
