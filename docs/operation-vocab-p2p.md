# Operation Node Vocabulary: P2P & Sync Perspective

**Created:** 2026-04-11
**Context:** Designing the operation Node vocabulary for a graph execution engine where code IS Nodes and Edges, and the application graph syncs between instances. When a module is installed, its handler logic arrives as operation subgraphs via sync.

**Dependencies:** `SPECIFICATION.md` (Sections 2.1-2.5), `critique-p2p.md`, `critique-holochain-perspective.md`, `critique-crdt-graph.md`, `critique-security.md`, `critique-mesh-sync.md`

---

## 1. The Core Insight: Code-as-Data Has Radical Consequences for P2P

In a traditional CMS or application platform, "installing a module" means downloading code from a registry, running it in a trusted environment, and hoping it does what it claims. The code is opaque. You can sandbox it (WASM, QuickJS, V8 isolates), but you cannot inspect its intent without executing it.

In the Benten model, a module's handler logic is encoded as a subgraph of operation Nodes connected by dataflow Edges. This means:

1. **Installing a module is syncing a subgraph.** The same CRDT merge that handles user data also handles handler definitions. There is no separate "install" step.

2. **Handler logic is inspectable before execution.** Because the operation graph IS the handler, you can traverse it to determine: what it reads, what it writes, what capabilities it requires, and what operations it performs. This is static analysis by graph traversal.

3. **Handler logic is deterministic by construction (if designed correctly).** Because each operation Node has a fixed type with defined semantics, and the dataflow edges define a DAG of computation, the engine can guarantee that the same inputs produce the same outputs -- without needing a general-purpose sandbox.

4. **Handler versions are first-class.** Because handlers are Nodes with version chains, updating a handler is a graph mutation. The old version is preserved. Migration between versions can be expressed as graph transformations.

These properties are individually found in other systems. Ethereum has deterministic execution. Holochain has validation-before-execution. Unison has content-addressed code. Node-RED has visual dataflow graphs. No existing system combines all four in a graph-native way.

---

## 2. The Threat Model: Operation Subgraphs as Untrusted Input

When an operation subgraph arrives via sync from a remote peer, it is adversarial input. The receiving instance must assume:

**A malicious peer could send operation Nodes that:**
- Claim to be deterministic but contain hidden non-determinism (e.g., a "ConditionOp" that behaves differently based on wall-clock time)
- Escalate privileges (e.g., a "QueryOp" that reads the capability graph)
- Consume unbounded resources (e.g., a "LoopOp" that iterates forever)
- Corrupt data (e.g., a "MutateOp" that writes to system Nodes)
- Exfiltrate data (e.g., an "ExternalOp" that sends data to an attacker's endpoint)
- Create circular dataflow (e.g., operation A feeds into B feeds into A)

**Defenses must be structural, not behavioral.** We cannot rely on "well-behaved modules won't do X" because modules arrive from untrusted peers. Every defense must be enforced by the engine's execution model.

### 2.1 Lessons from Existing Systems

**Ethereum/EVM:** Determinism is guaranteed by a restricted instruction set. There is no opcode for "get current time" (only block timestamp, which is consensus-agreed), no opcode for "read file," no opcode for "make HTTP request." Non-determinism is eliminated by not providing the tools for it. Resource exhaustion is prevented by the gas mechanism: every opcode has a cost, and execution halts when gas runs out.

**Holochain:** Validation is separate from execution. Each DNA defines `validate_*` callbacks that run BEFORE data is committed. These callbacks are deterministic (they cannot call non-deterministic host functions). If validation fails, the data is rejected. The author's source chain is checked for tampering. The validation DNA is the same WASM binary on all peers, so all peers agree on validity.

**CosmWasm:** Each contract runs in its own WASM sandbox with an explicit capability model. The contract declares what it needs (storage, querying other contracts, sending messages) via `Deps` and `Env` parameters. The runtime provides exactly those capabilities and nothing else. Storage is per-contract, key-prefixed.

**Unison:** Code is content-addressed by the hash of its AST. When distributing code, the sender ships the AST tree; the receiver checks which hashes it already has and requests only the missing ones. This means code identity is independent of naming. Two modules that define the same function produce the same hash regardless of what they call it.

---

## 3. Determinism Classification of Operation Types

Every operation type in the vocabulary must be classified along two axes:

### 3.1 Determinism Axis

| Classification | Definition | Example Operations |
|---|---|---|
| **Pure** | Same inputs always produce same outputs. No side effects. No external dependencies. | `TransformOp`, `FilterOp`, `MapOp`, `ConditionOp` (when condition is data-only), `ValidateOp`, `MergeOp`, `CompareOp`, `ArithmeticOp`, `StringOp`, `SchemaCheckOp` |
| **Read-deterministic** | Output depends on graph state. Deterministic if the graph state is the same. Two instances with the same synced subgraph produce the same output. | `QueryOp`, `TraverseOp`, `LookupOp`, `ResolveRefOp`, `AggregateOp`, `ViewReadOp` |
| **Write-deterministic** | Produces a deterministic mutation. The mutation itself is deterministic, but the resulting graph state depends on the order of application relative to other writes. | `CreateNodeOp`, `UpdateNodeOp`, `DeleteNodeOp`, `CreateEdgeOp`, `DeleteEdgeOp`, `SetPropertyOp` |
| **Non-deterministic** | Output depends on external state, randomness, or wall-clock time. Two instances cannot guarantee the same output. | `ExternalCallOp` (HTTP), `RandomOp`, `TimestampOp`, `UuidGenerateOp`, `WasmCallOp` (if WASM is non-deterministic) |

### 3.2 Sync Safety Axis

| Classification | Can sync the operation subgraph? | Can sync the results? |
|---|---|---|
| **Sync-safe** | Yes. Any instance can execute it and get the same result. | Yes. Results are verifiable. |
| **Sync-with-context** | Yes, but result depends on local graph state. Instance must have the same data to reproduce. | Yes, if the dependent data is also synced. |
| **Sync-result-only** | No. The operation itself cannot be meaningfully executed on another instance. | Yes. The result (the data mutation) can be synced as a fait accompli. |
| **Sync-forbidden** | No. | No. The operation has effects that only make sense locally. |

### 3.3 Combined Classification Matrix

| Operation Type | Determinism | Sync Safety | Rationale |
|---|---|---|---|
| TransformOp | Pure | Sync-safe | Pure data transformation. Any instance can reproduce. |
| FilterOp | Pure | Sync-safe | Pure predicate evaluation. |
| ConditionOp | Pure | Sync-safe | Branch selection based on data values. |
| ValidateOp | Pure | Sync-safe | Schema/constraint checking. |
| MergeOp | Pure | Sync-safe | Combining multiple values. |
| QueryOp | Read-deterministic | Sync-with-context | Depends on graph state. Deterministic if state matches. |
| TraverseOp | Read-deterministic | Sync-with-context | Graph traversal depends on current edges. |
| LookupOp | Read-deterministic | Sync-with-context | Key-value lookup in graph. |
| AggregateOp | Read-deterministic | Sync-with-context | count/sum/avg over graph data. |
| ViewReadOp | Read-deterministic | Sync-with-context | IVM view read -- O(1) but state-dependent. |
| CreateNodeOp | Write-deterministic | Sync-result-only | Creates data. Result can sync; re-execution would duplicate. |
| UpdateNodeOp | Write-deterministic | Sync-result-only | Modifies data. Result syncs via version chain. |
| DeleteNodeOp | Write-deterministic | Sync-result-only | Removes data. Result syncs via tombstone/soft-delete. |
| CreateEdgeOp | Write-deterministic | Sync-result-only | Creates relationship. Result syncs via add-wins CRDT. |
| DeleteEdgeOp | Write-deterministic | Sync-result-only | Removes relationship. Result syncs via tombstone. |
| EmitEventOp | Write-deterministic | Sync-result-only | Fires a reactive notification. Local-only effect but result-data syncs. |
| ExternalCallOp | Non-deterministic | Sync-forbidden | HTTP calls, external APIs. Cannot reproduce across instances. |
| TimestampOp | Non-deterministic | Sync-forbidden | Wall-clock time. Use HLC for sync-safe timestamps. |
| RandomOp | Non-deterministic | Sync-forbidden | Random values. Cannot reproduce. |
| LogOp | Non-deterministic | Sync-forbidden | Logging. Instance-local side effect. |

### 3.4 The Rule for Synced Contexts

**A synced operation subgraph MUST NOT contain non-deterministic operations.**

If an operation subgraph contains any `Sync-forbidden` operation, the receiving instance MUST reject it during validation (before execution). The subgraph is structurally invalid for sync.

This is enforced by static analysis: the engine walks the operation subgraph's Nodes and checks their types against the classification table. No execution needed.

**What about handlers that legitimately need non-determinism?** (e.g., a handler that calls an external API to enrich content)

Answer: The handler's *definition* (the operation subgraph) syncs. The handler's *execution* does not. When Instance A executes a handler that calls an external API, the result (the data mutation) syncs to Instance B via the normal CRDT mechanism. Instance B receives the enriched data, not the instruction to call the API.

This is the same as Holochain's model: validation rules sync (they are deterministic). The data produced by running those rules locally does not need to re-execute on the peer -- it syncs as validated data.

---

## 4. Static Analysis via Graph Traversal

The homoiconic property (code IS Nodes and Edges) enables a powerful capability: **you can analyze what a handler does without executing it.**

### 4.1 Capability Extraction

Given an operation subgraph, the engine can determine its required capabilities by traversing the graph:

```
Handler Subgraph Analysis:
  1. Find all QueryOp Nodes -> extract target labels/types -> READ capabilities needed
  2. Find all CreateNodeOp/UpdateNodeOp Nodes -> extract target labels -> WRITE capabilities needed
  3. Find all TraverseOp Nodes -> extract edge types -> TRAVERSE capabilities needed
  4. Find all ExternalCallOp Nodes -> extract endpoints -> EXTERNAL capabilities needed
  5. Find all ViewReadOp Nodes -> extract view names -> VIEW capabilities needed
```

This produces a **capability manifest** -- a complete list of what the handler requires to execute. The engine can:

1. **Pre-validate:** Before executing, check whether the handler's capability manifest is satisfied by the grants available to the module that owns it.
2. **Auto-grant:** When installing a module, present the user with a summary: "This module reads Posts, writes Analytics, and calls api.example.com. Allow?"
3. **Detect escalation:** If a synced subgraph's capability manifest exceeds the sync agreement's scope, reject it.

### 4.2 Dataflow Analysis

The edges between operation Nodes define a dataflow graph (a DAG, if well-formed). The engine can analyze this DAG to determine:

**Input/Output types:** By following the dataflow edges from the handler's entry point to its exit point, the engine can determine what data the handler consumes and produces. This enables type-checking at the graph level.

**Reachability:** If operation A feeds data to operation B, and operation B writes to content type C, then the handler has a transitive write dependency on C. This is computed via graph reachability from the handler's root to all write operations.

**Side-effect freedom:** A handler that contains only Pure and Read-deterministic operations is side-effect-free. It can be executed speculatively, cached, or re-executed without consequence.

**Cycle detection:** The existing DAG cycle detection in `@benten/engine/dag` applies directly. An operation subgraph with cycles is structurally invalid (infinite loop potential). Reject before execution.

### 4.3 The Sandbox IS the Graph

This is the key insight that distinguishes Benten from WASM-sandboxed systems:

In a WASM sandbox, the host provides a set of imports (functions the WASM module can call). The module can call any provided import in any order. The sandbox prevents access to anything not imported, but it cannot prevent the module from doing arbitrary computation with the imported capabilities.

In a graph-based operation model, the handler IS its computation graph. You do not provide capabilities and hope the handler uses them responsibly. You inspect the handler's graph and verify that every operation is within the allowed set. The handler cannot "decide at runtime" to access something it does not have an operation Node for. The graph structure IS the sandbox.

**Comparison:**

| Property | WASM Sandbox | Graph Sandbox |
|---|---|---|
| Inspection before execution | No (opaque bytecode) | Yes (traverse Nodes and Edges) |
| Capability enforcement | Runtime (trap on unauthorized import call) | Pre-execution (reject invalid subgraph) |
| Composition visibility | Black box | Full: you can see how capabilities are composed |
| Non-determinism detection | Impossible without execution | Graph traversal (check for non-deterministic Node types) |
| Resource bounding | Gas metering (runtime) | Subgraph size limit + operation count limit (structural) |
| Turing completeness | Yes (within gas limits) | No (DAG, no loops -- by design) |

The trade-off: graph-based handlers are NOT Turing-complete. They cannot express arbitrary loops. This is a feature, not a limitation. For the same reason that SQL is not Turing-complete (to guarantee query termination), operation DAGs are not Turing-complete (to guarantee handler termination).

**Escape hatch:** For handlers that genuinely need general-purpose computation (e.g., image processing, complex algorithms), the `WasmCallOp` operation delegates to a WASM sandbox. This is the "unsafe" block -- it breaks the static-analysis guarantee and requires runtime sandboxing. The engine treats WasmCallOp as a non-deterministic, sync-forbidden operation.

---

## 5. Versioning: Handler Version Coexistence

### 5.1 The Problem

Instance A has handler H at version 1. Instance B has handler H at version 2. Both instances process data using their local handler version. When they sync, the data is consistent (CRDT merge), but it was produced by different handler versions. Is this a problem?

### 5.2 Analysis

**It depends on what the handler does.**

**Case 1: Read-only handlers (validators, queries, views).** If handler H is a validator that checks whether a post has a title, v1 might allow empty titles and v2 might reject them. Data created on Instance A (under v1's lenient rules) syncs to Instance B (running v2's strict rules). Instance B receives a post with an empty title that its local validator would reject.

This is the Holochain DNA mismatch problem. Holochain's answer: **all peers in a network MUST run the same DNA version.** Version mismatch = different network. You cannot sync across DNA versions.

**Case 2: Transform handlers (enrichment, denormalization).** If handler H transforms content on save (e.g., v1 generates a slug from the title, v2 generates a slug from the title + date), data produced by v1 and v2 has different slugs for the same content. After sync, both instances have the same data (CRDT merge resolves the slug field via LWW), but the "winning" slug depends on HLC timestamps, not handler version.

**Case 3: Write handlers (automation, workflows).** If handler H triggers on `content:afterCreate` and creates related Nodes, v1 might create a `SeoScore` Node and v2 might create both `SeoScore` and `ReadabilityScore` Nodes. After sync, the data state depends on which instance processed the event first.

### 5.3 The Solution: Version-Stamped Operations

Every operation result (data mutation) is tagged with the handler version that produced it:

```
Node: {
  id: "post-123/version/4",
  labels: ["VersionNode"],
  properties: {
    ...content...,
    _producedBy: "handler:seo-enrichment/v2",
    _producedAt: HLC_TIMESTAMP
  }
}
```

This enables:

1. **Version-aware conflict resolution.** When merging data from different handler versions, the engine can apply version-specific merge strategies instead of blindly using LWW.

2. **Retroactive re-processing.** When Instance A upgrades from H-v1 to H-v2, it can identify all data produced by H-v1 (via the `_producedBy` tag) and optionally re-process it under H-v2.

3. **Version compatibility checking.** During sync handshake, instances exchange their handler version manifests. If handler versions are incompatible (e.g., breaking schema change), the sync protocol can flag the conflict before data transfer.

### 5.4 Migration as Graph Transformation

When a handler is updated (v1 to v2), the migration can itself be expressed as an operation subgraph:

```
MigrationSubgraph (v1 -> v2):
  QueryOp: Find all Nodes with _producedBy = "handler:H/v1"
  TransformOp: Apply the v1->v2 transformation
  UpdateNodeOp: Write the transformed data with _producedBy = "handler:H/v2"
```

This migration subgraph is:
- **Inspectable:** The engine can determine what data it will modify before running it.
- **Syncable:** The migration itself is a subgraph that can sync to other instances.
- **Deterministic:** If it contains only pure and read-deterministic operations, any instance can run it and get the same result.
- **Versioned:** The migration has its own version chain, so you can roll it back.

---

## 6. Sync-Specific Operations

Some operations only make sense in the context of synchronization between instances. These form a separate category in the vocabulary.

### 6.1 Inbound Sync Operations (Receiving Data)

| Operation | Purpose | Classification |
|---|---|---|
| **ValidateInboundOp** | Check a received Node/Edge against the local schema and capability constraints. Reject structurally invalid data before it enters the local graph. | Pure (validation is deterministic given schema + data) |
| **MergePropertyOp** | Apply CRDT merge for a single Node property. Compares local HLC with incoming HLC, applies LWW per-field. | Read-deterministic (depends on local state) |
| **MergeEdgeOp** | Apply add-wins CRDT merge for an Edge. If the Edge exists locally (possibly tombstoned), resolve conflict. | Read-deterministic |
| **DetectConflictOp** | Identify when a merge produces a semantic conflict (not just a CRDT resolution). E.g., two instances both set `status` to different values -- CRDT resolves it, but the application may want to flag this for human review. | Read-deterministic |
| **ApplyDeltaOp** | Apply a batch of sync deltas atomically (within a transaction). Ensures the local graph transitions from one consistent state to another. | Write-deterministic |

### 6.2 Outbound Sync Operations (Sending Data)

| Operation | Purpose | Classification |
|---|---|---|
| **ComputeDeltaOp** | Given a peer's vector clock, compute the set of Nodes/Edges that the peer is missing. | Read-deterministic |
| **SerializeSubgraphOp** | Serialize a set of Nodes/Edges into the wire format (CBOR, MessagePack, etc.) for transport. | Pure (given the same Nodes, produces the same bytes) |
| **FilterByCapabilityOp** | Before sending, remove any Nodes/Edges that the receiving peer does not have capability to access. Prevents information leakage. | Read-deterministic (depends on capability grants) |

### 6.3 Sync Protocol Operations (Session Management)

| Operation | Purpose | Classification |
|---|---|---|
| **HandshakeOp** | Exchange vector clocks, capability proofs, and handler version manifests. Establish sync session parameters. | Non-deterministic (network I/O) |
| **AcknowledgeOp** | Confirm receipt and successful application of a delta batch. Updates the local vector clock entry for the peer. | Write-deterministic |
| **ForkOp** | Record a fork point in the version chain. Mark the sync agreement as forked. Guarantee snapshot consistency. | Write-deterministic |
| **ResumeOp** | Resume an interrupted sync session from the last acknowledged delta batch. | Read-deterministic (lookup last ack point) |

### 6.4 Sync Operations Are Not Handler Operations

Sync operations and handler operations occupy different layers:

```
Layer 3: Application handlers (content:afterCreate, seo:score, etc.)
  - Authored by module developers
  - Encoded as operation subgraphs
  - Sync between instances as data
  - Subject to capability checking and static analysis

Layer 2: Sync protocol operations
  - Authored by the engine
  - NOT encoded as operation subgraphs (they are engine-internal)
  - NOT synced (they are protocol mechanics)
  - Run with platform-level capabilities

Layer 1: Core graph operations
  - CreateNode, UpdateNode, DeleteNode, CreateEdge, etc.
  - The primitives that both layers above compose
```

This separation is critical. If sync operations were themselves operation subgraphs, a malicious peer could craft a "handler" that mimics a sync operation, potentially bypassing capability checks. By making sync operations engine-internal (not representable as user-authored Node types), the attack surface is eliminated.

---

## 7. The Homoiconicity Advantage: Module Installation as Data Sync

### 7.1 How It Works

Traditional module installation:
```
1. Developer publishes module to npm/registry
2. Server operator runs `npm install module-x`
3. Node.js loads the module's JavaScript
4. Module registers handlers, content types, etc.
5. Handlers are opaque functions in memory
```

Benten module installation via sync:
```
1. Developer publishes module as a subgraph on their instance
2. User's instance syncs the module subgraph
3. Engine receives Nodes: ContentTypeDef, BlockDef, HandlerDef (operation subgraph), CapabilityManifest
4. Engine validates: Is the operation subgraph structurally valid? Are all operation types allowed? Does the capability manifest match the operation analysis?
5. If valid, the module is "installed" -- its definitions are now in the local graph
6. IVM updates materialized views (handler resolution, content type registry, etc.)
7. Handlers are now active -- the engine can execute them when triggered
```

Steps 2-6 use the exact same sync mechanism as any other data sync. There is no separate "install" step. The module's code (operation subgraphs) and data (content type definitions, block definitions) arrive through the same channel.

### 7.2 What This Simplifies

**No package manager.** Modules are discovered and installed through sync relationships. A module author publishes by making their module subgraph available for sync. A user installs by establishing a sync agreement with the author's instance (or a mirror/relay).

**No build step.** Modules are not compiled, bundled, or transpiled. They are graph structures. What the author creates is exactly what the user receives.

**Automatic updates.** When the module author updates their module (new version of a handler, new content type definition), the changes propagate via the existing sync mechanism. The user's instance receives the new version Nodes through the version chain. Migration subgraphs handle data transformations.

**Offline-first.** If the user's instance is offline when an update is published, it receives the update on the next sync. The version chain ensures ordering. The CRDT merge ensures consistency.

### 7.3 What This Complicates

**Broken handlers can sync.** If a module author publishes a handler with a bug (structurally valid but logically wrong), it syncs to all subscribed instances. There is no "unpublish" -- the version is in the graph. The author can publish a fix (new version), but the broken version is part of the history.

Mitigation: Handler execution is always gated by the handler's version. If a user pins to a specific handler version, they are not affected by a broken newer version until they explicitly upgrade. This is the "lockfile" equivalent.

**Schema evolution is harder.** When a module adds a new field to a content type, the definition change syncs immediately. But existing data does not have the new field. Migration subgraphs can handle this, but they must also sync and execute. The ordering matters: the migration must execute AFTER the schema change is applied but BEFORE any handlers that depend on the new field.

Mitigation: Migrations are tagged with a `dependsOn` edge pointing to the schema version they apply to. The engine executes migrations in dependency order, not sync-arrival order.

**Trust bootstrapping.** The first module sync requires an out-of-band trust decision. The user must decide: "I trust this peer enough to sync their module subgraph into my instance." After that, the capability system governs what the module can do. But the initial trust decision is not automatable.

Mitigation: Module manifests (capability requirements, author identity, content hash of the operation subgraph) can be published and verified independently of the sync channel. A user could review a manifest on a web page before establishing the sync agreement.

---

## 8. Graph-Level Sandboxing: Restricting Traversal

### 8.1 The Mechanism

Instead of (or in addition to) WASM sandboxing, the engine can sandbox handlers by restricting which Node types and Edge types they can traverse:

```
CapabilityGrant: {
  grantedTo: "module:seo-plugin",
  domain: "graph",
  operations: {
    read: {
      nodeLabels: ["ContentType:post", "ContentType:page", "SeoScore"],
      edgeTypes: ["HAS_FIELD", "HAS_SCORE", "BELONGS_TO"]
    },
    write: {
      nodeLabels: ["SeoScore"],
      edgeTypes: ["HAS_SCORE"]
    },
    traverse: {
      maxDepth: 3,
      startLabels: ["ContentType:post", "ContentType:page"]
    }
  }
}
```

When executing the SEO plugin's handler:
1. The engine creates a **restricted traversal context** scoped to the capability grant.
2. Any QueryOp or TraverseOp in the handler that attempts to access a Node label not in `read.nodeLabels` is rejected at execution time.
3. Any CreateNodeOp/UpdateNodeOp that targets a label not in `write.nodeLabels` is rejected.
4. Any traversal that exceeds `maxDepth` is truncated.

### 8.2 Static vs Runtime Enforcement

**Static enforcement (pre-execution):** The engine extracts the handler's capability manifest from its operation subgraph (Section 4.1) and checks it against the module's grants. If the manifest exceeds the grants, the handler is rejected before execution.

**Runtime enforcement (during execution):** Even if static analysis passes, the engine enforces capabilities at each operation step. This is defense-in-depth: static analysis catches honest mistakes, runtime enforcement catches adversarial construction.

**Why both?** Static analysis can be defeated by indirection. A handler might contain a QueryOp that targets a variable label (computed from data at runtime). Static analysis cannot determine the label without executing the handler. Runtime enforcement catches this case.

### 8.3 The "Walled Garden" Pattern

For high-security contexts (e.g., syncing modules from unknown peers), the engine can create a **walled garden**: an isolated subgraph where the module's operations can only see and modify Nodes within the garden.

```
WalledGarden: {
  id: "garden:seo-plugin",
  boundary: {
    roots: ["module:seo-plugin"],
    edgeTypes: ["OWNS", "DEPENDS_ON"],
    maxDepth: 10
  },
  exports: {
    // What the module can expose to the outside world
    nodeLabels: ["SeoScore"],
    edgeTypes: ["HAS_SCORE"]
  },
  imports: {
    // What the module can see from the outside world (read-only copies)
    nodeLabels: ["ContentType:post", "ContentType:page"],
    edgeTypes: ["HAS_FIELD"]
  }
}
```

Inside the walled garden, the module operates on a virtual subgraph. Reads from `imports` are proxied from the main graph (read-only). Writes go to the garden's own subgraph. Exports are the only Nodes/Edges that leak from the garden into the main graph.

This is the capability equivalent of a Docker container with mounted volumes: the module has full access within its garden but limited, controlled access to the outside world.

---

## 9. The Complete Operation Vocabulary

Based on the analysis above, here is the full vocabulary organized by category:

### 9.1 Data Operations (Deterministic, Graph-Primitive)

These are the lowest-level operations. They correspond directly to the engine's core graph primitives.

| Node Type | Properties | Determinism | Sync Safety |
|---|---|---|---|
| `op:CreateNode` | `label`, `properties`, `targetAnchor?` | Write-det | Result-only |
| `op:UpdateNode` | `targetAnchor`, `properties` (partial) | Write-det | Result-only |
| `op:DeleteNode` | `targetAnchor`, `softDelete: bool` | Write-det | Result-only |
| `op:CreateEdge` | `type`, `fromAnchor`, `toAnchor`, `properties?` | Write-det | Result-only |
| `op:DeleteEdge` | `type`, `fromAnchor`, `toAnchor` | Write-det | Result-only |
| `op:SetProperty` | `targetAnchor`, `field`, `value` | Write-det | Result-only |

**Edges:**
- `WRITES_TO` from any write op to its target anchor Node
- `READS_FROM` from any op that references data to its source anchor Node

### 9.2 Query Operations (Read-Deterministic)

| Node Type | Properties | Sync Safety | Notes |
|---|---|---|---|
| `op:Query` | `pattern` (traversal spec), `filters`, `limit`, `offset` | With-context | Reads graph state |
| `op:Traverse` | `startAnchor`, `edgeTypes`, `maxDepth`, `direction` | With-context | Graph walk |
| `op:Lookup` | `targetAnchor` or `label` + `property` + `value` | With-context | Point lookup |
| `op:Aggregate` | `source` (query ref), `function` (count/sum/avg/min/max), `groupBy?` | With-context | Aggregation |
| `op:ViewRead` | `viewName` | With-context | IVM view read (O(1)) |
| `op:ResolveRef` | `anchor` | With-context | Follow anchor -> CURRENT -> version |

**Edges:**
- `DATA_SOURCE` from query op to its schema/type target
- `FEEDS_INTO` from query op to consuming transform/write op

### 9.3 Transform Operations (Pure)

| Node Type | Properties | Notes |
|---|---|---|
| `op:Transform` | `expression` (safe expression language, cf. `@benten/expressions`) | Maps input data to output data |
| `op:Filter` | `condition` (boolean expression) | Removes items that don't match |
| `op:Map` | `expression` | Applies a function to each item in a collection |
| `op:Reduce` | `accumulator`, `expression` | Folds a collection into a single value |
| `op:Merge` | `strategy` (concat/union/intersection/override) | Combines multiple inputs |
| `op:Condition` | `condition`, branches: true/false edges | Conditional branching in the DAG |
| `op:Validate` | `schema` (reference to a schema Node) | Validates input against a schema |
| `op:SchemaCheck` | `targetLabel`, `expectedFields` | Verifies a Node matches expected structure |
| `op:Compare` | `operator` (eq/neq/gt/lt/gte/lte), `left`, `right` | Comparison, returns boolean |
| `op:Arithmetic` | `operator` (+/-/*/div/mod), `left`, `right` | Math, returns number |
| `op:String` | `operator` (concat/split/slice/upper/lower/replace/match) | String manipulation |
| `op:Coerce` | `targetType` | Type conversion (string->number, etc.) |
| `op:Default` | `value` | Provides a default when input is null/undefined |
| `op:Pick` | `fields: string[]` | Select specific properties from an object |
| `op:Omit` | `fields: string[]` | Remove specific properties from an object |
| `op:Spread` | (no extra props) | Flatten nested object one level |
| `op:Compose` | (no extra props) | Pass output of one transform as input to next (syntactic sugar for chaining) |

**Edges:**
- `TRANSFORM_INPUT` / `TRANSFORM_OUTPUT` for data flow
- `TRUE_BRANCH` / `FALSE_BRANCH` from `op:Condition`

### 9.4 Control Flow Operations

| Node Type | Properties | Determinism | Notes |
|---|---|---|---|
| `op:Condition` | `condition` | Pure | Branch point in DAG. Two outgoing edges: TRUE_BRANCH, FALSE_BRANCH. |
| `op:ForEach` | (none -- implicit from collection input) | Pure | Fan-out: applies subsequent operations to each item. NOT a loop -- it's a map over a finite collection from a QueryOp. |
| `op:Parallel` | `branches: int` | Pure | Execute independent sub-DAGs concurrently. Gather results at a `op:Merge` or `op:Collect` Node. |
| `op:Collect` | `timeout?` | Pure | Gather results from parallel branches. |
| `op:Sequence` | (implicit from edge ordering) | Pure | Execute sub-DAGs in order. Default behavior when edges have explicit ordering. |
| `op:TryCatch` | (none) | Pure | Error boundary. If the "try" sub-DAG fails, execute the "catch" sub-DAG. |
| `op:Abort` | `errorCode`, `message` | Pure | Halt execution with an error. Triggers compensation in workflow contexts. |

**Key design decision: no general loops.** `op:ForEach` iterates over a finite collection (the result of a QueryOp). There is no `op:While` or `op:Loop`. This guarantees termination. If you need to process items one at a time with complex logic, you use `op:ForEach` with a bounded input.

### 9.5 Event/Reactive Operations

| Node Type | Properties | Determinism | Notes |
|---|---|---|---|
| `op:EmitEvent` | `eventType`, `payload` (data from prior operations) | Write-det | Fires a reactive notification into the local engine. |
| `op:SubscribePattern` | `pattern` (query pattern for IVM subscription) | Write-det | Register a reactive subscription. Used during handler registration, not during handler execution. |
| `op:OnEvent` | `eventType`, `filter?` | N/A | Not an executable operation -- it's a trigger Node. It marks the entry point of a handler subgraph. When the specified event fires, the handler DAG executes. |

**`op:OnEvent` is special.** It is not executed as part of a dataflow DAG. It is the root Node of a handler subgraph. The engine's reactive system watches for matching events and triggers execution of the subgraph rooted at this Node. It is the graph-native equivalent of `addEventListener` or `onEmit`.

### 9.6 Capability Operations

| Node Type | Properties | Determinism | Notes |
|---|---|---|---|
| `op:CheckCapability` | `entity`, `domain`, `action`, `scope` | Read-det | Returns boolean. Used in Condition branches. |
| `op:RequireCapability` | `entity`, `domain`, `action`, `scope` | Read-det | Throws Abort if capability not present. Fail-closed. |
| `op:WithCapability` | `grantRef` | Pure | Scopes subsequent operations to a specific capability grant. Defense-in-depth: even if the handler has broader capabilities, this sub-DAG runs with a restricted set. |

### 9.7 Non-Deterministic Operations (Sync-Forbidden)

| Node Type | Properties | Notes |
|---|---|---|
| `op:ExternalCall` | `url`, `method`, `headers`, `body`, `timeout` | HTTP/network calls. MUST be wrapped in op:TryCatch. |
| `op:WasmCall` | `wasmModuleRef`, `function`, `args` | Delegate to WASM sandbox. Subject to gas metering. |
| `op:Random` | `min`, `max`, `type` (int/float/uuid) | Random value generation. |
| `op:Timestamp` | `format` | Wall-clock time. Use `op:HlcTimestamp` for sync-safe timestamps. |
| `op:HlcTimestamp` | (none) | Hybrid Logical Clock timestamp. Deterministic within a single causal history. |
| `op:Log` | `level`, `message` | Instance-local logging. |

### 9.8 Sync Operations (Engine-Internal, Not User-Authorable)

These are NOT representable as user-authored operation Nodes. They are engine-internal primitives used by the sync protocol layer (Section 6).

| Internal Operation | Purpose |
|---|---|
| `sync:ValidateInbound` | Schema + capability check on received data |
| `sync:MergeProperty` | Per-field LWW CRDT merge |
| `sync:MergeEdge` | Add-wins Edge merge |
| `sync:ComputeDelta` | Vector clock comparison + delta extraction |
| `sync:ApplyDelta` | Atomic batch application |
| `sync:Handshake` | Session establishment |
| `sync:Acknowledge` | Delta receipt confirmation |
| `sync:Fork` | Fork point recording |
| `sync:Resume` | Interrupted session resumption |

These use a `sync:` prefix that is reserved and rejected if encountered in a user-authored operation subgraph.

---

## 10. Structural Validation Rules

Before executing any operation subgraph (whether received via sync or authored locally), the engine validates:

### 10.1 Graph Structure Rules

| Rule | Check | Failure Mode |
|---|---|---|
| **Acyclicity** | The operation subgraph, following dataflow edges only, forms a DAG. | Reject: `CYCLE_DETECTED` |
| **Single root** | The subgraph has exactly one entry point (an `op:OnEvent` Node for handlers, or a designated start Node for workflows). | Reject: `SCHEMA_VIOLATION` |
| **Connected** | Every operation Node is reachable from the root via dataflow edges. Orphan operations are not allowed. | Reject: `ORPHAN_NODE` |
| **Type-safe edges** | Dataflow edges connect compatible operation types (e.g., a QueryOp output can feed a TransformOp input, but not vice versa). | Reject: `INVALID_EDGE` |
| **Bounded fan-out** | ForEach and Parallel operations have a configurable maximum branch count. | Reject: `RECURSION_LIMIT` |
| **No sync-forbidden ops in synced subgraphs** | If the subgraph arrived via sync, it must not contain non-deterministic operation types. | Reject: `TRUST_VIOLATION` |
| **No reserved prefixes** | User-authored operation Nodes cannot use the `sync:` or `engine:` type prefixes. | Reject: `TIER_RESTRICTION` |

### 10.2 Capability Rules

| Rule | Check | Failure Mode |
|---|---|---|
| **Manifest satisfaction** | The extracted capability manifest is a subset of the module's granted capabilities. | Reject: `FORBIDDEN` |
| **Write scope** | Every write operation targets a label within the module's write scope. | Reject: `FORBIDDEN` |
| **Read scope** | Every query/traverse operation targets labels within the module's read scope. | Reject: `FORBIDDEN` |
| **Traversal depth** | No TraverseOp specifies a depth exceeding the module's traversal depth limit. | Reject: `TIER_RESTRICTION` |
| **External call whitelist** | ExternalCallOp URLs are in the module's approved external endpoint list. | Reject: `FORBIDDEN` |

### 10.3 Resource Rules

| Rule | Check | Failure Mode |
|---|---|---|
| **Subgraph size** | Total Node count in the operation subgraph does not exceed the configured maximum (e.g., 1000 operations). | Reject: `RECURSION_LIMIT` |
| **Estimated cost** | Sum of per-operation cost weights does not exceed the configured budget. | Reject: `TIER_RESTRICTION` |
| **External call count** | Number of ExternalCallOp Nodes does not exceed the per-handler limit. | Reject: `TIER_RESTRICTION` |
| **Write count** | Number of write operations does not exceed the per-handler limit. | Reject: `TIER_RESTRICTION` |

---

## 11. Content-Addressing for Operation Subgraphs

Taking a lesson from Unison's content-addressed code model:

### 11.1 Hashing Operation Subgraphs

Each operation subgraph has a content hash computed from:
- The operation Node types and their properties (excluding instance-specific metadata like timestamps)
- The edge structure (connections between operations)
- The order of edges (for deterministic hashing)

This hash serves as the subgraph's identity, independent of naming.

### 11.2 Benefits

**Deduplication:** If two modules define the same handler logic, they produce the same hash. The engine stores the subgraph once and links both modules to it.

**Integrity verification:** When receiving a synced operation subgraph, the receiver recomputes the hash from the received Nodes/Edges. If the hash does not match, the subgraph has been tampered with.

**Cache keying:** Execution results can be cached by (subgraph hash + input data hash). If the same handler processes the same data, the result is guaranteed identical.

**Version comparison:** Two versions of a handler can be compared by their hashes. If the hashes match, the versions are identical regardless of what the author calls them.

### 11.3 The Hash Chain

Combined with version chains, this produces a hash chain for each handler:

```
HandlerAnchor: "handler:seo-enrichment"
  -> CURRENT -> v3 (hash: abc123)
  -> v2 (hash: def456) -> v1 (hash: ghi789)
```

Each version Node's hash is the content hash of its operation subgraph. The version chain provides ordering; the content hash provides integrity.

---

## 12. Open Design Questions

### 12.1 Expression Language

The `op:Transform`, `op:Filter`, and `op:Condition` operations reference an "expression" -- a safe sub-language for data manipulation. What is this language?

**Option A: Reuse @benten/expressions.** The existing sandboxed expression evaluator (jsep-based, no @benten deps) already supports safe property access, comparison, arithmetic, and boolean logic. Extend it with collection operations (map, filter, reduce).

**Option B: A graph-native expression model.** Instead of string expressions evaluated at runtime, expressions are themselves small operation sub-DAGs. A condition like `post.title.length > 0` becomes: `op:Pick[field: 'title'] -> op:String[operator: 'length'] -> op:Compare[operator: 'gt', right: 0]`. This is more verbose but fully inspectable and deterministic.

**Recommendation:** Option A for the first release, with a migration path to Option B. String expressions are sufficient for common cases and already tested. Graph-native expressions add significant complexity and should be deferred until the operation model is proven.

### 12.2 Cost Model

The resource rules (Section 10.3) reference "per-operation cost weights." What are these weights? This is the engine's equivalent of Ethereum's gas costs. Each operation type has a fixed cost:

| Operation | Cost Weight | Rationale |
|---|---|---|
| Pure transforms | 1 | Cheap: in-memory computation |
| Point lookups | 5 | IVM read: O(1) but still a cache access |
| Query/traverse | 10-50 (depends on depth) | Variable: depends on graph structure |
| Create/update/delete Node | 20 | Write: storage + IVM update |
| Create/delete Edge | 15 | Write: storage + IVM update |
| External call | 100 | Network I/O: slow, unpredictable |
| WASM call | 50 | Sandbox overhead + metered execution |

These weights are configurable by the instance operator. A high-security instance might set ExternalCall to 1000 (effectively banning it by exhausting the budget). A performance-focused instance might set write costs lower.

### 12.3 Error Propagation in Handler DAGs

When an operation in the middle of a handler DAG fails, what happens to downstream operations?

**Option A: Abort the entire handler.** Simple. The handler either succeeds completely or fails completely. Combined with `op:TryCatch`, the author can define fallback behavior.

**Option B: Dataflow-based error propagation.** Errors flow through the DAG like data. Downstream operations receive an Error value instead of a Data value and can inspect it. This enables partial-failure handlers.

**Recommendation:** Option A for correctness and simplicity. Option B adds significant complexity and is rarely needed in a CMS context. The `op:TryCatch` boundary provides sufficient error handling.

### 12.4 Handler Composition

Can one handler's operation subgraph reference another handler's subgraph? This is the equivalent of function calls in a traditional language.

**Answer:** Yes, via `op:InvokeHandler`. This operation Node references another handler's anchor. At execution time, the engine loads and executes the referenced handler with the provided input data, then passes the result to the next operation in the calling handler.

**Key constraint:** The invoked handler's capability manifest must be a subset of the calling handler's grants. You cannot escalate capabilities by calling another handler.

**Cycle prevention:** `op:InvokeHandler` creates a transitive dependency. The engine must check that the handler call graph (across subgraphs) is also acyclic. This is a cross-subgraph DAG check.

---

## 13. Relationship to Existing Thrum Primitives

### 13.1 Migration Path from Current Event System

| Current (Thrum V3) | Future (Benten Engine) |
|---|---|
| `onPipeline('content:afterCreate', handler)` | `op:OnEvent { eventType: 'content:afterCreate' }` as root of handler subgraph |
| `pipeline('content:afterCreate', payload)` | Engine fires event, IVM resolves all `op:OnEvent` Nodes matching the event type, executes their subgraphs |
| `RestrictedEventBus` + `TIER_MATRIX` | Capability grants on handler modules. The handler's capability manifest is checked before execution. |
| `HandlerOpts.priority` | `priority` property on `op:OnEvent` Node. IVM view sorts handlers by priority. |
| `subscribesTo` on ThrumModule | `op:OnEvent` Nodes in the module's subgraph declare what events the module handles. No separate `subscribesTo` needed -- it's implicit in the operation graph. |

### 13.2 Migration Path from Current Module System

| Current (Thrum V3) | Future (Benten Engine) |
|---|---|
| `defineModule({ id, onRegister, ... })` | Module definition is a subgraph: ModuleDef Node + handler subgraphs + content type Nodes + block Nodes |
| `onRegister()` callback | `op:OnEvent { eventType: 'module:registered' }` handler subgraph that creates/registers definitions |
| `onMigrate(db)` callback | Migration subgraph with schema-change operations |
| `onBootstrap(db)` callback | `op:OnEvent { eventType: 'engine:ready' }` handler subgraph |
| `onDestroy()` callback | `op:OnEvent { eventType: 'module:uninstalling' }` handler subgraph |
| `ctx.registerCleanup()` | Cleanup subgraph linked to module via `CLEANUP_HANDLER` edge |

### 13.3 Migration Path from Materializer Pipeline

| Current (Thrum V3) | Future (Benten Engine) |
|---|---|
| `createMaterializerPipeline(steps)` | A handler subgraph where each step is an operation Node, connected by dataflow edges in sequence |
| `MaterializerStep.execute(ctx)` | The body of each step becomes a sub-DAG of operation Nodes |
| `PipelineSecurityOptions.freezeContext` | The engine's immutable data flow handles this: each operation receives immutable input and produces new output |

---

## 14. Summary: Design Principles for the Operation Vocabulary

1. **Determinism by default.** Pure and read-deterministic operations are the norm. Non-deterministic operations are explicit, flagged, and restricted.

2. **Static analysis before execution.** The graph structure IS the analysis target. No need for separate linting tools -- the engine's validation layer inspects the operation subgraph before running it.

3. **No general loops.** DAG-only control flow guarantees termination. `op:ForEach` over bounded collections is the only iteration mechanism.

4. **Capabilities are graph-native.** The same graph that stores the operation subgraph also stores the capability grants that govern its execution. The engine checks both in the same traversal.

5. **Sync-safety is a structural property.** An operation subgraph's sync classification is determined by its Node types, not by runtime behavior. The engine can classify a subgraph without executing it.

6. **Code syncs like data.** Module installation uses the same CRDT sync mechanism as content replication. No separate install protocol.

7. **Content-addressing for integrity.** Operation subgraphs are hashed for deduplication, integrity verification, and cache keying.

8. **Layers are strict.** Sync operations are engine-internal and cannot be mimicked by user-authored operation Nodes. Handler operations are user-authorable and subject to capability enforcement. Core graph operations are the primitives both layers compose.

9. **The graph IS the sandbox.** Restricting which Node types and Edge types an operation can traverse provides capability enforcement without a separate sandbox runtime. WASM is the escape hatch for general-purpose computation.

10. **Non-Turing-complete is a feature.** The operation vocabulary is deliberately less expressive than a general-purpose language. This enables safety guarantees (termination, determinism, static analysis) that no Turing-complete system can provide.

---

## Sources

- [Deterministic Execution in Smart Contracts -- Altius](https://www.altiuslabs.xyz/learn/deterministic-execution-why-its-essential-for-smart-contracts)
- [Thunderbolt: Concurrent Smart Contract Execution (EDBT 2026)](https://openproceedings.org/2026/conf/edbt/paper-29.pdf)
- [Holochain Validation: Assuring Data Integrity](https://developer.holochain.org/concepts/7_validation/)
- [Holochain Deterministic Integrity (hdi crate)](https://docs.rs/hdi/latest/hdi/)
- [CosmWasm Smart Contract Platform](https://cosmwasm.com/)
- [CosmWasm: Pushing Boundaries of Smart Contract Innovation](https://medium.com/@NilesRiver3/cosmwasm-pushing-the-boundaries-of-smart-contract-innovation-51f385f2b995)
- [Unison: The Big Idea (content-addressed code)](https://www.unison-lang.org/docs/the-big-idea/)
- [Trying out Unison: Code as Hashes -- SoftwareMill](https://softwaremill.com/trying-out-unison-part-1-code-as-hashes/)
- [CRDT Implementation Guide (2025) -- Velt](https://velt.dev/blog/crdt-implementation-guide-conflict-free-apps)
- [The CRDT Dictionary -- Ian Duncan (2025)](https://www.iankduncan.com/engineering/2025-11-27-crdt-dictionary/)
- [Capability-based Security -- Wikipedia](https://en.wikipedia.org/wiki/Capability-based_security)
- [Object-capability Model -- Wikipedia](https://en.wikipedia.org/wiki/Object-capability_model)
- [How to Implement Capability-based Security (2026)](https://oneuptime.com/blog/post/2026-01-30-capability-based-security/view)
- [Node-RED Multi-server Sync Discussion](https://discourse.nodered.org/t/node-red-multi-server-sync-and-or-deployment/38392)
- [Arbigraph: Verifiable Turing-Complete Execution Delegation (2025)](https://eprint.iacr.org/2025/710.pdf)
- [EVM Opcodes Interactive Reference](https://www.evm.codes/)
- [Ethereum Virtual Machine Opcodes](https://ethervm.io/)
- [Higher-Order Graph Databases (2025)](https://arxiv.org/html/2506.19661v1)
