# Developer Experience Critique: Benten Engine Specification (Section 4.3)

**Reviewer:** Developer Experience Agent
**Date:** 2026-04-11
**Scope:** TypeScript API surface (Section 4.3), error handling across napi-rs boundary, in-process debugging, migration from current Thrum codebase
**Reference documents:** `docs/SPECIFICATION.md`, `CLAUDE.md`, existing Thrum codebase (`/Users/benwork/Documents/thrum`)

---

## DX Score: 4/10

The spec describes an ambitious and technically sound system, but the developer-facing API surface is under-designed. What exists in Section 4.3 reads like a C-style procedural interface pasted into TypeScript — it does not build on the patterns and ergonomics that the existing Thrum codebase has already established. A module developer who is productive in Thrum V3 today would face significant friction adopting this API.

---

## 1. Onboarding Assessment (First 30 Minutes)

**What works:**
- The README is clear about what this project IS and IS NOT ("This is not a database").
- CLAUDE.md gives a good mental model: "graph IS the runtime," "answers before questions."
- The crate structure is logical and well-named.
- The "What Gets Replaced" table (Section 3.2) is genuinely helpful for understanding the migration story.

**What does not work:**
- Section 4.3 is the ONLY code a TypeScript developer sees, and it is 40 lines of untyped function signatures with no JSDoc, no examples, no error documentation, and no indication of what types `NodeId`, `EdgeId`, `ViewId`, `SubscriptionId`, `Value`, `Capability`, `EntityId`, `PeerConnection`, `TraversalPattern`, `ChangeEvent`, `SyncResult`, `QueryResult`, or `VersionNode` actually are. A new developer reads this section and has more questions than answers.
- There is no "Hello World" equivalent. No "here is how a module developer creates a content type and queries it." The first 30 minutes would be spent reading the spec, not writing code.
- The open questions (Section 8) are high-impact architectural decisions that remain unresolved. A developer cannot even start prototyping a module until question 1 ("Cypher as primary API vs Rust-native API?") is answered.

---

## 2. API Ergonomics Analysis

### Issue 1: Raw Cypher as the Query Interface (Critical)

```typescript
engine.query(cypher: string, params?: Record<string, Value>): QueryResult
```

This is the most consequential DX decision in the entire spec, and it is the wrong default for module developers.

**The problem:** The current Thrum codebase has TWO query interfaces:
1. `store.query(tableId, options)` -- a structured, typed query builder with `where`, `orderBy`, `limit`, `offset`, `search`, and `aggregate`. This is what module developers use 95% of the time. It is safe, composable, and produces typed results.
2. `store.raw(query, params, mode, secCtx)` -- raw SQL/Cypher for platform-tier code that needs escape hatches. Gated behind `SecurityContext` and tier enforcement.

The spec collapses both into `engine.query(cypher, params)`. This means a community-tier module developer who wants to list products sorted by price must write:

```typescript
const products = engine.query(
  'MATCH (p:Product) WHERE p.status = $status RETURN p ORDER BY p.price LIMIT $limit',
  { status: 'active', limit: 20 }
);
```

Instead of the current:

```typescript
const products = await store.query('products', {
  where: { status: 'active' },
  orderBy: { field: 'price', direction: 'asc' },
  limit: 20,
});
```

The structured API is safer (no injection risk), discoverable (IDE autocomplete on `where`, `orderBy`, `limit`), and testable (you can mock the options, not parse Cypher strings). Cypher should be the POWER USER escape hatch, not the default.

**Recommendation:** Carry forward the `Store` interface's structured query pattern as the primary API. Cypher becomes `engine.rawQuery()` -- available but not the first thing you reach for.

### Issue 2: No Type Safety Across the Boundary (Critical)

```typescript
engine.createNode(labels: string[], properties: Record<string, Value>): NodeId
```

This returns `NodeId` (a string alias). There is no way to express "I created a Product node and I want a Product-typed result back." Compare with:

- **tRPC**: The return type is inferred from the resolver. You write `trpc.product.create.mutate(data)` and TypeScript knows the exact shape.
- **Payload CMS**: `payload.create({ collection: 'products', data })` returns a typed `Product` document.
- **Current Thrum**: `defineContentType()` defines the schema, and `contentTypeToValibot()` generates runtime validation. The type flows through.

The spec's API is stringly-typed throughout. `labels: string[]`, `type: string`, `properties: Record<string, Value>`. There is no generic parameter, no schema binding, no way for TypeScript to catch "you passed `price` as a string but the schema says number."

**Recommendation:** Design a typed Node creation API:

```typescript
// Schema-bound creation
const product = engine.create<Product>('Product', { name: 'Widget', price: 9.99 });
// Returns typed Product, validates against registered schema at runtime
```

Or at minimum, carry forward the `defineContentType` + `TableDeclaration` pattern where schemas are registered and the engine validates writes against them.

### Issue 3: `engine.open()` / `engine.create()` Naming Ambiguity (Moderate)

```typescript
engine.open(path: string): void   // disk-backed
engine.create(): void              // in-memory
```

`engine.create()` does not create an engine -- it creates an in-memory database. `engine.open()` opens a disk-backed one. This naming conflates lifecycle management with storage mode.

Compare with better patterns:
- SQLite: `new Database(':memory:')` vs `new Database('./data.db')`
- PGlite: `new PGlite()` vs `new PGlite('./pgdata')`
- redb: `Database::create(path)` vs `Database::open(path)` (but this is Rust-side)

**Recommendation:** Unify into a single constructor:

```typescript
const engine = await BentenEngine.open({ storage: 'memory' });
const engine = await BentenEngine.open({ storage: { path: './data' } });
```

Or follow the PGlite pattern which the Thrum team already knows:

```typescript
const engine = new BentenEngine();           // in-memory
const engine = new BentenEngine('./data');   // disk-backed
```

### Issue 4: Materialized Views Require Cypher Knowledge (Moderate)

```typescript
engine.createView(name: string, query: string): ViewId
```

IVM is the spec's "key innovation," but it is exposed as "write a Cypher string." The module developer who wants "all active products sorted by price" must know Cypher to get O(1) reads.

The current Thrum pattern is better here: `QueryOptions` objects describe what you want, and the system figures out how. The engine should automatically maintain views for common patterns (content listings, event handler lookup, capability checks) and provide a declarative API for custom views.

**Recommendation:**

```typescript
// Declarative view definition
engine.defineView('activeProducts', {
  label: 'Product',
  where: { status: 'active' },
  orderBy: { field: 'price', direction: 'asc' },
});

// O(1) read
const products = engine.readView('activeProducts');
```

Cypher-based views are the escape hatch for complex graph traversals, not the primary interface.

### Issue 5: Subscription Callbacks Have No Error Contract (Moderate)

```typescript
engine.subscribe(query: string, callback: (change: ChangeEvent) => void): SubscriptionId
```

What happens when the callback throws? Is it retried? Is it removed? Is the error swallowed? The current Thrum event system has clear semantics: `emit` uses `Promise.allSettled` (errors are swallowed), `pipeline` propagates errors, `filter` is synchronous. The spec says nothing about error handling in reactive subscriptions.

**Recommendation:** Define the contract. Options:
1. Callback errors are caught and reported to a configurable error handler (like `window.onerror`)
2. Callback errors cause the subscription to be paused, with a dead-letter queue
3. Callback errors cause the subscription to be removed (harsh but simple)

Document it explicitly in the API surface.

---

## 3. Error Message Audit: napi-rs Boundary Crossing

This is the spec's biggest DX blind spot. There is zero discussion of how errors propagate from Rust to TypeScript.

### The Problem

When napi-rs throws, the JavaScript side gets a generic `Error` with a message string. The current Thrum codebase has a rich error model:

```typescript
class EngineError extends Error {
  code: EngineErrorCode;  // 'NOT_FOUND' | 'SCHEMA_VIOLATION' | 'CYCLE_DETECTED' | ...
  source: string;         // '@benten/engine' | '@benten/store-postgres' | ...
  nodeId?: string;        // The node involved
}
```

17 error codes, each actionable. The `source` field tells you which package threw. The `nodeId` field tells you which node is involved. Module developers write:

```typescript
try {
  await store.updateRecord('products', id, data, { expectedVersion: 3 });
} catch (err) {
  if (isEngineError(err) && err.code === 'VERSION_CONFLICT') {
    // Handle optimistic locking failure
  }
}
```

napi-rs default error handling does NOT preserve this structure. It produces:

```typescript
try {
  engine.createNode(['Product'], { name: 'Widget' });
} catch (err) {
  // err is a plain Error with message: "EngineError: SCHEMA_VIOLATION: ..."
  // err.code is undefined
  // err.source is undefined
  // isEngineError(err) returns FALSE
}
```

### What Needs to Happen

1. **Structured error serialization**: The Rust side must serialize errors as JSON: `{ code, message, source, nodeId }`. The napi binding must deserialize this into an `EngineError` instance (not a plain `Error`).
2. **`isEngineError()` must work across the boundary**: The existing type guard checks `instanceof EngineError`. If the napi binding constructs a new `EngineError` from the Rust error, this works. If it throws a plain `Error`, every error handler in the existing codebase breaks.
3. **Stack traces**: Rust panics produce Rust stack traces. These are useless to a TypeScript developer. The napi binding must catch Rust panics and convert them to JavaScript errors with the Rust source location as metadata, not as the stack trace.
4. **Error codes must be shared**: The 17 error codes in `EngineErrorCode` must be defined in Rust and re-exported to TypeScript. The napi `@napi-rs/cli` generates `.d.ts` files, but only for function signatures -- error types need manual bridging.

### Specific Recommendation

Add a Section 4.4 "Error Model" to the spec:

```rust
// Rust side
#[derive(Debug, Serialize)]
pub struct EngineError {
    pub code: ErrorCode,
    pub message: String,
    pub source: String,
    pub node_id: Option<String>,
}

// napi binding converts to:
// new EngineError(code, message, source, nodeId)
```

---

## 4. In-Process Debugging

The spec's in-process design (no external PostgreSQL) has debugging implications that are not addressed.

### Issue 6: No Query Inspector / Explain (Significant)

With PostgreSQL, developers use `EXPLAIN ANALYZE` to understand query plans. With an in-process engine, there is no equivalent mentioned. When a materialized view read is slow (violating the <0.01ms target), how does a developer diagnose it?

**Recommendation:** Add `engine.explain(query)` that returns the query plan and IVM dependency graph. Add `engine.stats()` that returns view counts, cache sizes, and write amplification metrics.

### Issue 7: No Data Browser (Significant)

PostgreSQL has pgAdmin, DataGrip, psql. The in-process engine has... nothing. A module developer who wants to inspect the graph state during development has no tool.

**Recommendation:** Either:
1. Build a web-based inspector (like PGlite's planned DevTools)
2. Export the graph in a format compatible with existing graph visualization tools (Neo4j Browser, Gephi)
3. At minimum, provide `engine.dump()` that serializes the graph to JSON for debugging

### Issue 8: WASM Debugging is Still Painful

The spec targets WASM (browsers/edge). WASM debugging in 2026 has improved but is still far behind JavaScript debugging. Source maps work via DWARF, but breakpoints in Rust code viewed through Chrome DevTools are a poor experience compared to stepping through TypeScript.

When a module developer's code interacts with the engine in a browser context:
- Error stack traces will show WASM function indices, not readable function names (unless DWARF is enabled)
- Console.log in the engine is not straightforward -- you must use `web_sys::console::log_1`
- Memory issues (leaks, corruption) manifest as opaque WASM traps

**Recommendation:** Acknowledge this in the spec. Plan for:
1. Human-readable error messages that do not require reading WASM stack traces
2. A JavaScript-side debug logging system that the engine can emit to
3. A `BentenEngine.debug()` mode that enables verbose logging without requiring WASM source map setup

---

## 5. Migration from Current Thrum Codebase

### Issue 9: The Migration Path is Undefined (Critical)

Section 3 says what gets replaced but not HOW. The current Thrum codebase has:
- **~2,900+ tests** that depend on the existing Store interface, EventBus, EngineError model
- **15 packages** with deep integration into PostgreSQL-specific patterns (Drizzle ORM, Apache AGE Cypher, PGlite for testing)
- **9 domain modules** that use `defineModule()`, `onRegister()`, `store.query()`, `store.insertRecord()`, etc.

The spec proposes replacing the Store interface, the event system, the error model, and the query language. This is not a migration -- it is a rewrite of every module's data access layer.

**What needs answering:**
1. **Can the existing `Store` interface be an adapter over the engine?** If `store.query(tableId, options)` maps to engine operations under the hood, modules do not need to change. This is the critical migration question.
2. **Will `defineModule()` still work?** The spec says "defineModule -> writes to graph, lifecycle hooks" but does not show what this looks like.
3. **What happens to Drizzle ORM?** 6 modules use Drizzle `PgTableWithColumns`. The rest use `TableDeclaration`. Both need a migration path.
4. **What happens to tests?** PGlite fallback is used in 11+ packages. The engine replaces PGlite, but the test patterns need to work with the in-process engine.

**Recommendation:** Add a Section 9 "Migration Strategy" with three phases:
1. **Phase 1 (Adapter):** The engine implements the existing `Store` interface. `store.query()` works. `store.insertRecord()` works. All 2,900+ tests pass against the engine. Zero module changes.
2. **Phase 2 (Enhancement):** New engine-native APIs (IVM views, reactive subscriptions, version chains) are available alongside the Store adapter. Modules opt in incrementally.
3. **Phase 3 (Deprecation):** Once all modules use engine-native APIs, the Store adapter is removed.

### Issue 10: The `Record<string, Value>` Type Regression

The existing Thrum codebase uses `Record<string, unknown>` for node properties and query results. The spec uses `Record<string, Value>` where `Value` is a closed union (`String | i64 | f64 | bool | Vec<Value> | HashMap<String, Value> | null`).

This is actually stricter than the current system, which allows any JSON-serializable value. Module developers who store complex nested objects (like `BlockInstance[]` in compositions) would need to restructure their data. The existing `config?: Record<string, JsonValue>` on `Node` is close but the Rust `Value` type needs to match exactly.

**Recommendation:** Ensure the Rust `Value` enum maps 1:1 to `JsonValue` from `@benten/engine`. Document the mapping table in the napi binding layer.

---

## 6. Discoverability Gaps

1. **No TypeScript type definitions in the spec.** Section 4.3 shows function signatures but not the 13+ types they reference (`NodeId`, `EdgeId`, `ViewId`, etc.). A developer cannot use IDE autocomplete to explore the API.

2. **No subpath export plan.** The current `@benten/engine` has 7 subpath exports (`./registry`, `./dag`, `./events`, `./store`, `./auth`, `./materializer`, root). The spec does not say how the Rust engine's TypeScript bindings will be organized as npm packages or subpaths.

3. **No example of a complete module.** The current Thrum codebase has `@benten/seo` as a dogfood module (92 lines, clear lifecycle). The spec has no equivalent showing "here is what a module looks like after migration."

4. **No error code catalog.** The current codebase has 17 error codes with implicit conventions (e.g., `NOT_FOUND` for missing entities, `VERSION_CONFLICT` for optimistic locking). The spec does not define error codes for the new engine operations (e.g., what error code does `engine.rollback()` throw when the version does not exist?).

5. **Capability model has no examples.** The spec describes capabilities abstractly but does not show a concrete example: "Module X needs capability Y to do Z. Here is how the operator grants it. Here is the error when it is missing."

---

## 7. DX Improvements (Prioritized by Impact)

| Priority | Improvement | Impact | Effort |
|----------|------------|--------|--------|
| P0 | Define the Store adapter migration path (Section 9) | Unlocks migration without module rewrites | Design |
| P0 | Carry forward structured query API alongside Cypher | Module developers never need to learn Cypher for CRUD | Medium |
| P0 | Design error serialization across napi-rs boundary | Existing error handling code works unchanged | Medium |
| P1 | Add full TypeScript type definitions to spec (all 13+ types) | Developers can understand the API without reading Rust | Low |
| P1 | Add typed Node creation with schema binding | End-to-end type safety (the tRPC lesson) | Design |
| P1 | Provide `engine.explain()` and `engine.stats()` for debugging | Developers can diagnose performance issues | Medium |
| P2 | Declarative view definition API (not just Cypher) | IVM accessible to non-graph-experts | Medium |
| P2 | Unify `open()`/`create()` into single constructor | Less cognitive load | Low |
| P2 | Add error code catalog with examples | Developers know what to catch and when | Low |
| P2 | Provide data browser / graph dump for debugging | Developers can inspect state during development | High |
| P3 | Document WASM debugging story | Sets expectations for browser-side development | Low |
| P3 | Show complete module example (post-migration SEO module) | Concrete migration target for module developers | Low |
| P3 | Define subscription error contract | Prevents subtle bugs in reactive code | Low |

---

## 8. What Developers Love (Lessons from Research)

The frameworks that developers love share these traits:

| Framework | Why Developers Love It | Lesson for Benten Engine |
|-----------|----------------------|--------------------------|
| **tRPC** | Zero-codegen end-to-end type safety. Change the server, the client breaks at compile time. | The engine's TypeScript bindings must preserve types across the napi-rs boundary. Stringly-typed APIs kill DX. |
| **Payload CMS** | Local API eliminates HTTP layers. Schema-as-code generates everything. Lightweight -- no admin panel unless you want it. | Carry forward Thrum's `defineContentType` pattern. The engine should feel like a function call, not a database connection. |
| **SvelteKit** | File-based routing with full type inference. `load()` functions have typed return values that flow to components. | The engine's reactive subscriptions should integrate with SvelteKit's load/invalidation pattern, not fight it. |
| **Remix** | Nested routes with typed loaders. `useLoaderData()` is typed from the server. Error boundaries are compositional. | Error boundaries at the engine level: each materialized view should have a defined error state, not just throw. |

The common thread: **types flow, errors are structured, and the default path is safe.** The spec's current API surface is typeless, error-unaware, and defaults to raw Cypher. It needs to be inverted.

---

## Summary

The Benten Engine specification describes a technically impressive system. The Rust architecture is sound, the IVM concept is genuinely innovative, and the vision of a self-sovereign data runtime is compelling. But the TypeScript API surface -- the part that module developers actually touch -- is the weakest part of the document.

The five critical issues:
1. Raw Cypher as the default query interface (should be structured queries with Cypher as escape hatch)
2. No type safety across the napi-rs boundary (stringly-typed throughout)
3. No error model for the Rust-to-TypeScript boundary (existing `EngineError` patterns will break)
4. No migration strategy (2,900+ tests, 15 packages, 9 modules need a path)
5. No debugging story for an in-process engine (no explain, no data browser, no inspector)

Fix these five and the DX score goes from 4 to 7. Add the typed Node creation, declarative views, and subscription error contracts and it reaches 8-9.

The engine should feel like a better version of the existing Thrum Store interface, not a different paradigm that module developers must relearn.
