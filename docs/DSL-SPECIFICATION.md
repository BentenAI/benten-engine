# Benten Operation DSL -- Complete Specification

**Created:** 2026-04-11
**Last rewritten:** 2026-04-20 (aligned with the shipped 12-primitive set)
**Status:** Matches the shipped Phase 1 DSL (`@benten/engine` v0.x)
**Package:** `@benten/engine`
**Audience:** Module developers who compose operation subgraphs using TypeScript

This document describes the shipped 12-primitive DSL: **READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM**. The authoritative primitive list lives in [`ENGINE-SPEC.md`](ENGINE-SPEC.md) §3; this doc is its developer-facing mirror.

Four of the twelve primitives -- **WAIT, STREAM, SUBSCRIBE, SANDBOX** -- have their Node types and DSL helpers in Phase 1 (so structural validation recognises them and toolchains can build subgraphs that use them), but their executors ship in Phase 2. Attempting to evaluate one in Phase 1 returns the typed error `E_PRIMITIVE_NOT_IMPLEMENTED`. Everything else -- READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT -- executes today.

Two primitives from the pre-revision draft -- **VALIDATE** and **GATE** -- are not part of the shipped DSL. See [Appendix D: Migration from the pre-revision draft](#appendix-d-migration-from-the-pre-revision-draft) for the composition patterns that replace them.

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
- [Appendix A: Learning Ladder](#appendix-a-learning-ladder)
- [Appendix B: Quick Reference Card](#appendix-b-quick-reference-card)
- [Appendix C: Expression Quick Reference](#appendix-c-expression-quick-reference)
- [Appendix D: Migration from the pre-revision draft](#appendix-d-migration-from-the-pre-revision-draft)

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
  read, write, transform, branch, iterate,
  wait, call, respond, emit, sandbox,
  subscribe, stream,
} from '@benten/engine';
```

All 12 primitives plus `subgraph` and `crud` are named exports from the `@benten/engine` entry point. Typed errors (e.g. `EPrimitiveNotImplemented`, `EDslInvalidShape`) live on the sibling subpath `@benten/engine/errors`.

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
| 6 | `wait()` | WAIT | Suspend until signal or timeout (executor Phase 2) | No |
| 7 | `call()` | CALL | Execute another subgraph | No |
| 8 | `respond()` | RESPOND | Terminal: produce output | Yes |
| 9 | `emit()` | EMIT | Fire-and-forget event | No |
| 10 | `sandbox()` | SANDBOX | Execute code in WASM sandbox (executor Phase 2) | No |
| 11 | `subscribe()` | SUBSCRIBE | Reactive change notification (executor Phase 2) | No |
| 12 | `stream()` | STREAM | Partial output with back-pressure (executor Phase 2) | No |

The ordering matches [`ENGINE-SPEC.md`](ENGINE-SPEC.md) §3. Primitives marked *executor Phase 2* are type-valid -- they build well-formed subgraph Nodes and pass structural validation -- but the engine returns `E_PRIMITIVE_NOT_IMPLEMENTED` if evaluation actually reaches them in Phase 1.

---

### 2.1 `read(args)`

Retrieve data from the graph: a single Node by CID / property lookup, or a query against an IVM-backed view.

```typescript
function read(args: ReadArgs): { primitive: 'read'; args: ReadArgs };

interface ReadArgs {
  /** Label to read from. */
  label: string;
  /** Lookup key (`"id"` / `"cid"` / a property name). */
  by?: string;
  /** Literal value to filter on (when `by` is set). */
  value?: JsonValue;
  /** Bind the READ result under this key on `$result`. */
  as?: string;
}
```

**Implied mode.** When `by` + `value` identify a single record (e.g. `by: 'cid'`), the engine uses a single-node lookup (O(1) via IVM). Omitting `by` (or passing a view key such as `'_listView'`) returns a list from the matching IVM view.

**Error edges (typed):**

| Edge Type | When |
|-----------|------|
| `ON_NOT_FOUND` | Single lookup resolved to no Node |
| `ON_EMPTY` | Query/view returned zero results |
| `ON_DENIED` | Capability policy rejected the read |

**Examples:**

```typescript
// Single node by CID
read({ label: 'post', by: 'cid', value: '$input.cid' })

// Lookup by unique property
read({ label: 'post', by: 'slug', value: '$input.slug' })

// IVM list view
read({ label: 'post', by: '_listView' })

// Routing a not-found result to a 404 RESPOND
subgraph('get-post')
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .respond({ body: '$result' })
  .respond({ edge: 'ON_NOT_FOUND', status: 404, body: '{ error: "not found" }' })
  .build();
```

---

### 2.2 `write(args)`

Create or update a Node. Writes are content-addressed: the returned `$result.cid` is the CID of the persisted Node.

```typescript
function write(args: WriteArgs): { primitive: 'write'; args: WriteArgs };

interface WriteArgs {
  /** Label for the Node being written. */
  label: string;
  /** Properties to write. Expressions (e.g. '$input.title') resolve at evaluation time. */
  properties?: Record<string, JsonValue>;
  /** Optional `requires` capability (gates the WRITE at commit under a capability policy). */
  requires?: string;
}
```

Deletes are modeled as a tombstone write (`properties.tombstone = true`); see the `crud()` delete case in §4.3. Optimistic-lock semantics use the CID as the version discriminator: supplying `properties.cid` causes the evaluator to CAS against the existing Node's CID and emit `ON_CONFLICT` if they differ.

**Error edges (typed):**

| Edge Type | When |
|-----------|------|
| `ON_CONFLICT` | CAS failure (cid / tombstone mismatch) |
| `ON_DENIED` | `requires` property rejected by the policy |

**Examples:**

```typescript
// Create
write({ label: 'post', properties: { title: '$input.title', status: 'draft' }, requires: 'store:write:post/*' })

// Update with CAS
write({
  label: 'post',
  properties: { cid: '$input.cid', title: '$input.title', updatedAt: 'now()' },
  requires: 'store:write:post/*',
})

// Tombstone (soft delete)
write({ label: 'post', properties: { cid: '$input.cid', tombstone: true }, requires: 'store:write:post/*' })
```

---

### 2.3 `transform(args)`

Pure data reshaping. No I/O, no graph access. Uses the TRANSFORM expression language (Section 5).

```typescript
function transform(args: TransformArgs): { primitive: 'transform'; args: TransformArgs };

interface TransformArgs {
  /**
   * TRANSFORM expression source (a subset of JS per
   * `docs/TRANSFORM-GRAMMAR.md`). Parsed at registration.
   */
  expr: string;
  /** Where to bind the result on `$result`. Defaults to replacing `$result`. */
  as?: string;
}
```

**Two common forms** (both use the single `expr` string):

```typescript
// Object construction -- the common case
transform({ expr: '{ id: $result.id, title: $result.title, summary: truncate($result.content, 200) }' })

// Piped expression
transform({ expr: 'filter($result.items, i => i.active) | map(i => i.name)' })
```

---

### 2.4 `branch(args)`

Open a BRANCH Node switching on `args.on`. Case bodies are added via the `BranchBuilder.case(value, body)` chain from §3.3.

```typescript
function branch(args: BranchArgs): { primitive: 'branch'; args: BranchArgs };

interface BranchArgs {
  /** Expression over `$result` / `$input` to switch on. */
  on: string;
}
```

**Examples:**

```typescript
// Boolean predicate (2-way)
subgraph('publish')
  .branch({ on: '$result != null' })
    .case('true',  s => s.respond({ body: '$result' }))
    .case('false', s => s.respond({ status: 404, body: '{ error: "not found" }' }))
  .endBranch()
  .build();

// Multi-way dispatch (used by crud() internally)
subgraph('payments')
  .branch({ on: '$input.method' })
    .case('stripe', s => s.call({ handler: 'payment/charge-stripe' }).respond({ body: '$result' }))
    .case('paypal', s => s.call({ handler: 'payment/charge-paypal' }).respond({ body: '$result' }))
  .endBranch()
  .build();
```

See §3.3 for the BranchBuilder chain semantics (why the shipped DSL has no explicit `.otherwise()`).

---

### 2.5 `iterate(args)`

Bounded iteration over a collection. The body follows the ITERATE Node's NEXT chain until a RESPOND or a composed `ON_ITEM_ERROR` edge terminates the current iteration.

```typescript
function iterate(args: IterateArgs): { primitive: 'iterate'; args: IterateArgs };

interface IterateArgs {
  /** Source list expression. */
  over: string;
  /** Max iteration count (required -- invariant 9 / `E_INV_ITERATE_MAX_MISSING`). */
  max: number;
}
```

**Why `max` is required.** Operation subgraphs are not Turing complete. Every iteration must be bounded. The builder's `.iterate(...)` method throws `E_INV_ITERATE_MAX_MISSING` at build time if `max` is missing or non-positive; the engine re-checks structural invariant #9 at registration. This is a security invariant, not a convenience default.

**Error edges (typed):**

| Edge Type | When |
|-----------|------|
| `ON_LIMIT` | Source sequence longer than `max` |
| `ON_ITEM_ERROR` | Body aborted for the current iteration |

**Examples:**

```typescript
// Sequential iteration; body is everything up to the next RESPOND
subgraph('check-cart')
  .iterate({ over: '$cart.items', max: 100 })
  .call({ handler: 'commerce/check-inventory', input: '{ itemId: $item.id, qty: $item.quantity }' })
  .respond({ body: '$results' })
  .build();
```

Parallelism (see §7.4) and concurrency hints attach to the ITERATE args in the Phase 2 executor; the Phase 1 executor runs iterations sequentially.

---

### 2.6 `wait(args)`

Suspend execution until either an external signal arrives or a duration elapses. Phase 2a ships both forms: the timed form (shipped in Phase 1 as a structural stub) plus the signal-keyed form (promoted from Phase-2b plan per dx-r1-8). When a WAIT suspends, the engine returns a `SuspendedHandle`; call `engine.suspendToBytes(handle)` to persist the execution state and `engine.resumeFromBytes(bytes, signal_value)` to resume.

```typescript
function wait(args: WaitArgs): { primitive: 'wait'; args: WaitArgs };

// dx-r1-8: exactly one of `signal` / `duration` must be present.
type WaitArgs = WaitSignalArgs | WaitDurationArgs;

interface WaitSignalArgs {
  /** Signal name the WAIT suspends on (e.g. `"external:payment"`). */
  signal: string;
  /**
   * Optional schema constraining the resume-time payload. If omitted
   * (default), any `Value` is accepted. When set, a resume with a payload
   * whose shape does not match fires `E_WAIT_SIGNAL_SHAPE_MISMATCH`
   * BEFORE any downstream primitive executes.
   */
  signal_shape?: string;
  /** Optional timeout — if the signal does not arrive in time, `E_WAIT_TIMEOUT` fires. */
  duration?: string;
}

interface WaitDurationArgs {
  /** Duration string (e.g. `"5m"`, `"30s"`, `"2h"`). */
  duration: string;
}
```

**Error edges (typed):**

| Edge Type | When |
|-----------|------|
| `ON_TIMEOUT` | Deadline expired before the signal arrived (`E_WAIT_TIMEOUT`). |
| `ON_ERROR` | Signal payload failed `signal_shape` (`E_WAIT_SIGNAL_SHAPE_MISMATCH`), or the resume envelope failed integrity/principal/pin/capability checks (`E_EXEC_STATE_TAMPERED` / `E_RESUME_ACTOR_MISMATCH` / `E_RESUME_SUBGRAPH_DRIFT` / `E_CAP_REVOKED_MID_EVAL`). |

**Examples:**

```typescript
// Signal-keyed: suspend until an external event arrives.
wait({ signal: 'external:payment' })

// Typed signal payload — resume rejects shapes that don't match.
wait({
  signal: 'external:payment',
  signal_shape: '{ amount: Int, currency: Text }',
})

// Timed WAIT (Phase-1 form preserved).
wait({ duration: '5s' })

// Signal with fallback timeout.
wait({ signal: 'external:ack', duration: '1h' })
```

See `docs/ERROR-CATALOG.md` for the full list of resume-protocol error codes (`E_EXEC_STATE_TAMPERED`, `E_RESUME_ACTOR_MISMATCH`, `E_RESUME_SUBGRAPH_DRIFT`, `E_WAIT_TIMEOUT`, `E_WAIT_SIGNAL_SHAPE_MISMATCH`).

---

### 2.7 `call(args)`

Execute another subgraph. The function-call equivalent for operation graphs: a "charge payment" subgraph is defined once and called from checkout, subscription renewal, and refund retry flows.

```typescript
function call(args: CallArgs): { primitive: 'call'; args: CallArgs };

interface CallArgs {
  /** Handler id to CALL. */
  handler: string;
  /** Optional action on the target handler (e.g. `"post:get"`). */
  action?: string;
  /** Input expression bound to the callee's `$input`. */
  input?: string;
  /**
   * If `true`, the CALL enters an isolated capability scope and cannot
   * delegate parent caps. Default `false`. (ENGINE-SPEC §3 ships with
   * `true` as the long-term default; the DSL-side default is `false`
   * for Phase 1 ergonomics and flips in Phase 2 once UCAN attenuation
   * lands.)
   */
  isolated?: boolean;
}
```

**Error edges (typed):**

| Edge Type | When |
|-----------|------|
| `ON_ERROR` | Callee subgraph aborted |
| `ON_TIMEOUT` | Callee exceeded the configured timeout |

**Examples:**

```typescript
// Simple call
call({ handler: 'commerce/charge-stripe' })

// With input mapping and an action
call({
  handler: 'notifications/send-email',
  action: 'email:send',
  input: '{ to: $result.email, template: "order-confirmation", data: $result }',
})

// Isolated capability scope (callee cannot inherit parent caps)
call({ handler: 'thirdparty/analytics', isolated: true })
```

---

### 2.8 `respond(args)`

Terminal node. Produces the subgraph's output. Every execution path must end with a `respond()`.

```typescript
function respond(args?: RespondArgs): { primitive: 'respond'; args: RespondArgs };

interface RespondArgs {
  /** Response body expression. */
  body?: string;
  /** Optional typed error edge to route through (e.g. `"ON_NOT_FOUND"`). */
  edge?: string;
  /** Optional status-code override (HTTP mapping -- not enforced in Phase 1). */
  status?: number;
}
```

**Examples:**

```typescript
// Echo the current result
respond({ body: '$result' })

// Created with shaped body
respond({ status: 201, body: '{ id: $result.cid, slug: $result.slug }' })

// Route through a typed error edge
respond({ edge: 'ON_NOT_FOUND', body: '{ error: "not found" }' })

// No body
respond({ status: 204 })
```

---

### 2.9 `emit(args)`

Fire-and-forget event. Does not wait for subscribers. Execution continues along the NEXT edge immediately.

```typescript
function emit(args: EmitArgs): { primitive: 'emit'; args: EmitArgs };

interface EmitArgs {
  /** Event label. */
  event: string;
  /** Event payload expression. */
  payload?: string;
}
```

**Examples:**

```typescript
// Simple event
emit({ event: 'content:afterCreate', payload: '{ id: $result.cid }' })

// Full payload passthrough
emit({ event: 'commerce:orderCreated', payload: '$result' })
```

Subscribers attach via the `subscribe()` primitive (section 2.11) or via the engine's internal IVM wiring.

---

### 2.10 `sandbox(args)` *(executor Phase 2)*

Execute code in a WASM sandbox. For untrusted code, AI-generated code, or computationally intensive operations that need isolation and fuel metering. The **subgraph Node is valid in Phase 1** (structural validation recognises it), but calling a subgraph that reaches a SANDBOX node returns `E_PRIMITIVE_NOT_IMPLEMENTED` until the Phase 2 executor lands (see [`docs/future/phase-2-backlog.md`](future/phase-2-backlog.md)).

```typescript
function sandbox(args: SandboxArgs): { primitive: 'sandbox'; args: SandboxArgs };

interface SandboxArgs {
  /** WASM module CID to execute. */
  module: string;
  /** Fuel budget (per-subgraph, not per-call). */
  fuel?: number;
}
```

**Error edges (typed):**

| Edge Type | When |
|-----------|------|
| `ON_ERROR` | Runtime error or fuel exhaustion (Phase 2) |
| `ON_TIMEOUT` | Exceeded wall-clock timeout (Phase 2) |

**Examples:**

```typescript
// AI content generation (registers in Phase 1; executes in Phase 2)
sandbox({ module: 'bafyr4i...ai-module-cid', fuel: 100_000 })

// Image processing
sandbox({ module: 'bafyr4i...media-module-cid', fuel: 500_000 })
```

The Phase-2 executor uses `wasmtime` with fuel metering and capability membranes. See [`ENGINE-SPEC.md`](ENGINE-SPEC.md) §8 for the SANDBOX contract.

---

### 2.11 `subscribe(args)` *(executor Phase 2)*

Reactive change notification. Subscribes the current subgraph to a named event stream; the engine fires the subgraph each time the stream publishes. This is the base primitive that IVM, sync delta propagation, and event-driven handlers all compose on.

```typescript
function subscribe(args: SubscribeArgs): { primitive: 'subscribe'; args: SubscribeArgs };

interface SubscribeArgs {
  /** Event label to subscribe to (e.g. `"content:afterCreate"`). */
  event: string;
  /** Optional handler id to route deliveries through. */
  handler?: string;
}
```

**Phase status.** As with SANDBOX, the subgraph Node is well-formed in Phase 1 -- `SubgraphBuilder.subscribe({ ... })` compiles, the builder rejects malformed args with `EDslInvalidShape`, and registration passes structural validation. Phase 1 subgraphs that try to *evaluate* a SUBSCRIBE node return `E_PRIMITIVE_NOT_IMPLEMENTED`. The Phase-1 IVM subscriber wires into the same change-stream infrastructure at the `benten-graph` layer; user-visible SUBSCRIBE as an active operation in a user subgraph is a Phase 2 deliverable.

**Examples:**

```typescript
// Subscribe a subgraph to content-creation events
subgraph('audit-content-seo')
  .subscribe({ event: 'content:afterCreate' })
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .transform({ expr: '{ scoredAt: now() }' })
  .respond({ body: '$result' })
  .build();

// Subscribe via a handler-id router
subscribe({ event: 'commerce:orderCreated', handler: 'analytics/record-order' })
```

See [`ENGINE-SPEC.md`](ENGINE-SPEC.md) §3 on what SUBSCRIBE composes (IVM materialized views, event handler registration, sync delta propagation).

---

### 2.12 `stream(args)` *(executor Phase 2)*

Partial/ongoing output with back-pressure. Used for SSE, WebSocket messages, LLM token streams, and progress updates -- patterns that cannot be cleanly composed from RESPOND (terminal) or ITERATE (no back-pressure). WinterTC-targeted runtimes make streaming table stakes for 2026 web APIs.

```typescript
function stream(args: StreamArgs): { primitive: 'stream'; args: StreamArgs };

interface StreamArgs {
  /** Expression yielding the source sequence. */
  source: string;
  /** Optional chunk-size hint. */
  chunkSize?: number;
}
```

**Phase status.** Same shape as `subscribe()`: the subgraph Node registers and passes structural validation in Phase 1, but evaluation returns `E_PRIMITIVE_NOT_IMPLEMENTED` until the Phase-2 executor ships. The napi binding will bridge STREAM to a WinterTC-compatible `ReadableStream` when the executor lands.

**Error edges (typed):**

| Edge Type | When |
|-----------|------|
| `ON_ERROR` | Source sequence errored mid-stream (Phase 2) |
| `ON_TIMEOUT` | Stream stalled past the configured deadline (Phase 2) |

**Examples:**

```typescript
// Stream LLM tokens (Phase 2)
stream({ source: '$tokens', chunkSize: 16 })

// Stream a large query result as NDJSON (Phase 2)
subgraph('api/posts-ndjson')
  .read({ label: 'post' })
  .stream({ source: '$result' })
  .respond({ status: 200 })
  .build();
```

---

## 3. Fluent Builder API

The `subgraph()` function returns a `SubgraphBuilder` that chains primitives into a linear flow. Branching is nested via `.branch({ on: ... }).case('value', s => ...).endBranch()`.

### 3.1 Basic Chain

```typescript
const listPosts = subgraph('list-posts-handler')
  .action('post:list')
  .read({ label: 'post', by: '_listView' })
  .transform({ expr: '{ items: $result, total: len($result) }' })
  .respond({ body: '$result' })
  .build();
```

### 3.2 Chain Methods

The builder exposes one chain method per primitive. Each call appends a Node and wires a default `NEXT` edge from the previous Node.

```typescript
class SubgraphBuilder {
  // -- Data --
  read(args: ReadArgs): this;
  write(args: WriteArgs): this;

  // -- Logic --
  transform(args: TransformArgs): this;
  branch(args: BranchArgs): BranchBuilder;  // see 3.3 below
  iterate(args: IterateArgs): this;

  // -- Flow --
  call(args: CallArgs): this;
  wait(args: WaitArgs): this;         // Phase 2 executor

  // -- Output --
  respond(args?: RespondArgs): this;
  emit(args: EmitArgs): this;

  // -- Reactive / streaming (Phase 2 executors) --
  subscribe(args: SubscribeArgs): this;
  stream(args: StreamArgs): this;
  sandbox(args: SandboxArgs): this;

  // -- Meta --
  action(name: string): this;         // declare an exposed action string
  build(): Subgraph;                  // materialize the Subgraph
}
```

Capability checking is not a separate method. Any primitive Node that mutates or reads sensitive state accepts a `requires` property (e.g. `write({ label: 'post', requires: 'store:write:post/*', ... })`). The evaluator honours `requires` automatically: when the grant is absent the Node routes through `ON_DENIED` with no explicit builder method needed. See [Appendix D](#appendix-d-migration-from-the-pre-revision-draft) for the pre-revision `.require()` pattern this replaced.

### 3.3 Branching

```typescript
subgraph('publish-post')
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .branch({ on: '$result.status' })
    .case('draft', s => s
      .write({ label: 'post', properties: { status: 'published' } })
      .respond({ body: '$result' }),
    )
    .case('published', s => s
      .respond({ status: 400, body: '{ error: "already published" }' }),
    )
  .endBranch()
  .build();
```

`.case(value, body)` opens a sub-scope; whatever primitives `body` adds are attached to the BRANCH via a `CASE:<value>` edge. `.endBranch()` returns to the parent `SubgraphBuilder`.

### 3.4 Typed Error Edges

The engine emits typed error edges on primitives that can fail. Routing is declarative: the engine follows the edge to the first node the subgraph provides a handler for, otherwise it aborts with the corresponding typed error.

| Primitive | Edge Types | Triggered By |
|-----------|-----------|-------------|
| `read()` | `ON_NOT_FOUND`, `ON_EMPTY`, `ON_DENIED` | Missing Node / empty query / capability denial |
| `write()` | `ON_CONFLICT`, `ON_DENIED` | CAS failure / capability denial |
| `call()` | `ON_ERROR`, `ON_TIMEOUT` | Callee aborted / exceeded timeout |
| `iterate()` | `ON_LIMIT`, `ON_ITEM_ERROR` | Source longer than `max` / body aborted |
| `wait()` | `ON_TIMEOUT` | Deadline expired (Phase 2) |
| `sandbox()` | `ON_ERROR`, `ON_TIMEOUT` | Runtime error / fuel exhaustion (Phase 2) |
| `stream()` | `ON_ERROR`, `ON_TIMEOUT` | Source errored / stream stalled (Phase 2) |

To route a specific error to a terminal response, use a `respond()` with an `edge` override that matches the source error:

```typescript
subgraph('get-post')
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .respond({ body: '$result' })
  .respond({ edge: 'ON_NOT_FOUND', status: 404, body: '{ error: "not found" }' })
  .respond({ edge: 'ON_DENIED', status: 403, body: '{ error: "forbidden" }' })
  .build();
```

When no explicit handler is provided, the engine returns a fail-closed default (404 / 409 / 500 / 504 as appropriate) using codes from [`ERROR-CATALOG.md`](ERROR-CATALOG.md).

---

## 4. CRUD Shorthand

The most important DX feature. A single `crud('post')` call builds **one handler subgraph** that exposes five canonical actions (`create`, `get`, `list`, `update`, `delete`) via a dispatch BRANCH on `$input.action`. The Phase 1 zero-config path needs no `schema`, no `capability` -- just a label.

### 4.1 Signature

The shipped type lives in `packages/engine/src/dsl.ts`. Fields that look minimal are minimal on purpose: Phase 1 ships the tight middle; Phase 2 adds soft-delete, optimistic locking, after-hooks, and per-action schemas.

```typescript
function crud(label: string, opts?: CrudOptions): CrudHandler;

interface CrudOptions {
  /** Override the rendered label (default: the first `crud()` argument). */
  label?: string;
  /** Supply your own HLC source (useful for deterministic tests). */
  hlc?: () => number;
  /**
   * Capability expression required to execute the mutating actions
   * (`create`, `update`, `delete`). Stamped as a `requires` property on
   * each WRITE Node in the produced subgraph. Informational under
   * `PolicyKind.NoAuth` (default); enforced under
   * `PolicyKind.GrantBacked`.
   */
  capability?: string;
  /**
   * When `true`, flags the handler as expecting the `debug:read`
   * capability (named compromise #2, Option C). The flag is
   * informational in Phase 1 -- the real gate is
   * `engine.grantCapability({ actor, scope: "store:debug:read" })`;
   * the flag is a hint for tooling. Defaults `false`.
   */
  debugRead?: boolean;
}

interface CrudHandler {
  /** The underlying Subgraph. Pass directly to `engine.registerSubgraph()`. */
  readonly subgraph: Subgraph;
  /** The action strings exposed (e.g. `["create", "get", "list", "update", "delete"]`). */
  readonly actions: string[];
  /** The label used for this CRUD handler. */
  readonly label: string;
  /** HLC-stamped createdAt ms-since-epoch. */
  stampCreatedAt(): number;
}
```

### 4.2 Usage

```typescript
import { Engine, crud } from '@benten/engine';

const engine = await Engine.open('./data');

// Zero-config: one line, five actions.
const postHandler = await engine.registerSubgraph(crud('post'));

// With capability attenuation (checked under GrantBacked policy).
const gated = await engine.registerSubgraph(
  crud('post', { capability: 'store:write:post/*' }),
);

// Invoke an action.
await engine.call(postHandler.id, 'post:create', { title: 'hello' });
await engine.call(postHandler.id, 'post:get', { cid: '...' });
```

### 4.3 What It Generates

For `crud('post')` the builder produces **one** handler subgraph of the shape below. Every action is dispatched through a single BRANCH on `$input.action`; each case is a linear chain.

```
BRANCH[on = $input.action]
  CASE:create  -> WRITE[label=post, properties={from: $input} (+ requires if opts.capability)]
                  -> RESPOND[body = $result]
  CASE:get     -> READ [label=post, by=cid, value=$input.cid]
                  -> RESPOND[body = $result]
  CASE:list    -> READ [label=post, by=_listView]
                  -> RESPOND[body = $result]
  CASE:update  -> WRITE[label=post, properties={cid: $input.cid, patch: $input.patch}
                       (+ requires if opts.capability)]
                  -> RESPOND[body = $result]
  CASE:delete  -> WRITE[label=post, properties={cid: $input.cid, tombstone: true}
                       (+ requires if opts.capability)]
                  -> RESPOND[body = $result]
```

The five executable primitives used are **BRANCH, READ, WRITE, RESPOND** (plus the `transform`-free shape). No GATE, no VALIDATE -- capability gating happens via the `requires` property the engine reads directly at commit time, and schema validation is supplied by the 14 structural invariants at registration time (invariants 1-6, 9-10, 12 in Phase 1). See [Appendix D](#appendix-d-migration-from-the-pre-revision-draft) for the old GATE / VALIDATE patterns and the composed replacements when stricter body-shape checks are needed.

**Error edge routing.** The engine still emits the four typed error edges from [ENGINE-SPEC §5](ENGINE-SPEC.md#5-the-evaluator):

- `ON_NOT_FOUND` - READ could not resolve the lookup (fires on `get` / `update` / `delete` when the target cid is missing). Default: 404 response.
- `ON_CONFLICT` - WRITE aborted because the version / tombstone already present. Default: 409 response.
- `ON_DENIED` - `requires` property rejected at commit. Default: 403 response.
- `ON_ERROR` - Anything else the evaluator could not route. Default: 500 response.

To customize, append `respond({ edge: 'ON_NOT_FOUND', status: 404, body: ... })` chains to the underlying `handler.subgraph` before `registerSubgraph()`.

### 4.4 Customization

`CrudHandler.subgraph` is a plain `Subgraph` value. Extend it with additional Nodes / edges before registration; for more involved shapes, copy the canonical CRUD out and rebuild with `subgraph()` directly.

```typescript
const handle = crud('post', { capability: 'store:write:post/*' });

// Re-wrap to append a trailing EMIT after the existing RESPOND-create case.
const custom = subgraph('post-handler')
  .action('create').action('get').action('list').action('update').action('delete')
  .branch({ on: '$input.action' })
    .case('create', s => s
      .write({ label: 'post', properties: { from: '$input' }, requires: 'store:write:post/*' })
      .emit({ event: 'post:afterCreate', payload: '{ cid: $result.cid }' })
      .respond({ body: '$result' }),
    )
    .case('get', s => s
      .read({ label: 'post', by: 'cid', value: '$input.cid' })
      .respond({ body: '$result' }),
    )
    // ... list / update / delete unchanged
  .endBranch()
  .build();

await engine.registerSubgraph(custom);
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
| Function definition | Would create closures | Use a SANDBOX node or extract to a separate subgraph and `call()` it |
| `async`/`await` | Expressions are synchronous | Graph handles async via edges |
| `try`/`catch` | Expressions cannot fail (fail-closed) | Typed error edges on nodes |
| `new` | Object construction syntax differs | Use `{ }` object literals |
| `import`/`require` | No module system | Built-in functions only |
| `this` | No context binding | Use `$ctx` |
| Regular expressions | Complexity + ReDoS risk | Use `contains()`, `startsWith()`, `replace()` |
| Bitwise operators | Rarely needed in business logic | Use a SANDBOX node |
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
// Idiomatic pattern: validation as a BRANCH on a schema predicate;
// slug uniqueness via a READ that routes through ON_EMPTY (slug is free).
const createPost = subgraph('create-post')
  .action('post:create')
  // Schema validation composes from BRANCH + TRANSFORM + RESPOND
  // (the pre-revision VALIDATE primitive; see Appendix D).
  .branch({ on: 'isValid($input, "contentType:post")' })
    .case('false', s => s
      .respond({ status: 400, body: '{ error: "invalid input", details: $error.fields }' }),
    )
    .case('true', s => s
      .read({ label: 'post', by: 'slug', value: '$input.slug' })
      .respond({ edge: 'ON_NOT_FOUND', status: 0 })  // slug free: fall through
      .respond({ edge: 'ON_EMPTY', status: 0 })      // query variant: continue
      .write({
        label: 'post',
        properties: { ...('$input' as any), createdAt: 'now()', updatedAt: 'now()' },
        requires: 'store:write:post/*',
      })
      .emit({ event: 'content:afterCreate', payload: '{ cid: $result.cid }' })
      .respond({ status: 201, body: '$result' }),
    )
  .endBranch()
  .build();
```

The slug-uniqueness idiom is the typed-error-edge pattern: READ fires `ON_NOT_FOUND` (single lookup) or `ON_EMPTY` (query) when the slug is free, so the happy path continues to WRITE; if the READ *does* return a Node, the chain naturally terminates with the configured conflict response.

### 6.2 Error Edge Summary

| Primitive | Edge Types | Default if Unhandled |
|-----------|-----------|---------------------|
| `read()` (by id/lookup) | `ON_NOT_FOUND` | `respond(404)` |
| `read()` (query) | `ON_EMPTY` | Continue (empty array is valid) |
| `read()` / `write()` | `ON_DENIED` | `respond(403)` |
| `write()` (update) | `ON_CONFLICT` | `respond(409)` |
| `sandbox()` | `ON_ERROR`, `ON_TIMEOUT` | `respond(500)` / `respond(504)` (Phase 2 executor) |
| `call()` | `ON_ERROR`, `ON_TIMEOUT` | `respond(500)` / `respond(504)` |
| `wait()` | `ON_TIMEOUT` | `respond(504)` (Phase 2 executor) |
| `iterate()` | `ON_ITEM_ERROR`, `ON_LIMIT` | abort subgraph / `respond(413)` |

Capability denial (`ON_DENIED`) replaces the old `.require()`-based `GATE` pattern: the `requires` property on any Node triggers an automatic BRANCH to `ON_DENIED` when the policy rejects it. See [Appendix D](#appendix-d-migration-from-the-pre-revision-draft).

### 6.3 Compensation (Saga Pattern)

For multi-step operations where failure requires undoing previous steps, use the `compensate()` wrapper. Compensation is a stdlib pattern that composes BRANCH + CALL (undo subgraph) rather than a first-class primitive.

```typescript
import { subgraph, call, write, emit, iterate, compensate } from '@benten/engine';  // compensate is a stdlib helper

const checkoutFlow = subgraph('checkout')
  .action('checkout:process')
  .compensate('Checkout Transaction', (saga) => {
    saga
      .step(
        call({ handler: 'commerce/calculate-total' }),
        // No compensation needed for a pure calculation
      )
      .step(
        call({ handler: 'payment/charge-card', input: '{ token: $input.paymentToken, amount: $total }' }),
        // On failure: refund the charge
        { undo: call({ handler: 'payment/refund-charge', input: '{ chargeId: $result.chargeId }' }) },
      )
      .step(
        write({ label: 'order', properties: '$orderData', requires: 'commerce:write' }),
        // On failure: write a tombstone.
        { undo: write({ label: 'order', properties: { cid: '$result.cid', tombstone: true } }) },
      )
      .step(
        iterate({ over: '$cart.items', max: 100 }),
        // Body and undo omitted for brevity; see Example 3 in §12.
      );
  })
  .emit({ event: 'commerce:orderCreated', payload: '$result' })
  .respond({ status: 201, body: '$result' })
  .build();
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
// 400 - Validation failed (composed via BRANCH + RESPOND)
{
  "error": "E_DSL_INVALID_SHAPE",
  "message": "Input validation failed",
  "details": [
    { "path": "title", "message": "Required" },
    { "path": "email", "message": "Must be a valid email address" }
  ],
  "subgraph": "create-post",
  "node": "branch-1"
}

// 403 - Capability denied (requires-property rejection)
{
  "error": "E_CAP_DENIED_WRITE",
  "message": "Missing required capability: store:write:post/*",
  "subgraph": "create-post",
  "node": "write-3"
}

// 404 - Not found
{
  "error": "E_NODE_NOT_FOUND",
  "message": "post with cid 'bafyr4i...' not found",
  "subgraph": "post-handler",
  "node": "read-2"
}

// 409 - Version conflict
{
  "error": "E_WRITE_CONFLICT",
  "message": "Expected version 3, found version 5",
  "subgraph": "post-handler",
  "node": "write-4"
}

// 500 - Sub-call error
{
  "error": "E_CALL_FAILED",
  "message": "Callee 'commerce/calculate-tax' aborted: Tax service unavailable",
  "subgraph": "checkout",
  "node": "call-5"
}
```

Error codes are stable and catalogued in [`ERROR-CATALOG.md`](ERROR-CATALOG.md).

All error responses include `subgraph` and `node` for debuggability. In production, set `sanitizeErrors: true` to omit internal details from client responses while preserving them in logs.

---

## 7. Branching and Iteration

### 7.1 Two-Way Branch

BRANCH on a single expression, with `true` / `false` case handlers.

```typescript
const publishPost = subgraph('publish-post')
  .action('post:publish')
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .branch({ on: '$result.status === "draft"' })
    .case('true', s => s
      .write({
        label: 'post',
        properties: { cid: '$input.cid', status: 'published', publishedAt: 'now()' },
        requires: 'content:publish',
      })
      .emit({ event: 'content:afterPublish', payload: '{ cid: $input.cid }' })
      .respond({ status: 200, body: '{ published: true }' }),
    )
    .case('false', s => s
      .respond({ status: 400, body: '{ error: "Post is already published" }' }),
    )
  .endBranch()
  .build();
```

### 7.2 Multi-Way Branch

BRANCH accepts an arbitrary number of `.case(value, body)` entries; the evaluator routes on exact-match of `on` against the case value.

```typescript
const processPayment = subgraph('process-payment')
  .action('payment:process')
  .branch({ on: '$input.method' })
    .case('stripe', s => s
      .call({ handler: 'payment/charge-stripe', input: '{ token: $input.token, amount: $input.amount }' })
      .respond({ status: 200, body: '$result' }),
    )
    .case('paypal', s => s
      .call({ handler: 'payment/charge-paypal', input: '{ orderId: $input.orderId }' })
      .respond({ status: 200, body: '$result' }),
    )
    .case('crypto', s => s
      .sandbox({ module: 'bafyr4i...crypto-charge-module-cid', fuel: 50_000 })
      .respond({ status: 200, body: '$result' }),
    )
  .endBranch()
  .build();
```

For an "otherwise" default, add a trailing `.case('*', s => ...)` body that explicitly matches any unhandled value -- the shipped DSL uses exact-match semantics plus an explicit catch-all case; there is no separate `.otherwise()` today.

### 7.3 Iteration with Per-Item Processing

```typescript
const importProducts = subgraph('import-products')
  .action('product:import')
  .iterate({ over: '$input.rows', max: 1000 })
  .write({ label: 'product', properties: '$item', requires: 'store:write:product/*' })
  .transform({ expr: '{ success: true, cid: $result.cid, index: $index }' })
  .respond({ body: '$results' })
  .build();
```

The ITERATE body follows the immediate NEXT chain until the next RESPOND or the subgraph ends. Per-item shape checks compose from an inner BRANCH that short-circuits into a RESPOND when the predicate fails; the outer ITERATE collects successes and aborted-body items into `$results`. See [Appendix D](#appendix-d-migration-from-the-pre-revision-draft) for the VALIDATE → composed-BRANCH pattern.

### 7.4 Parallel Execution via Iterate

Parallelism is expressed through `iterate()`'s parallel mode (configured on the ITERATE Node args; the Phase 2 executor extends this to a host-provided concurrency limit). For fork-join patterns where branches are heterogeneous (not iterating over a collection), use multiple `call()` primitives -- the evaluator executes independent `call()` paths concurrently when they fan out from the same predecessor.

```typescript
// Homogeneous parallel: iterate over items
subgraph('resize-images')
  .iterate({ over: '$input.imageUrls', max: 20 })
  .sandbox({ module: 'bafyr4i...resize-module-cid', fuel: 100_000 })
  .respond({ body: '$results' })
  .build();

// Heterogeneous parallel: fan out via multiple call() nodes
subgraph('dashboard-fetch')
  .call({ handler: 'analytics/fetch-pageviews' })
  .call({ handler: 'commerce/fetch-revenue' })
  .call({ handler: 'content/fetch-recent' })
  .transform({ expr: '{ pageviews: $r0, revenue: $r1, recent: $r2 }' })
  .respond({ body: '$result' })
  .build();
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

  expect(result.trace).toHaveLength(5);
  expect(result.trace.map(t => t.type)).toEqual([
    'BRANCH',     // schema predicate (composed VALIDATE)
    'READ',       // slug uniqueness check
    'WRITE',      // create post (requires-property gates at commit)
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

  // Mock the payment subgraph to fail
  engine.mockHandler('payment/charge-card', async () => {
    throw new Error('Card declined');
  });

  // Track compensation calls
  const refundCalls: any[] = [];
  engine.mockHandler('payment/refund-charge', async (input) => {
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
  expect(result.trace.some(t => t.type === 'CALL' && t.label === 'undo:payment')).toBe(true);
});
```

### 9.5 Snapshot Testing Graph Structure

```typescript
it('crud generates expected graph structure', () => {
  const handle = crud('post', { capability: 'store:write:post/*' });

  const sg = handle.subgraph;

  // One dispatch BRANCH + five cases, each a tiny linear chain.
  expect(sg.nodes.map(n => n.primitive)).toEqual([
    'branch',
    'write', 'respond',  // create
    'read',  'respond',  // get
    'read',  'respond',  // list
    'write', 'respond',  // update
    'write', 'respond',  // delete
  ]);

  // Snapshot for regression detection
  expect(sg).toMatchSnapshot();
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

The PyO3-backed Python API is a Phase 2+ deliverable; the shapes below mirror the shipped TypeScript DSL and are included for reference once bindings land. Until then, treat this section as a design sketch -- the TypeScript sections are the load-bearing source of truth.

### 10.1 Basic Chain

```python
from benten import subgraph

list_posts = (subgraph('list-posts')
  .action('post:list')
  .read(label='post', by='_listView')
  .transform(expr='{ items: $result, total: len($result) }')
  .respond(body='$result')
  .build())
```

### 10.2 CRUD Shorthand

```python
from benten import crud

post_handle = crud('post', capability='store:write:post/*')
engine.register_subgraph(post_handle)
```

### 10.3 Branching

```python
from benten import subgraph

payment_handler = (subgraph('process-payment')
  .action('payment:process')
  .branch(on='$input.method')
    .case('stripe', lambda s: (s
      .call(handler='payment/charge-stripe', input='{ token: $input.token, amount: $input.amount }')
      .respond(status=200, body='$result')))
    .case('paypal', lambda s: (s
      .call(handler='payment/charge-paypal', input='{ orderId: $input.orderId }')
      .respond(status=200, body='$result')))
  .end_branch()
  .build())
```

### 10.4 Error Handling

```python
# Slug uniqueness via a READ that routes ON_NOT_FOUND to the WRITE path.
create_post = (subgraph('create-post')
  .action('post:create')
  .read(label='post', by='slug', value='$input.slug')
  .respond(edge='ON_NOT_FOUND', status=0)  # slug free: fall through
  .write(label='post', properties='$input', requires='store:write:post/*')
  .emit(event='content:afterCreate', payload='{ cid: $result.cid }')
  .respond(status=201, body='$result')
  .build())
```

### 10.5 Iteration

```python
import_products = (subgraph('import-products')
  .action('product:import')
  .iterate(over='$input.products', max=5000)
  .write(label='product', properties='$item', requires='store:write:product/*')
  .respond(body='$results')
  .build())
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
    assert result.body['error'] == 'E_DSL_INVALID_SHAPE'
```

### 10.7 Naming Conventions

| TypeScript | Python |
|-----------|--------|
| `subgraph()` | `subgraph()` |
| `.endBranch()` | `.end_branch()` |
| `.case(value, body)` | `.case(value, body)` |
| `build()` | `build()` |
| `debugRead` (option) | `debug_read` |
| `chunkSize` (arg) | `chunk_size` |
| camelCase arg keys | snake_case arg keys |

The expression language is identical across both languages. Expressions are strings evaluated by the engine, not by the host language.

---

## 11. DSL-to-Graph Compilation

### 11.1 What the DSL Produces

The DSL is a builder that constructs a `Subgraph` object at registration time. A `Subgraph` is:

```typescript
interface Subgraph {
  /** Handler id (unique namespace for the subgraph). */
  handlerId: string;
  /** Declared action strings (e.g. ['post:create', 'post:get', ...]). */
  actions: string[];
  /** All Subgraph Nodes, keyed by local id (`read-1`, `branch-2`, etc.). */
  nodes: SubgraphNode[];
  /** The root Node id (entry point of evaluation). */
  root: string;
}

interface SubgraphNode {
  /** Local id, auto-generated as `<primitive>-<counter>`. */
  id: string;
  /** One of the 12 primitive types. */
  primitive: Primitive;
  /** Primitive-specific args (shape from §2). */
  args: Record<string, JsonValue>;
  /** Outgoing edges, keyed by edge label. */
  edges: Record<string, string>;
}

type Primitive =
  | 'read' | 'write' | 'transform' | 'branch' | 'iterate'
  | 'wait' | 'call' | 'respond' | 'emit'
  | 'sandbox' | 'subscribe' | 'stream';
```

Edge labels follow these conventions:

| Label | When |
|-------|------|
| `NEXT` | Default forward edge (top-to-bottom chain order). |
| `CASE:<value>` | BRANCH case body (one per `.case(value, body)` call). |
| `ON_NOT_FOUND` | READ of a single lookup returned nothing. |
| `ON_EMPTY` | READ of a query returned zero rows. |
| `ON_CONFLICT` | WRITE CAS failed. |
| `ON_DENIED` | `requires`-property rejection at commit. |
| `ON_ERROR`, `ON_TIMEOUT`, `ON_LIMIT`, `ON_ITEM_ERROR` | Typed error edges per ENGINE-SPEC §5. |

### 11.2 Compilation Example

Given this DSL code:

```typescript
const createPost = subgraph('create-post')
  .action('post:create')
  .read({ label: 'post', by: 'slug', value: '$input.slug' })
  .respond({ edge: 'ON_NOT_FOUND', status: 0 })  // slug free: fall through
  .write({
    label: 'post',
    properties: { title: '$input.title', slug: '$input.slug', createdAt: 'now()' },
    requires: 'store:write:post/*',
  })
  .emit({ event: 'content:afterCreate', payload: '{ cid: $result.cid }' })
  .respond({ status: 201, body: '$result' })
  .build();
```

The builder produces:

```typescript
// createPost (returned by .build())
{
  handlerId: 'create-post',
  actions: ['post:create'],
  root: 'read-1',
  nodes: [
    {
      id: 'read-1',
      primitive: 'read',
      args: { label: 'post', by: 'slug', value: '$input.slug' },
      edges: { NEXT: 'write-3' },  // slug free -> proceed to write
    },
    {
      id: 'respond-2',
      primitive: 'respond',
      args: { edge: 'ON_NOT_FOUND', status: 0 },
      edges: {},  // terminal
    },
    {
      id: 'write-3',
      primitive: 'write',
      args: {
        label: 'post',
        properties: { title: '$input.title', slug: '$input.slug', createdAt: 'now()' },
        requires: 'store:write:post/*',
      },
      edges: { NEXT: 'emit-4' },
    },
    {
      id: 'emit-4',
      primitive: 'emit',
      args: { event: 'content:afterCreate', payload: '{ cid: $result.cid }' },
      edges: { NEXT: 'respond-5' },
    },
    {
      id: 'respond-5',
      primitive: 'respond',
      args: { status: 201, body: '$result' },
      edges: {},
    },
  ],
}
```

The evaluator routes typed error edges (`ON_DENIED` on `write-3`, `ON_NOT_FOUND` on `read-1`) to the nearest matching `respond({ edge: '<code>', ... })` Node; if none is provided, it falls back to the engine's default error responses using codes from [`ERROR-CATALOG.md`](ERROR-CATALOG.md).

### 11.3 Structural Validation at Build Time

`SubgraphBuilder.build()` enforces shape-level checks immediately (failures throw `EDslInvalidShape`). The engine re-runs the full 14-invariant structural validation at `registerSubgraph()` time (invariants 1-6, 9-10, 12 in Phase 1; the remainder in Phase 2). Sample failures:

| Invariant | Error code | Message shape |
|-----------|-----------|---------------|
| (build-time) root present | `E_DSL_INVALID_SHAPE` | `subgraph 'x' has no nodes - add at least one primitive before calling .build()` |
| #1 DAG-ness | `E_INV_CYCLE` | `cycle detected: read-1 -> branch-2 -> read-1` |
| #5 Max nodes | `E_INV_TOO_MANY_NODES` | `subgraph has 4097 nodes; limit is 4096` |
| #9 ITERATE `max` | `E_INV_ITERATE_MAX_MISSING` | `iterate requires a positive integer 'max'` |
| #10 Hash stability | `E_INV_HASH_MISMATCH` | `registered subgraph does not hash to its declared cid` |
| #12 Root reachability | `E_INV_UNREACHABLE_NODE` | `node 'respond-4' is not reachable from root 'read-1'` |

### 11.4 Node ID Generation

The builder assigns node ids as `<primitive>-<counter>`, where the counter is per-builder-instance and monotonic:

- `read-1`, `read-2` (for multiple READs)
- `write-3`
- `branch-4`, with case bodies becoming `<primitive>-<n+1>`, `<primitive>-<n+2>`, ...
- `respond-5`, `respond-6`

Per-instance counters make two parallel `crud('post')` calls produce identical subgraph shapes (and therefore identical content-addressed CIDs), which is load-bearing for handler-cid stability across Vitest workers -- see the class-level JSDoc on `SubgraphBuilder` in `packages/engine/src/dsl.ts`.

### 11.5 How the Engine Stores the Subgraph

When `engine.registerSubgraph(sg)` is called, the built `Subgraph` is hashed (BLAKE3 over DAG-CBOR of `{ handlerId, actions, root, nodes }`) and written to the engine's graph as:

1. A **handler anchor Node** under the system-zone label `system:Handler`:
   ```
   Node { labels: ['system:Handler'],
          properties: { handlerId: 'create-post', actions: ['post:create'], root: 'read-1' } }
   ```

2. One child Node per subgraph Node, labeled `system:HandlerNode`, linked by a `CONTAINS` Edge from the anchor.

3. Intra-subgraph edges are serialized into each HandlerNode's `edges` property (not as separate graph Edges) -- keeps the handler self-contained and content-addressed.

System-zone labels are unreachable from user operations (invariant #11); user subgraphs cannot READ or WRITE the handler metadata directly. The `op:` namespace prefix on edge labels (e.g. `op:NEXT` for handler-internal NEXT) is purely a convention; the shipped builder stores them under their bare names.

---

## 12. Complete Handler Examples

### Example 1: Blog Post CRUD (Minimal)

```typescript
import { Engine, crud } from '@benten/engine';

const engine = await Engine.open('./data');
const postHandler = await engine.registerSubgraph(crud('post'));

// One dispatch BRANCH with five cases (create/get/list/update/delete).
await engine.call(postHandler.id, 'post:create', { title: 'hello' });
```

### Example 2: Content Creation with Slug Uniqueness

```typescript
import { subgraph } from '@benten/engine';

const createPost = subgraph('create-post')
  .action('post:create')
  .read({ label: 'post', by: 'slug', value: '$input.slug' })
  // Slug is free -> fall through; if present the chain terminates through
  // the engine's default ON_CONFLICT response for the WRITE below.
  .respond({ edge: 'ON_NOT_FOUND', status: 0 })
  .transform({ expr: '{ ...$input, createdAt: now(), updatedAt: now(), status: "draft" }' })
  .write({ label: 'post', properties: '$result', requires: 'store:write:post/*' })
  .emit({ event: 'content:afterCreate', payload: '{ cid: $result.cid, type: "post" }' })
  .respond({ status: 201, body: '$result' })
  .build();
```

### Example 3: E-Commerce Checkout with Compensation

Compensation is a composed pattern (BRANCH on error + CALL to an undo subgraph), not a primitive. See the composed helper sketch at `compensate(label, fn)` for ergonomic reuse.

```typescript
import { subgraph, compensate } from '@benten/engine';

const checkout = subgraph('checkout')
  .action('checkout:process')
  .read({ label: 'cart', by: 'cid', value: '$input.cartId' })
  .iterate({ over: '$result.items', max: 100 })
  .call({ handler: 'commerce/check-inventory', input: '{ productId: $item.productId, qty: $item.quantity }' })
  .branch({ on: 'some($results, r => !r.available)' })
    .case('true', s => s
      .respond({ status: 409, body: '{ error: "out of stock", unavailable: filter($results, r => !r.available) }' }),
    )
    .case('false', s => s
      // Compensation: each step has an optional undo that the engine
      // walks in reverse order on failure.
      .call({ handler: 'commerce/calculate-total', input: '{ items: $cart.items, shipping: $input.shippingMethod }' })
      .call({ handler: 'payment/charge',
              input: '{ token: $input.paymentToken, amount: $total, currency: $input.currency }' })
      .write({
        label: 'order',
        properties: {
          userId: '$ctx.user.id',
          items: '$cart.items',
          total: '$total',
          chargeId: '$chargeId',
          status: 'confirmed',
        },
        requires: 'commerce:write',
      })
      .emit({ event: 'commerce:orderCreated', payload: '{ orderId: $result.cid, userId: $ctx.user.id }' })
      .respond({ status: 201, body: '{ orderId: $result.cid, total: $total, status: "confirmed" }' }),
    )
  .endBranch()
  .build();
```

### Example 4: Content Approval Workflow

```typescript
import { subgraph } from '@benten/engine';

const submitForReview = subgraph('submit-for-review')
  .action('post:submit-review')
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .branch({ on: '$result.status === "draft"' })
    .case('false', s => s
      .respond({ status: 400, body: '{ error: "Only draft posts can be submitted" }' }),
    )
    .case('true', s => s
      .write({
        label: 'post',
        properties: { cid: '$input.cid', status: 'pending_review', submittedAt: 'now()' },
        requires: 'content:submit',
      })
      .emit({ event: 'content:submittedForReview', payload: '{ postId: $input.cid, authorId: $ctx.user.id }' })
      .respond({ status: 200, body: '{ status: "pending_review" }' }),
    )
  .endBranch()
  .build();

const approvePost = subgraph('approve-post')
  .action('post:approve')
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .branch({ on: '$result.status === "pending_review"' })
    .case('false', s => s
      .respond({ status: 400, body: '{ error: "Post is not pending review" }' }),
    )
    .case('true', s => s
      .write({
        label: 'post',
        properties: { cid: '$input.cid', status: 'published', publishedAt: 'now()', approvedBy: '$ctx.user.id' },
        requires: 'content:approve',
      })
      .emit({ event: 'content:published', payload: '{ postId: $input.cid }' })
      .call({
        handler: 'notifications/notify-author',
        input: '{ authorId: $result.authorId, postTitle: $result.title, action: "approved" }',
      })
      .respond({ status: 200, body: '{ status: "published" }' }),
    )
  .endBranch()
  .build();
```

### Example 5: Real-Time Game State Update

```typescript
import { subgraph } from '@benten/engine';

const makeMove = subgraph('game-make-move')
  .action('game:move')
  .read({ label: 'game', by: 'cid', value: '$input.cid' })
  .branch({ on: '$result.status === "active" && $result.currentTurn === $ctx.user.id' })
    .case('false', s => s
      .respond({ status: 403, body: '{ error: "not your turn or game inactive" }' }),
    )
    .case('true', s => s
      // Move validation is a TRANSFORM that asserts shape; if the
      // move is invalid the TRANSFORM returns an error object that
      // the subsequent BRANCH routes into a 400.
      .transform({ expr: 'validateMove($result.board, $input.move, $ctx.user.id)', as: '$applied' })
      .branch({ on: '$applied.valid' })
        .case('false', s2 => s2
          .respond({ status: 400, body: '{ error: "Invalid move: " + $applied.reason }' }),
        )
        .case('true', s2 => s2
          .write({
            label: 'game',
            properties: {
              cid: '$input.cid',
              board: '$applied.newBoard',
              currentTurn: '$applied.nextPlayer',
              moveHistory: '$applied.moveHistory',
              status: '$applied.gameStatus',
              version: '$result.version',
            },
            requires: 'game:play',
          })
          .emit({
            event: 'game:moveMade',
            payload: '{ gameId: $input.cid, move: $input.move, player: $ctx.user.id, status: $applied.gameStatus }',
          })
          .respond({
            status: 200,
            body: '{ board: $applied.newBoard, currentTurn: $applied.nextPlayer, status: $applied.gameStatus }',
          }),
        )
      .endBranch(),
    )
  .endBranch()
  .build();
```

### Example 6: AI Content Generation with Sandbox *(Phase 2 executor)*

```typescript
import { subgraph } from '@benten/engine';

// The SANDBOX executor ships in Phase 2; this subgraph registers
// cleanly in Phase 1 but `engine.call('content:generate')` returns
// `E_PRIMITIVE_NOT_IMPLEMENTED` until the executor lands.
const generateContent = subgraph('generate-content')
  .action('content:generate')
  .sandbox({ module: 'bafyr4i...ai-generate-module-cid', fuel: 200_000 })
  // Schema validation composes as BRANCH + RESPOND.
  .branch({ on: 'isValid($result, "contentType:post")' })
    .case('false', s => s
      .respond({ status: 500, body: '{ error: "generated content failed validation" }' }),
    )
    .case('true', s => s
      .write({
        label: 'post',
        properties: {
          title: '$result.title',
          body: '$result.body',
          status: 'draft',
          generatedBy: 'ai',
          generatedAt: 'now()',
          authorId: '$ctx.user.id',
        },
        requires: 'store:write:content/*',
      })
      .emit({ event: 'content:generated', payload: '{ cid: $result.cid, topic: $input.topic }' })
      .respond({ status: 201, body: '$result' }),
    )
  .endBranch()
  .build();
```

### Example 7: Batch Data Import

```typescript
import { subgraph } from '@benten/engine';

const importProducts = subgraph('import-products')
  .action('product:import')
  .iterate({ over: '$input.products', max: 5000 })
  // Per-item shape check as an inline BRANCH.
  .branch({ on: 'isValid($item, "contentType:product")' })
    .case('false', s => s
      .transform({ expr: '{ success: false, error: $error, index: $index }' })
      .respond({ edge: 'ON_ITEM_ERROR' }),
    )
    .case('true', s => s
      .write({ label: 'product', properties: '$item', requires: 'store:write:product/*' })
      .transform({ expr: '{ success: true, cid: $result.cid, index: $index }' }),
    )
  .endBranch()
  .transform({
    expr: '{ total: len($input.products), succeeded: count($results, r => r.success), failed: count($results, r => !r.success) }',
  })
  .respond({ body: '$result' })
  .build();
```

### Example 8: SEO Content Audit (Event-Triggered via SUBSCRIBE) *(Phase 2 executor)*

Event-triggered subgraphs compose via the SUBSCRIBE primitive. The Node registers and passes structural validation in Phase 1; the executor that fires the subgraph on event delivery ships in Phase 2.

```typescript
import { subgraph } from '@benten/engine';

const auditContentSeo = subgraph('audit-content-seo')
  .subscribe({ event: 'content:afterCreate' })
  .read({ label: 'post', by: 'cid', value: '$input.cid' })
  .sandbox({ module: 'bafyr4i...seo-score-module-cid', fuel: 50_000 })
  .write({
    label: 'seo_score',
    properties: {
      contentId: '$input.cid',
      overall: '$result.overall',
      readability: '$result.readability',
      keywords: '$result.keywords',
      suggestions: '$result.suggestions',
      scoredAt: 'now()',
    },
    requires: 'store:write:seo/*',
  })
  .emit({ event: 'seo:scored', payload: '{ contentId: $input.cid, score: $result.overall }' })
  .respond({})  // subscriber subgraphs still terminate with a RESPOND
  .build();
```

---

## Appendix A: Learning Ladder

Structure your learning path through the 12 primitives. Four of the twelve (`wait`, `stream`, `subscribe`, `sandbox`) have Phase 2 executors; the remaining eight execute today.

| Stage | Primitives | What You Can Build |
|-------|-----------|-------------------|
| **Hour 1** | `crud()` | Full CRUD handler for any label. One function call. |
| **Hour 2** | `read`, `write`, `respond` | Custom endpoints, typed error edges. |
| **Day 1** | `transform`, `branch`, `emit` | Data shaping, conditional logic, events. Composed VALIDATE via BRANCH. |
| **Day 2** | `call`, `iterate` | Subgraph composition and bounded batch processing. |
| **Week 2** | `subscribe`, `stream` | Event-triggered subgraphs and back-pressured output (Phase 2 executors). |
| **Advanced** | `sandbox`, `wait`, compensation pattern | WASM isolation, suspendable workflows, saga patterns (Phase 2 executors). |

Most module developers will spend 90% of their time in the first three stages. `sandbox` and `stream` are for advanced use cases.

## Appendix B: Quick Reference Card

```
subgraph(id)                    -- Create a new SubgraphBuilder
  .action(name)                 -- Declare an action string this handler exposes
  .read({ label, by, value })   -- Retrieve data (emits ON_NOT_FOUND / ON_EMPTY / ON_DENIED)
  .write({ label, properties,   -- Mutate data (emits ON_CONFLICT / ON_DENIED; `requires` gates the write)
           requires })
  .transform({ expr, as })      -- Reshape data (pure, no I/O)
  .branch({ on })               -- Open BRANCH; chain `.case(value, body).endBranch()`
    .case(value, body)          -- Add a case body (sub-scope with the same primitives)
    .endBranch()                -- Return to the parent builder
  .iterate({ over, max })       -- Bounded loop; body follows until next RESPOND
  .call({ handler, action,      -- Invoke another subgraph (emits ON_ERROR / ON_TIMEOUT)
          input, isolated })
  .respond({ body, edge,        -- Terminal: produce output. `edge` routes this RESPOND
             status })              as the handler for a specific typed error edge.
  .emit({ event, payload })     -- Fire-and-forget event

  // Phase 2 executors (build fine in Phase 1; evaluator returns E_PRIMITIVE_NOT_IMPLEMENTED):
  .wait({ duration })           -- Suspend until deadline
  .subscribe({ event, handler })-- Attach to a change-stream event
  .stream({ source, chunkSize })-- Partial output with back-pressure
  .sandbox({ module, fuel })    -- WASM sandbox with fuel metering

  .build()                      -- Materialize the Subgraph

crud(label, opts?)              -- Build a 5-action CRUD handler subgraph in one call
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

## Appendix D: Migration from the pre-revision draft

The DSL was originally designed around a 12-primitive set that included **VALIDATE** and **GATE**. After the 2026-04-14 critic review, those two primitives were dropped and **SUBSCRIBE** and **STREAM** were added, keeping the count at 12. The shipped DSL (described above) matches the revised set; this appendix documents the migration path for readers returning from the old draft.

### VALIDATE -> composed pattern

The old `validate(schema, opts).onInvalid(...)` primitive composes cleanly from primitives that already existed:

```typescript
// Pre-revision:
validate('contentType:post')
  .onInvalid(respond(400, { error: 'invalid', details: '$error.fields' }))

// Post-revision (shipped): BRANCH on schema predicate; TRANSFORM
// shapes the error detail; RESPOND with the 400; typed error edges do
// the routing. `isValid(obj, schemaId)` is a TRANSFORM built-in.
subgraph('x')
  .branch({ on: 'isValid($input, "contentType:post")' })
    .case('false', s => s
      .transform({ expr: '{ error: "invalid", details: validationErrors($input, "contentType:post") }' })
      .respond({ status: 400, body: '$result' }),
    )
    .case('true', s => s
      /* happy path */
    )
  .endBranch();
```

For schema validation at the handler-registration layer (not as a user-subgraph Node), register a validation function against the engine's 14 structural invariants; it runs at `registerSubgraph()` time and rejects malformed subgraphs before they are written. See [ENGINE-SPEC §4](ENGINE-SPEC.md#4-structural-invariants) for the invariant list.

### GATE -> `requires` property on any Node, plus SANDBOX / CALL / TRANSFORM for logic

The old `gate(handler, args)` had two conflated roles: **capability checking** and **custom-logic escape hatch**. Each role gets a distinct post-revision shape:

| Old shape | New shape |
|-----------|-----------|
| `gate('require:store:write:post/*')` | `write({ ..., requires: 'store:write:post/*' })` -- the `requires` property on any Node fires an automatic BRANCH to `ON_DENIED` when the configured policy (e.g. `PolicyKind.GrantBacked`) rejects it. No user-visible node. |
| `.require(capability)` builder method | Same: pass `requires` on the Node that needs the grant. For multi-step handlers, stamp `requires` on each mutating Node; the DSL does not require a separate "upfront" capability node. |
| `gate('commerce/calculateTax', { ... })` (business logic) | `call({ handler: 'commerce/calculate-tax', input: '{ ... }' })` -- register the logic as a subgraph and compose via CALL. |
| `gate('image/resize', { ... })` (untrusted / CPU-intensive) | `sandbox({ module: '...', fuel: 100_000 })` -- WASM with fuel metering (Phase 2 executor). |
| `gate('trim-title', { title: '$input.title' })` (one-liner reshape) | `transform({ expr: 'trim($input.title)' })` -- the expression language has 50+ built-in functions. |

The net effect: the "escape hatch" semantics that were undefined in the old GATE primitive (Open Question 8 in the pre-revision draft) are now split across three primitives with clear, separate contracts.

### SUBSCRIBE (new in the revised set)

SUBSCRIBE is the primitive that reactive change notification, IVM materialized views, sync delta propagation, and cross-module event handling all compose on. In the pre-revision draft these were implicit features of the engine (IVM was internal; event handlers were magic); the revision makes the underlying primitive explicit so each of those patterns is composable. See §2.11 above.

### STREAM (new in the revised set)

STREAM addresses a gap in the pre-revision set: partial output with back-pressure. RESPOND is terminal (no subsequent Nodes), ITERATE has no back-pressure, and SANDBOX exits all-at-once -- none of which compose cleanly into Server-Sent Events, WebSocket messages, LLM token streams, or large NDJSON responses. WinterTC targets make streaming table stakes for 2026 web APIs. See §2.12 above.

### Quick substitution table

| Pre-revision API | Post-revision (shipped) API |
|------------------|-----------------------------|
| `validate(schema).onInvalid(...)` | `branch({ on: 'isValid(obj, schema)' }).case('false', s => s.respond(...))` |
| `.require(cap)` | `write({ ..., requires: cap })` (or any Node that accepts `requires`) |
| `gate('handler', args)` | `call({ handler: 'handler', input: 'args' })` |
| `gate(<cpu-intensive>)` | `sandbox({ module: '...', fuel: N })` |
| `gate(<one-liner>)` | `transform({ expr: '...' })` |
| `respond(status, body)` | `respond({ status, body })` |
| `emit(event, payload)` | `emit({ event, payload })` |
| `read(label, { id })` | `read({ label, by: 'cid', value: '$input.cid' })` |
| `write(label, { data })` | `write({ label, properties })` |

All examples in sections 1-12 above use the post-revision (shipped) shapes; this appendix exists solely to help readers coming from the old draft.
