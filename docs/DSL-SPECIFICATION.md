# Benten Operation DSL -- Complete Specification

**Created:** 2026-04-11
**Status:** Design specification (pre-implementation)
**Package:** `@benten/engine/operations`
**Audience:** Module developers who compose operation subgraphs using TypeScript

> **⚠️ Primitive set revised 2026-04-14.** The DSL was designed around the original 12 primitives. After critic review, two primitives were dropped (**VALIDATE**, **GATE**) and two were added (**SUBSCRIBE**, **STREAM**). The authoritative primitive list is in [`ENGINE-SPEC.md`](ENGINE-SPEC.md) Section 3. Examples in this document that use VALIDATE or GATE are historical — see the migration notes below.
>
> **Migration:**
> - **VALIDATE** → compose from BRANCH (on schema predicate) + TRANSFORM (to format error) + RESPOND (with error) + error edges. Or let it register as a validation function hooked to the engine's 14 structural invariants.
> - **GATE** → capability checks use the `requires` property on any Node (engine-enforced automatic BRANCH). Custom validation logic uses TRANSFORM or SANDBOX.
> - **SUBSCRIBE** (new) → reactive change notification; IVM views, sync delta propagation, and event-driven handlers all compose on top.
> - **STREAM** (new) → partial output with back-pressure; replaces patterns that previously attempted to use RESPOND for streaming.
>
> A DSL rewrite against the revised primitives is a Phase 1 deliverable. Until then, treat occurrences of VALIDATE and GATE in examples as illustrative of the *pattern*, not the final API shape.

**Also pending (DX critic findings, Phase 1 scope):**
- Zero-config `crud('post')` path (currently requires `{schema, capability}`)
- Error catalog integration (see [`ERROR-CATALOG.md`](ERROR-CATALOG.md))
- Debug tooling: `.toMermaid()` method, evaluation trace
- TypeScript wrapper layer (`@benten/engine` over `@benten/engine-native`)

---

## Table of Contents

1. [Design Philosophy](#1-design-philosophy)
2. [The 12 Primitives in the DSL](#2-the-12-primitives-in-the-dsl)
3. [Fluent Builder API](#3-fluent-builder-api)
4. [CRUD Shorthand](#4-crud-shorthand)
5. [TRANSFORM Expression Language](#5-transform-expression-language)
6. [Error Handling Patterns](#6-error-handling-patterns)
7. [Branching and Iteration](#7-branching-and-iteration)
8. [SANDBOX (WASM) Integration](#8-sandbox-wasm-integration)
9. [Testing API](#9-testing-api)
10. [Python Equivalent API](#10-python-equivalent-api)
11. [DSL-to-Graph Compilation](#11-dsl-to-graph-compilation)
12. [Complete Handler Examples](#12-complete-handler-examples)

---

## 1. Design Philosophy

### Three Rules

1. **The DSL is the primary interface.** Module developers write TypeScript that reads like a sequence of steps. They never hand-craft Nodes and Edges. The DSL compiles to the operation graph at registration time.

2. **Every primitive is a function.** Each of the 12 operation types has a corresponding function in the DSL. Functions chain into a builder. The builder produces a serializable `Subgraph` that is written to the engine.

3. **Errors are explicit, not exceptional.** Every operation that can fail exposes a handler for its failure case. The developer sees the error path at the call site, not in a distant `catch` block.

### What the DSL Does NOT Do

- No runtime logic. The DSL describes structure. It executes at registration time to produce a graph.
- No TypeScript evaluation inside the chain. Expressions are strings, not closures. This is what makes the graph inspectable by AI agents and debuggable via visual trace.
- No implicit ordering. Every step is an explicit node. The chain reads top-to-bottom because the builder adds NEXT edges in call order.

### Import

```typescript
import {
  subgraph, crud,
  read, write, validate, transform, branch, iterate,
  wait, gate, call, respond, emit, sandbox,
} from '@benten/engine/operations';
```

All 12 primitives plus `subgraph` and `crud` are named exports from a single entry point.

---

## 2. The 12 Primitives in the DSL

Each primitive maps 1:1 to an operation Node type in the graph. The function signature is the DSL surface; the Node properties are the compilation target.

### Overview Table

| # | DSL Function | Graph Node Type | Purpose | Terminal? |
|---|-------------|----------------|---------|-----------|
| 1 | `read()` | READ | Retrieve data from the graph | No |
| 2 | `write()` | WRITE | Create, update, or delete data | No |
| 3 | `transform()` | TRANSFORM | Reshape data with expressions | No |
| 4 | `branch()` | BRANCH | Route to one of several paths | No |
| 5 | `iterate()` | ITERATE | Process a collection (bounded) | No |
| 6 | `wait()` | WAIT | Suspend until signal or timeout | No |
| 7 | `gate()` | GATE | Execute a registered TypeScript handler | No |
| 8 | `call()` | CALL | Execute another subgraph | No |
| 9 | `respond()` | RESPOND | Terminal: produce output | Yes |
| 10 | `emit()` | EMIT | Fire-and-forget event | No |
| 11 | `sandbox()` | SANDBOX | Execute code in WASM sandbox | No |
| 12 | `validate()` | VALIDATE | Check data against a schema | No |

---

### 2.1 `read(target, options?)`

Retrieve data from the graph. The `target` is either a Node label (for queries) or a Node ID (for single lookups).

```typescript
function read(target: string, options?: ReadOptions): ReadStep;

interface ReadOptions {
  /** Single node by ID. If set, returns one Node or triggers onNotFound. */
  id?: string | Expression;
  /** Lookup by a unique field instead of ID. */
  lookup?: { field: string; value: string | Expression };
  /** Filter conditions for queries (when no id/lookup). */
  where?: Record<string, any> | Expression;
  /** Sort order. */
  orderBy?: Record<string, 'asc' | 'desc'>;
  /** Maximum results. */
  limit?: number | Expression;
  /** Skip N results. */
  offset?: number | Expression;
  /** Project specific fields (default: all). */
  fields?: string[];
  /** Alias for the result in the pipeline context. Default: `$result`. */
  as?: string;
}
```

**Implied mode:** When `id` or `lookup` is provided, the engine uses single-node lookup (O(1) via IVM). Otherwise it uses a query.

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onNotFound(handler)` | `ON_NOT_FOUND` | Single lookup returns null |
| `.onEmpty(handler)` | `ON_EMPTY` | Query returns zero results |

**Examples:**

```typescript
// Single node by ID
read('post', { id: '$input.id' })

// Lookup by unique field
read('post', { lookup: { field: 'slug', value: '$input.slug' } })

// Query with filters
read('post', {
  where: { published: true, authorId: '$ctx.user.id' },
  orderBy: { createdAt: 'desc' },
  limit: 20,
})

// With error handling
read('post', { id: '$input.id' })
  .onNotFound(respond(404, { error: 'Post not found' }))
```

---

### 2.2 `write(target, options)`

Create, update, or delete data in the graph.

```typescript
function write(target: string, options: WriteOptions): WriteStep;

interface WriteOptions {
  /** Mutation type. Default: 'create'. */
  action?: 'create' | 'update' | 'delete' | 'upsert';
  /** Node ID for update/delete. */
  id?: string | Expression;
  /** Data to write. Expressions allowed in values. */
  data?: Record<string, any> | Expression;
  /** Expected version for optimistic locking (update). */
  version?: number | Expression;
  /** Alias for the result. Default: `$result`. */
  as?: string;
}
```

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onConflict(handler)` | `ON_CONFLICT` | Version mismatch on update |
| `.onNotFound(handler)` | `ON_NOT_FOUND` | Target does not exist for update/delete |

**Examples:**

```typescript
// Create
write('post', {
  data: {
    title: '$input.title',
    slug: '$input.slug',
    status: 'draft',
    createdAt: 'now()',
    updatedAt: 'now()',
  },
})

// Update with optimistic locking
write('post', {
  action: 'update',
  id: '$input.id',
  data: { title: '$input.title', updatedAt: 'now()' },
  version: '$input.version',
}).onConflict(respond(409, { error: 'Version conflict. Reload and retry.' }))

// Delete
write('post', { action: 'delete', id: '$input.id' })
  .onNotFound(respond(404, { error: 'Post not found' }))
```

---

### 2.3 `transform(expression)`

Pure data reshaping. No I/O, no graph access. Uses the TRANSFORM expression language (Section 5).

```typescript
function transform(expression: Expression | Record<string, Expression>): TransformStep;
function transform(expression: Expression | Record<string, Expression>, options?: TransformOptions): TransformStep;

interface TransformOptions {
  /** Merge output into existing context (true) or replace (false). Default: false. */
  merge?: boolean;
  /** Alias for the result. Default: `$result`. */
  as?: string;
}
```

**Two forms:**

```typescript
// Object construction -- the common case
transform({
  id: '$result.id',
  title: '$result.title',
  summary: 'truncate($result.content, 200)',
  date: 'formatDate($result.createdAt, "YYYY-MM-DD")',
})

// Single expression
transform('$result.items | filter(i => i.active) | map(i => i.name)')
```

---

### 2.4 `branch(conditions)`

Route execution to one of several paths based on conditions.

```typescript
function branch(conditions: BranchCondition[]): BranchStep;
function branch(condition: Expression): BranchStep; // shorthand: 2-way boolean

interface BranchCondition {
  /** Boolean expression to evaluate. */
  when: Expression;
  /** The step(s) to execute if this condition is true. */
  then: Step | Step[];
}
```

**Methods:**

| Method | Purpose |
|--------|---------|
| `.when(expr, ...steps)` | Add a conditional branch (alternative to constructor array) |
| `.otherwise(...steps)` | Default path when no condition matches |

**Examples:**

```typescript
// Boolean shorthand (2-way)
branch('$result != null')

// Multi-way
branch([
  { when: '$input.method === "stripe"', then: call('commerce/charge-stripe') },
  { when: '$input.method === "paypal"', then: call('commerce/charge-paypal') },
]).otherwise(respond(400, { error: 'Unsupported payment method' }))

// Fluent multi-way (equivalent)
branch()
  .when('$input.method === "stripe"', call('commerce/charge-stripe'))
  .when('$input.method === "paypal"', call('commerce/charge-paypal'))
  .otherwise(respond(400, { error: 'Unsupported payment method' }))
```

---

### 2.5 `iterate(source, body, options?)`

Bounded iteration over a collection. Each item is processed by the body steps.

```typescript
function iterate(source: Expression, body: Step | ((item: StepContext) => Step), options?: IterateOptions): IterateStep;

interface IterateOptions {
  /** Variable name for the current item. Default: '$item'. */
  as?: string;
  /** Variable name for the current index. Default: '$index'. */
  indexAs?: string;
  /** Maximum number of iterations. REQUIRED -- no default. */
  max: number;
  /** Execute iterations in parallel. Default: false. */
  parallel?: boolean;
  /** Concurrency limit for parallel mode. Default: 10. */
  maxConcurrency?: number;
  /** Alias for collected results. Default: '$results'. */
  collectAs?: string;
}
```

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onItemError(handler)` | `ON_ERROR` | A single iteration fails |
| `.onLimitExceeded(handler)` | `ON_LIMIT` | Collection exceeds `max` |

**Why `max` is required:** Operation subgraphs are not Turing complete. Every iteration must be bounded. The engine rejects subgraphs where ITERATE lacks a `max` property. This is a security invariant, not a convenience default.

**Examples:**

```typescript
// Sequential iteration
iterate('$cart.items', (item) =>
  gate('commerce/checkInventory', { itemId: '$item.id', qty: '$item.quantity' }),
  { max: 100, as: '$item' }
)

// Parallel iteration with limit
iterate('$input.imageUrls', (img) =>
  sandbox('image/resize', { url: '$item', width: 800 }),
  { max: 50, parallel: true, maxConcurrency: 5 }
)
```

---

### 2.6 `wait(options)`

Suspend execution until a signal arrives or a timeout expires.

```typescript
function wait(options: WaitOptions): WaitStep;

interface WaitOptions {
  /** Wait for a named signal. */
  signal?: string;
  /** Wait for a duration in milliseconds. */
  delay?: number;
  /** Wait until an ISO 8601 timestamp. */
  until?: string | Expression;
  /** Maximum wait time in milliseconds. */
  timeout?: number;
}
```

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onTimeout(handler)` | `ON_TIMEOUT` | Timeout expires before signal/time |

**Examples:**

```typescript
// Wait for external signal (e.g., payment webhook)
wait({ signal: 'payment:confirmed:$orderId', timeout: 300_000 })
  .onTimeout(call('commerce/cancelPendingOrder'))

// Scheduled delay (rate limiting, retry backoff)
wait({ delay: 5000 })

// Wait until a specific time
wait({ until: '$input.scheduledAt', timeout: 86_400_000 })
```

---

### 2.7 `gate(handler, args?)`

Execute a registered TypeScript handler function. This is the escape hatch for logic too complex for the expression language. One purpose only: run a function, get its result.

```typescript
function gate(handler: string, args?: Record<string, Expression>): GateStep;

// handler: a registered handler ID (e.g., 'commerce/calculateTax')
// args: optional argument mapping (expressions resolved before invocation)
```

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onError(handler)` | `ON_ERROR` | Handler threw an error |

**Handler registration (in module onRegister):**

```typescript
ctx.registerHandler('commerce/calculateTax', async (input, ctx) => {
  // Complex tax calculation -- multiple jurisdictions, rate lookups, exemptions
  const tax = calculateTaxForJurisdiction(input.address, input.items);
  return { ...input, tax, total: input.subtotal + tax };
});
```

**When to use GATE vs TRANSFORM:**

| Use TRANSFORM when... | Use GATE when... |
|-----------------------|-----------------|
| Reshaping fields (rename, project) | Business logic (tax, scoring, ranking) |
| Arithmetic on known fields | Algorithm with conditional logic |
| Built-in string/date functions suffice | Third-party library needed |
| The logic fits in one expression | The logic is 5+ lines of TypeScript |

**Examples:**

```typescript
// Simple handler call
gate('commerce/calculateTax')

// With argument mapping
gate('seo/scoreContent', {
  title: '$result.title',
  body: '$result.content',
  url: '$result.slug',
})

// With error handling
gate('payment/chargeCard', { token: '$input.paymentToken', amount: '$total' })
  .onError(respond(402, { error: 'Payment failed: $error.message' }))
```

---

### 2.8 `call(subgraphId, options?)`

Execute another subgraph. This is the function call equivalent for operation graphs. Enables reuse: a "charge payment" subgraph is defined once and called from checkout, subscription renewal, and refund retry flows.

```typescript
function call(subgraphId: string, options?: CallOptions): CallStep;

interface CallOptions {
  /** Map fields from current context to the subgraph's expected input. */
  input?: Record<string, Expression>;
  /** Map fields from subgraph output to the current context. */
  output?: Record<string, Expression>;
  /** Timeout in milliseconds. */
  timeout?: number;
  /** Run the subgraph with attenuated capabilities. */
  capabilities?: string[];
}
```

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onError(handler)` | `ON_ERROR` | Subgraph execution failed |
| `.onTimeout(handler)` | `ON_TIMEOUT` | Subgraph exceeded timeout |

**Examples:**

```typescript
// Simple call
call('commerce/charge-stripe')

// With input/output mapping
call('notifications/send-email', {
  input: { to: '$result.email', template: 'order-confirmation', data: '$result' },
})

// With capability attenuation
call('thirdparty/analytics', {
  input: { event: 'pageView', path: '$input.path' },
  capabilities: ['store:read:analytics/*'],  // narrow: only analytics reads
})
```

---

### 2.9 `respond(status, body?)`

Terminal node. Produces the subgraph's output. Every execution path must end with a `respond()`.

```typescript
function respond(status: number, body?: any | Expression): RespondStep;
function respond(status: number, body?: any | Expression, options?: RespondOptions): RespondStep;

interface RespondOptions {
  /** Response headers. */
  headers?: Record<string, string>;
  /** Response channel (default: 'http'). Use 'event' for event handler responses. */
  channel?: string;
}
```

**Examples:**

```typescript
// Success with body
respond(200, '$result')

// Created with transformed body
respond(201, { id: '$result.id', slug: '$result.slug' })

// Error with static message
respond(404, { error: 'Not found' })

// No content
respond(204)

// With custom headers
respond(200, '$result', { headers: { 'X-Total-Count': '$total' } })
```

---

### 2.10 `emit(event, payload?)`

Fire-and-forget event. Does not wait for handlers. The event is dispatched after the current operation completes. Execution continues along the NEXT edge immediately.

```typescript
function emit(event: string, payload?: Record<string, Expression> | Expression): EmitStep;
```

**Examples:**

```typescript
// Simple event
emit('content:afterCreate', { id: '$result.id', type: '$result.type' })

// Full payload passthrough
emit('commerce:orderCreated', '$result')

// Chained after write
write('post', { data: '$input' })
  // .emit() is also available as a chain method (syntactic sugar)
```

---

### 2.11 `sandbox(runtimeId, options)`

Execute code in a WASM sandbox (QuickJS-in-WASM). For untrusted code, AI-generated code, or computationally intensive operations that need isolation and fuel metering.

```typescript
function sandbox(runtimeId: string, options: SandboxOptions): SandboxStep;

interface SandboxOptions {
  /** Entry point function name within the WASM module. */
  entryPoint?: string;
  /** Arguments passed to the function (serialized to JSON across the boundary). */
  input?: Record<string, Expression>;
  /** Gas/fuel budget. Execution halts when exhausted. */
  fuel?: number;
  /** Memory limit in bytes. */
  memoryLimit?: number;
  /** Timeout in milliseconds. */
  timeout?: number;
  /** Capabilities granted to the sandbox (for host function callbacks). */
  capabilities?: string[];
}
```

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onError(handler)` | `ON_ERROR` | Runtime error or fuel exhaustion |
| `.onTimeout(handler)` | `ON_TIMEOUT` | Exceeded timeout |

**How the developer writes sandbox functions:**

Sandbox functions are plain TypeScript files compiled to WASM at build time. During development and testing, they run as native TypeScript (no WASM boundary). In production, they run inside QuickJS-in-WASM with fuel metering and capability membranes.

```typescript
// File: modules/ai/generate-content.ts
// This file is compiled to a WASM module at build time.
// During development, it runs natively in Node.js.

export function generateContent(input: { topic: string; style: string }): ContentResult {
  // Complex content generation logic
  // Can call host functions via the provided API:
  //   hostRead(label, id) -- reads from graph (if capabilities allow)
  //   hostWrite(label, data) -- writes to graph (if capabilities allow)
  const template = getTemplateForStyle(input.style);
  return {
    title: formatTitle(input.topic),
    body: applyTemplate(template, input.topic),
    status: 'draft',
  };
}
```

**Registration:**

```typescript
// In module onRegister():
ctx.registerSandbox('ai/generateContent', {
  source: './generate-content.ts',  // compiled to WASM at build
  entryPoint: 'generateContent',
  defaultFuel: 100_000,
  defaultTimeout: 30_000,
  requiredCapabilities: ['store:read:content/*'],
});
```

**Examples:**

```typescript
// AI content generation
sandbox('ai/generateContent', {
  input: { topic: '$input.topic', style: '$input.style' },
  fuel: 100_000,
  timeout: 30_000,
})

// Image processing (CPU-intensive, needs isolation)
sandbox('media/resizeImage', {
  input: { url: '$item.url', width: 800, quality: 80 },
  fuel: 500_000,
  memoryLimit: 64 * 1024 * 1024,  // 64MB
  timeout: 10_000,
})

// Third-party untrusted plugin code
sandbox('thirdparty/process', {
  input: { data: '$result' },
  fuel: 10_000,
  capabilities: ['store:read:content/post'],  // minimal capabilities
}).onError(respond(500, { error: 'Plugin execution failed' }))
```

---

### 2.12 `validate(schema, options?)`

Check data against a declared schema. Produces structured field-level errors on failure. The schema is either a reference to a schema Node in the graph or an inline Valibot-compatible definition.

```typescript
function validate(schema: string | SchemaDefinition, options?: ValidateOptions): ValidateStep;

interface ValidateOptions {
  /** What to validate. Default: '$input' (the current input). */
  target?: Expression;
  /** How to handle extra fields. Default: 'strip'. */
  mode?: 'strict' | 'strip' | 'passthrough';
}
```

**Error edges:**

| Method | Edge Type | When |
|--------|-----------|------|
| `.onInvalid(handler)` | `ON_INVALID` | Validation failed (receives field-level errors) |

**Examples:**

```typescript
// Validate against a content type schema stored in the graph
validate('contentType:post')
  .onInvalid(respond(400, { error: 'Validation failed', details: '$error.fields' }))

// Validate with strict mode (reject unknown fields)
validate('contentType:post', { mode: 'strict' })

// Validate a specific part of the context
validate('CheckoutSchema', { target: '$input.shippingAddress' })
```

---

## 3. Fluent Builder API

The `subgraph()` function creates a named builder that chains operations into a linear flow. For branching, iteration, and parallel execution, the builder nests.

### 3.1 Basic Chain

```typescript
const listPosts = subgraph('GET /api/posts')
  .require('store:read:post/*')
  .read('post', {
    where: { published: true },
    orderBy: { createdAt: 'desc' },
    limit: 20,
  })
  .transform({
    items: '$result',
    total: 'len($result)',
  })
  .respond(200);
```

### 3.2 Chain Methods

The builder supports every primitive as a chain method. Each call appends a Node and a NEXT edge.

```typescript
interface SubgraphBuilder {
  // -- Capability --
  /** Require a capability. Adds a capability check at the current point. */
  require(capability: string): this;

  // -- Data --
  read(target: string, options?: ReadOptions): ReadChain;
  write(target: string, options: WriteOptions): WriteChain;

  // -- Logic --
  validate(schema: string | SchemaDefinition, options?: ValidateOptions): ValidateChain;
  transform(expression: Expression | Record<string, Expression>, options?: TransformOptions): this;
  branch(conditions: BranchCondition[]): BranchChain;
  branch(condition: Expression): BranchChain;

  // -- Flow --
  iterate(source: Expression, body: Step | ((item: StepContext) => Step), options: IterateOptions): IterateChain;
  wait(options: WaitOptions): WaitChain;

  // -- Execution --
  gate(handler: string, args?: Record<string, Expression>): GateChain;
  call(subgraphId: string, options?: CallOptions): CallChain;
  sandbox(runtimeId: string, options: SandboxOptions): SandboxChain;

  // -- Output --
  respond(status: number, body?: any | Expression, options?: RespondOptions): SubgraphResult;
  emit(event: string, payload?: Record<string, Expression> | Expression): this;

  // -- Meta --
  /** Label this subgraph for debugging/tracing. */
  label(name: string): this;
  /** Set the overall timeout for the subgraph. */
  timeout(ms: number): this;
  /** Compile the builder into a Subgraph (Nodes + Edges). */
  compile(): Subgraph;
}
```

### 3.3 The `.require()` Method

Capability checking is a cross-cutting concern. Rather than manually inserting GATE nodes for every capability check, `.require()` adds a capability check at the current point in the chain. If the check fails, the engine follows the ON_DENIED edge to a default 403 response (or a custom handler if provided).

```typescript
subgraph('POST /api/posts')
  .require('store:write:post/*')             // capability check here
  .require('content:create')                  // AND another capability
  .validate('contentType:post')
  .write('post', { data: '$input' })
  .respond(201);
```

This compiles to two GATE Nodes at the start of the subgraph, each with `ON_DENIED -> RESPOND(403)`.

`.require()` can also be used mid-chain for operations that need additional capabilities:

```typescript
subgraph('POST /api/posts/publish')
  .require('store:write:post/*')
  .read('post', { id: '$input.id' })
  .require('content:publish')                 // additional check before publish
  .write('post', { action: 'update', id: '$input.id', data: { status: 'published' } })
  .respond(200);
```

### 3.4 Error Edge Methods

Every chainable step that can fail returns an enriched chain with error-handling methods. These methods attach edges to alternate paths:

```typescript
// ReadChain extends the base chain with onNotFound/onEmpty
interface ReadChain extends SubgraphBuilder {
  onNotFound(...steps: Step[]): this;
  onNotFound(step: Step): this;
  onEmpty(...steps: Step[]): this;
  onEmpty(step: Step): this;
}

// WriteChain extends with onConflict/onNotFound
interface WriteChain extends SubgraphBuilder {
  onConflict(...steps: Step[]): this;
  onNotFound(...steps: Step[]): this;
}

// ValidateChain extends with onInvalid
interface ValidateChain extends SubgraphBuilder {
  onInvalid(...steps: Step[]): this;
}

// GateChain/SandboxChain extends with onError
interface GateChain extends SubgraphBuilder {
  onError(...steps: Step[]): this;
}

// WaitChain extends with onTimeout
interface WaitChain extends SubgraphBuilder {
  onTimeout(...steps: Step[]): this;
}
```

Error handlers are optional. If not provided, the engine uses fail-closed defaults:
- `.onNotFound()` defaults to `respond(404, { error: '{target} not found' })`
- `.onInvalid()` defaults to `respond(400, { error: 'Validation failed', details: '$error.fields' })`
- `.onConflict()` defaults to `respond(409, { error: 'Version conflict' })`
- `.onError()` defaults to `respond(500, { error: 'Internal error' })`
- `.onTimeout()` defaults to `respond(504, { error: 'Timeout' })`

---

## 4. CRUD Shorthand

The single most important DX feature. One function call generates 5 complete handler subgraphs (list, get, create, update, delete) with proper validation, error handling, capability checks, event emission, and response shaping.

### 4.1 Signature

```typescript
function crud(label: string, options: CrudOptions): CrudHandlers;

interface CrudOptions {
  /** Schema reference for validation. */
  schema: string;
  /** Capability base string. Will be expanded to action-specific capabilities. */
  capability: string;
  /** Content type fields for response shaping (optional -- uses schema fields if not provided). */
  fields?: string[];

  /** List endpoint options. */
  list?: {
    /** Default sort. */
    sort?: Record<string, 'asc' | 'desc'>;
    /** Default page size. */
    limit?: number;
    /** Maximum allowed page size. */
    maxLimit?: number;
    /** Additional where clause applied to all list queries. */
    where?: Record<string, any>;
    /** Custom transform for list items. */
    transform?: Record<string, Expression>;
  };

  /** Create endpoint options. */
  create?: {
    /** Hooks to run after create (before response). */
    after?: Step[];
    /** Default values merged into input. */
    defaults?: Record<string, any>;
    /** Event name to emit. Default: '{label}:afterCreate'. */
    event?: string | false;
  };

  /** Update endpoint options. */
  update?: {
    /** Enable optimistic locking via version field. Default: true. */
    optimisticLocking?: boolean;
    /** Hooks to run after update. */
    after?: Step[];
    /** Event name to emit. Default: '{label}:afterUpdate'. */
    event?: string | false;
  };

  /** Delete endpoint options. */
  delete?: {
    /** Use soft delete (set deletedAt) instead of hard delete. Default: false. */
    soft?: boolean;
    /** Hooks to run after delete. */
    after?: Step[];
    /** Event name to emit. Default: '{label}:afterDelete'. */
    event?: string | false;
  };

  /** Add timestamps (createdAt, updatedAt) automatically. Default: true. */
  timestamps?: boolean;
}

interface CrudHandlers {
  /** All 5 subgraphs as an array (for bulk registration). */
  all: Subgraph[];
  /** Individual subgraphs for selective registration or customization. */
  list: Subgraph;
  get: Subgraph;
  create: Subgraph;
  update: Subgraph;
  delete: Subgraph;
}
```

### 4.2 Usage

```typescript
const postHandlers = crud('post', {
  schema: 'contentType:post',
  capability: 'store:post',
  list: {
    sort: { createdAt: 'desc' },
    limit: 20,
  },
  create: {
    defaults: { status: 'draft' },
    after: [emit('content:afterCreate')],
  },
  update: {
    optimisticLocking: true,
  },
  delete: {
    soft: true,
  },
});

// Register all 5
ctx.registerSubgraphs(postHandlers.all);

// Or register selectively
ctx.registerSubgraph(postHandlers.list);
ctx.registerSubgraph(postHandlers.get);
```

### 4.3 What It Generates

For `crud('post', { schema: 'contentType:post', capability: 'store:post', timestamps: true })`, the following 5 subgraphs are generated:

**List (GET /api/posts):**

```
GATE[require store:read:post/*]
  -> READ[query post, where=$query.where, orderBy=$query.sort, limit=$query.limit, offset=$query.offset]
    -> TRANSFORM[{items: $result, total: len($result)}]
      -> RESPOND[200]
  ON_DENIED -> RESPOND[403]
```

**Get (GET /api/posts/:id):**

```
GATE[require store:read:post/*]
  -> READ[node post, id=$params.id]
    -> RESPOND[200, $result]
    ON_NOT_FOUND -> RESPOND[404]
  ON_DENIED -> RESPOND[403]
```

**Create (POST /api/posts):**

```
GATE[require store:create:post/*]
  -> VALIDATE[contentType:post]
    -> TRANSFORM[merge $input + {createdAt: now(), updatedAt: now()}]
      -> WRITE[create post, data=$result]
        -> EMIT[post:afterCreate]
          -> RESPOND[201, $result]
    ON_INVALID -> RESPOND[400, $error]
  ON_DENIED -> RESPOND[403]
```

**Update (PUT /api/posts/:id):**

```
GATE[require store:update:post/*]
  -> READ[node post, id=$params.id]
    -> VALIDATE[contentType:post]
      -> TRANSFORM[merge $input + {updatedAt: now()}]
        -> WRITE[update post, id=$params.id, data=$result, version=$input.version]
          -> EMIT[post:afterUpdate]
            -> RESPOND[200, $result]
        ON_CONFLICT -> RESPOND[409]
      ON_INVALID -> RESPOND[400, $error]
    ON_NOT_FOUND -> RESPOND[404]
  ON_DENIED -> RESPOND[403]
```

**Delete (DELETE /api/posts/:id):**

```
GATE[require store:delete:post/*]
  -> READ[node post, id=$params.id]
    -> WRITE[delete post, id=$params.id]
      -> EMIT[post:afterDelete]
        -> RESPOND[200]
    ON_NOT_FOUND -> RESPOND[404]
  ON_DENIED -> RESPOND[403]
```

**Total: 5 subgraphs, ~30 Nodes, ~35 Edges.** All generated from one `crud()` call.

### 4.4 Customization

The generated subgraphs are standard `Subgraph` objects. Developers can modify them before registration:

```typescript
const postHandlers = crud('post', { schema: 'contentType:post', capability: 'store:post' });

// Insert a slug generation step before write in the create handler
postHandlers.create.insertBefore('WRITE',
  gate('content/generateUniqueSlug', { title: '$input.title' })
);

// Add a search index update after the create event
postHandlers.create.insertAfter('EMIT',
  call('search/indexDocument', { input: { id: '$result.id', type: 'post' } })
);

ctx.registerSubgraphs(postHandlers.all);
```

---

## 5. TRANSFORM Expression Language

The expression language used in TRANSFORM nodes (and in `where`, `data`, and other expression-accepting properties across all primitives). It is a pure, sandboxed subset of JavaScript with extensions for data manipulation.

### 5.1 Design Principles

1. **Pure.** No I/O, no graph access, no side effects. Expressions transform data. Period.
2. **Sandboxed.** Runs in a restricted evaluator (jsep AST + custom walker). No `eval`, no `Function`, no prototype access.
3. **Inspectable.** Every expression is a string that an AI agent can read and reason about. No opaque closures.
4. **Total.** Every expression terminates. No loops, no recursion, no unbounded computation. Array built-ins operate on bounded collections (the ITERATE node's `max` bounds the input).

### 5.2 Context Variables

Expressions access data through named context variables. These are the "registers" of the expression language.

| Variable | Type | Description |
|----------|------|-------------|
| `$input` | `Value` | The original request input (body, query params) |
| `$result` | `Value` | The output of the previous operation |
| `$ctx` | `Object` | Execution context: `$ctx.user`, `$ctx.tenant`, `$ctx.params`, `$ctx.query` |
| `$item` | `Value` | Current item during ITERATE (configurable via `as`) |
| `$index` | `number` | Current index during ITERATE |
| `$error` | `Object` | Error details on error edges: `$error.code`, `$error.message`, `$error.fields` |
| `$results` | `Value[]` | Collected results from ITERATE (configurable via `collectAs`) |

Custom aliases are created by the `as` option on any step:

```typescript
read('user', { id: '$input.authorId', as: '$author' })
// Now $author is available in subsequent expressions
```

### 5.3 Operators

**Arithmetic:**

| Operator | Example | Description |
|----------|---------|-------------|
| `+` | `$item.price + $item.tax` | Addition (numbers or string concatenation) |
| `-` | `$item.price - $item.discount` | Subtraction |
| `*` | `$item.price * $item.quantity` | Multiplication |
| `/` | `$total / len($items)` | Division |
| `%` | `$index % 2` | Modulo |

**Comparison:**

| Operator | Example |
|----------|---------|
| `===` | `$result.status === 'published'` |
| `!==` | `$result.type !== 'draft'` |
| `>` | `$result.price > 100` |
| `>=` | `$result.quantity >= 1` |
| `<` | `$result.age < 18` |
| `<=` | `$result.rating <= 5` |

**Logical:**

| Operator | Example |
|----------|---------|
| `&&` | `$result.active && $result.verified` |
| `\|\|` | `$result.title \|\| 'Untitled'` |
| `!` | `!$result.deleted` |
| `??` | `$result.nickname ?? $result.name ?? 'Anonymous'` |

**Ternary:**

```
$result.status === 'published' ? 'Live' : 'Draft'
```

**Optional chaining:**

```
$result.author?.name
$result.tags?.[0]
$result.metadata?.seo?.title ?? $result.title
```

### 5.4 Property Access

```
$result.title                    // dot access
$result.author.name              // nested
$result.tags[0]                  // array index
$result['field-with-dashes']     // bracket access
$result.items[0].name            // chained
```

Blocked keys: `__proto__`, `constructor`, `prototype`, `toString`, `valueOf`. Access to these returns `undefined` silently (fail-closed, no error thrown).

### 5.5 Object Construction

Object literals are supported. This is the primary mechanism for TRANSFORM output shaping.

```
// Object literal
{ id: $result.id, title: $result.title, slug: $result.slug }

// Nested objects
{ post: { id: $result.id, title: $result.title }, author: $author.name }

// With computed values
{ total: $item.price * $item.quantity, tax: $item.price * 0.1 }

// Spread (merge objects)
{ ...$input, updatedAt: now(), status: 'published' }
```

**Spread constraints:** Only one level of spread. No nested spread. No computed spread keys. The spread must reference a context variable or a property access, not an arbitrary expression.

### 5.6 Array Literals

```
// Array literal
[$result.id, $result.title, $result.slug]

// Mixed types
[1, 'hello', true, $result.id]
```

### 5.7 Built-in Functions

Functions are safe, re-implemented (never called on actual objects), and pure.

**Array functions (called as pipe operators or method syntax):**

| Function | Signature | Description |
|----------|-----------|-------------|
| `len(arr)` | `(arr: any[]) -> number` | Array length |
| `filter(arr, predicate)` | `(arr: T[], pred: string) -> T[]` | Filter elements |
| `map(arr, mapper)` | `(arr: T[], mapper: string) -> U[]` | Transform elements |
| `find(arr, predicate)` | `(arr: T[], pred: string) -> T \| null` | First match |
| `some(arr, predicate)` | `(arr: T[], pred: string) -> boolean` | Any match |
| `every(arr, predicate)` | `(arr: T[], pred: string) -> boolean` | All match |
| `includes(arr, value)` | `(arr: T[], val: T) -> boolean` | Contains value |
| `flat(arr)` | `(arr: T[][]) -> T[]` | Flatten one level |
| `sort(arr, field, dir?)` | `(arr: T[], field: string, dir?: 'asc'\|'desc') -> T[]` | Sort by field |
| `unique(arr, field?)` | `(arr: T[], field?: string) -> T[]` | Deduplicate |
| `slice(arr, start, end?)` | `(arr: T[], start: number, end?: number) -> T[]` | Subarray |
| `reverse(arr)` | `(arr: T[]) -> T[]` | Reverse order |
| `first(arr)` | `(arr: T[]) -> T \| null` | First element |
| `last(arr)` | `(arr: T[]) -> T \| null` | Last element |

**Aggregate functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `sum(arr, field?)` | `(arr: number[] \| T[], field?: string) -> number` | Sum values |
| `avg(arr, field?)` | `(arr: number[] \| T[], field?: string) -> number` | Average |
| `min(arr, field?)` | `(arr: number[] \| T[], field?: string) -> number` | Minimum |
| `max(arr, field?)` | `(arr: number[] \| T[], field?: string) -> number` | Maximum |
| `count(arr, predicate?)` | `(arr: T[], pred?: string) -> number` | Count (optionally filtered) |
| `groupBy(arr, field)` | `(arr: T[], field: string) -> Record<string, T[]>` | Group by field value |

**String functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `lower(str)` | `(str: string) -> string` | Lowercase |
| `upper(str)` | `(str: string) -> string` | Uppercase |
| `trim(str)` | `(str: string) -> string` | Trim whitespace |
| `contains(str, sub)` | `(str: string, sub: string) -> boolean` | Contains substring |
| `startsWith(str, prefix)` | `(str: string, prefix: string) -> boolean` | Starts with |
| `endsWith(str, suffix)` | `(str: string, suffix: string) -> boolean` | Ends with |
| `replace(str, search, rep)` | `(str: string, search: string, rep: string) -> string` | Replace first |
| `replaceAll(str, search, rep)` | `(str: string, search: string, rep: string) -> string` | Replace all |
| `split(str, separator)` | `(str: string, sep: string) -> string[]` | Split to array |
| `join(arr, separator)` | `(arr: string[], sep: string) -> string` | Join array |
| `truncate(str, maxLen)` | `(str: string, maxLen: number) -> string` | Truncate with ellipsis |
| `slugify(str)` | `(str: string) -> string` | URL-safe slug |
| `padStart(str, len, fill?)` | `(str: string, len: number, fill?: string) -> string` | Left-pad |
| `padEnd(str, len, fill?)` | `(str: string, len: number, fill?: string) -> string` | Right-pad |

**Number functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `round(n, decimals?)` | `(n: number, d?: number) -> number` | Round |
| `floor(n)` | `(n: number) -> number` | Floor |
| `ceil(n)` | `(n: number) -> number` | Ceiling |
| `abs(n)` | `(n: number) -> number` | Absolute value |
| `clamp(n, min, max)` | `(n: number, min: number, max: number) -> number` | Clamp to range |

**Date functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `now()` | `() -> string` | Current ISO 8601 timestamp |
| `formatDate(date, fmt)` | `(date: string, fmt: string) -> string` | Format date |
| `parseDate(str)` | `(str: string) -> string` | Parse to ISO 8601 |
| `addDays(date, n)` | `(date: string, n: number) -> string` | Add days |
| `addHours(date, n)` | `(date: string, n: number) -> string` | Add hours |
| `diffDays(a, b)` | `(a: string, b: string) -> number` | Days between |
| `isAfter(a, b)` | `(a: string, b: string) -> boolean` | Date comparison |
| `isBefore(a, b)` | `(a: string, b: string) -> boolean` | Date comparison |

**Utility functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `coalesce(a, b, ...)` | `(...values: any[]) -> any` | First non-null |
| `typeOf(val)` | `(val: any) -> string` | Type name |
| `keys(obj)` | `(obj: object) -> string[]` | Object keys |
| `values(obj)` | `(obj: object) -> any[]` | Object values |
| `entries(obj)` | `(obj: object) -> [string, any][]` | Key-value pairs |
| `fromEntries(arr)` | `(arr: [string, any][]) -> object` | Pairs to object |
| `merge(a, b, ...)` | `(...objs: object[]) -> object` | Shallow merge |
| `pick(obj, ...keys)` | `(obj: object, ...keys: string[]) -> object` | Pick fields |
| `omit(obj, ...keys)` | `(obj: object, ...keys: string[]) -> object` | Omit fields |
| `uuid()` | `() -> string` | Generate UUID v7 |

### 5.8 Lambda Expressions in Array Functions

Array functions that take predicates or mappers accept a restricted lambda syntax:

```
// Filter with lambda
filter($result.items, i => i.active)
filter($result.items, i => i.price > 10 && i.inStock)

// Map with lambda
map($result.items, i => { id: i.id, name: i.name, total: i.price * i.qty })

// Find
find($result.items, i => i.slug === $input.slug)

// Some/Every
some($result.items, i => i.quantity <= 0)
every($result.items, i => i.validated)
```

**Lambda constraints:**
- Single parameter only (no destructuring)
- Body must be a single expression (no statements, no blocks)
- Cannot reference other lambdas (no higher-order functions)
- Cannot call functions that take lambdas (no nested map/filter)
- Maximum body depth: 5 AST levels

These constraints keep lambdas inspectable and total. An AI agent can read `filter($items, i => i.price > 10)` and understand it completely. Nested `map(filter(...), ...)` is allowed because it is two separate function calls, not a lambda within a lambda.

### 5.9 What the Expression Language Does NOT Support

| Feature | Why Not | Alternative |
|---------|---------|-------------|
| Variable declaration (`let`, `const`) | Would create state | Use `as` option on steps |
| Loops (`for`, `while`) | Unbounded computation | Use ITERATE node |
| Function definition | Would create closures | Use GATE handler |
| `async`/`await` | Expressions are synchronous | Graph handles async via edges |
| `try`/`catch` | Expressions cannot fail (fail-closed) | Error edges on nodes |
| `new` | Object construction syntax differs | Use `{ }` object literals |
| `import`/`require` | No module system | Built-in functions only |
| `this` | No context binding | Use `$ctx` |
| Regular expressions | Complexity + ReDoS risk | Use `contains()`, `startsWith()`, `replace()` |
| Bitwise operators | Rarely needed in business logic | Use GATE handler |
| `typeof`, `instanceof` | Use `typeOf()` function instead | `typeOf($result)` |
| Template literals | Complexity | String concatenation with `+` |

### 5.10 Relation to `@benten/expressions`

The TRANSFORM expression language is a superset of the current `@benten/expressions` evaluator. It adds:

| Added | Current `@benten/expressions` |
|-------|-------------------------------|
| Arithmetic: `+`, `-`, `*`, `/`, `%` | Only comparison + logical operators |
| Object construction: `{ key: expr }` | Not supported |
| Array literals: `[a, b, c]` | Not supported |
| Spread: `{ ...$input }` | Not supported |
| 50+ built-in functions | 3 methods (`includes`, `startsWith`, `endsWith`) |
| Lambda expressions | Not supported |
| `$`-prefixed context variables | `record`, `user` identifiers |

The implementation reuses jsep for parsing and the custom AST walker for evaluation, extended with:
1. Arithmetic operator evaluation
2. Object/array literal node handling
3. Built-in function dispatch table
4. Lambda expression parsing and evaluation
5. `$`-prefixed identifier resolution from the pipeline context

---

## 6. Error Handling Patterns

### 6.1 Typed Error Edges (Preferred)

Every operation that can fail has typed error edges. Handling errors at the point of failure keeps the error path visible in the graph.

```typescript
const handler = subgraph('POST /api/posts')
  .require('store:write:post/*')
  .validate('contentType:post')
    .onInvalid(respond(400, { error: 'Invalid input', details: '$error.fields' }))
  .read('post', { lookup: { field: 'slug', value: '$input.slug' } })
    .onNotFound(
      // Slug is available -- proceed to create
      write('post', { data: '$input' })
    )
    // .onNotFound() not triggered means slug exists -- conflict
  .respond(409, { error: 'A post with this slug already exists' });
```

Wait -- that pattern is awkward. Let us show the correct idiomatic pattern:

```typescript
const createPost = subgraph('POST /api/posts')
  .require('store:write:post/*')
  .validate('contentType:post')
    .onInvalid(respond(400, { error: 'Invalid input', details: '$error.fields' }))
  .read('post', { lookup: { field: 'slug', value: '$input.slug' } })
    .onNotEmpty(respond(409, { error: 'Slug already exists' }))
  .write('post', { data: { ...'$input', createdAt: 'now()', updatedAt: 'now()' } })
  .emit('content:afterCreate', { id: '$result.id' })
  .respond(201, '$result');
```

The `.onNotEmpty()` method on a read-by-lookup checks the INVERSE condition: "if a record WAS found, that is the error." This is the uniqueness-check pattern.

### 6.2 Error Edge Summary

| Step Type | Error Methods | Default if Unhandled |
|-----------|-------------|---------------------|
| `read()` (by id/lookup) | `.onNotFound()` | `respond(404)` |
| `read()` (query) | `.onEmpty()` | Continue (empty array is valid) |
| `read()` (by lookup) | `.onNotEmpty()` | Continue (finding a record is valid) |
| `write()` (update) | `.onConflict()`, `.onNotFound()` | `respond(409)` / `respond(404)` |
| `write()` (delete) | `.onNotFound()` | `respond(404)` |
| `validate()` | `.onInvalid()` | `respond(400, $error)` |
| `gate()` | `.onError()` | `respond(500)` |
| `sandbox()` | `.onError()`, `.onTimeout()` | `respond(500)` / `respond(504)` |
| `call()` | `.onError()`, `.onTimeout()` | `respond(500)` / `respond(504)` |
| `wait()` | `.onTimeout()` | `respond(504)` |
| `iterate()` | `.onItemError()`, `.onLimitExceeded()` | abort subgraph / `respond(413)` |
| `.require()` | (implicit) | `respond(403)` |

### 6.3 Compensation (Saga Pattern)

For multi-step operations where failure requires undoing previous steps, use the `compensate()` wrapper:

```typescript
import { compensate } from '@benten/engine/operations';

const checkoutFlow = subgraph('POST /api/checkout')
  .require('commerce:write')
  .validate('CheckoutSchema')
  .compensate('Checkout Transaction', (saga) => {
    saga
      .step(
        gate('commerce/calculateTotal'),
        // No compensation needed for a pure calculation
      )
      .step(
        gate('payment/chargeCard', { token: '$input.paymentToken', amount: '$total' }),
        // On failure: refund the charge
        { undo: gate('payment/refundCharge', { chargeId: '$result.chargeId' }) }
      )
      .step(
        write('order', { data: '$orderData' }),
        // On failure: delete the order
        { undo: write('order', { action: 'delete', id: '$result.id' }) }
      )
      .step(
        iterate('$cart.items', (item) =>
          write('inventory', {
            action: 'update',
            id: '$item.inventoryId',
            data: { quantity: '$item.currentQty - $item.orderQty' },
          }),
          { max: 100 }
        ),
        // On failure: restore inventory quantities
        {
          undo: iterate('$cart.items', (item) =>
            write('inventory', {
              action: 'update',
              id: '$item.inventoryId',
              data: { quantity: '$item.currentQty' },
            }),
            { max: 100 }
          )
        }
      );
  })
  .emit('commerce:orderCreated', '$result')
  .respond(201, '$result');
```

**How compensation works:**

1. Steps execute in order (following NEXT edges within the saga).
2. Each step may have an `undo` handler.
3. If step N fails, undo handlers run for steps N-1, N-2, ..., 0 in reverse order.
4. After compensation completes, the saga follows its `ON_FAILURE` edge.
5. If an undo handler itself fails, the engine logs the compensation failure and continues undoing remaining steps (best-effort compensation).

### 6.4 Default Error Responses

When error edges are not explicitly handled, the engine generates sensible defaults:

```json
// 400 - Validation failed
{
  "error": "VALIDATION_FAILED",
  "message": "Input validation failed",
  "details": [
    { "path": "title", "message": "Required" },
    { "path": "email", "message": "Must be a valid email address" }
  ],
  "subgraph": "POST /api/posts",
  "node": "validate-0"
}

// 403 - Capability denied
{
  "error": "CAPABILITY_DENIED",
  "message": "Missing required capability: store:write:post/*",
  "subgraph": "POST /api/posts",
  "node": "require-0"
}

// 404 - Not found
{
  "error": "NOT_FOUND",
  "message": "post with id 'abc-123' not found",
  "subgraph": "GET /api/posts/:id",
  "node": "read-0"
}

// 409 - Version conflict
{
  "error": "VERSION_CONFLICT",
  "message": "Expected version 3, found version 5",
  "subgraph": "PUT /api/posts/:id",
  "node": "write-0"
}

// 500 - Handler error
{
  "error": "HANDLER_ERROR",
  "message": "Handler 'commerce/calculateTax' failed: Tax service unavailable",
  "subgraph": "POST /api/checkout",
  "node": "gate-2"
}
```

All error responses include `subgraph` and `node` for debuggability. In production, set `sanitizeErrors: true` to omit internal details from client responses while preserving them in logs.

---

## 7. Branching and Iteration

### 7.1 Two-Way Branch (Boolean)

The simplest branch: true or false.

```typescript
const handler = subgraph('POST /api/posts/publish')
  .require('content:publish')
  .read('post', { id: '$input.id' })
    .onNotFound(respond(404))
  .branch('$result.status === "draft"')
    .then(
      write('post', { action: 'update', id: '$input.id', data: { status: 'published', publishedAt: 'now()' } }),
      emit('content:afterPublish', { id: '$input.id' }),
      respond(200, { published: true }),
    )
    .otherwise(
      respond(400, { error: 'Post is already published' }),
    );
```

### 7.2 Multi-Way Branch

```typescript
const handler = subgraph('POST /api/payments/process')
  .require('commerce:payment')
  .validate('PaymentSchema')
  .branch()
    .when('$input.method === "stripe"',
      gate('payment/chargeStripe', { token: '$input.token', amount: '$input.amount' }),
      respond(200, '$result'),
    )
    .when('$input.method === "paypal"',
      gate('payment/chargePaypal', { orderId: '$input.orderId' }),
      respond(200, '$result'),
    )
    .when('$input.method === "crypto"',
      sandbox('payment/chargeCrypto', {
        input: { address: '$input.address', amount: '$input.amount' },
        fuel: 50_000,
        timeout: 60_000,
      }),
      respond(200, '$result'),
    )
    .otherwise(
      respond(400, { error: 'Unsupported payment method: $input.method' }),
    );
```

### 7.3 Iteration with Per-Item Processing

```typescript
const importProducts = subgraph('POST /api/products/import')
  .require('store:write:product/*')
  .validate('ImportSchema')
  .iterate('$input.rows', (row) => [
    validate('ProductRowSchema'),
    write('product', { action: 'upsert', data: '$item' }),
  ], {
    max: 1000,
    parallel: true,
    maxConcurrency: 10,
    as: '$item',
    collectAs: '$imported',
  })
    .onItemError(
      transform({ failedRow: '$index', error: '$error.message' })
      // Collected into $errors automatically
    )
  .transform({
    imported: 'len(filter($imported, r => r.success))',
    failed: 'len(filter($imported, r => !r.success))',
    errors: '$errors',
  })
  .respond(200);
```

### 7.4 Parallel Execution via Iterate

There is no separate `parallel()` node. Parallelism is expressed through `iterate()` with `parallel: true`. For fork-join patterns where the branches are heterogeneous (not iterating over a collection), use multiple `call()` steps or a `gate()` handler.

```typescript
// Homogeneous parallel: iterate over items
iterate('$input.imageUrls', (img) =>
  sandbox('media/resize', { input: { url: '$item' }, fuel: 100_000 }),
  { max: 20, parallel: true, maxConcurrency: 5 }
)

// Heterogeneous parallel: use gate handler
gate('dashboard/fetchAllData')
// The gate handler internally uses Promise.all/allSettled
```

---

## 8. SANDBOX (WASM) Integration

### 8.1 The Dual-Target Model

During development and testing, SANDBOX handlers run as plain TypeScript functions in the Node.js process. In production, they run inside QuickJS-in-WASM with fuel metering and capability membranes. The developer writes TypeScript. The build tool compiles it to WASM. The test harness runs it natively.

```
Development:  TypeScript -> Node.js (direct execution)
Testing:      TypeScript -> Node.js (direct execution, mocked host functions)
Production:   TypeScript -> QuickJS WASM (fuel-metered, capability membrane)
```

### 8.2 Writing a Sandbox Function

Sandbox functions are TypeScript files with a specific export convention:

```typescript
// File: modules/ai/content-scorer.ts

import type { SandboxContext } from '@benten/engine/sandbox';

export interface ScoreInput {
  title: string;
  body: string;
  targetKeywords: string[];
}

export interface ScoreResult {
  overall: number;
  readability: number;
  seo: number;
  suggestions: string[];
}

export function scoreContent(input: ScoreInput, ctx: SandboxContext): ScoreResult {
  // ctx.read() -- reads from graph (if capabilities allow)
  // ctx.log() -- structured logging (forwarded to host)
  // NO: ctx.write(), ctx.fetch(), ctx.import()

  const readability = computeReadabilityScore(input.body);
  const seo = computeSeoScore(input.title, input.body, input.targetKeywords);

  return {
    overall: Math.round((readability + seo) / 2),
    readability,
    seo,
    suggestions: generateSuggestions(readability, seo),
  };
}
```

### 8.3 Serialization Boundary

All data crossing the WASM boundary is serialized as JSON. This means:

| Type | Behavior | Developer Action |
|------|----------|-----------------|
| `string`, `number`, `boolean`, `null` | Pass through | None |
| Plain objects/arrays | Pass through | None |
| `Date` | Becomes ISO 8601 string | Use `parseDate()` on the other side |
| `Map`, `Set` | **Rejected** | Convert to object/array before passing |
| Class instances | **Rejected** | Use plain objects |
| Functions | **Rejected** | Not serializable |
| `BigInt` | **Rejected** | Use string or number |
| `undefined` | Becomes `null` | Use explicit `null` |

The engine validates arguments BEFORE serialization and produces a clear error:

```
SANDBOX_SERIALIZATION_ERROR: Argument 'input.createdAt' is a Date object.
Use .toISOString() to convert it to a string, or pass a numeric timestamp.
Subgraph: POST /api/content/score
Node: sandbox-0
```

### 8.4 Fuel and Memory

Fuel is the WASM equivalent of gas in Ethereum. It measures computation cost. The engine deducts fuel for each WASM instruction executed. When fuel reaches zero, execution halts.

```typescript
sandbox('ai/scoreContent', {
  input: { title: '$result.title', body: '$result.body' },
  fuel: 100_000,        // ~100ms of computation
  memoryLimit: 16_777_216,  // 16MB
  timeout: 5_000,       // 5s wall-clock safety net
})
```

**Fuel estimation:** During development, use the fuel estimator:

```typescript
const estimate = await engine.estimateFuel('ai/scoreContent', sampleInput);
// { fuelUsed: 47_230, peakMemory: 2_451_000, wallTime: 23 }
// Recommendation: set fuel to 75_000 (1.5x measured, safety margin)
```

---

## 9. Testing API

### 9.1 Test Engine

```typescript
import { createTestEngine } from '@benten/engine/testing';

const engine = createTestEngine();
```

`createTestEngine()` creates an in-memory engine instance with:
- No WASM boundary (SANDBOX handlers run as native TypeScript)
- No persistence (in-memory graph)
- No capability enforcement by default (override with `{ capabilities: true }`)
- Full execution tracing enabled
- Deterministic `now()` and `uuid()` (seeded, reproducible)

### 9.2 Testing a Subgraph

```typescript
import { describe, it, expect } from 'vitest';
import { createTestEngine, testSubgraph } from '@benten/engine/testing';
import { createPost } from './handlers.js';

describe('create post handler', () => {
  it('creates a post with valid input', async () => {
    const engine = createTestEngine();
    engine.registerSubgraph(createPost);

    const result = await testSubgraph(engine, createPost, {
      input: {
        title: 'Hello World',
        slug: 'hello-world',
        content: 'This is my first post.',
      },
      context: {
        user: { id: 'user-1', role: 'editor' },
      },
      capabilities: ['store:write:post/*', 'content:create'],
    });

    expect(result.status).toBe(201);
    expect(result.body).toMatchObject({
      title: 'Hello World',
      slug: 'hello-world',
    });
    expect(result.body.id).toBeDefined();
    expect(result.body.createdAt).toBeDefined();
  });

  it('rejects invalid input', async () => {
    const engine = createTestEngine();
    engine.registerSubgraph(createPost);

    const result = await testSubgraph(engine, createPost, {
      input: { title: '' },  // missing required fields
      context: { user: { id: 'user-1', role: 'editor' } },
      capabilities: ['store:write:post/*'],
    });

    expect(result.status).toBe(400);
    expect(result.body.error).toBe('VALIDATION_FAILED');
    expect(result.body.details).toContainEqual(
      expect.objectContaining({ path: 'title', message: expect.stringContaining('required') })
    );
  });

  it('rejects unauthorized users', async () => {
    const engine = createTestEngine({ capabilities: true });
    engine.registerSubgraph(createPost);

    const result = await testSubgraph(engine, createPost, {
      input: { title: 'Hello World', slug: 'hello-world' },
      context: { user: { id: 'user-1', role: 'member' } },
      capabilities: ['store:read:post/*'],  // read, not write
    });

    expect(result.status).toBe(403);
    expect(result.body.error).toBe('CAPABILITY_DENIED');
  });

  it('handles slug conflict', async () => {
    const engine = createTestEngine();
    engine.registerSubgraph(createPost);

    // Seed an existing post with the same slug
    await engine.seed('post', { id: 'existing-1', slug: 'hello-world', title: 'Existing Post' });

    const result = await testSubgraph(engine, createPost, {
      input: { title: 'New Post', slug: 'hello-world' },
      context: { user: { id: 'user-1', role: 'editor' } },
      capabilities: ['store:write:post/*'],
    });

    expect(result.status).toBe(409);
    expect(result.body.error).toContain('Slug already exists');
  });
});
```

### 9.3 Execution Trace Assertions

```typescript
it('executes nodes in expected order', async () => {
  const engine = createTestEngine();
  engine.registerSubgraph(createPost);

  const result = await testSubgraph(engine, createPost, {
    input: { title: 'Test', slug: 'test', content: 'Body' },
    context: { user: { id: 'user-1', role: 'editor' } },
    capabilities: ['store:write:post/*'],
    trace: true,
  });

  expect(result.trace).toHaveLength(6);
  expect(result.trace.map(t => t.type)).toEqual([
    'GATE',       // require capability
    'VALIDATE',   // schema validation
    'READ',       // slug uniqueness check
    'WRITE',      // create post
    'EMIT',       // afterCreate event
    'RESPOND',    // 201 response
  ]);

  // Assert specific node data
  expect(result.trace[3].output).toMatchObject({ id: expect.any(String) });
  expect(result.trace[3].duration).toBeLessThan(50);
});
```

### 9.4 Mocking Services and Handlers

```typescript
it('handles payment failure with compensation', async () => {
  const engine = createTestEngine();
  engine.registerSubgraph(checkoutFlow);

  // Mock the payment handler to fail
  engine.mockHandler('payment/chargeCard', async () => {
    throw new Error('Card declined');
  });

  // Track compensation calls
  const refundCalls: any[] = [];
  engine.mockHandler('payment/refundCharge', async (input) => {
    refundCalls.push(input);
    return { refunded: true };
  });

  const result = await testSubgraph(engine, checkoutFlow, {
    input: { items: [{ id: 'prod-1', qty: 1 }], paymentToken: 'tok_declined' },
    context: { user: { id: 'user-1' } },
    capabilities: ['commerce:write'],
    trace: true,
  });

  expect(result.status).toBe(402);
  // Verify compensation ran (no stale charge)
  expect(result.trace.some(t => t.type === 'GATE' && t.label === 'undo:payment')).toBe(true);
});
```

### 9.5 Snapshot Testing Graph Structure

```typescript
it('crud generates expected graph structure', () => {
  const handlers = crud('post', {
    schema: 'contentType:post',
    capability: 'store:post',
  });

  const createGraph = handlers.create.compile();

  expect(createGraph.nodes).toHaveLength(6);
  expect(createGraph.nodes.map(n => n.type)).toEqual([
    'GATE', 'VALIDATE', 'TRANSFORM', 'WRITE', 'EMIT', 'RESPOND',
  ]);
  expect(createGraph.edges.filter(e => e.type === 'ON_INVALID')).toHaveLength(1);
  expect(createGraph.edges.filter(e => e.type === 'ON_DENIED')).toHaveLength(1);

  // Snapshot for regression detection
  expect(createGraph).toMatchSnapshot();
});
```

### 9.6 Sandbox Testing

```typescript
it('scores content correctly', async () => {
  const engine = createTestEngine();

  // Register the sandbox function (runs as native TypeScript in test)
  engine.registerSandbox('ai/scoreContent', {
    source: scoreContent,  // direct function reference in tests
  });

  const result = await engine.executeSandbox('ai/scoreContent', {
    title: 'Complete Guide to TypeScript',
    body: 'TypeScript is a typed superset of JavaScript...',
    targetKeywords: ['typescript', 'guide'],
  });

  expect(result.overall).toBeGreaterThan(50);
  expect(result.readability).toBeGreaterThan(0);
  expect(result.seo).toBeGreaterThan(0);
  expect(result.suggestions).toBeInstanceOf(Array);
});
```

---

## 10. Python Equivalent API

The Python API uses PyO3 bindings to call the same Rust engine. The DSL mirrors the TypeScript API with Python idioms: snake_case, keyword arguments, context managers.

### 10.1 Basic Chain

```python
from benten import subgraph, read, write, validate, transform, respond, emit

list_posts = (subgraph('GET /api/posts')
  .require('store:read:post/*')
  .read('post',
    where={'published': True},
    order_by={'created_at': 'desc'},
    limit=20,
  )
  .transform({
    'items': '$result',
    'total': 'len($result)',
  })
  .respond(200))
```

### 10.2 CRUD Shorthand

```python
from benten import crud

post_handlers = crud('post',
  schema='contentType:post',
  capability='store:post',
  list={'sort': {'created_at': 'desc'}, 'limit': 20},
  create={'defaults': {'status': 'draft'}},
  update={'optimistic_locking': True},
  delete={'soft': True},
)

ctx.register_subgraphs(post_handlers.all)
```

### 10.3 Branching

```python
from benten import subgraph, branch, gate, call, respond

payment_handler = (subgraph('POST /api/payments/process')
  .require('commerce:payment')
  .validate('PaymentSchema')
  .branch()
    .when('$input.method === "stripe"',
      gate('payment/charge_stripe', token='$input.token', amount='$input.amount'),
      respond(200, '$result'),
    )
    .when('$input.method === "paypal"',
      gate('payment/charge_paypal', order_id='$input.order_id'),
      respond(200, '$result'),
    )
    .otherwise(
      respond(400, {'error': 'Unsupported payment method'}),
    ))
```

### 10.4 Error Handling

```python
create_post = (subgraph('POST /api/posts')
  .require('store:write:post/*')
  .validate('contentType:post')
    .on_invalid(respond(400, {'error': 'Validation failed', 'details': '$error.fields'}))
  .read('post', lookup={'field': 'slug', 'value': '$input.slug'})
    .on_not_empty(respond(409, {'error': 'Slug already exists'}))
  .write('post', data='$input')
  .emit('content:afterCreate', {'id': '$result.id'})
  .respond(201, '$result'))
```

### 10.5 Iteration

```python
import_products = (subgraph('POST /api/products/import')
  .require('store:write:product/*')
  .validate('ImportSchema')
  .iterate('$input.rows',
    body=lambda row: [
      validate('ProductRowSchema'),
      write('product', action='upsert', data='$item'),
    ],
    max=1000,
    parallel=True,
    max_concurrency=10,
  )
  .respond(200, {'imported': 'len($results)'}))
```

### 10.6 Testing

```python
from benten.testing import create_test_engine, test_subgraph
import pytest

def test_create_post():
    engine = create_test_engine()
    engine.register_subgraph(create_post)

    result = test_subgraph(engine, create_post,
      input={'title': 'Hello', 'slug': 'hello', 'content': 'Body'},
      context={'user': {'id': 'user-1', 'role': 'editor'}},
      capabilities=['store:write:post/*'],
    )

    assert result.status == 201
    assert result.body['title'] == 'Hello'
    assert 'id' in result.body


def test_rejects_invalid():
    engine = create_test_engine()
    engine.register_subgraph(create_post)

    result = test_subgraph(engine, create_post,
      input={'title': ''},
      context={'user': {'id': 'user-1', 'role': 'editor'}},
      capabilities=['store:write:post/*'],
    )

    assert result.status == 400
    assert result.body['error'] == 'VALIDATION_FAILED'
```

### 10.7 Naming Conventions

| TypeScript | Python |
|-----------|--------|
| `subgraph()` | `subgraph()` |
| `.onNotFound()` | `.on_not_found()` |
| `.onInvalid()` | `.on_invalid()` |
| `.onConflict()` | `.on_conflict()` |
| `.onNotEmpty()` | `.on_not_empty()` |
| `.onError()` | `.on_error()` |
| `.onTimeout()` | `.on_timeout()` |
| `.onItemError()` | `.on_item_error()` |
| `orderBy` | `order_by` |
| `maxConcurrency` | `max_concurrency` |
| `optimisticLocking` | `optimistic_locking` |

The expression language is identical across both languages. Expressions are strings evaluated by the engine, not by the host language.

---

## 11. DSL-to-Graph Compilation

### 11.1 What the DSL Produces

The DSL is a builder that constructs a `Subgraph` object at registration time. A `Subgraph` is:

```typescript
interface Subgraph {
  /** Unique identifier (e.g., 'GET /api/posts', 'commerce/checkout'). */
  id: string;
  /** Human-readable label. */
  label?: string;
  /** The entry-point Node ID. */
  entryNode: string;
  /** All Nodes in the subgraph. */
  nodes: OperationNode[];
  /** All Edges connecting the Nodes. */
  edges: OperationEdge[];
  /** Required capabilities for the entire subgraph. */
  capabilities: string[];
  /** Maximum execution timeout. */
  timeout?: number;
}

interface OperationNode {
  /** Unique ID within the subgraph (auto-generated: '{type}-{index}'). */
  id: string;
  /** One of the 12 operation types. */
  type: OperationType;
  /** Human-readable label (for debugging/tracing). */
  label?: string;
  /** Type-specific properties. */
  properties: Record<string, JsonValue>;
}

type OperationType =
  | 'READ' | 'WRITE' | 'TRANSFORM' | 'BRANCH' | 'ITERATE'
  | 'WAIT' | 'GATE' | 'CALL' | 'RESPOND' | 'EMIT'
  | 'SANDBOX' | 'VALIDATE';

interface OperationEdge {
  /** Source Node ID. */
  from: string;
  /** Target Node ID. */
  to: string;
  /** Edge type (determines semantics). */
  type: OperationEdgeType;
  /** Optional data transform applied as data flows through this edge. */
  transform?: Record<string, string>;
}

type OperationEdgeType =
  | 'NEXT'
  | 'ON_NOT_FOUND' | 'ON_EMPTY' | 'ON_NOT_EMPTY'
  | 'ON_INVALID' | 'ON_CONFLICT' | 'ON_DENIED'
  | 'ON_ERROR' | 'ON_TIMEOUT' | 'ON_FAILURE' | 'ON_LIMIT'
  | 'BRANCH' | 'BRANCH_DEFAULT'
  | 'BODY' | 'UNDO';
```

### 11.2 Compilation Example

Given this DSL code:

```typescript
const createPost = subgraph('POST /api/posts')
  .require('store:write:post/*')
  .validate('contentType:post')
    .onInvalid(respond(400, { error: 'Invalid input', details: '$error.fields' }))
  .read('post', { lookup: { field: 'slug', value: '$input.slug' } })
    .onNotEmpty(respond(409, { error: 'Slug already exists' }))
  .write('post', { data: { title: '$input.title', slug: '$input.slug', createdAt: 'now()' } })
  .emit('content:afterCreate', { id: '$result.id' })
  .respond(201, '$result');
```

The builder produces:

```typescript
// createPost.compile() returns:
{
  id: 'POST /api/posts',
  entryNode: 'gate-0',
  capabilities: ['store:write:post/*'],
  nodes: [
    {
      id: 'gate-0',
      type: 'GATE',
      label: 'require store:write:post/*',
      properties: {
        mode: 'capability',
        check: 'store:write:post/*',
      },
    },
    {
      id: 'validate-0',
      type: 'VALIDATE',
      label: 'validate contentType:post',
      properties: {
        schema: 'contentType:post',
        mode: 'strip',
      },
    },
    {
      id: 'respond-invalid',
      type: 'RESPOND',
      label: '400 Invalid input',
      properties: {
        status: 400,
        body: { error: 'Invalid input', details: '$error.fields' },
      },
    },
    {
      id: 'read-0',
      type: 'READ',
      label: 'lookup post by slug',
      properties: {
        target: 'post',
        mode: 'lookup',
        lookupField: 'slug',
        lookupValue: '$input.slug',
      },
    },
    {
      id: 'respond-conflict',
      type: 'RESPOND',
      label: '409 Slug already exists',
      properties: {
        status: 409,
        body: { error: 'Slug already exists' },
      },
    },
    {
      id: 'write-0',
      type: 'WRITE',
      label: 'create post',
      properties: {
        target: 'post',
        action: 'create',
        data: { title: '$input.title', slug: '$input.slug', createdAt: 'now()' },
      },
    },
    {
      id: 'emit-0',
      type: 'EMIT',
      label: 'content:afterCreate',
      properties: {
        event: 'content:afterCreate',
        payload: { id: '$result.id' },
      },
    },
    {
      id: 'respond-0',
      type: 'RESPOND',
      label: '201 Created',
      properties: {
        status: 201,
        body: '$result',
      },
    },
    {
      id: 'respond-denied',
      type: 'RESPOND',
      label: '403 Forbidden',
      properties: {
        status: 403,
        body: { error: 'CAPABILITY_DENIED', message: 'Missing required capability: store:write:post/*' },
      },
    },
  ],
  edges: [
    // Happy path
    { from: 'gate-0',      to: 'validate-0',       type: 'NEXT' },
    { from: 'validate-0',  to: 'read-0',           type: 'NEXT' },
    { from: 'read-0',      to: 'write-0',          type: 'NEXT' },           // lookup found nothing -> proceed
    { from: 'write-0',     to: 'emit-0',           type: 'NEXT' },
    { from: 'emit-0',      to: 'respond-0',        type: 'NEXT' },

    // Error paths
    { from: 'gate-0',      to: 'respond-denied',   type: 'ON_DENIED' },
    { from: 'validate-0',  to: 'respond-invalid',  type: 'ON_INVALID' },
    { from: 'read-0',      to: 'respond-conflict', type: 'ON_NOT_EMPTY' },  // slug exists = conflict
  ],
}
```

### 11.3 Structural Validation at Compile Time

When `.compile()` runs (or when the subgraph is registered), the builder validates:

| Check | Error if Violated |
|-------|------------------|
| Every execution path ends with RESPOND | `SUBGRAPH_NO_TERMINAL: Path from 'write-0' does not reach a RESPOND node` |
| No unreachable Nodes | `SUBGRAPH_UNREACHABLE: Node 'transform-3' is not reachable from entry node 'gate-0'` |
| ITERATE has `max` property | `ITERATE_UNBOUNDED: ITERATE node 'iterate-0' must specify 'max' (bounded iteration required)` |
| No cycles in the execution graph | `SUBGRAPH_CYCLE: Cycle detected: gate-0 -> read-0 -> gate-0` |
| BRANCH has at least one condition | `BRANCH_EMPTY: BRANCH node 'branch-0' has no conditions` |
| `.require()` capability strings are valid | `INVALID_CAPABILITY: Capability 'store:write' is missing scope (expected format: 'domain:action:scope')` |
| Expression strings parse successfully | `EXPRESSION_PARSE_ERROR: Expression '$input.title +' is not valid: unexpected end of input (in node 'transform-0')` |
| Schema references exist (if graph is available) | `SCHEMA_NOT_FOUND: Schema 'contentType:article' not found in graph (referenced by node 'validate-0')` |

These are **compile-time** checks, not runtime checks. They run when the developer registers the subgraph, providing immediate feedback. A subgraph that fails structural validation is never written to the graph.

### 11.4 Node ID Generation

Node IDs within a subgraph are auto-generated as `{type}-{index}`:
- `gate-0`, `gate-1` (for multiple capability checks)
- `validate-0`
- `read-0`, `read-1`
- `write-0`
- `emit-0`
- `respond-0`, `respond-denied`, `respond-invalid`, `respond-conflict`

Error-path RESPOND nodes get descriptive suffixes (`-denied`, `-invalid`, `-conflict`, `-notfound`, `-error`, `-timeout`) for readability in traces and debugging.

Custom IDs can be set via the `id` option on any step:

```typescript
.gate('commerce/calculateTax', { id: 'calc-tax' })
```

### 11.5 How the Engine Stores the Subgraph

When `ctx.registerSubgraph(sg)` is called, the compiled Subgraph is written to the engine's graph as:

1. A **SubgraphDef** anchor Node with label `OperationSubgraph`:
   ```
   Node { id: 'subgraph:POST /api/posts', type: 'OperationSubgraph', version: 1,
          config: { entryNode: 'gate-0', capabilities: ['store:write:post/*'] } }
   ```

2. One **OperationNode** per step, with label `Operation`:
   ```
   Node { id: 'subgraph:POST /api/posts:gate-0', type: 'Operation',
          config: { opType: 'GATE', mode: 'capability', check: 'store:write:post/*' } }
   ```

3. One **Edge** per connection:
   ```
   Edge { from: 'subgraph:POST /api/posts:gate-0',
          to: 'subgraph:POST /api/posts:validate-0',
          type: 'op:NEXT' }
   ```

4. A **CONTAINS** edge from the SubgraphDef to each OperationNode (for subgraph-scoped queries).

The `op:` prefix on edge types distinguishes operation edges from data edges in the graph.

---

## 12. Complete Handler Examples

### Example 1: Blog Post CRUD (Minimal)

```typescript
import { crud } from '@benten/engine/operations';

const postHandlers = crud('post', {
  schema: 'contentType:post',
  capability: 'store:post',
  list: { sort: { createdAt: 'desc' }, limit: 20 },
  update: { optimisticLocking: true },
  delete: { soft: true },
});

// 5 subgraphs, ~30 Nodes, 1 line of code.
ctx.registerSubgraphs(postHandlers.all);
```

### Example 2: Content Creation with Slug Uniqueness

```typescript
import { subgraph, respond, emit } from '@benten/engine/operations';

const createPost = subgraph('POST /api/posts')
  .require('store:write:post/*')
  .validate('contentType:post')
    .onInvalid(respond(400, { error: 'Validation failed', details: '$error.fields' }))
  .read('post', { lookup: { field: 'slug', value: '$input.slug' } })
    .onNotEmpty(respond(409, { error: 'A post with this slug already exists' }))
  .transform({
    ...'$input',
    createdAt: 'now()',
    updatedAt: 'now()',
    status: 'draft',
  })
  .write('post', { data: '$result' })
  .emit('content:afterCreate', { id: '$result.id', type: 'post' })
  .respond(201, '$result');
```

### Example 3: E-Commerce Checkout with Compensation

```typescript
import { subgraph, compensate, gate, write, emit, iterate, respond } from '@benten/engine/operations';

const checkout = subgraph('POST /api/checkout')
  .require('commerce:checkout')
  .validate('CheckoutSchema')
    .onInvalid(respond(400, { error: 'Invalid checkout data', details: '$error.fields' }))
  .read('cart', { id: '$input.cartId' })
    .onNotFound(respond(404, { error: 'Cart not found' }))
  .iterate('$result.items', (item) =>
    gate('commerce/checkInventory', { productId: '$item.productId', qty: '$item.quantity' }),
    { max: 100, parallel: true, as: '$item' }
  )
  .branch('some($results, r => !r.available)')
    .then(respond(409, {
      error: 'Items out of stock',
      unavailable: 'filter($results, r => !r.available)',
    }))
  .compensate('Checkout Transaction', (saga) => {
    saga
      .step(
        gate('commerce/calculateTotal', {
          items: '$cart.items',
          shipping: '$input.shippingMethod',
        }),
      )
      .step(
        gate('payment/charge', {
          token: '$input.paymentToken',
          amount: '$total',
          currency: '$input.currency',
        }),
        { undo: gate('payment/refund', { chargeId: '$result.chargeId' }) }
      )
      .step(
        write('order', {
          data: {
            userId: '$ctx.user.id',
            items: '$cart.items',
            total: '$total',
            chargeId: '$chargeId',
            status: 'confirmed',
          },
        }),
        { undo: write('order', { action: 'delete', id: '$result.id' }) }
      );
  })
  .emit('commerce:orderCreated', { orderId: '$result.id', userId: '$ctx.user.id' })
  .respond(201, { orderId: '$result.id', total: '$total', status: 'confirmed' });
```

### Example 4: Content Approval Workflow

```typescript
import { subgraph, branch, gate, write, emit, call, respond, wait } from '@benten/engine/operations';

const submitForReview = subgraph('POST /api/posts/:id/submit')
  .require('content:submit')
  .read('post', { id: '$ctx.params.id' })
    .onNotFound(respond(404))
  .branch('$result.status !== "draft"')
    .then(respond(400, { error: 'Only draft posts can be submitted for review' }))
  .write('post', {
    action: 'update',
    id: '$ctx.params.id',
    data: { status: 'pending_review', submittedAt: 'now()' },
  })
  .emit('content:submittedForReview', { postId: '$ctx.params.id', authorId: '$ctx.user.id' })
  .respond(200, { status: 'pending_review' });

const approvePost = subgraph('POST /api/posts/:id/approve')
  .require('content:approve')
  .read('post', { id: '$ctx.params.id' })
    .onNotFound(respond(404))
  .branch('$result.status !== "pending_review"')
    .then(respond(400, { error: 'Post is not pending review' }))
  .write('post', {
    action: 'update',
    id: '$ctx.params.id',
    data: { status: 'published', publishedAt: 'now()', approvedBy: '$ctx.user.id' },
  })
  .emit('content:published', { postId: '$ctx.params.id' })
  .call('notifications/notifyAuthor', {
    input: { authorId: '$result.authorId', postTitle: '$result.title', action: 'approved' },
  })
  .respond(200, { status: 'published' });
```

### Example 5: Real-Time Game State Update

```typescript
import { subgraph, gate, write, emit, respond } from '@benten/engine/operations';

const makeMove = subgraph('POST /api/games/:id/move')
  .require('game:play')
  .read('game', { id: '$ctx.params.id' })
    .onNotFound(respond(404, { error: 'Game not found' }))
  .branch('$result.status !== "active"')
    .then(respond(400, { error: 'Game is not active' }))
  .branch('$result.currentTurn !== $ctx.user.id')
    .then(respond(403, { error: 'Not your turn' }))
  .validate('MoveSchema')
  .gate('chess/validateAndApplyMove', {
    board: '$result.board',
    move: '$input.move',
    player: '$ctx.user.id',
  })
    .onError(respond(400, { error: 'Invalid move: $error.message' }))
  .write('game', {
    action: 'update',
    id: '$ctx.params.id',
    data: {
      board: '$result.newBoard',
      currentTurn: '$result.nextPlayer',
      moveHistory: '$result.moveHistory',
      status: '$result.gameStatus',
    },
    version: '$result.version',
  })
    .onConflict(respond(409, { error: 'Concurrent move detected. Reload game state.' }))
  .emit('game:moveMade', {
    gameId: '$ctx.params.id',
    move: '$input.move',
    player: '$ctx.user.id',
    status: '$result.gameStatus',
  })
  .respond(200, {
    board: '$result.newBoard',
    currentTurn: '$result.nextPlayer',
    status: '$result.gameStatus',
  });
```

### Example 6: AI Content Generation with Sandbox

```typescript
import { subgraph, sandbox, validate, write, emit, respond } from '@benten/engine/operations';

const generateContent = subgraph('POST /api/content/generate')
  .require('ai:generate')
  .require('store:write:content/*')
  .validate('GenerateContentSchema')
  .sandbox('ai/generateContent', {
    input: { topic: '$input.topic', style: '$input.style', length: '$input.length' },
    fuel: 200_000,
    timeout: 30_000,
    capabilities: ['store:read:content/*'],  // AI can read existing content for context
  })
    .onError(respond(500, { error: 'Content generation failed. Try again or simplify the topic.' }))
    .onTimeout(respond(504, { error: 'Content generation timed out. Try a shorter length.' }))
  .validate('contentType:post', { target: '$result' })
    .onInvalid(respond(500, { error: 'Generated content failed validation. Please try again.' }))
  .write('post', {
    data: {
      ...'$result',
      status: 'draft',
      generatedBy: 'ai',
      generatedAt: 'now()',
      authorId: '$ctx.user.id',
    },
  })
  .emit('content:generated', { id: '$result.id', topic: '$input.topic' })
  .respond(201, '$result');
```

### Example 7: Batch Data Import with Progress

```typescript
import { subgraph, validate, iterate, write, transform, respond } from '@benten/engine/operations';

const importProducts = subgraph('POST /api/products/import')
  .require('store:write:product/*')
  .validate('ImportBatchSchema')
  .iterate('$input.products', (product) => [
    validate('ProductSchema')
      .onInvalid(
        transform({ success: false, error: '$error', index: '$index' })
      ),
    write('product', { action: 'upsert', data: '$item' }),
    transform({ success: true, id: '$result.id', index: '$index' }),
  ], {
    max: 5000,
    parallel: true,
    maxConcurrency: 20,
    as: '$item',
    collectAs: '$importResults',
  })
  .transform({
    total: 'len($input.products)',
    succeeded: 'count($importResults, r => r.success)',
    failed: 'count($importResults, r => !r.success)',
    errors: 'filter($importResults, r => !r.success) | map(r => { index: r.index, error: r.error })',
  })
  .respond(200);
```

### Example 8: SEO Content Audit (Cross-Module Event Listener)

This is not a route handler -- it is an event-triggered subgraph that runs when content is created or updated. It demonstrates the `subscribesTo` pattern from Thrum V3-4.5.

```typescript
import { subgraph, gate, write, emit } from '@benten/engine/operations';

const auditContentSeo = subgraph('event:content:afterCreate')
  .label('SEO Content Audit')
  .read('post', { id: '$input.id' })
  .sandbox('seo/scoreContent', {
    input: { title: '$result.title', body: '$result.content', slug: '$result.slug' },
    fuel: 50_000,
    timeout: 5_000,
  })
  .write('seo_score', {
    action: 'upsert',
    data: {
      contentId: '$input.id',
      overall: '$result.overall',
      readability: '$result.readability',
      keywords: '$result.keywords',
      suggestions: '$result.suggestions',
      scoredAt: 'now()',
    },
  })
  .emit('seo:scored', { contentId: '$input.id', score: '$result.overall' });
// No respond() -- event handlers do not produce HTTP responses
```

---

## Appendix A: Learning Ladder

Structure your learning path through the 12 primitives:

| Stage | Primitives | What You Can Build |
|-------|-----------|-------------------|
| **Hour 1** | `crud()` | Full CRUD API for any content type. One function call. |
| **Hour 2** | `read`, `write`, `respond`, `validate` | Custom endpoints with validation. |
| **Day 1** | `transform`, `branch`, `emit` | Data shaping, conditional logic, events. |
| **Day 2** | `gate`, `call` | Business logic handlers, subgraph composition. |
| **Day 3** | `iterate`, `wait` | Batch processing, async workflows. |
| **Week 2** | `sandbox`, `compensate` | WASM isolation, saga patterns. |

Most module developers will spend 90% of their time in the first three stages. `sandbox` and `compensate` are for advanced use cases.

## Appendix B: Quick Reference Card

```
subgraph(id)                    -- Create a named subgraph builder
  .require(capability)          -- Check capability (403 on deny)
  .read(target, opts)           -- Retrieve data (.onNotFound, .onEmpty, .onNotEmpty)
  .write(target, opts)          -- Mutate data (.onConflict, .onNotFound)
  .validate(schema, opts)       -- Validate against schema (.onInvalid)
  .transform(expr, opts)        -- Reshape data (pure, no I/O)
  .branch(conditions)           -- Conditional routing (.when, .otherwise)
  .iterate(source, body, opts)  -- Bounded loop (.onItemError, .onLimitExceeded)
  .wait(opts)                   -- Suspend (.onTimeout)
  .gate(handler, args)          -- TypeScript escape hatch (.onError)
  .call(subgraphId, opts)       -- Execute another subgraph (.onError, .onTimeout)
  .sandbox(runtimeId, opts)     -- WASM sandbox (.onError, .onTimeout)
  .emit(event, payload)         -- Fire-and-forget event
  .respond(status, body)        -- Terminal: produce output
  .compensate(label, fn)        -- Saga with automatic undo
  .compile()                    -- Compile to Subgraph (Nodes + Edges)

crud(label, opts)               -- Generate 5 CRUD subgraphs from one call
```

## Appendix C: Expression Quick Reference

```
// Context variables
$input, $result, $ctx, $item, $index, $error, $results

// Arithmetic: + - * / %
// Comparison: === !== > >= < <=
// Logical: && || ! ??
// Ternary: cond ? a : b
// Optional chaining: $x?.y, $x?.[0]

// Object: { key: expr, ...spread }
// Array: [a, b, c]
// Lambda: x => expr

// String: lower upper trim contains startsWith endsWith replace replaceAll
//         split join truncate slugify padStart padEnd
// Array: len filter map find some every includes flat sort unique
//        slice reverse first last
// Aggregate: sum avg min max count groupBy
// Number: round floor ceil abs clamp
// Date: now formatDate parseDate addDays addHours diffDays isAfter isBefore
// Utility: coalesce typeOf keys values entries fromEntries merge pick omit uuid
```
