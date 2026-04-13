# Operation Node Vocabulary -- Systems/Engine Perspective

**Author:** Engine Philosophy Guardian
**Date:** 2026-04-11
**Status:** Design proposal (pre-implementation)
**Constraint:** MINIMAL set of irreducible operation primitives for a universal composable platform

---

## Design Philosophy

These operation types are the **instruction set** of the engine. The engine walks an operation subgraph (a DAG of operation Nodes connected by Edges) and evaluates each Node according to its type. Everything the platform does -- HTTP handlers, event listeners, data pipelines, AI agent actions, module lifecycle -- is expressed as a composition of these primitives.

### Design constraints

1. **Fewer types is better.** Each type must earn its place by being irreducible -- it cannot be composed from other types without unacceptable cost (performance, safety, or expressiveness).

2. **Deliberately NOT Turing complete.** No general recursion. No unbounded loops. Operation subgraphs are DAGs (acyclic by construction). Bounded iteration exists but the bound is declared upfront. This makes every subgraph inspectable, terminable, and auditable. An AI agent or human can read any operation subgraph and predict its behavior.

3. **Microsecond evaluation.** Each primitive evaluates in microseconds. Complex operations arise from composition, not from complex primitives. The engine's IVM pre-computes expensive results; operations read pre-computed answers.

4. **Data-in, data-out.** Every operation takes a Value and produces a Value. The intermediate state flows through Edges. No hidden side channels. An observer can trace every value through the subgraph.

5. **Capability-gated at every boundary.** The engine checks capabilities before executing each operation Node. An operation subgraph cannot do more than its capability grant allows.

---

## The 10 Primitives

These 10 types cover data access, control flow, data transformation, communication, security, and meta-operations. We justify why each cannot be eliminated.

---

### 1. READ

**One-line:** Retrieve data from the graph.

**Why irreducible:** The engine must have a way to fetch data. Without READ, nothing can observe the graph.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `target` | `string` | What to read: a Node ID, a view name, or a Cypher pattern |
| `mode` | `"node" \| "view" \| "query" \| "traverse"` | Resolution strategy |
| `params` | `Record<string, Value>` | Query parameters (for parameterized Cypher) |
| `projection` | `string[]?` | Which properties to return (null = all) |
| `options` | `{ limit?, offset?, sort?, direction? }` | Pagination and ordering (for query/traverse) |

**Input:** `{ context: Value }` -- the current pipeline context (may contain IDs or patterns to parameterize the read)

**Output:** `{ result: Value }` -- the read data (a Node, an array of Nodes, or a query result set)

**Edges:**
- `NEXT` -- to the operation that receives this data
- `DEPENDS_ON` -- to a Node whose value parameterizes the read (e.g., an ID from a previous step)

**Example -- fetch a content item by ID from a route parameter:**

```
[READ mode="node" target="${routeParams.id}"]
    --NEXT--> [TRANSFORM expression="$.result"]
    --NEXT--> [RESPOND status=200]
```

---

### 2. WRITE

**One-line:** Mutate the graph (create, update, delete Nodes/Edges).

**Why irreducible:** The engine must have a way to modify data. READ alone makes the system read-only.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `action` | `"create" \| "update" \| "delete" \| "createEdge" \| "deleteEdge"` | What mutation to perform |
| `target` | `string?` | Node/Edge ID (for update/delete) |
| `labels` | `string[]?` | Labels for new Nodes |
| `edgeType` | `string?` | Type for new Edges |
| `edgeFrom` | `string?` | Source Node for new Edges |
| `edgeTo` | `string?` | Target Node for new Edges |

**Input:** `{ data: Value }` -- the properties to write (for create/update), or empty (for delete)

**Output:** `{ id: string, version: number }` -- the created/updated Node/Edge identity, or `{ deleted: true }`

**Edges:**
- `NEXT` -- to the operation that runs after the write
- `DEPENDS_ON` -- to the operation that provides the data to write

**Example -- create a content Node:**

```
[VALIDATE schema="contentType:page"]
    --NEXT--> [WRITE action="create" labels=["Content","Page"]]
    --NEXT--> [EMIT event="content:afterCreate"]
```

**Write interception:** The engine runs all registered GATE operations (see #6) before executing the WRITE. A GATE can reject or transform the data. This replaces the TypeScript `pipeline('content:beforeCreate')` pattern.

---

### 3. TRANSFORM

**One-line:** Reshape, compute, or derive a new Value from an existing Value.

**Why irreducible:** Data rarely arrives in the shape the next step needs. Without TRANSFORM, every READ would need to return exactly what every consumer expects, which violates composability.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `expression` | `string` | A sandboxed expression (JSONPath-like, arithmetic, string ops) |
| `template` | `Record<string, string>?` | Object template with expression-valued fields |
| `mode` | `"expression" \| "template" \| "pick" \| "merge" \| "flatten"` | Transform strategy |
| `fields` | `string[]?` | For pick: which fields to keep |

**Input:** `{ value: Value }` -- the data to transform

**Output:** `{ value: Value }` -- the transformed data

**Edges:**
- `NEXT` -- to the next operation
- `MERGE_FROM` -- additional data sources to merge into the result (for merge mode)

`expression` uses the same sandboxed evaluator as `@benten/expressions` -- no prototype access, no function calls, no side effects. The engine compiles expressions to an AST at registration time and evaluates the AST at runtime (microseconds).

**Example -- extract and reshape API response data:**

```
[READ mode="query" target="MATCH (p:Page {status:'published'}) RETURN p"]
    --NEXT--> [TRANSFORM mode="template" template={"items": "$.result", "count": "len($.result)"}]
    --NEXT--> [RESPOND status=200]
```

**Why not just use TRANSFORM for everything?** Because `expression` is deliberately limited: no graph access, no I/O, no mutation. It is a pure function from Value to Value. This is what makes it safe and fast. READ and WRITE are the only operations that touch the graph.

---

### 4. BRANCH

**One-line:** Route execution to one of several paths based on a condition.

**Why irreducible:** Without conditional execution, every request follows the same path. Any non-trivial application needs "if A then X, else Y." BRANCH is the only control-flow primitive that changes which operations execute.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `condition` | `string` | Sandboxed expression that evaluates to a value |
| `mode` | `"boolean" \| "match"` | Boolean (true/false paths) or match (pattern matching on value) |

**Input:** `{ value: Value }` -- the data to evaluate the condition against

**Output:** `{ value: Value }` -- passes through unchanged (BRANCH does not transform data, it only selects a path)

**Edges:**
- `TRUE` -- path when condition is truthy (boolean mode)
- `FALSE` -- path when condition is falsy (boolean mode)
- `MATCH:{pattern}` -- path for a specific pattern match value (match mode)
- `DEFAULT` -- fallback path when no match (match mode)

**Example -- check authentication:**

```
[BRANCH condition="$.user != null" mode="boolean"]
    --TRUE-->  [READ mode="view" target="admin_dashboard"]
    --FALSE--> [RESPOND status=401]
```

**Example -- route by content type:**

```
[BRANCH condition="$.contentType" mode="match"]
    --MATCH:page-->    [READ target="page_handler"]
    --MATCH:post-->    [READ target="post_handler"]
    --DEFAULT-->       [RESPOND status=404]
```

---

### 5. ITERATE

**One-line:** Apply an operation subgraph to each item in a collection, with a declared upper bound.

**Why irreducible:** Many operations are "do X for each item in a list" -- render each block in a composition, validate each field in a form, process each item in a cart. Without ITERATE, you need the caller to unroll the loop, which makes subgraphs explode in size and prevents reuse.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `source` | `string` | Expression that resolves to the collection to iterate over |
| `maxIterations` | `number` | Hard upper bound (required, enforced by engine, max 10000) |
| `parallel` | `boolean` | Whether items can be processed concurrently (default: false) |
| `collectAs` | `string` | Property name where results are collected |

**Input:** `{ value: Value }` -- must contain a collection at the path specified by `source`

**Output:** `{ value: Value, [collectAs]: Value[] }` -- original value plus collected results

**Edges:**
- `BODY` -- the operation subgraph to execute for each item (receives `{ item: Value, index: number, value: Value }`)
- `NEXT` -- the operation after all iterations complete
- `ON_ERROR` -- error handler (receives `{ error, item, index }`)

**Example -- render blocks in a composition:**

```
[READ mode="node" target="${compositionId}"]
    --NEXT--> [ITERATE source="$.blocks" maxIterations=500 collectAs="rendered" parallel=true]
                  --BODY--> [CALL subgraph="render_block"]
              --NEXT--> [TRANSFORM mode="template" template={"html": "join($.rendered)"}]
              --NEXT--> [RESPOND status=200]
```

**Why `maxIterations` is required:** This is the key safety property. A subgraph author must declare the maximum number of iterations. The engine rejects subgraphs without this bound. An AI agent can read `maxIterations=500` and know the operation will terminate. There is a system-wide ceiling (default: 10000, operator-configurable) that no subgraph can exceed.

---

### 6. GATE

**One-line:** Check a condition and either allow, reject, or transform the data flowing through.

**Why irreducible:** This is the write-interception mechanism. It replaces the TypeScript `pipeline()` and `filter()` dispatch modes. Without GATE, modules cannot enforce business rules, validate data, or apply transformations before writes occur. BRANCH routes to different paths; GATE allows/blocks/transforms on a single path.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `check` | `string` | Expression or capability requirement |
| `mode` | `"capability" \| "validate" \| "condition" \| "transform"` | What the gate does |
| `schema` | `string?` | Validation schema reference (for validate mode) |
| `onReject` | `"error" \| "skip" \| "default"` | What happens when the gate rejects |
| `errorCode` | `string?` | Error code when rejected |

**Input:** `{ value: Value, actor?: ActorRef }` -- the data being gated and the actor performing the operation

**Output:** `{ value: Value }` -- the (possibly transformed) data, or an error

**Edges:**
- `NEXT` -- the operation that runs if the gate passes
- `REJECT` -- the operation that runs if the gate fails (optional; if missing, throws)

**Modes explained:**

| Mode | Behavior |
|------|----------|
| `capability` | Check that the actor has the required capability. Pure binary: pass or reject. |
| `validate` | Validate the data against a schema (stored as a Node in the graph). Pass or reject. |
| `condition` | Evaluate a sandboxed expression. Pass (truthy) or reject (falsy). |
| `transform` | Apply a transform expression. The result replaces the data. Always passes (use for normalization, sanitization, default injection). |

**Example -- capability check + validation on content create:**

```
[GATE mode="capability" check="store:create:content/page"]
    --NEXT--> [GATE mode="validate" schema="contentType:page"]
    --NEXT--> [GATE mode="transform" check="merge($.value, {createdAt: now(), updatedAt: now()})"]
    --NEXT--> [WRITE action="create" labels=["Content","Page"]]
```

**Why GATE is not BRANCH:** BRANCH selects which path to take. GATE decides whether to continue on the CURRENT path. BRANCH never modifies data; GATE can transform it. GATE is the middleware pattern; BRANCH is the router pattern.

---

### 7. CALL

**One-line:** Invoke another operation subgraph by reference.

**Why irreducible:** Without CALL, every operation subgraph is a flat sequence. You cannot reuse common patterns (authentication, logging, rendering). CALL is the function-call primitive. It is what makes the system composable -- complex operations are built from named subgraphs.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `subgraph` | `string` | ID of the subgraph to invoke (a Node in the graph) |
| `inputMap` | `Record<string, string>?` | Map current context fields to subgraph input fields |
| `outputMap` | `Record<string, string>?` | Map subgraph output fields back to current context |
| `timeout` | `number?` | Maximum execution time in milliseconds (default: 5000) |
| `isolated` | `boolean` | Whether the subgraph runs with its own capability scope (default: true) |

**Input:** `{ value: Value }` -- the current context, mapped through `inputMap`

**Output:** `{ value: Value }` -- the subgraph result, mapped through `outputMap`

**Edges:**
- `NEXT` -- the operation after the call returns
- `ON_ERROR` -- error handler
- `ON_TIMEOUT` -- timeout handler (distinct from ON_ERROR for observability)

**Example -- reuse an authentication subgraph:**

```
# Subgraph: "authenticate_request" (reusable)
[GATE mode="condition" check="$.headers.authorization != null"]
    --NEXT--> [READ mode="node" target="session:${extractToken($.headers.authorization)}"]
    --NEXT--> [BRANCH condition="$.result != null" mode="boolean"]
                  --TRUE-->  [TRANSFORM expression="{user: $.result, authenticated: true}"]
                  --FALSE--> [TRANSFORM expression="{user: null, authenticated: false}"]

# Main handler calls it:
[CALL subgraph="authenticate_request"]
    --NEXT--> [BRANCH condition="$.authenticated" mode="boolean"]
                  --TRUE-->  [READ ...]
                  --FALSE--> [RESPOND status=401]
```

**Why `isolated` matters:** When `isolated=true`, the called subgraph inherits a narrowed capability scope. It can only exercise capabilities that the caller has AND that the subgraph's own capability grant allows. This is UCAN attenuation at the operation level. An untrusted module's subgraph runs with `isolated=true` by default, preventing capability escalation.

---

### 8. RESPOND

**One-line:** Produce a final output value that exits the operation subgraph.

**Why irreducible:** The subgraph must have a way to produce a result. Without RESPOND, the engine does not know when the subgraph is done or what its output is. For HTTP handlers, RESPOND produces the response. For event listeners, RESPOND produces the result value. For AI agent actions, RESPOND produces the action result.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `status` | `number?` | HTTP status code (for HTTP-triggered subgraphs) |
| `headers` | `Record<string, string>?` | Response headers (for HTTP) |
| `channel` | `string?` | Output channel: `"http"`, `"event"`, `"sync"`, `"value"` (default: inferred from trigger) |

**Input:** `{ value: Value }` -- the data to include in the response

**Output:** Terminal -- the subgraph stops here. The engine delivers the value to the appropriate channel.

**Edges:** None outgoing. RESPOND is always a leaf node.

**Example -- JSON API response:**

```
[RESPOND status=200 headers={"content-type": "application/json"}]
```

**Example -- event listener result:**

```
[RESPOND channel="event"]  # returns the transformed event payload
```

---

### 9. EMIT

**One-line:** Fire a one-way notification into the reactive system.

**Why irreducible:** RESPOND exits the subgraph. EMIT continues execution. Many operations need to notify other parts of the system without waiting for a response -- audit logging, cache invalidation, webhook delivery, analytics. Without EMIT, you need CALL to invoke a notification subgraph, which blocks the caller and creates unnecessary coupling.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `event` | `string` | Event name (namespaced: `content:afterCreate`, `commerce:orderPlaced`) |
| `async` | `boolean` | Whether to wait for handlers to complete (default: true = fire-and-forget) |

**Input:** `{ value: Value }` -- the event payload

**Output:** `{ value: Value }` -- passes through unchanged (EMIT does not alter the pipeline data)

**Edges:**
- `NEXT` -- the operation after the emit (execution continues immediately unless `async=false`)

**How EMIT interacts with the reactive system:** The engine's reactive layer (IVM + subscriptions) handles delivery. EMIT pushes a value into the event stream. Materialized views that depend on the event are updated. Subscriptions that match the event are fired. The emitting subgraph does not know or care who receives the event.

**Example -- emit after content creation:**

```
[WRITE action="create" labels=["Content","Page"]]
    --NEXT--> [EMIT event="content:afterCreate"]
    --NEXT--> [RESPOND status=201]
```

---

### 10. WAIT

**One-line:** Pause execution until a condition is met or a timeout expires.

**Why irreducible:** Some operations require external input: human approval for destructive actions, webhook callbacks, AI agent decisions, sync completion. Without WAIT, the engine can only handle synchronous request-response patterns. WAIT enables long-running operations and approval workflows.

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `until` | `string` | What to wait for: a subscription pattern, a Node state change, or an external signal |
| `mode` | `"subscription" \| "signal" \| "timeout"` | Wait strategy |
| `timeout` | `number` | Maximum wait time in milliseconds (required, enforced) |
| `signalId` | `string?` | ID for external signal correlation (for signal mode) |

**Input:** `{ value: Value }` -- the current context (preserved across the wait)

**Output:** `{ value: Value, signal?: Value }` -- the context plus any data from the resolved wait

**Edges:**
- `NEXT` -- the operation after the wait resolves
- `ON_TIMEOUT` -- the operation if the wait times out
- `ON_ERROR` -- error handler

**Example -- approval gate for content deletion:**

```
[GATE mode="capability" check="store:delete:content/*"]
    --NEXT--> [EMIT event="approval:requested" async=false]
    --NEXT--> [WAIT mode="signal" signalId="approval:${$.nodeId}" timeout=86400000]
                  --NEXT--> [BRANCH condition="$.signal.approved" mode="boolean"]
                                --TRUE-->  [WRITE action="delete" target="${$.nodeId}"]
                                            --NEXT--> [RESPOND status=200]
                                --FALSE--> [RESPOND status=403]
                  --ON_TIMEOUT--> [RESPOND status=408]
```

**Why WAIT is not external:** The engine suspends the subgraph execution internally. The subgraph state is serialized to the graph as a pending operation Node. When the signal arrives (another subgraph completes the approval), the engine resumes execution. This is how the engine supports long-running workflows without holding threads.

---

## Edge Types

Operation Nodes are connected by these Edge types:

| Edge Type | Semantics |
|-----------|-----------|
| `NEXT` | Sequential: execute the target after the source completes |
| `BODY` | Loop body: the subgraph executed for each iteration (ITERATE only) |
| `TRUE` | Conditional true branch (BRANCH boolean mode) |
| `FALSE` | Conditional false branch (BRANCH boolean mode) |
| `MATCH:{value}` | Pattern match branch (BRANCH match mode) |
| `DEFAULT` | Fallback branch (BRANCH match mode) |
| `REJECT` | Gate rejection path (GATE) |
| `MERGE_FROM` | Additional data source (TRANSFORM merge mode) |
| `DEPENDS_ON` | Data dependency: the source needs data from the target (not an execution edge) |
| `ON_ERROR` | Error handler path |
| `ON_TIMEOUT` | Timeout handler path (CALL, WAIT) |

**Execution Edges vs Data Edges:** `NEXT`, `BODY`, `TRUE`, `FALSE`, `MATCH:*`, `DEFAULT`, `REJECT`, `ON_ERROR`, `ON_TIMEOUT` are execution edges -- they determine control flow. `DEPENDS_ON` and `MERGE_FROM` are data edges -- they declare where a Node gets its input. The engine resolves data edges first (topologically), then follows execution edges sequentially.

---

## What Is NOT a Primitive (and Why)

### Rejected: PARALLEL

**Reasoning:** ITERATE with `parallel=true` covers the common case (process items concurrently). For executing independent subgraphs concurrently, CALL multiple subgraphs from the same Node with multiple NEXT edges. The engine can detect independent NEXT edges and execute them in parallel (speculative parallelism). A dedicated PARALLEL node is syntactic sugar, not a primitive.

### Rejected: RETRY

**Reasoning:** Retry is a policy (how many times? with what backoff?) not a primitive. It can be composed from ITERATE + BRANCH + CALL:

```
[ITERATE source="[1,2,3]" maxIterations=3 collectAs="attempts"]
    --BODY--> [CALL subgraph="the_operation"]
                  --ON_ERROR--> [WAIT mode="timeout" timeout="${$.index * 1000}"]
                                    --NEXT--> [TRANSFORM expression="{retry: true}"]
    --NEXT--> [BRANCH condition="last($.attempts).success" mode="boolean"]
```

### Rejected: MAP / FILTER / REDUCE

**Reasoning:** MAP is ITERATE + TRANSFORM. FILTER is ITERATE + BRANCH (skip items). REDUCE is ITERATE with an accumulator (the ITERATE output is the reduction). These are patterns on existing primitives, not new primitives.

### Rejected: TRANSACTION

**Reasoning:** Transactions are a property of WRITE execution, not a separate operation. When multiple WRITEs appear in a subgraph with transactional semantics, the engine wraps them in a transaction automatically. The `transactional: true` property on the root Node of a subgraph declares this intent. A separate TRANSACTION operation would create confusing nesting semantics.

**How transactions work:** If any WRITE in a transactional subgraph fails, all preceding WRITEs in the same subgraph are rolled back. The version chain records the entire transaction as one unit. This matches the TypeScript workflow engine's compensation pattern, but at the engine level.

### Rejected: LOG / AUDIT

**Reasoning:** Audit logging is an EMIT to a specific channel, not a separate primitive. The engine can be configured to automatically EMIT events for all WRITEs (via a system-level gate), which replaces explicit logging operations.

### Rejected: SCHEDULE

**Reasoning:** Scheduling is WAIT with `mode="timeout"` plus external trigger infrastructure. A cron-like scheduler is a module that creates pending WAIT operations, not an engine primitive.

### Rejected: RENDER

**Reasoning:** Rendering is domain-specific (HTML, JSON, Markdown). It is a CALL to a rendering subgraph, not a primitive. The CMS materializer pipeline is a subgraph composed of READ + ITERATE + TRANSFORM + CALL operations.

---

## Composability: Building Complex Operations from Primitives

### HTTP Route Handler (replaces SvelteKit API endpoint)

```
# Subgraph: "GET /api/content/page/:id"
[GATE mode="capability" check="store:read:content/page"]
    --NEXT--> [READ mode="node" target="${routeParams.id}"]
    --NEXT--> [BRANCH condition="$.result != null" mode="boolean"]
                  --TRUE-->  [TRANSFORM mode="pick" fields=["id","title","content","status"]]
                                 --NEXT--> [RESPOND status=200]
                  --FALSE--> [RESPOND status=404]
```

### Event Listener (replaces `bus.onEmit('content:afterCreate', fn)`)

```
# Subgraph: "on:content:afterCreate -> invalidate_seo_score"
# (This subgraph is registered as a reactive subscription on the event.)
[READ mode="node" target="${event.payload.id}"]
    --NEXT--> [CALL subgraph="seo:score_content"]
    --NEXT--> [WRITE action="update" target="${$.seoScoreNodeId}"]
```

### Composition Rendering (replaces `resolveComposition` materializer pipeline)

```
# Subgraph: "render_composition"
[READ mode="node" target="${compositionId}"]                              # Fetch step
    --NEXT--> [ITERATE source="$.blocks" maxIterations=500 collectAs="visible"]
                  --BODY--> [CALL subgraph="evaluate_visibility"]         # VisibilityFilter step
              --NEXT--> [ITERATE source="$.visible" maxIterations=500 collectAs="resolved"]
                  --BODY--> [BRANCH condition="$.item.compositionRef != null" mode="boolean"]
                                --TRUE-->  [CALL subgraph="render_composition"]  # RefResolver (recursive via CALL)
                                --FALSE--> [TRANSFORM expression="$.item"]
              --NEXT--> [ITERATE source="$.resolved" maxIterations=500 collectAs="bound"]
                  --BODY--> [CALL subgraph="resolve_block_data"]          # DataBinding step
              --NEXT--> [TRANSFORM mode="template" template={"blocks": "$.bound", "meta": "$.meta"}]
              --NEXT--> [RESPOND channel="value"]
```

Note: The recursive CALL to `render_composition` is safe because:
1. Each CALL has a timeout (default 5000ms).
2. The engine tracks CALL depth and enforces a maximum (default: 20).
3. Cycle detection in compositionRef data prevents infinite recursion at the data level.

### Module Lifecycle (replaces `onRegister` / `onMigrate` / `onBootstrap`)

```
# Subgraph: "module:commerce:onRegister"
[WRITE action="create" labels=["FieldType"] ...]                        # Register field types
    --NEXT--> [WRITE action="create" labels=["ContentType"] ...]         # Register content types  
    --NEXT--> [WRITE action="create" labels=["Block"] ...]               # Register blocks
    --NEXT--> [EMIT event="module:registered"]

# Subgraph: "module:commerce:onMigrate"
[WRITE action="create" labels=["Table","Declaration"] ...]               # Declare tables
    --NEXT--> [CALL subgraph="engine:create_tables"]

# Subgraph: "module:commerce:onBootstrap"
[CALL subgraph="commerce:seed_default_products"]
    --NEXT--> [EMIT event="module:bootstrapped"]
```

### AI Agent Action (replaces MCP tool execution)

```
# Subgraph: "mcp:content:create" (auto-generated from ContentType definition Nodes)
[GATE mode="capability" check="store:create:content/${$.contentType}"]
    --NEXT--> [GATE mode="validate" schema="contentType:${$.contentType}"]
    --NEXT--> [WRITE action="create" labels=["Content","${$.contentType}"]]
    --NEXT--> [EMIT event="content:afterCreate"]
    --NEXT--> [RESPOND channel="value"]
```

### Approval Workflow (replaces custom workflow engine)

```
# Subgraph: "delete_with_approval"
[GATE mode="capability" check="store:delete:content/*"]
    --NEXT--> [WRITE action="create" labels=["PendingApproval"] ...]     # Record the intent
    --NEXT--> [EMIT event="approval:requested"]
    --NEXT--> [WAIT mode="signal" signalId="approve:${$.pendingId}" timeout=86400000]
                  --NEXT--> [BRANCH condition="$.signal.approved"]
                                --TRUE-->  [WRITE action="delete" target="${$.nodeId}"]
                                            --NEXT--> [RESPOND status=200]
                                --FALSE--> [RESPOND status=403]
                  --ON_TIMEOUT--> [WRITE action="update" target="${$.pendingId}" ...]  # Mark expired
                                      --NEXT--> [RESPOND status=408]
```

---

## Subgraph Execution Model

### Entry Points

A subgraph is triggered by one of:
- **HTTP request** -- the engine matches the route to a subgraph
- **Reactive subscription** -- a graph change matches a subscription pattern
- **CALL** -- another subgraph invokes it
- **Signal** -- an external signal resolves a pending WAIT
- **Timer** -- a timeout resolves a pending WAIT

### Execution Algorithm

```
1. Resolve the entry-point Node of the subgraph.
2. Create an ExecutionContext: { value: input, actor: ActorRef, capabilities: [...], depth: 0 }.
3. Set current = entry-point Node.
4. Loop:
   a. Check capabilities for current Node (GATE mode="capability" is explicit; all WRITEs also check implicitly).
   b. Evaluate current Node according to its type (READ, WRITE, TRANSFORM, etc.).
   c. Determine the next Node:
      - RESPOND: stop, return result.
      - BRANCH: follow the matching edge.
      - ITERATE: enter BODY subgraph for each item, then follow NEXT.
      - CALL: push frame, enter subgraph, pop frame on return.
      - WAIT: serialize context, suspend, resume on signal/timeout.
      - All others: follow NEXT edge.
   d. If no valid edge exists: throw ORPHAN_NODE error.
   e. Increment depth counter. If depth > MAX_DEPTH: throw RECURSION_LIMIT.
5. Return the RESPOND value to the trigger channel.
```

### Error Propagation

Each Node can have an `ON_ERROR` edge. When a Node throws:
1. If it has an `ON_ERROR` edge, follow it (the error handler receives `{ error, value }`).
2. If not, propagate up to the calling subgraph (CALL stack).
3. If no handler in the call stack, the subgraph fails with the error.
4. If the subgraph is transactional, all WRITEs are rolled back.

---

## Safety Properties

| Property | Mechanism |
|----------|-----------|
| **Termination** | ITERATE requires `maxIterations`. CALL has `timeout`. WAIT has `timeout`. Subgraphs are DAGs (no backward edges). |
| **Bounded memory** | `maxIterations` bounds collection sizes. CALL `depth` is bounded. No unbounded recursion. |
| **Capability enforcement** | Every WRITE checks capabilities. GATE makes checks explicit. CALL with `isolated=true` narrows scope. |
| **Inspectability** | Every operation is a Node in the graph. An AI agent can READ any subgraph to understand what it does. The engine can explain any subgraph before executing it (dry-run). |
| **Determinism** | Given the same graph state and input, a subgraph produces the same output. EMIT is the only side effect, and it is fire-and-forget (does not affect the subgraph's output). WAIT introduces non-determinism (external signal) but the subgraph declares this explicitly. |
| **Auditability** | Every execution produces a trace: which Nodes executed, in what order, with what inputs and outputs. The trace is a subgraph in the version chain. |

---

## Mapping from Thrum TypeScript Patterns

| TypeScript Pattern | Operation Subgraph Equivalent |
|--------------------|-------------------------------|
| `pipeline(event, payload)` | GATE chain (transform mode) before WRITE |
| `filter(event, payload)` | GATE chain (condition mode) before WRITE |
| `emit(event, payload)` | EMIT Node |
| `onPipeline(event, handler)` | Subgraph registered as subscription on event |
| `bus.onEmit(event, handler)` | Subgraph registered as subscription on event |
| `createRegistry()` | Nodes with a shared label + READ mode="query" |
| `defineModule().onRegister()` | Module registration subgraph |
| `MaterializerPipeline.execute()` | ITERATE + CALL chain |
| `MaterializerStep.execute()` | A named subgraph (CALLable) |
| `configureRoles/configurePermissions` | WRITE Nodes creating Role/Permission Nodes |
| `hasPermission()` | GATE mode="capability" |
| `resolveComposition()` | Composition rendering subgraph |
| `dispatchModuleRoute()` | HTTP entry point to module's subgraph |
| `runWorkflow()` | Transactional subgraph with WRITE + error compensation via ON_ERROR |
| `createMaterializerPipeline()` | Composing a subgraph from CALL steps |
| `evaluateCondition()` | TRANSFORM or GATE expression evaluation |
| `evaluateVisibility()` | BRANCH + GATE inside ITERATE |

---

## Open Design Questions

### Q1: Should TRANSFORM support user-defined functions?

Currently, TRANSFORM expressions are limited to the sandboxed evaluator (arithmetic, string ops, JSONPath navigation, built-in functions like `len()`, `now()`, `join()`). Should modules be able to register custom functions?

**Argument for:** Modules need domain-specific transforms (e.g., `calculateTax()`, `formatCurrency()`, `slugify()`).

**Argument against:** Custom functions break inspectability -- an AI agent cannot understand `calculateTax()` without reading its implementation. They also create a code-loading problem (Rust crate? WASM? QuickJS?).

**Proposed resolution:** Custom functions are subgraphs. `calculateTax` is a named subgraph that uses TRANSFORM primitives. The AI agent can read it. The engine evaluates it. No code loading needed. For performance-critical functions (cryptography, compression), the engine provides a small set of built-in functions that are hardcoded in Rust.

### Q2: How are subgraphs versioned and migrated?

When a module updates its subgraph (e.g., adding a new validation step), what happens to in-flight executions? Proposed: subgraphs are versioned via the engine's version chains. In-flight executions continue on the version they started. New triggers use the CURRENT version.

### Q3: Should ITERATE support early exit?

Currently ITERATE always processes all items (up to `maxIterations`). Should there be a way to break out early (e.g., "find the first item matching a condition")? Proposed: BRANCH inside the ITERATE BODY can produce a special `BREAK` signal that terminates iteration. This is a convention, not a new primitive.

### Q4: How deep should dry-run go?

The AI agent critique recommended a `dryRun` mode. With operation subgraphs, dry-run means: walk the subgraph, evaluate all GATEs and BRANCHes against the current state, collect all WRITEs that would occur, but do not commit them. EMITs are suppressed. CALLs to subgraphs are recursively dry-run. The engine returns: `{ wouldWrite: Node[], wouldEmit: Event[], gateResults: GateResult[], path: NodeId[] }`.

---

## Summary Table

| # | Primitive | Category | Irreducibility Argument |
|---|-----------|----------|------------------------|
| 1 | READ | Data access | Only way to observe the graph |
| 2 | WRITE | Data access | Only way to mutate the graph |
| 3 | TRANSFORM | Data transformation | Only way to reshape data without I/O |
| 4 | BRANCH | Control flow | Only way to conditionally route execution |
| 5 | ITERATE | Control flow | Only way to process collections (bounded) |
| 6 | GATE | Security / interception | Only way to intercept and validate/transform inline |
| 7 | CALL | Meta / composition | Only way to invoke a reusable subgraph |
| 8 | RESPOND | Communication | Only way to produce a final output |
| 9 | EMIT | Communication | Only way to produce a fire-and-forget side effect |
| 10 | WAIT | Control flow | Only way to suspend and resume execution |

10 primitives. 11 edge types. No Turing completeness. Every subgraph terminates. Every operation is inspectable. Every mutation is capability-gated.
