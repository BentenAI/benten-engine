# Operation Node Vocabulary -- Developer Experience Perspective

**Author:** Developer Experience Agent
**Date:** 2026-04-11
**Context:** Designing the operation Node types for the Benten graph execution engine, where code IS Nodes and Edges. This document proposes the vocabulary from the perspective of a module developer building real applications.

---

## Table of Contents

1. [Design Philosophy](#1-design-philosophy)
2. [The Honest Question: Graph vs TypeScript](#2-the-honest-question-graph-vs-typescript)
3. [Operation Node Taxonomy](#3-operation-node-taxonomy)
4. [The 18 Operation Node Types](#4-the-18-operation-node-types)
5. [Edge Types for Operations](#5-edge-types-for-operations)
6. [Common Subgraph Patterns](#6-common-subgraph-patterns)
7. [How Developers Create Subgraphs](#7-how-developers-create-subgraphs)
8. [How Developers Debug Subgraphs](#8-how-developers-debug-subgraphs)
9. [How Developers Test Subgraphs](#9-how-developers-test-subgraphs)
10. [The TypeScript Escape Hatch](#10-the-typescript-escape-hatch)
11. [DX Principles That Govern Everything](#11-dx-principles-that-govern-everything)

---

## 1. Design Philosophy

### The Module Developer's Mental Model

A module developer today writes code like this:

```typescript
// Thrum V3 — a route handler in the commerce module
export async function checkoutHandler(ctx: ModuleRouteContext): Promise<Response> {
  // 1. Validate input
  // 2. Check permissions
  // 3. Calculate price
  // 4. Charge payment
  // 5. Create order
  // 6. Send confirmation email
  // 7. Return response
}
```

This is a linear sequence of operations with error handling and branching. The engine needs to represent this same logic as a graph -- but it must not feel like a downgrade. The developer should think: "I can see my checkout flow. I can reuse the 'charge payment' step in the subscription renewal flow. I can insert a 'fraud check' step without rewriting the handler."

### What We Learned from Visual Programming Systems

| System | Strength | Weakness | Lesson |
|--------|----------|----------|--------|
| Unreal Blueprints | Excellent for simple behaviors, visual debugging | "Spaghetti" at scale, complex math is painful | Need a text escape hatch for complex logic |
| Node-RED | Great for IoT/integration flows, message passing | Limited for general-purpose programming | Good model for data-flow-oriented operations |
| LangGraph | State machine + graph for AI agents | Python-centric, state management overhead | State-passing between nodes is the right model |
| Temporal.io | Durable execution, compensation (saga) | Heavy infrastructure for simple flows | Compensation/rollback should be built into step nodes |
| Thrum Materializer | Composable pipeline, step insertion/removal | Linear only, no branching | Pipelines are subgraphs; subgraphs are pipelines + branches |
| Thrum Workflows | Sequential + compensation, clean DX | No parallelism, no branching | Must support parallel and conditional paths |

### The Two Audiences

The operation vocabulary must serve two audiences with different needs:

1. **Module developers** (90% of users) -- build features by composing existing operation types. They think in terms of "validate this, transform that, check permissions, write to store." They should never need to write Cypher or understand graph internals.

2. **Platform developers** (10% of users) -- build new operation types, custom validators, performance-critical paths. They understand the graph model and can work at the engine level.

The vocabulary is designed for audience 1. Audience 2 extends it.

---

## 2. The Honest Question: Graph vs TypeScript

### Where Graph Wins

| Scenario | Graph Advantage |
|----------|----------------|
| Multi-step flows with compensation | Compensation edges are declarative; no try/catch boilerplate |
| Reusable operation sequences | A "charge payment" subgraph is reused across checkout, subscription, refund |
| Visual debugging | See which node failed, what data flowed through each edge |
| Runtime modification | Insert a "fraud check" node without redeploying code |
| AI agent composition | An AI can construct and modify operation subgraphs at runtime |
| Parallel execution | Fork/join is a graph pattern, not nested Promise.all |
| Cross-module interception | Module B inserts a validation node into Module A's flow via graph edge |
| Audit trail | Every execution is a traversal with timestamps; history IS the graph |

### Where TypeScript Wins

| Scenario | TypeScript Advantage |
|----------|---------------------|
| Complex conditionals | `if (a && (b \|\| c) && d.nested?.field > threshold)` is one line, not 8 nodes |
| Mathematical computation | `total = items.reduce((sum, i) => sum + i.price * i.qty * (1 + taxRate), 0)` |
| String manipulation | Template literals, regex, parsing -- painful as graph nodes |
| Type-safe data transformation | `const response = { ...order, items: items.map(formatLineItem) }` |
| Iteration with early exit | `for (const item of items) { if (invalid(item)) return error; }` |
| Debugging complex logic | Breakpoints in a text editor beat clicking through graph nodes |
| Refactoring | Rename, extract function, inline -- text editors excel at this |
| Code review | Diffs of graph JSON are unreadable; diffs of TypeScript are clear |

### The Crossover Point

Graph-based execution becomes HARDER than TypeScript when:

1. **Branching depth exceeds 3 levels.** A checkout flow with 1-2 branches (payment method switch, inventory check) is fine. A tax calculation engine with 15 jurisdictional rules is not.

2. **Data transformation is the primary concern.** If a step is mostly reshaping data (mapping, filtering, reducing), a TypeScript function is more readable than a Transform node with a JSON path expression.

3. **The logic is algorithmic.** Sorting, searching, graph traversal algorithms, mathematical optimization -- these are text-code territory.

4. **Tight loops are needed.** Iterating over 1000 items with per-item logic is a function call, not 1000 node executions.

### The Rule of Thumb

**Graph nodes should be COARSE-GRAINED operations.** Each node represents a meaningful business step ("validate order," "charge payment," "send confirmation"), not a programming primitive ("add two numbers," "check if string contains substring"). A single operation node may execute dozens of lines of TypeScript internally.

This is the lesson from every successful visual programming system: the visual layer is for orchestration and composition. The text layer is for implementation.

---

## 3. Operation Node Taxonomy

Operations are organized into 6 categories based on what a module developer is trying to accomplish:

```
Operations
  |
  +-- Data Access (read/write/query the graph)
  |     Query, Read, Write, Delete
  |
  +-- Logic (make decisions, validate, transform)
  |     Validate, Transform, Branch, Gate
  |
  +-- Flow Control (manage execution order)
  |     Sequence, Parallel, Loop, Defer
  |
  +-- Integration (talk to external systems)
  |     Invoke, Notify, Webhook
  |
  +-- Safety (handle errors, compensate)
  |     Guard, Compensate
  |
  +-- Composition (reference other subgraphs)
        SubgraphRef
```

Total: **18 operation Node types.** This is deliberately small. More types can be added, but the initial vocabulary should be memorable. A developer should be able to list all 18 from memory within a week of use.

---

## 4. The 18 Operation Node Types

### 4.1 DATA ACCESS

#### `Query`

**What it does:** Reads data from the graph using a structured query (not Cypher). Returns a result set.

**Properties:**
```
label:      string          -- The Node label to query (e.g., "Product", "Order")
where:      Condition[]     -- Filter conditions (field, operator, value)
orderBy:    SortSpec[]      -- Sort order (field, direction)
limit:      number?         -- Maximum results
offset:     number?         -- Skip N results
fields:     string[]?       -- Project specific fields (default: all)
```

**Edges:**
- `NEXT` -> next operation (receives query results as input)
- `ON_EMPTY` -> alternative path when result set is empty

**Why not Cypher?** Module developers should not need to learn a query language for CRUD. The structured query maps directly to the existing Thrum `store.query()` pattern and generates IDE autocomplete on field names when paired with schema Nodes.

**Example -- list active products:**
```
[Query: label=Product, where=[{field: "status", op: "eq", value: "active"}],
        orderBy=[{field: "price", dir: "asc"}], limit=20]
  --NEXT--> [Transform: map output to API response shape]
```

---

#### `Read`

**What it does:** Reads a single Node by ID or unique field. Returns the Node or null.

**Properties:**
```
label:      string          -- The Node label
id:         string?         -- Direct ID lookup
lookupField: string?        -- Alternative unique field to look up by
lookupValue: string?        -- Value for the lookup field
```

**Edges:**
- `NEXT` -> next operation (receives the Node)
- `ON_NOT_FOUND` -> alternative path when the Node does not exist

**Why separate from Query?** Single-entity lookup is the most common operation. Making it a distinct type provides clearer semantics (returns one Node, not a result set), better error messaging ("Product X not found" vs "empty result set"), and optimizes to O(1) via IVM.

---

#### `Write`

**What it does:** Creates or updates a Node in the graph. Supports both create (no ID) and update (with ID) modes.

**Properties:**
```
label:      string          -- The Node label to create/update
mode:       "create" | "update" | "upsert"
data:       Record          -- The properties to write (can reference input via expressions)
version:    number?         -- Expected version for optimistic locking (update mode)
```

**Edges:**
- `NEXT` -> next operation (receives the written Node with its new ID/version)
- `ON_CONFLICT` -> alternative path for version conflicts (409)

**Why `data` uses expressions:** The data object can reference values from the input using path expressions: `{ title: "$input.title", slug: "$input.slug", status: "draft" }`. The `$input` prefix refers to whatever data the previous operation passed along the `NEXT` edge. This bridges the gap between static configuration and dynamic data flow without requiring a full programming language.

---

#### `Delete`

**What it does:** Removes a Node from the graph. Checks referential integrity.

**Properties:**
```
label:      string          -- The Node label
id:         string          -- ID of the Node to delete (can be expression: "$input.id")
cascade:    boolean?        -- Whether to cascade-delete edges (default: false)
softDelete: boolean?        -- Set a deletedAt property instead of hard delete (default: false)
```

**Edges:**
- `NEXT` -> next operation (receives deletion confirmation)
- `ON_REFERENCED` -> alternative path when the Node is referenced by other Nodes

---

### 4.2 LOGIC

#### `Validate`

**What it does:** Validates the input data against a schema. Rejects invalid data with structured error details.

**Properties:**
```
schema:     string          -- Reference to a schema Node ID (Valibot-compatible definition stored in graph)
mode:       "strict" | "strip" | "passthrough"  -- How to handle extra fields
```

**Edges:**
- `NEXT` -> next operation (receives validated/coerced data)
- `ON_INVALID` -> error path (receives validation error details: field paths, messages)

**Why schema is a Node reference, not inline:** Schemas are shared. A "Product" validation schema is used by the create, update, and import flows. Storing it as a Node in the graph means it is versioned, synced, and reusable. The `Validate` operation points to it via edge.

---

#### `Transform`

**What it does:** Reshapes data from one structure to another using a declarative mapping.

**Properties:**
```
mapping:    Record          -- Output field -> input expression
                            -- e.g., { "name": "$input.title", "total": "$input.price * $input.qty" }
merge:      boolean?        -- Merge output into input (default: false = replace)
```

**Edges:**
- `NEXT` -> next operation (receives transformed data)

**Expression language:** Uses the same sandboxed expression evaluator as `@benten/expressions` (jsep + custom AST walker). Simple property access, arithmetic, string concatenation, ternary. No function calls, no loops, no side effects.

**When it is not enough:** When the transformation is complex (nested maps, conditional logic, reduce), use a `Gate` node that delegates to a TypeScript function. The Transform node is for the 80% case of field renaming and simple computation.

---

#### `Branch`

**What it does:** Routes execution to one of several paths based on a condition.

**Properties:**
```
conditions: Array<{
  expression: string        -- Boolean expression (e.g., "$input.paymentMethod == 'stripe'")
  target:     string        -- ID of the target operation Node
}>
default:    string?         -- ID of the default target if no condition matches
```

**Edges:**
- Named `BRANCH:{conditionIndex}` edges to each target
- `BRANCH:default` edge to the default target

**Example -- payment method routing:**
```
[Branch: conditions=[
    {expr: "$input.method == 'stripe'", target: "charge-stripe"},
    {expr: "$input.method == 'paypal'", target: "charge-paypal"},
  ],
  default: "charge-manual"
]
  --BRANCH:0--> [Invoke: id=charge-stripe, ...]
  --BRANCH:1--> [Invoke: id=charge-paypal, ...]
  --BRANCH:default--> [Write: manual payment record]
```

---

#### `Gate`

**What it does:** The TypeScript escape hatch. Executes a registered TypeScript function and passes the result to the next operation.

**Properties:**
```
handler:    string          -- Reference to a registered handler function ID
                            -- (registered via module system: "commerce/calculateTax")
timeout:    number?         -- Maximum execution time in ms (default: 5000)
```

**Edges:**
- `NEXT` -> next operation (receives the handler's return value)
- `ON_ERROR` -> error path (receives the error)

**This is the critical DX escape hatch.** Every visual programming system that survives past toy projects has one. Unreal has C++ functions callable from Blueprints. Node-RED has Function nodes with inline JavaScript. LangGraph nodes ARE Python functions.

The Gate node says: "Here is where complex logic lives. It is a TypeScript function registered with the module system. The graph orchestrates WHEN it runs and WHAT data it receives. The function decides HOW."

**Handler registration:**
```typescript
// In module onRegister():
ctx.registerOperationHandler('commerce/calculateTax', async (input) => {
  // Complex tax calculation with jurisdiction rules
  const tax = calculateTaxForJurisdiction(input.address, input.items);
  return { ...input, tax, total: input.subtotal + tax };
});
```

---

### 4.3 FLOW CONTROL

#### `Sequence`

**What it does:** Groups multiple operations into an ordered sequence. This is the most basic flow control -- "do A, then B, then C."

**Properties:**
```
label:      string?         -- Human-readable label for the sequence
```

**Edges:**
- `FIRST` -> first operation in the sequence
- `NEXT` -> next operation after the sequence completes (receives the last step's output)

**Why a dedicated Sequence node?** Without it, every chain of operations is an implicit sequence defined purely by `NEXT` edges. The Sequence node makes the grouping explicit -- it is a named scope that can be referenced, compensated, and debugged as a unit. It is the graph equivalent of a function body.

---

#### `Parallel`

**What it does:** Executes multiple operations concurrently and waits for all to complete (fork-join).

**Properties:**
```
mode:       "all" | "any" | "settled"
            -- all: wait for all, fail if any fails (Promise.all)
            -- any: resolve with first success (Promise.any)
            -- settled: wait for all, never fail (Promise.allSettled)
timeout:    number?         -- Maximum wait time in ms
```

**Edges:**
- `FORK:{index}` edges to each parallel branch (0, 1, 2, ...)
- `NEXT` -> next operation (receives an array of results, one per branch)
- `ON_TIMEOUT` -> timeout path

**Example -- fetch data for a dashboard in parallel:**
```
[Parallel: mode=all]
  --FORK:0--> [Query: label=Order, where=[{field: "status", op: "eq", value: "pending"}]]
  --FORK:1--> [Query: label=Product, limit=5, orderBy=[{field: "sales", dir: "desc"}]]
  --FORK:2--> [Query: label=User, where=[{field: "createdAt", op: "gte", value: "$today"}]]
  --NEXT--> [Transform: merge results into dashboard shape]
```

---

#### `Loop`

**What it does:** Iterates over an array in the input, executing a subgraph for each item.

**Properties:**
```
over:       string          -- Expression for the array to iterate (e.g., "$input.items")
as:         string          -- Variable name for the current item (e.g., "item")
mode:       "sequential" | "parallel"   -- Execute iterations in order or concurrently
maxParallel: number?        -- Concurrency limit for parallel mode (default: 10)
```

**Edges:**
- `BODY` -> the operation to execute for each item
- `NEXT` -> next operation (receives array of per-item results)
- `ON_ERROR` -> error path (receives the item that failed + the error)

**When NOT to use Loop:** Avoid Loop for performance-critical paths with large arrays (>100 items). Use a Gate node with a TypeScript function instead. The Loop node is for small collections where each iteration involves graph operations (e.g., "for each line item, check inventory").

---

#### `Defer`

**What it does:** Schedules an operation to execute later -- after a delay, at a specific time, or as a background job.

**Properties:**
```
delay:      number?         -- Delay in milliseconds
at:         string?         -- ISO 8601 timestamp for scheduled execution
queue:      string?         -- Named queue for background processing (default: "default")
retries:    number?         -- Max retry count on failure (default: 0)
```

**Edges:**
- `DEFERRED` -> the operation to execute later
- `NEXT` -> next operation (receives a job ID immediately, does not wait)
- `ON_FAILURE` -> error path for when the deferred operation fails after all retries

**Example -- send confirmation email asynchronously:**
```
[Write: create order]
  --NEXT--> [Defer: queue="email"]
              --DEFERRED--> [Notify: template="order-confirmation", to="$input.email"]
  --NEXT--> [Transform: shape order response]
```

---

### 4.4 INTEGRATION

#### `Invoke`

**What it does:** Calls an external service or API. Handles serialization, deserialization, timeouts, and retries.

**Properties:**
```
service:    string          -- Service identifier (registered via module system)
method:     string          -- Method name on the service
input:      Record?         -- Data to pass (can use expressions)
timeout:    number?         -- Timeout in ms (default: 30000)
retries:    number?         -- Retry count (default: 0)
retryDelay: number?         -- Delay between retries in ms (default: 1000)
```

**Edges:**
- `NEXT` -> next operation (receives the service response)
- `ON_ERROR` -> error path (receives the error)
- `ON_TIMEOUT` -> timeout-specific error path

**Service registration:**
```typescript
// In module onRegister():
ctx.registerService('commerce/stripe', {
  chargeCard: async (input) => { /* Stripe API call */ },
  refund: async (input) => { /* Stripe refund */ },
  createCustomer: async (input) => { /* Stripe customer */ },
});
```

**Why Invoke instead of raw HTTP?** Module developers should not construct HTTP requests in graph nodes. The Invoke node references a registered service that encapsulates the HTTP/SDK details. The graph describes WHAT to call and WHEN, not HOW to make the HTTP request.

---

#### `Notify`

**What it does:** Sends a notification via a registered channel (email, push, in-app, webhook).

**Properties:**
```
channel:    string          -- Channel identifier (e.g., "email", "push", "webhook")
template:   string?         -- Template identifier for formatted messages
to:         string          -- Recipient expression (e.g., "$input.userEmail")
data:       Record?         -- Template data (can use expressions)
```

**Edges:**
- `NEXT` -> next operation (does not wait for delivery by default)
- `ON_ERROR` -> error path for delivery failures

**Why separate from Invoke?** Notification is so common in application development that it deserves its own node type. It encapsulates channel routing, template rendering, and delivery semantics. Every CMS, commerce, and social module sends notifications. Making it first-class means the graph visually shows "here is where the user gets notified."

---

#### `Webhook`

**What it does:** Receives an external webhook call and feeds it into a subgraph. This is the entry point for external systems to trigger graph execution.

**Properties:**
```
path:       string          -- URL path to listen on (e.g., "/webhooks/stripe")
method:     string          -- HTTP method to accept (default: "POST")
secret:     string?         -- Shared secret for signature verification
validate:   string?         -- Schema Node ID for payload validation
```

**Edges:**
- `NEXT` -> first operation in the handler subgraph

**This node is the entry point, not a mid-flow operation.** It is always the root of a subgraph. When an HTTP request arrives at the webhook path, the engine finds the Webhook node, validates the payload, and begins executing the connected subgraph.

---

### 4.5 SAFETY

#### `Guard`

**What it does:** Checks a capability or permission before allowing execution to continue. Fails fast with a structured error.

**Properties:**
```
capability: string          -- Capability to check (e.g., "store:write:products")
actor:      string?         -- Expression for the actor (default: "$context.user")
message:    string?         -- Custom error message on denial
```

**Edges:**
- `NEXT` -> next operation (only if the check passes)
- `ON_DENIED` -> denial path (receives structured denial with missing capability)

**Why Guard is not just a Branch:** Guard encodes a security invariant. It should be visually distinct in editors, auditable in logs, and enforceable by the engine. A Branch that checks permissions looks the same as a Branch that checks payment method. A Guard screams "THIS IS A SECURITY CHECK" to anyone reading the graph.

---

#### `Compensate`

**What it does:** Wraps a sequence of operations with automatic compensation (rollback) on failure. This is the saga pattern.

**Properties:**
```
label:      string?         -- Human-readable label
```

**Edges:**
- `BODY` -> the sequence of operations to execute
- `UNDO:{index}` edges to compensation operations (one per step, in reverse order)
- `NEXT` -> next operation after successful completion
- `ON_FAILURE` -> path taken after compensation completes (receives the original error + compensation results)

**How compensation works:**
1. Execute BODY steps in order (following NEXT edges within the body)
2. Each step can have a paired UNDO operation
3. If step N fails, execute UNDO for steps N-1, N-2, ..., 0 in reverse order
4. After compensation, follow ON_FAILURE edge

**Example -- checkout with compensation:**
```
[Compensate: label="Checkout"]
  --BODY--> [Invoke: commerce/stripe.chargeCard]   (UNDO:0 -> [Invoke: commerce/stripe.refund])
              --NEXT--> [Write: mode=update, label=Inventory, data={qty: "$input.qty - $item.qty"}]
                                                    (UNDO:1 -> [Write: restore inventory])
              --NEXT--> [Write: mode=create, label=Order, data={...}]
                                                    (UNDO:2 -> [Delete: label=Order])
  --NEXT--> [Notify: channel=email, template=order-confirmation]
  --ON_FAILURE--> [Notify: channel=email, template=checkout-failed]
```

This directly replaces the current Thrum workflow engine's sequential compensation model but makes it visual and graph-native.

---

### 4.6 COMPOSITION

#### `SubgraphRef`

**What it does:** References another subgraph by ID and executes it inline. This is the function call equivalent.

**Properties:**
```
subgraph:   string          -- ID of the subgraph to execute
inputMap:   Record?         -- Map input fields to the subgraph's expected input
outputMap:  Record?         -- Map subgraph output fields to this node's output
```

**Edges:**
- `NEXT` -> next operation (receives the mapped output)
- `ON_ERROR` -> error path

**Why SubgraphRef matters:** This is how reuse works. A "charge payment" subgraph is defined once and referenced by checkout, subscription renewal, manual charge, and refund retry flows. Without SubgraphRef, every flow would duplicate the payment charging logic.

**This mirrors CompositionRef in the existing CMS.** The page builder already has `compositionRef` for reusing block compositions. SubgraphRef applies the same pattern to operation logic.

---

## 5. Edge Types for Operations

Operations connect via typed edges. The edge types form a closed vocabulary:

| Edge Type | Meaning | From | To |
|-----------|---------|------|----|
| `NEXT` | Default flow (success path) | Any operation | Any operation |
| `ON_ERROR` | Error/failure path | Validate, Gate, Invoke, Loop, Compensate, SubgraphRef | Any operation |
| `ON_NOT_FOUND` | Entity not found | Read | Any operation |
| `ON_EMPTY` | Empty result set | Query | Any operation |
| `ON_INVALID` | Validation failed | Validate | Any operation |
| `ON_CONFLICT` | Version conflict | Write | Any operation |
| `ON_REFERENCED` | Cannot delete (FK) | Delete | Any operation |
| `ON_DENIED` | Permission denied | Guard | Any operation |
| `ON_TIMEOUT` | Operation timed out | Invoke, Parallel | Any operation |
| `ON_FAILURE` | Post-compensation | Compensate | Any operation |
| `BRANCH:{n}` | Conditional branch | Branch | Any operation |
| `BRANCH:default` | Default branch | Branch | Any operation |
| `FORK:{n}` | Parallel branch | Parallel | Any operation |
| `BODY` | Body of a wrapper | Sequence, Compensate, Loop | Any operation |
| `FIRST` | First in sequence | Sequence | Any operation |
| `DEFERRED` | Deferred execution | Defer | Any operation |
| `UNDO:{n}` | Compensation step | Compensate | Any operation |

### Edge Property: `transform`

Every edge can carry an optional `transform` property -- a lightweight field mapping that reshapes data as it flows between operations. This eliminates the need for explicit Transform nodes in simple cases:

```
[Query: label=Product] --NEXT {transform: {items: "$result.records", count: "$result.total"}}--> [Transform: ...]
```

---

## 6. Common Subgraph Patterns

### Pattern 1: CRUD Route (5 nodes)

The bread and butter of module development. Handles one REST endpoint.

```
[Guard: capability="store:read:products"]
  --NEXT--> [Validate: schema="ProductQuerySchema"]
    --NEXT--> [Query: label=Product, where=$input.filters]
      --NEXT--> [Transform: mapping={records: "$input.records", total: "$input.total"}]
        --NEXT--> [RESPONSE]
    --ON_INVALID--> [RESPONSE: 400, validation errors]
  --ON_DENIED--> [RESPONSE: 403]
```

This pattern is so common that it should have a **shorthand builder** (see Section 7).

### Pattern 2: Checkout Flow (8 nodes)

Multi-step with compensation, external service calls, and async notification.

```
[Guard: capability="commerce:checkout"]
  --NEXT--> [Validate: schema="CheckoutSchema"]
    --NEXT--> [Compensate: label="Checkout Transaction"]
                --BODY--> [Gate: handler="commerce/calculateTotal"]
                  --NEXT--> [Invoke: service="commerce/stripe", method="chargeCard"]
                    --NEXT--> [Write: mode=create, label=Order]
                --UNDO:0--> [noop]
                --UNDO:1--> [Invoke: service="commerce/stripe", method="refund"]
                --UNDO:2--> [Delete: label=Order]
              --NEXT--> [Defer: queue="email"]
                          --DEFERRED--> [Notify: channel=email, template="order-confirmation"]
              --ON_FAILURE--> [Notify: channel=email, template="checkout-failed"]
```

### Pattern 3: Content Lifecycle Hook (3 nodes)

An SEO module intercepts content creation to add scoring.

```
[TRIGGER: on content:afterCreate]
  --NEXT--> [Gate: handler="seo/scoreContent"]
    --NEXT--> [Write: mode=update, label=$input.nodeId, data={seoScore: "$result.score"}]
```

### Pattern 4: Real-time Game State Update (4 nodes)

Fast path -- validation + write + broadcast.

```
[Validate: schema="MoveSchema"]
  --NEXT--> [Gate: handler="chess/validateMove"]
    --NEXT--> [Write: mode=update, label=GameState, data=$result.newState]
      --NEXT--> [Notify: channel=broadcast, data={gameId: "$input.gameId", move: "$input.move"}]
    --ON_ERROR--> [RESPONSE: 400, "Invalid move"]
```

### Pattern 5: Data Import with Per-Item Processing (5 nodes)

Loop pattern for batch operations.

```
[Validate: schema="ImportSchema"]
  --NEXT--> [Loop: over="$input.rows", as="row", mode=parallel, maxParallel=5]
    --BODY--> [Validate: schema="ProductRowSchema"]
      --NEXT--> [Write: mode=upsert, label=Product, data="$row"]
      --ON_INVALID--> [Transform: mapping={error: "$error", row: "$index"}]
    --NEXT--> [Transform: mapping={imported: "$result.length", errors: "$errors"}]
```

---

## 7. How Developers Create Subgraphs

This is the most important DX question. Three interfaces, one graph.

### 7.1 TypeScript DSL (Primary Interface)

For module developers, this is the main way to define operation subgraphs. It reads like code but produces graph Nodes and Edges.

```typescript
import { subgraph, query, guard, validate, transform, write, invoke, notify, defer, compensate, gate, branch } from '@benten/engine/operations';

// Define a checkout flow
const checkoutFlow = subgraph('commerce/checkout', (flow) => {
  flow
    .guard('commerce:checkout')
    .validate('CheckoutSchema')
    .compensate('Checkout Transaction', (tx) => {
      tx
        .step(
          gate('commerce/calculateTotal'),
          { undo: () => { /* noop */ } }
        )
        .step(
          invoke('commerce/stripe', 'chargeCard'),
          { undo: invoke('commerce/stripe', 'refund') }
        )
        .step(
          write('Order', { mode: 'create', data: '$result' }),
          { undo: (ctx) => delete_('Order', ctx.orderId) }
        );
    })
    .defer('email', notify('email', 'order-confirmation', { to: '$input.email' }))
    .onFailure(notify('email', 'checkout-failed', { to: '$input.email' }));
});

// Register during module onRegister
ctx.registerSubgraph(checkoutFlow);
```

The DSL is:
- **Fluent** -- reads top to bottom like the execution order.
- **Typed** -- each builder method constrains what can follow it.
- **Produces Nodes and Edges** -- the `subgraph()` call returns a serializable graph fragment that is written to the engine.

### 7.2 Cypher (Power User Interface)

For platform developers who want direct graph manipulation.

```cypher
CREATE (g:Operation:Guard {id: 'checkout-guard', capability: 'commerce:checkout'})
CREATE (v:Operation:Validate {id: 'checkout-validate', schema: 'CheckoutSchema'})
CREATE (c:Operation:Compensate {id: 'checkout-compensate', label: 'Checkout Transaction'})
CREATE (g)-[:NEXT]->(v)
CREATE (v)-[:NEXT]->(c)
// ... etc
```

This is the escape hatch, not the default. Most module developers never write Cypher.

### 7.3 Visual Editor (Future Interface)

A node-based visual editor in the admin panel. Drag operations from a palette, connect them with edges, configure properties in a side panel. This is the long-term goal for non-developer users (content ops, marketing, business analysts).

The visual editor reads and writes the same graph Nodes that the DSL produces. There is no separate visual representation -- the graph IS the representation.

### 7.4 CRUD Shorthand

Since CRUD routes are so common, provide a high-level builder that generates the standard pattern:

```typescript
import { crud } from '@benten/engine/operations';

const productRoutes = crud('Product', {
  schema: 'ProductSchema',
  capability: 'store:products',
  list: { defaultSort: { field: 'createdAt', dir: 'desc' }, limit: 50 },
  create: { afterCreate: notify('email', 'product-created', { to: '$context.adminEmail' }) },
  update: { optimisticLocking: true },
  delete: { softDelete: true },
});

// Generates 4 subgraphs: list, get, create, update, delete
// Each follows the Guard -> Validate -> Operation -> Transform pattern
ctx.registerSubgraph(productRoutes);
```

One function call generates 20+ operation Nodes with all the standard edges, error handling, and response shaping. The developer can customize any part by accessing the generated subgraph and inserting/removing nodes.

---

## 8. How Developers Debug Subgraphs

### 8.1 Execution Trace

Every subgraph execution produces a trace -- a record of which nodes executed, what data flowed through each edge, how long each node took, and which path was chosen at each branch.

```typescript
const result = await engine.executeSubgraph('commerce/checkout', input, { trace: true });

console.log(result.trace);
// [
//   { node: 'checkout-guard', duration: '0.01ms', status: 'passed' },
//   { node: 'checkout-validate', duration: '0.12ms', status: 'passed', output: { ... } },
//   { node: 'checkout-compensate', duration: '245ms', status: 'passed', steps: [
//     { node: 'calculate-total', duration: '2ms', status: 'passed' },
//     { node: 'charge-stripe', duration: '230ms', status: 'passed' },
//     { node: 'create-order', duration: '13ms', status: 'passed' },
//   ]},
//   { node: 'defer-email', duration: '0.5ms', status: 'deferred', jobId: 'job-abc123' },
// ]
```

### 8.2 Breakpoints

In development mode, operations can be paused:

```typescript
engine.setBreakpoint('checkout-validate', {
  condition: '$input.total > 1000',  // Only break for large orders
  action: 'pause',                   // Pause execution, inspect in dev tools
});
```

When a breakpoint fires, the execution suspends and the dev tools show the current node, input data, and available edges. The developer can step through nodes one at a time, inspect data at each edge, or resume.

### 8.3 Error Messages

Every operation node type produces structured errors that include:

1. **Which node failed** (node ID + human-readable label)
2. **What the input was** (the data that arrived via the incoming edge)
3. **Why it failed** (structured error with code, not just a message string)
4. **What the developer can do** (actionable suggestion)

Example for a Validate failure:
```json
{
  "error": "VALIDATION_FAILED",
  "node": "checkout-validate",
  "nodeLabel": "Validate Checkout Input",
  "subgraph": "commerce/checkout",
  "details": [
    { "path": "email", "message": "Required field missing" },
    { "path": "items[0].quantity", "message": "Must be a positive integer" }
  ],
  "suggestion": "Check the input shape against schema 'CheckoutSchema'. Run: engine.getSchema('CheckoutSchema') to see expected fields."
}
```

### 8.4 Visual Trace (Admin Panel)

In the admin panel, a completed execution can be visualized as a highlighted graph. Green nodes executed successfully. Red nodes failed. Gray nodes were skipped (branch not taken). Blue nodes are deferred. Clicking a node shows its input/output data.

This is where graph execution provides a genuinely better debugging experience than text code. You can SEE the flow, not just read a stack trace.

---

## 9. How Developers Test Subgraphs

### 9.1 Unit Testing an Operation Node

Individual operations can be tested in isolation by providing input and asserting output:

```typescript
import { describe, it, expect } from 'vitest';
import { createTestEngine } from '@benten/engine/testing';

describe('Validate operation', () => {
  it('rejects invalid input', async () => {
    const engine = createTestEngine();
    engine.registerSchema('ProductSchema', productSchema);

    const result = await engine.executeNode(
      { type: 'Validate', schema: 'ProductSchema' },
      { name: '', price: -5 }  // invalid input
    );

    expect(result.status).toBe('error');
    expect(result.error.code).toBe('VALIDATION_FAILED');
    expect(result.error.details).toHaveLength(2);
  });
});
```

### 9.2 Integration Testing a Subgraph

Subgraphs are tested end-to-end with mocked services:

```typescript
describe('Checkout flow', () => {
  it('creates order and sends confirmation', async () => {
    const engine = createTestEngine();

    // Register the checkout subgraph
    engine.registerSubgraph(checkoutFlow);

    // Mock the Stripe service
    engine.mockService('commerce/stripe', {
      chargeCard: async (input) => ({ chargeId: 'ch_test123', amount: input.total }),
      refund: async () => ({ refunded: true }),
    });

    // Mock the notification channel
    const sentEmails = [];
    engine.mockChannel('email', (msg) => sentEmails.push(msg));

    // Execute
    const result = await engine.executeSubgraph('commerce/checkout', {
      email: 'test@example.com',
      items: [{ id: 'prod-1', qty: 2, price: 9.99 }],
      paymentMethod: 'stripe',
    });

    expect(result.success).toBe(true);
    expect(result.output.orderId).toBeDefined();
    // Deferred email -- flush the queue to test it
    await engine.flushDeferred();
    expect(sentEmails).toHaveLength(1);
    expect(sentEmails[0].template).toBe('order-confirmation');
  });

  it('compensates on payment failure', async () => {
    const engine = createTestEngine();
    engine.registerSubgraph(checkoutFlow);

    // Mock Stripe to fail
    engine.mockService('commerce/stripe', {
      chargeCard: async () => { throw new Error('Card declined'); },
    });

    const result = await engine.executeSubgraph('commerce/checkout', {
      email: 'test@example.com',
      items: [{ id: 'prod-1', qty: 1, price: 29.99 }],
    });

    expect(result.success).toBe(false);
    expect(result.compensated).toBe(true);
    // Verify no order was created (or was cleaned up)
  });
});
```

### 9.3 Snapshot Testing Subgraph Structure

The subgraph DSL produces a deterministic graph structure. Test it:

```typescript
it('checkout flow has expected structure', () => {
  const graph = checkoutFlow.serialize();
  expect(graph.nodes).toHaveLength(8);
  expect(graph.nodes.map(n => n.type)).toEqual([
    'Guard', 'Validate', 'Compensate', 'Gate', 'Invoke', 'Write', 'Defer', 'Notify'
  ]);
  expect(graph.edges.filter(e => e.type === 'UNDO')).toHaveLength(2);
});
```

---

## 10. The TypeScript Escape Hatch

### When to Use Gate vs When to Use Pure Graph

| Use graph operations when... | Use Gate (TypeScript) when... |
|------------------------------|------------------------------|
| The step is CRUD (read, write, delete) | The step is complex computation |
| The step is a security check | The step has algorithmic complexity |
| The step branches on a simple condition | The step iterates with complex per-item logic |
| The step calls an external service | The step does string/date/math manipulation |
| The step needs compensation | The step needs third-party library access |
| The step should be visible in the graph | The step is an implementation detail |
| The step might be modified at runtime | The step is stable and well-tested |
| An AI agent might need to understand the step | The step is opaque by design |

### The 70/30 Rule

For a typical module, approximately 70% of the logic lives in graph operations (CRUD, validation, branching, service calls, notifications) and 30% lives in Gate handlers (business logic, calculations, transformations). If a module has more than 50% Gate nodes, the developer is fighting the graph model -- they should consider whether a traditional TypeScript module is a better fit.

### Hybrid Modules

A module can expose BOTH graph subgraphs and traditional TypeScript APIs. The graph handles orchestration (checkout flow, content lifecycle, notification routing) while TypeScript handles computation (price calculation, SEO scoring, search indexing). They are complementary, not competing.

```typescript
export const commerceModule = {
  id: 'commerce',
  version: '1.0.0',

  // Graph subgraphs for orchestration
  subgraphs: [checkoutFlow, refundFlow, subscriptionRenewalFlow],

  // TypeScript handlers for computation
  handlers: {
    'commerce/calculateTotal': calculateTotal,
    'commerce/applyDiscounts': applyDiscounts,
    'commerce/calculateTax': calculateTax,
  },

  // Traditional TypeScript services
  services: {
    'commerce/stripe': stripeService,
  },
};
```

---

## 11. DX Principles That Govern Everything

### Principle 1: The Graph Should Disappear

The best DX is when the developer does not think about graphs. They think about business logic. The TypeScript DSL reads like a sequence of steps. The CRUD shorthand generates a standard pattern. The visual editor looks like a flowchart. The fact that these are Nodes and Edges in Apache AGE (or the future Benten engine) is an implementation detail.

### Principle 2: Operations Are Coarse-Grained

An operation node represents a meaningful business step, not a programming primitive. "Validate checkout input" is a node. "Check if string is empty" is NOT a node -- it is a condition expression inside a Validate or Branch node.

### Principle 3: Error Paths Are First-Class

Every operation type has explicit error edges (`ON_ERROR`, `ON_INVALID`, `ON_NOT_FOUND`, etc.). The developer is FORCED to think about what happens when things go wrong, because the edge types make errors visible. This is better than try/catch, where error handling is optional and invisible.

### Principle 4: The TypeScript Escape Hatch Is Not a Failure

Using a Gate node is the RIGHT choice for complex logic. The graph model is not trying to replace TypeScript. It is trying to make orchestration visible, composable, and inspectable. Complex computation stays in TypeScript where it belongs.

### Principle 5: Reuse Through Subgraph References

The `SubgraphRef` node is the most important node type for ecosystem growth. It means a module developer can publish a "Stripe checkout" subgraph, and other developers can reference it in their flows without copy-pasting nodes. This is the npm model applied to operation graphs.

### Principle 6: AI-Native by Design

Every operation subgraph is introspectable by AI agents. An AI can:
- Read a subgraph and explain what it does in natural language
- Suggest where to insert a new step (e.g., "add fraud detection before payment")
- Generate a subgraph from a natural language description
- Modify a subgraph at runtime based on user intent

This only works because operations are coarse-grained, well-named, and connected by typed edges. A graph of 8 nodes is comprehensible to an AI. A graph of 800 fine-grained "add number" nodes is not.

### Principle 7: Visual Debugging Is the Payoff

The single strongest argument for graph-based execution is visual debugging. When a checkout fails, the developer sees a graph with green nodes up to the failure point, a red node where it broke, and the exact data at each edge. This is strictly better than a stack trace. It justifies the overhead of graph-based programming for orchestration flows.

---

## Appendix A: Complete Node Type Reference

| # | Type | Category | Edges Out | Key Property |
|---|------|----------|-----------|--------------|
| 1 | Query | Data Access | NEXT, ON_EMPTY | label, where, orderBy |
| 2 | Read | Data Access | NEXT, ON_NOT_FOUND | label, id |
| 3 | Write | Data Access | NEXT, ON_CONFLICT | label, mode, data |
| 4 | Delete | Data Access | NEXT, ON_REFERENCED | label, id |
| 5 | Validate | Logic | NEXT, ON_INVALID | schema |
| 6 | Transform | Logic | NEXT | mapping |
| 7 | Branch | Logic | BRANCH:{n}, BRANCH:default | conditions |
| 8 | Gate | Logic | NEXT, ON_ERROR | handler |
| 9 | Sequence | Flow Control | FIRST, NEXT | label |
| 10 | Parallel | Flow Control | FORK:{n}, NEXT, ON_TIMEOUT | mode |
| 11 | Loop | Flow Control | BODY, NEXT, ON_ERROR | over, as, mode |
| 12 | Defer | Flow Control | DEFERRED, NEXT, ON_FAILURE | delay, queue |
| 13 | Invoke | Integration | NEXT, ON_ERROR, ON_TIMEOUT | service, method |
| 14 | Notify | Integration | NEXT, ON_ERROR | channel, template, to |
| 15 | Webhook | Integration | NEXT | path, method |
| 16 | Guard | Safety | NEXT, ON_DENIED | capability |
| 17 | Compensate | Safety | BODY, UNDO:{n}, NEXT, ON_FAILURE | label |
| 18 | SubgraphRef | Composition | NEXT, ON_ERROR | subgraph |

---

## Appendix B: Migration Path from Thrum V3

| Current Thrum Pattern | Becomes | Notes |
|----------------------|---------|-------|
| `store.query(table, opts)` | Query node | Same structured query model |
| `store.getRecord(table, id)` | Read node | Same single-entity lookup |
| `store.insertRecord(table, data)` | Write node (mode=create) | Same create semantics |
| `store.updateRecord(table, id, data, opts)` | Write node (mode=update) | Supports optimistic locking |
| `store.deleteRecord(table, id)` | Delete node | Cascade/softDelete options |
| `requirePermission(user, action, resource)` | Guard node | Capability string maps to current RBAC |
| Valibot schema validation | Validate node | Schema stored as graph Node |
| `pipeline('content:beforeCreate', data)` | Transform / Gate in a subgraph | Write interception via subgraph composition |
| `emit('content:afterCreate', data)` | Reactive subscription triggers subgraph | Engine notifies subscribers on data change |
| WorkflowStep with compensate | Compensate node | Same saga pattern, now visual |
| Module route handler | Subgraph triggered by Webhook or route match | Handler body becomes a subgraph |
| MaterializerPipeline | Sequence of operation nodes | Same composable pipeline, now graph-native |

---

## Appendix C: What This Document Does NOT Cover

1. **Engine internals** -- how operation nodes are stored, indexed, and executed at the Rust level. This is an engine implementation concern, not a DX concern.
2. **IVM integration** -- how operation results feed into materialized views. This is automatic and invisible to the developer.
3. **CRDT sync** -- how operation subgraphs sync between instances. Subgraph definitions sync like any other graph data.
4. **Capability grant semantics** -- Guard nodes check capabilities, but how capabilities are granted/attenuated is a separate specification.
5. **Visual editor design** -- the UI/UX of the node editor. This is a frontend concern.
6. **Performance benchmarks** -- execution time targets for each operation type.

These are important topics. They belong in their own specification documents. This document answers one question: **What operations does a module developer need, and how do they feel to use?**
