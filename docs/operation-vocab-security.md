# Operation Node Vocabulary -- Security Perspective

**Date:** 2026-04-11
**Author:** Security & Trust Reviewer
**Context:** Benten Engine graph execution model where code IS Nodes and Edges. An operation subgraph defines what happens when a request arrives. The engine walks the subgraph, executing each operation Node in traversal order.
**Prerequisite reading:** `SPECIFICATION.md` (Sections 2.1-2.4), `critique-security.md`, `critique-ai-agents.md`

---

## Design Thesis

Operation subgraphs must be **deliberately NOT Turing complete**. The operation vocabulary is a total language: every subgraph provably terminates, every resource cost is bounded before execution begins, and no operation can manufacture capabilities it was not granted. This is the single most important security decision in the engine.

The vocabulary achieves useful computation through composition (chaining bounded operations via edges) rather than through unbounded looping. Where Turing-complete behavior is needed (complex business logic, AI agent code), it runs OUTSIDE the operation graph in a sandboxed runtime (QuickJS-in-WASM) that the graph invokes via a metered `Invoke` operation -- the sandbox has a gas budget and capability membrane controlled by the graph, not by the sandboxed code.

---

## Why NOT Turing Complete

If operation subgraphs support unbounded loops + conditionals + writes, three problems become unsolvable:

1. **Halting problem.** You cannot determine in advance whether a subgraph will terminate. A malicious module submits a subgraph that loops forever, consuming the engine's single-writer budget.
2. **Cost estimation.** You cannot bound the resource cost of executing a subgraph. Without cost bounds, capability budgets (Section S2 of the AI agent critique) are unenforceable.
3. **Self-modification.** A Turing-complete subgraph can compute new operation Nodes that bypass the capability checks present in the original subgraph -- it can "think its way around" restrictions.

**What we sacrifice:** Modules cannot express arbitrary algorithms as operation subgraphs. Complex logic must be delegated to the sandboxed runtime via `Invoke`. This is the right trade-off: the operation graph is the security enforcement layer (predictable, auditable, bounded), while the sandbox is the computation layer (expressive, metered, isolated).

**Precedent:** Ethereum's EVM is Turing complete with gas metering. This works but creates gas estimation complexity, out-of-gas failures that leave partial state, and re-entrancy attacks (the DAO hack). Benten's approach -- total operation graph + metered sandbox -- avoids all three: the graph always completes (total), cost is known before execution (bounded), and the sandbox cannot call back into the graph mid-operation (no re-entrancy).

---

## Foundational Security Invariants

Before defining individual operations, these invariants constrain the entire vocabulary:

**INV-1: Capability attenuation only.** An operation subgraph executes within a CapabilityEnvelope. Every operation in the subgraph can only use capabilities present in the envelope. No operation can widen the envelope. Operations can narrow the envelope for sub-invocations (attenuation). This is the object-capability model applied to graph execution.

**INV-2: No self-modification.** An operation subgraph cannot create, modify, or delete operation Nodes or their connecting edges during execution. The operation graph is immutable during its own execution. Modification of operation subgraphs is a separate lifecycle action (OperationDef registration) that requires a distinct capability and occurs outside execution.

**INV-3: Bounded execution.** Every operation has a declared cost model. The engine computes the total cost bound of a subgraph BEFORE execution begins (this is possible because the graph is acyclic and total). Execution is rejected if the cost exceeds the caller's budget. No timeout-based termination -- the engine knows in advance whether the subgraph will complete within budget.

**INV-4: System zone inviolability.** CapabilityGrant Nodes, OperationDef Nodes, Anchor Nodes, VersionMeta Nodes, and IVM definition Nodes are in the system zone. No operation in the user zone can read or write system zone Nodes unless explicitly projected through a system-zone operation (`CheckCapability`, `ReadCapability`). This addresses CRITICAL-2 from the security critique.

**INV-5: Causal attribution.** Every operation execution produces a CausalRecord (Node in the graph) that records: who initiated it, which capabilities were used, which data Nodes were read/written, and the timestamp. This is the audit trail. The CausalRecord is in the system zone -- the executor cannot modify or suppress it.

---

## Operation Node Types

### Category 1: Capability Enforcement

These operations enforce what a caller is allowed to do. They appear at the START of operation subgraphs (before any data access) and at NARROWING points (before delegated sub-invocations).

---

#### 1.1 `CheckCapability`

**Purpose:** Assert that the current execution context holds a specific capability. Fail-closed: if the capability is absent, execution aborts immediately.

**Properties:**
- `domain: String` -- capability domain (e.g., `"store"`, `"graph"`, `"ivm"`)
- `action: String` -- required action (e.g., `"read"`, `"write"`, `"create"`, `"delete"`)
- `scope: String` -- required scope pattern (e.g., `"content/*"`, `"commerce/product"`)

**What it enforces:**
- The caller's CapabilityEnvelope includes a grant that satisfies `(domain, action, scope)`.
- Scope matching uses prefix semantics: `content/*` satisfies `content/page` but not `commerce/product`.
- If the envelope was attenuated by a parent `AttenuateCapability` operation, only the attenuated set is checked.

**Where in the subgraph:** First node(s) in any operation subgraph. Before any `ReadGraph`, `WriteGraph`, or `Invoke` operation.

**On failure:** Execution aborts. The CausalRecord records the denied capability, the requester identity, and the subgraph ID. No partial effects. Error type: `CAPABILITY_DENIED`. The error message includes the required capability (for debugging) but NOT the caller's actual capabilities (information disclosure prevention).

**Security notes:**
- Multiple `CheckCapability` nodes can appear in sequence for operations requiring multiple capabilities (e.g., "read content AND write audit log").
- `CheckCapability` is O(1) when backed by a materialized IVM view of the caller's effective capabilities.
- The scope parameter is validated at OperationDef registration time against injection patterns (no glob injection, no path traversal via `../`).

---

#### 1.2 `AttenuateCapability`

**Purpose:** Create a narrowed capability envelope for a sub-invocation. The attenuated envelope is a strict subset of the current envelope. This is how delegation works: a module invokes a sub-operation on behalf of a user, passing only the capabilities the sub-operation needs.

**Properties:**
- `allow: CapabilityGrant[]` -- the capabilities to include in the attenuated envelope
- `mode: "explicit" | "subtract"` -- `explicit` means ONLY the listed capabilities; `subtract` means current envelope MINUS the listed capabilities

**What it enforces:**
- Every grant in `allow` must be satisfiable by the current envelope (you cannot grant what you do not have).
- The attenuated envelope is passed to all downstream operations via the edge from this node.
- `subtract` mode is for "give everything EXCEPT these" -- useful for revoking destructive capabilities before delegation.

**Where in the subgraph:** Before any `InvokeSubgraph` or `Invoke` operation that delegates to less-trusted code.

**On failure:** If `allow` contains a capability not present in the current envelope, execution aborts with `ATTENUATION_VIOLATION`. This prevents capability escalation via crafted attenuation lists.

**Security notes:**
- Attenuation is irreversible within an execution path. Once narrowed, no subsequent operation can widen.
- The `subtract` mode requires care: if the current envelope changes (e.g., due to a concurrent revocation), the subtraction result changes. Attenuation snapshots the envelope at execution time to prevent TOCTOU.

---

#### 1.3 `RequireIdentity`

**Purpose:** Assert that the execution context has a specific identity type. Not a capability check -- an identity check. "Is this request from an authenticated user? An AI agent? A remote instance?"

**Properties:**
- `identityType: "user" | "agent" | "module" | "remote" | "system"`
- `identityId: String?` -- optional: require a SPECIFIC identity (e.g., a specific user ID)
- `attributionMode: "direct" | "onBehalfOf"` -- `direct` means the identity IS the actor; `onBehalfOf` means the identity is acting as a delegate for another identity

**What it enforces:**
- The execution context includes an identity of the specified type.
- For `onBehalfOf` mode: both the delegate identity AND the principal identity must be present. The CausalRecord records both (audit: "Agent X did this on behalf of User Y").

**Where in the subgraph:** Immediately after `CheckCapability`. Before any data operations.

**On failure:** `IDENTITY_REQUIRED`. Abort, no partial effects.

**Security notes:**
- This operation exists because capabilities answer "can this entity do X?" but not "who is this entity?" Some operations need both: "only users can publish content" (identity) AND "only users with content:publish capability" (capability).
- AI agents executing on behalf of users carry BOTH identities. The agent's capabilities are intersected with the user's capabilities -- the agent cannot do more than the user, even if the agent's own grants are broader. This prevents the confused deputy problem.

---

#### 1.4 `EnforceBudget`

**Purpose:** Assert that the remaining execution budget is sufficient for the next operation(s). If the subgraph's pre-computed cost exceeds the caller's budget, this operation fails before any work is done.

**Properties:**
- `computeUnits: u64` -- estimated compute cost of the protected subtree
- `memoryBytes: u64` -- estimated peak memory of the protected subtree
- `ioOps: u64` -- estimated I/O operations (graph reads/writes)

**What it enforces:**
- The caller's remaining budget >= declared cost.
- Budget is decremented atomically (no race between check and decrement).
- Supports both per-request budgets (for module routes) and per-session budgets (for AI agent sessions).

**Where in the subgraph:** Before any `Invoke` (sandbox call) or `InvokeSubgraph` (delegation) that could consume significant resources.

**On failure:** `BUDGET_EXCEEDED`. Abort, no partial effects. The CausalRecord records the requested budget and the remaining budget (for capacity planning, not leaked to the caller).

**Security notes:**
- Budget enforcement is what makes the non-Turing-complete guarantee practical. Even if a sandboxed `Invoke` is Turing complete internally, its gas budget is bounded by the `EnforceBudget` node in the calling graph.
- Budgets are per-CapabilityEnvelope, not per-operation. Attenuation can narrow the budget (delegate gets less budget than delegator).
- Pre-computation of subgraph cost is possible because the operation graph is a DAG (no cycles) and each operation type has a declared cost model. The engine sums the cost along the longest path.

---

### Category 2: Data Access (Read Path)

These operations read data from the graph. Every read operation respects the current CapabilityEnvelope's scope.

---

#### 2.1 `ReadGraph`

**Purpose:** Read Nodes and/or Edges from the user zone of the graph. All reads are scoped to the current capability envelope.

**Properties:**
- `pattern: TraversalPattern` -- declarative graph pattern (labels, property filters, edge types, depth limit)
- `maxResults: u32` -- hard cap on returned results (prevents unbounded traversal)
- `maxDepth: u32` -- maximum traversal depth (prevents graph-wide scans)
- `projection: String[]` -- which properties to return (prevents over-fetching)

**What it enforces:**
- The traversal pattern is restricted to the capability envelope's scope. A module with `store:read:content/*` scope cannot traverse to `commerce/*` Nodes.
- `maxResults` and `maxDepth` are mandatory (no default that could be unbounded). The engine rejects ReadGraph operations without these limits.
- System zone Nodes are never returned. If a traversal would pass through a system zone Node (e.g., an Anchor Node), the traversal stops at the zone boundary.
- The `projection` filter is enforced at the engine level -- even if the Node has 50 properties, only the projected ones are included in the result. This prevents accidental exposure of sensitive fields.

**Where in the subgraph:** After `CheckCapability` with a read-compatible grant.

**On failure:** `SCOPE_VIOLATION` if the pattern escapes the capability scope. `RESULT_LIMIT_EXCEEDED` if the traversal hits `maxResults` (the partial result is returned with a truncation flag, NOT an error -- this prevents denial-of-service via error-induced retries).

**Security notes:**
- `ReadGraph` operates on a MVCC snapshot. The snapshot is taken at the start of the enclosing operation subgraph execution, not at the `ReadGraph` node. This ensures consistent reads across multiple `ReadGraph` operations in the same subgraph.
- Variable-length path patterns (`*1..N`) are allowed but `N` must be <= `maxDepth`. Open-ended patterns (`*1..`) are rejected at OperationDef registration time.

---

#### 2.2 `ReadView`

**Purpose:** Read a pre-computed materialized view (IVM). O(1) cost.

**Properties:**
- `viewName: String` -- name of the materialized view
- `slice: { offset: u32, limit: u32 }?` -- optional pagination

**What it enforces:**
- The caller must have a capability that covers the view's underlying data scope. A view defined over `content/*` Nodes requires `store:read:content/*` capability.
- The view definition's scope is validated at view creation time and cached. `ReadView` checks the caller's capability against the cached scope, not against the view's Cypher definition (O(1) check, not query analysis).
- View names in the system zone namespace (prefixed with `__benten:`) are not readable via `ReadView`. They require `ReadSystemView` (a platform-only operation).

**Where in the subgraph:** After `CheckCapability`. Typically the primary read mechanism for hot-path operations (event handler lookup, content listing).

**On failure:** `VIEW_NOT_FOUND` or `SCOPE_VIOLATION`.

---

#### 2.3 `ReadCapability`

**Purpose:** Read the current execution context's effective capabilities. This is an introspection operation -- the caller learns what IT can do, not what others can do.

**Properties:**
- `filter: { domain?: String, action?: String }?` -- optional filter to narrow results

**What it enforces:**
- Returns ONLY the caller's own capabilities. Never returns other entities' capabilities (prevents reconnaissance).
- Does not traverse the capability graph -- returns the pre-computed effective capability set from the CapabilityEnvelope.

**Where in the subgraph:** Anywhere. Used for conditional logic: "if I have publish capability, show the publish button."

**On failure:** Never fails. Returns an empty set if no capabilities match the filter.

**Security notes:**
- This is safe to expose to all trust tiers because it reveals nothing about other entities.
- The AI agent critique (S1) calls for dry-run mode. `ReadCapability` enables client-side dry-run: the agent checks its capabilities before constructing a write operation, avoiding wasted computation.

---

### Category 3: Data Access (Write Path)

These operations modify data in the graph. Every write operation is atomic within the enclosing subgraph's transaction, capability-checked, and causally recorded.

---

#### 3.1 `WriteGraph`

**Purpose:** Create, update, or delete Nodes and Edges in the user zone.

**Properties:**
- `mutations: Mutation[]` -- ordered list of graph mutations
  - `CreateNode { labels, properties }`
  - `UpdateNode { nodeId, properties }` -- merge semantics (only specified properties change)
  - `DeleteNode { nodeId }`
  - `CreateEdge { type, from, to, properties }`
  - `DeleteEdge { edgeId }`
- `expectedVersions: Map<NodeId, u64>?` -- optimistic concurrency: fail if any node's version differs

**What it enforces:**
- Every mutation is checked against the capability envelope's scope. `store:write:content/*` allows writing to `content/page` Nodes but not `commerce/product` Nodes.
- The label check uses the Node's labels, not the Node's ID. A Node with labels `["ContentType", "SEOExtension"]` requires capability for BOTH label namespaces.
- `DeleteNode` additionally checks for a `store:delete` capability (separate from `store:write`). Destructive operations are a distinct capability tier.
- System zone Nodes are never writable via `WriteGraph`. Attempting to write to a system zone Node produces `ZONE_VIOLATION` (not `SCOPE_VIOLATION` -- different error to distinguish "wrong scope" from "wrong zone").
- `expectedVersions` enforces optimistic concurrency. If any specified node's current version differs from the expected version, the entire subgraph aborts with `VERSION_CONFLICT`. No partial writes.

**Where in the subgraph:** After `CheckCapability` with write-compatible grants. After `ReadGraph` if the write depends on read data (read-then-write pattern).

**On failure:** `SCOPE_VIOLATION`, `ZONE_VIOLATION`, `VERSION_CONFLICT`, `SCHEMA_VIOLATION` (if the graph has schema constraints on labels/properties). All failures abort the entire enclosing subgraph transaction.

**Security notes:**
- Writes produce version chain entries (new version Node, NEXT_VERSION edge, CURRENT pointer update). These are system zone operations performed by the engine implicitly -- the caller's `WriteGraph` specifies user zone mutations only, and the engine adds version chain maintenance automatically.
- Each `WriteGraph` records the identity context (user/agent/module/remote) in the version Node. This is automatic, not optional -- the caller cannot suppress attribution (INV-5).
- IVM views affected by the mutations are updated after the subgraph transaction commits (not during). This means reads within the same subgraph see uncommitted state via MVCC, but other subgraphs see the IVM views update atomically on commit.

---

#### 3.2 `ConditionalWrite`

**Purpose:** Write only if a condition holds. This is the graph equivalent of a compare-and-swap: "update Node X's status to 'published' only if its current status is 'draft'."

**Properties:**
- `condition: ReadGraph` -- inline read operation that must return a non-empty result
- `onMatch: WriteGraph` -- mutations to apply if condition is met
- `onNoMatch: "abort" | "skip"` -- behavior when condition fails

**What it enforces:**
- The condition and the write are evaluated atomically within the same MVCC snapshot. No TOCTOU gap between the read and the write.
- Both the condition's `ReadGraph` and the `onMatch`'s `WriteGraph` are individually capability-checked. A caller needs BOTH read and write capabilities.
- `onNoMatch: "abort"` fails the entire subgraph. `onNoMatch: "skip"` continues execution with no mutations from this node.

**Where in the subgraph:** Wherever an atomic read-modify-write is needed. Common pattern: conditional state transitions (draft -> published), idempotent writes (create only if not exists).

**On failure:** `CONDITION_NOT_MET` (when `onNoMatch: "abort"`), plus all the failure modes of `ReadGraph` and `WriteGraph`.

**Security notes:**
- This prevents the common TOCTOU attack where an adversary modifies data between a check and a subsequent write. By making the check-and-write atomic, the engine guarantees the condition held at the moment of the write.
- The condition is a full `ReadGraph` (not a Cypher string), so it benefits from the same scope enforcement and traversal limits.

---

### Category 4: External Invocation

These operations cross the boundary between the operation graph (total, bounded) and external computation (potentially Turing complete, metered).

---

#### 4.1 `Invoke`

**Purpose:** Call into a sandboxed runtime (QuickJS-in-WASM) to execute arbitrary computation with a gas budget and a capability membrane.

**Properties:**
- `runtimeId: String` -- identifies the sandbox instance (module-scoped, isolated)
- `entryPoint: String` -- function name in the sandbox to call
- `args: Value[]` -- serialized arguments (JSON-compatible)
- `gasBudget: u64` -- maximum computation steps in the sandbox
- `memoryLimit: u64` -- maximum memory bytes the sandbox can allocate
- `timeout: Duration` -- wall-clock timeout (defense-in-depth against gas metering bugs)
- `capabilities: CapabilityGrant[]` -- capabilities the sandbox code can exercise via host functions

**What it enforces:**
- The sandbox code can ONLY interact with the graph via host functions. Each host function is gated by the `capabilities` list. If the sandbox code calls a host function it was not granted, the call returns an error (NOT an abort -- the sandbox can handle the error).
- `gasBudget` is decremented for every instruction the sandbox executes. When exhausted, the sandbox is terminated and the `Invoke` node produces `GAS_EXHAUSTED`.
- `memoryLimit` is enforced by the WASM runtime. Allocation beyond the limit produces `MEMORY_EXHAUSTED`.
- `timeout` is a wall-clock backstop. If the sandbox has not returned after `timeout`, it is forcibly terminated. This catches pathological cases where the gas metering itself has a bug.
- The sandbox CANNOT call back into the operation graph. There is no re-entrancy. The sandbox receives data via `args` and returns data via the function return value. Any graph mutations it needs are expressed as a return value that the calling operation graph interprets and applies via subsequent `WriteGraph` nodes.

**Where in the subgraph:** After `CheckCapability`, after `EnforceBudget`, after `AttenuateCapability` (to narrow capabilities for the sandbox).

**On failure:** `GAS_EXHAUSTED`, `MEMORY_EXHAUSTED`, `INVOKE_TIMEOUT`, `SANDBOX_ERROR` (the sandbox code threw an exception). All failures are non-fatal to the enclosing subgraph by default (the subgraph can handle the error via a `Branch` node). If the subgraph does not handle the error, it propagates as a subgraph abort.

**Security notes:**
- **No re-entrancy.** This is the critical difference from Ethereum's EVM. In the EVM, a contract can call another contract which can call back into the first contract, leading to re-entrancy attacks (the DAO hack). Benten's `Invoke` is strictly one-way: graph -> sandbox -> return value -> graph. The sandbox cannot trigger graph operations directly.
- **Capability membrane.** The host functions exposed to the sandbox are a strict subset of the graph API, filtered by `capabilities`. A sandbox with `store:read:content/*` gets a `readGraph()` host function that is pre-scoped -- it cannot read `commerce/*` data even if it constructs a broader pattern. The membrane is enforced at the host function layer, not trusted to the sandbox code.
- **Isolated runtimes.** Each module gets its own WASM sandbox instance. Module A's sandbox cannot access Module B's sandbox memory. The runtime ID ensures isolation.

---

#### 4.2 `InvokeSubgraph`

**Purpose:** Execute another registered operation subgraph within the current execution context, with optionally attenuated capabilities.

**Properties:**
- `subgraphId: String` -- the registered operation subgraph to invoke
- `args: Value[]` -- input arguments
- `capabilityOverride: CapabilityGrant[]?` -- if present, attenuate the capability envelope for the invoked subgraph

**What it enforces:**
- The invoked subgraph must be a registered OperationDef in the system zone. Ad-hoc subgraph construction at runtime is NOT allowed (INV-2: no self-modification).
- If `capabilityOverride` is provided, it must be a subset of the current envelope (attenuation only, never escalation).
- The invoked subgraph's pre-computed cost is checked against the current budget before invocation. If it would exceed the budget, `BUDGET_EXCEEDED` is returned without executing the subgraph.
- The invoked subgraph executes within the same MVCC snapshot and transaction as the caller. Writes are visible to subsequent operations in the caller after the invoked subgraph returns.

**Where in the subgraph:** After capability and budget checks. Used for composing complex operations from smaller registered subgraphs.

**On failure:** Any failure in the invoked subgraph propagates to the caller. The caller can use `Branch` to handle specific error types.

**Security notes:**
- `InvokeSubgraph` does NOT create a new transaction. The invoked subgraph shares the caller's transaction. This means all-or-nothing atomicity: if any operation in the chain fails, the entire top-level transaction rolls back. This prevents partial state from sub-invocations.
- The invoked subgraph's CausalRecord is a child of the caller's CausalRecord, creating a causal chain for audit.

---

### Category 5: Control Flow

These operations control the execution path through the operation subgraph. They are deliberately limited to maintain the total (non-Turing-complete) property.

---

#### 5.1 `Branch`

**Purpose:** Conditional execution. Route to one of N downstream paths based on a condition.

**Properties:**
- `condition: BranchCondition` -- one of:
  - `PropertyEquals { nodeId, property, value }` -- check a Node property
  - `ResultEmpty { sourceOp }` -- check if a ReadGraph returned empty
  - `ErrorType { sourceOp, errorCode }` -- check if a previous operation failed with a specific error
  - `CapabilityHeld { domain, action, scope }` -- check if a capability is present (non-failing variant of `CheckCapability`)
- `onTrue: OperationNodeId` -- edge to follow if condition is true
- `onFalse: OperationNodeId` -- edge to follow if condition is false

**What it enforces:**
- No looping. `Branch` directs execution forward in the DAG. Since the operation graph is a DAG (directed acyclic graph), there are no back-edges. `Branch` cannot create a cycle because the edge targets must have higher topological order than the `Branch` node.
- The condition is evaluated in O(1) -- it reads from the current execution context (MVCC snapshot, previous operation results), not from arbitrary graph queries. This keeps branching cheap and predictable.

**Where in the subgraph:** Anywhere after the operations whose results the condition references.

**On failure:** If the condition itself cannot be evaluated (e.g., `sourceOp` does not exist), the subgraph aborts with `INVALID_BRANCH`. This is a registration-time error (caught when the OperationDef is stored), not a runtime error.

**Security notes:**
- `Branch` is the only conditional operation. There is no general `if/else` with computed conditions (which could hide information leakage in the condition expression). The condition types are enumerated and each has a well-defined security profile.
- `CapabilityHeld` is a non-failing check: it returns true/false without aborting. This enables patterns like "if the user has admin capability, include extra data; otherwise return the basic response."

---

#### 5.2 `Merge`

**Purpose:** Synchronization point where multiple parallel execution paths converge. All incoming paths must complete before the Merge node executes.

**Properties:**
- `awaitAll: boolean` -- if true, wait for ALL incoming edges. If false, wait for ANY (first to complete wins, others are cancelled).
- `timeout: Duration?` -- maximum wait time for convergence (only relevant for `awaitAll: false` with async invoke paths)

**What it enforces:**
- All incoming paths run within the same transaction. The Merge does not commit or checkpoint -- it is a synchronization barrier within a single atomic operation.
- If ANY incoming path failed and the failure was not handled by a `Branch`, the Merge propagates the failure (the subgraph aborts).
- `awaitAll: false` (race semantics) cancels losing paths. Cancelled paths produce no writes (their mutations are discarded because the transaction has not committed). This is safe because the operation graph is within a single MVCC transaction.

**Where in the subgraph:** After parallel `ReadGraph` or `InvokeSubgraph` operations that can execute concurrently.

**On failure:** Propagates the first unhandled failure from any incoming path.

---

#### 5.3 `Transform`

**Purpose:** Pure data transformation. Takes input values and produces output values without any side effects. No graph reads, no graph writes, no external calls.

**Properties:**
- `expression: TransformExpression` -- a restricted expression language (arithmetic, string operations, property access, array manipulation, object construction). Deliberately NOT Turing complete: no loops, no function definitions, no recursion. Equivalent to a spreadsheet formula.
- `inputs: Map<String, OperationNodeId>` -- named references to previous operation results
- `outputName: String` -- name for the result (referenced by downstream operations)

**What it enforces:**
- The expression language is sandboxed: it cannot access the graph, cannot perform I/O, cannot allocate unbounded memory.
- Expression evaluation has a hard step limit (e.g., 10,000 operations). Expressions exceeding this limit fail with `EXPRESSION_LIMIT`. This prevents pathological expressions like deeply nested string concatenations.
- The expression language is the same as `@benten/expressions` (jsep + custom AST walker), ported to Rust. This means the security properties proven in the TypeScript implementation (toString/valueOf blocked, prototype-safe property access, no computed property keys) carry forward.

**Where in the subgraph:** Between `ReadGraph` and `WriteGraph` operations. Used to reshape data from a read into the format needed for a write.

**On failure:** `EXPRESSION_ERROR` (type mismatch, null dereference, etc.), `EXPRESSION_LIMIT`.

---

### Category 6: Validation and Schema Enforcement

These operations enforce data integrity constraints. They are separate from capability checks (which enforce authorization) -- validation ensures data is structurally correct.

---

#### 6.1 `ValidateSchema`

**Purpose:** Validate data against a schema definition. Used before writes to ensure data conforms to the content type's field definitions.

**Properties:**
- `schemaRef: NodeId` -- reference to a schema definition Node in the graph (e.g., a ContentType definition)
- `data: OperationNodeId` -- reference to the data to validate (output of a previous `ReadGraph` or `Transform`)
- `mode: "strict" | "partial"` -- `strict` requires all required fields; `partial` allows subset (for PATCH operations)

**What it enforces:**
- Field types match the schema definition.
- Required fields are present (in `strict` mode).
- Field values satisfy constraints (min/max, pattern, enum membership).
- No unknown fields are present (reject extra properties that could carry injection payloads).
- The schema definition is read from the system zone (schema definitions are protected). A module cannot supply its own schema definition to validate against -- it must reference an existing registered schema.

**Where in the subgraph:** After `Transform` (which reshapes input data) and before `WriteGraph` (which persists it).

**On failure:** `SCHEMA_VIOLATION` with a list of specific field errors. Abort, no partial effects.

**Security notes:**
- Schema validation in the operation graph replaces application-level validation. This is defense-in-depth: even if the TypeScript layer has a validation bug, the engine rejects structurally invalid data.
- For sync scenarios: incoming data from remote instances passes through `ValidateSchema` before being written to the graph. This is the first line of defense against malicious Nodes (the sync scenario from the requirements).

---

#### 6.2 `ValidateRelationship`

**Purpose:** Validate that an edge creation or deletion maintains referential integrity. Prevents orphaned references and dangling edges.

**Properties:**
- `edgeType: String` -- the edge type being validated
- `sourceId: NodeId` -- source Node
- `targetId: NodeId` -- target Node
- `sourceLabels: String[]` -- required labels on the source Node
- `targetLabels: String[]` -- required labels on the target Node
- `cardinality: "one" | "many"` -- if `one`, ensure no other edge of this type exists from the source

**What it enforces:**
- Both source and target Nodes exist (no dangling edges).
- Both Nodes have the required labels (type-safe edges).
- Cardinality constraint is satisfied.
- The target Node is not tombstoned (in a CRDT context, a tombstoned Node should not accept new incoming edges).

**Where in the subgraph:** Before `WriteGraph` operations that create edges.

**On failure:** `RELATIONSHIP_VIOLATION` with specifics (missing source, wrong labels, cardinality exceeded, tombstoned target).

---

### Category 7: Audit and Observability

These operations produce audit records and enable post-execution analysis. They are inserted by the engine automatically (not by module authors) but module authors can add additional audit points.

---

#### 7.1 `AuditPoint`

**Purpose:** Create an explicit audit record at a specific point in the operation subgraph. The engine automatically records subgraph start/end, but modules can add domain-specific audit points.

**Properties:**
- `eventType: String` -- domain-specific event identifier (e.g., `"content:published"`, `"commerce:orderPlaced"`)
- `details: Value` -- structured data to include in the audit record
- `severity: "info" | "warn" | "security"` -- classification for monitoring/alerting

**What it enforces:**
- The audit record is written to the system zone. The module cannot suppress or modify it after creation.
- The `details` value is size-limited (e.g., 4KB) to prevent audit log flooding.
- `security` severity triggers real-time monitoring (if configured). Used for events like failed capability checks, schema violations, or suspicious patterns.

**Where in the subgraph:** At meaningful business logic boundaries. After `WriteGraph` (to record what was written), after `Branch` (to record which path was taken), after `Invoke` (to record sandbox execution results).

**On failure:** Never fails. Audit writing failures are logged but do not abort the subgraph. Audit is best-effort within a best-effort system zone write.

---

#### 7.2 `DryRun`

**Purpose:** Execute a subgraph in preview mode: all reads are real, all writes are simulated. Returns what WOULD happen without committing any changes.

**Properties:**
- `targetSubgraph: OperationNodeId` -- the subgraph (or portion) to dry-run
- `includeViewEffects: boolean` -- if true, compute which IVM views would be affected

**What it enforces:**
- Writes within the dry-run scope are captured but not committed. The MVCC transaction is rolled back after the dry-run completes.
- Capability checks still run (the dry-run tells you "would succeed" or "would fail: CAPABILITY_DENIED").
- The dry-run result includes: mutations that would be applied, views that would be updated, capability checks that passed/failed, estimated cost.

**Where in the subgraph:** Wraps a portion of the subgraph. Typically used as the top-level node when an AI agent requests a preview.

**On failure:** The dry-run itself does not fail (it reports failures that WOULD occur). If the dry-run infrastructure itself fails, `DRY_RUN_ERROR`.

**Security notes:**
- Dry-run still requires capabilities. You cannot use dry-run to probe what capabilities you WOULD need without holding them. This prevents capability enumeration attacks.
- The dry-run CausalRecord is tagged as `dryRun: true` so it is distinguishable from real executions in audit logs.

---

### Category 8: Sync and Remote Operations

These operations handle data arriving from or departing to remote instances. They enforce trust boundaries at the protocol level.

---

#### 8.1 `ValidateRemoteOrigin`

**Purpose:** Verify the cryptographic identity and capabilities of a remote instance before accepting its data.

**Properties:**
- `peerId: DID` -- the remote instance's decentralized identifier
- `proof: UCANChain` -- the UCAN delegation chain proving the remote instance's capabilities
- `maxClockSkew: Duration` -- maximum acceptable HLC timestamp divergence (default: 5 minutes, per CRITICAL-3 recommendation from security critique)

**What it enforces:**
- The UCAN chain is cryptographically valid (signatures verify, no expired tokens, no revoked tokens).
- The remote instance's claimed capabilities are satisfied by the UCAN chain (attenuation is correct at every delegation step).
- The remote instance's HLC timestamps are within `maxClockSkew` of the local clock. Timestamps exceeding the skew are rejected, preventing the clock manipulation attack described in CRITICAL-3 of the security critique.
- Revocation check: the engine queries its local revocation store for any revoked UCANs in the chain. If any link in the chain is revoked, the entire proof is rejected.

**Where in the subgraph:** First node in any sync-handling operation subgraph. Before any `WriteGraph` that persists received data.

**On failure:** `PEER_UNTRUSTED` (signature failure), `UCAN_REVOKED`, `CLOCK_SKEW_EXCEEDED`, `CAPABILITY_INSUFFICIENT`.

---

#### 8.2 `SanitizeIncoming`

**Purpose:** Validate and sanitize Nodes and Edges received from a remote instance before they enter the local graph.

**Properties:**
- `maxNodes: u32` -- maximum number of Nodes to accept in one sync batch
- `maxEdges: u32` -- maximum number of Edges to accept in one sync batch
- `maxPropertySize: u64` -- maximum bytes for any single property value
- `allowedLabels: String[]?` -- if present, only accept Nodes with these labels (whitelist)
- `blockedLabels: String[]` -- always reject Nodes with these labels (system zone labels)

**What it enforces:**
- Incoming Nodes with system zone labels (`__benten:CapabilityGrant`, `__benten:OperationDef`, `__benten:Anchor`, `__benten:IVMView`) are ALWAYS rejected. A remote instance cannot inject operation subgraphs or capability grants via sync. This is the defense against "malicious Nodes that self-modify to escalate" from the requirements.
- Property values exceeding `maxPropertySize` are rejected (prevents memory exhaustion via oversized data).
- The total batch size is bounded by `maxNodes` and `maxEdges` (prevents sync flooding, part of the defense for CRITICAL-3).
- Schema validation: if the incoming Node has labels that correspond to registered ContentType definitions, the Node's properties are validated against the schema. Schema-invalid Nodes are rejected with `SYNC_SCHEMA_VIOLATION`.

**Where in the subgraph:** After `ValidateRemoteOrigin`, before `WriteGraph`.

**On failure:** Rejects individual invalid Nodes/Edges (partial acceptance: valid items in the batch are accepted, invalid ones are rejected with error details returned to the remote peer).

**Security notes:**
- The `blockedLabels` list is hardcoded at the engine level and cannot be overridden by modules or operators. System zone labels are NEVER acceptable in sync payloads.
- This is where self-modifying operation subgraph attacks are stopped. An attacker who crafts a Node with label `__benten:OperationDef` and properties that define a new operation subgraph with escalated capabilities will have that Node rejected at the `SanitizeIncoming` boundary before it ever reaches the graph.

---

## Critical Design Questions -- Answered

### Can an operation subgraph modify OTHER operation subgraphs?

**No.** INV-2 (no self-modification) applies globally: operation subgraphs cannot create, modify, or delete ANY operation Nodes, including those belonging to other subgraphs. Operation definitions are system zone Nodes writable only through a dedicated `RegisterOperationDef` lifecycle API that requires `system:operationDef:write` capability -- a capability that is NEVER granted to modules, only to the platform runtime during module installation.

An operation subgraph CAN invoke other registered subgraphs via `InvokeSubgraph`, but it cannot modify them. It can only compose them.

### Can an operation subgraph create NEW operation Nodes with more capabilities than the creator?

**No.** Operation subgraphs cannot create operation Nodes at all during execution (INV-2). But even during the registration lifecycle (outside execution), the `RegisterOperationDef` API validates that every `CheckCapability` node in the new subgraph references capabilities that are a subset of the registering module's own capability grants. A module cannot register an operation that checks for a capability the module itself does not hold. This is compile-time (registration-time) enforcement, not runtime.

### How do you prevent a malicious operation subgraph from consuming infinite resources?

Three layers of defense:

1. **Totality.** The operation vocabulary is not Turing complete. No loops, no unbounded recursion. Every subgraph is a DAG with finite nodes. The engine can compute the maximum number of operations before execution begins.

2. **Pre-computed cost bounds.** Every operation type has a declared cost model. The engine sums costs along the longest path in the DAG. If the sum exceeds the caller's budget, execution is rejected before any work is done. This works because the graph is a DAG (topological sort gives a total ordering of operations) and each operation's cost is bounded.

3. **Sandbox metering.** The one place where Turing-complete code runs (the `Invoke` sandbox) has a gas budget and wall-clock timeout. The sandbox is forcibly terminated when either limit is reached. The sandbox cannot call back into the graph (no re-entrancy), so terminating it does not leave partial graph state.

### How do you audit what an operation subgraph DID?

Every subgraph execution produces a CausalRecord tree:

- **Root CausalRecord:** subgraph ID, caller identity, capability envelope, timestamp, total cost consumed.
- **Per-operation CausalRecord:** operation type, inputs, outputs, duration, capability checks passed/failed.
- **WriteGraph records:** exact mutations applied (Node IDs, property diffs, edge changes).
- **Invoke records:** sandbox entry point, gas consumed, return value.
- **Branch records:** which condition evaluated to which value, which path was taken.

CausalRecords are system zone Nodes linked by `CAUSED_BY` edges. They form a tree that mirrors the operation subgraph's execution path. They are immutable after creation (system zone, no user-zone writes). They participate in IVM: an audit dashboard backed by a materialized view over CausalRecords updates in real-time as operations execute.

### Should there be "blessed" operation types that only platform-level code can use?

**Yes.** The following operations are platform-only (require `system:*` capabilities that are never granted to modules):

| Operation | Capability Required | Why Platform-Only |
|-----------|-------------------|-------------------|
| `RegisterOperationDef` | `system:operationDef:write` | Prevents modules from injecting arbitrary operation subgraphs |
| `ReadSystemView` | `system:ivm:read` | System zone IVM views contain capability data, version chain metadata |
| `WriteSystemZone` | `system:zone:write` | Direct writes to system zone Nodes (capability grants, anchors, IVM defs) |
| `OverrideBudget` | `system:budget:override` | Allows platform code to execute without budget constraints (for bootstrapping) |
| `RevokeCapability` | `system:capability:revoke` | Only the platform can revoke capabilities (not modules, not AI agents) |

Modules interact with the system zone exclusively through the dedicated operations in categories 1-8 above, which provide mediated, capability-checked access. They never write to the system zone directly.

---

## Operation Subgraph Structure -- Security Constraints

An operation subgraph must satisfy these structural constraints (validated at registration time, not at execution time):

1. **DAG property.** The operation graph must be a directed acyclic graph. No cycles means no unbounded execution. Validated via topological sort at registration time -- if topological sort fails, the subgraph is rejected.

2. **Entry node.** Every subgraph has exactly one entry node (no parallel starts). The entry node must be `CheckCapability` or `RequireIdentity`. This enforces that security checks are always first.

3. **No unreachable nodes.** Every node must be reachable from the entry node. Unreachable nodes could hide malicious operations that are triggered via a future edge modification (which INV-2 prevents, but defense-in-depth says reject the subgraph).

4. **Terminal nodes are typed.** Every path through the subgraph must end at a `Return` node (success) or an `Abort` node (failure). No hanging paths. This ensures the engine always knows whether the subgraph completed successfully.

5. **Maximum depth.** The longest path (entry to terminal) is bounded by a platform-configurable limit (default: 64 operation nodes). This provides an additional cost bound beyond the per-operation cost model.

6. **Maximum fan-out.** Each node can have at most N outgoing edges (default: 16). This bounds the width of parallel execution and prevents exponential path expansion.

7. **Invoke depth limit.** `InvokeSubgraph` can be nested to a maximum depth (default: 8). This prevents deeply nested subgraph chains from exceeding the total cost bound due to aggregation of per-subgraph costs.

---

## Mapping to Security Scenarios

### Scenario: A community module's handler runs

```
Entry: CheckCapability { domain: "store", action: "read", scope: "seo/*" }
  |
  v
RequireIdentity { identityType: "module" }
  |
  v
EnforceBudget { computeUnits: 1000, memoryBytes: 1MB, ioOps: 50 }
  |
  v
ReadGraph { pattern: "seo/* nodes", maxResults: 100, maxDepth: 2 }
  |
  v
Transform { expression: "compute SEO score from read data" }
  |
  v
WriteGraph { mutations: [UpdateNode { seo score }], expectedVersions: {...} }
  |
  v
Return
```

- The module can only read `seo/*` data (capability scope).
- It cannot read `content/*` or `commerce/*` data (scope violation).
- It cannot modify CapabilityGrant Nodes (zone violation).
- It cannot create new operation Nodes (INV-2).
- Its budget is bounded (1000 compute units, 50 I/O ops).

### Scenario: An AI agent's action runs

```
Entry: CheckCapability { domain: "store", action: "write", scope: "content/*" }
  |
  v
RequireIdentity { identityType: "agent", attributionMode: "onBehalfOf" }
  |
  v
EnforceBudget { computeUnits: 5000, memoryBytes: 10MB, ioOps: 200 }
  |
  v
Branch { condition: CapabilityHeld { domain: "store", action: "delete", scope: "content/*" } }
  |                    |
  onTrue               onFalse
  |                    |
  v                    v
ReadGraph (full)     ReadGraph (limited)
  |                    |
  v                    v
Merge { awaitAll: true }
  |
  v
AttenuateCapability { allow: [store:write:content/*, store:read:content/*] }
  |
  v
Invoke { runtimeId: "agent-001", entryPoint: "generateContent",
         gasBudget: 100000, memoryLimit: 50MB, timeout: 30s,
         capabilities: [store:read:content/*] }
  |
  v
ValidateSchema { schemaRef: ContentType("page"), data: invokeResult, mode: "strict" }
  |
  v
WriteGraph { mutations: [CreateNode from validated data] }
  |
  v
AuditPoint { eventType: "content:agentCreated", severity: "info",
             details: { agentId, userId, contentId } }
  |
  v
Return
```

- The agent has per-action capability grants (CheckCapability at entry).
- Writes are attributed to BOTH the agent AND the user (RequireIdentity with onBehalfOf).
- The agent's capabilities are intersected with the user's (the agent cannot exceed the user's permissions).
- Dry-run mode: wrap the entire subgraph in a `DryRun` node to preview effects.
- Budget enforcement prevents runaway agent loops.
- The sandbox call (Invoke) has gas metering and cannot call back into the graph.

### Scenario: A remote instance syncs data

```
Entry: ValidateRemoteOrigin { peerId: "did:key:...", proof: UCANChain,
                              maxClockSkew: 5min }
  |
  v
CheckCapability { domain: "sync", action: "write", scope: "content/*" }
  |
  v
SanitizeIncoming { maxNodes: 1000, maxEdges: 3000, maxPropertySize: 1MB,
                   blockedLabels: ["__benten:CapabilityGrant", "__benten:OperationDef",
                                   "__benten:Anchor", "__benten:IVMView"] }
  |
  v
ValidateSchema { schemaRef: inferred-from-labels, mode: "strict" }
  |
  v
ValidateRelationship { for each edge in batch }
  |
  v
WriteGraph { mutations: validated batch }
  |
  v
AuditPoint { eventType: "sync:batchReceived", severity: "info",
             details: { peerId, nodesAccepted, nodesRejected, edgesAccepted } }
  |
  v
Return
```

- Remote identity is cryptographically verified (ValidateRemoteOrigin).
- Clock skew is bounded (prevents timestamp manipulation attacks).
- System zone labels are rejected (prevents operation subgraph injection).
- Schema validation catches structurally invalid data.
- Relationship validation catches dangling edges and tombstoned targets.

---

## Summary: Complete Operation Vocabulary

| # | Operation | Category | Trust Level | Cost Model |
|---|-----------|----------|-------------|------------|
| 1 | `CheckCapability` | Enforcement | All | O(1) -- IVM lookup |
| 2 | `AttenuateCapability` | Enforcement | All | O(n) -- n = grants to attenuate |
| 3 | `RequireIdentity` | Enforcement | All | O(1) -- context check |
| 4 | `EnforceBudget` | Enforcement | All | O(1) -- counter check |
| 5 | `ReadGraph` | Read | All (scoped) | O(n) -- n = result count, bounded by maxResults |
| 6 | `ReadView` | Read | All (scoped) | O(1) -- pre-computed |
| 7 | `ReadCapability` | Read | All (self only) | O(1) -- envelope copy |
| 8 | `WriteGraph` | Write | Requires write capability | O(n) -- n = mutations |
| 9 | `ConditionalWrite` | Write | Requires read + write | O(n) -- condition read + mutations |
| 10 | `Invoke` | External | All (metered) | gas budget + wall-clock timeout |
| 11 | `InvokeSubgraph` | External | All (attenuated) | Pre-computed subgraph cost |
| 12 | `Branch` | Control | All | O(1) -- condition eval |
| 13 | `Merge` | Control | All | O(1) -- barrier |
| 14 | `Transform` | Control | All | O(steps) -- bounded by expression limit |
| 15 | `ValidateSchema` | Validation | All | O(fields) -- schema check |
| 16 | `ValidateRelationship` | Validation | All | O(1) -- existence + label check |
| 17 | `AuditPoint` | Observability | All | O(1) -- write to system zone |
| 18 | `DryRun` | Observability | All | Cost of wrapped subgraph (no commit) |
| 19 | `ValidateRemoteOrigin` | Sync | Platform + remote | O(chain) -- UCAN chain verification |
| 20 | `SanitizeIncoming` | Sync | Platform | O(batch) -- per-node validation |

**20 operation types.** The vocabulary is deliberately small. Every operation has a known cost model, a defined failure behavior, and clear capability requirements. Complex behavior emerges from COMPOSITION of these primitives, not from individual operation complexity.

---

## Sources

- [UCAN Specification](https://github.com/ucan-wg/spec)
- [Object-Capability Model](https://en.wikipedia.org/wiki/Object-capability_model)
- [Graph-Based Access Control](https://en.wikipedia.org/wiki/Graph-based_access_control)
- [Turing Incomplete Advantages (Increment)](https://increment.com/programming-languages/turing-incomplete-advantages/)
- [Ethereum DAO Re-entrancy Attack](https://en.wikipedia.org/wiki/The_DAO)
- [Securing AI Agent Execution (arXiv)](https://arxiv.org/pdf/2510.21236)
- [SAFE-MCP Framework (The New Stack)](https://thenewstack.io/safe-mcp-a-community-built-framework-for-ai-agent-security/)
- [CVE-2025-68613: Node.js Sandbox Escape in n8n](https://www.penligent.ai/hackinglabs/cve-2025-68613-deep-dive-how-node-js-sandbox-escapes-shatter-the-n8n-workflow-engine/)
- [Neo4j Fine-Grained Access Control](https://neo4j.com/product/neo4j-graph-database/security/)
- [Analysing Object-Capability Security (Oxford)](https://www.cs.ox.ac.uk/files/2690/AOCS.pdf)
