# Composability & Extensibility Critique: Benten Engine Specification

**Date:** 2026-04-11
**Reviewer perspective:** Third-party developer trying to extend the engine without modifying its source
**Scope:** `/Users/benwork/Documents/benten-engine/docs/SPECIFICATION.md`, `CLAUDE.md`, and cross-reference with Thrum TypeScript codebase

---

## Plugin Readiness Score: 3/10

The specification describes a powerful runtime but is almost entirely silent on how external code extends it. Every extension point that exists in the TypeScript Thrum system (custom field types, custom blocks, module routes, event hooks, theme tokens, admin UI sections, content type extensions) has no specified analogue in the Rust engine. The engine is designed as a monolithic computation substrate, not a composable platform that third parties can extend.

---

## 1. What Works End-to-End (or Could)

### 1.1 Graph Data Model Is Inherently Extensible

The Node/Edge/Label/Property model is schema-free by design. Any module can create Nodes with arbitrary labels and properties without engine modifications. This is a genuine strength -- the data model itself is open.

**Trace:** Module creates `Node { labels: ["SEOScore"], properties: { score: 85, url: "/about" } }` via `engine.createNode()`. No engine change needed. Edge creation to link SEO scores to content Nodes also works. The API surface in Section 4.3 supports this.

### 1.2 Cypher as Extension Language

Cypher queries are strings evaluated at runtime. Any module can compose arbitrary graph patterns without compile-time coupling to the engine. A newsletter plugin can write `MATCH (s:Subscriber)-[:SUBSCRIBED_TO]->(l:List {name: 'weekly'}) RETURN s` without the engine knowing what subscribers or lists are.

### 1.3 Materialized Views as a Module Primitive

`engine.createView(name, query)` allows modules to define their own pre-computed views. An SEO plugin could create a view for "all pages missing meta descriptions" and read it in O(1). This is a powerful extension mechanism that goes beyond what the TypeScript codebase offers.

### 1.4 Reactive Subscriptions for Event-Driven Modules

`engine.subscribe(query, callback)` replaces the explicit event bus registration. A newsletter module can subscribe to `MATCH (c:Content {type: 'post', status: 'published'}) RETURN c` and get notified when new posts are published. This is structurally cleaner than string-keyed event names.

---

## 2. Where Plugins Hit Walls

### Issue 1: No Storage Backend Abstraction (Critical)

**Score impact:** -2 points

The specification names redb as the persistent storage backend (Section 4.2) but provides no trait/interface for swapping it. The CLAUDE.md lists "redb (pure Rust embedded KV)" as the storage layer. There is no `StorageBackend` trait, no `PersistenceProvider` interface, nothing.

**Why this matters:** The TypeScript Thrum system already solved this problem. The `Store` interface (17 methods, defined in `@benten/engine/store`) is implementation-agnostic -- `@benten/store-postgres` is one implementation, and others could be written. The Rust engine specification loses this.

**What happens when someone needs RocksDB:** They fork `benten-persist/`. There is no way to swap the storage backend without modifying engine source code. This is the most basic composability requirement for a database engine, and it is missing.

**What redb itself provides:** redb is a key-value store with typed tables (`TableDefinition<K, V>`). Custom key/value types can implement `Key`/`Value` traits. But the benten-engine spec does not expose this -- there is no documented way for a module to define custom storage types or index strategies that leverage redb's type system.

**Recommendation:** Define a `PersistenceBackend` trait in `benten-persist` with methods for read/write/scan/transaction. Make redb one implementation. This mirrors the TypeScript pattern where `Store` is the interface and `store-postgres` is the implementation.

```rust
// benten-persist/src/lib.rs
pub trait PersistenceBackend: Send + Sync {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, table: &str, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&self, table: &str, key: &[u8]) -> Result<()>;
    fn scan(&self, table: &str, range: impl RangeBounds<Vec<u8>>) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    fn transaction<F, T>(&self, f: F) -> Result<T> where F: FnOnce(&dyn TransactionOps) -> Result<T>;
}
```

---

### Issue 2: No Custom Index Types (Critical)

**Score impact:** -1.5 points

Section 4.1 mentions `benten-graph/` handles "Graph storage, indexes (hash + B-tree), traversal." Two index types are specified: hash and B-tree. There is no trait for defining custom index types.

**Why this matters:** Consider these real module needs:
- **Full-text search plugin:** Needs an inverted index. Hash and B-tree cannot serve full-text queries.
- **Geospatial plugin:** Needs R-tree or H3 spatial indexes. Neither hash nor B-tree supports "find all nodes within 5km."
- **Vector/embedding plugin (AI):** Needs approximate nearest neighbor indexes (HNSW, IVF). This is the hottest use case in 2026 -- every CMS platform is adding vector search.
- **Time-series plugin:** Needs time-partitioned indexes for efficient range scans over temporal data.

**What competitors offer:** PostgreSQL has CREATE INDEX ... USING (btree, hash, gist, gin, brin, spgist) and custom index methods via extension API. SurrealDB supports full-text, vector (HNSW, M-tree), and geospatial indexes. Even SQLite has FTS5 and R-tree as loadable extensions.

**Recommendation:** Define an `IndexType` trait:

```rust
pub trait IndexType: Send + Sync {
    fn name(&self) -> &str;
    fn build(&self, entries: &[(NodeId, Value)]) -> Result<Box<dyn Index>>;
    fn insert(&self, index: &mut dyn Index, key: NodeId, value: Value) -> Result<()>;
    fn query(&self, index: &dyn Index, predicate: &IndexPredicate) -> Result<Vec<NodeId>>;
}
```

---

### Issue 3: No Custom CRDT Merge Strategies (High)

**Score impact:** -1 point

Section 2.5 specifies exactly two merge strategies:
- Node properties: per-field last-write-wins (LWW) with Hybrid Logical Clocks
- Edges: add-wins semantics

These are hardcoded. There is no mechanism for modules to define custom merge behavior.

**Why this matters:**
- **Collaborative text editing:** Needs CRDT text types (RGA, Fugue) not LWW. If two users edit the same rich text field, LWW throws away one user's changes entirely.
- **Counters and accumulators:** A "view count" or "stock quantity" field needs a PN-Counter CRDT, not LWW. LWW on a counter produces wrong results under concurrent increments.
- **Set-valued properties:** Tags, categories, permissions lists -- these need OR-Set or Add-Wins-Set semantics, not LWW per field.
- **Custom domain types:** A game engine module might need a custom merge strategy for game state (e.g., "highest score wins" not "last write wins").

**What the CRDT ecosystem provides:** Automerge 2.0 supports text, lists, maps, counters, and timestamps as built-in types. Yrs (Yjs Rust port) supports Text, Array, Map, and XmlFragment. Both are extensible through their type systems. Loro (another Rust CRDT library) supports custom codecs and aggregators via traits.

**Recommendation:** The specification already lists "yrs or automerge" as a CRDT dependency. Either of these provides richer type support than the spec describes. Surface their extensibility:

```rust
pub trait MergeStrategy: Send + Sync {
    fn merge(&self, local: &Value, remote: &Value, metadata: &MergeContext) -> Result<Value>;
    fn supports_type(&self) -> &str; // "text", "counter", "set", etc.
}
```

Allow modules to register merge strategies for specific property types (annotated on the Node schema or property key). Default to LWW when no custom strategy is registered.

---

### Issue 4: No Custom View Types or IVM Extension Points (High)

**Score impact:** -1 point

Section 2.2 describes IVM but the only view creation API is `engine.createView(name, query)` where `query` is a Cypher string. There are several missing extension points:

**4a: No custom aggregation functions.** A finance module needs `SUM(revenue)` grouped by month. A statistics module needs `PERCENTILE(score, 0.95)`. The specification does not mention whether Cypher views support aggregation, or whether custom aggregation functions can be registered.

**4b: No view update hooks.** When a materialized view is incrementally updated, there is no mechanism for a module to intercept the update (e.g., to send a notification, update a secondary cache, or trigger a side effect).

**4c: No custom view materialization strategies.** The spec says "the engine identifies which views are affected" and "incrementally updated." But some views might need custom update logic. For example, a view that computes "trending content" based on a time-decay formula cannot be expressed as a simple query -- it needs a custom incremental maintenance function.

**Recommendation:** Add a `ViewMaintainer` trait for custom view types, and add before/after hooks on view updates:

```rust
pub trait ViewMaintainer: Send + Sync {
    fn initial_compute(&self, graph: &GraphSnapshot) -> Result<ViewResult>;
    fn on_change(&self, graph: &GraphSnapshot, changes: &[GraphChange]) -> Result<ViewDelta>;
}
```

---

### Issue 5: No Module Behavior Extension for Nodes and Edges (High)

**Score impact:** -1 point

Nodes and Edges are plain data containers: id, labels, properties. There is no mechanism for modules to attach behavior to them:

- **Validation hooks:** No way for a module to say "before creating a Node with label 'Product', validate that it has a 'price' property > 0." The TypeScript system has `contentTypeToValibot()` for schema validation on content writes; the Rust spec has no equivalent.
- **Computed properties:** No way to define a property that is computed from other properties (e.g., `fullName = firstName + " " + lastName`). The IVM system could theoretically support this, but there is no API for it.
- **Lifecycle hooks on Node/Edge operations:** No before/after hooks on create/update/delete. The TypeScript system has `pipeline('content:beforeCreate', data)` and `emit('content:afterCreate', data)`. The Rust spec replaces this with reactive subscriptions, but subscriptions are after-the-fact notifications -- there is no way to intercept and transform or reject a write before it happens.

This last point is critical. The TypeScript `pipeline` and `filter` dispatch modes let modules **transform data before it is written** and **reject writes**. Reactive subscriptions only notify after the fact. The spec loses the interception capability.

**Recommendation:** Add a write-interception layer:

```rust
pub trait WriteInterceptor: Send + Sync {
    /// Called before a node is created. Return Err to reject, Ok(modified) to transform.
    fn before_create_node(&self, labels: &[Label], props: &Properties) -> Result<Properties>;
    /// Called before a node is updated.
    fn before_update_node(&self, id: NodeId, props: &Properties) -> Result<Properties>;
    /// Called before a node is deleted. Return Err to prevent deletion.
    fn before_delete_node(&self, id: NodeId) -> Result<()>;
}
```

---

### Issue 6: Crate Coupling -- The Orchestrator Problem (Medium)

**Score impact:** -0.5 points

The crate structure shows 10 crates where `benten-engine` is the "Orchestrator: ties everything together." This creates a coupling bottleneck. Every crate is designed to be consumed by the orchestrator, but there is no evidence that the crates are independently usable or that the orchestrator can be extended without modification.

**Questions the spec does not answer:**
- Can I use `benten-graph` without `benten-capability`? (For a use case where I do not need capability enforcement.)
- Can I add a new crate (e.g., `benten-fulltext`) and wire it into the orchestrator without modifying `benten-engine/src/`?
- Is there a plugin/extension point on the orchestrator itself?

**Comparison with Thrum TypeScript architecture:** The TypeScript `@benten/engine` package exports interfaces. `@benten/store-postgres` implements the Store interface. `@benten/core` provides the module system. `@benten/cms` adds CMS domain logic. Each package can be used independently. The Rust crate structure, by contrast, appears to flow one way: everything feeds into the orchestrator.

**Recommendation:** Define the `benten-engine` orchestrator as a composition of trait implementations, not a monolith. Use a builder pattern:

```rust
let engine = EngineBuilder::new()
    .with_persistence(RedbBackend::new(path))
    .with_capability(UcanCapabilitySystem::new())
    .with_sync(AutomergeSyncEngine::new())
    .with_index(FullTextIndex::new())  // custom index extension
    .build()?;
```

This makes the orchestrator a configurable composition rather than a hardcoded assembly.

---

### Issue 7: Rust Is Inherently Less Composable at Runtime Than TypeScript (Medium)

**Score impact:** -0.5 points

This is not a flaw in the specification per se, but a fundamental tension the spec does not address.

**TypeScript composability model:**
- Open union types (`FieldDefinition` has 17 known types + catch-all)
- Declaration merging (modules augment `EventMap` at compile time)
- Dynamic imports (modules loaded at runtime from npm packages)
- `createRegistry<T>()` with generic type parameters
- No compilation step for plugins (just `npm install` and import)

**Rust composability model:**
- Trait objects (`Box<dyn Trait>`) for runtime polymorphism -- vtable overhead, no generic associated types through trait objects
- Feature flags for compile-time feature selection -- requires recompilation
- Procedural macros for code generation -- complex, hard to debug
- Dynamic loading via `libloading` / `dlopen` -- unsafe, platform-specific, ABI fragile
- WASM modules for sandboxed extensions -- serialization overhead at boundary

The spec does not acknowledge this difference or describe how the "thousands of developers building modules" vision (from the Thrum codebase) translates to a Rust binary. In the TypeScript world, a plugin is `npm install @acme/seo-plugin` and an import. In the Rust world, a plugin is either:

1. **Compiled in:** Requires adding to Cargo.toml and recompiling. Not suitable for runtime extension.
2. **Loaded dynamically:** Requires stable ABI, unsafe code, and careful version management. Fragile.
3. **WASM module:** Clean but limited -- WASM modules cannot access Rust traits, hold references to Rust objects, or use async Rust directly.

**The spec assumes option 1 (compiled in) without stating it.** All 10 crates are in the same Cargo workspace. There is no discussion of a plugin loading mechanism.

**Recommendation:** The spec needs a "Plugin Model" section that explicitly addresses:
- How third-party code is loaded (compiled-in crates? WASM modules? dynamic libraries?)
- What the plugin interface looks like (traits? message passing? Cypher-only?)
- What performance tradeoff is acceptable (WASM serialization overhead vs native speed)
- Whether there are multiple plugin tiers (WASM for untrusted, compiled crate for trusted)

The most practical approach for 2026, given the WASM ecosystem maturity, is a two-tier model:
- **Core extensions:** Rust crates compiled into the engine (custom indexes, storage backends, merge strategies)
- **Application extensions:** WASM modules that interact via the engine's API (Cypher queries, views, subscriptions, capability-gated node/edge CRUD)

---

### Issue 8: No TypeScript-Side Extension Mechanism Specified (Medium)

**Score impact:** -0.5 points

Section 4.3 shows the TypeScript API surface, but it is a flat procedural API (createNode, getNode, createView, etc.). There is no:

- Module registration mechanism (the TypeScript `defineModule()` pattern)
- Namespace scoping (the TypeScript `withScope()` pattern)
- Capability granting during registration (the `OnRegisterContext` pattern)
- Lifecycle hooks (onRegister, onMigrate, onBootstrap, onDestroy)

The Thrum TypeScript codebase has a mature module system (decomposed into 8 files in `@benten/core/modules/`) with topological dependency resolution, IoC cleanup, settings persistence, and event scoping. The Rust engine spec says the TypeScript layer "does NOT contain storage logic, event dispatch, capability checks, version management, sync logic" -- but it also does not specify how the TypeScript layer retains module system functionality.

**Where modules currently live vs where they would live:**

| Concern | Current (TypeScript) | Proposed (Rust Engine) | Gap |
|---------|---------------------|----------------------|-----|
| Content type definitions | `defineContentType()` + registry Map | Nodes in graph | How are schemas validated? |
| Block definitions | `defineBlock()` + registry Map | Nodes in graph | How are block props validated? |
| Event handlers | `bus.onEmit('content:afterCreate', fn)` | `engine.subscribe(query, callback)` | How do write interceptors work? |
| Field types | `defineFieldType()` + registry | Nodes in graph? | How is `toSchema()`/`toColumn()` stored? |
| Route definitions | `ThrumModule.routes` | Not specified | Where do module API routes live? |
| Admin UI sections | `defineAdminSection()` | Not specified | How do modules extend the admin panel? |
| Theme tokens | `defineTheme()` + DTCG tokens | Not specified | Where do design tokens live? |
| Module settings | `module_settings` table + Valibot validation | Settings as graph Nodes | How is Valibot validation preserved? |

The spec says "definitions ARE the graph" (Section 3.1) but does not address how executable code (validation functions, column builders, UI components) can be stored in or referenced from graph Nodes.

**Recommendation:** The spec needs a "Module System" section that describes:
- How TypeScript modules register themselves with the engine
- How the existing ThrumModule lifecycle hooks (onRegister/onMigrate/onBootstrap/onDestroy) map to engine operations
- How executable code (validators, transformers, UI components) coexists with graph data
- Whether the module system lives in TypeScript (using the engine as storage) or in the engine (with TypeScript bindings)

---

## 3. Missing Extension Points Summary

| Extension Point | Status | Priority |
|----------------|--------|----------|
| Custom storage backends (redb -> RocksDB -> custom) | Missing | Critical |
| Custom index types (full-text, vector, spatial, time-series) | Missing | Critical |
| Custom CRDT merge strategies (text, counters, sets) | Missing | High |
| Write interception (before-create/update/delete hooks) | Missing | High |
| Custom view types / IVM extensions | Missing | High |
| Plugin loading mechanism (WASM / dynamic / compiled) | Missing | High |
| Custom aggregation functions for views | Missing | Medium |
| View update hooks (post-IVM-update notifications) | Missing | Medium |
| Node/Edge schema validation hooks | Missing | Medium |
| Computed property definitions | Missing | Medium |
| Module system specification (lifecycle, registration, scoping) | Missing | Medium |
| Custom Cypher functions | Missing | Low |
| Custom serialization formats for sync | Missing | Low |

---

## 4. Comparison to Competitors

### Payload CMS 3.x (TypeScript)

Payload's plugin system lets plugins:
- Add collections (content types) with full field definitions
- Add custom field types with admin UI components
- Register hooks (beforeChange, afterChange, beforeRead, afterRead, beforeDelete, afterDelete)
- Add custom REST and GraphQL endpoints
- Extend the admin panel with React components
- Access the full API context (req, user, locale) in hooks

**What Benten Engine lacks:** Hooks (write interception), admin UI extension mechanism, custom endpoint registration.

### Strapi 5 (TypeScript)

Strapi plugins:
- Register custom fields via server + admin panel registration
- Define content types and components
- Access lifecycle hooks (beforeCreate, afterCreate, beforeUpdate, etc.) on models
- Extend the admin panel via the Admin Panel API (register, bootstrap, registerTrads)
- Add custom controllers, services, routes, policies, and middlewares

**What Benten Engine lacks:** Lifecycle hooks on data operations, custom field type registration, admin panel extension API, middleware/policy system.

**Key limitation in Strapi that Benten avoids:** Strapi custom fields "cannot add new data types" -- they must use existing built-in types. Benten's schema-free graph model does not have this limitation; any property type is valid.

### SurrealDB (Rust)

SurrealDB is a closer architectural peer (Rust, multi-model, embedded):
- Custom functions via DEFINE FUNCTION
- Custom analyzers for full-text search
- Multiple index types (unique, search/full-text, vector HNSW, vector M-tree)
- Record-level access control via DEFINE ACCESS
- Event triggers on table changes (DEFINE EVENT ... WHEN ... THEN ...)
- Live queries (reactive subscriptions)

**What Benten Engine lacks:** Custom functions, custom analyzers, multiple index types, event triggers on data changes (the closest equivalent is reactive subscriptions, but those are read-only notifications, not interceptors).

### PostgreSQL (C, reference architecture)

PostgreSQL's extension system is the gold standard for database extensibility:
- Custom types (CREATE TYPE)
- Custom operators (CREATE OPERATOR)
- Custom index access methods (CREATE ACCESS METHOD)
- Custom aggregation functions (CREATE AGGREGATE)
- Custom procedural languages (CREATE LANGUAGE)
- Hooks for every stage of query processing (planner_hook, executor_start_hook, etc.)
- Foreign data wrappers (CREATE FOREIGN TABLE)
- Background workers for custom services

**What Benten Engine lacks:** Effectively all of these. The spec is closer to SQLite (embedded, limited extensibility) than PostgreSQL (extensible by design).

### Apache AGE (C, the system Benten replaces)

AGE extends PostgreSQL with:
- Custom graph functions (registered via CREATE FUNCTION)
- Custom operators for Cypher expressions
- Integration with PostgreSQL's extension ecosystem (so you get all PG extensions for free)

**What Benten Engine loses by replacing AGE:** The entire PostgreSQL extension ecosystem. Every PG extension (PostGIS, pg_trgm, pgvector, pg_cron, etc.) is no longer available. This is a massive composability regression unless the engine provides its own extension mechanism.

---

## 5. The "Thin Engine" Philosophy Question

The Thrum TypeScript codebase has a strong "thin engine" philosophy: `@benten/engine` provides mechanisms (registries, DAG, events, store interface), and domain modules provide policies. The Rust engine specification appears to abandon this -- it moves domain concerns (capability enforcement, version chains, CRDT sync) into the engine itself.

**This is defensible** for performance reasons (sub-0.01ms targets require these operations to be engine-native), but it creates a tension: the engine becomes thick with concerns that are not extensible.

Consider what happens when a domain module needs:
- A capability model different from UCAN (e.g., RBAC for backwards compatibility)
- A versioning strategy different from version chains (e.g., event sourcing with projections)
- A sync protocol different from CRDT (e.g., operational transform for real-time collaboration)

In the TypeScript system, these are all replaceable: swap the auth check, swap the composition pipeline, swap the event bus. In the Rust engine, these are baked in.

**Recommendation:** Apply the "thin engine" philosophy to Rust via traits:
- `benten-capability` defines a `CapabilityEnforcer` trait, with UCAN as the default implementation
- `benten-version` defines a `VersionStrategy` trait, with version chains as default
- `benten-sync` defines a `SyncProtocol` trait, with CRDT as default

The engine orchestrator composes these via dependency injection (the builder pattern recommended in Issue 6).

---

## 6. Priority Recommendations

### P0 (Must have before any implementation)

1. **Define a `PersistenceBackend` trait** in `benten-persist`. Make redb one implementation. This is the single most important composability decision.

2. **Define an `IndexProvider` trait** in `benten-graph`. Support custom index types. Without this, the engine cannot compete with PostgreSQL extensions or SurrealDB's multi-index support.

3. **Add write interception to the API.** Either via a `WriteInterceptor` trait or by making reactive subscriptions bidirectional (subscribe returns a capability to approve/reject/transform the change). Without this, the engine loses the pipeline/filter dispatch modes that are central to Thrum's module system.

4. **Specify the plugin loading model.** Answer: how does third-party code get into the engine? WASM? Compiled crate? Both? This affects every other design decision.

### P1 (Should have for first usable release)

5. **Define extension traits for all major subsystems** (capability, versioning, sync, view maintenance). Make the defaults excellent, but allow replacement.

6. **Specify the TypeScript module system bridge.** How does `defineModule()` in TypeScript translate to engine operations? How do lifecycle hooks work across the napi-rs boundary?

7. **Add custom CRDT merge strategies.** The per-field LWW default is correct for most properties, but text, counters, and sets need specialized merge.

### P2 (Important for ecosystem growth)

8. **Add custom Cypher functions.** Let modules register functions callable from Cypher queries and view definitions.

9. **Add custom aggregation functions** for IVM views.

10. **Design the WASM extension interface** for sandboxed module code. This is the path to "thousands of developers building modules" that does not require recompiling the engine.

---

## 7. Summary

The Benten Engine specification describes an ambitious and technically impressive runtime. The core data model (graph + IVM + version chains + CRDT sync) is sound and addresses real limitations of the current PostgreSQL+AGE stack. The performance targets are aggressive but achievable given the in-process architecture.

However, the specification is written as a self-contained system, not an extensible platform. It describes what the engine does but not how others extend it. Every major subsystem (persistence, indexing, view maintenance, sync, capability enforcement) is described in terms of its behavior, not its interfaces. There are no traits, no extension points, no plugin mechanisms.

This is the opposite of the philosophy that made the TypeScript Thrum system successful. The TypeScript codebase is built on interfaces: `Store` is an interface, `PermissionReader` is an interface, `EventBus` is an interface, `MaterializerStep` is an interface. The Rust spec needs the same discipline: define the interfaces first, then build the default implementations.

The single most impactful change would be to add a "Extensibility" section to the specification that defines traits for every major subsystem and describes the two-tier plugin model (compiled crates for core extensions, WASM modules for application extensions). Without this, the engine risks becoming a powerful but closed system that cannot support the "universal composable platform" vision.
