# Benten Engine Technical Specification

**Created:** 2026-04-13
**Last Updated:** 2026-04-14 (post-critic revisions: primitives revised to new 12, 6-crate structure, performance honesty, capability hook extraction)
**Status:** WORKING DRAFT -- core architecture defined. Dependencies validated against 2026 Rust ecosystem. Primitives and crate structure revised after 8-critic review. Open questions flagged for resolution before/during Phase 1 implementation.
**Audience:** Rust engineers building the engine. TypeScript developers building bindings and modules.
**Related documents:** [Platform Design](./PLATFORM-DESIGN.md) (networking, governance, identity) | [Business Plan](./BUSINESS-PLAN.md) (economics, revenue, regulatory)

---

## 1. Context

Benten is a self-evaluating graph -- a platform where data and code are both Nodes and Edges, the graph evaluates itself, and every person, family, or organization can own their data, share it selectively, and fork at any time. This document specifies the Rust engine that makes this possible: the data model, the 12 operation primitives, the evaluator, version chains, incremental view maintenance, the capability system, the WASM sandbox, and the crate structure. For the networking, governance, and economic layers that sit above this engine, see the [Platform Design](./PLATFORM-DESIGN.md).

---

## 2. Architecture: A Self-Evaluating Graph

The Benten engine is a Rust-native system where code is represented AS graph structure, not stored IN graph properties. A route handler is not a string of source code -- it is a subgraph of operation Nodes connected by control-flow Edges. "Executing" a handler means the engine walks the subgraph, performing each operation.

```
[RouteHandler: GET /api/posts, requires=store:read:post/*]
    |--[FIRST_STEP]--> [READ: query posts where published=true]
    |                      +--[NEXT]--> [TRANSFORM: to JSON]
    |                                      +--[NEXT]--> [RESPOND: 200]
    +--[ON_DENIED]--> [RESPOND: 403]
```

(The `requires` property on any Node is checked automatically by the evaluator; capability denial routes to `ON_DENIED` edges. No separate GATE primitive is needed — that was in the original 12 and was dropped during the 2026-04-14 revision.)

**Why code-as-graph:**
- **Inspectable.** An AI agent can read any handler by traversing its subgraph.
- **Modifiable at runtime.** Insert a caching step without recompilation.
- **Versionable.** The handler is a subgraph with a version chain. Roll back by moving the CURRENT pointer.
- **Syncable.** Installing a module = syncing its operation subgraphs. Code travels as data.
- **Statically analyzable.** The engine can pre-compute cost, check capabilities, and verify determinism BEFORE execution -- because subgraphs are DAGs.

---

## 3. The 12 Operation Primitives (revised 2026-04-14)

Every computation in Benten is composed from 12 primitive operation Node types. The vocabulary is deliberately NOT Turing complete -- subgraphs are DAGs with bounded iteration. This guarantees termination, enables static analysis, and prevents denial-of-service.

The set was revised on 2026-04-14 after critic review. See "Revision history" at the end of this section for what changed and why.

| # | Primitive | Purpose | Key Property |
|---|-----------|---------|-------------|
| 1 | **READ** | Retrieve from graph (by ID, query, materialized view) | Typed error edges: ON_NOT_FOUND, ON_EMPTY |
| 2 | **WRITE** | Mutate graph (create, update, delete, conditional CAS) | Auto version-stamp (if versioning enabled). Typed error: ON_CONFLICT, ON_DENIED |
| 3 | **TRANSFORM** | Pure data reshaping | Sandboxed expression with arithmetic, array built-ins (filter, map, sum, etc.), object construction. No I/O. |
| 4 | **BRANCH** | Conditional routing | Forward-only, no cycles |
| 5 | **ITERATE** | Bounded collection processing | Mandatory maxIterations. Optional parallel. Multiplicative cost through nesting. |
| 6 | **WAIT** | Suspend until signal/timeout | For workflows, approval patterns. Decomposes to data Nodes for cross-instance. |
| 7 | **CALL** | Execute another subgraph | `isolated` defaults to `true` for capability attenuation. Mandatory timeout. |
| 8 | **RESPOND** | Terminal: single output | HTTP response, event result, agent action result |
| 9 | **EMIT** | Fire-and-forget notification | `deliveryMode` as subscriber-side strategy, not primitive flag. `audience` restriction for exfiltration prevention. |
| 10 | **SANDBOX** | WASM computation escape hatch | No re-entrancy. Fuel-metered per-subgraph (not per-call). Time-limited. Max output 1MB. Uses `wasmtime`. |
| 11 | **SUBSCRIBE** | Reactive change notification | Base primitive that IVM, sync, and EMIT delivery compose on. Added in 2026-04-14 revision. |
| 12 | **STREAM** | Partial/ongoing output with back-pressure | SSE, WebSocket messages, LLM token streams, progress updates. Added in 2026-04-14 revision. |

**Composed from the 12 (NOT primitives):**
- Retry (ITERATE + BRANCH + CALL)
- Parallel (ITERATE with parallel flag)
- Compensate (BRANCH on error + CALL undo subgraph)
- DryRun (evaluation mode property)
- Audit (EMIT to audit channel)
- Map/Filter/Reduce (ITERATE + TRANSFORM)
- **Validate (BRANCH on schema predicate + RESPOND with error)** -- was a primitive, now a stdlib pattern
- **Capability check on any Node (`requires` property + automatic BRANCH on failure)** -- was GATE, now a property the evaluator honors
- **Event handler (SUBSCRIBE + trigger subgraph registration)** -- was implicit inside IVM, now explicit composition
- **IVM materialized view (SUBSCRIBE + TRANSFORM + WRITE to view Node)** -- was engine-internal, now a composable pattern `benten-ivm` implements using SUBSCRIBE

### Revision history

**2026-04-14 revision (after 8-critic review):**

*Dropped:*
- **VALIDATE** -- redundant with (BRANCH + TRANSFORM + RESPOND on error) plus the 14 structural invariants enforced at registration time. Provided as stdlib pattern, not primitive.
- **GATE** -- "custom logic escape hatch" semantics were undefined (Open Question 8 in earlier draft). Capability checking uses the `requires` property on any Node (engine-enforced automatic BRANCH). Custom validation uses TRANSFORM or SANDBOX. No residual need for GATE.

*Added:*
- **SUBSCRIBE** -- identified by engine-philosophy critic as the primitive that IVM, sync, and reactive event dispatch all build on. Making it explicit means IVM becomes a composable module (`benten-ivm`) rather than engine-internal. Keeps the engine thinner.
- **STREAM** -- identified by composability critic as missing for SSE, WebSocket, LLM token streams, and large JSON responses. WinterTC targets make streaming table stakes for 2026 web APIs. Cannot be cleanly composed from RESPOND (terminal) + ITERATE (no back-pressure).

*Unchanged behavior but revised framing:*
- **EMIT `deliveryMode`** is now a subscriber-side strategy, not a primitive flag. Enforced via SUBSCRIBE configuration.
- **CALL `isolated`** defaults to `true` (previously ambiguous).

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

**Canonical serialization:** DAG-CBOR via the `serde_ipld_dagcbor` crate. This produces deterministic encoding by default (map keys sorted during serialization, no caller-side canonicalization needed). DAG-CBOR is a strict subset of CBOR used by the IPLD ecosystem; its sort order (RFC 7049 length-first) is equivalent to RFC 8949 bytewise sort for string-keyed maps, which is Benten's case. This replaces the initial choice of `ciborium`, which does not sort map keys and would require a custom canonicalization wrapper.

**CIDv1 format:** Benten content hashes adopt the IPLD CIDv1 format (version byte + multicodec + multihash). Every Benten content hash is a valid CIDv1 at a cost of 2 extra bytes. This gives us tooling interop with the entire IPLD ecosystem, AT Protocol compatibility, and a well-understood standard instead of a custom format. The multicodec is 0x71 (dag-cbor); the multihash uses BLAKE3 (code 0x1e).

**Dependency note (2026-04-14):** The `cid` crate (v0.11.1) and the `multihash` crate (v0.19.3) both transitively depend on `core2`, which was archived upstream on 2026-04-14 and whose sole published 0.4.0 version is yanked from crates.io. Until upstream releases a `core2`-free version, BentenAI maintains a minimal fork of `rust-cid` at [`BentenAI/rust-cid`](https://github.com/BentenAI/rust-cid) that replaces `core2` with `no_std_io2` (an API-compatible drop-in), pinned to commit `e11cf45399c951597725a9bc3ed49c805f7aa640`. An upstream PR is open at [multiformats/rust-cid#185](https://github.com/multiformats/rust-cid/pull/185); a matching PR for the sibling crate is tracked at [multiformats/rust-multihash#407](https://github.com/multiformats/rust-multihash/pull/407). Workspace `[patch.crates-io]` will be reverted to crates.io once both merge and release. See `SPIKE-phase-1-stack-RESULTS.md` Surprises #1 and Next Actions #1.

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

For computation that can't be expressed as operation Nodes, the SANDBOX primitive calls into `wasmtime` (v35+, Rust-native, Bytecode Alliance). The initial spec referenced `@sebastianwessel/quickjs`; this was a JavaScript library and inappropriate for a Rust engine. `wasmtime` provides fuel-based execution metering that maps directly to SANDBOX's fuel budget (guaranteed termination via fuel exhaustion), AArch64 support, and Component Model support for composability.

- **No re-entrancy.** The sandbox receives data, returns data. Cannot call back into the graph.
- **Fuel-metered.** Every WASM instruction costs fuel. Execution terminates when fuel runs out.
- **Memory-limited.** Configurable max heap.
- **Time-limited.** Wall-clock timeout as backstop.
- **Capability-gated.** Which host functions exist is determined by the caller's capabilities.

**Host function surface:** The SANDBOX exposes a defined set of read-only host functions to sandboxed code. Sandboxed code cannot create, update, or delete Nodes/Edges directly -- it receives input data and returns output data. The evaluator applies the output as a WRITE (subject to capability checks) after SANDBOX execution completes. The complete host function manifest will be specified as part of Phase 2 implementation.

---

## 11. Rust Crate Structure (revised 2026-04-14)

Revised after critic review from 4 crates to **6**. The architecture-purity and engine-philosophy critics independently identified that IVM and capability enforcement were inside `benten-graph`/`benten-eval` in ways that made the engine thicker than its "thin engine" tagline promised. Extracting `benten-ivm` (so the evaluator doesn't know IVM exists) and `benten-caps` (so capability policy is pluggable) brings the total to 6 crates. Version chains stayed in `benten-core` as an opt-in convention rather than a separate crate.

```
benten-engine/
|-- crates/
|   |-- benten-core/       # Node, Edge, Value types; content hashing (BLAKE3 + DAG-CBOR + CIDv1);
|   |                      # version chain primitives (Anchor + Version Node + CURRENT Edge pattern)
|   |-- benten-graph/      # Graph storage via KVBackend trait (redb impl); indexes (hash + B-tree);
|   |                      # MVCC via redb transactions; change notification stream
|   |-- benten-ivm/        # Incremental View Maintenance; subscribes to graph change stream;
|   |                      # per-view strategy selection (Algorithm B default, A / C optional)
|   |-- benten-caps/       # Capability types + pre-write hook trait + `NoAuthBackend` default.
|   |                      # UCAN implementation as one pluggable backend (in `benten-id`, Phase 3)
|   |-- benten-eval/       # 12 operation primitives; iterative evaluator; structural validation (14 invariants);
|   |                      # transaction primitive (begin/commit/rollback); wasmtime SANDBOX host
|   +-- benten-engine/     # Orchestrator: composes the crates above into the public API;
|                           # wires capability backend, storage backend, IVM subscriber
|-- bindings/
|   +-- napi/              # Node.js bindings via napi-rs v3; same codebase compiles to WASM target
+-- tests/                 # Cross-crate integration tests and benchmarks
```

**Phase 3+ additional crates:** `benten-sync` (CRDT merge, Merkle Search Tree diff, sync protocol over iroh), `benten-id` (Ed25519, UCAN as capability backend, DID/VC support), optional `benten-query` (Cypher parser if demand emerges beyond operation subgraphs).

**The thinness test:** A developer should be able to use `benten-core` + `benten-graph` + `benten-engine` with `NoAuthBackend`, version chains disabled, no IVM subscribers, and get a pure content-addressed graph database with no Benten-specific conventions. If that configuration requires anything from `benten-eval`, `benten-ivm`, or `benten-caps`, the engine is too thick.

**Persistence:** `benten-graph` uses `redb` v4 (validated April 2026, actively maintained, production-ready). redb provides serializable isolation with copy-on-write B-trees, WAL-equivalent crash recovery via two-phase commit with checksummed pages, MVCC (concurrent readers with single writer), multiple named tables, range queries, and automatic garbage collection of old page versions. The engine relies on redb's durability guarantees rather than implementing a custom WAL.

**Storage abstraction:** `benten-graph` exposes a `GraphBackend` trait. The native implementation uses redb. A future WASM implementation will fetch content-addressed data from the peer network (via iroh or HTTP) with an in-memory cache. This abstraction is defined in Phase 1 but only redb is implemented. The trait boundary is critical: it preserves the option to run the engine inside a browser, an edge function, or an encrypted peer-distributed model without requiring changes to the evaluator or the application layer. See `docs/research/explore-distributed-compute-vision.md` for the motivation.

**MVCC:** Snapshot isolation via redb's built-in transaction model. Read transactions see a consistent snapshot; write transactions are serialized. Graph-level version chains (Section 6) are an application-level pattern built on top of the storage layer -- from redb's perspective, creating a version Node is just a write like any other. MVCC operates on storage pages; version chains operate on graph Nodes. They do not interact directly.

---

## 12. Standards Adopted

| Standard | Purpose | Phase |
|----------|---------|-------|
| BLAKE3 | Hashing (content addressing, Merkle trees) | 1 |
| Ed25519 | Signatures (votes, attestations, UCAN, sync) | 1 |
| DAG-CBOR (IPLD, equivalent to deterministic RFC 8949 for our use) | Canonical serialization | 1 |
| Multihash | Algorithm-agile hash format | 1 |
| CIDv1 (IPLD) | Content identifier format | 1 |
| UCAN | Authorization tokens | 1 |
| did:key | Default identity | 1 |
| Hybrid Logical Clocks | Causal ordering for CRDT | 1 |
| WinterTC (Ecma TC55) | Edge runtime API surface | 2+ |
| iroh | P2P networking (QUIC, holepunch, relay) | 2 |
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

### Phase 1: Core Engine ("tight middle" scope, reconciled 2026-04-14)

Scope reconciled with CLAUDE.md "Phase 1 Scope" and `FULL-ROADMAP.md` Phase 1 to a single coherent shape. Phase 1 ships everything Phase 1's exit criteria (`crud('post')` + audit-trail viz) actually exercises, plus the primitives and invariants needed to prove the architectural thesis. WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX and the remaining invariants ship in Phase 2 alongside evaluator completion.

- **benten-core**: Node, Edge, Value, content hashing (BLAKE3 + DAG-CBOR + CIDv1), version chain primitives (opt-in)
- **benten-graph**: storage via `KVBackend` trait (redb v4 impl), indexes, MVCC via redb snapshot isolation, change-notification stream that IVM subscribes to
- **benten-ivm**: 5 hand-written IVM views from the prototype benchmark (capability grants, event handler dispatch, content listing, governance inheritance, version-chain CURRENT). Subscribes to the graph change stream. Evaluator-agnostic. Generalized Algorithm B ships Phase 2.
- **benten-caps**: `CapabilityPolicy` pre-write hook trait, `NoAuthBackend` default, UCAN backend stub
- **benten-eval**: all 12 primitive *types* defined; iterative evaluator executes **8 primitives** (READ, WRITE, TRANSFORM, RESPOND, BRANCH, ITERATE, CALL, EMIT); registration-time structural validation for **invariants 1-6, 9-10, 12**; transaction primitive (begin/commit/rollback); TRANSFORM expression evaluator (arithmetic, built-ins, object construction)
- **benten-engine**: public API composing the 5 crates above; wires capability backend, storage backend, IVM subscriber
- napi-rs v3 bindings (TypeScript creates/reads/updates/deletes Nodes + Edges, reads IVM views, registers and evaluates 8-primitive operation subgraphs); WASM runtime is Phase 2
- Storage abstraction trait (redb for native; network-fetch stub defined but implementation deferred to Phase 2)
- Benchmark suite via `criterion 0.8` (validate honest §14.6 targets; compare against PostgreSQL + AGE for CRUD hot paths)
- Property-based testing via `proptest` (Node CID round-trip, MVCC correctness, version chain invariants)
- `cargo-nextest` as default test runner
- Developer tooling: `create-benten-app` scaffolder, `subgraph.toMermaid()`, `engine.trace()`, error-catalog integration

### Phase 2: Evaluator Completion + WASM + SANDBOX
- **4 remaining primitives executed**: WAIT (suspend/resume with serializable execution state), STREAM (chunked output with back-pressure, SSE/WebSocket), SUBSCRIBE (reactive change notification as a user-visible operation), SANDBOX (wasmtime-hosted fuel-metered computation)
- **6 remaining invariants enforced**: 4 (SANDBOX nesting), 7 (SANDBOX output ≤1MB), 8 (cumulative iteration budget multiplicative), 11 (system-zone labels unreachable), 13 (immutability enforcement, TOCTOU protection), 14 (causal attribution unsuppressible)
- wasmtime SANDBOX host (Rust-native, fuel metering) with instance pool and host-function manifest
- Capability enforcement hardened across all 12 primitives (Phase 1 covered the 8-primitive subset)
- Generalized IVM Algorithm B + per-view strategy selection (A/B/C per view based on access pattern)
- WASM build target via napi-rs v3 with network-fetch `KVBackend` backend
- Transaction-primitive API shape finalized (closure-based vs. `WriteBatch`) based on Phase 1 usage feedback
- Module manifest format (requires-caps, provides-subgraphs, migrations)
- Paper-prototype re-validation: confirm <30% SANDBOX rate against the revised 12-primitive vocabulary (original 12 saw 2.5%; re-measure against the 2026-04-14 set with SUBSCRIBE + STREAM added, VALIDATE + GATE removed)

### Phase 3: Sync + Networking
- CRDT merge for version chains
- libp2p integration (peer discovery, GossipSub)
- Atrium sync (peer-to-peer)
- Sync protocol (delta exchange, Merkle comparison)
- See [Platform Design, Section 4](./PLATFORM-DESIGN.md) for the full networking specification

---

## 14.5. Validated Dependencies (2026-04-14)

All Phase 1 dependencies were validated against the 2026 Rust ecosystem during pre-work Step 2. Five spec changes emerged:

| Concern | Original Choice | Validated Choice | Reason |
|---------|-----------------|------------------|--------|
| CBOR serialization | ciborium | **serde_ipld_dagcbor** | ciborium does not sort map keys; serde_ipld_dagcbor is deterministic by default and IPLD-native |
| Benchmarking | divan | **criterion 0.8** | criterion was revived with a new maintainer; divan is pre-1.0 and stalled |
| SANDBOX runtime | @sebastianwessel/quickjs | **wasmtime** | Rust-native, fuel metering built in, appropriate for a Rust engine |
| WASM bindings | wasm-bindgen (separate from napi-rs) | **napi-rs v3** handles both | v3 compiles to WASM target from the same codebase |
| Test runner | cargo test | **cargo-nextest** (with cargo test fallback) | 3x faster, per-test isolation, widely adopted in 2026 |

Confirmed stack with versions (April 2026):

| Crate | Version | Status |
|-------|---------|--------|
| blake3 | 1.8.4 | Actively maintained |
| serde_ipld_dagcbor | 0.6.1 | Actively maintained, IPLD-native |
| multihash | 0.19.3 | Stable, use with custom codetable for BLAKE3 only |
| redb | 4.0.1 | Production-ready, actively maintained |
| napi | 3.8.4 | Active (v3 launched July 2025) |
| papaya | 0.2.4 | Active, lock-free, read-optimized |
| mimalloc | 0.1.48 | Stable |
| thiserror | 2.0.18 | Active, v2 stable |
| tracing | 0.1.44 | Active (0.1.x still standard) |
| proptest | 1.11.0 | Active |
| criterion | 0.8.2 | Active (revived) |
| wasmtime | 35+ | Production, fuel metering |
| cargo-nextest | current | Adopted as default test runner |

Rust toolchain: 1.94.1 (2024 edition stable since 1.85.0, February 2025).

---

## 14.6. Performance Targets and Honest Caveats (revised 2026-04-14)

The performance critic identified that several targets were aspirational and the spec's "Rust will be 10-100x faster" language was misleading for algorithmic costs (not all speedups are language-based).

**Revised targets with honest caveats:**

| Target | Realistic Range | Caveat |
|--------|----------------|--------|
| Node lookup by ID | 1-50us | <0.01ms (10us) requires hot cache; cold redb lookup is 5-50us due to DAG-CBOR deserialization + multihash decode + capability check |
| IVM view read (clean) | 0.04us-1us | Achievable for HashMap/sorted-list strategies; measured in TypeScript prototype |
| Node creation + IVM update | 100-500us realistic, 0.1ms aspirational | fsync to disk is 0.1-10ms; spec must define durability policy per write class (group commit for bulk, immediate for capability grants) |
| 10-node handler evaluation | 150-300us for mixed handlers | <100us achievable for pure TRANSFORM pipelines; handlers with 2+ WRITEs + IVM miss the target by 2-3x |
| Concurrent writers (per community/instance) | 100-1000 writes/sec | redb single-writer serialization is a hard ceiling. Multi-tenant shared-replica scenarios hit this. Benchmark explicitly during spike. |
| SANDBOX instantiation | 100us-1ms per call | wasmtime module instantiate + fuel meter setup dominates. Requires instance pool in Phase 2 to avoid 5-50x target miss on SANDBOX-heavy handlers. |

**What Rust DOES fix vs what it does NOT:**

- **Rust fixes:** V8 sort overhead (Content Listing Algorithm C), GC pauses, Z-set creation costs, TypeScript iteration loops. Real gains of 10-100x on these patterns.
- **Rust does NOT fix:** the Capability Check delete pattern showing 2.3s p50 in the prototype. That's an **algorithmic** O(E×G) cost of rebuilding from entities × grants. Rust makes the rebuild ~20-50x faster (50-100ms), still unacceptable. The real fix is the Hybrid A+B algorithm (HashMap for O(1) reads + targeted invalidation for edges + eager rebuild only on rare node deletions), as identified in `research/ivm-benchmark/RESULTS.md`. The algorithm is the fix, not the language.

**WASM target constraints acknowledged:**

- BLAKE3 has no SIMD on WASM until SIMD128 fully lands -- 3-10x slower than native
- redb does NOT work inside WASM (no filesystem in WASI Preview 2). WASM builds use a network-fetch `KVBackend` backend with 50-500ms uncached read latency
- **SANDBOX does not nest in WASM builds.** wasmtime cannot embed in a WASM host; the browser/edge WASM runtime IS the SANDBOX. WASM builds expose SANDBOX as the host's WASM facility, not as a nested wasmtime.
- Target binary size 2-5MB achievable with wasmtime excluded from the WASM build

**Scalability ceilings (document honestly, don't claim unlimited scale):**

- **~1000-peer community** without hierarchical relays. Beyond that, GossipSub message amplification creates 60k+ message deliveries per write at 10k peers. AT Protocol works at scale because of centralized relays; a pure P2P mesh does not.
- **~1000 writes/sec per community** against a single redb replica. Sharding via community partitioning or multi-replica writers is a Phase 3+ design problem.

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

8. **GATE semantics:** **RESOLVED 2026-04-14.** GATE has been removed from the primitive set. Capability checking uses the `requires` property on any Node (engine-enforced). Complex validation uses TRANSFORM or composition of BRANCH + TRANSFORM + RESPOND. Complex computation uses SANDBOX. No residual need for a GATE primitive.

9. **Multi-node transaction semantics:** The 12 primitives include WRITE for single mutations but no TRANSACTION primitive. Real use cases (credit transfers, governance vote tallying, multi-party fee distribution) require multi-node atomicity. Recommended: a subgraph evaluation IS a transaction -- all WRITEs in a single subgraph evaluation are atomic. If any WRITE fails, all WRITEs in that evaluation are rolled back. This is feasible because subgraphs are bounded DAGs.
