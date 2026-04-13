# Benten Engine Technical Specification

**Created:** 2026-04-13
**Status:** WORKING DRAFT -- core architecture defined, open questions flagged for resolution before implementation.
**Audience:** Rust engineers building the engine. TypeScript developers building bindings and modules.
**Related documents:** [Platform Design](./PLATFORM-DESIGN.md) (networking, governance, identity) | [Business Plan](./BUSINESS-PLAN.md) (economics, revenue, regulatory)

---

## 1. Context

Benten is a self-evaluating graph -- a platform where data and code are both Nodes and Edges, the graph evaluates itself, and every person, family, or organization can own their data, share it selectively, and fork at any time. This document specifies the Rust engine that makes this possible: the data model, the 12 operation primitives, the evaluator, version chains, incremental view maintenance, the capability system, the WASM sandbox, and the crate structure. For the networking, governance, and economic layers that sit above this engine, see the [Platform Design](./PLATFORM-DESIGN.md).

---

## 2. Architecture: A Self-Evaluating Graph

The Benten engine is a Rust-native system where code is represented AS graph structure, not stored IN graph properties. A route handler is not a string of source code -- it is a subgraph of operation Nodes connected by control-flow Edges. "Executing" a handler means the engine walks the subgraph, performing each operation.

```
[RouteHandler: GET /api/posts]
    |--[FIRST_STEP]--> [GATE: require capability store:read:post/*]
    |                      +--[NEXT]--> [READ: query posts where published=true]
    |                                      +--[NEXT]--> [TRANSFORM: to JSON]
    |                                                      +--[NEXT]--> [RESPOND: 200]
    +--[ON_DENIED]--> [RESPOND: 403]
```

**Why code-as-graph:**
- **Inspectable.** An AI agent can read any handler by traversing its subgraph.
- **Modifiable at runtime.** Insert a caching step without recompilation.
- **Versionable.** The handler is a subgraph with a version chain. Roll back by moving the CURRENT pointer.
- **Syncable.** Installing a module = syncing its operation subgraphs. Code travels as data.
- **Statically analyzable.** The engine can pre-compute cost, check capabilities, and verify determinism BEFORE execution -- because subgraphs are DAGs.

---

## 3. The 12 Operation Primitives

Every computation in Benten is composed from 12 primitive operation Node types. The vocabulary is deliberately NOT Turing complete -- subgraphs are DAGs with bounded iteration. This guarantees termination, enables static analysis, and prevents denial-of-service.

| # | Primitive | Purpose | Key Property |
|---|-----------|---------|-------------|
| 1 | **READ** | Retrieve from graph (by ID, query, materialized view) | Typed error edges: ON_NOT_FOUND, ON_EMPTY |
| 2 | **WRITE** | Mutate graph (create, update, delete, conditional CAS) | Auto version-stamp. Typed error: ON_CONFLICT, ON_DENIED |
| 3 | **TRANSFORM** | Pure data reshaping | Sandboxed expression with arithmetic, array built-ins (filter, map, sum, etc.), object construction. No I/O. |
| 4 | **BRANCH** | Conditional routing | Forward-only, no cycles |
| 5 | **ITERATE** | Bounded collection processing | Mandatory maxIterations. Optional parallel. Multiplicative cost through nesting. |
| 6 | **WAIT** | Suspend until signal/timeout | For workflows, approval patterns. Decomposes to data Nodes for cross-instance. |
| 7 | **GATE** | Custom logic escape hatch | For complex validation/transformation that can't be expressed as TRANSFORM. Capability checking via `requires` property on any Node. |
| 8 | **CALL** | Execute another subgraph | `isolated` flag for capability attenuation. Mandatory timeout. |
| 9 | **RESPOND** | Terminal: produce output | HTTP response, event result, agent action result |
| 10 | **EMIT** | Fire-and-forget notification | `deliveryMode`: local, exactly-once, broadcast |
| 11 | **SANDBOX** | WASM computation escape hatch | No re-entrancy. Fuel-metered. Time-limited. Max output 1MB. For complex computation that can't be expressed as operation Nodes. |
| 12 | **VALIDATE** | Schema + referential integrity check | Before writes, on sync receive |

**Composed from the 12 (NOT primitives):** Retry (ITERATE + BRANCH + CALL), Parallel (ITERATE with parallel flag), Compensate (BRANCH on error + CALL undo subgraph), DryRun (evaluation mode property), Audit (EMIT to audit channel), Map/Filter/Reduce (ITERATE + TRANSFORM).

---

## 4. Structural Invariants

| # | Invariant | Purpose |
|---|-----------|---------|
| 1 | Subgraphs are DAGs (no cycles) | Guarantees termination |
| 2 | Max depth: configurable per capability grant | Bounds sequential operations |
| 3 | Max fan-out per node: configurable | Prevents combinatorial explosion |
| 4 | Max SANDBOX nesting: configurable | Bounds WASM call depth |
| 5 | Max total nodes per subgraph: 4096 default | Bounds validation cost |
| 6 | Max total edges per subgraph: 8192 default | Bounds graph size |
| 7 | Max SANDBOX output: 1MB default | Prevents memory exhaustion |
| 8 | Cumulative iteration budget (multiplicative) | Prevents nested loop explosion |
| 9 | Determinism classification per operation type | Enables sync correctness |
| 10 | Content-addressed hash per subgraph | Integrity verification, deduplication |
| 11 | System-zone labels unreachable from user operations | Kernel/userspace boundary |
| 12 | Registration-time structural validation | Malformed subgraphs never execute |
| 13 | Immutable once registered (new version for changes) | Prevents TOCTOU attacks |
| 14 | Causal attribution on every evaluation | Unsuppressible audit trail |

All numeric limits are configurable per capability grant -- the operator (or community governance) decides the bounds.

**Enforcement timing:** Invariants 1-10 and 12 are enforced at registration time (structural validation). Invariant 11 is enforced at both the evaluator level (operation dispatch) and the storage level (write path) for defense in depth. Invariant 13 is enforced by the storage layer (version nodes are append-only). Invariant 14 is enforced by the evaluator (attribution records are emitted automatically, not opt-in).

**Version pinning during execution:** When the evaluator begins walking a subgraph, it pins the CURRENT version of that subgraph and all transitively-CALLed subgraphs. Version changes during execution do not affect in-progress evaluations. This prevents mid-execution inconsistency.

---

## 5. The Evaluator

An iterative graph walker with an explicit execution stack:

```rust
let mut stack = vec![entry_node];
while let Some(node) = stack.pop() {
    let result = execute_operation(node, &mut context);
    match result {
        Ok(next_nodes) => stack.extend(next_nodes),
        Err(error_edge) => stack.push(follow_error_edge(node, error_edge)),
    }
}
```

**Properties:**
- Iterative, not recursive (no stack overflow risk, can pause/resume)
- Per-node overhead target: sub-100-microseconds for typical 10-node handlers (to be validated by Phase 1 benchmark suite; actual overhead depends on capability checks, IVM updates, and storage access patterns)
- Can serialize execution state (for WAIT -- save stack, resume later)
- Can step-through debug (pop one node, execute, inspect context)
- Shared context with declared read/write keys per operation

---

## 6. Version Chains

Every versionable entity has:
- **Anchor Node** -- stable identity, never changes
- **Version Nodes** -- complete snapshots
- **NEXT_VERSION edges** -- linking the chain (becomes a commit DAG when concurrent edits branch)
- **CURRENT pointer** -- edge from anchor to latest version

External edges point to anchors, not versions. Resolving current state: one hop (anchor -> CURRENT -> version).

History = traverse version chain. Undo = move CURRENT pointer. Sync = exchange version Nodes. Fork = stop syncing, keep all version history.

**Concurrent edit handling:** When two instances edit the same anchor concurrently, the version chain branches into a commit DAG. Both branches are valid. Merge occurs on sync receive: per-field LWW with HLC determines the merged version's property values. The merge result becomes a new version node with NEXT_VERSION edges from both branch tips.

**CURRENT pointer atomicity:** Moving the CURRENT pointer from version N to version N+1 is an atomic operation: create version N+1, create NEXT_VERSION edge, update CURRENT -- all within a single storage transaction. If the process crashes mid-operation, the transaction is rolled back and the anchor still points to version N.

**Version retention:** Configurable max versions per anchor (default: 100). Oldest versions are eligible for pruning after sync confirmation from all peers in active sync agreements. Delta compression (storing diffs instead of full snapshots) is a Phase 1 consideration to control storage growth. Pruned versions are replaced by a "compaction tombstone" that records the pruned range for sync protocol awareness.

---

## 7. Content-Addressed Hashing

Every version Node is hashed using BLAKE3 with a multihash prefix for algorithm agility. What gets hashed: labels + properties (NOT anchor ID, NOT timestamps, NOT edges).

**Used for:**
- Module identity and community attestation
- Sync integrity verification (what I received = what was sent)
- Version comparison (same hash = same content, instant comparison)
- Deduplication across instances
- Governance decision anchoring (votes reference specific hashes)
- Merkle trees for efficient sync (compare roots, transfer only differences)

**Canonical serialization:** CBOR with RFC 8949 deterministic encoding.

**Note on edge exclusion:** Edges are excluded from the content hash for performance and to allow the same version content to exist in different structural contexts. This means edge tampering (adding or removing relationships) is not detectable via content hash alone. Edge integrity during sync is verified separately through the sync protocol's Merkle tree comparison, which covers both node content and edge structure. See [Platform Design, Section 4.2](./PLATFORM-DESIGN.md) for the sync protocol's integrity guarantees.

---

## 8. Incremental View Maintenance (IVM)

The engine maintains materialized views that update incrementally on writes:

1. Define a view (a query pattern, stored as a Node)
2. Engine pre-computes the result
3. On write: engine identifies affected views and incrementally updates them
4. Read from view: O(1) -- return pre-computed result

**Critical for:** Event handler resolution, capability checks, content listings, knowledge attestation values, governance rule resolution.

**The key property:** Governance resolution (deep nesting, polycentric multi-parent) is O(1) at check time because the "effective rules" view is maintained incrementally when governance changes (rare), not recomputed when rules are checked (constant).

**Algorithm:** DBSP (Database Stream Processing) using Z-set algebra is the recommended approach, based on the formal foundations established by Feldera and the VLDB 2023 paper. DBSP supports incremental maintenance of recursive queries (required for transitive closure in governance inheritance and capability chain resolution). However, the IVM algorithm is the highest-risk component of the engine. Before Phase 2 implementation begins, a prototype must validate that DBSP can maintain Benten's specific view patterns (capability grants, event handler dispatch, content listings) within acceptable write latency budgets.

**Phase 1 scope:** Implement 3-5 specific materialized views (capability grants for a module, event handler dispatch table, content listing) using hand-written incremental update logic. This validates the view patterns and establishes performance baselines before committing to a general IVM engine.

**Resource bounds for views:** Since IVM views are defined as operation subgraphs, they inherit all structural invariants (Sections 4.1-4.14). Additionally:
- View creation requires a dedicated capability (`ivm:createView`), not included in community presets
- Maximum traversal depth in view definitions: configurable (default: 5 hops)
- Per-view CPU/memory budget for incremental updates: if a single write triggers an update exceeding the budget, the view is marked "stale" and recomputed asynchronously rather than blocking the write path

---

## 9. Capability System

Capabilities are UCAN-compatible typed objects stored as Nodes with GRANTED_TO edges:

```
[CapabilityGrant: {domain: 'store', action: 'read', scope: 'post/*'}]
    +-- GRANTED_TO --> [Entity: seo-module]
```

**Properties:**
- Operator-configured (not hardcoded tiers)
- Enforced at the engine level (every WRITE checks capabilities before executing)
- Attenuation: scope can only narrow, never widen (CALL with `isolated=true`)
- UCAN-compatible serialization for P2P (same grant, signed for network transport)
- Same system for modules, WASM sandboxes, remote instances, AI agents
- Multi-tenancy is just capability scopes (no separate tenant infrastructure)

**Replaces:** The 4 fixed trust tiers (platform/verified/community/untrusted) from V3. Tiers become optional presets for developer convenience.

**System-zone protection (Invariant 11):** Capability Nodes, version chain metadata, IVM view definitions, and other engine-internal Nodes live in the system zone. The evaluator rejects any operation Node whose target resolves to a system-zone label. System-zone Nodes are only writable through dedicated engine APIs (e.g., `engine.grantCapability()`, `engine.createView()`), never through operation subgraphs directly. This prevents privilege escalation via graph mutation.

**Transaction-capability interaction:** Capabilities are checked at transaction commit time, not at individual operation time. This ensures atomicity (the entire transaction reflects the capability state at commit) and avoids TOCTOU windows where a capability is revoked between two operations in the same transaction.

---

## 10. WASM Sandbox (SANDBOX Primitive)

For computation that can't be expressed as operation Nodes, the SANDBOX primitive calls into a WASM runtime (@sebastianwessel/quickjs v3.x):

- **No re-entrancy.** The sandbox receives data, returns data. Cannot call back into the graph.
- **Fuel-metered.** Every WASM instruction costs fuel. Execution terminates when fuel runs out.
- **Memory-limited.** Configurable max heap.
- **Time-limited.** Wall-clock timeout as backstop.
- **Capability-gated.** Which host functions exist is determined by the caller's capabilities.

**Host function surface:** The SANDBOX exposes a defined set of read-only host functions to sandboxed code. Sandboxed code cannot create, update, or delete Nodes/Edges directly -- it receives input data and returns output data. The evaluator applies the output as a WRITE (subject to capability checks) after SANDBOX execution completes. The complete host function manifest will be specified as part of Phase 2 implementation.

---

## 11. Rust Crate Structure

Revised from critic feedback (10 crates -> 4-5 for V1):

```
benten-engine/
|-- crates/
|   |-- benten-core/       # Node, Edge, Value types, content hashing, version chain primitives
|   |-- benten-graph/      # Graph storage, indexes (hash + B-tree), MVCC, persistence (redb), IVM
|   |-- benten-eval/       # Operation evaluator, 12 primitives, capability enforcement
|   +-- benten-engine/     # Orchestrator: public API, ties crates together
|-- bindings/
|   |-- napi/              # Node.js bindings (@benten/engine-native)
|   +-- wasm/              # WASM bindings (@benten/engine-wasm)
+-- tests/
```

Additional crates (added incrementally): `benten-sync` (CRDT merge, sync protocol), `benten-query` (Cypher/query parser, if needed beyond operation subgraphs).

**Persistence:** `benten-graph` uses redb for crash-safe persistence. redb provides serializable isolation with copy-on-write B-trees, WAL-equivalent crash recovery via two-phase commit with checksummed pages, and automatic garbage collection of old page versions. The engine relies on redb's durability guarantees rather than implementing a custom WAL.

**MVCC:** Snapshot isolation via redb's built-in transaction model. Read transactions see a consistent snapshot; write transactions are serialized. Graph-level version chains (Section 6) are an application-level pattern built on top of the storage layer -- from redb's perspective, creating a version Node is just a write like any other. MVCC operates on storage pages; version chains operate on graph Nodes. They do not interact directly.

---

## 12. Standards Adopted

| Standard | Purpose | Phase |
|----------|---------|-------|
| BLAKE3 | Hashing (content addressing, Merkle trees) | 1 |
| Ed25519 | Signatures (votes, attestations, UCAN, sync) | 1 |
| CBOR (RFC 8949) | Canonical serialization | 1 |
| Multihash | Algorithm-agile hash format | 1 |
| UCAN | Authorization tokens | 1 |
| did:key | Default identity | 1 |
| Hybrid Logical Clocks | Causal ordering for CRDT | 1 |
| libp2p | P2P networking (NAT traversal, DHT, GossipSub) | 2 |
| W3C Verifiable Credentials | Identity attestations | 2 |
| Merkle Search Trees | Efficient sync negotiation | 2 |

---

## 13. Migration from Thrum V3

### 13.1 What Carries Forward

- CMS domain code (content types, blocks, compositions, field types)
- SvelteKit web app (UI components, routes, pages)
- Module definitions (adapted to operation subgraphs)
- Test infrastructure patterns
- 3,200+ behavioral test expectations (the contracts, not the implementations)

### 13.2 What Gets Replaced

| Current (Thrum V3) | Replacement (Benten Engine) |
|--------------------|-----------------------------|
| In-memory registries | IVM materialized views |
| Event bus | Reactive subscriptions + EMIT primitive |
| PostgreSQL + AGE | Benten engine native storage |
| RestrictedEventBus + TIER_MATRIX | Capability enforcement |
| Trust tiers | Capability grants |
| compositions.previous_blocks | Version chains |
| content_revisions table | Version Nodes |
| module_settings | Settings as graph Nodes |

### 13.3 Migration Strategy

The engine exposes a TypeScript API (via napi-rs) that implements the existing Thrum Store interface. Existing modules can run unmodified against this adapter. Migration is then incremental -- each module is rewritten to use operation subgraphs at its own pace.

**Store interface compatibility:** The Thrum Store interface has 16 methods (graph CRUD, relational queries, file storage, transactions). The napi-rs adapter must handle:

| Store Method Category | Adapter Strategy |
|----------------------|------------------|
| Graph operations (createNode, getNode, createEdge, traverse) | Direct mapping to engine Node/Edge operations |
| Relational operations (createRecord, getRecord, queryRecords) | Nodes with typed labels, property indexes, query translation |
| File operations (storeFile, getFile, deleteFile) | Dedicated file storage layer (engine does not handle files natively; adapter delegates to filesystem or S3) |
| DDL (createTable) | Label-based Node collections with property schema validation |
| SecurityContext / TrustTier | Mapped to capability grants via preset configurations |

File storage and DDL are the least natural fits. Modules that depend heavily on relational table semantics may require adapter-specific accommodations.

---

## 14. Build Order

### Phase 1: Core Engine
- benten-core (Node, Edge, Value, content hashing, version chains)
- benten-graph (storage, indexes, MVCC, persistence via redb)
- napi-rs bindings (TypeScript can create/read/query Nodes)
- Benchmark suite (validate sub-100-microsecond handler execution for typical 10-node handlers; compare against PostgreSQL + AGE for Thrum's actual query patterns)
- Property-based testing infrastructure (MVCC correctness, version chain invariants, crash recovery)

### Phase 2: Evaluator + Capabilities
- benten-eval (12 primitives, evaluator, structural validation)
- Capability enforcement (UCAN grants as Nodes, checked on every WRITE)
- IVM (materialized views for capabilities, event handlers, content listings)
- SANDBOX integration (@sebastianwessel/quickjs)
- Prerequisite: paper-prototype 5 representative handlers as operation subgraphs before implementation. Measure what percentage of logic fits in the 12 primitives vs. requires SANDBOX. If >30% SANDBOX, revise the primitive vocabulary.

### Phase 3: Sync + Networking
- CRDT merge for version chains
- libp2p integration (peer discovery, GossipSub)
- Atrium sync (peer-to-peer)
- Sync protocol (delta exchange, Merkle comparison)
- See [Platform Design, Section 4](./PLATFORM-DESIGN.md) for the full networking specification

---

## 15. Open Questions

These are design decisions that must be resolved before Phase 2 implementation begins. They are not optional features -- they are load-bearing choices that affect the engine's architecture.

1. **Expression language for TRANSFORM:** What specific syntax? JavaScript subset? Custom DSL? How much of @benten/expressions carries forward? The expression language determines what TRANSFORM can compute without falling through to SANDBOX. Recommended approach: a restricted JavaScript subset (no closures, no `this`, no prototype access, no I/O, deterministic Math) compatible with the existing @benten/expressions evaluator (jsep + custom AST walker).

2. **Cypher support:** Do we need a Cypher parser, or are operation subgraphs sufficient for all queries? (Existing Rust Cypher parser: open-cypher crate.) If Cypher is needed, use an existing parser (Grafeo's or openCypher) rather than building from scratch.

3. **libp2p vs Yggdrasil:** libp2p is mature and production-proven. Yggdrasil is alpha but offers address=identity elegance. Support both behind a transport abstraction?

4. **MVCC garbage collection:** Snapshot isolation is the chosen model (Section 11). Garbage collection strategy for old redb snapshots needs specification. redb handles page-level GC automatically; the question is whether application-level GC (pruning old version chain nodes) needs additional coordination.

5. **IVM algorithm validation:** DBSP is the recommended approach (Section 8). Before implementation, prototype one materialized view using DBSP Z-set algebra in Rust and benchmark write latency. If DBSP proves too complex or too slow for Benten's patterns, fall back to hand-written incremental update logic for the 3-5 critical views, deferring general IVM to a later phase.

6. **Disk-backed graph:** redb B-tree backed, LRU cache for hot Nodes, or custom storage engine? The `PersistenceBackend` trait question: should benten-graph abstract over its storage backend to allow future replacement, or commit to redb? Recommended: commit to redb for V1 but define internal module boundaries that would allow future extraction.

7. **Binary size for WASM target:** PGlite is 860MB. Target for benten-engine-wasm? This constrains which dependencies can be included in the WASM build.

8. **GATE semantics:** GATE is described as a "custom logic escape hatch" but its execution model is undefined. What does GATE execute? Options: (a) a restricted expression (same as TRANSFORM but with I/O capabilities), (b) a CALL to another subgraph (overlaps with CALL primitive), (c) a WASM module (overlaps with SANDBOX). Recommended: GATE evaluates a condition expression and routes to success/failure edges. Complex validation uses VALIDATE. Complex computation uses SANDBOX. Capability checking is automatic (via `requires` property on any Node, enforced by the evaluator).

9. **Multi-node transaction semantics:** The 12 primitives include WRITE for single mutations but no TRANSACTION primitive. Real use cases (credit transfers, governance vote tallying, multi-party fee distribution) require multi-node atomicity. Recommended: a subgraph evaluation IS a transaction -- all WRITEs in a single subgraph evaluation are atomic. If any WRITE fails, all WRITEs in that evaluation are rolled back. This is feasible because subgraphs are bounded DAGs.
